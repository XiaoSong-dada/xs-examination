use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherRuntimeStatusDto {
    pub endpoint: Option<String>,
    #[serde(rename = "connectionStatus")]
    pub connection_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherEndpointAppliedEvent {
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsConnectionEvent {
    pub endpoint: Option<String>,
    pub connected: bool,
    pub message: Option<String>,
}
