use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExamSession {
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
    pub encryption_nonce: Option<Vec<u8>>,
    pub last_synced_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExamSnapshot {
    pub session_id: String,
    pub exam_meta: Vec<u8>,
    pub questions_payload: Vec<u8>,
    pub downloaded_at: i64,
    pub expires_at: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LocalAnswer {
    pub id: String,
    pub session_id: String,
    pub question_id: String,
    pub answer: Option<String>,
    pub answer_blob: Option<Vec<u8>>,
    pub revision: i64,
    pub sync_status: String,
    pub last_synced_at: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SyncOutboxItem {
    pub id: i64,
    pub session_id: String,
    pub event_type: String,
    pub aggregate_id: Option<String>,
    pub payload: Vec<u8>,
    pub status: String,
    pub retry_count: i64,
    pub next_retry_at: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RuntimeKv {
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}