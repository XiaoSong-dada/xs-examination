use anyhow::{anyhow, Context, Result};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Clone)]
pub struct ZipAssetEntry {
    pub source_path: PathBuf,
    pub archive_path: String,
}

#[derive(Debug, Clone)]
pub struct ExtractedZipEntry {
    pub archive_path: String,
    pub output_path: PathBuf,
    pub byte_size: u64,
}

fn normalize_archive_path(raw: &str) -> String {
    raw.replace('\\', "/").trim_start_matches('/').to_string()
}

/// 将指定资源文件打包为 zip。
///
/// # 参数
/// * `output_zip_path` - 输出 zip 文件路径。
/// * `entries` - 参与打包的资源条目。
///
/// # 返回值
/// 返回成功写入的文件条目数量。
pub fn create_asset_zip(output_zip_path: &Path, entries: &[ZipAssetEntry]) -> Result<usize> {
    if entries.is_empty() {
        return Ok(0);
    }

    if let Some(parent) = output_zip_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建 zip 输出目录失败: {}", parent.display()))?;
    }

    let file = File::create(output_zip_path)
        .with_context(|| format!("创建 zip 文件失败: {}", output_zip_path.display()))?;
    let mut writer = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let mut written = 0usize;
    for entry in entries {
        let archive_path = normalize_archive_path(&entry.archive_path);
        if archive_path.is_empty() {
            return Err(anyhow!("zip 条目 archive_path 不能为空"));
        }

        let mut source = File::open(&entry.source_path).with_context(|| {
            format!("打开待打包资源失败: {}", entry.source_path.display())
        })?;

        writer
            .start_file(archive_path, options)
            .context("写入 zip 条目头失败")?;

        let mut buf = Vec::new();
        source
            .read_to_end(&mut buf)
            .with_context(|| format!("读取资源文件失败: {}", entry.source_path.display()))?;
        writer.write_all(&buf).context("写入 zip 条目内容失败")?;
        written += 1;
    }

    writer.finish().context("结束 zip 写入失败")?;
    Ok(written)
}

/// 解压资源 zip 到目标目录，并返回解压后的文件列表。
///
/// # 参数
/// * `zip_path` - zip 文件路径。
/// * `destination_dir` - 解压目标目录。
///
/// # 返回值
/// 返回每个解压文件的归档路径、输出路径和大小。
pub fn extract_asset_zip(zip_path: &Path, destination_dir: &Path) -> Result<Vec<ExtractedZipEntry>> {
    fs::create_dir_all(destination_dir)
        .with_context(|| format!("创建解压目录失败: {}", destination_dir.display()))?;

    let file = File::open(zip_path)
        .with_context(|| format!("打开 zip 文件失败: {}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file).context("读取 zip 结构失败")?;

    let mut extracted = Vec::new();
    for idx in 0..archive.len() {
        let mut zipped_file = archive.by_index(idx).context("读取 zip 条目失败")?;
        let enclosed = zipped_file.enclosed_name().ok_or_else(|| {
            anyhow!("zip 条目路径非法，可能存在目录穿越: {}", zipped_file.name())
        })?;

        let output_path = destination_dir.join(enclosed);
        if zipped_file.is_dir() {
            fs::create_dir_all(&output_path)
                .with_context(|| format!("创建目录失败: {}", output_path.display()))?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建父目录失败: {}", parent.display()))?;
        }

        let mut out = File::create(&output_path)
            .with_context(|| format!("创建解压文件失败: {}", output_path.display()))?;
        std::io::copy(&mut zipped_file, &mut out)
            .with_context(|| format!("写入解压文件失败: {}", output_path.display()))?;

        extracted.push(ExtractedZipEntry {
            archive_path: zipped_file.name().to_string(),
            output_path,
            byte_size: zipped_file.size(),
        });
    }

    Ok(extracted)
}
