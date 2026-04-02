use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::db::entities::local_answers;
use crate::schemas::exam_runtime_schema::LocalAnswerDto;

/// 根据会话 ID 获取本地答案。
pub async fn get_answers_by_session_id(
    db: &DatabaseConnection,
    session_id: &str,
) -> Result<Vec<local_answers::Model>> {
    let answers = local_answers::Entity::find()
        .filter(local_answers::Column::SessionId.eq(session_id.to_string()))
        .order_by_desc(local_answers::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(answers)
}

/// 根据会话 ID 和问题 ID 获取本地答案。
pub async fn get_answers_by_session_and_questions(
    db: &DatabaseConnection,
    session_id: &str,
    question_ids: &[String],
) -> Result<Vec<local_answers::Model>> {
    let answers = local_answers::Entity::find()
        .filter(local_answers::Column::SessionId.eq(session_id.to_string()))
        .filter(local_answers::Column::QuestionId.is_in(question_ids.iter().cloned()))
        .all(db)
        .await?;
    Ok(answers)
}

/// 标记答案为已同步。
pub async fn mark_answers_synced(
    db: &DatabaseConnection,
    session_id: &str,
    question_ids: &[String],
    acked_at: i64,
) -> Result<usize> {
    let full_sync = question_ids.is_empty();
    let mut synced_count = 0usize;

    let mut query = local_answers::Entity::find()
        .filter(local_answers::Column::SessionId.eq(session_id.to_string()));
    if !full_sync {
        query = query.filter(local_answers::Column::QuestionId.is_in(question_ids.iter().cloned()));
    }

    let rows = query.all(db).await?;
    for row in rows {
        let mut model: local_answers::ActiveModel = row.into();
        model.sync_status = Set("synced".to_string());
        model.last_synced_at = Set(Some(acked_at));
        model.updated_at = Set(acked_at);
        model.update(db).await?;
        synced_count += 1;
    }

    Ok(synced_count)
}

/// 标记答案为同步失败。
pub async fn mark_answers_failed(
    db: &DatabaseConnection,
    session_id: &str,
    question_ids: &[String],
    failed_at: i64,
) -> Result<usize> {
    let full_sync = question_ids.is_empty();
    let mut failed_count = 0usize;

    let mut answer_query = local_answers::Entity::find()
        .filter(local_answers::Column::SessionId.eq(session_id.to_string()));
    if !full_sync {
        answer_query = answer_query
            .filter(local_answers::Column::QuestionId.is_in(question_ids.iter().cloned()));
    }

    let answer_rows = answer_query.all(db).await?;
    for row in answer_rows {
        let mut model: local_answers::ActiveModel = row.into();
        model.sync_status = Set("pending".to_string());
        model.updated_at = Set(failed_at);
        model.update(db).await?;
        failed_count += 1;
    }

    Ok(failed_count)
}

/// 将本地答案模型转换为 DTO。
pub fn answer_to_dto(row: local_answers::Model) -> Option<LocalAnswerDto> {
    row.answer.map(|answer| LocalAnswerDto {
        question_id: row.question_id,
        answer,
        revision: row.revision,
        updated_at: row.updated_at,
    })
}

/// 将本地答案模型列表转换为 DTO 列表。
pub fn answers_to_dtos(rows: Vec<local_answers::Model>) -> Vec<LocalAnswerDto> {
    rows
        .into_iter()
        .filter_map(answer_to_dto)
        .collect()
}