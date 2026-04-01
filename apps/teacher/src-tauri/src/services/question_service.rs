use anyhow::{Context, Result};
use calamine::{open_workbook_auto, Reader};
use sea_orm::DatabaseConnection;
use tauri::Manager;

use crate::models::question::Model as QuestionModel;
use crate::repos::question_repo;
use crate::utils::asset_zip::extract_asset_zip;

#[derive(Debug, Clone)]
pub struct QuestionWritePayload {
    pub id: Option<String>,
    pub seq: i32,
    pub r#type: String,
    pub content: String,
    pub content_image_paths: Option<String>,
    pub options: Option<String>,
    pub answer: String,
    pub score: i32,
    pub explanation: Option<String>,
}

pub async fn list_questions(db: &DatabaseConnection, exam_id: String) -> Result<Vec<QuestionModel>> {
    question_repo::get_all_questions(db, &exam_id).await
}

pub async fn replace_questions_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
    payloads: Vec<QuestionWritePayload>,
) -> Result<Vec<QuestionModel>> {
    let rows = payloads
        .into_iter()
        .map(|payload| question_repo::QuestionBatchInsertItem {
            id: payload.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            exam_id: exam_id.clone(),
            seq: payload.seq,
            r#type: payload.r#type.trim().to_string(),
            content: payload.content.trim().to_string(),
            content_image_paths: payload
                .content_image_paths
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            options: payload.options.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()),
            answer: payload.answer.trim().to_string(),
            score: payload.score,
            explanation: payload
                .explanation
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
        })
        .collect();

    question_repo::replace_questions_by_exam_id(db, &exam_id, rows).await
}

