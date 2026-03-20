use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use tauri::Manager;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::core::setting::SETTINGS;
use crate::network::protocol::{ExamStartPayload, MessageType, WsMessage};

#[derive(Debug, Clone, Deserialize)]
struct AnswerItem {
    #[serde(rename = "questionId")]
    question_id: String,
    answer: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AnswerSyncPayload {
    #[serde(rename = "examId")]
    exam_id: String,
    #[serde(rename = "studentId")]
    student_id: String,
    answers: Vec<AnswerItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct HeartbeatPayload {
    #[serde(rename = "studentId")]
    student_id: String,
}

pub async fn start_ws_server(app_handle: tauri::AppHandle) -> Result<()> {
    let bind_addr = format!("0.0.0.0:{}", SETTINGS.ws_server_port);
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("WebSocket 服务监听失败: {}", bind_addr))?;

    println!("[ws-server] listening on {}", bind_addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let app_handle = app_handle.clone();

        tokio::spawn(async move {
            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("[ws-server] handshake failed from {}: {}", peer_addr, e);
                    return;
                }
            };

            println!("[ws-server] connected: {}", peer_addr);
            let (mut writer, mut reader) = ws_stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<String>();
            let peer_id = uuid::Uuid::new_v4().to_string();

            {
                let state = app_handle.state::<crate::state::AppState>();
                state.register_ws_peer(peer_id.clone(), tx);
            }

            let writer_peer_id = peer_id.clone();
            let writer_app = app_handle.clone();
            let writer_task = tokio::spawn(async move {
                while let Some(text) = rx.recv().await {
                    if writer.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }

                let state = writer_app.state::<crate::state::AppState>();
                state.remove_ws_peer(&writer_peer_id);
            });

            while let Some(next_message) = reader.next().await {
                let message = match next_message {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("[ws-server] recv error from {}: {}", peer_addr, e);
                        break;
                    }
                };

                if let Message::Text(text) = message {
                    if let Err(e) = handle_text_message(&app_handle, &peer_id, &text) {
                        eprintln!("[ws-server] invalid message from {}: {}", peer_addr, e);
                    }
                }
            }

            writer_task.abort();
            {
                let state = app_handle.state::<crate::state::AppState>();
                state.remove_ws_peer(&peer_id);
            }

            println!("[ws-server] disconnected: {}", peer_addr);
        });
    }
}

pub fn send_exam_start_to_student(
    app_handle: &tauri::AppHandle,
    payload: ExamStartPayload,
) -> Result<bool> {
    let envelope = WsMessage {
        r#type: MessageType::ExamStart,
        timestamp: payload.timestamp,
        signature: String::new(),
        payload,
    };
    let text = serde_json::to_string(&envelope)?;
    let state = app_handle.state::<crate::state::AppState>();
    Ok(state.send_ws_text_to_student(&envelope.payload.student_id, text))
}

fn handle_text_message(app_handle: &tauri::AppHandle, peer_id: &str, text: &str) -> Result<()> {
    let envelope: WsMessage<Value> = serde_json::from_str(text)?;
    match envelope.r#type {
        MessageType::Heartbeat => {
            let payload: HeartbeatPayload = serde_json::from_value(envelope.payload)?;
            let state = app_handle.state::<crate::state::AppState>();
            state.touch_connection(&payload.student_id, envelope.timestamp);
            state.bind_student_peer(&payload.student_id, peer_id);
            println!(
                "[ws-server] heartbeat student_id={} ts={}",
                payload.student_id, envelope.timestamp
            );
        }
        MessageType::AnswerSync => {
            let payload: AnswerSyncPayload = serde_json::from_value(envelope.payload)?;
            let answer_count = payload.answers.len();
            let state = app_handle.state::<crate::state::AppState>();
            state.touch_connection(&payload.student_id, envelope.timestamp);
            state.bind_student_peer(&payload.student_id, peer_id);

            if let Some(first) = payload.answers.first() {
                println!(
                    "[ws-server] answer_sync exam_id={} student_id={} count={} first_q={} first_answer={}",
                    payload.exam_id,
                    payload.student_id,
                    answer_count,
                    first.question_id,
                    first.answer
                );
            } else {
                println!(
                    "[ws-server] answer_sync exam_id={} student_id={} count=0",
                    payload.exam_id, payload.student_id
                );
            }
        }
        other => {
            println!("[ws-server] received message type: {:?}", other);
        }
    }

    Ok(())
}
