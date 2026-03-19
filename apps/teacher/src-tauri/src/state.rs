use anyhow::Result;
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use tauri::AppHandle;
use tokio::sync::mpsc::UnboundedSender;

pub struct AppState {
    pub db: DatabaseConnection,
    pub connections: DashMap<String, i64>,
    ws_peers: DashMap<String, UnboundedSender<String>>,
    student_peer_map: DashMap<String, String>,
}

impl AppState {
    pub async fn new(app_handle: &AppHandle) -> Result<Self> {
        let db = crate::db::init(app_handle).await?;
        Ok(Self {
            db,
            connections: DashMap::new(),
            ws_peers: DashMap::new(),
            student_peer_map: DashMap::new(),
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

    pub fn register_ws_peer(&self, peer_id: String, sender: UnboundedSender<String>) {
        self.ws_peers.insert(peer_id, sender);
    }

    pub fn remove_ws_peer(&self, peer_id: &str) {
        self.ws_peers.remove(peer_id);
        self.student_peer_map.retain(|_, mapped_peer| mapped_peer != peer_id);
    }

    pub fn bind_student_peer(&self, student_id: &str, peer_id: &str) {
        self.student_peer_map
            .insert(student_id.to_string(), peer_id.to_string());
    }

    pub fn broadcast_ws_text(&self, text: String) -> usize {
        let mut sent_count = 0usize;
        let mut stale_peers = Vec::new();

        for entry in self.ws_peers.iter() {
            if entry.value().send(text.clone()).is_ok() {
                sent_count += 1;
            } else {
                stale_peers.push(entry.key().clone());
            }
        }

        for peer_id in stale_peers {
            self.ws_peers.remove(&peer_id);
        }

        sent_count
    }

    pub fn send_ws_text_to_student(&self, student_id: &str, text: String) -> bool {
        let Some(peer_id) = self
            .student_peer_map
            .get(student_id)
            .map(|entry| entry.value().clone())
        else {
            return false;
        };

        let Some(sender) = self
            .ws_peers
            .get(&peer_id)
            .map(|entry| entry.value().clone())
        else {
            self.student_peer_map.remove(student_id);
            return false;
        };

        if sender.send(text).is_ok() {
            true
        } else {
            self.ws_peers.remove(&peer_id);
            self.student_peer_map.remove(student_id);
            false
        }
    }
}
