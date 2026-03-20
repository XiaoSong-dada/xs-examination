use once_cell::sync::Lazy;
use crate::utils::env::get_env_u16;

pub struct AppConfig {
    pub ws_server_port: u16,
    pub std_server_port: u16,
    pub std_controller_port: u16,
}

// 创建一个全局可访问的实例
pub static SETTINGS: Lazy<AppConfig> = Lazy::new(|| {
    AppConfig {
        ws_server_port: get_env_u16("WS_SERVER_PORT", 18888),
        std_server_port: get_env_u16("STD_SERVER_PORT", 28888),
        std_controller_port: get_env_u16("STD_CONTROLLER_PORT", 38888),
    }
});