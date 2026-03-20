use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::network::transport::tcp_request_reply::{RequestReplyTimeouts, send_json_request};

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
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(rename = "examId")]
    pub exam_id: Option<String>,
    #[serde(rename = "examTitle")]
    pub exam_title: Option<String>,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "studentNo")]
    pub student_no: Option<String>,
    #[serde(rename = "studentName")]
    pub student_name: Option<String>,
    #[serde(rename = "assignedIpAddr")]
    pub assigned_ip_addr: Option<String>,
    #[serde(rename = "assignmentStatus")]
    pub assignment_status: Option<String>,
    #[serde(rename = "startTime")]
    pub start_time: Option<i64>,
    #[serde(rename = "endTime")]
    pub end_time: Option<i64>,
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
pub struct DistributeExamQuestionItem {
    pub id: String,
    pub seq: i32,
    #[serde(rename = "type")]
    pub r#type: String,
    pub content: String,
    pub options: Option<String>,
    pub score: i32,
    pub explanation: Option<String>,
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

pub async fn apply_teacher_endpoints(
    device_ip: &str,
    control_port: u16,
    request: &ApplyTeacherEndpointsRequest,
) -> Result<ApplyTeacherEndpointsAck> {
    let addr = format!("{}:{}", device_ip, control_port);
    send_json_request(
        &addr,
        request,
        RequestReplyTimeouts::apply_teacher_endpoints(),
        "apply_teacher_endpoints",
    )
    .await
}

pub async fn distribute_exam_paper(
    device_ip: &str,
    control_port: u16,
    request: &DistributeExamPaperRequest,
) -> Result<DistributeExamPaperAck> {
    let addr = format!("{}:{}", device_ip, control_port);
    send_json_request(
        &addr,
        request,
        RequestReplyTimeouts::distribute_exam_paper(),
        "distribute_exam_paper",
    )
    .await
}
