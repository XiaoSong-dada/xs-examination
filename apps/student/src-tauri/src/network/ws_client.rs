use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use futures_util::StreamExt;
use serde_json::json;
use tauri::{Emitter, Manager};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::Message;

use crate::network::protocol::{
    AnswerItem, AnswerSyncAckPayload, ExamStartPayload, HeartbeatPayload, MessageType,
    WsMessage, build_message, decode_value_message, encode_message,
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

        match crate::services::exam_runtime_service::ExamRuntimeService::flush_pending_answer_sync(
            &app_for_full_sync,
            200,
        )
        .await
        {
            Ok(count) => {
                if count > 0 {
                    println!("[ws-client] flushed pending answer sync count={}", count);
                }
            }
            Err(err) => {
                eprintln!("[ws-client] flush pending answer sync failed: {}", err);
            }
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

async fn send_full_answer_sync_for_current_session(app_handle: &tauri::AppHandle) -> Result<()> {
    let state = app_handle.state::<crate::state::AppState>();
    let Some(sender) = state.ws_sender() else {
        return Ok(());
    };

    let Some((session_id, exam_id, student_id, answers)) =
        crate::services::exam_runtime_service::ExamRuntimeService::get_current_session_answer_sync_bundle(
            app_handle,
        )
        .await?
    else {
        return Ok(());
    };

    if answers.is_empty() {
        return Ok(());
    }

    let payload_answers: Vec<AnswerItem> = answers
        .into_iter()
        .map(|item| AnswerItem {
            question_id: item.question_id,
            answer: item.answer,
            revision: Some(item.revision),
            answer_updated_at: Some(item.updated_at),
        })
        .collect();

    let batch_id = uuid::Uuid::new_v4().to_string();
    let message = build_answer_sync_message(
        &exam_id,
        &student_id,
        Some(&session_id),
        payload_answers,
        "full",
        Some(&batch_id),
    )?;

    if sender.send(message).is_err() {
        return Err(anyhow::anyhow!("全量答案同步发送失败：发送通道不可用"));
    }

    Ok(())
}

async fn handle_server_message(
    app_handle: tauri::AppHandle,
    local_student_id: &str,
    text: &str,
) -> Result<()> {
    let envelope: WsMessage<serde_json::Value> = decode_value_message(text)?;

    match envelope.r#type {
        MessageType::ExamStart => {
            let payload: ExamStartPayload = serde_json::from_value(envelope.payload)?;
            if payload.student_id != local_student_id {
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
                let _ = app_handle.emit(
                    "exam_status_changed",
                    json!({
                        "examId": payload.exam_id,
                        "studentId": payload.student_id,
                        "status": "active"
                    }),
                );
            }
        }
        MessageType::AnswerSyncAck => {
            let payload: AnswerSyncAckPayload = serde_json::from_value(envelope.payload)?;
            if !payload.success {
                eprintln!(
                    "[ws-client] answer sync ack failed exam_id={} student_id={} mode={:?} message={}",
                    payload.exam_id,
                    payload.student_id,
                    payload.sync_mode,
                    payload.message
                );
                return Ok(());
            }

            let synced = crate::services::exam_runtime_service::ExamRuntimeService::mark_answers_synced(
                &app_handle,
                &payload.exam_id,
                &payload.student_id,
                payload.session_id.as_deref(),
                &payload.question_ids,
                payload.acked_at,
            )
            .await?;

            println!(
                "[ws-client] answer sync acked exam_id={} student_id={} mode={:?} synced_count={}",
                payload.exam_id,
                payload.student_id,
                payload.sync_mode,
                synced
            );
        }
        _ => {
            println!("[ws-client] recv: {}", text);
        }
    }

    Ok(())
}
