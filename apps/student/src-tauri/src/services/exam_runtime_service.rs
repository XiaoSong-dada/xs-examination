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

    pub async fn get_current_session_answer_sync_bundle(
        app_handle: &tauri::AppHandle,
    ) -> Result<Option<(String, String, String, Vec<LocalAnswerDto>)>> {
        let state = app_handle.state::<crate::state::AppState>();
        let latest_session = exam_sessions::Entity::find()
            .order_by_desc(exam_sessions::Column::UpdatedAt)
            .one(&state.db)
            .await?;

        let Some(session) = latest_session else {
            return Ok(None);
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
            .one(&state.db)
            .await?;

        let target_session_id = existing_same_exam
            .as_ref()
            .map(|item| item.id.clone())
            .unwrap_or_else(|| payload.session_id.clone());

        // 第二阶段策略：命中同 exam_id 时，保留本地 exam_sessions 基础信息，不做覆盖更新。
        if existing_same_exam.is_none() {
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
        let row = exam_sessions::Entity::find()
            .filter(exam_sessions::Column::ExamId.eq(exam_id.to_string()))
            .filter(exam_sessions::Column::StudentId.eq(student_id.to_string()))
            .one(&state.db)
            .await?;

        let Some(current) = row else {
            return Ok(false);
        };

        let mut model: exam_sessions::ActiveModel = current.into();
        model.status = Set("active".to_string());
        model.started_at = Set(Some(start_time));
        model.ends_at = Set(end_time);
        model.updated_at = Set(now_ms());
        model.update(&state.db).await?;

        Ok(true)
    }

    pub async fn get_current_exam_bundle(
        app_handle: &tauri::AppHandle,
    ) -> Result<CurrentExamBundleDto> {
        let state = app_handle.state::<crate::state::AppState>();
        let latest_session = exam_sessions::Entity::find()
            .order_by_desc(exam_sessions::Column::UpdatedAt)
            .one(&state.db)
            .await?;

        let Some(session) = latest_session else {
            return Ok(CurrentExamBundleDto {
                session: None,
                snapshot: None,
            });
        };

        let snapshot = exam_snapshots::Entity::find_by_id(session.id.clone())
            .one(&state.db)
            .await?;

        Ok(CurrentExamBundleDto {
            session: Some(ExamSessionDto {
                id: session.id.clone(),
                exam_id: session.exam_id,
                student_id: session.student_id,
                student_no: session.student_no,
                student_name: session.student_name,
                assigned_ip_addr: session.assigned_ip_addr,
                assigned_device_name: session.assigned_device_name,
                exam_title: session.exam_title,
                status: session.status,
                assignment_status: session.assignment_status,
                started_at: session.started_at,
                ends_at: session.ends_at,
                paper_version: session.paper_version,
                last_synced_at: session.last_synced_at,
                created_at: session.created_at,
                updated_at: session.updated_at,
            }),
            snapshot: snapshot.map(|item| ExamSnapshotDto {
                session_id: item.session_id,
                exam_meta: String::from_utf8_lossy(&item.exam_meta).to_string(),
                questions_payload: String::from_utf8_lossy(&item.questions_payload).to_string(),
                downloaded_at: item.downloaded_at,
                expires_at: item.expires_at,
                updated_at: item.updated_at,
            }),
        })
    }
}
