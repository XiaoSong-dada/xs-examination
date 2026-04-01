use tauri::State;

use crate::schemas::question_bank_schema;
use crate::services::question_bank_service::{self, QuestionBankOptionValue, QuestionBankWritePayload};
use crate::state::AppState;

/// 查询全部全局题库题目。
///
/// # 参数
/// - `state`: 教师端共享应用状态，提供数据库连接。
///
/// # 返回值
/// - 返回前端可直接消费的题库题目数组；查询或解析失败时返回错误字符串。
#[tauri::command]
pub async fn get_question_bank_items(
    state: State<'_, AppState>,
) -> Result<Vec<question_bank_schema::QuestionBankItemDto>, String> {
    let pool = &state.db;
    match question_bank_service::list_question_bank_items(pool).await {
        Ok(list) => Ok(list.into_iter().map(to_question_bank_item_dto).collect()),
        Err(err) => Err(err.to_string()),
    }
}

/// 按 ID 查询单条全局题库题目。
///
/// # 参数
/// - `state`: 教师端共享应用状态，提供数据库连接。
/// - `payload`: 包含题目 ID 的查询参数。
///
/// # 返回值
/// - 返回题目详情；题目不存在、查询失败或解析失败时返回错误字符串。
#[tauri::command]
pub async fn get_question_bank_item_by_id(
    state: State<'_, AppState>,
    payload: question_bank_schema::GetQuestionBankItemByIdInput,
) -> Result<question_bank_schema::QuestionBankItemDto, String> {
    let pool = &state.db;
    match question_bank_service::get_question_bank_item_by_id(pool, payload.id).await {
        Ok(item) => Ok(to_question_bank_item_dto(item)),
        Err(err) => Err(err.to_string()),
    }
}

/// 新增一条全局题库题目。
///
/// # 参数
/// - `state`: 教师端共享应用状态，提供数据库连接。
/// - `payload`: 前端提交的题目表单数据。
///
/// # 返回值
/// - 返回新增后的题目详情；字段校验、数据库写入或解析失败时返回错误字符串。
#[tauri::command]
pub async fn create_question_bank_item(
    state: State<'_, AppState>,
    payload: question_bank_schema::CreateQuestionBankItemInput,
) -> Result<question_bank_schema::QuestionBankItemDto, String> {
    let pool = &state.db;
    let write_payload = to_question_bank_write_payload(
        payload.r#type,
        payload.content,
        payload.content_image_paths,
        payload.options,
        payload.answer,
        payload.score,
        payload.explanation,
        payload.created_at,
        payload.updated_at,
    );

    match question_bank_service::create_question_bank_item(pool, write_payload).await {
        Ok(item) => Ok(to_question_bank_item_dto(item)),
        Err(err) => Err(err.to_string()),
    }
}

/// 更新一条全局题库题目。
///
/// # 参数
/// - `state`: 教师端共享应用状态，提供数据库连接。
/// - `payload`: 包含题目 ID 与最新表单数据的更新参数。
///
/// # 返回值
/// - 返回更新后的题目详情；字段校验、数据库写入或解析失败时返回错误字符串。
#[tauri::command]
pub async fn update_question_bank_item(
    state: State<'_, AppState>,
    payload: question_bank_schema::UpdateQuestionBankItemInput,
) -> Result<question_bank_schema::QuestionBankItemDto, String> {
    let pool = &state.db;
    let id = payload.id;
    let write_payload = to_question_bank_write_payload(
        payload.r#type,
        payload.content,
        payload.content_image_paths,
        payload.options,
        payload.answer,
        payload.score,
        payload.explanation,
        payload.created_at,
        payload.updated_at,
    );

    match question_bank_service::update_question_bank_item(pool, id, write_payload).await {
        Ok(item) => Ok(to_question_bank_item_dto(item)),
        Err(err) => Err(err.to_string()),
    }
}

