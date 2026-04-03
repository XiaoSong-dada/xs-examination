use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use dashmap::DashMap;
use std::sync::{Arc, OnceLock};
use tokio::net::TcpStream;
use tauri::Emitter;
use tauri::Manager;

use crate::network::protocol::P2pDistributionProgressPayload;
use crate::network::transport::tcp_request_reply::{
    read_json_response, write_json_request,
};
use crate::schemas::control_protocol::{
    DistributeExamPaperPayload, P2pChunkAck, P2pRequestChunkPayload, P2pRequestChunkRequest,
};
use crate::utils::datetime::now_ms;
use crate::utils::p2p_chunker::{ChunkInfo, P2PChunker};

/// 对等设备信息
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub device_id: String,
    pub ip_addr: String,
    pub control_port: u16,
}

/// 会话的 P2P 下载状态
#[derive(Debug, Clone)]
struct SessionDownloadState {
    pub exam_id: String,
    pub total_chunks: usize,
    pub received_chunks: DashMap<usize, ChunkInfo>,
    pub peers: Vec<PeerInfo>,
    pub is_complete: bool,
    pub paper_payload: Option<DistributeExamPaperPayload>,
}

/// 全局 P2P 客户端状态
static P2P_CLIENT: OnceLock<Arc<P2PClient>> = OnceLock::new();

/// P2P 客户端结构体
pub struct P2PClient {
    /// 会话下载状态 (session_id -> SessionDownloadState)
    sessions: DashMap<String, SessionDownloadState>,
}

