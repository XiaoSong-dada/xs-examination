use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tokio::time::{timeout, Duration};

const PROBE_PORTS: [u16; 3] = [135, 445, 3389];
const CONNECT_TIMEOUT_MS: u64 = 160;

fn resolve_local_ipv4() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    match addr.ip() {
        IpAddr::V4(ipv4) => Some(ipv4),
        IpAddr::V6(_) => None,
    }
}

fn build_subnet_candidates(local_ip: Ipv4Addr) -> Vec<Ipv4Addr> {
    let octets = local_ip.octets();
    let mut ips = Vec::with_capacity(254);
    for host in 1u8..=254u8 {
        if host == octets[3] {
            continue;
        }
        ips.push(Ipv4Addr::new(octets[0], octets[1], octets[2], host));
    }
    ips
}

async fn probe_single_ip(ip: Ipv4Addr) -> bool {
    for port in PROBE_PORTS {
        let target = SocketAddr::new(IpAddr::V4(ip), port);
        let connected = timeout(
            Duration::from_millis(CONNECT_TIMEOUT_MS),
            TcpStream::connect(target),
        )
        .await;

        if matches!(connected, Ok(Ok(_))) {
            return true;
        }
    }

    false
}

pub async fn discover_active_ips() -> anyhow::Result<Vec<String>> {
    let local_ip = resolve_local_ipv4().ok_or_else(|| anyhow::anyhow!("无法获取本机 IPv4 地址"))?;
    let candidates = build_subnet_candidates(local_ip);

    let mut set = JoinSet::new();
    for ip in candidates {
        set.spawn(async move {
            if probe_single_ip(ip).await {
                Some(ip.to_string())
            } else {
                None
            }
        });
    }

    let mut seen = HashSet::new();
    let mut discovered = Vec::new();

    while let Some(result) = set.join_next().await {
        if let Ok(Some(ip)) = result {
            if seen.insert(ip.clone()) {
                discovered.push(ip);
            }
        }
    }

    discovered.sort();
    Ok(discovered)
}
