use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::db::entities::exam_sessions;
use crate::schemas::control_protocol::{ApplyTeacherEndpointsPayload, DistributeExamPaperPayload};
use crate::schemas::exam_runtime_schema::ExamSessionDto;

/// 获取所有考试会话，按更新时间倒序排列。
pub async fn get_all_sessions(db: &DatabaseConnection) -> Result<Vec<exam_sessions::Model>> {
    let sessions = exam_sessions::Entity::find()
        .order_by_desc(exam_sessions::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(sessions)
}

/// 根据会话 ID 获取考试会话。
pub async fn get_session_by_id(db: &DatabaseConnection, id: &str) -> Result<Option<exam_sessions::Model>> {
    let session = exam_sessions::Entity::find_by_id(id.to_string())
        .one(db)
        .await?;
    Ok(session)
}

/// 根据考试 ID 和学生 ID 获取最新的考试会话。
pub async fn get_session_by_exam_and_student(
    db: &DatabaseConnection,
    exam_id: &str,
    student_id: &str,
) -> Result<Option<exam_sessions::Model>> {
    let session = exam_sessions::Entity::find()
        .filter(exam_sessions::Column::ExamId.eq(exam_id.to_string()))
        .filter(exam_sessions::Column::StudentId.eq(student_id.to_string()))
        .order_by_desc(exam_sessions::Column::UpdatedAt)
        .one(db)
        .await?;
    Ok(session)
}

/// 更新会话状态为已开始。
pub async fn mark_session_started(
    db: &DatabaseConnection,
    session: exam_sessions::Model,
    start_time: i64,
    end_time: Option<i64>,
    updated_at: i64,
) -> Result<exam_sessions::Model> {
    let mut model: exam_sessions::ActiveModel = session.into();
    model.status = Set("active".to_string());
    model.started_at = Set(Some(start_time));
    model.ends_at = Set(end_time);
    model.updated_at = Set(updated_at);
    let updated = model.update(db).await?;
    Ok(updated)
}

/// 更新会话状态为已结束。
pub async fn mark_session_ended(
    db: &DatabaseConnection,
    session: exam_sessions::Model,
    end_time: i64,
    updated_at: i64,
) -> Result<exam_sessions::Model> {
    let mut model: exam_sessions::ActiveModel = session.into();
    model.status = Set("ended".to_string());
    model.ends_at = Set(Some(end_time));
    model.last_synced_at = Set(Some(updated_at));
    model.updated_at = Set(updated_at);
    let updated = model.update(db).await?;
    Ok(updated)
}

/// 插入或更新连接会话。
pub async fn upsert_connected_session(
    db: &DatabaseConnection,
    payload: &ApplyTeacherEndpointsPayload,
    ts: i64,
) -> Result<()> {
    let session_id = payload.session_id.as_ref().unwrap();
    let exam_id = payload.exam_id.as_ref().unwrap();
    let student_id = &payload.student_id;
    let student_no = payload.student_no.as_ref().unwrap();
    let student_name = payload.student_name.as_ref().unwrap();
    let assigned_ip_addr = payload.assigned_ip_addr.as_ref().unwrap();
    let exam_title = payload.exam_title.as_ref().unwrap();
    let assignment_status = payload
        .assignment_status
        .clone()
        .unwrap_or_else(|| "assigned".to_string());

    let existing_session = exam_sessions::Entity::find_by_id(session_id.clone())
        .one(db)
        .await?;

    match existing_session {
        Some(row) => {
            let mut model: exam_sessions::ActiveModel = row.into();
            model.exam_id = Set(exam_id.clone());
            model.student_id = Set(student_id.clone());
            model.student_no = Set(student_no.clone());
            model.student_name = Set(student_name.clone());
            model.assigned_ip_addr = Set(assigned_ip_addr.clone());
            model.exam_title = Set(exam_title.clone());
            model.status = Set("connected_pending_distribution".to_string());
            model.assignment_status = Set(assignment_status);
            model.started_at = Set(None);
            model.ends_at = Set(payload.end_time);
            model.paper_version = Set(None);
            model.encryption_nonce = Set(None);
            model.updated_at = Set(ts);
            model.update(db).await?;
        }
        None => {
            let model = exam_sessions::ActiveModel {
                id: Set(session_id.clone()),
                exam_id: Set(exam_id.clone()),
                student_id: Set(student_id.clone()),
                student_no: Set(student_no.clone()),
                student_name: Set(student_name.clone()),
                assigned_ip_addr: Set(assigned_ip_addr.clone()),
                assigned_device_name: Set(None),
                exam_title: Set(exam_title.clone()),
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
            model.insert(db).await?;
        }
    }

    Ok(())
}

/// 插入或更新分发会话。
pub async fn upsert_distribution(
    db: &DatabaseConnection,
    payload: &DistributeExamPaperPayload,
    ts: i64,
) -> Result<String> {
    let existing_same_exam = exam_sessions::Entity::find()
        .filter(exam_sessions::Column::ExamId.eq(payload.exam_id.clone()))
        .order_by_desc(exam_sessions::Column::UpdatedAt)
        .one(db)
        .await?;

    let target_session_id = existing_same_exam
        .as_ref()
        .map(|item| item.id.clone())
        .unwrap_or_else(|| payload.session_id.clone());

    if let Some(row) = existing_same_exam {
        let mut model: exam_sessions::ActiveModel = row.into();
        model.status = Set("waiting".to_string());
        model.ends_at = Set(payload.end_time);
        model.paper_version = Set(payload.paper_version.clone());
        model.updated_at = Set(ts);
        model.update(db).await?;
    } else {
        let existing_session = exam_sessions::Entity::find_by_id(payload.session_id.clone())
            .one(db)
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
                model.update(db).await?;
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
                model.insert(db).await?;
            }
        }
    }

    Ok(target_session_id)
}

/// 将考试会话模型转换为 DTO。
pub fn session_to_dto(session: exam_sessions::Model) -> ExamSessionDto {
    ExamSessionDto {
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
    }
}