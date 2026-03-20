use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRuntimeStatusDto {
    pub ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIpUpdatedEvent {
    pub ip: Option<String>,
}
