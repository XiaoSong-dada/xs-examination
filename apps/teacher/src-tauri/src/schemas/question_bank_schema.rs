use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionBankOptionDto {
    pub key: String,
    pub text: String,
    pub option_type: String,
    #[serde(default)]
    pub image_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionBankItemDto {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub content: String,
    #[serde(default)]
    pub content_image_paths: Vec<String>,
    #[serde(default)]
    pub options: Vec<QuestionBankOptionDto>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetQuestionBankItemByIdInput {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateQuestionBankItemInput {
    #[serde(rename = "type")]
    pub r#type: String,
    pub content: String,
    #[serde(default)]
    pub content_image_paths: Vec<String>,
    #[serde(default)]
    pub options: Vec<QuestionBankOptionDto>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateQuestionBankItemInput {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub content: String,
    #[serde(default)]
    pub content_image_paths: Vec<String>,
    #[serde(default)]
    pub options: Vec<QuestionBankOptionDto>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteQuestionBankItemInput {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportQuestionBankPackageInput {
    pub file_name: String,
    pub xlsx_bytes: Vec<u8>,
    #[serde(default)]
    pub image_relative_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportQuestionBankPackageOutput {
    pub path: String,
    pub packed_image_count: usize,
    pub missing_image_count: usize,
}