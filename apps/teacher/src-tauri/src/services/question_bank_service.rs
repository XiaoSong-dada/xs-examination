use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::models::question_bank_item::Model as QuestionBankItemModel;
use crate::repos::question_bank_repo;
use crate::utils::asset_zip::{create_asset_zip, ZipAssetEntry};

#[derive(Debug, Clone)]
pub struct ExportQuestionBankPackageResult {
    pub output_path: String,
    pub packed_image_count: usize,
    pub missing_image_count: usize,
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