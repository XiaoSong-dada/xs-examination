pub mod commands;
pub mod config;
pub mod crypto;
pub mod db;
pub mod network;
pub mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_state = tauri::async_runtime::block_on(state::AppState::new(&app_handle))
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::exam::get_exams,
            commands::exam::create_exam,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
