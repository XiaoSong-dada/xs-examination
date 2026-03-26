use std::fs;

use anyhow::Result;
use sea_orm::{
    ConnectOptions, Database, DatabaseConnection,
};
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use tauri::{AppHandle, Manager};

use crate::core::setting::DbConfig;

pub mod models;

/// 初始化 SQLite 数据库连接池。
///
/// 数据库文件保存在系统 AppData 目录下，文件名来自 `AppConfig::db_name`。
/// 数据库不存在时自动创建，并开启 WAL 模式以提升并发写入性能。
///
/// # 参数
/// * `app_handle` - Tauri 应用句柄，用于获取系统数据目录路径。
///
/// # 返回值
/// 返回已连接的 `DatabaseConnection`；路径解析失败或连接失败时返回 `Err`。
pub async fn init(app_handle: &AppHandle) -> Result<DatabaseConnection> {
    let config = DbConfig::load()?;

    let app_data_dir = app_handle.path().app_data_dir()?;
    fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join(&config.db_name);
    println!("[db] teacher sqlite path: {}", db_path.display());

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // 开启 WAL 模式提升并发写入性能
    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&pool)
        .await?;

    // 开启外键约束（SQLite 默认关闭）
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    // 运行根目录 migrations/ 目录下的所有迁移脚本（按版本号升序，仅执行尚未执行的）。
    //
    // 将来若需添加表或字段，**不要在此处修改代码**，只需在
    // `src-tauri/migrations/` 下创建新 SQL 文件，如
    // `0002_add_field_to_exams.sql`，里面写 ALTER TABLE 或 CREATE TABLE
    // 操作。sqlx 会在运行时自动执行这些迁移，并维护 `_sqlx_migrations`
    // 表记录已应用版本。这样可以保持 schema 历史可追溯。
    //
    // `sqlx::migrate!()` 无参数时会查找相对于 Cargo.toml 的 migrations 文件夹，
    // 避免使用相对路径造成 "paths relative to the current file" 错误。
    sqlx::migrate!()
        .run(&pool)
        .await?;

    let migration_versions = sqlx::query("SELECT version FROM _sqlx_migrations ORDER BY version")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| row.get::<i64, _>("version"))
        .collect::<Vec<_>>();
    println!("[db] teacher applied migrations: {:?}", migration_versions);

    let answer_sheet_fk_parents = sqlx::query("PRAGMA foreign_key_list(answer_sheets)")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| row.get::<String, _>("table"))
        .collect::<Vec<_>>();
    println!(
        "[db] answer_sheets foreign key parents: {:?}",
        answer_sheet_fk_parents
    );

    drop(pool);

    // SeaORM 使用 sqlite URL 建立连接，路径来自统一配置目录。
    let db_url = format!("sqlite:{}", db_path.to_string_lossy());
    let mut connect_options = ConnectOptions::new(db_url);
    connect_options.max_connections(10);

    let connection = Database::connect(connect_options).await?;

    Ok(connection)
}
