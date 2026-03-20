use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tauri::{Emitter, Manager};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::network::protocol::{ExamStartPayload, HeartbeatPayload, MessageType, WsMessage};
use crate::schemas::teacher_endpoint_schema::WsConnectionEvent;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

fn unsigned_message<T>(message_type: MessageType, payload: T) -> WsMessage<T> {
    WsMessage {
        r#type: message_type,
        timestamp: now_ms(),
        signature: String::new(),
        payload,
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

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .with_context(|| format!("连接教师端失败: {}", ws_url))?;

    println!("[ws-client] connected to {}", ws_url);

    let (mut writer, mut reader) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

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

    let app_for_writer = app_handle.clone();
    let ws_url_for_writer = ws_url.clone();
    tokio::spawn(async move {
        while let Some(text) = rx.recv().await {
            if let Err(err) = writer.send(Message::Text(text.into())).await {
                eprintln!("[ws-client] send error: {}", err);
                break;
            }
        }

        let state = app_for_writer.state::<crate::state::AppState>();
        state.set_ws_connected(false);
        state.clear_ws_sender();
        state.clear_ws_endpoint();

        let _ = app_for_writer.emit(
            "ws_disconnected",
            WsConnectionEvent {
                endpoint: Some(ws_url_for_writer),
                connected: false,
                message: Some("连接已关闭".to_string()),
            },
        );
    });

    let app_for_reader = app_handle.clone();
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

    tokio::spawn(async move {
        loop {
            let heartbeat = unsigned_message(
                MessageType::Heartbeat,
                HeartbeatPayload {
                    student_id: student_id.clone(),
                },
            );

            match serde_json::to_string(&heartbeat) {
                Ok(text) => {
                    if heartbeat_tx.send(text).is_err() {
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
    question_id: &str,
    answer: &str,
) -> Result<String> {
    let payload = json!({
        "examId": exam_id,
        "studentId": student_id,
        "answers": [
            {
                "questionId": question_id,
                "answer": answer
            }
        ]
    });

    let message = unsigned_message(MessageType::AnswerSync, payload);
    Ok(serde_json::to_string(&message)?)
}

async fn handle_server_message(
    app_handle: tauri::AppHandle,
    local_student_id: &str,
    text: &str,
) -> Result<()> {
    let envelope: WsMessage<serde_json::Value> = serde_json::from_str(text)?;

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
        _ => {
            println!("[ws-client] recv: {}", text);
        }
    }

    Ok(())
}
