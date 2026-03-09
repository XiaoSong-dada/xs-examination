use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};

use crate::models::question::{
    ActiveModel, Column, Entity as QuestionEntity, Model as QuestionModel,
};

#[derive(Debug, Clone)]
pub struct QuestionBatchInsertItem {
    pub id: String,
    pub exam_id: String,
    pub seq: i32,
    pub r#type: String,
    pub content: String,
    pub options: Option<String>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
}


pub async fn get_all_questions(db: &DatabaseConnection, exam_id: &str) -> Result<Vec<QuestionModel>> {
    let questions = QuestionEntity::find()
        .filter(Column::ExamId.eq(exam_id.to_string()))
        .order_by_asc(Column::Seq)
        .all(db)
        .await?;
    Ok(questions)
}

pub async fn replace_questions_by_exam_id(
    db: &DatabaseConnection,
    exam_id: &str,
    rows: Vec<QuestionBatchInsertItem>,
) -> Result<Vec<QuestionModel>> {
    let txn = db.begin().await?;

    QuestionEntity::delete_many()
        .filter(Column::ExamId.eq(exam_id.to_string()))
        .exec(&txn)
        .await?;

    let mut inserted = Vec::with_capacity(rows.len());
    for row in rows {
        let model = ActiveModel {
            id: Set(row.id),
            exam_id: Set(row.exam_id),
            seq: Set(row.seq),
            r#type: Set(row.r#type),
            content: Set(row.content),
            options: Set(row.options),
            answer: Set(row.answer),
            score: Set(row.score),
            explanation: Set(row.explanation),
        };
        inserted.push(model.insert(&txn).await?);
    }

    txn.commit().await?;
    Ok(inserted)
}