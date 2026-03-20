use tauri::State;

use crate::core::setting::SETTINGS;
use crate::schemas::device_schema;
use crate::network::student_control_client;
use crate::schemas::student_exam_schema;
use crate::services::exam_service;
use crate::services::device_service;
use crate::services::student_exam_service;
use crate::state::AppState;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

#[tauri::command]
pub async fn get_students_by_exam_id(
    state: State<'_, AppState>,
    payload: student_exam_schema::GetStudentExamsInput,
) -> Result<Vec<student_exam_schema::ExamStudentDto>, String> {
    let pool = &state.db;
    match student_exam_service::list_student_exams_by_exam_id(pool, payload.exam_id).await {
        Ok(list) => Ok(list
            .into_iter()
            .map(|item| student_exam_schema::ExamStudentDto {
                id: item.id,
                student_no: item.student_no,
                name: item.name,
                created_at: item.created_at,
                updated_at: item.updated_at,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn import_students_by_exam_id(
    state: State<'_, AppState>,
    payload: student_exam_schema::ImportStudentsByExamIdInput,
) -> Result<Vec<student_exam_schema::ExamStudentDto>, String> {
    let pool = &state.db;
    match student_exam_service::import_students_by_exam_id(pool, payload.exam_id, payload.student_ids)
        .await
    {
        Ok(list) => Ok(list
            .into_iter()
            .map(|item| student_exam_schema::ExamStudentDto {
                id: item.id,
                student_no: item.student_no,
                name: item.name,
                created_at: item.created_at,
                updated_at: item.updated_at,
            })
            .collect()),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn get_student_device_assignments_by_exam_id(
    state: State<'_, AppState>,
    payload: student_exam_schema::GetStudentExamsInput,
) -> Result<Vec<student_exam_schema::StudentDeviceAssignDto>, String> {
    let pool = &state.db;
    student_exam_service::list_student_device_assignments_by_exam_id(pool, payload.exam_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn assign_devices_to_student_exams(
    state: State<'_, AppState>,
    payload: student_exam_schema::AssignDevicesToStudentExamsInput,
) -> Result<Vec<student_exam_schema::StudentDeviceAssignDto>, String> {
    let pool = &state.db;
    student_exam_service::assign_devices_to_student_exams(pool, payload.exam_id, payload.assignments)
        .await
        .map_err(|err| err.to_string())
}

fn resolve_local_ipv4() -> Result<String, String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")
        .map_err(|err| format!("获取本机地址失败: {}", err))?;
    socket
        .connect("8.8.8.8:80")
        .map_err(|err| format!("探测本机地址失败: {}", err))?;
    let addr = socket
        .local_addr()
        .map_err(|err| format!("读取本机地址失败: {}", err))?;

    match addr.ip() {
        std::net::IpAddr::V4(ip) => Ok(ip.to_string()),
        std::net::IpAddr::V6(_) => Err("当前环境未获取到 IPv4 地址".to_string()),
    }
}

#[tauri::command]
pub async fn connect_student_devices_by_exam_id(
    state: State<'_, AppState>,
    payload: student_exam_schema::ConnectStudentDevicesByExamInput,
) -> Result<device_schema::PushTeacherEndpointsOutput, String> {
    let pool = &state.db;
    let exam_id = payload.exam_id;

    let exam = exam_service::get_exam_by_id(pool, exam_id.clone())
        .await
        .map_err(|err| err.to_string())?;

    let assignments = student_exam_service::list_student_device_assignments_by_exam_id(
        pool,
        exam_id,
    )
    .await
    .map_err(|err| err.to_string())?;

    let devices = device_service::list_devices(pool, None, None)
        .await
        .map_err(|err| err.to_string())?;

    let mut ip_to_device_id = std::collections::HashMap::with_capacity(devices.len());
    for device in devices {
        ip_to_device_id.insert(device.ip, device.id);
    }

    let mut targets = Vec::new();
    let mut seen_ip = std::collections::HashSet::new();
    for item in assignments {
        let Some(ip) = item.ip_addr.clone() else {
            continue;
        };

        if ip.trim().is_empty() {
            continue;
        }

        // 同一设备 IP 只下发一次，避免重复请求；student_id 以第一条分配记录为准。
        if !seen_ip.insert(ip.clone()) {
            continue;
        }

        let device_id = ip_to_device_id
            .get(&ip)
            .cloned()
            .unwrap_or_else(|| item.student_id.clone());

        targets.push((
            device_id,
            ip,
            item.student_exam_id,
            item.student_id,
            item.student_no,
            item.student_name,
        ));
    }

    if targets.is_empty() {
        return Ok(device_schema::PushTeacherEndpointsOutput {
            request_id: uuid::Uuid::new_v4().to_string(),
            total: 0,
            success_count: 0,
            results: Vec::new(),
        });
    }

    let local_ip = resolve_local_ipv4()?;
    let master_endpoint = format!("ws://{}:{}", local_ip, SETTINGS.ws_server_port);
    let request_id = uuid::Uuid::new_v4().to_string();
    let endpoints = vec![student_control_client::TeacherEndpointInput {
        id: uuid::Uuid::new_v4().to_string(),
        endpoint: master_endpoint,
        name: Some("主教师端".to_string()),
        remark: Some("分配页一键连接考生设备".to_string()),
        is_master: true,
    }];

    let mut results = Vec::with_capacity(targets.len());
    for (device_id, device_ip, student_exam_id, student_id, student_no, student_name) in targets {
        let req = student_control_client::ApplyTeacherEndpointsRequest {
            r#type: "APPLY_TEACHER_ENDPOINTS".to_string(),
            request_id: format!("{}-{}", request_id, device_id),
            timestamp: now_ms(),
            payload: student_control_client::ApplyTeacherEndpointsPayload {
                config_version: Some(1),
                session_id: Some(student_exam_id),
                exam_id: Some(exam.id.clone()),
                exam_title: Some(exam.title.clone()),
                student_id,
                student_no: Some(student_no),
                student_name: Some(student_name),
                assigned_ip_addr: Some(device_ip.clone()),
                assignment_status: Some("assigned".to_string()),
                start_time: exam.start_time,
                end_time: exam.end_time,
                endpoints: endpoints.clone(),
            },
        };

        match student_control_client::apply_teacher_endpoints(&device_ip, SETTINGS.std_controller_port, &req).await {
            Ok(ack) => results.push(device_schema::PushTeacherEndpointsResultItem {
                device_id,
                device_ip,
                success: ack.payload.success,
                message: ack.payload.message,
                connected_master: ack.payload.connected_master,
            }),
            Err(err) => results.push(device_schema::PushTeacherEndpointsResultItem {
                device_id,
                device_ip,
                success: false,
                message: err.to_string(),
                connected_master: None,
            }),
        }
    }

    let success_count = results.iter().filter(|item| item.success).count();
    Ok(device_schema::PushTeacherEndpointsOutput {
        request_id,
        total: results.len(),
        success_count,
        results,
    })
}

#[tauri::command]
pub async fn get_student_device_connection_status_by_exam_id(
    state: State<'_, AppState>,
    payload: student_exam_schema::GetStudentExamsInput,
) -> Result<Vec<student_exam_schema::StudentDeviceConnectionStatusDto>, String> {
    let pool = &state.db;
    let connection_map = state
        .snapshot_connections()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();

    student_exam_service::list_student_device_connection_status_by_exam_id(
        pool,
        payload.exam_id,
        &connection_map,
    )
    .await
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn distribute_exam_papers_by_exam_id(
    state: State<'_, AppState>,
    payload: student_exam_schema::DistributeExamPapersByExamInput,
) -> Result<student_exam_schema::DistributeExamPapersOutput, String> {
    let pool = &state.db;
    let exam_id = payload.exam_id;

    let result = student_exam_service::distribute_exam_papers_by_exam_id(pool, exam_id.clone())
        .await
        .map_err(|err| err.to_string())?;

    if result.total > 0 && result.success_count == result.total {
        exam_service::update_exam_status(pool, exam_id, "published".to_string())
            .await
            .map_err(|err| err.to_string())?;
    }

    Ok(result)
}

#[tauri::command]
pub async fn start_exam_by_exam_id(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    payload: student_exam_schema::StartExamByExamInput,
) -> Result<student_exam_schema::StartExamOutput, String> {
    let pool = &state.db;
    let exam_id = payload.exam_id;

    let result = student_exam_service::start_exam_by_exam_id(&app_handle, pool, exam_id.clone())
        .await
        .map_err(|err| err.to_string())?;

    if result.sent_count > 0 {
        exam_service::update_exam_status(pool, exam_id, "active".to_string())
            .await
            .map_err(|err| err.to_string())?;
    }

    Ok(result)
}
