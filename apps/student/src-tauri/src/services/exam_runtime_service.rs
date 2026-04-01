use anyhow::Result;
use crate::network::protocol::AnswerItem;
use serde::Deserialize;
use tauri::Manager;

use crate::schemas::control_protocol::{ApplyTeacherEndpointsPayload, DistributeExamPaperPayload};
use crate::schemas::exam_runtime_schema::{CurrentExamBundleDto, LocalAnswerDto};
use crate::utils::datetime::now_ms;
use crate::repos::{exam_session_repo, exam_snapshot_repo, local_answer_repo, sync_outbox_repo};


pub struct ExamRuntimeService;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingAnswerSyncPayload {
    exam_id: String,
    student_id: String,
    question_id: String,
    answer: String,
    revision: Option<i64>,
    timestamp: Option<i64>,
}

impl ExamRuntimeService {
    pub async fn flush_pending_answer_sync(
        app_handle: &tauri::AppHandle,
        max_count: usize,
    ) -> Result<usize> {
        let state = app_handle.state::<crate::state::AppState>();
        let Some(sender) = state.ws_sender() else {
            return Ok(0);
        };

        let rows = sync_outbox_repo::get_pending_answer_syncs(&state.db, max_count).await?;

        let mut flushed = 0usize;
        for row in rows {
            let session_status = exam_session_repo::get_session_by_id(&state.db, &row.session_id)
                .await?
                .map(|session| session.status)
                .unwrap_or_else(|| "waiting".to_string());

            if session_status == "ended" {
                sync_outbox_repo::mark_outbox_failed(&state.db, row, "考试已结束，停止继续同步答案", now_ms()).await?;
                continue;
            }

            let payload: PendingAnswerSyncPayload = match serde_json::from_slice(&row.payload) {
                Ok(v) => v,
                Err(err) => {
                    sync_outbox_repo::mark_outbox_failed(&state.db, row, &format!("payload parse failed: {}", err), now_ms()).await?;
                    continue;
                }
            };

            let answer_item = AnswerItem {
                question_id: payload.question_id.clone(),
                answer: payload.answer.clone(),
                revision: payload.revision,
                answer_updated_at: payload.timestamp,
            };

            let message = crate::network::ws_client::build_answer_sync_message(
                &payload.exam_id,
                &payload.student_id,
                Some(&row.session_id),
                vec![answer_item],
                "incremental",
                Some(&format!("outbox-{}", row.id)),
            )?;

            if sender.send(message).is_err() {
                break;
            }

            sync_outbox_repo::mark_outbox_sent(&state.db, row, now_ms()).await?;
            flushed += 1;
        }

        Ok(flushed)
    }

