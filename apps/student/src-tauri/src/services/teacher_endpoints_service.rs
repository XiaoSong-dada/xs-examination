use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use tauri::Manager;

use crate::db::entities::teacher_endpoints;
use crate::schemas::control_protocol::TeacherEndpointInput;

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
        let txn = state.db.begin().await?;

        teacher_endpoints::Entity::delete_many().exec(&txn).await?;

        let ts = now_ms();
        for endpoint in endpoints {
            let model = teacher_endpoints::ActiveModel {
                id: Set(endpoint.id.clone()),
                endpoint: Set(endpoint.endpoint.clone()),
                name: Set(endpoint.name.clone()),
                remark: Set(endpoint.remark.clone()),
                is_master: Set(if endpoint.is_master { 1 } else { 0 }),
                last_seen: Set(None),
                created_at: Set(ts),
                updated_at: Set(ts),
            };
            model.insert(&txn).await?;
        }

        txn.commit().await?;
        Ok(())
    }

    pub fn master_endpoint(endpoints: &[TeacherEndpointInput]) -> Option<String> {
        endpoints
            .iter()
            .find(|e| e.is_master)
            .map(|e| e.endpoint.clone())
    }

    pub async fn get_master_endpoint(app_handle: &tauri::AppHandle) -> Result<Option<String>> {
        let state = app_handle.state::<crate::state::AppState>();
        let master = teacher_endpoints::Entity::find()
            .filter(teacher_endpoints::Column::IsMaster.eq(1))
            .one(&state.db)
            .await?;

        Ok(master.map(|item| item.endpoint))
    }
}
