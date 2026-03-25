use anyhow::{Context, Result};
use tauri::Emitter;
use tokio::net::UdpSocket;

use crate::config::AppConfig;
use crate::schemas::control_protocol::{DiscoverAck, DiscoverAckPayload, DiscoverRequest};
use crate::schemas::device_schema::DeviceIpUpdatedEvent;
use crate::utils::datetime::{now_ms};

pub async fn start(app_handle: tauri::AppHandle) -> Result<()> {
    let config = AppConfig::load()?;
    let bind_addr = format!("0.0.0.0:{}", config.listener_port);
    let socket = UdpSocket::bind(&bind_addr)
        .await
        .with_context(|| format!("学生端发现监听启动失败: {}", bind_addr))?;

    println!("[discovery-listener] listening on {}", bind_addr);

    let mut buf = vec![0_u8; 2048];
    loop {
        let (size, peer) = socket.recv_from(&mut buf).await?;
        let text = match std::str::from_utf8(&buf[..size]) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let req: DiscoverRequest = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if req.r#type != "DISCOVER_STUDENT_DEVICES" {
            continue;
        }

        let device_ip = crate::network::device_network::resolve_device_ip()?
            .unwrap_or_else(|| peer.ip().to_string());

        let _ = app_handle.emit(
            "device_ip_updated",
            DeviceIpUpdatedEvent {
                ip: Some(device_ip.clone()),
            },
        );

        let ack = DiscoverAck {
            r#type: "DISCOVER_STUDENT_DEVICES_ACK".to_string(),
            timestamp: now_ms(),
            payload: DiscoverAckPayload {
                device_id: format!("student-{}", std::process::id()),
                ip: device_ip,
                name: "学生端设备".to_string(),
                control_port: config.control_port,
                db_ready: true,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let output = serde_json::to_vec(&ack)?;
        let _ = socket.send_to(&output, peer).await;
    }
}
