use crate::utils::datetime::now_ms;

use anyhow::{bail, Context, Result};
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::config::AppConfig;
use crate::schemas::control_protocol::{
    ApplyTeacherEndpointsAck,
    ApplyTeacherEndpointsAckPayload,
    ApplyTeacherEndpointsRequest,
    DistributeExamPaperAck,
    DistributeExamPaperAckPayload,
    DistributeExamPaperRequest,
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

async fn read_json_request(stream: &mut TcpStream) -> Result<serde_json::Value> {
    // 发卷报文包含完整题目集合，可能超过单次 read 缓冲区，需循环读取直到 JSON 完整。
    const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;
    let mut data = Vec::with_capacity(16 * 1024);
    let mut chunk = [0_u8; 4096];

    loop {
        let size = stream.read(&mut chunk).await?;
        if size == 0 {
            break;
        }

        data.extend_from_slice(&chunk[..size]);
        if data.len() > MAX_REQUEST_SIZE {
            eprintln!(
                "[control-server] request too large: {} bytes (max={})",
                data.len(),
                MAX_REQUEST_SIZE
            );
            bail!("控制消息过大: {} bytes", data.len());
        }

        match serde_json::from_slice::<serde_json::Value>(&data) {
            Ok(value) => return Ok(value),
            Err(err) if err.is_eof() => continue,
            Err(err) => return Err(err.into()),
        }
    }

    if data.is_empty() {
        bail!("空请求体");
    }

    Ok(serde_json::from_slice::<serde_json::Value>(&data)?)
}

async fn handle_client(app_handle: tauri::AppHandle, mut stream: TcpStream) -> Result<()> {
    let raw = read_json_request(&mut stream).await?;
    let req_type = raw
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    if req_type == "DISTRIBUTE_EXAM_PAPER" {
        let req: DistributeExamPaperRequest = serde_json::from_value(raw)?;
        eprintln!(
            "[control-server] receive DISTRIBUTE_EXAM_PAPER request_id={} session_id={} questions_size={}",
            req.request_id,
            req.payload.session_id,
            req.payload.questions_payload.len()
        );

        let result = crate::services::exam_runtime_service::ExamRuntimeService::upsert_distribution(
            &app_handle,
            &req.payload,
        )
        .await;

        let (success, message) = match result {
            Ok(()) => (true, "试卷已落库".to_string()),
            Err(err) => {
                eprintln!("[control-server] distribute persist failed: {}", err);
                (false, format!("试卷落库失败: {}", err))
            }
        };

        let ack = DistributeExamPaperAck {
            r#type: "DISTRIBUTE_EXAM_PAPER_ACK".to_string(),
            request_id: req.request_id,
            timestamp: now_ms(),
            payload: DistributeExamPaperAckPayload {
                success,
                message,
                session_id: Some(req.payload.session_id),
            },
        };

        let output = serde_json::to_vec(&ack)?;
        stream.write_all(&output).await?;
        return Ok(());
    }

    let req: ApplyTeacherEndpointsRequest = serde_json::from_value(raw)?;

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
