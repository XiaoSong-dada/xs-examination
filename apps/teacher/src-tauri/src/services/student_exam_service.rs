use anyhow::Result;
use sea_orm::DatabaseConnection;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use tauri::Manager;
use tokio::time::{sleep, Duration};

use crate::models::student::Model as StudentModel;
use crate::core::setting::SETTINGS;
use crate::network::protocol::{ExamEndPayload, ExamStartPayload, FinalSyncRequestPayload};
use crate::network::student_control_client;
use crate::schemas::student_exam_schema;
use crate::repos::student_exam_repo;
use crate::services::{exam_service, question_service};
use crate::utils::datetime::now_ms;

const HEARTBEAT_TIMEOUT_MS: i64 = 15_000;

fn derive_connection_status(ip_addr: Option<&str>, last_heartbeat_at: Option<i64>, now: i64) -> (String, bool) {
    if ip_addr.map(|v| v.trim().is_empty()).unwrap_or(true) {
        return ("待分配".to_string(), false);
    }

    match last_heartbeat_at {
        None => ("未连接".to_string(), false),
        Some(last) => {
            if now - last <= HEARTBEAT_TIMEOUT_MS {
                ("正常".to_string(), true)
            } else {
                ("异常".to_string(), true)
            }
        }
    }
}

pub async fn list_student_exams_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
) -> Result<Vec<StudentModel>> {
    student_exam_repo::get_students_by_exam_id(db, &exam_id).await
}

pub async fn import_students_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
    student_ids: Vec<String>,
) -> Result<Vec<StudentModel>> {
    let mut seen = HashSet::new();
    let normalized_student_ids: Vec<String> = student_ids
        .into_iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty() && seen.insert(id.clone()))
        .collect();

    student_exam_repo::replace_students_by_exam_id(db, &exam_id, normalized_student_ids).await
}

pub async fn list_student_device_assignments_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
) -> Result<Vec<student_exam_schema::StudentDeviceAssignDto>> {
    student_exam_repo::get_student_device_assignments_by_exam_id(db, &exam_id).await
}

pub async fn assign_devices_to_student_exams(
    db: &DatabaseConnection,
    exam_id: String,
    assignments: Vec<student_exam_schema::AssignStudentDeviceItem>,
) -> Result<Vec<student_exam_schema::StudentDeviceAssignDto>> {
    student_exam_repo::assign_devices_to_student_exams(db, &exam_id, assignments).await
}

pub async fn list_student_device_connection_status_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
    connection_map: &HashMap<String, i64>,
) -> Result<Vec<student_exam_schema::StudentDeviceConnectionStatusDto>> {
    let assignments =
        student_exam_repo::get_student_device_assignments_by_exam_id(db, &exam_id).await?;
    let progress_map = student_exam_repo::get_student_answer_progress_by_exam_id(db, &exam_id).await?;
    let now = now_ms();

    Ok(assignments
        .into_iter()
        .map(|item| {
            let last_heartbeat_at = connection_map.get(&item.student_id).copied();
            let (connection_status, has_heartbeat_seen) =
                derive_connection_status(item.ip_addr.as_deref(), last_heartbeat_at, now);
            let (answered_count, total_questions, progress_percent) =
                progress_map.get(&item.student_id).copied().unwrap_or((0, 0, 0));

            student_exam_schema::StudentDeviceConnectionStatusDto {
                student_exam_id: item.student_exam_id,
                student_id: item.student_id,
                student_no: item.student_no,
                student_name: item.student_name,
                ip_addr: item.ip_addr,
                device_name: item.device_name,
                connection_status,
                last_heartbeat_at,
                has_heartbeat_seen,
                answered_count,
                total_questions,
                progress_percent,
            }
        })
        .collect())
}

