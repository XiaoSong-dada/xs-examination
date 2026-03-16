use anyhow::Result;
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use tauri::AppHandle;

pub struct AppState {
    pub db: DatabaseConnection,
    pub connections: DashMap<String, i64>,
}

impl AppState {
    pub async fn new(app_handle: &AppHandle) -> Result<Self> {
        let db = crate::db::init(app_handle).await?;
        Ok(Self {
            db,
            connections: DashMap::new(),
        })
    }

    pub fn touch_connection(&self, student_id: &str, timestamp_ms: i64) {
        self.connections.insert(student_id.to_string(), timestamp_ms);
    }

    pub fn snapshot_connections(&self) -> Vec<(String, i64)> {
        self.connections
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect()
    }
}