impl P2PClient {
    /// 获取或初始化全局 P2P 客户端
    pub fn get_or_init() -> &'static Arc<Self> {
        P2P_CLIENT.get_or_init(|| Arc::new(P2PClient {
            sessions: DashMap::new(),
        }))
    }

    /// 初始化一个新的试卷下载任务
    pub fn init_download_task(
        &self,
        session_id: String,
        exam_id: String,
        total_chunks: usize,
        peers: Vec<PeerInfo>,
        paper_payload: DistributeExamPaperPayload,
    ) {
        let state = SessionDownloadState {
            exam_id,
            total_chunks,
            received_chunks: DashMap::new(),
            peers,
            is_complete: false,
            paper_payload: Some(paper_payload),
        };
        
        self.sessions.insert(session_id, state);
        eprintln!("[p2p-client] initialized download task, total_chunks={}", total_chunks);
    }

    /// 获取下载进度
    pub fn get_download_progress(&self, session_id: &str) -> Option<(usize, usize)> {
        self.sessions.get(session_id).map(|state| {
            (state.received_chunks.len(), state.total_chunks)
        })
    }

    /// 开始下载试卷块
    pub async fn start_download(&self, session_id: &str, app_handle: tauri::AppHandle) -> Result<()> {
        let state = self.sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?
            .clone();

        eprintln!("[p2p-client] starting download for session_id={}", session_id);

        // 并行从所有对等方下载缺失的块
        let mut missing_indices: Vec<usize> = (0..state.total_chunks)
            .filter(|i| !state.received_chunks.contains_key(i))
            .collect();

        while !missing_indices.is_empty() && !state.is_complete {
            let mut handles = Vec::new();
            
            // 为每个缺失的块创建一个下载任务
            for &chunk_index in &missing_indices {
                let state_clone = state.clone();
                let session_id_clone = session_id.to_string();
                let client_clone = self.clone();
                let app_handle_clone = app_handle.clone();
                
                handles.push(tokio::spawn(async move {
                    client_clone.try_download_chunk(
                        &session_id_clone,
                        chunk_index,
                        state_clone,
                        app_handle_clone,
                    ).await
                }));
            }

            // 等待所有下载任务完成
            for handle in handles {
                let _ = handle.await;
            }

            // 更新缺失块列表
            missing_indices = (0..state.total_chunks)
                .filter(|i| !state.received_chunks.contains_key(i))
                .collect();

            // 上报当前进度
            self.report_progress(session_id, &app_handle).await?;

            // 如果没有缺失块了，组装试卷
            if missing_indices.is_empty() {
                self.assemble_and_save(session_id, &app_handle).await?;
            }
        }

        Ok(())
    }

    /// 尝试从对等方下载单个块
    async fn try_download_chunk(
        &self,
        session_id: &str,
        chunk_index: usize,
        state: SessionDownloadState,
        _app_handle: tauri::AppHandle,
    ) -> Result<()> {
        // 轮询所有对等方，尝试下载块
        for peer in &state.peers {
            match self.request_chunk_from_peer(peer, session_id, chunk_index).await {
                Ok(chunk_info) => {
                    // 验证并存储块
                    if P2PChunker::verify_hash(&chunk_info.data, &chunk_info.hash) {
                        state.received_chunks.insert(chunk_index, chunk_info);
                        eprintln!("[p2p-client] received chunk_index={} from peer={}", chunk_index, peer.device_id);
                        return Ok(());
                    } else {
                        eprintln!("[p2p-client] chunk hash verification failed for index={} from peer={}", chunk_index, peer.device_id);
                    }
                }
                Err(err) => {
                    eprintln!("[p2p-client] failed to download chunk_index={} from peer={}: {}", chunk_index, peer.device_id, err);
                }
            }
        }

        Err(anyhow!("Failed to download chunk_index={} from any peer", chunk_index))
    }

    /// 从指定对等方请求单个块
    async fn request_chunk_from_peer(
        &self,
        peer: &PeerInfo,
        session_id: &str,
        chunk_index: usize,
    ) -> Result<ChunkInfo> {
        let addr = format!("{}:{}", peer.ip_addr, peer.control_port);
        let mut stream = TcpStream::connect(&addr)
            .await
            .with_context(|| format!("Failed to connect to peer {}", addr))?;

        // 构建 P2P 块请求
        let request_id = uuid::Uuid::new_v4().to_string();
        let request = P2pRequestChunkRequest {
            r#type: "P2P_REQUEST_CHUNK".to_string(),
            request_id: request_id.clone(),
            timestamp: now_ms(),
            payload: P2pRequestChunkPayload {
                session_id: session_id.to_string(),
                chunk_index,
            },
        };

        // 发送请求并等待响应
        write_json_request(&mut stream, &request).await?;
        let response: P2pChunkAck = read_json_response(&mut stream).await?;

        if !response.payload.success {
            return Err(anyhow!("Peer returned error: {}", response.payload.message));
        }

        // 解码块数据
        let chunk_hash = response.payload.chunk_hash
            .ok_or_else(|| anyhow!("No chunk hash in response"))?;
        let chunk_data_b64 = response.payload.chunk_data
            .ok_or_else(|| anyhow!("No chunk data in response"))?;
        let chunk_data = general_purpose::STANDARD.decode(chunk_data_b64)?;

        Ok(ChunkInfo {
            index: chunk_index,
            hash: chunk_hash,
            data: chunk_data,
        })
    }



    /// 组装完整试卷并保存到数据库
    async fn assemble_and_save(&self, session_id: &str, app_handle: &tauri::AppHandle) -> Result<()> {
        let state = self.sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?
            .clone();

        // 收集所有块
        let mut chunks: Vec<ChunkInfo> = Vec::new();
        for i in 0..state.total_chunks {
            if let Some(chunk) = state.received_chunks.get(&i) {
                chunks.push(chunk.clone());
            }
        }

        // 组装数据
        let assembled_data = P2PChunker::reassemble_data(&chunks)
            .map_err(|e| anyhow!("Failed to reassemble data: {}", e))?;

        // 转换为字符串
        let questions_payload = String::from_utf8(assembled_data)
            .map_err(|e| anyhow!("Failed to convert data to UTF-8: {}", e))?;

        // 获取原始的 paper payload 并更新 questions_payload
        let mut paper_payload = state.paper_payload
            .ok_or_else(|| anyhow!("No paper payload found"))?;
        paper_payload.questions_payload = questions_payload;

        // 保存到数据库
        crate::services::exam_runtime_service::ExamRuntimeService::upsert_distribution(
            app_handle,
            &paper_payload,
        )
        .await
        .with_context(|| "Failed to save distribution to database")?;

        // 标记下载完成
        if let Some(mut state_mut) = self.sessions.get_mut(session_id) {
            state_mut.is_complete = true;
        }

        eprintln!("[p2p-client] successfully assembled and saved paper for session_id={}", session_id);

        Ok(())
    }

    /// 上报下载进度
    async fn report_progress(&self, session_id: &str, app_handle: &tauri::AppHandle) -> Result<()> {
        let state = self.sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?
            .clone();

        let (received, total) = (state.received_chunks.len(), state.total_chunks);
        let progress = if total > 0 {
            received as f64 / total as f64
        } else {
            0.0
        };

        eprintln!("[p2p-client] progress: {}/{} ({:.2}%)", received, total, progress * 100.0);

        // 发送进度事件到前端
        let _ = app_handle.emit(
            "p2p-download-progress",
            serde_json::json!({
                "sessionId": session_id,
                "received": received,
                "total": total,
                "progress": progress,
            }),
        );

        // 上报进度到教师端
        let app_state = app_handle.state::<crate::state::AppState>();
        if let Some(sender) = app_state.ws_sender() {
            let payload = P2pDistributionProgressPayload {
                exam_id: state.exam_id.clone(),
                device_id: "student-device".to_string(),
                received_blocks: received as u64,
                total_blocks: total as u64,
                progress,
                timestamp: now_ms(),
            };

            let message = crate::network::ws_client::build_p2p_progress_message(&payload)?;
            let _ = sender.send(message);
        }

        Ok(())
    }
}

impl Clone for P2PClient {
    fn clone(&self) -> Self {
        // 这是一个空实现，因为实际状态存储在全局 OnceLock 中
        // Arc 已经提供了引用计数的克隆
        P2PClient {
            sessions: DashMap::new(),
        }
    }
}