pub async fn distribute_exam_papers_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
) -> Result<student_exam_schema::DistributeExamPapersOutput> {
    let exam = exam_service::get_exam_by_id(db, exam_id.clone()).await?;
    let questions = question_service::list_questions(db, exam_id.clone()).await?;
    let assignments =
        student_exam_repo::get_student_device_assignments_by_exam_id(db, &exam_id).await?;

    let request_id = uuid::Uuid::new_v4().to_string();
    let now = now_ms();

    let exam_meta = json!({
        "id": exam.id,
        "title": exam.title,
        "description": exam.description,
        "startTime": exam.start_time,
        "endTime": exam.end_time,
        "status": exam.status,
        "passScore": exam.pass_score,
        "shuffleQuestions": exam.shuffle_questions,
        "shuffleOptions": exam.shuffle_options,
    })
    .to_string();

    let questions_payload = json!(
        questions
            .into_iter()
            .map(|q| {
                json!({
                    "id": q.id,
                    "seq": q.seq,
                    "type": q.r#type,
                    "content": q.content,
                    "options": q.options,
                    "score": q.score,
                    "explanation": q.explanation,
                })
            })
            .collect::<Vec<_>>()
    )
    .to_string();

    let mut results = Vec::new();
    for item in assignments {
        let Some(device_ip) = item.ip_addr.clone() else {
            continue;
        };
        if device_ip.trim().is_empty() {
            continue;
        }

        let req = student_control_client::DistributeExamPaperRequest {
            r#type: "DISTRIBUTE_EXAM_PAPER".to_string(),
            request_id: format!("{}-{}", request_id, item.student_exam_id),
            timestamp: now,
            payload: student_control_client::DistributeExamPaperPayload {
                session_id: item.student_exam_id.clone(),
                exam_id: exam_id.clone(),
                student_id: item.student_id.clone(),
                student_no: item.student_no.clone(),
                student_name: item.student_name.clone(),
                assigned_ip_addr: device_ip.clone(),
                exam_title: exam.title.clone(),
                status: "waiting".to_string(),
                assignment_status: "assigned".to_string(),
                start_time: exam.start_time,
                end_time: exam.end_time,
                paper_version: Some(exam.updated_at.to_string()),
                exam_meta: exam_meta.clone(),
                questions_payload: questions_payload.clone(),
                downloaded_at: now,
                expires_at: exam.end_time,
            },
        };

        let control_port = SETTINGS.std_controller_port;
        match student_control_client::distribute_exam_paper(&device_ip, control_port, &req).await {
            Ok(ack) => results.push(student_exam_schema::DistributeExamPapersResultItem {
                student_exam_id: item.student_exam_id,
                student_id: item.student_id,
                device_ip,
                success: ack.payload.success,
                message: ack.payload.message,
                session_id: ack.payload.session_id,
            }),
            Err(err) => results.push(student_exam_schema::DistributeExamPapersResultItem {
                student_exam_id: item.student_exam_id,
                student_id: item.student_id,
                device_ip,
                success: false,
                message: err.to_string(),
                session_id: None,
            }),
        }
    }

    let success_count = results.iter().filter(|item| item.success).count();
    Ok(student_exam_schema::DistributeExamPapersOutput {
        request_id,
        total: results.len(),
        success_count,
        results,
    })
}

pub async fn start_exam_by_exam_id(
    app_handle: &tauri::AppHandle,
    db: &DatabaseConnection,
    exam_id: String,
) -> Result<student_exam_schema::StartExamOutput> {
    let exam = exam_service::get_exam_by_id(db, exam_id.clone()).await?;
    let assignments =
        student_exam_repo::get_student_device_assignments_by_exam_id(db, &exam_id).await?;
    let now = now_ms();

    let mut sent_count = 0usize;
    let mut total_targets = 0usize;
    for item in assignments {
        if item.ip_addr.as_deref().map(|v| !v.trim().is_empty()).unwrap_or(false) {
            total_targets += 1;
            let target_student_id = item.student_id.clone();
            let payload = ExamStartPayload {
                exam_id: exam_id.clone(),
                student_id: target_student_id.clone(),
                start_time: now,
                end_time: exam.end_time,
                timestamp: now,
            };

            let delivered = crate::network::ws_server::send_exam_start_to_student(app_handle, payload)?;
            println!(
                "[start-exam] exam_id={} student_id={} delivered={}",
                exam_id, target_student_id, delivered
            );
            if delivered {
                sent_count += 1;
            }
        }
    }

    Ok(student_exam_schema::StartExamOutput {
        exam_id,
        total_targets,
        sent_count,
    })
}