/// 从题库资源包（zip）导入题目到指定考试。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `app_handle`: Tauri 应用句柄，用于解析临时目录。
/// - `exam_id`: 目标考试 ID。
/// - `package_path`: 题库资源包绝对路径。
///
/// # 返回值
/// - 返回按考试覆盖导入后的题目数组；解压、解析或入库失败时返回错误。
pub async fn import_question_package_by_exam_id(
    db: &DatabaseConnection,
    app_handle: &tauri::AppHandle,
    exam_id: String,
    package_path: String,
) -> Result<Vec<QuestionModel>> {
    let app_data_dir = app_handle.path().app_data_dir()?;
    let temp_dir = app_data_dir
        .join("temp")
        .join(format!("question-package-import-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&temp_dir)?;

    let extracted = extract_asset_zip(std::path::Path::new(package_path.trim()), &temp_dir)?;
    let xlsx_path = extracted
        .iter()
        .find_map(|item| {
            let lower = item.archive_path.to_ascii_lowercase();
            if lower == "question_bank.xlsx" || lower.ends_with(".xlsx") {
                Some(item.output_path.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("资源包中未找到 xlsx 文件"))?;

    let asset_mapping = materialize_package_assets(&app_data_dir, &exam_id, &extracted)?;
    let payloads = parse_question_payloads_from_xlsx(&xlsx_path, &asset_mapping)?;
    if payloads.is_empty() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Ok(Vec::new());
    }

    let result = replace_questions_by_exam_id(db, exam_id, payloads).await;
    let _ = std::fs::remove_dir_all(&temp_dir);
    result
}

fn parse_question_payloads_from_xlsx(
    xlsx_path: &std::path::Path,
    asset_mapping: &std::collections::HashMap<String, String>,
) -> Result<Vec<QuestionWritePayload>> {
    let mut workbook = open_workbook_auto(xlsx_path)?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| anyhow::anyhow!("xlsx 中不存在工作表"))??;

    let mut rows = range.rows();
    let headers = rows
        .next()
        .ok_or_else(|| anyhow::anyhow!("xlsx 表头为空"))?
        .iter()
        .map(cell_to_string)
        .collect::<Vec<String>>();

    let mut payloads = Vec::new();
    for (index, row) in rows.enumerate() {
        let row_map = headers
            .iter()
            .enumerate()
            .map(|(col_idx, header)| {
                let value = row.get(col_idx).map(cell_to_string).unwrap_or_default();
                (header.to_string(), value)
            })
            .collect::<std::collections::HashMap<String, String>>();

        let seq = pick_value(&row_map, &["序号", "seq"])
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or((index + 1) as i32);
        let r#type = pick_value(&row_map, &["题型", "type"]).unwrap_or_default();
        let content = pick_value(&row_map, &["题目内容", "题干", "content"]).unwrap_or_default();
        let answer = pick_value(&row_map, &["答案", "answer"]).unwrap_or_default();
        let score = pick_value(&row_map, &["分值", "score"])
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0);
        let explanation = pick_value(&row_map, &["解析", "explanation"]).and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        let content_image_paths = parse_string_array(
            pick_value(&row_map, &["题干图片", "content_image_paths"]).unwrap_or_default(),
            asset_mapping,
        )?;

        let options = pick_value(&row_map, &["选项", "options"]).and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let options = normalize_options_with_assets(options, asset_mapping)?;

        if r#type.trim().is_empty() || content.trim().is_empty() || answer.trim().is_empty() {
            continue;
        }

        payloads.push(QuestionWritePayload {
            id: None,
            seq,
            r#type,
            content,
            content_image_paths: serialize_string_array(content_image_paths)?,
            options,
            answer,
            score,
            explanation,
        });
    }

    Ok(payloads)
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

fn materialize_package_assets(
    app_data_dir: &std::path::Path,
    exam_id: &str,
    extracted: &[crate::utils::asset_zip::ExtractedZipEntry],
) -> Result<std::collections::HashMap<String, String>> {
    let mut mapping = std::collections::HashMap::new();
    let exam_folder = sanitize_path_component(exam_id);

    for entry in extracted {
        let normalized_archive = entry.archive_path.trim().replace('\\', "/");
        let normalized_lower = normalized_archive.to_ascii_lowercase();
        let biz_folder = if normalized_lower.starts_with("assets/content/") {
            "content"
        } else if normalized_lower.starts_with("assets/options/") {
            "options"
        } else {
            continue;
        };

        let extension = entry
            .output_path
            .extension()
            .and_then(|value| value.to_str())
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
            .join("questions")
            .join(&exam_folder)
            .join(biz_folder);
        std::fs::create_dir_all(&target_dir).context("创建考试题目图片目录失败")?;
        let target_file_name = format!(
            "{}_{}.{}",
            stem,
            uuid::Uuid::new_v4().simple(),
            extension
        );
        let target_path = target_dir.join(&target_file_name);
        std::fs::copy(&entry.output_path, &target_path).context("复制考试题目资源包图片失败")?;

        mapping.insert(
            normalized_archive,
            format!(
                "uploads/images/questions/{}/{}/{}",
                exam_folder, biz_folder, target_file_name
            ),
        );
    }

    Ok(mapping)
}

fn sanitize_path_component(input: &str) -> String {
    let filtered = input
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>();

    if filtered.is_empty() {
        "unknown_exam".to_string()
    } else {
        filtered
    }
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

fn serialize_string_array(value: Vec<String>) -> Result<Option<String>> {
    if value.is_empty() {
        return Ok(None);
    }
    Ok(Some(serde_json::to_string(&value)?))
}

fn normalize_options_with_assets(
    raw: Option<String>,
    asset_mapping: &std::collections::HashMap<String, String>,
) -> Result<Option<String>> {
    let Some(raw_value) = raw else {
        return Ok(None);
    };

    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let Ok(mut options) = serde_json::from_str::<Vec<serde_json::Value>>(trimmed) else {
        return Ok(Some(trimmed.to_string()));
    };

    for option in &mut options {
        let images = option
            .get("image_paths")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        let mapped = images
            .into_iter()
            .filter_map(|value| value.as_str().map(|text| text.to_string()))
            .map(|value| map_asset_path(value, asset_mapping))
            .map(serde_json::Value::String)
            .collect::<Vec<serde_json::Value>>();

        if let Some(object) = option.as_object_mut() {
            object.insert("image_paths".to_string(), serde_json::Value::Array(mapped));
        }
    }

    Ok(Some(serde_json::to_string(&options)?))
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
