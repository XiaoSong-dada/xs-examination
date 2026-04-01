use crate::schemas::file_asset_schema;
use crate::services::file_asset_service;

/// 将学生端图片相对路径解析为可在前端直接显示的预览地址。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄，用于解析应用数据目录。
/// - `payload`: 包含图片相对路径。
///
/// # 返回值
/// - 返回相对路径对应的 data URL；失败时返回错误字符串。
#[tauri::command]
pub async fn resolve_image_asset_preview(
    app_handle: tauri::AppHandle,
    payload: file_asset_schema::ResolveImageAssetPreviewInput,
) -> Result<file_asset_schema::ResolveImageAssetPreviewOutput, String> {
    file_asset_service::resolve_image_asset_preview(&app_handle, payload)
        .map_err(|err| err.to_string())
}
