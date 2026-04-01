use anyhow::{anyhow, Context, Result};
use base64::Engine;
use tauri::Manager;

use crate::schemas::file_asset_schema;

/// 将学生端已落盘图片的相对路径解析为可直接渲染的 data URL。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄，用于解析应用数据目录。
/// - `payload`: 包含图片相对路径。
///
/// # 返回值
/// - 返回相对路径与预览地址；路径非法、文件不存在或读取失败时返回错误。
pub fn resolve_image_asset_preview(
    app_handle: &tauri::AppHandle,
    payload: file_asset_schema::ResolveImageAssetPreviewInput,
) -> Result<file_asset_schema::ResolveImageAssetPreviewOutput> {
    let relative_segments = sanitize_relative_path(payload.relative_path);
    if relative_segments.len() < 3 {
        return Err(anyhow!("图片相对路径不合法"));
    }

    if relative_segments[0] != "uploads" || relative_segments[1] != "images" {
        return Err(anyhow!("图片相对路径不在允许目录中"));
    }

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .context("解析应用数据目录失败")?;

    let file_path = relative_segments
        .iter()
        .fold(app_data_dir, |acc, segment| acc.join(segment));

    if !file_path.exists() {
        return Err(anyhow!("图片文件不存在"));
    }

    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let mime = infer_image_mime(&extension).ok_or_else(|| anyhow!("不支持的图片类型"))?;
    let bytes = std::fs::read(&file_path).context("读取图片文件失败")?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let preview_url = format!("data:{};base64,{}", mime, encoded);

    Ok(file_asset_schema::ResolveImageAssetPreviewOutput {
        relative_path: relative_segments.join("/"),
        preview_url,
    })
}

fn sanitize_relative_path(raw_relative_path: String) -> Vec<String> {
    raw_relative_path
        .trim()
        .replace('\\', "/")
        .split('/')
        .filter_map(sanitize_segment)
        .collect()
}

fn sanitize_segment(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        return None;
    }

    let normalized = trimmed
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn infer_image_mime(extension: &str) -> Option<&'static str> {
    match extension {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        _ => None,
    }
}