/// 结束指定考试：向在线考生发送最终同步请求与结束考试指令，并等待最终同步 ACK 聚合。
///
/// # 参数
/// * `app_handle` - Tauri 应用句柄，用于访问运行时状态与 WebSocket 发送能力。
/// * `db` - 数据库连接，用于读取考试与考生分配信息。
/// * `exam_id` - 目标考试 ID。
///
/// # 返回值
/// 返回结束考试发送与聚合结果；当且仅当在线目标全部完成 final ACK 才会在服务内部把考试状态更新为 `finished`。
pub async fn end_exam_by_exam_id(
    app_handle: &tauri::AppHandle,
    db: &DatabaseConnection,
    exam_id: String,
) -> Result<student_exam_schema::EndExamOutput> {
    let _ = exam_service::get_exam_by_id(db, exam_id.clone()).await?;
    let assignments =
        student_exam_repo::get_student_device_assignments_by_exam_id(db, &exam_id).await?;
    let state = app_handle.state::<crate::state::AppState>();
    let connection_map = state
        .snapshot_connections()
        .into_iter()
        .collect::<HashMap<_, _>>();

    let request_id = uuid::Uuid::new_v4().to_string();
    let now = now_ms();
    let mut total_targets = 0usize;
    let mut sent_count = 0usize;
    let mut expected_batch_ids: Vec<String> = Vec::new();

    for item in assignments {
        let Some(ip_addr) = item.ip_addr.as_deref() else {
            continue;
        };
        if ip_addr.trim().is_empty() {
            continue;
        }

        let (status, _) = derive_connection_status(
            item.ip_addr.as_deref(),
            connection_map.get(&item.student_id).copied(),
            now,
        );
        if status != "正常" {
            continue;
        }

        total_targets += 1;
        let batch_id = format!("{}:{}", request_id, item.student_id);

        let final_sync_delivered = crate::network::ws_server::send_final_sync_request_to_student(
            app_handle,
            FinalSyncRequestPayload {
                exam_id: exam_id.clone(),
                student_id: item.student_id.clone(),
                session_id: item.student_exam_id.clone(),
                batch_id: batch_id.clone(),
                timestamp: now,
            },
        )?;

        let end_delivered = crate::network::ws_server::send_exam_end_to_student(
            app_handle,
            ExamEndPayload {
                exam_id: exam_id.clone(),
                student_id: item.student_id.clone(),
                end_time: now,
                final_batch_id: batch_id.clone(),
                timestamp: now,
            },
        )?;

        if final_sync_delivered && end_delivered {
            sent_count += 1;
            expected_batch_ids.push(batch_id);
        }
    }

    let mut acked_count = 0usize;
    if !expected_batch_ids.is_empty() {
        let started_at = now_ms();
        let timeout_ms = 10_000;

        loop {
            acked_count = expected_batch_ids
                .iter()
                .filter(|batch_id| state.has_final_sync_received(batch_id))
                .count();
            if acked_count == expected_batch_ids.len() {
                break;
            }

            if now_ms() - started_at >= timeout_ms {
                break;
            }

            sleep(Duration::from_millis(200)).await;
        }
    }

    if total_targets == 0 || (sent_count == total_targets && acked_count == total_targets) {
        exam_service::update_exam_status(db, exam_id.clone(), "finished".to_string()).await?;
    }

    state.clear_final_sync_tracking(&expected_batch_ids);

    Ok(student_exam_schema::EndExamOutput {
        request_id,
        exam_id,
        total_targets,
        sent_count,
        acked_count,
        failed_count: total_targets.saturating_sub(acked_count),
    })
}

/// 查询指定考试的成绩汇总。
///
/// # 参数
/// * `db` - 数据库连接。
/// * `exam_id` - 考试 ID。
///
/// # 返回值
/// 返回该考试当前已落库的成绩汇总；查询失败时返回错误。
pub async fn list_student_score_summary_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
) -> Result<Vec<student_exam_schema::StudentScoreSummaryDto>> {
    student_exam_repo::get_student_score_summary_by_exam_id(db, &exam_id).await
}

/// 对指定考试执行成绩重算并覆盖写入。
///
/// # 参数
/// * `db` - 数据库连接。
/// * `exam_id` - 考试 ID。
///
/// # 返回值
/// 返回最新成绩汇总列表；当考试状态不是 `finished` 时返回业务错误。
pub async fn recalculate_student_score_summary_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
) -> Result<Vec<student_exam_schema::StudentScoreSummaryDto>> {
    let exam = exam_service::get_exam_by_id(db, exam_id.clone()).await?;
    if exam.status != "finished" {
        return Err(anyhow::anyhow!("仅已结束考试可统计成绩"));
    }

    student_exam_repo::recalculate_student_score_summary_by_exam_id(db, &exam_id).await
}
