use anyhow::Result;
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use tauri::AppHandle;
use tokio::sync::mpsc::UnboundedSender;
use crate::network::p2p_distributor::P2PDistributor;

pub struct AppState {
    pub db: DatabaseConnection,
    pub connections: DashMap<String, i64>,
    ws_peers: DashMap<String, UnboundedSender<String>>,
    student_peer_map: DashMap<String, String>,
    final_sync_tracker: DashMap<String, bool>,
    paper_package_ack_tracker: DashMap<String, String>,
    pub p2p_distributor: P2PDistributor,
}

impl AppState {
    pub async fn new(app_handle: &AppHandle) -> Result<Self> {
        let db = crate::db::init(app_handle).await?;
        Ok(Self {
            db,
            connections: DashMap::new(),
            ws_peers: DashMap::new(),
            student_peer_map: DashMap::new(),
            final_sync_tracker: DashMap::new(),
            paper_package_ack_tracker: DashMap::new(),
            p2p_distributor: P2PDistributor::new(),
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

    pub fn send_ws_text_to_peer(&self, peer_id: &str, text: String) -> bool {
        let Some(sender) = self
            .ws_peers
            .get(peer_id)
            .map(|entry| entry.value().clone())
        else {
            return false;
        };

        if sender.send(text).is_ok() {
            true
        } else {
            self.ws_peers.remove(peer_id);
            self.student_peer_map.retain(|_, mapped_peer| mapped_peer != peer_id);
            false
        }
    }

    pub fn mark_final_sync_received(&self, batch_id: &str) {
        self.final_sync_tracker.insert(batch_id.to_string(), true);
    }

    pub fn has_final_sync_received(&self, batch_id: &str) -> bool {
        self.final_sync_tracker
            .get(batch_id)
            .map(|entry| *entry.value())
            .unwrap_or(false)
    }

    pub fn clear_final_sync_tracking(&self, batch_ids: &[String]) {
        for batch_id in batch_ids {
            self.final_sync_tracker.remove(batch_id);
        }
    }

    pub fn mark_paper_package_ack(&self, batch_id: &str, message: String) {
        self.paper_package_ack_tracker
            .insert(batch_id.to_string(), message);
    }

    pub fn take_paper_package_ack(&self, batch_id: &str) -> Option<String> {
        self.paper_package_ack_tracker
            .remove(batch_id)
            .map(|(_, value)| value)
    }
}
