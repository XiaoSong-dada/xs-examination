use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

use crate::services::exam_service;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ExamDto {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExamInput {
    pub title: String,
}

#[tauri::command]
pub async fn get_exams(state: State<'_, AppState>) -> Result<Vec<ExamDto>, String> {
    let pool = &state.db;
    match exam_service::list_exams(pool).await {
        Ok(list) => {
            let dto: Vec<ExamDto> = list
                .into_iter()
                .map(|e| ExamDto { id: e.id, title: e.title, status: e.status })
                .collect();
            Ok(dto)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn create_exam(
    state: State<'_, AppState>,
    payload: CreateExamInput,
) -> Result<ExamDto, String> {
    let pool = &state.db;
    match exam_service::create_exam(pool, payload.title).await {
        Ok(e) => Ok(ExamDto { id: e.id, title: e.title, status: e.status }),
        Err(err) => Err(err.to_string()),
    }
}
