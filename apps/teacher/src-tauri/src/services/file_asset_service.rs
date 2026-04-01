use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use tauri::Manager;

use crate::schemas::file_asset_schema;

const ALLOWED_IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "gif", "webp", "bmp"];

/// 复制本地图片到教师端应用数据目录并返回可持久化的相对路径。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄，用于解析应用数据目录。
/// - `payload`: 图片原始路径与业务目录标识。
///
/// # 返回值
/// - 返回相对路径和最终文件名；源文件不存在、文件类型非法或写入失败时返回错误。
pub fn upload_local_image_asset(
    app_handle: &tauri::AppHandle,
    payload: file_asset_schema::UploadLocalImageInput,
) -> Result<file_asset_schema::UploadLocalImageOutput> {
    let source_path = PathBuf::from(payload.source_path.trim());
    if !source_path.exists() {
        return Err(anyhow!("图片文件不存在"));
    }

    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if !ALLOWED_IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        return Err(anyhow!("仅支持 png/jpg/jpeg/gif/webp/bmp 图片"));
    }

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .context("解析应用数据目录失败")?;

    let biz_segments = sanitize_biz_segments(payload.biz);
    let mut target_dir = app_data_dir.join("uploads").join("images");
    for segment in &biz_segments {
        target_dir = target_dir.join(segment);
    }

    std::fs::create_dir_all(&target_dir).context("创建图片目录失败")?;

    let original_stem = source_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("image");
    let safe_stem = sanitize_file_name(original_stem);
    let file_name = format!("{}_{}.{}", safe_stem, uuid::Uuid::new_v4().simple(), extension);

    let target_path = target_dir.join(&file_name);
    std::fs::copy(&source_path, &target_path).context("复制图片文件失败")?;

    let mut relative_segments = vec!["uploads".to_string(), "images".to_string()];
    relative_segments.extend(biz_segments);
    relative_segments.push(file_name.clone());

    Ok(file_asset_schema::UploadLocalImageOutput {
        relative_path: relative_segments.join("/"),
        file_name,
    })
}

fn sanitize_biz_segments(raw_biz: String) -> Vec<String> {
    let cleaned = raw_biz.trim();
    if cleaned.is_empty() {
        return vec!["question-bank".to_string()];
    }

    let mut segments: Vec<String> = cleaned
        .replace('\\', "/")
        .split('/')
        .filter_map(|segment| sanitize_segment(segment))
        .collect();

    if segments.is_empty() {
        segments.push("question-bank".to_string());
    }

    segments
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

fn sanitize_file_name(raw: &str) -> String {
    let sanitized = sanitize_segment(raw).unwrap_or_else(|| "image".to_string());
    if sanitized.len() > 64 {
        return sanitized.chars().take(64).collect();
    }
    sanitized
}
