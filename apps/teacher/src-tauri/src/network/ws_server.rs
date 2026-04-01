use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use sea_orm::ConnectionTrait;
use sea_orm::sea_query::{Alias, Expr, Query};
use tauri::Manager;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

use crate::core::setting::SETTINGS;
use crate::network::protocol::{
    AnswerSyncAckPayload, AnswerSyncPayload, ExamEndPayload, ExamStartPayload,
    FinalSyncRequestPayload, MessageType, WsMessage, build_message, decode_value_message,
    encode_message, PaperPackageAckPayload, PaperPackageChunkPayload,
    PaperPackageManifestPayload,
};
use crate::network::transport::ws_transport::{
    accept_ws, new_text_channel, run_text_writer_loop,
};

#[derive(Debug, Clone, Deserialize)]
struct HeartbeatPayload {
    #[serde(rename = "studentId")]
    student_id: String,
}

struct PersistAnswerSyncResult {
    success_question_ids: Vec<String>,
    failed_question_ids: Vec<String>,
    first_error: Option<String>,
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
            let ws_stream = match accept_ws(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("[ws-server] handshake failed from {}: {}", peer_addr, e);
                    return;
                }
            };

            println!("[ws-server] connected: {}", peer_addr);
            let (writer, mut reader) = ws_stream.split();
            let (tx, rx) = new_text_channel();
            let peer_id = uuid::Uuid::new_v4().to_string();

            {
                let state = app_handle.state::<crate::state::AppState>();
                state.register_ws_peer(peer_id.clone(), tx);
            }

            let writer_peer_id = peer_id.clone();
            let writer_app = app_handle.clone();
            let writer_task = tokio::spawn(async move {
                if let Err(err) = run_text_writer_loop(writer, rx).await {
                    eprintln!("[ws-server] send loop error: {}", err);
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
                    if let Err(e) = handle_text_message(&app_handle, &peer_id, &text).await {
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
    let envelope = build_message(MessageType::ExamStart, payload.timestamp, payload);
    let text = encode_message(&envelope)?;
    let state = app_handle.state::<crate::state::AppState>();
    Ok(state.send_ws_text_to_student(&envelope.payload.student_id, text))
}

pub fn send_final_sync_request_to_student(
    app_handle: &tauri::AppHandle,
    payload: FinalSyncRequestPayload,
) -> Result<bool> {
    let envelope = build_message(MessageType::FinalSyncRequest, payload.timestamp, payload);
    let text = encode_message(&envelope)?;
    let state = app_handle.state::<crate::state::AppState>();
    Ok(state.send_ws_text_to_student(&envelope.payload.student_id, text))
}

pub fn send_exam_end_to_student(
    app_handle: &tauri::AppHandle,
    payload: ExamEndPayload,
) -> Result<bool> {
    let envelope = build_message(MessageType::ExamEnd, payload.timestamp, payload);
    let text = encode_message(&envelope)?;
    let state = app_handle.state::<crate::state::AppState>();
    Ok(state.send_ws_text_to_student(&envelope.payload.student_id, text))
}

pub fn send_paper_package_manifest_to_student(
    app_handle: &tauri::AppHandle,
    payload: PaperPackageManifestPayload,
) -> Result<bool> {
    let envelope = build_message(MessageType::PaperPackageManifest, payload.timestamp, payload);
    let text = encode_message(&envelope)?;
    let state = app_handle.state::<crate::state::AppState>();
    Ok(state.send_ws_text_to_student(&envelope.payload.student_id, text))
}

pub fn send_paper_package_chunk_to_student(
    app_handle: &tauri::AppHandle,
    payload: PaperPackageChunkPayload,
) -> Result<bool> {
    let envelope = build_message(MessageType::PaperPackageChunk, payload.timestamp, payload);
    let text = encode_message(&envelope)?;
    let state = app_handle.state::<crate::state::AppState>();
    Ok(state.send_ws_text_to_student(&envelope.payload.student_id, text))
}

async fn handle_text_message(app_handle: &tauri::AppHandle, peer_id: &str, text: &str) -> Result<()> {
    let envelope: WsMessage<serde_json::Value> = decode_value_message(text)?;
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

            let persist_result = persist_answer_sync(app_handle, &payload, envelope.timestamp).await?;
            let success_count = persist_result.success_question_ids.len() as i64;
            let failed_count = persist_result.failed_question_ids.len() as i64;
            let success = failed_count == 0;

            if success
                && payload.sync_mode.as_deref() == Some("final")
                && payload.batch_id.as_deref().map(|v| !v.trim().is_empty()).unwrap_or(false)
            {
                if let Some(batch_id) = payload.batch_id.as_deref() {
                    state.mark_final_sync_received(batch_id);
                }
            }

            let message = if failed_count == 0 {
                format!("答案已落库（{}/{}）", success_count, answer_count)
            } else {
                let reason = persist_result
                    .first_error
                    .unwrap_or_else(|| "部分题目落库失败".to_string());
                format!(
                    "答案部分落库（成功 {}/{}，失败 {}）：{}",
                    success_count, answer_count, failed_count, reason
                )
            };

            send_answer_sync_ack(
                app_handle,
                peer_id,
                &payload,
                envelope.timestamp,
                success,
                message,
                persist_result.success_question_ids,
                persist_result.failed_question_ids,
            )?;

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
        MessageType::PaperPackageAck => {
            let payload: PaperPackageAckPayload = serde_json::from_value(envelope.payload)?;
            let state = app_handle.state::<crate::state::AppState>();
            state.mark_paper_package_ack(
                &payload.batch_id,
                format!(
                    "{}|{}|{}/{}",
                    if payload.success { "ok" } else { "fail" },
                    payload.message,
                    payload.received_chunks,
                    payload.total_chunks
                ),
            );
            state.touch_connection(&payload.student_id, envelope.timestamp);
            state.bind_student_peer(&payload.student_id, peer_id);
        }
        other => {
            println!("[ws-server] received message type: {:?}", other);
        }
    }

    Ok(())
}

async fn persist_answer_sync(
    app_handle: &tauri::AppHandle,
    payload: &AnswerSyncPayload,
    received_at: i64,
) -> Result<PersistAnswerSyncResult> {
    let state = app_handle.state::<crate::state::AppState>();

    let exam_alias = Alias::new("exam");
    let exam_status_query = Query::select()
        .expr_as(
            Expr::col((exam_alias.clone(), Alias::new("status"))),
            Alias::new("status"),
        )
        .from_as(Alias::new("exams"), exam_alias)
        .and_where(Expr::cust(format!(
            "id = '{}'",
            sql_escape(&payload.exam_id)
        )))
        .limit(1)
        .to_owned();

    let exam_status: Option<String> = state
        .db
        .query_one(&exam_status_query)
        .await?
        .and_then(|row| row.try_get("", "status").ok());

    if exam_status.as_deref() == Some("finished") {
        return Ok(PersistAnswerSyncResult {
            success_question_ids: Vec::new(),
            failed_question_ids: payload
                .answers
                .iter()
                .map(|item| item.question_id.clone())
                .collect(),
            first_error: Some("考试已结束，教师端拒收答案同步".to_string()),
        });
    }

    let se = Alias::new("se");
    let mut student_exam_id: Option<String> = None;
    let mut resolved_student_id: Option<String> = None;

    if let Some(session_id) = payload.session_id.as_deref() {
        if !session_id.trim().is_empty() {
            let student_exam_by_session_query = Query::select()
                .expr_as(Expr::col((se.clone(), Alias::new("id"))), Alias::new("id"))
                .expr_as(
                    Expr::col((se.clone(), Alias::new("student_id"))),
                    Alias::new("student_id"),
                )
                .from_as(Alias::new("student_exams"), se.clone())
                .and_where(Expr::cust(format!("id = '{}'", sql_escape(session_id))))
                .and_where(Expr::cust(format!(
                    "exam_id = '{}'",
                    sql_escape(&payload.exam_id)
                )))
                .limit(1)
                .to_owned();

            if let Some(row) = state.db.query_one(&student_exam_by_session_query).await? {
                student_exam_id = Some(row.try_get("", "id")?);
                resolved_student_id = Some(row.try_get("", "student_id")?);
            }
        }
    }

    if student_exam_id.is_none() {
        let student_exam_query = Query::select()
            .expr_as(Expr::col((se.clone(), Alias::new("id"))), Alias::new("id"))
            .expr_as(
                Expr::col((se.clone(), Alias::new("student_id"))),
                Alias::new("student_id"),
            )
            .from_as(Alias::new("student_exams"), se.clone())
            .and_where(Expr::cust(format!(
                "exam_id = '{}'",
                sql_escape(&payload.exam_id)
            )))
            .and_where(Expr::cust(format!(
                "student_id = '{}'",
                sql_escape(&payload.student_id)
            )))
            .limit(1)
            .to_owned();

        if let Some(row) = state.db.query_one(&student_exam_query).await? {
            student_exam_id = Some(row.try_get("", "id")?);
            resolved_student_id = Some(row.try_get("", "student_id")?);
        }
    }

    let student_exam_id = student_exam_id.ok_or_else(|| {
        anyhow::anyhow!(
            "未找到 student_exams 记录（exam_id={}, student_id={}, session_id={:?}）",
            payload.exam_id,
            payload.student_id,
            payload.session_id
        )
    })?;
    let resolved_student_id = resolved_student_id.unwrap_or_else(|| payload.student_id.clone());

    if resolved_student_id != payload.student_id {
        println!(
            "[ws-server] answer sync student_id remapped payload_student_id={} resolved_student_id={} session_id={:?}",
            payload.student_id,
            resolved_student_id,
            payload.session_id
        );
    }

    let q = Alias::new("q");
    let total_questions_query = Query::select()
        .expr_as(Expr::cust("COUNT(1)"), Alias::new("total_questions"))
        .from_as(Alias::new("questions"), q.clone())
        .and_where(Expr::cust(format!(
            "exam_id = '{}'",
            sql_escape(&payload.exam_id)
        )))
        .to_owned();
    let total_questions = match state.db.query_one(&total_questions_query).await? {
        Some(row) => row.try_get("", "total_questions").unwrap_or(0),
        None => 0,
    };

    let mut success_question_ids: Vec<String> = Vec::new();
    let mut failed_question_ids: Vec<String> = Vec::new();
    let mut first_error: Option<String> = None;

    for item in &payload.answers {
        let revision = std::cmp::max(item.revision.unwrap_or(1), 1);
        let answer_updated_at = item.answer_updated_at.unwrap_or(0);

        let escaped_id = sql_escape(&uuid::Uuid::new_v4().to_string());
        let escaped_student_exam_id = sql_escape(&student_exam_id);
        let escaped_student_id = sql_escape(&resolved_student_id);
        let escaped_exam_id = sql_escape(&payload.exam_id);
        let escaped_question_id = sql_escape(&item.question_id);
        let escaped_answer = sql_escape(&item.answer);

        let upsert_sql = format!(
            "INSERT INTO answer_sheets (id, student_exam_id, student_id, exam_id, question_id, answer, revision, answer_updated_at, received_at, synced_at) \
             VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {}, {}) \
                         ON CONFLICT(student_exam_id, question_id) DO UPDATE SET \
               student_exam_id = excluded.student_exam_id, \
                             student_id = excluded.student_id, \
               exam_id = excluded.exam_id, \
                             answer = CASE \
                                 WHEN excluded.revision > COALESCE(answer_sheets.revision, 0) THEN excluded.answer \
                                 WHEN excluded.revision = COALESCE(answer_sheets.revision, 0) \
                                     AND excluded.answer_updated_at > COALESCE(answer_sheets.answer_updated_at, 0) THEN excluded.answer \
                                 ELSE answer_sheets.answer END, \
                             revision = CASE \
                                 WHEN excluded.revision > COALESCE(answer_sheets.revision, 0) THEN excluded.revision \
                                 ELSE answer_sheets.revision END, \
                             answer_updated_at = CASE \
                                 WHEN excluded.revision > COALESCE(answer_sheets.revision, 0) THEN excluded.answer_updated_at \
                                 WHEN excluded.revision = COALESCE(answer_sheets.revision, 0) \
                                     AND excluded.answer_updated_at > COALESCE(answer_sheets.answer_updated_at, 0) THEN excluded.answer_updated_at \
                                 ELSE answer_sheets.answer_updated_at END, \
               received_at = excluded.received_at, \
               synced_at = excluded.synced_at",
            escaped_id,
            escaped_student_exam_id,
            escaped_student_id,
            escaped_exam_id,
            escaped_question_id,
            escaped_answer,
            revision,
            answer_updated_at,
            received_at,
            received_at,
        );

        match state.db.execute_unprepared(&upsert_sql).await {
            Ok(_) => {
                success_question_ids.push(item.question_id.clone());
            }
            Err(err) => {
                if first_error.is_none() {
                    first_error = Some(err.to_string());
                }
                failed_question_ids.push(item.question_id.clone());
                eprintln!(
                    "[ws-server] answer upsert failed exam_id={} student_exam_id={} question_id={} err={}",
                    payload.exam_id,
                    student_exam_id,
                    item.question_id,
                    err
                );
                continue;
            }
        }

        let verify_alias = Alias::new("ans_verify");
        let verify_query = Query::select()
            .expr_as(
                Expr::col((verify_alias.clone(), Alias::new("answer"))),
                Alias::new("answer"),
            )
            .expr_as(
                Expr::col((verify_alias.clone(), Alias::new("revision"))),
                Alias::new("revision"),
            )
            .from_as(Alias::new("answer_sheets"), verify_alias.clone())
            .and_where(Expr::cust(format!(
                "student_exam_id = '{}'",
                sql_escape(&student_exam_id)
            )))
            .and_where(Expr::cust(format!(
                "question_id = '{}'",
                sql_escape(&item.question_id)
            )))
            .limit(1)
            .to_owned();

        if let Some(row) = state.db.query_one(&verify_query).await? {
            let persisted_answer: Option<String> = row.try_get("", "answer").ok();
            let persisted_revision: i64 = row.try_get("", "revision").unwrap_or(0);

            println!(
                "[ws-server] answer_sync upsert verify exam_id={} student_exam_id={} question_id={} payload_answer={} persisted_answer={:?} payload_revision={} persisted_revision={}",
                payload.exam_id,
                student_exam_id,
                item.question_id,
                item.answer,
                persisted_answer,
                revision,
                persisted_revision
            );
        }
    }

    if success_question_ids.is_empty() {
        return Ok(PersistAnswerSyncResult {
            success_question_ids,
            failed_question_ids,
            first_error,
        });
    }

    let ans = Alias::new("ans");
    let answered_count_query = Query::select()
        .expr_as(Expr::cust("COUNT(1)"), Alias::new("answered_count"))
        .from_as(Alias::new("answer_sheets"), ans.clone())
        .and_where(Expr::cust(format!(
            "student_exam_id = '{}'",
            sql_escape(&student_exam_id)
        )))
        .and_where(Expr::cust("answer IS NOT NULL"))
        .and_where(Expr::cust("TRIM(answer) <> ''"))
        .to_owned();
    let answered_count = match state.db.query_one(&answered_count_query).await? {
        Some(row) => row.try_get("", "answered_count").unwrap_or(0),
        None => 0,
    };

    let progress_percent = if total_questions > 0 {
        (answered_count * 100 / total_questions).clamp(0, 100)
    } else {
        0
    };

    let last_question_id = payload
        .answers
        .last()
        .map(|item| item.question_id.clone());

    let escaped_student_exam_id = sql_escape(&student_exam_id);
    let escaped_exam_id = sql_escape(&payload.exam_id);
    let escaped_student_id = sql_escape(&resolved_student_id);
    let last_question_sql = last_question_id
        .map(|value| format!("'{}'", sql_escape(&value)))
        .unwrap_or_else(|| "NULL".to_string());

    let upsert_progress_sql = format!(
        "INSERT INTO student_exam_progress (student_exam_id, exam_id, student_id, answered_count, total_questions, progress_percent, last_question_id, last_answer_at, updated_at) \
         VALUES ('{}', '{}', '{}', {}, {}, {}, {}, {}, {}) \
         ON CONFLICT(student_exam_id) DO UPDATE SET \
           answered_count = excluded.answered_count, \
           total_questions = excluded.total_questions, \
           progress_percent = excluded.progress_percent, \
           last_question_id = excluded.last_question_id, \
           last_answer_at = excluded.last_answer_at, \
           updated_at = excluded.updated_at",
        escaped_student_exam_id,
        escaped_exam_id,
        escaped_student_id,
        answered_count,
        total_questions,
        progress_percent,
        last_question_sql,
        received_at,
        received_at,
    );

    if let Err(err) = state.db.execute_unprepared(&upsert_progress_sql).await {
        if first_error.is_none() {
            first_error = Some(err.to_string());
        }
        for qid in success_question_ids.iter() {
            if !failed_question_ids.contains(qid) {
                failed_question_ids.push(qid.clone());
            }
        }
        success_question_ids.clear();
        eprintln!(
            "[ws-server] progress upsert failed exam_id={} student_exam_id={} err={}",
            payload.exam_id,
            student_exam_id,
            err
        );
    }

    Ok(PersistAnswerSyncResult {
        success_question_ids,
        failed_question_ids,
        first_error,
    })
}

fn send_answer_sync_ack(
    app_handle: &tauri::AppHandle,
    peer_id: &str,
    payload: &AnswerSyncPayload,
    ts: i64,
    success: bool,
    message: String,
    success_question_ids: Vec<String>,
    failed_question_ids: Vec<String>,
) -> Result<()> {
    let ack_payload = AnswerSyncAckPayload {
        exam_id: payload.exam_id.clone(),
        student_id: payload.student_id.clone(),
        session_id: payload.session_id.clone(),
        sync_mode: payload.sync_mode.clone(),
        batch_id: payload.batch_id.clone(),
        success,
        message,
        acked_at: ts,
        question_ids: success_question_ids.clone(),
        failed_question_ids: failed_question_ids.clone(),
        success_count: success_question_ids.len() as i64,
        failed_count: failed_question_ids.len() as i64,
    };

    let envelope = build_message(MessageType::AnswerSyncAck, ts, ack_payload);
    let text = encode_message(&envelope)?;
    let state = app_handle.state::<crate::state::AppState>();
    let delivered = state.send_ws_text_to_peer(peer_id, text);
    if !delivered {
        return Err(anyhow::anyhow!("ACK 发送失败：目标连接不可用"));
    }

    Ok(())
}

fn sql_escape(input: &str) -> String {
    input.replace('\'', "''")
}
