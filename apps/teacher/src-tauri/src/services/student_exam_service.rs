use anyhow::Result;
use sea_orm::DatabaseConnection;
use std::collections::HashSet;

use crate::models::student::Model as StudentModel;
use crate::repos::student_exam_repo;

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
