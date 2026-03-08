use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, ColumnTrait};

use crate::models::question::{Entity as QuestionEntity, Model as QuestionModel, Column};


pub async fn get_all_questions(db: &DatabaseConnection, exam_id: &str) -> Result<Vec<QuestionModel>> {
    let questions = QuestionEntity::find()
        .filter(Column::ExamId.eq(exam_id.to_string()))
        .order_by_asc(Column::Seq)
        .all(db)
        .await?;
    Ok(questions)
}