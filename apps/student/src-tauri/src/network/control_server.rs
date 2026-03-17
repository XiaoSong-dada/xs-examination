use crate::utils::datetime::now_ms;

use anyhow::{Context, Result};
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::config::AppConfig;
use crate::schemas::control_protocol::{
    ApplyTeacherEndpointsAck, ApplyTeacherEndpointsAckPayload, ApplyTeacherEndpointsRequest,
};
use crate::schemas::teacher_endpoint_schema::{
    TeacherEndpointAppliedEvent, WsConnectionEvent,
};
use crate::services::teacher_endpoints_service::TeacherEndpointsService;



pub async fn start(app_handle: tauri::AppHandle) -> Result<()> {
    let config = AppConfig::load()?;
    let bind_addr = format!("0.0.0.0:{}", config.control_port);
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("学生端控制服务启动失败: {}", bind_addr))?;

    println!("[control-server] listening on {}", bind_addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let app_handle = app_handle.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(app_handle, stream).await {
                eprintln!("[control-server] handle client {} failed: {}", peer_addr, err);
            }
        });
    }
}

async fn handle_client(app_handle: tauri::AppHandle, mut stream: TcpStream) -> Result<()> {
    let mut buf = vec![0_u8; 16 * 1024];
    let size = stream.read(&mut buf).await?;
    if size == 0 {
        return Ok(());
    }

    let req: ApplyTeacherEndpointsRequest = serde_json::from_slice(&buf[..size])?;

    if req.r#type != "APPLY_TEACHER_ENDPOINTS" {
        let ack = ApplyTeacherEndpointsAck {
            r#type: "APPLY_TEACHER_ENDPOINTS_ACK".to_string(),
            request_id: req.request_id,
            timestamp: now_ms(),
            payload: ApplyTeacherEndpointsAckPayload {
                success: false,
                message: "不支持的消息类型".to_string(),
                connected_master: None,
            },
        };
        let output = serde_json::to_vec(&ack)?;
        stream.write_all(&output).await?;
        return Ok(());
    }

    let result = TeacherEndpointsService::replace_all(&app_handle, &req.payload.endpoints).await;

    let (success, message) = match result {
        Ok(()) => (true, "配置已落库".to_string()),
        Err(err) => (false, format!("配置落库失败: {}", err)),
    };

    let connected_master = if success {
        TeacherEndpointsService::master_endpoint(&req.payload.endpoints)
    } else {
        None
    };

    if success {
        let _ = app_handle.emit(
            "teacher_endpoint_applied",
            TeacherEndpointAppliedEvent {
                endpoint: connected_master.clone(),
            },
        );
    }

    if success {
        if let Some(master_url) = &connected_master {
            // 骨架阶段：收到配置后立刻尝试连接主教师端，不做重试策略。
            let connect_result = crate::network::ws_client::connect(
                app_handle.clone(),
                master_url.clone(),
                req.payload.student_id.clone(),
            )
            .await;

            if let Err(err) = connect_result {
                let _ = app_handle.emit(
                    "ws_disconnected",
                    WsConnectionEvent {
                        endpoint: Some(master_url.clone()),
                        connected: false,
                        message: Some(err.to_string()),
                    },
                );
            }
        }
    }

    let ack = ApplyTeacherEndpointsAck {
        r#type: "APPLY_TEACHER_ENDPOINTS_ACK".to_string(),
        request_id: req.request_id,
        timestamp: now_ms(),
        payload: ApplyTeacherEndpointsAckPayload {
            success,
            message,
            connected_master,
        },
    };

    let output = serde_json::to_vec(&ack)?;
    stream.write_all(&output).await?;
    Ok(())
}
