use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn test_db_connection(state: State<'_, AppState>) -> Result<String, String> {
    state.db.ping().await.map_err(|e| e.to_string())?;
    Ok("数据库连接正常".to_string())
}