/// 删除一条全局题库题目。
///
/// # 参数
/// - `state`: 教师端共享应用状态，提供数据库连接。
/// - `payload`: 包含题目 ID 的删除参数。
///
/// # 返回值
/// - 删除成功返回 `()`；题目不存在或数据库删除失败时返回错误字符串。
#[tauri::command]
pub async fn delete_question_bank_item(
    state: State<'_, AppState>,
    payload: question_bank_schema::DeleteQuestionBankItemInput,
) -> Result<(), String> {
    let pool = &state.db;
    question_bank_service::delete_question_bank_item(pool, payload.id)
        .await
        .map_err(|err| err.to_string())
}

/// 导出题库资源包（xlsx + 图片）到本机下载目录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄，用于解析导出目录。
/// - `payload`: 包含导出文件名、xlsx 字节与图片路径列表。
///
/// # 返回值
/// - 返回导出包路径和资源统计；写盘、打包或路径解析失败时返回错误字符串。
#[tauri::command]
pub async fn export_question_bank_package(
    app_handle: tauri::AppHandle,
    payload: question_bank_schema::ExportQuestionBankPackageInput,
) -> Result<question_bank_schema::ExportQuestionBankPackageOutput, String> {
    match question_bank_service::export_question_bank_package(
        &app_handle,
        payload.file_name,
        payload.xlsx_bytes,
        payload.image_relative_paths,
    ) {
        Ok(result) => Ok(question_bank_schema::ExportQuestionBankPackageOutput {
            path: result.output_path,
            packed_image_count: result.packed_image_count,
            missing_image_count: result.missing_image_count,
        }),
        Err(err) => Err(err.to_string()),
    }
}

/// 导入题库资源包到 `question_bank_items`，并在导入前清空原有数据。
///
/// # 参数
/// - `state`: 教师端共享应用状态，提供数据库连接。
/// - `app_handle`: Tauri 应用句柄，用于解析应用数据目录。
/// - `payload`: 包含资源包绝对路径。
///
/// # 返回值
/// - 返回导入条数；解压、解析、清空或写入失败时返回错误字符串。
#[tauri::command]
pub async fn import_question_bank_package(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    payload: question_bank_schema::ImportQuestionBankPackageInput,
) -> Result<question_bank_schema::ImportQuestionBankPackageOutput, String> {
    let pool = &state.db;
    match question_bank_service::import_question_bank_package(
        pool,
        &app_handle,
        payload.package_path,
    )
    .await
    {
        Ok(result) => Ok(question_bank_schema::ImportQuestionBankPackageOutput {
            imported_count: result.imported_count,
        }),
        Err(err) => Err(err.to_string()),
    }
}

fn to_question_bank_write_payload(
    r#type: String,
    content: String,
    content_image_paths: Vec<String>,
    options: Vec<question_bank_schema::QuestionBankOptionDto>,
    answer: String,
    score: i32,
    explanation: Option<String>,
    created_at: Option<i64>,
    updated_at: Option<i64>,
) -> QuestionBankWritePayload {
    QuestionBankWritePayload::normalized(
        r#type,
        content,
        content_image_paths,
        options
            .into_iter()
            .map(|item| QuestionBankOptionValue {
                key: item.key,
                text: item.text,
                option_type: item.option_type,
                image_paths: item.image_paths,
            })
            .collect(),
        answer,
        score,
        explanation,
        created_at,
        updated_at,
    )
}

fn to_question_bank_item_dto(
    item: question_bank_service::QuestionBankItemView,
) -> question_bank_schema::QuestionBankItemDto {
    question_bank_schema::QuestionBankItemDto {
        id: item.id,
        r#type: item.r#type,
        content: item.content,
        content_image_paths: item.content_image_paths,
        options: item
            .options
            .into_iter()
            .map(|option| question_bank_schema::QuestionBankOptionDto {
                key: option.key,
                text: option.text,
                option_type: option.option_type,
                image_paths: option.image_paths,
            })
            .collect(),
        answer: item.answer,
        score: item.score,
        explanation: item.explanation,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}