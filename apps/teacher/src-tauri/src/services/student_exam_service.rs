use anyhow::Result;
use base64::Engine;
use rust_xlsxwriter::Workbook;
use sea_orm::DatabaseConnection;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tauri::{Manager, Emitter};
use tokio::time::{sleep, Duration};

use crate::models::student::Model as StudentModel;
use crate::network::protocol::{ExamEndPayload, ExamStartPayload, FinalSyncRequestPayload};
use crate::network::protocol::{PaperPackageChunkPayload, PaperPackageManifestPayload};
use crate::schemas::student_exam_schema;
use crate::repos::student_exam_repo;
use crate::services::{exam_service, question_service};
use crate::utils::datetime::now_ms;
use crate::utils::asset_zip::{create_asset_zip, ZipAssetEntry};

const HEARTBEAT_TIMEOUT_MS: i64 = 15_000;
const PACKAGE_CHUNK_SIZE: usize = 64 * 1024;

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
    app_handle: &tauri::AppHandle,
    db: &DatabaseConnection,
    exam_id: String,
) -> Result<student_exam_schema::DistributeExamPapersOutput> {
    let exam = exam_service::get_exam_by_id(db, exam_id.clone()).await?;
    let questions = question_service::list_questions(db, exam_id.clone()).await?;
    let assignments = student_exam_repo::get_student_device_assignments_by_exam_id(db, &exam_id).await?;

    let request_id = uuid::Uuid::new_v4().to_string();
    let now = now_ms();

    let package = build_exam_package_zip(app_handle, &questions)?;

    // 发送分发开始事件
    send_distribute_progress(app_handle, &exam_id, 0, assignments.len(), "开始分发试卷".to_string());

    // 并行处理每个学生的分发
    let mut handles = Vec::new();
    for (index, item) in assignments.iter().enumerate() {
        let Some(device_ip) = item.ip_addr.clone() else {
            continue;
        };
        if device_ip.trim().is_empty() {
            continue;
        }

        let app_handle_clone = app_handle.clone();
        let exam_id_clone = exam_id.clone();
        let exam_clone = exam.clone();
        let package_clone = package.clone();
        let item_clone = item.clone();
        let request_id_clone = request_id.clone();
        let total_students = assignments.len();

        handles.push(tokio::spawn(async move {
            let batch_id = format!("{}:{}", request_id_clone, item_clone.student_id);
            let manifest_sent = crate::network::ws_server::send_paper_package_manifest_to_student(
                &app_handle_clone,
                PaperPackageManifestPayload {
                    exam_id: exam_id_clone.clone(),
                    student_id: item_clone.student_id.clone(),
                    session_id: item_clone.student_exam_id.clone(),
                    batch_id: batch_id.clone(),
                    file_name: package_clone.file_name.clone(),
                    total_bytes: package_clone.total_bytes,
                    total_chunks: package_clone.total_chunks,
                    sha256: package_clone.sha256.clone(),
                    exam_title: exam_clone.title.clone(),
                    assignment_status: "assigned".to_string(),
                    start_time: exam_clone.start_time,
                    end_time: exam_clone.end_time,
                    paper_version: Some(exam_clone.updated_at.to_string()),
                    timestamp: now_ms(),
                },
            );

            if let Err(_) | Ok(false) = manifest_sent {
                return student_exam_schema::DistributeExamPapersResultItem {
                    student_exam_id: item_clone.student_exam_id,
                    student_id: item_clone.student_id,
                    device_ip: device_ip.clone(),
                    success: false,
                    message: "学生端未在线，无法下发试卷包".to_string(),
                    session_id: None,
                };
            }

            // 并行发送所有chunk
            let mut chunk_handles = Vec::new();
            for (chunk_index, chunk) in package_clone.zip_bytes.chunks(PACKAGE_CHUNK_SIZE).enumerate() {
                let app_handle_clone = app_handle_clone.clone();
                let exam_id_clone = exam_id_clone.clone();
                let item_clone = item_clone.clone();
                let batch_id_clone = batch_id.clone();
                let chunk_data: Vec<u8> = chunk.to_vec();
                let total_chunks = package_clone.total_chunks;

                chunk_handles.push(tokio::spawn(async move {
                    let payload = PaperPackageChunkPayload {
                        exam_id: exam_id_clone,
                        student_id: item_clone.student_id,
                        session_id: item_clone.student_exam_id,
                        batch_id: batch_id_clone,
                        chunk_index: chunk_index as u32,
                        total_chunks,
                        content_base64: base64::engine::general_purpose::STANDARD.encode(&chunk_data),
                        is_last: (chunk_index as u32 + 1) == total_chunks,
                        timestamp: now_ms(),
                    };

                    crate::network::ws_server::send_paper_package_chunk_to_student(&app_handle_clone, payload)
                }));
            }

            // 等待所有chunk发送完成
            let mut send_failed = None;
            for handle in chunk_handles {
                if let Ok(Ok(false)) = handle.await {
                    send_failed = Some("试卷包分片发送失败，连接已断开".to_string());
                    break;
                }
            }

            if let Some(message) = send_failed {
                return student_exam_schema::DistributeExamPapersResultItem {
                    student_exam_id: item_clone.student_exam_id,
                    student_id: item_clone.student_id,
                    device_ip: device_ip.clone(),
                    success: false,
                    message,
                    session_id: None,
                };
            }

            let ack = wait_for_paper_package_ack(&app_handle_clone, &batch_id).await;
            let (success, message) = match ack {
                Some(raw) => {
                    if let Some(rest) = raw.strip_prefix("ok|") {
                        (true, rest.split('|').next().unwrap_or("发卷成功").to_string())
                    } else if let Some(rest) = raw.strip_prefix("fail|") {
                        (false, rest.split('|').next().unwrap_or("学生端接收失败").to_string())
                    } else {
                        (false, "学生端ACK格式异常".to_string())
                    }
                }
                None => (false, "等待学生端试卷包ACK超时".to_string()),
            };

            // 发送进度更新
            send_distribute_progress(&app_handle_clone, &exam_id_clone, index + 1, total_students, 
                format!("已完成 {} 的分发", item_clone.student_name));

            student_exam_schema::DistributeExamPapersResultItem {
                student_exam_id: item_clone.student_exam_id.clone(),
                student_id: item_clone.student_id.clone(),
                device_ip: device_ip.clone(),
                success,
                message,
                session_id: Some(item_clone.student_exam_id),
            }
        }));
    }

    // 收集所有结果
    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    // 发送分发完成事件
    send_distribute_progress(app_handle, &exam_id, results.len(), results.len(), "试卷分发完成".to_string());

    let success_count = results.iter().filter(|item| item.success).count();
    Ok(student_exam_schema::DistributeExamPapersOutput {
        request_id,
        total: results.len(),
        success_count,
        results,
    })
}

