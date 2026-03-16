use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tauri::Manager;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::network::protocol::{HeartbeatPayload, MessageType, WsMessage};

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
    }

    let app_for_writer = app_handle.clone();
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
    });

    tokio::spawn(async move {
        while let Some(next_message) = reader.next().await {
            match next_message {
                Ok(Message::Text(text)) => {
                    println!("[ws-client] recv: {}", text);
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
