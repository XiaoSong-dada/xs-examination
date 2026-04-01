use anyhow::Result;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::AppHandle;
use tokio::task::JoinHandle;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct ReceivingPackageState {
    pub exam_id: String,
    pub student_id: String,
    pub session_id: String,
    pub batch_id: String,
    pub file_path: String,
    pub sha256: String,
    pub total_bytes: u64,
    pub total_chunks: u32,
    pub received_chunks: u32,
}

pub struct AppState {
    pub db: DatabaseConnection,
    ws_sender: Mutex<Option<UnboundedSender<String>>>,
    ws_connected: AtomicBool,
    ws_endpoint: Mutex<Option<String>>,
    reconnect_target: Mutex<Option<(String, String)>>,
    reconnect_task: Mutex<Option<JoinHandle<()>>>,
    last_full_sync_marker: Mutex<Option<(String, i64)>>,
    receiving_packages: Mutex<HashMap<String, ReceivingPackageState>>,
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
            last_full_sync_marker: Mutex::new(None),
            receiving_packages: Mutex::new(HashMap::new()),
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

    pub fn should_send_full_sync(&self, session_id: &str, now_ms: i64, cooldown_ms: i64) -> bool {
        if let Ok(mut guard) = self.last_full_sync_marker.lock() {
            if let Some((last_session_id, last_ts)) = guard.as_ref() {
                if last_session_id == session_id && now_ms - *last_ts < cooldown_ms {
                    return false;
                }
            }

            *guard = Some((session_id.to_string(), now_ms));
            return true;
        }

        true
    }

    pub fn set_receiving_package(&self, batch_id: String, state: ReceivingPackageState) {
        if let Ok(mut guard) = self.receiving_packages.lock() {
            guard.insert(batch_id, state);
        }
    }

    pub fn get_receiving_package(&self, batch_id: &str) -> Option<ReceivingPackageState> {
        self.receiving_packages
            .lock()
            .ok()
            .and_then(|guard| guard.get(batch_id).cloned())
    }

    pub fn update_receiving_package<F>(&self, batch_id: &str, mut updater: F) -> Option<ReceivingPackageState>
    where
        F: FnMut(&mut ReceivingPackageState),
    {
        let Ok(mut guard) = self.receiving_packages.lock() else {
            return None;
        };
        let Some(state) = guard.get_mut(batch_id) else {
            return None;
        };
        updater(state);
        Some(state.clone())
    }

    pub fn remove_receiving_package(&self, batch_id: &str) -> Option<ReceivingPackageState> {
        self.receiving_packages
            .lock()
            .ok()
            .and_then(|mut guard| guard.remove(batch_id))
    }
}