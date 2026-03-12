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
