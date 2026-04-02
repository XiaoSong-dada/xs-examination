use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use crate::utils::env::get_env_u16;

const DEFAULT_DB_NAME: &str = "teacher.db";

pub struct AppConfig {
    pub ws_server_port: u16,
    pub std_server_port: u16,
    pub std_controller_port: u16,
}

/// 数据库相关配置。
pub struct DbConfig {
    /// SQLite 数据库文件名（不含路径，如 "teacher.db"）
    pub db_name: String,
}

impl DbConfig {
    /// 初始化数据库配置。
    ///
    /// 优先读取进程运行目录下的 `.env` 文件，
    /// 再从环境变量（包括系统环境与已加载的 .env）中提取字段。
    ///
    /// # 参数
    /// 无。
    ///
    /// # 返回值
    /// 成功返回 `DbConfig`；当 `DB_NAME` 包含路径分隔符等非法值时返回 `Err`。
    pub fn load() -> Result<Self> {
        // 加载 .env 文件（找不到文件不报错，使用已有系统环境变量）
        let _ = dotenvy::dotenv();

        let db_name = std::env::var("DB_NAME")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_DB_NAME.to_string());

        if db_name.contains(['/', '\\']) {
            return Err(anyhow::anyhow!(
                "环境变量 DB_NAME 仅支持文件名，不应包含路径分隔符"
            ))
            .context("DB_NAME 配置非法");
        }

        Ok(Self { db_name })
    }
}

// 创建一个全局可访问的实例
pub static SETTINGS: Lazy<AppConfig> = Lazy::new(|| {
    AppConfig {
        ws_server_port: get_env_u16("WS_SERVER_PORT", 18888),
        std_server_port: get_env_u16("STD_SERVER_PORT", 28888),
        std_controller_port: get_env_u16("STD_CONTROLLER_PORT", 38888),
    }
});