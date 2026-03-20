pub mod config;
pub mod db;
pub mod state;
pub mod commands;
pub mod controllers;
pub mod network;
pub mod schemas;
pub mod services;
pub mod utils;

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

            let discovery_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = crate::network::discovery_listener::start(discovery_handle).await {
                    eprintln!("[bootstrap] discovery listener stopped: {}", err);
                }
            });

            let control_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = crate::network::control_server::start(control_handle).await {
                    eprintln!("[bootstrap] control server stopped: {}", err);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::test_db_connection,
            commands::connect_teacher_ws,
            commands::send_answer_sync,
            commands::get_ws_status,
            commands::get_teacher_runtime_status,
            commands::get_current_exam_bundle,
            controllers::device_controller::get_device_runtime_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
