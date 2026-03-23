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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StudentDeviceAssignDto {
    pub student_exam_id: String,
    pub student_id: String,
    pub student_no: String,
    pub student_name: String,
    pub ip_addr: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignStudentDeviceItem {
    pub student_exam_id: String,
    pub ip_addr: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignDevicesToStudentExamsInput {
    pub exam_id: String,
    pub assignments: Vec<AssignStudentDeviceItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectStudentDevicesByExamInput {
    pub exam_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistributeExamPapersByExamInput {
    pub exam_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartExamByExamInput {
    pub exam_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributeExamPapersResultItem {
    pub student_exam_id: String,
    pub student_id: String,
    pub device_ip: String,
    pub success: bool,
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributeExamPapersOutput {
    pub request_id: String,
    pub total: usize,
    pub success_count: usize,
    pub results: Vec<DistributeExamPapersResultItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartExamOutput {
    pub exam_id: String,
    pub total_targets: usize,
    pub sent_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StudentDeviceConnectionStatusDto {
    pub student_exam_id: String,
    pub student_id: String,
    pub student_no: String,
    pub student_name: String,
    pub ip_addr: Option<String>,
    pub device_name: Option<String>,
    pub connection_status: String,
    pub last_heartbeat_at: Option<i64>,
    pub has_heartbeat_seen: bool,
    pub answered_count: i64,
    pub total_questions: i64,
    pub progress_percent: i64,
}
