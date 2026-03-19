use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverRequest {
    pub r#type: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverAckPayload {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub ip: String,
    pub name: String,
    #[serde(rename = "controlPort")]
    pub control_port: u16,
    #[serde(rename = "dbReady")]
    pub db_ready: bool,
    #[serde(rename = "appVersion")]
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverAck {
    pub r#type: String,
    pub timestamp: i64,
    pub payload: DiscoverAckPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherEndpointInput {
    pub id: String,
    pub endpoint: String,
    pub name: Option<String>,
    pub remark: Option<String>,
    #[serde(rename = "isMaster")]
    pub is_master: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTeacherEndpointsPayload {
    #[serde(rename = "configVersion")]
    pub config_version: Option<i64>,
    #[serde(rename = "studentId")]
    pub student_id: String,
    pub endpoints: Vec<TeacherEndpointInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTeacherEndpointsRequest {
    pub r#type: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub timestamp: i64,
    pub payload: ApplyTeacherEndpointsPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTeacherEndpointsAckPayload {
    pub success: bool,
    pub message: String,
    #[serde(rename = "connectedMaster")]
    pub connected_master: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTeacherEndpointsAck {
    pub r#type: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub timestamp: i64,
    pub payload: ApplyTeacherEndpointsAckPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributeExamPaperPayload {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "studentNo")]
    pub student_no: String,
    #[serde(rename = "studentName")]
    pub student_name: String,
    #[serde(rename = "assignedIpAddr")]
    pub assigned_ip_addr: String,
    #[serde(rename = "examTitle")]
    pub exam_title: String,
    pub status: String,
    #[serde(rename = "assignmentStatus")]
    pub assignment_status: String,
    #[serde(rename = "startTime")]
    pub start_time: Option<i64>,
    #[serde(rename = "endTime")]
    pub end_time: Option<i64>,
    #[serde(rename = "paperVersion")]
    pub paper_version: Option<String>,
    #[serde(rename = "examMeta")]
    pub exam_meta: String,
    #[serde(rename = "questionsPayload")]
    pub questions_payload: String,
    #[serde(rename = "downloadedAt")]
    pub downloaded_at: i64,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributeExamPaperRequest {
    pub r#type: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub timestamp: i64,
    pub payload: DistributeExamPaperPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributeExamPaperAckPayload {
    pub success: bool,
    pub message: String,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributeExamPaperAck {
    pub r#type: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub timestamp: i64,
    pub payload: DistributeExamPaperAckPayload,
}
