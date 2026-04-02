use crate::utils::datetime::now_ms;
use crate::utils::p2p_chunker::P2PChunker;

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use dashmap::DashMap;
use std::sync::OnceLock;
use tauri::Emitter;
use tokio::net::TcpStream;

use crate::config::AppConfig;
use crate::network::transport::tcp_request_reply::{
    bind_listener, read_json_request, write_json_response,
};
use crate::schemas::control_protocol::{
    ApplyTeacherEndpointsAck,
    ApplyTeacherEndpointsAckPayload,
    ApplyTeacherEndpointsRequest,
    DistributeExamPaperAck,
    DistributeExamPaperAckPayload,
    DistributeExamPaperRequest,
    P2pChunkAck,
    P2pChunkAckPayload,
    P2pRequestChunkRequest,
    P2pStatusAck,
    P2pStatusAckPayload,
    P2pStatusQueryRequest,
    P2pStoreChunkAck,
    P2pStoreChunkAckPayload,
    P2pStoreChunkRequest,
};
use crate::schemas::teacher_endpoint_schema::{
    TeacherEndpointAppliedEvent, WsConnectionEvent,
};
use crate::services::teacher_endpoints_service::TeacherEndpointsService;
use crate::services::ws_reconnect_service::WsReconnectService;

// ========== P2P 本地块缓存和状态管理 ==========

/// 单个块的数据结构
#[derive(Debug, Clone)]
struct CachedChunk {
    pub hash: String,
    pub data: Vec<u8>,
}

/// 会话的块缓存
#[derive(Debug, Clone)]
struct SessionChunkCache {
    pub chunks: DashMap<usize, CachedChunk>,
    pub total_chunks: Option<usize>,
}

/// 全局块缓存管理器
static CHUNK_CACHE: OnceLock<DashMap<String, SessionChunkCache>> = OnceLock::new();

/// 获取或初始化全局块缓存
fn get_chunk_cache() -> &'static DashMap<String, SessionChunkCache> {
    CHUNK_CACHE.get_or_init(|| DashMap::new())
}

/// 为指定会话初始化块缓存
fn init_session_cache(session_id: &str, total_chunks: Option<usize>) {
    let cache = get_chunk_cache();
    cache.entry(session_id.to_string()).or_insert_with(|| SessionChunkCache {
        chunks: DashMap::new(),
        total_chunks,
    });
}

/// 存储块到本地缓存
fn store_chunk(session_id: &str, chunk_index: usize, hash: &str, data: Vec<u8>) -> Result<()> {
    let cache = get_chunk_cache();
    let session_cache = cache.entry(session_id.to_string()).or_insert_with(|| SessionChunkCache {
        chunks: DashMap::new(),
        total_chunks: None,
    });
    
    session_cache.chunks.insert(chunk_index, CachedChunk {
        hash: hash.to_string(),
        data,
    });
    
    Ok(())
}

/// 从本地缓存获取块
fn get_chunk(session_id: &str, chunk_index: usize) -> Option<CachedChunk> {
    let cache = get_chunk_cache();
    cache.get(session_id).and_then(|session_cache| {
        session_cache.chunks.get(&chunk_index).map(|chunk| chunk.clone())
    })
}

/// 获取会话的可用块索引列表
fn get_available_chunks(session_id: &str) -> Vec<usize> {
    let cache = get_chunk_cache();
    cache.get(session_id).map(|session_cache| {
        session_cache.chunks.iter().map(|entry| *entry.key()).collect()
    }).unwrap_or_else(Vec::new)
}

/// 检查会话是否有完整试卷
fn has_paper(session_id: &str) -> bool {
    let cache = get_chunk_cache();
    cache.get(session_id).map_or(false, |session_cache| {
        if let Some(total) = session_cache.total_chunks {
            session_cache.chunks.len() == total
        } else {
            !session_cache.chunks.is_empty()
        }
    })
}



pub async fn start(app_handle: tauri::AppHandle) -> Result<()> {
    let config = AppConfig::load()?;
    let bind_addr = format!("0.0.0.0:{}", config.control_port);
    let listener = bind_listener(&bind_addr)
        .await
        .with_context(|| format!("学生端控制服务启动失败: {}", bind_addr))?;

    println!("[control-server] listening on {}", bind_addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let app_handle = app_handle.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(app_handle, stream).await {
                eprintln!("[control-server] handle client {} failed: {}", peer_addr, err);
            }
        });
    }
}

