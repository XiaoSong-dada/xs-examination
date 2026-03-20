use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

pub struct RequestReplyTimeouts {
    pub connect: Duration,
    pub write: Duration,
    pub shutdown_write: Option<Duration>,
    pub read: Duration,
}

pub async fn send_json_request<TReq, TAck>(
    addr: &str,
    request: &TReq,
    timeouts: RequestReplyTimeouts,
    context_tag: &str,
) -> Result<TAck>
where
    TReq: Serialize,
    TAck: DeserializeOwned,
{
    let mut stream = timeout(timeouts.connect, TcpStream::connect(addr))
        .await
        .with_context(|| format!("{}: 连接超时: {}", context_tag, addr))??;

    let payload = serde_json::to_vec(request)?;
    timeout(timeouts.write, stream.write_all(&payload))
        .await
        .with_context(|| format!("{}: 发送超时: {}", context_tag, addr))??;

    if let Some(shutdown_timeout) = timeouts.shutdown_write {
        timeout(shutdown_timeout, stream.shutdown())
            .await
            .with_context(|| format!("{}: 关闭写入通道超时: {}", context_tag, addr))??;
    }

    let mut buf = Vec::with_capacity(4096);
    timeout(timeouts.read, stream.read_to_end(&mut buf))
        .await
        .with_context(|| format!("{}: 读取回执超时: {}", context_tag, addr))??;

    let ack: TAck =
        serde_json::from_slice(&buf).with_context(|| format!("{}: 回执解析失败: {}", context_tag, addr))?;

    Ok(ack)
}
