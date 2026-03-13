use tauri::State;

use crate::schemas::student_exam_schema;
use crate::services::student_exam_service;
use crate::state::AppState;

#[tauri::command]
pub async fn get_students_by_exam_id(
    state: State<'_, AppState>,
    payload: student_exam_schema::GetStudentExamsInput,
) -> Result<Vec<student_exam_schema::ExamStudentDto>, String> {
    let pool = &state.db;
    match student_exam_service::list_student_exams_by_exam_id(pool, payload.exam_id).await {
        Ok(list) => Ok(list
            .into_iter()
            .map(|item| student_exam_schema::ExamStudentDto {
                id: item.id,
                student_no: item.student_no,
                name: item.name,
                created_at: item.created_at,
                updated_at: item.updated_at,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn import_students_by_exam_id(
    state: State<'_, AppState>,
    payload: student_exam_schema::ImportStudentsByExamIdInput,
) -> Result<Vec<student_exam_schema::ExamStudentDto>, String> {
    let pool = &state.db;
    match student_exam_service::import_students_by_exam_id(pool, payload.exam_id, payload.student_ids)
        .await
    {
        Ok(list) => Ok(list
            .into_iter()
            .map(|item| student_exam_schema::ExamStudentDto {
                id: item.id,
                student_no: item.student_no,
                name: item.name,
                created_at: item.created_at,
                updated_at: item.updated_at,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn get_student_device_assignments_by_exam_id(
    state: State<'_, AppState>,
    payload: student_exam_schema::GetStudentExamsInput,
) -> Result<Vec<student_exam_schema::StudentDeviceAssignDto>, String> {
    let pool = &state.db;
    student_exam_service::list_student_device_assignments_by_exam_id(pool, payload.exam_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn assign_devices_to_student_exams(
    state: State<'_, AppState>,
    payload: student_exam_schema::AssignDevicesToStudentExamsInput,
) -> Result<Vec<student_exam_schema::StudentDeviceAssignDto>, String> {
    let pool = &state.db;
    student_exam_service::assign_devices_to_student_exams(pool, payload.exam_id, payload.assignments)
        .await
        .map_err(|err| err.to_string())
}
