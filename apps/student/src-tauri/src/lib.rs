pub mod config;
pub mod db;
pub mod state;
pub mod commands;
pub mod network;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_state = tauri::async_runtime::block_on(state::AppState::new(&app_handle))
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::test_db_connection,
            commands::connect_teacher_ws,
            commands::send_answer_sync,
            commands::get_ws_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