async fn handle_client(app_handle: tauri::AppHandle, mut stream: TcpStream) -> Result<()> {
    // 发卷报文包含完整题目集合，沿用浅封装中的大小限制。
    let raw = read_json_request(&mut stream, 10 * 1024 * 1024).await?;
    let req_type = raw
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    if req_type == "DISTRIBUTE_EXAM_PAPER" {
        let req: DistributeExamPaperRequest = serde_json::from_value(raw)?;
        eprintln!(
            "[control-server] receive DISTRIBUTE_EXAM_PAPER request_id={} session_id={} questions_size={}",
            req.request_id,
            req.payload.session_id,
            req.payload.questions_payload.len()
        );

        let result = crate::services::exam_runtime_service::ExamRuntimeService::upsert_distribution(
            &app_handle,
            &req.payload,
        )
        .await;

        // 将试卷分块并缓存到本地，用于 P2P 分发
        if result.is_ok() {
            let session_id = req.payload.session_id.clone();
            let paper_data = req.payload.questions_payload.as_bytes();
            let chunker = P2PChunker::new();
            let chunks = chunker.split_data(paper_data);
            let total_chunks = chunks.len();
            
            // 初始化会话缓存
            init_session_cache(&session_id, Some(total_chunks));
            
            // 存储所有块
            for chunk in chunks {
                let _ = store_chunk(&session_id, chunk.index, &chunk.hash, chunk.data);
            }
            
            eprintln!(
                "[control-server] paper cached for P2P, session_id={}, total_chunks={}",
                session_id, total_chunks
            );
        }

        let (success, message) = match result {
            Ok(()) => (true, "试卷已落库".to_string()),
            Err(err) => {
                eprintln!("[control-server] distribute persist failed: {}", err);
                (false, format!("试卷落库失败: {}", err))
            }
        };

        let ack = DistributeExamPaperAck {
            r#type: "DISTRIBUTE_EXAM_PAPER_ACK".to_string(),
            request_id: req.request_id,
            timestamp: now_ms(),
            payload: DistributeExamPaperAckPayload {
                success,
                message,
                session_id: Some(req.payload.session_id),
            },
        };

        write_json_response(&mut stream, &ack).await?;
        return Ok(());
    }

    // ========== P2P 请求处理分支 ==========

    if req_type == "P2P_REQUEST_CHUNK" {
        let req: P2pRequestChunkRequest = serde_json::from_value(raw)?;
        eprintln!(
            "[control-server] receive P2P_REQUEST_CHUNK request_id={} session_id={} chunk_index={}",
            req.request_id,
            req.payload.session_id,
            req.payload.chunk_index
        );

        let (success, message, chunk_hash, chunk_data) = match get_chunk(&req.payload.session_id, req.payload.chunk_index) {
            Some(chunk) => {
                let encoded_data = general_purpose::STANDARD.encode(&chunk.data);
                (true, "Chunk found".to_string(), Some(chunk.hash), Some(encoded_data))
            }
            None => (false, "Chunk not found".to_string(), None, None),
        };

        let ack = P2pChunkAck {
            r#type: "P2P_CHUNK_ACK".to_string(),
            request_id: req.request_id,
            timestamp: now_ms(),
            payload: P2pChunkAckPayload {
                success,
                message,
                session_id: req.payload.session_id,
                chunk_index: req.payload.chunk_index,
                chunk_hash,
                chunk_data,
            },
        };

        write_json_response(&mut stream, &ack).await?;
        return Ok(());
    }

    if req_type == "P2P_STATUS_QUERY" {
        let req: P2pStatusQueryRequest = serde_json::from_value(raw)?;
        eprintln!(
            "[control-server] receive P2P_STATUS_QUERY request_id={} session_id={}",
            req.request_id,
            req.payload.session_id
        );

        let has_paper = has_paper(&req.payload.session_id);
        let available_chunks = get_available_chunks(&req.payload.session_id);
        let (total_chunks, success, message) = if has_paper || !available_chunks.is_empty() {
            let cache = get_chunk_cache();
            let total = cache.get(&req.payload.session_id).and_then(|s| s.total_chunks);
            (total, true, "Status retrieved".to_string())
        } else {
            (None, true, "No paper available".to_string())
        };

        let ack = P2pStatusAck {
            r#type: "P2P_STATUS_ACK".to_string(),
            request_id: req.request_id,
            timestamp: now_ms(),
            payload: P2pStatusAckPayload {
                success,
                message,
                session_id: req.payload.session_id,
                has_paper,
                total_chunks,
                available_chunks: if available_chunks.is_empty() { None } else { Some(available_chunks) },
            },
        };

        write_json_response(&mut stream, &ack).await?;
        return Ok(());
    }

    if req_type == "P2P_STORE_CHUNK" {
        let req: P2pStoreChunkRequest = serde_json::from_value(raw)?;
        eprintln!(
            "[control-server] receive P2P_STORE_CHUNK request_id={} session_id={} chunk_index={}",
            req.request_id,
            req.payload.session_id,
            req.payload.chunk_index
        );

        let (success, message) = match general_purpose::STANDARD.decode(&req.payload.chunk_data) {
            Ok(decoded_data) => {
                // 验证哈希
                let computed_hash = P2PChunker::compute_hash(&decoded_data);
                if computed_hash != req.payload.chunk_hash {
                    (false, "Hash verification failed".to_string())
                } else {
                    match store_chunk(&req.payload.session_id, req.payload.chunk_index, &req.payload.chunk_hash, decoded_data) {
                        Ok(_) => (true, "Chunk stored".to_string()),
                        Err(err) => (false, format!("Failed to store chunk: {}", err)),
                    }
                }
            }
            Err(err) => (false, format!("Failed to decode chunk data: {}", err)),
        };

        let ack = P2pStoreChunkAck {
            r#type: "P2P_STORE_CHUNK_ACK".to_string(),
            request_id: req.request_id,
            timestamp: now_ms(),
            payload: P2pStoreChunkAckPayload {
                success,
                message,
                session_id: req.payload.session_id,
                chunk_index: req.payload.chunk_index,
            },
        };

        write_json_response(&mut stream, &ack).await?;
        return Ok(());
    }

    let req: ApplyTeacherEndpointsRequest = serde_json::from_value(raw)?;

    if req.r#type != "APPLY_TEACHER_ENDPOINTS" {
        let ack = ApplyTeacherEndpointsAck {
            r#type: "APPLY_TEACHER_ENDPOINTS_ACK".to_string(),
            request_id: req.request_id,
            timestamp: now_ms(),
            payload: ApplyTeacherEndpointsAckPayload {
                success: false,
                message: "不支持的消息类型".to_string(),
                connected_master: None,
            },
        };
        write_json_response(&mut stream, &ack).await?;
        return Ok(());
    }

    let (success, message) = match TeacherEndpointsService::replace_all(&app_handle, &req.payload.endpoints).await {
        Ok(()) => {
            match crate::services::exam_runtime_service::ExamRuntimeService::upsert_connected_session(
                &app_handle,
                &req.payload,
            )
            .await
            {
                Ok(true) => (true, "配置与考生会话已落库".to_string()),
                Ok(false) => (true, "配置已落库（未携带会话信息）".to_string()),
                Err(err) => (false, format!("会话落库失败: {}", err)),
            }
        }
        Err(err) => (false, format!("配置落库失败: {}", err)),
    };

    let connected_master = if success {
        TeacherEndpointsService::master_endpoint(&req.payload.endpoints)
    } else {
        None
    };

    if success {
        let _ = app_handle.emit(
            "teacher_endpoint_applied",
            TeacherEndpointAppliedEvent {
                endpoint: connected_master.clone(),
            },
        );
    }

    if success {
        if let Some(master_url) = &connected_master {
            let connect_result = WsReconnectService::start_or_update(
                app_handle.clone(),
                master_url.clone(),
                req.payload.student_id.clone(),
            )
            .await;

            if let Err(err) = connect_result {
                let _ = app_handle.emit(
                    "ws_disconnected",
                    WsConnectionEvent {
                        endpoint: Some(master_url.clone()),
                        connected: false,
                        message: Some(err.to_string()),
                    },
                );
            }
        }
    }

    let ack = ApplyTeacherEndpointsAck {
        r#type: "APPLY_TEACHER_ENDPOINTS_ACK".to_string(),
        request_id: req.request_id,
        timestamp: now_ms(),
        payload: ApplyTeacherEndpointsAckPayload {
            success,
            message,
            connected_master,
        },
    };

    write_json_response(&mut stream, &ack).await?;
    Ok(())
}
