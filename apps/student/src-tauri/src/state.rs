use anyhow::Result;
use sea_orm::DatabaseConnection;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::AppHandle;
use tokio::task::JoinHandle;
use tokio::sync::mpsc::UnboundedSender;

pub struct AppState {
    pub db: DatabaseConnection,
    ws_sender: Mutex<Option<UnboundedSender<String>>>,
    ws_connected: AtomicBool,
    ws_endpoint: Mutex<Option<String>>,
    reconnect_target: Mutex<Option<(String, String)>>,
    reconnect_task: Mutex<Option<JoinHandle<()>>>,
}

impl AppState {
    pub async fn new(app_handle: &AppHandle) -> Result<Self> {
        let db = crate::db::init(app_handle).await?;
        Ok(Self {
            db,
            ws_sender: Mutex::new(None),
            ws_connected: AtomicBool::new(false),
            ws_endpoint: Mutex::new(None),
            reconnect_target: Mutex::new(None),
            reconnect_task: Mutex::new(None),
        })
    }

    pub fn set_ws_sender(&self, sender: UnboundedSender<String>) {
        if let Ok(mut guard) = self.ws_sender.lock() {
            *guard = Some(sender);
        }
    }

    pub fn clear_ws_sender(&self) {
        if let Ok(mut guard) = self.ws_sender.lock() {
            *guard = None;
        }
    }

    pub fn set_ws_endpoint(&self, endpoint: String) {
        if let Ok(mut guard) = self.ws_endpoint.lock() {
            *guard = Some(endpoint);
        }
    }

    pub fn clear_ws_endpoint(&self) {
        if let Ok(mut guard) = self.ws_endpoint.lock() {
            *guard = None;
        }
    }

    pub fn ws_endpoint(&self) -> Option<String> {
        self.ws_endpoint
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned())
    }

    pub fn ws_sender(&self) -> Option<UnboundedSender<String>> {
        self.ws_sender
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned())
    }

    pub fn set_ws_connected(&self, connected: bool) {
        self.ws_connected.store(connected, Ordering::SeqCst);
    }

    pub fn ws_connected(&self) -> bool {
        self.ws_connected.load(Ordering::SeqCst)
    }

    pub fn set_reconnect_target(&self, endpoint: String, student_id: String) {
        if let Ok(mut guard) = self.reconnect_target.lock() {
            *guard = Some((endpoint, student_id));
        }
    }

    pub fn reconnect_target(&self) -> Option<(String, String)> {
        self.reconnect_target
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned())
    }

    pub fn replace_reconnect_task(&self, next_task: JoinHandle<()>) {
        if let Ok(mut guard) = self.reconnect_task.lock() {
            if let Some(existing) = guard.take() {
                existing.abort();
            }
            *guard = Some(next_task);
        }
    }
}