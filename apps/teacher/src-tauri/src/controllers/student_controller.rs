use serde::{Deserialize, Serialize};
use tauri::State;

use crate::services::student_service;
use crate::services::student_service::StudentWritePayload;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StudentDto {
    pub id: String,
    pub student_no: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetStudentByIdInput {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStudentInput {
    pub student_no: String,
    pub name: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStudentInput {
    pub id: String,
    pub student_no: String,
    pub name: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteStudentInput {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkCreateStudentsInput {
    pub students: Vec<CreateStudentInput>,
}

#[tauri::command]
pub async fn get_students(state: State<'_, AppState>) -> Result<Vec<StudentDto>, String> {
    let pool = &state.db;
    match student_service::list_students(pool).await {
        Ok(list) => Ok(list
            .into_iter()
            .map(|s| StudentDto {
                id: s.id,
                student_no: s.student_no,
                name: s.name,
                created_at: s.created_at,
                updated_at: s.updated_at,
            })
            .collect()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_student_by_id(
    state: State<'_, AppState>,
    payload: GetStudentByIdInput,
) -> Result<StudentDto, String> {
    let pool = &state.db;
    match student_service::get_student_by_id(pool, payload.id).await {
        Ok(s) => Ok(StudentDto {
            id: s.id,
            student_no: s.student_no,
            name: s.name,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn create_student(
    state: State<'_, AppState>,
    payload: CreateStudentInput,
) -> Result<StudentDto, String> {
    let pool = &state.db;
    let write_payload = StudentWritePayload::normalized(
        payload.student_no,
        payload.name,
        payload.created_at,
        payload.updated_at,
    );

    match student_service::create_student(pool, write_payload).await {
        Ok(s) => Ok(StudentDto {
            id: s.id,
            student_no: s.student_no,
            name: s.name,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn update_student(
    state: State<'_, AppState>,
    payload: UpdateStudentInput,
) -> Result<StudentDto, String> {
    let pool = &state.db;
    let id = payload.id;
    let write_payload = StudentWritePayload::normalized(
        payload.student_no,
        payload.name,
        payload.created_at,
        payload.updated_at,
    );

    match student_service::update_student(pool, id, write_payload).await {
        Ok(s) => Ok(StudentDto {
            id: s.id,
            student_no: s.student_no,
            name: s.name,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn delete_student(
    state: State<'_, AppState>,
    payload: DeleteStudentInput,
) -> Result<(), String> {
    let pool = &state.db;
    student_service::delete_student(pool, payload.id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn bulk_create_students(
    state: State<'_, AppState>,
    payload: BulkCreateStudentsInput,
) -> Result<Vec<StudentDto>, String> {
    let pool = &state.db;
    let write_payloads = payload
        .students
        .into_iter()
        .map(|item| {
            StudentWritePayload::normalized(
                item.student_no,
                item.name,
                item.created_at,
                item.updated_at,
            )
        })
        .collect();

    match student_service::bulk_create_students(pool, write_payloads).await {
        Ok(list) => Ok(list
            .into_iter()
            .map(|s| StudentDto {
                id: s.id,
                student_no: s.student_no,
                name: s.name,
                created_at: s.created_at,
                updated_at: s.updated_at,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}
