use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use base64::Engine;
use futures_util::StreamExt;
use serde_json::json;
use tauri::{Emitter, Manager};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::Message;

use crate::network::protocol::{
    AnswerItem, AnswerSyncAckPayload, ExamEndPayload, ExamStartPayload,
    FinalSyncRequestPayload, HeartbeatPayload, MessageType, WsMessage, build_message,
    decode_value_message, encode_message, PaperPackageAckPayload,
    PaperPackageChunkPayload, PaperPackageManifestPayload,
};
use crate::network::transport::ws_transport::{connect_ws, new_text_channel, run_text_writer_loop};
use crate::schemas::teacher_endpoint_schema::WsConnectionEvent;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

fn cleanup_connection_state(app_handle: &tauri::AppHandle) {
    let state = app_handle.state::<crate::state::AppState>();
    state.set_ws_connected(false);
    state.clear_ws_sender();
    state.clear_ws_endpoint();
}

fn emit_disconnected(
    app_handle: &tauri::AppHandle,
    endpoint: String,
    message: impl Into<String>,
) {
    let _ = app_handle.emit(
        "ws_disconnected",
        WsConnectionEvent {
            endpoint: Some(endpoint),
            connected: false,
            message: Some(message.into()),
        },
    );
}

pub fn force_disconnect(app_handle: &tauri::AppHandle, message: &str) {
    let endpoint = {
        let state = app_handle.state::<crate::state::AppState>();
        state.ws_endpoint()
    };

    cleanup_connection_state(app_handle);

    if let Some(current_endpoint) = endpoint {
        emit_disconnected(app_handle, current_endpoint, message.to_string());
    }
}

