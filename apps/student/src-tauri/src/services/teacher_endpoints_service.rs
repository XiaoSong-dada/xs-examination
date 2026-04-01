use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use tauri::Manager;

use crate::schemas::control_protocol::TeacherEndpointInput;
use crate::repos::teacher_endpoint_repo;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

pub struct TeacherEndpointsService;

impl TeacherEndpointsService {
    pub async fn replace_all(
        app_handle: &tauri::AppHandle,
        endpoints: &[TeacherEndpointInput],
    ) -> Result<()> {
        if endpoints.is_empty() {
            return Err(anyhow!("endpoints 不能为空"));
        }

        let master_count = endpoints.iter().filter(|e| e.is_master).count();
        if master_count != 1 {
            return Err(anyhow!("endpoints 中必须且只能有一个 isMaster=true"));
        }

        let state = app_handle.state::<crate::state::AppState>();
        let ts = now_ms();
        teacher_endpoint_repo::replace_all_endpoints(&state.db, endpoints, ts).await?;
        Ok(())
    }

    pub fn master_endpoint(endpoints: &[TeacherEndpointInput]) -> Option<String> {
        teacher_endpoint_repo::get_master_endpoint_from_input(endpoints)
    }

    pub async fn get_master_endpoint(app_handle: &tauri::AppHandle) -> Result<Option<String>> {
        let state = app_handle.state::<crate::state::AppState>();
        let master = teacher_endpoint_repo::get_master_endpoint(&state.db).await?;
        Ok(master.map(|item| item.endpoint))
    }
}
