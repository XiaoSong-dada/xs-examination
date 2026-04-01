use anyhow::{anyhow, Context, Result};
use calamine::{open_workbook_auto, Reader};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::models::question_bank_item::Model as QuestionBankItemModel;
use crate::repos::question_bank_repo;
use crate::utils::asset_zip::{create_asset_zip, extract_asset_zip, ZipAssetEntry};

#[derive(Debug, Clone)]
pub struct ExportQuestionBankPackageResult {
    pub output_path: String,
    pub packed_image_count: usize,
    pub missing_image_count: usize,
}

#[derive(Debug, Clone)]
pub struct ImportQuestionBankPackageResult {
    pub imported_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionBankOptionValue {
    pub key: String,
    pub text: String,
    pub option_type: String,
    pub image_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct QuestionBankItemView {
    pub id: String,
    pub r#type: String,
    pub content: String,
    pub content_image_paths: Vec<String>,
    pub options: Vec<QuestionBankOptionValue>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct QuestionBankWritePayload {
    pub r#type: String,
    pub content: String,
    pub content_image_paths: Vec<String>,
    pub options: Vec<QuestionBankOptionValue>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl QuestionBankWritePayload {
    pub fn normalized(
        r#type: String,
        content: String,
        content_image_paths: Vec<String>,
        options: Vec<QuestionBankOptionValue>,
        answer: String,
        score: i32,
        explanation: Option<String>,
        created_at: Option<i64>,
        updated_at: Option<i64>,
    ) -> Self {
        Self {
            r#type: r#type.trim().to_string(),
            content: content.trim().to_string(),
            content_image_paths: normalize_paths(content_image_paths),
            options: options
                .into_iter()
                .map(|item| QuestionBankOptionValue {
                    key: item.key.trim().to_string(),
                    text: item.text.trim().to_string(),
                    option_type: normalize_option_type(item.option_type),
                    image_paths: normalize_paths(item.image_paths),
                })
                .filter(|item| {
                    !item.key.is_empty() || !item.text.is_empty() || !item.image_paths.is_empty()
                })
                .collect(),
            answer: answer.trim().to_string(),
            score,
            explanation: explanation
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            created_at,
            updated_at,
        }
    }

    pub fn serialized_content_image_paths(&self) -> Result<Option<String>> {
        serialize_string_vec(&self.content_image_paths)
    }

    pub fn serialized_options(&self) -> Result<Option<String>> {
        if self.options.is_empty() {
            return Ok(None);
        }
        Ok(Some(serde_json::to_string(&self.options)?))
    }
}

/// 查询全部全局题库题目并转为前端可消费结构。
///
/// # 参数
/// - `db`: 数据库连接。
///
/// # 返回值
/// - 返回解析后的题库题目数组；数据库查询或 JSON 解析失败时返回错误。
pub async fn list_question_bank_items(
    db: &DatabaseConnection,
) -> Result<Vec<QuestionBankItemView>> {
    let items = question_bank_repo::get_all_question_bank_items(db).await?;
    items
        .into_iter()
        .map(materialize_question_bank_item)
        .collect()
}

/// 按 ID 获取单条全局题库题目。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `id`: 题目 ID。
///
/// # 返回值
/// - 返回解析后的题库题目；数据库查询或 JSON 解析失败时返回错误。
pub async fn get_question_bank_item_by_id(
    db: &DatabaseConnection,
    id: String,
) -> Result<QuestionBankItemView> {
    let item = question_bank_repo::get_question_bank_item_by_id(db, &id).await?;
    materialize_question_bank_item(item)
}

/// 新增一条全局题库题目。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `payload`: 题目写入载荷。
///
/// # 返回值
/// - 返回新增后的题目详情；字段校验、数据库写入或 JSON 解析失败时返回错误。
pub async fn create_question_bank_item(
    db: &DatabaseConnection,
    payload: QuestionBankWritePayload,
) -> Result<QuestionBankItemView> {
    validate_question_bank_payload(&payload)?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();
    let created_at = payload.created_at.unwrap_or(now);
    let updated_at = payload.updated_at.unwrap_or(now);
    let item =
        question_bank_repo::insert_question_bank_item(db, id, &payload, created_at, updated_at)
            .await?;
    materialize_question_bank_item(item)
}

/// 更新一条全局题库题目。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `id`: 题目 ID。
/// - `payload`: 最新题目写入载荷。
///
/// # 返回值
/// - 返回更新后的题目详情；字段校验、数据库写入或 JSON 解析失败时返回错误。
pub async fn update_question_bank_item(
    db: &DatabaseConnection,
    id: String,
    payload: QuestionBankWritePayload,
) -> Result<QuestionBankItemView> {
    validate_question_bank_payload(&payload)?;

    let now = Utc::now().timestamp_millis();
    let updated_at = payload.updated_at.unwrap_or(now);
    let item =
        question_bank_repo::update_question_bank_item_by_id(db, &id, &payload, updated_at).await?;
    materialize_question_bank_item(item)
}

/// 删除一条全局题库题目。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `id`: 题目 ID。
///
/// # 返回值
/// - 删除成功返回 `()`；题目不存在或数据库删除失败时返回错误。
pub async fn delete_question_bank_item(db: &DatabaseConnection, id: String) -> Result<()> {
    question_bank_repo::delete_question_bank_item_by_id(db, &id).await
}

/// 将题库导出的 xlsx 与图片资源打包为 zip 文件并写入本机下载目录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄，用于解析应用数据目录与下载目录。
/// - `file_name`: 导出文件名（会自动规范化为 `.zip` 后缀）。
/// - `xlsx_bytes`: 前端生成的 xlsx 二进制字节。
/// - `image_relative_paths`: 需要打包的图片相对路径列表。
///
/// # 返回值
/// - 返回导出包路径和资源统计；写盘、打包或目录解析失败时返回错误。
pub fn export_question_bank_package(
    app_handle: &tauri::AppHandle,
    file_name: String,
    xlsx_bytes: Vec<u8>,
    image_relative_paths: Vec<String>,
) -> Result<ExportQuestionBankPackageResult> {
    if xlsx_bytes.is_empty() {
        return Err(anyhow!("导出失败：xlsx 内容为空"));
    }

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .context("解析应用数据目录失败")?;

    let temp_dir = app_data_dir
        .join("temp")
        .join(format!("question-bank-export-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&temp_dir).context("创建导出临时目录失败")?;

    let xlsx_path = temp_dir.join("question_bank.xlsx");
    std::fs::write(&xlsx_path, xlsx_bytes).context("写入导出 xlsx 失败")?;

    let mut entries = vec![ZipAssetEntry {
        source_path: xlsx_path,
        archive_path: "question_bank.xlsx".to_string(),
    }];

    let mut packed_image_count = 0usize;
    let mut missing_image_count = 0usize;
    for relative_path in normalize_paths(image_relative_paths) {
        let source_path = relative_path
            .split('/')
            .filter(|segment| !segment.trim().is_empty())
            .fold(app_data_dir.clone(), |acc, segment| acc.join(segment));
        if !source_path.exists() {
            missing_image_count += 1;
            continue;
        }

        let archive_path = to_package_asset_path(&relative_path)?;
        entries.push(ZipAssetEntry {
            source_path,
            archive_path,
        });
        packed_image_count += 1;
    }

    let output_file_name = normalize_zip_file_name(file_name);
    let output_dir = app_handle
        .path()
        .download_dir()
        .or_else(|_| app_handle.path().document_dir())
        .unwrap_or(app_data_dir.clone());
    std::fs::create_dir_all(&output_dir).context("创建导出目录失败")?;
    let output_path = output_dir.join(output_file_name);

    create_asset_zip(&output_path, &entries).context("打包题库资源失败")?;

    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(ExportQuestionBankPackageResult {
        output_path: output_path.to_string_lossy().to_string(),
        packed_image_count,
        missing_image_count,
    })
}

/// 清空题库表后导入资源包中的题目数据。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `app_handle`: Tauri 应用句柄，用于解析应用数据目录。
/// - `package_path`: 题库资源包绝对路径。
///
/// # 返回值
/// - 返回导入条数；解压、解析、清空或写入失败时返回错误。
pub async fn import_question_bank_package(
    db: &DatabaseConnection,
    app_handle: &tauri::AppHandle,
    package_path: String,
) -> Result<ImportQuestionBankPackageResult> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .context("解析应用数据目录失败")?;
    let temp_dir = app_data_dir
        .join("temp")
        .join(format!("question-bank-import-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&temp_dir).context("创建资源包解压目录失败")?;

    let extracted = extract_asset_zip(std::path::Path::new(package_path.trim()), &temp_dir)
        .context("解压资源包失败")?;
    let xlsx_path = extracted
        .iter()
        .find_map(|entry| {
            let lowered = entry.archive_path.to_ascii_lowercase();
            if lowered == "question_bank.xlsx" || lowered.ends_with(".xlsx") {
                Some(entry.output_path.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("资源包中未找到 question_bank.xlsx"))?;

    let asset_mapping = materialize_package_assets(&app_data_dir, &extracted)?;
    let payloads = parse_question_bank_payloads_from_xlsx(&xlsx_path, &asset_mapping)?;

    question_bank_repo::delete_all_question_bank_items(db)
        .await
        .context("清空题库表失败")?;

    let mut imported_count = 0usize;
    for payload in payloads {
        validate_question_bank_payload(&payload)?;
        let now = Utc::now().timestamp_millis();
        let created_at = payload.created_at.unwrap_or(now);
        let updated_at = payload.updated_at.unwrap_or(now);
        let id = uuid::Uuid::new_v4().to_string();
        question_bank_repo::insert_question_bank_item(db, id, &payload, created_at, updated_at)
            .await
            .context("写入题库题目失败")?;
        imported_count += 1;
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(ImportQuestionBankPackageResult { imported_count })
}

fn materialize_question_bank_item(item: QuestionBankItemModel) -> Result<QuestionBankItemView> {
    Ok(QuestionBankItemView {
        id: item.id,
        r#type: item.r#type,
        content: item.content,
        content_image_paths: deserialize_string_vec(item.content_image_paths)?,
        options: deserialize_options(item.options)?,
        answer: item.answer,
        score: item.score,
        explanation: item.explanation,
        created_at: item.created_at,
        updated_at: item.updated_at,
    })
}

fn validate_question_bank_payload(payload: &QuestionBankWritePayload) -> Result<()> {
    if payload.r#type.is_empty() {
        return Err(anyhow!("题目类型不能为空"));
    }

    if payload.content.is_empty() {
        return Err(anyhow!("题目内容不能为空"));
    }

    if payload.answer.is_empty() {
        return Err(anyhow!("题目答案不能为空"));
    }

    if payload.score < 0 {
        return Err(anyhow!("题目分值不能小于 0"));
    }

    Ok(())
}

fn normalize_paths(paths: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for path in paths {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value = trimmed.to_string();
        if !normalized.contains(&value) {
            normalized.push(value);
        }
    }
    normalized
}

fn normalize_option_type(option_type: String) -> String {
    let trimmed = option_type.trim();
    if trimmed.is_empty() {
        return "text".to_string();
    }
    trimmed.to_string()
}

fn serialize_string_vec(values: &[String]) -> Result<Option<String>> {
    if values.is_empty() {
        return Ok(None);
    }
    Ok(Some(serde_json::to_string(values)?))
}

fn deserialize_string_vec(value: Option<String>) -> Result<Vec<String>> {
    match value {
        Some(raw) if !raw.trim().is_empty() => Ok(serde_json::from_str(&raw)?),
        _ => Ok(Vec::new()),
    }
}

fn deserialize_options(value: Option<String>) -> Result<Vec<QuestionBankOptionValue>> {
    match value {
        Some(raw) if !raw.trim().is_empty() => Ok(serde_json::from_str(&raw)?),
        _ => Ok(Vec::new()),
    }
}

fn normalize_zip_file_name(raw_name: String) -> String {
    let trimmed = raw_name.trim();
    let fallback = format!(
        "question-bank-export-{}.zip",
        Utc::now().format("%Y%m%d-%H%M%S")
    );
    if trimmed.is_empty() {
        return fallback;
    }

    let mut sanitized = trimmed
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>();

    if sanitized.is_empty() {
        return fallback;
    }

    if !sanitized.to_ascii_lowercase().ends_with(".zip") {
        sanitized.push_str(".zip");
    }

    sanitized
}

fn to_package_asset_path(relative_path: &str) -> Result<String> {
    let normalized = relative_path.trim().replace('\\', "/");
    if normalized.is_empty() {
        return Err(anyhow!("图片相对路径为空"));
    }

    let file_name = normalized
        .split('/')
        .filter(|segment| !segment.is_empty())
        .next_back()
        .ok_or_else(|| anyhow!("图片相对路径格式不合法"))?
        .to_string();

    if normalized.contains("/question-bank/content/") {
        return Ok(format!("assets/content/{}", file_name));
    }

    if normalized.contains("/question-bank/options/") {
        return Ok(format!("assets/options/{}", file_name));
    }

    Ok(format!("assets/options/{}", file_name))
}

fn materialize_package_assets(
    app_data_dir: &std::path::Path,
    extracted_entries: &[crate::utils::asset_zip::ExtractedZipEntry],
) -> Result<std::collections::HashMap<String, String>> {
    let mut mapping = std::collections::HashMap::new();
    for entry in extracted_entries {
        let normalized_archive = entry.archive_path.trim().replace('\\', "/");
        let lowered = normalized_archive.to_ascii_lowercase();
        let biz = if lowered.starts_with("assets/content/") {
            Some("content")
        } else if lowered.starts_with("assets/options/") {
            Some("options")
        } else {
            None
        };

        let Some(biz_folder) = biz else {
            continue;
        };

        if !entry.output_path.exists() {
            continue;
        }

        let extension = entry
            .output_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png")
            .to_ascii_lowercase();
        let stem = entry
            .output_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("asset")
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect::<String>();

        let target_dir = app_data_dir
            .join("uploads")
            .join("images")
            .join("question-bank")
            .join(biz_folder);
        std::fs::create_dir_all(&target_dir).context("创建题库图片目录失败")?;
        let target_file_name = format!(
            "{}_{}.{}",
            stem,
            uuid::Uuid::new_v4().simple(),
            extension
        );
        let target_path = target_dir.join(&target_file_name);
        std::fs::copy(&entry.output_path, &target_path).context("复制资源包图片失败")?;

        mapping.insert(
            normalized_archive,
            format!(
                "uploads/images/question-bank/{}/{}",
                biz_folder, target_file_name
            ),
        );
    }

    Ok(mapping)
}

fn parse_question_bank_payloads_from_xlsx(
    xlsx_path: &std::path::Path,
    asset_mapping: &std::collections::HashMap<String, String>,
) -> Result<Vec<QuestionBankWritePayload>> {
    let mut workbook = open_workbook_auto(xlsx_path)?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| anyhow!("xlsx 中不存在工作表"))??;

    let mut rows = range.rows();
    let headers = rows
        .next()
        .ok_or_else(|| anyhow!("xlsx 表头为空"))?
        .iter()
        .map(cell_to_string)
        .collect::<Vec<String>>();

    let mut payloads = Vec::new();
    for row in rows {
        let row_map = headers
            .iter()
            .enumerate()
            .map(|(index, header)| {
                let value = row.get(index).map(cell_to_string).unwrap_or_default();
                (header.to_string(), value)
            })
            .collect::<std::collections::HashMap<String, String>>();

        let r#type = pick_value(&row_map, &["题型", "type"]).unwrap_or_default();
        let content = pick_value(&row_map, &["题目内容", "题干", "content"]).unwrap_or_default();
        let answer = pick_value(&row_map, &["答案", "answer"]).unwrap_or_default();
        let score = pick_value(&row_map, &["分值", "score"])
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0);
        let explanation = pick_value(&row_map, &["解析", "explanation"]);

        if r#type.trim().is_empty() || content.trim().is_empty() || answer.trim().is_empty() {
            continue;
        }

        let content_image_paths = parse_string_array(
            pick_value(&row_map, &["题干图片", "content_image_paths"]).unwrap_or_default(),
            asset_mapping,
        )?;

        let options_value = pick_value(&row_map, &["选项", "options"]).unwrap_or_default();
        let options = parse_question_bank_options(&options_value, asset_mapping)?;

        payloads.push(QuestionBankWritePayload::normalized(
            r#type,
            content,
            content_image_paths,
            options,
            answer,
            score,
            explanation,
            None,
            None,
        ));
    }

    Ok(payloads)
}

fn parse_question_bank_options(
    raw: &str,
    asset_mapping: &std::collections::HashMap<String, String>,
) -> Result<Vec<QuestionBankOptionValue>> {
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let parsed: Vec<serde_json::Value> = match serde_json::from_str::<Vec<serde_json::Value>>(raw) {
        Ok(value) => value,
        Err(_) => raw
            .split('|')
            .filter_map(|item| {
                let trimmed = item.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(serde_json::json!({ "key": trimmed, "text": "", "option_type": "text", "image_paths": [] }))
                }
            })
            .collect::<Vec<serde_json::Value>>(),
    };

    let mut options = Vec::new();
    for item in parsed {
        let key = item
            .get("key")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        let text = item
            .get("text")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        let option_type = item
            .get("option_type")
            .and_then(|value| value.as_str())
            .unwrap_or("text")
            .to_string();

        let raw_images = item
            .get("image_paths")
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Array(Vec::new()));
        let image_paths = match raw_images {
            serde_json::Value::Array(values) => values
                .into_iter()
                .filter_map(|value| value.as_str().map(|text| text.to_string()))
                .map(|value| map_asset_path(value, asset_mapping))
                .collect::<Vec<String>>(),
            _ => Vec::new(),
        };

        options.push(QuestionBankOptionValue {
            key,
            text,
            option_type,
            image_paths,
        });
    }

    Ok(options)
}

fn parse_string_array(
    raw: String,
    asset_mapping: &std::collections::HashMap<String, String>,
) -> Result<Vec<String>> {
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let values: Vec<String> = serde_json::from_str::<Vec<String>>(&raw)
        .unwrap_or_else(|_| {
            raw.split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<String>>()
        });

    Ok(values
        .into_iter()
        .map(|value| map_asset_path(value, asset_mapping))
        .collect())
}

fn map_asset_path(
    value: String,
    asset_mapping: &std::collections::HashMap<String, String>,
) -> String {
    let normalized = value.trim().replace('\\', "/");
    asset_mapping
        .get(&normalized)
        .cloned()
        .unwrap_or(normalized)
}

fn pick_value(
    row_map: &std::collections::HashMap<String, String>,
    keys: &[&str],
) -> Option<String> {
    keys.iter().find_map(|key| {
        row_map.get(*key).and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    })
}

fn cell_to_string(cell: &impl std::fmt::Display) -> String {
    cell.to_string().trim().to_string()
}