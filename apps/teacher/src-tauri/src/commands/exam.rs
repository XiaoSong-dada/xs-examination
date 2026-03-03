use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Exam {
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
pub async fn get_exams(state: State<'_, AppState>) -> Result<Vec<Exam>, String> {
    sqlx::query_as::<_, Exam>("SELECT id, title, status FROM exams ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_exam(
    state: State<'_, AppState>,
    payload: CreateExamInput,
) -> Result<Exam, String> {
    let exam = Exam {
        id: uuid::Uuid::new_v4().to_string(),
        title: payload.title,
        status: "draft".to_string(),
    };

    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO exams (id, title, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&exam.id)
    .bind(&exam.title)
    .bind(&exam.status)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(exam)
}
