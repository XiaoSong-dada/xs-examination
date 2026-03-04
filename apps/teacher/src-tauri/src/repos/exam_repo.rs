use anyhow::Result;
use sqlx::SqlitePool;

use crate::db::models::Exam as DbExam;

pub async fn get_all_exams(pool: &SqlitePool) -> Result<Vec<DbExam>> {
    let recs = sqlx::query_as::<_, DbExam>(
        "SELECT id, title, description, start_time, end_time, pass_score, status, shuffle_questions, shuffle_options, created_at, updated_at FROM exams ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(recs)
}

pub async fn insert_exam(pool: &SqlitePool, id: String, title: String, now: i64) -> Result<DbExam> {
    let status = "draft".to_string();
    sqlx::query(
        "INSERT INTO exams (id, title, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&status)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // 返回插入的行
    let exam = DbExam {
        id,
        title,
        description: None,
        start_time: None,
        end_time: None,
        pass_score: 0,
        status,
        shuffle_questions: 0,
        shuffle_options: 0,
        created_at: now,
        updated_at: now,
    };

    Ok(exam)
}
