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
