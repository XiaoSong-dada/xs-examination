use anyhow::Result;
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

    let payloads = parse_question_payloads_from_xlsx(&xlsx_path)?;
    if payloads.is_empty() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Ok(Vec::new());
    }

    let result = replace_questions_by_exam_id(db, exam_id, payloads).await;
    let _ = std::fs::remove_dir_all(&temp_dir);
    result
}

fn parse_question_payloads_from_xlsx(xlsx_path: &std::path::Path) -> Result<Vec<QuestionWritePayload>> {
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

        let options = pick_value(&row_map, &["选项", "options"]).and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        if r#type.trim().is_empty() || content.trim().is_empty() || answer.trim().is_empty() {
            continue;
        }

        payloads.push(QuestionWritePayload {
            id: None,
            seq,
            r#type,
            content,
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
