use anyhow::Result;
use sea_orm::DatabaseConnection;

use crate::models::question::Model as QuestionModel;
use crate::repos::question_repo;

pub async fn list_questions(db: &DatabaseConnection, exam_id: String) -> Result<Vec<QuestionModel>> {
    question_repo::get_all_questions(db, &exam_id).await
}
