use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

use crate::services::exam_service;
use crate::services::exam_service::ExamWritePayload;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ExamDto {
    pub id: String,
    pub title: String,
    pub status: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamDetailDto {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub pass_score: i64,
    pub status: String,
    pub shuffle_questions: i64,
    pub shuffle_options: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetExamByIdInput {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateExamInput {
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub pass_score: Option<i64>,
    pub status: Option<String>,
    pub shuffle_questions: Option<IntOrBool>,
    pub shuffle_options: Option<IntOrBool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateExamInput {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub pass_score: Option<i64>,
    pub status: Option<String>,
    pub shuffle_questions: Option<IntOrBool>,
    pub shuffle_options: Option<IntOrBool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteExamInput {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum IntOrBool {
    Int(i64),
    Bool(bool),
}

impl IntOrBool {
    fn to_i64(&self) -> i64 {
        match self {
            Self::Int(v) => *v,
            Self::Bool(v) => i64::from(*v),
        }
    }
}

fn to_write_payload(
    title: String,
    description: Option<String>,
    start_time: Option<i64>,
    end_time: Option<i64>,
    pass_score: Option<i64>,
    status: Option<String>,
    shuffle_questions: Option<IntOrBool>,
    shuffle_options: Option<IntOrBool>,
) -> ExamWritePayload {
    ExamWritePayload::with_defaults(
        title,
        description,
        start_time,
        end_time,
        pass_score,
        status,
        shuffle_questions.map(|v| v.to_i64()),
        shuffle_options.map(|v| v.to_i64()),
    )
}

#[tauri::command]
pub async fn get_exams(state: State<'_, AppState>) -> Result<Vec<ExamDto>, String> {
    let pool = &state.db;
    match exam_service::list_exams(pool).await {
        Ok(list) => {
            let dto: Vec<ExamDto> = list
                .into_iter()
                .map(|e| ExamDto { id: e.id, title: e.title, status: e.status, description: e.description })
                .collect();
            Ok(dto)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_exam_by_id(
    state: State<'_, AppState>,
    payload: GetExamByIdInput,
) -> Result<ExamDetailDto, String> {
    let pool = &state.db;
    match exam_service::get_exam_by_id(pool, payload.id).await {
        Ok(e) => Ok(ExamDetailDto {
            id: e.id,
            title: e.title,
            description: e.description,
            start_time: e.start_time,
            end_time: e.end_time,
            pass_score: e.pass_score,
            status: e.status,
            shuffle_questions: e.shuffle_questions,
            shuffle_options: e.shuffle_options,
        }),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn create_exam(
    state: State<'_, AppState>,
    payload: CreateExamInput,
) -> Result<ExamDto, String> {
    let pool = &state.db;
    let write_payload = to_write_payload(
        payload.title,
        payload.description,
        payload.start_time,
        payload.end_time,
        payload.pass_score,
        payload.status,
        payload.shuffle_questions,
        payload.shuffle_options,
    );

    match exam_service::create_exam(pool, write_payload).await {
        Ok(e) => Ok(ExamDto { id: e.id, description: e.description, title: e.title, status: e.status }),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn update_exam(
    state: State<'_, AppState>,
    payload: UpdateExamInput,
) -> Result<ExamDto, String> {
    let pool = &state.db;
    let id = payload.id;
    let write_payload = to_write_payload(
        payload.title,
        payload.description,
        payload.start_time,
        payload.end_time,
        payload.pass_score,
        payload.status,
        payload.shuffle_questions,
        payload.shuffle_options,
    );

    match exam_service::update_exam(pool, id, write_payload).await {
        Ok(e) => Ok(ExamDto { id: e.id, description: e.description, title: e.title, status: e.status }),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn delete_exam(
    state: State<'_, AppState>,
    payload: DeleteExamInput,
) -> Result<(), String> {
    let pool = &state.db;
    exam_service::delete_exam(pool, payload.id)
        .await
        .map_err(|err| err.to_string())
}
