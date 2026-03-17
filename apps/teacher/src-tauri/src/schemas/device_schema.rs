use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeviceDto {
    pub id: String,
    pub ip: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDevicesInput {
    pub ip: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDeviceByIdInput {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDeviceInput {
    pub ip: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDeviceInput {
    pub id: String,
    pub ip: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteDeviceInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDeviceDto {
    pub ip: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReplaceDevicesInput {
    pub devices: Vec<DiscoveredDeviceDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherEndpointInputDto {
    pub id: String,
    pub endpoint: String,
    pub name: Option<String>,
    pub remark: Option<String>,
    #[serde(rename = "isMaster")]
    pub is_master: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushTeacherEndpointsInput {
    #[serde(rename = "deviceIds")]
    pub device_ids: Vec<String>,
    pub endpoints: Vec<TeacherEndpointInputDto>,
    #[serde(rename = "controlPort")]
    pub control_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushTeacherEndpointsResultItem {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceIp")]
    pub device_ip: String,
    pub success: bool,
    pub message: String,
    #[serde(rename = "connectedMaster")]
    pub connected_master: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushTeacherEndpointsOutput {
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub total: usize,
    #[serde(rename = "successCount")]
    pub success_count: usize,
    pub results: Vec<PushTeacherEndpointsResultItem>,
}
