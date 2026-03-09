use anyhow::Result;
use sea_orm::DatabaseConnection;

use crate::models::question::Model as QuestionModel;
use crate::repos::question_repo;

#[derive(Debug, Clone)]
pub struct QuestionWritePayload {
    pub id: Option<String>,
    pub seq: i32,
    pub r#type: String,
    pub content: String,
    pub options: Option<String>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
}

pub async fn list_questions(db: &DatabaseConnection, exam_id: String) -> Result<Vec<QuestionModel>> {
    question_repo::get_all_questions(db, &exam_id).await
}

pub async fn replace_questions_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
    payloads: Vec<QuestionWritePayload>,
) -> Result<Vec<QuestionModel>> {
    let rows = payloads
        .into_iter()
        .map(|payload| question_repo::QuestionBatchInsertItem {
            id: payload.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            exam_id: exam_id.clone(),
            seq: payload.seq,
            r#type: payload.r#type.trim().to_string(),
            content: payload.content.trim().to_string(),
            options: payload.options.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()),
            answer: payload.answer.trim().to_string(),
            score: payload.score,
            explanation: payload
                .explanation
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
        })
        .collect();

    question_repo::replace_questions_by_exam_id(db, &exam_id, rows).await
}
