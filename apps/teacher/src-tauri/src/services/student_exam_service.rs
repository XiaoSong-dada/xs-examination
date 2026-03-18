use anyhow::Result;
use sea_orm::DatabaseConnection;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::student::Model as StudentModel;
use crate::schemas::student_exam_schema;
use crate::repos::student_exam_repo;

const HEARTBEAT_TIMEOUT_MS: i64 = 15_000;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

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
    let now = now_ms();

    Ok(assignments
        .into_iter()
        .map(|item| {
            let last_heartbeat_at = connection_map.get(&item.student_id).copied();
            let (connection_status, has_heartbeat_seen) =
                derive_connection_status(item.ip_addr.as_deref(), last_heartbeat_at, now);

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
            }
        })
        .collect())
}
