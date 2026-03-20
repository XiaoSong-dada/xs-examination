use anyhow::Result;
use sea_orm::DatabaseConnection;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::device::Model as DeviceModel;
use crate::network::student_control_client;
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

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

pub async fn push_teacher_endpoints_to_devices(
    db: &DatabaseConnection,
    payload: device_schema::PushTeacherEndpointsInput,
) -> Result<device_schema::PushTeacherEndpointsOutput> {
    let control_port = payload.control_port.unwrap_or(18889);
    let request_id = uuid::Uuid::new_v4().to_string();

    let mut results = Vec::new();
    let mut seen = HashSet::new();

    for device_id in payload.device_ids {
        let device_id = device_id.trim().to_string();
        if device_id.is_empty() || !seen.insert(device_id.clone()) {
            continue;
        }

        let device = match device_repo::get_device_by_id(db, &device_id).await {
            Ok(device) => device,
            Err(err) => {
                results.push(device_schema::PushTeacherEndpointsResultItem {
                    device_id,
                    device_ip: String::new(),
                    success: false,
                    message: format!("设备查询失败: {}", err),
                    connected_master: None,
                });
                continue;
            }
        };

        let req = student_control_client::ApplyTeacherEndpointsRequest {
            r#type: "APPLY_TEACHER_ENDPOINTS".to_string(),
            request_id: format!("{}-{}", request_id, device.id),
            timestamp: now_ms(),
            payload: student_control_client::ApplyTeacherEndpointsPayload {
                config_version: Some(1),
                session_id: None,
                exam_id: None,
                exam_title: None,
                student_id: device.id.clone(),
                student_no: None,
                student_name: None,
                assigned_ip_addr: None,
                assignment_status: None,
                start_time: None,
                end_time: None,
                endpoints: payload
                    .endpoints
                    .iter()
                    .map(|item| student_control_client::TeacherEndpointInput {
                        id: item.id.clone(),
                        endpoint: item.endpoint.clone(),
                        name: item.name.clone(),
                        remark: item.remark.clone(),
                        is_master: item.is_master,
                    })
                    .collect(),
            },
        };

        match student_control_client::apply_teacher_endpoints(&device.ip, control_port, &req).await {
            Ok(ack) => results.push(device_schema::PushTeacherEndpointsResultItem {
                device_id: device.id,
                device_ip: device.ip,
                success: ack.payload.success,
                message: ack.payload.message,
                connected_master: ack.payload.connected_master,
            }),
            Err(err) => results.push(device_schema::PushTeacherEndpointsResultItem {
                device_id: device.id,
                device_ip: device.ip,
                success: false,
                message: err.to_string(),
                connected_master: None,
            }),
        }
    }

    let success_count = results.iter().filter(|item| item.success).count();

    Ok(device_schema::PushTeacherEndpointsOutput {
        request_id,
        total: results.len(),
        success_count,
        results,
    })
}
