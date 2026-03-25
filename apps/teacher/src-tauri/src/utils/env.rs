
use std::net::IpAddr;

pub fn get_env_u16(name: &str, default_value: u16) -> u16 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(default_value)
}

pub fn get_env_ip(name: &str, default_value: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|v| v.parse::<IpAddr>().is_ok())
        .unwrap_or_else(|| default_value.to_string())
}
