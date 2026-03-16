use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn get_online_students(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let list = state
        .snapshot_connections()
        .into_iter()
        .map(|(student_id, ts)| format!("{}:{}", student_id, ts))
        .collect();
    Ok(list)
}
