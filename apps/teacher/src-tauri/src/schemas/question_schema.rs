use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuestionDto {
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

#[derive(Debug, Clone, Deserialize)]
pub struct GetQuestionsInput {
    pub exam_id: String,
}
