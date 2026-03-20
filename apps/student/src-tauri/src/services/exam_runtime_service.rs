use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use tauri::Manager;

use crate::db::entities::{exam_sessions, exam_snapshots};
use crate::schemas::control_protocol::{ApplyTeacherEndpointsPayload, DistributeExamPaperPayload};
use crate::schemas::exam_runtime_schema::{CurrentExamBundleDto, ExamSessionDto, ExamSnapshotDto};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

pub struct ExamRuntimeService;

impl ExamRuntimeService {
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
