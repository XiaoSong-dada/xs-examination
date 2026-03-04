use anyhow::Result;
use sqlx::SqlitePool;
use chrono::Utc;

use crate::repos::exam_repo;
use crate::db::models::Exam as DbExam;

pub async fn list_exams(pool: &SqlitePool) -> Result<Vec<DbExam>> {
    // 目前业务仅返回按创建时间倒序的所有考试，未来可加分页/权限校验等
    exam_repo::get_all_exams(pool).await
}

pub async fn create_exam(pool: &SqlitePool, title: String) -> Result<DbExam> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();
    exam_repo::insert_exam(pool, id, title, now).await
}
