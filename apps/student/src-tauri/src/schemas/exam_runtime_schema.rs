use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSessionDto {
    pub id: String,
    pub exam_id: String,
    pub student_id: String,
    pub student_no: String,
    pub student_name: String,
    pub assigned_ip_addr: String,
    pub assigned_device_name: Option<String>,
    pub exam_title: String,
    pub status: String,
    pub assignment_status: String,
    pub started_at: Option<i64>,
    pub ends_at: Option<i64>,
    pub paper_version: Option<String>,
    pub last_synced_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSnapshotDto {
    pub session_id: String,
    pub exam_meta: String,
    pub questions_payload: String,
    pub downloaded_at: i64,
    pub expires_at: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentExamBundleDto {
    pub session: Option<ExamSessionDto>,
    pub snapshot: Option<ExamSnapshotDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalAnswerDto {
    pub question_id: String,
    pub answer: String,
    pub revision: i64,
    pub updated_at: i64,
}
