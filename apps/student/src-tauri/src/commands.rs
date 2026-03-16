use tauri::State;

use crate::state::AppState;

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
    crate::network::ws_client::connect(app_handle, ws_url, student_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok("已连接教师端 WebSocket".to_string())
}

#[tauri::command]
pub async fn send_answer_sync(
    state: State<'_, AppState>,
    exam_id: String,
    student_id: String,
    question_id: String,
    answer: String,
) -> Result<String, String> {
    let sender = state
        .ws_sender()
        .ok_or_else(|| "当前未建立 WebSocket 连接".to_string())?;

    let payload = crate::network::ws_client::build_answer_sync_message(
        &exam_id,
        &student_id,
        &question_id,
        &answer,
    )
    .map_err(|e| e.to_string())?;

    sender
        .send(payload)
        .map_err(|_| "发送失败：连接通道已关闭".to_string())?;

    Ok("答案同步消息已发送".to_string())
}

#[tauri::command]
pub async fn get_ws_status(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.ws_connected())
}