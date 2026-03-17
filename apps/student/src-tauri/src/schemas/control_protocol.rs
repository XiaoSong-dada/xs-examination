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
