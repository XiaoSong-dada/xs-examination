use anyhow::Result;
use sea_orm::DatabaseConnection;
use std::collections::HashSet;

use crate::models::device::Model as DeviceModel;
use crate::repos::device_repo;
use crate::schemas::device_schema;
use crate::utils::lan_discovery;

#[derive(Debug, Clone)]
pub struct DeviceWritePayload {
    pub ip: String,
    pub name: String,
}

impl DeviceWritePayload {
    pub fn normalized(ip: String, name: String) -> Self {
        Self {
            ip: ip.trim().to_string(),
            name: name.trim().to_string(),
        }
    }
}

pub async fn list_devices(
    db: &DatabaseConnection,
    ip: Option<String>,
    name: Option<String>,
) -> Result<Vec<DeviceModel>> {
    device_repo::get_all_devices(db, ip.as_deref(), name.as_deref()).await
}

pub async fn get_device_by_id(db: &DatabaseConnection, id: String) -> Result<DeviceModel> {
    device_repo::get_device_by_id(db, &id).await
}

pub async fn create_device(
    db: &DatabaseConnection,
    payload: DeviceWritePayload,
) -> Result<DeviceModel> {
    let id = uuid::Uuid::new_v4().to_string();
    device_repo::insert_device(db, id, payload).await
}

pub async fn update_device(
    db: &DatabaseConnection,
    id: String,
    payload: DeviceWritePayload,
) -> Result<DeviceModel> {
    device_repo::update_device_by_id(db, &id, payload).await
}

pub async fn delete_device(db: &DatabaseConnection, id: String) -> Result<()> {
    device_repo::delete_device_by_id(db, &id).await
}

pub async fn discover_student_devices() -> Result<Vec<device_schema::DiscoveredDeviceDto>> {
    let ips = lan_discovery::discover_active_ips().await?;
    Ok(ips
        .into_iter()
        .map(|ip| device_schema::DiscoveredDeviceDto { ip })
        .collect())
}

pub async fn replace_devices_by_discovery(
    db: &DatabaseConnection,
    devices: Vec<device_schema::DiscoveredDeviceDto>,
) -> Result<Vec<DeviceModel>> {
    let mut seen = HashSet::new();
    let mut ips = Vec::new();

    for item in devices {
        let ip = item.ip.trim().to_string();
        if ip.is_empty() {
            continue;
        }
        if seen.insert(ip.clone()) {
            ips.push(ip);
        }
    }

    device_repo::replace_devices_by_ips(db, ips).await
}
