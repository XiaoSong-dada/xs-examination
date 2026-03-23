use tauri::State;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde_json::json;
use crate::network::protocol::AnswerItem;

use crate::schemas::teacher_endpoint_schema;
use crate::schemas::exam_runtime_schema;
use crate::state::AppState;
use crate::db::entities::{exam_sessions, local_answers, sync_outbox};
use crate::services::teacher_endpoints_service::TeacherEndpointsService;
use crate::services::exam_runtime_service::ExamRuntimeService;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

#[tauri::command]
pub async fn test_db_connection(state: State<'_, AppState>) -> Result<String, String> {
    state.db.ping().await.map_err(|e| e.to_string())?;
    Ok("数据库连接正常".to_string())
}

#[tauri::command]
pub async fn connect_teacher_ws(
    app_handle: tauri::AppHandle,
    ws_url: String,
    student_id: String,
) -> Result<String, String> {
    crate::services::ws_reconnect_service::WsReconnectService::start_or_update(
        app_handle,
        ws_url,
        student_id,
    )
        .await
        .map_err(|e| e.to_string())?;
    Ok("已开始连接教师端 WebSocket（含自动重试）".to_string())
}

#[tauri::command]
pub async fn send_answer_sync(
    state: State<'_, AppState>,
    exam_id: String,
    student_id: String,
    question_id: String,
    answer: String,
) -> Result<String, String> {
    let ts = now_ms();

    let session = exam_sessions::Entity::find()
        .filter(exam_sessions::Column::ExamId.eq(exam_id.clone()))
        .filter(exam_sessions::Column::StudentId.eq(student_id.clone()))
        .order_by_desc(exam_sessions::Column::UpdatedAt)
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "未找到对应考试会话，无法保存答案".to_string())?;

    let session_id = session.id.clone();

    let existing_answer = local_answers::Entity::find()
        .filter(local_answers::Column::SessionId.eq(session_id.clone()))
        .filter(local_answers::Column::QuestionId.eq(question_id.clone()))
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let revision = existing_answer
        .as_ref()
        .map(|item| item.revision + 1)
        .unwrap_or(1);

    match existing_answer {
        Some(row) => {
            let mut model: local_answers::ActiveModel = row.into();
            model.answer = Set(Some(answer.clone()));
            model.revision = Set(revision);
            model.sync_status = Set("pending".to_string());
            model.updated_at = Set(ts);
            model.update(&state.db).await.map_err(|e| e.to_string())?;
        }
        None => {
            let model = local_answers::ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                session_id: Set(session_id.clone()),
                question_id: Set(question_id.clone()),
                answer: Set(Some(answer.clone())),
                answer_blob: Set(None),
                revision: Set(revision),
                sync_status: Set("pending".to_string()),
                last_synced_at: Set(None),
                updated_at: Set(ts),
            };
            model.insert(&state.db).await.map_err(|e| e.to_string())?;
        }
    }

    let outbox_payload = json!({
        "examId": exam_id.clone(),
        "studentId": student_id.clone(),
        "questionId": question_id.clone(),
        "answer": answer.clone(),
        "revision": revision,
        "timestamp": ts,
    })
    .to_string()
    .into_bytes();

    let outbox_model = sync_outbox::ActiveModel {
        id: Default::default(),
        session_id: Set(session_id.clone()),
        event_type: Set("ANSWER_SYNC".to_string()),
        aggregate_id: Set(Some(format!("{}:{}", session_id, question_id))),
        payload: Set(outbox_payload),
        status: Set("pending".to_string()),
        retry_count: Set(0),
        next_retry_at: Set(None),
        last_error: Set(None),
        created_at: Set(ts),
        updated_at: Set(ts),
    };
    let inserted_outbox = outbox_model
        .insert(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let sender = match state.ws_sender() {
        Some(sender) => sender,
        None => return Ok("答案已保存到本地，等待连接恢复后同步".to_string()),
    };

    let payload_answers = vec![AnswerItem {
        question_id: question_id.clone(),
        answer: answer.clone(),
        revision: Some(revision),
        answer_updated_at: Some(ts),
    }];

    let payload = crate::network::ws_client::build_answer_sync_message(
        &exam_id,
        &student_id,
        Some(&session_id),
        payload_answers,
        "incremental",
        None,
    )
    .map_err(|e| e.to_string())?;

    if sender.send(payload).is_err() {
        return Ok("答案已保存到本地，发送通道关闭，稍后重试".to_string());
    }

    let mut outbox_active: sync_outbox::ActiveModel = inserted_outbox.into();
    outbox_active.status = Set("sent".to_string());
    outbox_active.updated_at = Set(ts);
    outbox_active
        .update(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok("答案已保存并发送，等待教师端确认".to_string())
}

#[tauri::command]
pub async fn get_ws_status(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.ws_connected())
}

#[tauri::command]
pub async fn get_teacher_runtime_status(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<teacher_endpoint_schema::TeacherRuntimeStatusDto, String> {
    let endpoint = TeacherEndpointsService::get_master_endpoint(&app_handle)
        .await
        .map_err(|e| e.to_string())?;

    let connection_status = if state.ws_connected() {
        "connected"
    } else {
        "disconnected"
    }
    .to_string();

    Ok(teacher_endpoint_schema::TeacherRuntimeStatusDto {
        endpoint,
        connection_status,
    })
}

#[tauri::command]
pub async fn get_current_exam_bundle(
    app_handle: tauri::AppHandle,
) -> Result<exam_runtime_schema::CurrentExamBundleDto, String> {
    ExamRuntimeService::get_current_exam_bundle(&app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_current_session_answers(
    app_handle: tauri::AppHandle,
) -> Result<Vec<exam_runtime_schema::LocalAnswerDto>, String> {
    ExamRuntimeService::get_current_session_answers(&app_handle)
        .await
        .map_err(|e| e.to_string())
}