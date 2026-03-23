use anyhow::Result;
use tauri::{Emitter, Manager};
use tokio::time::{sleep, Duration};

use crate::schemas::teacher_endpoint_schema::{TeacherEndpointAppliedEvent, WsConnectionEvent};

const RETRY_INTERVAL_SECS: u64 = 3;

pub struct WsReconnectService;

impl WsReconnectService {
    pub async fn bootstrap_from_local_state(app_handle: tauri::AppHandle) -> Result<()> {
        let endpoint = crate::services::teacher_endpoints_service::TeacherEndpointsService::get_master_endpoint(&app_handle)
            .await?;

        let bundle = crate::services::exam_runtime_service::ExamRuntimeService::get_current_exam_bundle(&app_handle)
            .await?;

        let Some(target_endpoint) = endpoint else {
            return Ok(());
        };

        let Some(session) = bundle.session else {
            return Ok(());
        };

        Self::start_or_update(app_handle, target_endpoint, session.student_id).await
    }

    pub async fn start_or_update(
        app_handle: tauri::AppHandle,
        endpoint: String,
        student_id: String,
    ) -> Result<()> {
        let endpoint = endpoint.trim().to_string();
        let student_id = student_id.trim().to_string();
        if endpoint.is_empty() || student_id.is_empty() {
            return Ok(());
        }

        let state = app_handle.state::<crate::state::AppState>();
        let old_endpoint = state.ws_endpoint();
        let old_target = state.reconnect_target();
        if state.ws_connected()
            && old_endpoint
                .as_deref()
                .is_some_and(|existing| existing != endpoint.as_str())
        {
            crate::network::ws_client::force_disconnect(&app_handle, "连接目标已切换，准备重连");
        }

        // Same endpoint but different student_id also requires reconnect,
        // otherwise heartbeat and server-side student mapping may stay stale.
        if state.ws_connected()
            && old_endpoint
                .as_deref()
                .is_some_and(|existing| existing == endpoint.as_str())
            && old_target
                .as_ref()
                .is_some_and(|(_, old_student_id)| old_student_id != &student_id)
        {
            crate::network::ws_client::force_disconnect(&app_handle, "考生标识已切换，准备重连");
        }
        state.set_reconnect_target(endpoint.clone(), student_id.clone());

        let _ = app_handle.emit(
            "teacher_endpoint_applied",
            TeacherEndpointAppliedEvent {
                endpoint: Some(endpoint.clone()),
            },
        );

        let app_for_loop = app_handle.clone();
        let loop_handle = tokio::spawn(async move {
            loop {
                let target = {
                    let state = app_for_loop.state::<crate::state::AppState>();
                    state.reconnect_target()
                };

                let Some((target_endpoint, target_student_id)) = target else {
                    break;
                };

                if target_endpoint != endpoint || target_student_id != student_id {
                    break;
                }

                let is_connected = {
                    let state = app_for_loop.state::<crate::state::AppState>();
                    state.ws_connected()
                };

                if is_connected {
                    sleep(Duration::from_secs(RETRY_INTERVAL_SECS)).await;
                    continue;
                }

                let result = crate::network::ws_client::connect(
                    app_for_loop.clone(),
                    target_endpoint.clone(),
                    target_student_id.clone(),
                )
                .await;

                if let Err(err) = result {
                    let _ = app_for_loop.emit(
                        "ws_disconnected",
                        WsConnectionEvent {
                            endpoint: Some(target_endpoint),
                            connected: false,
                            message: Some(format!("连接失败，等待重试: {}", err)),
                        },
                    );
                }

                sleep(Duration::from_secs(RETRY_INTERVAL_SECS)).await;
            }
        });

        let state = app_handle.state::<crate::state::AppState>();
        state.replace_reconnect_task(loop_handle);

        Ok(())
    }
}
