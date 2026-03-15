use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn test_db_connection(state: State<'_, AppState>) -> Result<String, String> {
    // 简单的数据库连接测试
    // 这里可以添加实际的数据库查询来验证连接
    Ok("数据库连接正常".to_string())
}