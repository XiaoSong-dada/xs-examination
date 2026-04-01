use anyhow::Result;
use crate::network::protocol::AnswerItem;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use serde::Deserialize;
use tauri::Manager;

use crate::db::entities::{exam_sessions, exam_snapshots, local_answers, sync_outbox};
use crate::schemas::control_protocol::{ApplyTeacherEndpointsPayload, DistributeExamPaperPayload};
use crate::schemas::exam_runtime_schema::{CurrentExamBundleDto, ExamSessionDto, ExamSnapshotDto, LocalAnswerDto};
use crate::utils::datetime::now_ms;


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

        let rows = sync_outbox::Entity::find()
            .filter(sync_outbox::Column::EventType.eq("ANSWER_SYNC".to_string()))
            .filter(sync_outbox::Column::Status.is_in(["pending", "failed"]))
            .order_by_asc(sync_outbox::Column::CreatedAt)
            .limit(max_count as u64)
            .all(&state.db)
            .await?;

        let mut flushed = 0usize;
        for row in rows {
            let session_status = exam_sessions::Entity::find_by_id(row.session_id.clone())
                .one(&state.db)
                .await?
                .map(|session| session.status)
                .unwrap_or_else(|| "waiting".to_string());

            if session_status == "ended" {
                let next_retry_count = row.retry_count + 1;
                let mut model: sync_outbox::ActiveModel = row.into();
                model.status = Set("failed".to_string());
                model.retry_count = Set(next_retry_count);
                model.last_error = Set(Some("考试已结束，停止继续同步答案".to_string()));
                model.updated_at = Set(now_ms());
                model.update(&state.db).await?;
                continue;
            }

            let payload: PendingAnswerSyncPayload = match serde_json::from_slice(&row.payload) {
                Ok(v) => v,
                Err(err) => {
                    let next_retry_count = row.retry_count + 1;
                    let mut model: sync_outbox::ActiveModel = row.into();
                    model.status = Set("failed".to_string());
                    model.retry_count = Set(next_retry_count);
                    model.last_error = Set(Some(format!("payload parse failed: {}", err)));
                    model.updated_at = Set(now_ms());
                    model.update(&state.db).await?;
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

            let mut model: sync_outbox::ActiveModel = row.into();
            model.status = Set("sent".to_string());
            model.updated_at = Set(now_ms());
            model.last_error = Set(None);
            model.update(&state.db).await?;
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
        let sessions = exam_sessions::Entity::find()
            .order_by_desc(exam_sessions::Column::UpdatedAt)
            .all(&state.db)
            .await?;

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

        let answers = local_answers::Entity::find()
            .filter(local_answers::Column::SessionId.eq(session.id.clone()))
            .order_by_desc(local_answers::Column::UpdatedAt)
            .all(&state.db)
            .await?;

        let items: Vec<LocalAnswerDto> = answers
            .into_iter()
            .filter_map(|row| {
                row.answer.map(|answer| LocalAnswerDto {
                    question_id: row.question_id,
                    answer,
                    revision: row.revision,
                    updated_at: row.updated_at,
                })
            })
            .collect();

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
            let row = exam_sessions::Entity::find()
                .filter(exam_sessions::Column::ExamId.eq(exam_id.to_string()))
                .filter(exam_sessions::Column::StudentId.eq(student_id.to_string()))
                .order_by_desc(exam_sessions::Column::UpdatedAt)
                .one(&state.db)
                .await?;

            let Some(session) = row else {
                return Ok(0);
            };
            session.id
        };

        let full_sync = question_ids.is_empty();
        let mut synced_count = 0usize;

        let mut query = local_answers::Entity::find()
            .filter(local_answers::Column::SessionId.eq(resolved_session_id.clone()));
        if !full_sync {
            query = query.filter(local_answers::Column::QuestionId.is_in(question_ids.iter().cloned()));
        }

        let rows = query.all(&state.db).await?;
        for row in rows {
            let mut model: local_answers::ActiveModel = row.into();
            model.sync_status = Set("synced".to_string());
            model.last_synced_at = Set(Some(acked_at));
            model.updated_at = Set(acked_at);
            model.update(&state.db).await?;
            synced_count += 1;
        }

        let mut outbox_query = crate::db::entities::sync_outbox::Entity::find()
            .filter(crate::db::entities::sync_outbox::Column::SessionId.eq(resolved_session_id.clone()))
            .filter(crate::db::entities::sync_outbox::Column::EventType.eq("ANSWER_SYNC".to_string()))
            .filter(crate::db::entities::sync_outbox::Column::Status.is_in(["pending", "failed", "sent"]));

        if !full_sync {
            let aggregate_ids: Vec<String> = question_ids
                .iter()
                .map(|qid| format!("{}:{}", resolved_session_id, qid))
                .collect();
            outbox_query = outbox_query.filter(
                crate::db::entities::sync_outbox::Column::AggregateId.is_in(aggregate_ids),
            );
        }

        let outbox_rows = outbox_query.all(&state.db).await?;
        for row in outbox_rows {
            let mut model: crate::db::entities::sync_outbox::ActiveModel = row.into();
            model.status = Set("synced".to_string());
            model.updated_at = Set(acked_at);
            model.last_error = Set(None);
            model.update(&state.db).await?;
        }

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
            let row = exam_sessions::Entity::find()
                .filter(exam_sessions::Column::ExamId.eq(exam_id.to_string()))
                .filter(exam_sessions::Column::StudentId.eq(student_id.to_string()))
                .order_by_desc(exam_sessions::Column::UpdatedAt)
                .one(&state.db)
                .await?;

            let Some(session) = row else {
                return Ok(0);
            };
            session.id
        };

        let full_sync = question_ids.is_empty();
        let mut failed_count = 0usize;

        let mut answer_query = local_answers::Entity::find()
            .filter(local_answers::Column::SessionId.eq(resolved_session_id.clone()));
        if !full_sync {
            answer_query = answer_query
                .filter(local_answers::Column::QuestionId.is_in(question_ids.iter().cloned()));
        }

        let answer_rows = answer_query.all(&state.db).await?;
        for row in answer_rows {
            let mut model: local_answers::ActiveModel = row.into();
            model.sync_status = Set("pending".to_string());
            model.updated_at = Set(failed_at);
            model.update(&state.db).await?;
            failed_count += 1;
        }

        let mut outbox_query = sync_outbox::Entity::find()
            .filter(sync_outbox::Column::SessionId.eq(resolved_session_id.clone()))
            .filter(sync_outbox::Column::EventType.eq("ANSWER_SYNC".to_string()))
            .filter(sync_outbox::Column::Status.is_in(["pending", "failed", "sent"]));

        if !full_sync {
            let aggregate_ids: Vec<String> = question_ids
                .iter()
                .map(|qid| format!("{}:{}", resolved_session_id, qid))
                .collect();
            outbox_query = outbox_query
                .filter(sync_outbox::Column::AggregateId.is_in(aggregate_ids));
        }

        let outbox_rows = outbox_query.all(&state.db).await?;
        for row in outbox_rows {
            let next_retry_count = row.retry_count + 1;
            let mut model: sync_outbox::ActiveModel = row.into();
            model.status = Set("failed".to_string());
            model.retry_count = Set(next_retry_count);
            model.last_error = Set(Some(error_message.to_string()));
            model.updated_at = Set(failed_at);
            model.update(&state.db).await?;
        }

        Ok(failed_count)
    }

    pub async fn get_current_session_answers(
        app_handle: &tauri::AppHandle,
    ) -> Result<Vec<LocalAnswerDto>> {
        let state = app_handle.state::<crate::state::AppState>();
        let latest_session = exam_sessions::Entity::find()
            .order_by_desc(exam_sessions::Column::UpdatedAt)
            .one(&state.db)
            .await?;

        let Some(session) = latest_session else {
            return Ok(Vec::new());
        };

        let rows = local_answers::Entity::find()
            .filter(local_answers::Column::SessionId.eq(session.id.clone()))
            .order_by_desc(local_answers::Column::UpdatedAt)
            .all(&state.db)
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                row.answer.map(|answer| LocalAnswerDto {
                    question_id: row.question_id,
                    answer,
                    revision: row.revision,
                    updated_at: row.updated_at,
                })
            })
            .collect())
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
        let assignment_status = payload
            .assignment_status
            .clone()
            .unwrap_or_else(|| "assigned".to_string());

        let existing_session = exam_sessions::Entity::find_by_id(session_id.clone())
            .one(&state.db)
            .await?;

        match existing_session {
            Some(row) => {
                let mut model: exam_sessions::ActiveModel = row.into();
                model.exam_id = Set(exam_id);
                model.student_id = Set(payload.student_id.clone());
                model.student_no = Set(student_no);
                model.student_name = Set(student_name);
                model.assigned_ip_addr = Set(assigned_ip_addr);
                model.exam_title = Set(exam_title);
                model.status = Set("connected_pending_distribution".to_string());
                model.assignment_status = Set(assignment_status);
                model.started_at = Set(None);
                model.ends_at = Set(payload.end_time);
                model.paper_version = Set(None);
                model.encryption_nonce = Set(None);
                model.updated_at = Set(ts);
                model.update(&state.db).await?;
            }
            None => {
                let model = exam_sessions::ActiveModel {
                    id: Set(session_id),
                    exam_id: Set(exam_id),
                    student_id: Set(payload.student_id.clone()),
                    student_no: Set(student_no),
                    student_name: Set(student_name),
                    assigned_ip_addr: Set(assigned_ip_addr),
                    assigned_device_name: Set(None),
                    exam_title: Set(exam_title),
                    status: Set("connected_pending_distribution".to_string()),
                    assignment_status: Set(assignment_status),
                    started_at: Set(None),
                    ends_at: Set(payload.end_time),
                    paper_version: Set(None),
                    encryption_nonce: Set(None),
                    last_synced_at: Set(None),
                    created_at: Set(ts),
                    updated_at: Set(ts),
                };
                model.insert(&state.db).await?;
            }
        }

        Ok(true)
    }

    pub async fn upsert_distribution(
        app_handle: &tauri::AppHandle,
        payload: &DistributeExamPaperPayload,
    ) -> Result<()> {
        let state = app_handle.state::<crate::state::AppState>();
        let ts = now_ms();

        let existing_same_exam = exam_sessions::Entity::find()
            .filter(exam_sessions::Column::ExamId.eq(payload.exam_id.clone()))
            .order_by_desc(exam_sessions::Column::UpdatedAt)
            .one(&state.db)
            .await?;

        let target_session_id = existing_same_exam
            .as_ref()
            .map(|item| item.id.clone())
            .unwrap_or_else(|| payload.session_id.clone());

        // 第二阶段策略：命中同 exam_id 时，保留本地 exam_sessions 基础信息，不做覆盖更新。
        // 但仍需刷新状态与更新时间，确保前端读取到的最新会话可关联快照。
        if let Some(row) = existing_same_exam {
            let mut model: exam_sessions::ActiveModel = row.into();
            model.status = Set("waiting".to_string());
            model.ends_at = Set(payload.end_time);
            model.paper_version = Set(payload.paper_version.clone());
            model.updated_at = Set(ts);
            model.update(&state.db).await?;
        } else {
            let existing_session = exam_sessions::Entity::find_by_id(payload.session_id.clone())
                .one(&state.db)
                .await?;

            match existing_session {
                Some(row) => {
                    let mut model: exam_sessions::ActiveModel = row.into();
                    model.exam_id = Set(payload.exam_id.clone());
                    model.student_id = Set(payload.student_id.clone());
                    model.student_no = Set(payload.student_no.clone());
                    model.student_name = Set(payload.student_name.clone());
                    model.assigned_ip_addr = Set(payload.assigned_ip_addr.clone());
                    model.exam_title = Set(payload.exam_title.clone());
                    model.status = Set("waiting".to_string());
                    model.assignment_status = Set(payload.assignment_status.clone());
                    model.started_at = Set(None);
                    model.ends_at = Set(payload.end_time);
                    model.paper_version = Set(payload.paper_version.clone());
                    model.encryption_nonce = Set(None);
                    model.updated_at = Set(ts);
                    model.update(&state.db).await?;
                }
                None => {
                    let model = exam_sessions::ActiveModel {
                        id: Set(payload.session_id.clone()),
                        exam_id: Set(payload.exam_id.clone()),
                        student_id: Set(payload.student_id.clone()),
                        student_no: Set(payload.student_no.clone()),
                        student_name: Set(payload.student_name.clone()),
                        assigned_ip_addr: Set(payload.assigned_ip_addr.clone()),
                        assigned_device_name: Set(None),
                        exam_title: Set(payload.exam_title.clone()),
                        status: Set("waiting".to_string()),
                        assignment_status: Set(payload.assignment_status.clone()),
                        started_at: Set(None),
                        ends_at: Set(payload.end_time),
                        paper_version: Set(payload.paper_version.clone()),
                        encryption_nonce: Set(None),
                        last_synced_at: Set(None),
                        created_at: Set(ts),
                        updated_at: Set(ts),
                    };
                    model.insert(&state.db).await?;
                }
            }
        }

        let existing_snapshot = exam_snapshots::Entity::find_by_id(target_session_id.clone())
            .one(&state.db)
            .await?;
        match existing_snapshot {
            Some(row) => {
                let mut model: exam_snapshots::ActiveModel = row.into();
                model.exam_meta = Set(payload.exam_meta.clone().into_bytes());
                model.questions_payload = Set(payload.questions_payload.clone().into_bytes());
                model.downloaded_at = Set(payload.downloaded_at);
                model.expires_at = Set(payload.expires_at);
                model.assets_sync_status = Set(Some("pending".to_string()));
                model.assets_synced_at = Set(None);
                model.updated_at = Set(ts);
                model.update(&state.db).await?;
            }
            None => {
                let model = exam_snapshots::ActiveModel {
                    session_id: Set(target_session_id),
                    exam_meta: Set(payload.exam_meta.clone().into_bytes()),
                    questions_payload: Set(payload.questions_payload.clone().into_bytes()),
                    downloaded_at: Set(payload.downloaded_at),
                    expires_at: Set(payload.expires_at),
                    assets_sync_status: Set(Some("pending".to_string())),
                    assets_synced_at: Set(None),
                    updated_at: Set(ts),
                };
                model.insert(&state.db).await?;
            }
        }

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
        let rows = exam_sessions::Entity::find()
            .filter(exam_sessions::Column::ExamId.eq(exam_id.to_string()))
            .filter(exam_sessions::Column::StudentId.eq(student_id.to_string()))
            .order_by_desc(exam_sessions::Column::UpdatedAt)
            .all(&state.db)
            .await?;

        if rows.is_empty() {
            return Ok(false);
        }

        // Prefer a session that already has a snapshot to ensure frontend can enter exam view.
        let mut selected = rows[0].clone();
        for row in rows {
            let snapshot = exam_snapshots::Entity::find_by_id(row.id.clone())
                .one(&state.db)
                .await?;
            if snapshot.is_some() {
                selected = row;
                break;
            }
        }

        let mut model: exam_sessions::ActiveModel = selected.into();
        model.status = Set("active".to_string());
        model.started_at = Set(Some(start_time));
        model.ends_at = Set(end_time);
        model.updated_at = Set(now_ms());
        model.update(&state.db).await?;

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
        let row = exam_sessions::Entity::find()
            .filter(exam_sessions::Column::ExamId.eq(exam_id.to_string()))
            .filter(exam_sessions::Column::StudentId.eq(student_id.to_string()))
            .order_by_desc(exam_sessions::Column::UpdatedAt)
            .one(&state.db)
            .await?;

        let Some(session) = row else {
            return Ok(false);
        };

        let mut model: exam_sessions::ActiveModel = session.into();
        model.status = Set("ended".to_string());
        model.ends_at = Set(Some(end_time));
        model.last_synced_at = Set(Some(now_ms()));
        model.updated_at = Set(now_ms());
        model.update(&state.db).await?;

        Ok(true)
    }

    pub async fn get_current_exam_bundle(
        app_handle: &tauri::AppHandle,
    ) -> Result<CurrentExamBundleDto> {
        let state = app_handle.state::<crate::state::AppState>();
        let sessions = exam_sessions::Entity::find()
            .order_by_desc(exam_sessions::Column::UpdatedAt)
            .all(&state.db)
            .await?;

        let target_student_id = state
            .reconnect_target()
            .map(|(_, student_id)| student_id)
            .filter(|id| !id.trim().is_empty());

        let candidate_sessions: Vec<exam_sessions::Model> = if let Some(student_id) = target_student_id {
            let filtered: Vec<exam_sessions::Model> = sessions
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
        let mut selected_snapshot: Option<exam_snapshots::Model> = None;

        for session in candidate_sessions {
            let snapshot = exam_snapshots::Entity::find_by_id(session.id.clone())
                .one(&state.db)
                .await?;
            if snapshot.is_some() {
                selected_session = session;
                selected_snapshot = snapshot;
                break;
            }
        }

        if selected_snapshot.is_none() {
            selected_snapshot = exam_snapshots::Entity::find_by_id(selected_session.id.clone())
                .one(&state.db)
                .await?;
        }

        Ok(CurrentExamBundleDto {
            session: Some(ExamSessionDto {
                id: selected_session.id.clone(),
                exam_id: selected_session.exam_id,
                student_id: selected_session.student_id,
                student_no: selected_session.student_no,
                student_name: selected_session.student_name,
                assigned_ip_addr: selected_session.assigned_ip_addr,
                assigned_device_name: selected_session.assigned_device_name,
                exam_title: selected_session.exam_title,
                status: selected_session.status,
                assignment_status: selected_session.assignment_status,
                started_at: selected_session.started_at,
                ends_at: selected_session.ends_at,
                paper_version: selected_session.paper_version,
                last_synced_at: selected_session.last_synced_at,
                created_at: selected_session.created_at,
                updated_at: selected_session.updated_at,
            }),
            snapshot: selected_snapshot.map(|item| ExamSnapshotDto {
                session_id: item.session_id,
                exam_meta: String::from_utf8_lossy(&item.exam_meta).to_string(),
                questions_payload: String::from_utf8_lossy(&item.questions_payload).to_string(),
                downloaded_at: item.downloaded_at,
                expires_at: item.expires_at,
                assets_sync_status: item.assets_sync_status,
                assets_synced_at: item.assets_synced_at,
                updated_at: item.updated_at,
            }),
        })
    }
}
