use tauri::State;

use crate::services::question_service;
use crate::services::question_service::QuestionWritePayload;
use crate::state::AppState;
use crate::schemas::question_schema;

#[tauri::command]
pub async fn get_questions(
    state: State<'_, AppState>,
    payload: question_schema::GetQuestionsInput,
) -> Result<Vec<question_schema::QuestionDto>, String> {
    let pool = &state.db;
    match question_service::list_questions(pool, payload.exam_id).await {
        Ok(list) => Ok(list
            .into_iter()
            .map(|q| question_schema::QuestionDto {
                id: q.id,
                exam_id: q.exam_id,
                seq: q.seq,
                r#type: q.r#type,
                content: q.content,
                options: q.options,
                answer: q.answer,
                score: q.score,
                explanation: q.explanation,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn bulk_import_questions(
    state: State<'_, AppState>,
    payload: question_schema::BulkImportQuestionsInput,
) -> Result<Vec<question_schema::QuestionDto>, String> {
    let pool = &state.db;
    let exam_id = payload.exam_id;
    let write_payloads = payload
        .questions
        .into_iter()
        .map(|item| QuestionWritePayload {
            id: item.id,
            seq: item.seq,
            r#type: item.r#type,
            content: item.content,
            options: item.options,
            answer: item.answer,
            score: item.score,
            explanation: item.explanation,
        })
        .collect();

    match question_service::replace_questions_by_exam_id(pool, exam_id, write_payloads).await {
        Ok(list) => Ok(list
            .into_iter()
            .map(|q| question_schema::QuestionDto {
                id: q.id,
                exam_id: q.exam_id,
                seq: q.seq,
                r#type: q.r#type,
                content: q.content,
                options: q.options,
                answer: q.answer,
                score: q.score,
                explanation: q.explanation,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}

/// 按考试导入题目资源包（zip），并覆盖写入题目列表。
///
/// # 参数
/// - `state`: 教师端共享应用状态，提供数据库连接。
/// - `app_handle`: Tauri 应用句柄，用于解析临时目录。
/// - `payload`: 包含考试 ID 与资源包绝对路径。
///
/// # 返回值
/// - 返回导入后的题目列表；解压、解析或写入失败时返回错误字符串。
#[tauri::command]
pub async fn import_question_package_by_exam_id(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    payload: question_schema::ImportQuestionPackageInput,
) -> Result<Vec<question_schema::QuestionDto>, String> {
    let pool = &state.db;

    match question_service::import_question_package_by_exam_id(
        pool,
        &app_handle,
        payload.exam_id,
        payload.package_path,
    )
    .await
    {
        Ok(list) => Ok(list
            .into_iter()
            .map(|q| question_schema::QuestionDto {
                id: q.id,
                exam_id: q.exam_id,
                seq: q.seq,
                r#type: q.r#type,
                content: q.content,
                options: q.options,
                answer: q.answer,
                score: q.score,
                explanation: q.explanation,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}
