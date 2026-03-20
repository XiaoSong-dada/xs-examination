use tauri::Emitter;

use crate::schemas::device_schema::{DeviceIpUpdatedEvent, DeviceRuntimeStatusDto};
use crate::services::device_service::DeviceService;

#[tauri::command]
pub async fn get_device_runtime_status(
    app_handle: tauri::AppHandle,
) -> Result<DeviceRuntimeStatusDto, String> {
    let runtime = DeviceService::get_runtime_status().map_err(|e| e.to_string())?;

    let _ = app_handle.emit(
        "device_ip_updated",
        DeviceIpUpdatedEvent {
            ip: runtime.ip.clone(),
        },
    );

    Ok(runtime)
}