pub async fn connect(
    app_handle: tauri::AppHandle,
    ws_url: String,
    student_id: String,
) -> Result<()> {
    {
        let state = app_handle.state::<crate::state::AppState>();
        if state.ws_connected() {
            let current_endpoint = state.ws_endpoint();
            if current_endpoint.as_deref() == Some(ws_url.as_str()) {
                println!("[ws-client] already connected: {}", ws_url);
                return Ok(());
            }

            return Err(anyhow::anyhow!(
                "已存在活动 WebSocket 连接: {}",
                current_endpoint.unwrap_or_else(|| "unknown".to_string())
            ));
        }
    }

    let ws_stream = connect_ws(&ws_url).await?;

    println!("[ws-client] connected to {}", ws_url);

    let (writer, mut reader) = ws_stream.split();
    let (tx, rx) = new_text_channel();

    {
        let state = app_handle.state::<crate::state::AppState>();
        state.set_ws_sender(tx.clone());
        state.set_ws_connected(true);
        state.set_ws_endpoint(ws_url.clone());
    }

    let _ = app_handle.emit(
        "ws_connected",
        WsConnectionEvent {
            endpoint: Some(ws_url.clone()),
            connected: true,
            message: None,
        },
    );

    let app_for_full_sync = app_handle.clone();
    tokio::spawn(async move {
        if let Err(err) = send_full_answer_sync_for_current_session(&app_for_full_sync).await {
            eprintln!("[ws-client] full answer sync after reconnect failed: {}", err);
        }
    });

    let app_for_outbox_flush = app_handle.clone();
    tokio::spawn(async move {
        loop {
            let connected = {
                let state = app_for_outbox_flush.state::<crate::state::AppState>();
                state.ws_connected()
            };

            if !connected {
                break;
            }

            match crate::services::exam_runtime_service::ExamRuntimeService::flush_pending_answer_sync(
                &app_for_outbox_flush,
                20,
            )
            .await
            {
                Ok(flushed) if flushed > 0 => {
                    println!("[ws-client] flushed pending answer sync count={}", flushed);
                }
                Ok(_) => {}
                Err(err) => {
                    eprintln!("[ws-client] flush pending answer sync failed: {}", err);
                }
            }

            sleep(Duration::from_secs(2)).await;
        }
    });

    let app_for_writer = app_handle.clone();
    let ws_url_for_writer = ws_url.clone();
    tokio::spawn(async move {
        if let Err(err) = run_text_writer_loop(writer, rx).await {
            eprintln!("[ws-client] send loop error: {}", err);
        }

        cleanup_connection_state(&app_for_writer);
        emit_disconnected(&app_for_writer, ws_url_for_writer, "连接已关闭");
    });

    let app_for_reader = app_handle.clone();
    let ws_url_for_reader = ws_url.clone();
    let student_id_for_reader = student_id.clone();
    tokio::spawn(async move {
        while let Some(next_message) = reader.next().await {
            match next_message {
                Ok(Message::Text(text)) => {
                    if let Err(err) = handle_server_message(
                        app_for_reader.clone(),
                        &student_id_for_reader,
                        &text,
                    )
                    .await
                    {
                        eprintln!("[ws-client] handle message failed: {}", err);
                    }
                }
                Ok(_) => {}
                Err(err) => {
                    eprintln!("[ws-client] recv error: {}", err);
                    cleanup_connection_state(&app_for_reader);
                    emit_disconnected(
                        &app_for_reader,
                        ws_url_for_reader.clone(),
                        format!("接收失败: {}", err),
                    );
                    break;
                }
            }
        }
    });

    let heartbeat_tx = {
        let state = app_handle.state::<crate::state::AppState>();
        state
            .ws_sender()
            .ok_or_else(|| anyhow::anyhow!("连接建立后未找到发送通道"))?
    };

    let app_for_heartbeat = app_handle.clone();
    let ws_url_for_heartbeat = ws_url.clone();
    tokio::spawn(async move {
        loop {
            let heartbeat = build_message(
                MessageType::Heartbeat,
                now_ms(),
                HeartbeatPayload {
                    student_id: student_id.clone(),
                },
            );

            match encode_message(&heartbeat) {
                Ok(text) => {
                    if heartbeat_tx.send(text).is_err() {
                        cleanup_connection_state(&app_for_heartbeat);
                        emit_disconnected(&app_for_heartbeat, ws_url_for_heartbeat.clone(), "心跳发送失败");
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("[ws-client] serialize heartbeat failed: {}", err);
                }
            }

            sleep(Duration::from_secs(5)).await;
        }
    });

    Ok(())
}

pub fn build_answer_sync_message(
    exam_id: &str,
    student_id: &str,
    session_id: Option<&str>,
    answers: Vec<AnswerItem>,
    sync_mode: &str,
    batch_id: Option<&str>,
) -> Result<String> {
    let payload = json!({
        "examId": exam_id,
        "studentId": student_id,
        "sessionId": session_id,
        "syncMode": sync_mode,
        "batchId": batch_id,
        "answers": answers,
    });

    let message = build_message(MessageType::AnswerSync, now_ms(), payload);
    encode_message(&message)
}

pub fn build_p2p_progress_message(
    payload: &crate::network::protocol::P2pDistributionProgressPayload,
) -> Result<String> {
    let message = build_message(MessageType::P2pDistributionProgress, now_ms(), payload);
    encode_message(&message)
}

async fn send_full_answer_sync_for_current_session(app_handle: &tauri::AppHandle) -> Result<()> {
    let batch_id = uuid::Uuid::new_v4().to_string();
    let _ = crate::services::exam_runtime_service::ExamRuntimeService::send_current_session_answer_sync(
        app_handle,
        "full",
        Some(&batch_id),
        false,
    )
    .await?;

    Ok(())
}

async fn handle_server_message(
    app_handle: tauri::AppHandle,
    local_student_id: &str,
    text: &str,
) -> Result<()> {
    let envelope: WsMessage<serde_json::Value> = decode_value_message(text)?;

    match envelope.r#type {
        MessageType::PaperPackageManifest => {
            let payload: PaperPackageManifestPayload = serde_json::from_value(envelope.payload)?;
            if payload.student_id != local_student_id {
                return Ok(());
            }

            let app_data_dir = app_handle.path().app_data_dir()?;
            let package_dir = app_data_dir
                .join("paper_packages")
                .join(&payload.session_id);
            std::fs::create_dir_all(&package_dir)?;
            let package_path = package_dir.join(&payload.file_name);
            if package_path.exists() {
                let _ = std::fs::remove_file(&package_path);
            }

            crate::services::exam_runtime_service::ExamRuntimeService::prepare_exam_package_receive(
                &app_handle,
                &payload,
                &package_path.to_string_lossy(),
            )
            .await?;

            app_handle.state::<crate::state::AppState>().set_receiving_package(
                payload.batch_id.clone(),
                crate::state::ReceivingPackageState {
                    exam_id: payload.exam_id,
                    student_id: payload.student_id,
                    session_id: payload.session_id,
                    batch_id: payload.batch_id,
                    file_path: package_path.to_string_lossy().to_string(),
                    sha256: payload.sha256,
                    total_bytes: payload.total_bytes,
                    total_chunks: payload.total_chunks,
                    received_chunks: 0,
                },
            );
        }
        MessageType::PaperPackageChunk => {
            let payload: PaperPackageChunkPayload = serde_json::from_value(envelope.payload)?;
            if payload.student_id != local_student_id {
                return Ok(());
            }

            let state = app_handle.state::<crate::state::AppState>();
            let Some(mut receiving) = state.get_receiving_package(&payload.batch_id) else {
                return Ok(());
            };

            let bytes = base64::engine::general_purpose::STANDARD
                .decode(payload.content_base64.as_bytes())?;
            if let Some(parent) = std::path::Path::new(&receiving.file_path).parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&receiving.file_path)?;
            use std::io::Write;
            file.write_all(&bytes)?;

            receiving.received_chunks = receiving.received_chunks.saturating_add(1);
            state.update_receiving_package(&payload.batch_id, |current| {
                current.received_chunks = receiving.received_chunks;
            });

            if payload.is_last || receiving.received_chunks >= receiving.total_chunks {
                let package_bytes = std::fs::read(&receiving.file_path)?;
                let mut hasher = sha2::Sha256::new();
                use sha2::Digest;
                hasher.update(&package_bytes);
                let actual_sha = format!("{:x}", hasher.finalize());
                let is_ok = actual_sha == receiving.sha256;

                if is_ok {
                    crate::services::exam_runtime_service::ExamRuntimeService::mark_exam_package_received(
                        &app_handle,
                        &receiving.session_id,
                        &receiving.batch_id,
                        &receiving.file_path,
                        &receiving.sha256,
                    )
                    .await?;
                }

                let ack_payload = PaperPackageAckPayload {
                    exam_id: receiving.exam_id.clone(),
                    student_id: receiving.student_id.clone(),
                    session_id: receiving.session_id.clone(),
                    batch_id: receiving.batch_id.clone(),
                    success: is_ok,
                    message: if is_ok {
                        "试卷包接收成功，等待开考解压".to_string()
                    } else {
                        "试卷包校验失败".to_string()
                    },
                    received_chunks: receiving.received_chunks,
                    total_chunks: receiving.total_chunks,
                    timestamp: now_ms(),
                };

                let ack = build_message(MessageType::PaperPackageAck, now_ms(), ack_payload);
                if let Ok(text) = encode_message(&ack) {
                    if let Some(sender) = app_handle.state::<crate::state::AppState>().ws_sender() {
                        let _ = sender.send(text);
                    }
                }

                state.remove_receiving_package(&payload.batch_id);
            }
        }
        MessageType::ExamStart => {
            let payload: ExamStartPayload = serde_json::from_value(envelope.payload)?;
            if payload.student_id != local_student_id {
                println!(
                    "[ws-client] ignore EXAM_START: payload_student_id={} local_student_id={} exam_id={}",
                    payload.student_id, local_student_id, payload.exam_id
                );
                return Ok(());
            }

            let updated = crate::services::exam_runtime_service::ExamRuntimeService::mark_exam_started(
                &app_handle,
                &payload.exam_id,
                &payload.student_id,
                payload.start_time,
                payload.end_time,
            )
            .await?;

            if updated {
                println!(
                    "[ws-client] EXAM_START applied: exam_id={} student_id={}",
                    payload.exam_id, payload.student_id
                );
                let _ = app_handle.emit(
                    "exam_status_changed",
                    json!({
                        "examId": payload.exam_id,
                        "studentId": payload.student_id,
                        "status": "active"
                    }),
                );
            } else {
                println!(
                    "[ws-client] EXAM_START ignored: no matching local session (exam_id={}, student_id={})",
                    payload.exam_id, payload.student_id
                );
            }
        }
        MessageType::FinalSyncRequest => {
            let payload: FinalSyncRequestPayload = serde_json::from_value(envelope.payload)?;
            if payload.student_id != local_student_id {
                return Ok(());
            }

            let sent = crate::services::exam_runtime_service::ExamRuntimeService::send_current_session_answer_sync(
                &app_handle,
                "final",
                Some(&payload.batch_id),
                true,
            )
            .await?;

            println!(
                "[ws-client] FINAL_SYNC_REQUEST handled exam_id={} student_id={} batch_id={} sent={}",
                payload.exam_id,
                payload.student_id,
                payload.batch_id,
                sent
            );
        }
        MessageType::ExamEnd => {
            let payload: ExamEndPayload = serde_json::from_value(envelope.payload)?;
            if payload.student_id != local_student_id {
                return Ok(());
            }

            let _ = crate::services::exam_runtime_service::ExamRuntimeService::send_current_session_answer_sync(
                &app_handle,
                "final",
                Some(&payload.final_batch_id),
                true,
            )
            .await?;

            let updated = crate::services::exam_runtime_service::ExamRuntimeService::mark_exam_ended(
                &app_handle,
                &payload.exam_id,
                &payload.student_id,
                payload.end_time,
            )
            .await?;

            if updated {
                let _ = app_handle.emit(
                    "exam_status_changed",
                    json!({
                        "examId": payload.exam_id,
                        "studentId": payload.student_id,
                        "status": "ended"
                    }),
                );
            }
        }
        MessageType::AnswerSyncAck => {
            let payload: AnswerSyncAckPayload = serde_json::from_value(envelope.payload)?;
            let synced = if payload.question_ids.is_empty() {
                0
            } else {
                crate::services::exam_runtime_service::ExamRuntimeService::mark_answers_synced(
                    &app_handle,
                    &payload.exam_id,
                    &payload.student_id,
                    payload.session_id.as_deref(),
                    &payload.question_ids,
                    payload.acked_at,
                )
                .await?
            };

            let failed = if payload.failed_question_ids.is_empty() {
                0
            } else {
                crate::services::exam_runtime_service::ExamRuntimeService::mark_answers_failed(
                    &app_handle,
                    &payload.exam_id,
                    &payload.student_id,
                    payload.session_id.as_deref(),
                    &payload.failed_question_ids,
                    payload.acked_at,
                    &payload.message,
                )
                .await?
            };

            if !payload.success || failed > 0 {
                eprintln!(
                    "[ws-client] answer sync ack partial/failed exam_id={} student_id={} mode={:?} ack_success={} ack_success_count={} ack_failed_count={} local_synced={} local_failed={} message={}",
                    payload.exam_id,
                    payload.student_id,
                    payload.sync_mode,
                    payload.success,
                    payload.success_count,
                    payload.failed_count,
                    synced,
                    failed,
                    payload.message
                );
                return Ok(());
            }

            println!(
                "[ws-client] answer sync acked exam_id={} student_id={} mode={:?} synced_count={} failed_count={}",
                payload.exam_id,
                payload.student_id,
                payload.sync_mode,
                synced,
                failed
            );
        }
        _ => {
            println!("[ws-client] recv: {}", text);
        }
    }

    Ok(())
}
