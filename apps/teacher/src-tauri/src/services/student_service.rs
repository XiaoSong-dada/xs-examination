use anyhow::Result;
use chrono::Utc;
use sea_orm::DatabaseConnection;

use crate::models::student::Model as StudentModel;
use crate::repos::student_repo;

#[derive(Debug, Clone)]
pub struct StudentWritePayload {
    pub student_no: String,
    pub name: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl StudentWritePayload {
    pub fn normalized(
        student_no: String,
        name: String,
        created_at: Option<i64>,
        updated_at: Option<i64>,
    ) -> Self {
        Self {
            student_no: student_no.trim().to_string(),
            name: name.trim().to_string(),
            created_at,
            updated_at,
        }
    }
}

pub async fn list_students(db: &DatabaseConnection) -> Result<Vec<StudentModel>> {
    student_repo::get_all_students(db).await
}

pub async fn get_student_by_id(db: &DatabaseConnection, id: String) -> Result<StudentModel> {
    student_repo::get_student_by_id(db, &id).await
}

pub async fn create_student(
    db: &DatabaseConnection,
    payload: StudentWritePayload,
) -> Result<StudentModel> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();
    let created_at = payload.created_at.unwrap_or(now);
    let updated_at = payload.updated_at.unwrap_or(now);
    student_repo::insert_student(db, id, payload, created_at, updated_at).await
}

pub async fn update_student(
    db: &DatabaseConnection,
    id: String,
    payload: StudentWritePayload,
) -> Result<StudentModel> {
    let now = Utc::now().timestamp_millis();
    let updated_at = payload.updated_at.unwrap_or(now);
    student_repo::update_student_by_id(db, &id, payload, updated_at).await
}

pub async fn bulk_create_students(
    db: &DatabaseConnection,
    payloads: Vec<StudentWritePayload>,
) -> Result<Vec<StudentModel>> {
    let now = Utc::now().timestamp_millis();
    let rows = payloads
        .into_iter()
        .map(|payload| student_repo::StudentBatchInsertItem {
            id: uuid::Uuid::new_v4().to_string(),
            student_no: payload.student_no,
            name: payload.name,
            created_at: payload.created_at.unwrap_or(now),
            updated_at: payload.updated_at.unwrap_or(now),
        })
        .collect();

    student_repo::insert_students_batch(db, rows).await
}

pub async fn delete_student(db: &DatabaseConnection, id: String) -> Result<()> {
    student_repo::delete_student_by_id(db, &id).await
}
