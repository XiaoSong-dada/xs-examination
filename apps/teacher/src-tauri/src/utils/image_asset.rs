use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ImageAssetMeta {
    pub file_path: PathBuf,
    pub file_name: String,
    pub mime_type: String,
    pub byte_size: u64,
    pub sha256: String,
}

/// 计算二进制内容的 SHA-256 十六进制摘要。
///
/// # 参数
/// * `bytes` - 待计算摘要的二进制内容。
///
/// # 返回值
/// 返回小写十六进制字符串。
pub fn compute_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    format!("{:x}", digest)
}

/// 按扩展名推断图片 MIME 类型。
///
/// # 参数
/// * `path` - 图片文件路径。
///
/// # 返回值
/// 返回推断得到的 MIME 类型，未知时回退 `application/octet-stream`。
pub fn detect_mime_type(path: &Path) -> String {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|v| v.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png".to_string(),
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("webp") => "image/webp".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("bmp") => "image/bmp".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

/// 读取图片文件并返回二进制与元信息。
///
/// # 参数
/// * `path` - 图片文件路径。
///
/// # 返回值
/// 返回 `(bytes, meta)`，其中 `meta` 包含文件名、大小、MIME 和 SHA-256。
pub fn read_image_asset(path: &Path) -> Result<(Vec<u8>, ImageAssetMeta)> {
    let bytes = fs::read(path)
        .with_context(|| format!("读取图片文件失败: {}", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_string();

    let meta = ImageAssetMeta {
        file_path: path.to_path_buf(),
        file_name,
        mime_type: detect_mime_type(path),
        byte_size: bytes.len() as u64,
        sha256: compute_sha256_hex(&bytes),
    };

    Ok((bytes, meta))
}

/// 将二进制按固定大小切片并编码为 base64 文本片段。
///
/// # 参数
/// * `bytes` - 原始二进制。
/// * `chunk_size` - 单片最大字节数，建议不小于 1KB。
///
/// # 返回值
/// 返回按顺序排列的 base64 片段数组。
pub fn split_bytes_to_base64_chunks(bytes: &[u8], chunk_size: usize) -> Vec<String> {
    if bytes.is_empty() {
        return Vec::new();
    }

    let safe_chunk_size = chunk_size.max(1);
    bytes
        .chunks(safe_chunk_size)
        .map(|part| STANDARD.encode(part))
        .collect()
}

/// 解码单个 base64 片段为二进制。
///
/// # 参数
/// * `content_base64` - base64 文本片段。
///
/// # 返回值
/// 返回解码后的二进制。
pub fn decode_base64_chunk(content_base64: &str) -> Result<Vec<u8>> {
    STANDARD
        .decode(content_base64)
        .context("base64 片段解码失败")
}

/// 合并并解码多个 base64 片段。
///
/// # 参数
/// * `chunks` - 按顺序排列的 base64 片段数组。
///
/// # 返回值
/// 返回拼接后的完整二进制。
pub fn assemble_base64_chunks(chunks: &[String]) -> Result<Vec<u8>> {
    let mut merged = Vec::new();
    for chunk in chunks {
        merged.extend(decode_base64_chunk(chunk)?);
    }
    Ok(merged)
}

/// 将图片二进制写入目标路径，必要时自动创建父目录。
///
/// # 参数
/// * `output_path` - 输出文件路径。
/// * `bytes` - 待写入的二进制内容。
///
/// # 返回值
/// 成功返回 `Ok(())`；失败时返回带上下文的错误。
pub fn write_asset_bytes(output_path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建目录失败: {}", parent.display()))?;
    }

    fs::write(output_path, bytes)
        .with_context(|| format!("写入图片文件失败: {}", output_path.display()))
}
