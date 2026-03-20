use anyhow::Result;
use std::net::{IpAddr, UdpSocket};

pub fn resolve_device_ip() -> Result<Option<String>> {
    // Use UDP route probing to obtain the outbound interface address.
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;

    let ip = socket.local_addr()?.ip();
    Ok(pick_ipv4(ip))
}

fn pick_ipv4(ip: IpAddr) -> Option<String> {
    match ip {
        IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_unspecified() => Some(v4.to_string()),
        _ => None,
    }
}
