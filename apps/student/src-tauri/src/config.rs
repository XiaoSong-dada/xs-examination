use anyhow::{Context, Result};

/// 应用核心配置，从 .env 文件及环境变量中读取。
pub struct AppConfig {
    /// SQLite 数据库文件名（不含路径，如 "student.db"）
    pub db_name: String,
}

impl AppConfig {
    /// 初始化应用配置。
    ///
    /// 优先读取进程运行目录下的 `.env` 文件，
    /// 再从环境变量（包括系统环境与已加载的 .env）中提取各字段。
    ///
    /// # 返回值
    /// 成功返回 `AppConfig` 实例；缺失必填环境变量时返回 `Err`。
    pub fn load() -> Result<Self> {
        // 加载 .env 文件（找不到文件不报错，使用已有系统环境变量）
        let _ = dotenvy::dotenv();

        let db_name = std::env::var("DB_NAME")
            .context("缺少必填环境变量: DB_NAME（SQLite 数据库文件名）")?;

        Ok(Self { db_name })
    }
}