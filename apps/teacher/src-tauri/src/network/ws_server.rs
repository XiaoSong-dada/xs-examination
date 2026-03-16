use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use tauri::Manager;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::network::protocol::{MessageType, WsMessage};

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

fn ws_port() -> u16 {
    std::env::var("WS_SERVER_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(18765)
}

pub async fn start_ws_server(app_handle: tauri::AppHandle) -> Result<()> {
    let bind_addr = format!("0.0.0.0:{}", ws_port());
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
            let mut ws_stream = ws_stream;

            while let Some(next_message) = ws_stream.next().await {
                let message = match next_message {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("[ws-server] recv error from {}: {}", peer_addr, e);
                        break;
                    }
                };

                if let Message::Text(text) = message {
                    if let Err(e) = handle_text_message(&app_handle, &text) {
                        eprintln!("[ws-server] invalid message from {}: {}", peer_addr, e);
                    }
                }
            }

            println!("[ws-server] disconnected: {}", peer_addr);
        });
    }
}

fn handle_text_message(app_handle: &tauri::AppHandle, text: &str) -> Result<()> {
    let envelope: WsMessage<Value> = serde_json::from_str(text)?;
    match envelope.r#type {
        MessageType::Heartbeat => {
            let payload: HeartbeatPayload = serde_json::from_value(envelope.payload)?;
            let state = app_handle.state::<crate::state::AppState>();
            state.touch_connection(&payload.student_id, envelope.timestamp);
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
