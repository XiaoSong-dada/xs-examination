use anyhow::Result;
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use tauri::AppHandle;

pub struct AppState {
    pub db: DatabaseConnection,
    pub connections: DashMap<String, String>,
}

impl AppState {
    pub async fn new(app_handle: &AppHandle) -> Result<Self> {
        let db = crate::db::init(app_handle).await?;
        Ok(Self {
            db,
            connections: DashMap::new(),
        })
    }
}
