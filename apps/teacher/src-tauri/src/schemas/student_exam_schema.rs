use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExamStudentDto {
    pub id: String,
    pub student_no: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StudentExamDto {
    pub id: String,
    pub student_id: String,
    pub exam_id: String,
    pub ip_addr: Option<String>,
    pub status: String,
    pub join_time: Option<i64>,
    pub submit_time: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetStudentExamsInput {
    pub exam_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportStudentsByExamIdInput {
    pub exam_id: String,
    pub student_ids: Vec<String>,
}
