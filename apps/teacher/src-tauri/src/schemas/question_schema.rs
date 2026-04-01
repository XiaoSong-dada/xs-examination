use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuestionDto {
    pub id: String,
    pub exam_id: String,
    pub seq: i32,
    pub r#type: String,
    pub content: String,
    pub content_image_paths: Option<String>,
    pub options: Option<String>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetQuestionsInput {
    pub exam_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuestionImportItemInput {
    pub id: Option<String>,
    pub seq: i32,
    #[serde(rename = "type")]
    pub r#type: String,
    pub content: String,
    pub content_image_paths: Option<String>,
    pub options: Option<String>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportQuestionsInput {
    pub exam_id: String,
    pub questions: Vec<QuestionImportItemInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportQuestionPackageInput {
    pub exam_id: String,
    pub package_path: String,
}
