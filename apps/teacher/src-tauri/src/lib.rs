pub mod commands;
pub mod controllers;
pub mod core;
pub mod services;
pub mod repos;
pub mod models;
pub mod crypto;
pub mod db;
pub mod network;
pub mod utils;
pub mod state;

// schemas contains DTOs and input/output payloads used by the controllers
pub mod schemas;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_state = tauri::async_runtime::block_on(state::AppState::new(&app_handle))
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            app.manage(app_state);

            let ws_app_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = crate::network::ws_server::start_ws_server(ws_app_handle).await {
                    eprintln!("[ws-server] stopped with error: {}", err);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            controllers::exam_controller::get_exams,
            controllers::exam_controller::get_exam_by_id,
            controllers::exam_controller::create_exam,
            controllers::exam_controller::update_exam,
            controllers::exam_controller::delete_exam,
            controllers::device_controller::get_devices,
            controllers::device_controller::get_device_by_id,
            controllers::device_controller::create_device,
            controllers::device_controller::update_device,
            controllers::device_controller::delete_device,
            controllers::device_controller::discover_student_devices,
            controllers::device_controller::replace_devices_by_discovery,
            controllers::device_controller::push_teacher_endpoints_to_devices,
            controllers::student_controller::get_students,
            controllers::student_controller::get_student_by_id,
            controllers::student_controller::create_student,
            controllers::student_controller::update_student,
            controllers::student_controller::delete_student,
            controllers::student_controller::bulk_create_students,
            controllers::student_exam_controller::get_students_by_exam_id,
            controllers::student_exam_controller::import_students_by_exam_id,
            controllers::student_exam_controller::get_student_device_assignments_by_exam_id,
            controllers::student_exam_controller::assign_devices_to_student_exams,
            controllers::student_exam_controller::connect_student_devices_by_exam_id,
            controllers::student_exam_controller::get_student_device_connection_status_by_exam_id,
            controllers::student_exam_controller::get_student_score_summary_by_exam_id,
            controllers::student_exam_controller::calculate_student_score_summary_by_exam_id,
            controllers::student_exam_controller::save_score_report_file,
            controllers::student_exam_controller::resolve_report_download_path,
            controllers::student_exam_controller::distribute_exam_papers_by_exam_id,
            controllers::student_exam_controller::start_exam_by_exam_id,
            controllers::student_exam_controller::end_exam_by_exam_id,
            controllers::question_controller::get_questions,
            controllers::question_controller::bulk_import_questions,
            controllers::question_controller::import_question_package_by_exam_id,
            controllers::file_asset_controller::upload_local_image_asset,
            controllers::file_asset_controller::resolve_image_asset_preview,
            controllers::question_bank_controller::get_question_bank_items,
            controllers::question_bank_controller::get_question_bank_item_by_id,
            controllers::question_bank_controller::create_question_bank_item,
            controllers::question_bank_controller::update_question_bank_item,
            controllers::question_bank_controller::delete_question_bank_item,
            controllers::question_bank_controller::export_question_bank_package,
            controllers::question_bank_controller::import_question_bank_package,
            controllers::network_controller::get_online_students,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