/// 发送分发进度事件
fn send_distribute_progress(
    app_handle: &tauri::AppHandle,
    exam_id: &str,
    completed: usize,
    total: usize,
    message: String,
) {
    let progress = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let _ = app_handle.emit("distribute-progress", serde_json::json!({
        "exam_id": exam_id,
        "completed": completed,
        "total": total,
        "progress": progress,
        "message": message,
        "timestamp": now_ms(),
    }));
}

#[derive(Clone)]
struct ExamPackageBuildResult {
    file_name: String,
    zip_bytes: Vec<u8>,
    total_bytes: u64,
    total_chunks: u32,
    sha256: String,
}

fn build_exam_package_zip(
    app_handle: &tauri::AppHandle,
    questions: &[crate::models::question::Model],
) -> Result<ExamPackageBuildResult> {
    let app_data_dir = app_handle.path().app_data_dir()?;
    let temp_dir = app_data_dir
        .join("temp")
        .join(format!("exam-distribute-package-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&temp_dir)?;

    let xlsx_path = temp_dir.join("question_bank.xlsx");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "题目ID")?;
    worksheet.write_string(0, 1, "序号")?;
    worksheet.write_string(0, 2, "题型")?;
    worksheet.write_string(0, 3, "题目内容")?;
    worksheet.write_string(0, 4, "题干图片")?;
    worksheet.write_string(0, 5, "选项")?;
    worksheet.write_string(0, 6, "答案")?;
    worksheet.write_string(0, 7, "分值")?;
    worksheet.write_string(0, 8, "解析")?;

    let mut image_source_to_archive: HashMap<String, String> = HashMap::new();
    let mut zip_entries: Vec<ZipAssetEntry> = Vec::new();
    let mut dedupe_archive_paths: HashSet<String> = HashSet::new();

    for (index, q) in questions.iter().enumerate() {
        let row = (index + 1) as u32;

        let content_images = parse_json_string_vec(q.content_image_paths.as_deref())
            .into_iter()
            .map(|path| {
                map_source_image_to_archive(
                    &app_data_dir,
                    &path,
                    "content",
                    &mut image_source_to_archive,
                    &mut dedupe_archive_paths,
                    &mut zip_entries,
                )
            })
            .collect::<Vec<String>>();

        let remapped_options = remap_options_for_package(
            q.options.as_deref(),
            &app_data_dir,
            &mut image_source_to_archive,
            &mut dedupe_archive_paths,
            &mut zip_entries,
        )?;

        worksheet.write_string(row, 0, &q.id)?;
        worksheet.write_number(row, 1, q.seq as f64)?;
        worksheet.write_string(row, 2, &q.r#type)?;
        worksheet.write_string(row, 3, &q.content)?;
        worksheet.write_string(row, 4, serde_json::to_string(&content_images)?.as_str())?;
        worksheet.write_string(row, 5, remapped_options.as_deref().unwrap_or(""))?;
        worksheet.write_string(row, 6, &q.answer)?;
        worksheet.write_number(row, 7, q.score as f64)?;
        worksheet.write_string(row, 8, q.explanation.as_deref().unwrap_or(""))?;
    }

    workbook.save(&xlsx_path)?;
    zip_entries.push(ZipAssetEntry {
        source_path: xlsx_path.clone(),
        archive_path: "question_bank.xlsx".to_string(),
    });

    let zip_path = temp_dir.join("exam_package.zip");
    create_asset_zip(&zip_path, &zip_entries)?;
    let zip_bytes = std::fs::read(&zip_path)?;
    let total_bytes = zip_bytes.len() as u64;
    let total_chunks = ((zip_bytes.len() + PACKAGE_CHUNK_SIZE - 1) / PACKAGE_CHUNK_SIZE) as u32;
    let sha256 = sha256_hex(&zip_bytes);

    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(ExamPackageBuildResult {
        file_name: "exam_package.zip".to_string(),
        zip_bytes,
        total_bytes,
        total_chunks,
        sha256,
    })
}

fn parse_json_string_vec(raw: Option<&str>) -> Vec<String> {
    let Some(value) = raw else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<String>>(value).unwrap_or_default()
}

fn remap_options_for_package(
    raw: Option<&str>,
    app_data_dir: &Path,
    source_to_archive: &mut HashMap<String, String>,
    dedupe_archive_paths: &mut HashSet<String>,
    zip_entries: &mut Vec<ZipAssetEntry>,
) -> Result<Option<String>> {
    let Some(value) = raw else {
        return Ok(None);
    };

    let Ok(mut options) = serde_json::from_str::<Vec<serde_json::Value>>(value) else {
        return Ok(Some(value.to_string()));
    };

    for option in &mut options {
        let images = option
            .get("image_paths")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let remapped = images
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .map(|path| {
                map_source_image_to_archive(
                    app_data_dir,
                    &path,
                    "options",
                    source_to_archive,
                    dedupe_archive_paths,
                    zip_entries,
                )
            })
            .map(serde_json::Value::String)
            .collect::<Vec<_>>();

        if let Some(object) = option.as_object_mut() {
            object.insert("image_paths".to_string(), serde_json::Value::Array(remapped));
        }
    }

    Ok(Some(serde_json::to_string(&options)?))
}

fn map_source_image_to_archive(
    app_data_dir: &Path,
    source_path: &str,
    scope: &str,
    source_to_archive: &mut HashMap<String, String>,
    dedupe_archive_paths: &mut HashSet<String>,
    zip_entries: &mut Vec<ZipAssetEntry>,
) -> String {
    let normalized = source_path.trim().replace('\\', "/");
    if let Some(mapped) = source_to_archive.get(&normalized) {
        return mapped.clone();
    }

    let source_abs = app_data_dir.join(normalized.trim_start_matches('/'));
    if !source_abs.exists() {
        return normalized;
    }

    let file_name = source_abs
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("asset.bin");
    let archive_path = format!(
        "assets/{}/{}_{}",
        scope,
        uuid::Uuid::new_v4().simple(),
        file_name
    );

    if dedupe_archive_paths.insert(archive_path.clone()) {
        zip_entries.push(ZipAssetEntry {
            source_path: source_abs,
            archive_path: archive_path.clone(),
        });
    }

    source_to_archive.insert(normalized, archive_path.clone());
    archive_path
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

async fn wait_for_paper_package_ack(app_handle: &tauri::AppHandle, batch_id: &str) -> Option<String> {
    let state = app_handle.state::<crate::state::AppState>();
    let mut waited_ms = 0;
    while waited_ms < 20_000 {
        if let Some(message) = state.take_paper_package_ack(batch_id) {
            return Some(message);
        }
        sleep(Duration::from_millis(200)).await;
        waited_ms += 200;
    }
    None
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

pub async fn list_student_score_summary_by_exam_id(
    db: &DatabaseConnection,
    exam_id: String,
) -> Result<Vec<student_exam_schema::StudentScoreSummaryDto>> {
    student_exam_repo::get_student_score_summary_by_exam_id(db, &exam_id).await
}

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