    /// 发送当前会话答案同步消息，支持 full/incremental/final 三种同步模式。
    ///
    /// # 参数
    /// * `app_handle` - Tauri 应用句柄。
    /// * `sync_mode` - 同步模式，例如 `full`、`incremental`、`final`。
    /// * `batch_id` - 同步批次标识；用于教师端聚合 ACK。
    /// * `allow_empty` - 是否允许在无答案时仍发送空载荷同步消息。
    ///
    /// # 返回值
    /// 返回 `Ok(true)` 表示消息已成功入发送通道；`Ok(false)` 表示当前无可发送会话或未连接。
    pub async fn send_current_session_answer_sync(
        app_handle: &tauri::AppHandle,
        sync_mode: &str,
        batch_id: Option<&str>,
        allow_empty: bool,
    ) -> Result<bool> {
        let state = app_handle.state::<crate::state::AppState>();
        let Some(sender) = state.ws_sender() else {
            return Ok(false);
        };

        let Some((session_id, exam_id, student_id, answers)) =
            Self::get_current_session_answer_sync_bundle(app_handle).await?
        else {
            return Ok(false);
        };

        if !allow_empty && answers.is_empty() {
            return Ok(false);
        }

        if sync_mode == "full" && !state.should_send_full_sync(&session_id, now_ms(), 5_000) {
            println!(
                "[ws-client] skip duplicated full answer sync session_id={} within cooldown",
                session_id
            );
            return Ok(false);
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

        let message = crate::network::ws_client::build_answer_sync_message(
            &exam_id,
            &student_id,
            Some(&session_id),
            payload_answers,
            sync_mode,
            batch_id,
        )?;

        if sender.send(message).is_err() {
            return Err(anyhow::anyhow!("答案同步发送失败：发送通道不可用"));
        }

        Ok(true)
    }

    pub async fn get_current_session_answer_sync_bundle(
        app_handle: &tauri::AppHandle,
    ) -> Result<Option<(String, String, String, Vec<LocalAnswerDto>)>> {
        let state = app_handle.state::<crate::state::AppState>();
        let sessions = exam_session_repo::get_all_sessions(&state.db).await?;

        if sessions.is_empty() {
            return Ok(None);
        }

        let target_student_id = state.reconnect_target().map(|(_, sid)| sid);
        let session = if let Some(target_sid) = target_student_id {
            sessions
                .iter()
                .find(|item| item.student_id == target_sid)
                .cloned()
                .unwrap_or_else(|| sessions[0].clone())
        } else {
            sessions[0].clone()
        };

        let answers = local_answer_repo::get_answers_by_session_id(&state.db, &session.id).await?;
        let items = local_answer_repo::answers_to_dtos(answers);

        Ok(Some((session.id, session.exam_id, session.student_id, items)))
    }

    pub async fn mark_answers_synced(
        app_handle: &tauri::AppHandle,
        exam_id: &str,
        student_id: &str,
        session_id: Option<&str>,
        question_ids: &[String],
        acked_at: i64,
    ) -> Result<usize> {
        let state = app_handle.state::<crate::state::AppState>();

        let resolved_session_id = if let Some(value) = session_id {
            value.to_string()
        } else {
            let row = exam_session_repo::get_session_by_exam_and_student(&state.db, exam_id, student_id)
                .await?;

            let Some(session) = row else {
                return Ok(0);
            };
            session.id
        };

        let synced_count = local_answer_repo::mark_answers_synced(&state.db, &resolved_session_id, question_ids, acked_at).await?;
        sync_outbox_repo::mark_outbox_synced(&state.db, &resolved_session_id, question_ids, acked_at).await?;

        Ok(synced_count)
    }

    pub async fn mark_answers_failed(
        app_handle: &tauri::AppHandle,
        exam_id: &str,
        student_id: &str,
        session_id: Option<&str>,
        question_ids: &[String],
        failed_at: i64,
        error_message: &str,
    ) -> Result<usize> {
        let state = app_handle.state::<crate::state::AppState>();

        let resolved_session_id = if let Some(value) = session_id {
            value.to_string()
        } else {
            let row = exam_session_repo::get_session_by_exam_and_student(&state.db, exam_id, student_id)
                .await?;

            let Some(session) = row else {
                return Ok(0);
            };
            session.id
        };

        let failed_count = local_answer_repo::mark_answers_failed(&state.db, &resolved_session_id, question_ids, failed_at).await?;
        sync_outbox_repo::mark_outbox_failed_batch(&state.db, &resolved_session_id, question_ids, error_message, failed_at).await?;

        Ok(failed_count)
    }

    pub async fn get_current_session_answers(
        app_handle: &tauri::AppHandle,
    ) -> Result<Vec<LocalAnswerDto>> {
        let state = app_handle.state::<crate::state::AppState>();
        let sessions = exam_session_repo::get_all_sessions(&state.db).await?;

        let Some(session) = sessions.first() else {
            return Ok(Vec::new());
        };

        let answers = local_answer_repo::get_answers_by_session_id(&state.db, &session.id).await?;
        let items = local_answer_repo::answers_to_dtos(answers);

        Ok(items)
    }

    pub async fn upsert_connected_session(
        app_handle: &tauri::AppHandle,
        payload: &ApplyTeacherEndpointsPayload,
    ) -> Result<bool> {
        let (
            Some(session_id),
            Some(exam_id),
            Some(exam_title),
            Some(student_no),
            Some(student_name),
            Some(assigned_ip_addr),
        ) = (
            payload.session_id.clone(),
            payload.exam_id.clone(),
            payload.exam_title.clone(),
            payload.student_no.clone(),
            payload.student_name.clone(),
            payload.assigned_ip_addr.clone(),
        ) else {
            return Ok(false);
        };

        if session_id.trim().is_empty()
            || exam_id.trim().is_empty()
            || exam_title.trim().is_empty()
            || student_no.trim().is_empty()
            || student_name.trim().is_empty()
            || assigned_ip_addr.trim().is_empty()
        {
            return Ok(false);
        }

        let state = app_handle.state::<crate::state::AppState>();
        let ts = now_ms();

        exam_session_repo::upsert_connected_session(&state.db, payload, ts).await?;

        Ok(true)
    }

    pub async fn upsert_distribution(
        app_handle: &tauri::AppHandle,
        payload: &DistributeExamPaperPayload,
    ) -> Result<()> {
        let state = app_handle.state::<crate::state::AppState>();
        let ts = now_ms();

        let target_session_id = exam_session_repo::upsert_distribution(&state.db, payload, ts).await?;
        exam_snapshot_repo::upsert_snapshot(&state.db, &target_session_id, payload, ts).await?;

        Ok(())
    }

    pub async fn mark_exam_started(
        app_handle: &tauri::AppHandle,
        exam_id: &str,
        student_id: &str,
        start_time: i64,
        end_time: Option<i64>,
    ) -> Result<bool> {
        let state = app_handle.state::<crate::state::AppState>();
        let rows = exam_session_repo::get_all_sessions(&state.db).await?;

        let filtered_rows: Vec<_> = rows
            .into_iter()
            .filter(|row| row.exam_id == exam_id && row.student_id == student_id)
            .collect();

        if filtered_rows.is_empty() {
            return Ok(false);
        }

        // Prefer a session that already has a snapshot to ensure frontend can enter exam view.
        let mut selected = filtered_rows[0].clone();
        for row in filtered_rows {
            let snapshot = exam_snapshot_repo::get_snapshot_by_session_id(&state.db, &row.id)
                .await?;
            if snapshot.is_some() {
                selected = row;
                break;
            }
        }

        exam_session_repo::mark_session_started(&state.db, selected, start_time, end_time, now_ms()).await?;

        Ok(true)
    }

    /// 将指定考试会话标记为已结束。
    ///
    /// # 参数
    /// * `app_handle` - Tauri 应用句柄。
    /// * `exam_id` - 考试 ID。
    /// * `student_id` - 学生 ID。
    /// * `end_time` - 结束时间戳（毫秒）。
    ///
    /// # 返回值
    /// 若命中本地会话并成功更新状态，返回 `Ok(true)`；否则返回 `Ok(false)`。
    pub async fn mark_exam_ended(
        app_handle: &tauri::AppHandle,
        exam_id: &str,
        student_id: &str,
        end_time: i64,
    ) -> Result<bool> {
        let state = app_handle.state::<crate::state::AppState>();
        let session = exam_session_repo::get_session_by_exam_and_student(&state.db, exam_id, student_id)
            .await?;

        let Some(session) = session else {
            return Ok(false);
        };

        exam_session_repo::mark_session_ended(&state.db, session, end_time, now_ms()).await?;

        Ok(true)
    }

    pub async fn get_current_exam_bundle(
        app_handle: &tauri::AppHandle,
    ) -> Result<CurrentExamBundleDto> {
        let state = app_handle.state::<crate::state::AppState>();
        let sessions = exam_session_repo::get_all_sessions(&state.db).await?;

        let target_student_id = state
            .reconnect_target()
            .map(|(_, student_id)| student_id)
            .filter(|id| !id.trim().is_empty());

        let candidate_sessions: Vec<_> = if let Some(student_id) = target_student_id {
            let filtered: Vec<_> = sessions
                .iter()
                .filter(|item| item.student_id == student_id)
                .cloned()
                .collect();
            if filtered.is_empty() {
                sessions.clone()
            } else {
                filtered
            }
        } else {
            sessions.clone()
        };

        let Some(default_session) = candidate_sessions.first().cloned() else {
            return Ok(CurrentExamBundleDto {
                session: None,
                snapshot: None,
            });
        };

        // Prefer the most recently updated session that already has a snapshot.
        let mut selected_session = default_session;
        let mut selected_snapshot: Option<_> = None;

        for session in candidate_sessions {
            let snapshot = exam_snapshot_repo::get_snapshot_by_session_id(&state.db, &session.id)
                .await?;
            if snapshot.is_some() {
                selected_session = session;
                selected_snapshot = snapshot;
                break;
            }
        }

        if selected_snapshot.is_none() {
            selected_snapshot = exam_snapshot_repo::get_snapshot_by_session_id(&state.db, &selected_session.id)
                .await?;
        }

        Ok(CurrentExamBundleDto {
            session: Some(exam_session_repo::session_to_dto(selected_session)),
            snapshot: selected_snapshot.map(exam_snapshot_repo::snapshot_to_dto),
        })
    }
}
