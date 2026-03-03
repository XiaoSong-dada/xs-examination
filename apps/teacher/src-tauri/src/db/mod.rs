use std::fs;

use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tauri::{AppHandle, Manager};

pub async fn init(app_handle: &AppHandle) -> Result<SqlitePool> {
    let app_data_dir = app_handle.path().app_data_dir()?;
    fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("teacher.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS exams (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    let _ = db_url;

    Ok(pool)
}
