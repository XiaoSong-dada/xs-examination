use tauri::State;

use crate::schemas::device_schema;
use crate::services::device_service;
use crate::services::device_service::DeviceWritePayload;
use crate::state::AppState;

#[tauri::command]
pub async fn get_devices(
    state: State<'_, AppState>,
    payload: Option<device_schema::GetDevicesInput>,
) -> Result<Vec<device_schema::DeviceDto>, String> {
    let pool = &state.db;
    let input = payload.unwrap_or(device_schema::GetDevicesInput {
        ip: None,
        name: None,
    });

    match device_service::list_devices(pool, input.ip, input.name).await {
        Ok(list) => Ok(list
            .into_iter()
            .map(|item| device_schema::DeviceDto {
                id: item.id,
                ip: item.ip,
                name: item.name,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn get_device_by_id(
    state: State<'_, AppState>,
    payload: device_schema::GetDeviceByIdInput,
) -> Result<device_schema::DeviceDto, String> {
    let pool = &state.db;
    match device_service::get_device_by_id(pool, payload.id).await {
        Ok(item) => Ok(device_schema::DeviceDto {
            id: item.id,
            ip: item.ip,
            name: item.name,
        }),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn create_device(
    state: State<'_, AppState>,
    payload: device_schema::CreateDeviceInput,
) -> Result<device_schema::DeviceDto, String> {
    let pool = &state.db;
    let write_payload = DeviceWritePayload::normalized(payload.ip, payload.name);

    match device_service::create_device(pool, write_payload).await {
        Ok(item) => Ok(device_schema::DeviceDto {
            id: item.id,
            ip: item.ip,
            name: item.name,
        }),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn update_device(
    state: State<'_, AppState>,
    payload: device_schema::UpdateDeviceInput,
) -> Result<device_schema::DeviceDto, String> {
    let pool = &state.db;
    let id = payload.id;
    let write_payload = DeviceWritePayload::normalized(payload.ip, payload.name);

    match device_service::update_device(pool, id, write_payload).await {
        Ok(item) => Ok(device_schema::DeviceDto {
            id: item.id,
            ip: item.ip,
            name: item.name,
        }),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn delete_device(
    state: State<'_, AppState>,
    payload: device_schema::DeleteDeviceInput,
) -> Result<(), String> {
    let pool = &state.db;
    device_service::delete_device(pool, payload.id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn discover_student_devices() -> Result<Vec<device_schema::DiscoveredDeviceDto>, String> {
    device_service::discover_student_devices()
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn replace_devices_by_discovery(
    state: State<'_, AppState>,
    payload: device_schema::ReplaceDevicesInput,
) -> Result<Vec<device_schema::DeviceDto>, String> {
    let pool = &state.db;
    match device_service::replace_devices_by_discovery(pool, payload.devices).await {
        Ok(list) => Ok(list
            .into_iter()
            .map(|item| device_schema::DeviceDto {
                id: item.id,
                ip: item.ip,
                name: item.name,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn push_teacher_endpoints_to_devices(
    state: State<'_, AppState>,
    payload: device_schema::PushTeacherEndpointsInput,
) -> Result<device_schema::PushTeacherEndpointsOutput, String> {
    let pool = &state.db;
    device_service::push_teacher_endpoints_to_devices(pool, payload)
        .await
        .map_err(|err| err.to_string())
}
