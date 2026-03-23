use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use tauri::Manager;

use crate::db::entities::{exam_sessions, exam_snapshots, local_answers};
use crate::schemas::control_protocol::{ApplyTeacherEndpointsPayload, DistributeExamPaperPayload};
use crate::schemas::exam_runtime_schema::{CurrentExamBundleDto, ExamSessionDto, ExamSnapshotDto, LocalAnswerDto};
use crate::utils::datetime::now_ms;


pub struct ExamRuntimeService;

impl ExamRuntimeService {
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
                updated_at: item.updated_at,
            }),
        })
    }
}
