use crate::schemas::file_asset_schema;
use crate::services::file_asset_service;

/// 将教师端本地图片复制到应用规范目录，并返回相对路径和文件名。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄，用于解析应用数据目录。
/// - `payload`: 包含源文件路径与业务目录标识。
///
/// # 返回值
/// - 返回复制后的相对路径和文件名；失败时返回错误字符串。
#[tauri::command]
pub async fn upload_local_image_asset(
    app_handle: tauri::AppHandle,
    payload: file_asset_schema::UploadLocalImageInput,
) -> Result<file_asset_schema::UploadLocalImageOutput, String> {
    file_asset_service::upload_local_image_asset(&app_handle, payload).map_err(|err| err.to_string())
}
