use crate::utils::datetime::now_ms;

use anyhow::{Context, Result};
use tauri::Emitter;
use tokio::net::TcpStream;

use crate::config::AppConfig;
use crate::network::transport::tcp_request_reply::{
    bind_listener, read_json_request, write_json_response,
};
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
use crate::services::ws_reconnect_service::WsReconnectService;



pub async fn start(app_handle: tauri::AppHandle) -> Result<()> {
    let config = AppConfig::load()?;
    let bind_addr = format!("0.0.0.0:{}", config.control_port);
    let listener = bind_listener(&bind_addr)
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
    // 发卷报文包含完整题目集合，沿用浅封装中的大小限制。
    let raw = read_json_request(&mut stream, 10 * 1024 * 1024).await?;
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

        write_json_response(&mut stream, &ack).await?;
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
        write_json_response(&mut stream, &ack).await?;
        return Ok(());
    }

    let (success, message) = match TeacherEndpointsService::replace_all(&app_handle, &req.payload.endpoints).await {
        Ok(()) => {
            match crate::services::exam_runtime_service::ExamRuntimeService::upsert_connected_session(
                &app_handle,
                &req.payload,
            )
            .await
            {
                Ok(true) => (true, "配置与考生会话已落库".to_string()),
                Ok(false) => (true, "配置已落库（未携带会话信息）".to_string()),
                Err(err) => (false, format!("会话落库失败: {}", err)),
            }
        }
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
            let connect_result = WsReconnectService::start_or_update(
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

    write_json_response(&mut stream, &ack).await?;
    Ok(())
}
