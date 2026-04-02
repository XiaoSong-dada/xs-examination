use anyhow::{Result, bail};
use serde::Serialize;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn bind_listener(bind_addr: &str) -> Result<TcpListener> {
    Ok(TcpListener::bind(bind_addr).await?)
}

pub async fn read_json_request(stream: &mut TcpStream, max_request_size: usize) -> Result<Value> {
    let mut data = Vec::with_capacity(16 * 1024);
    let mut chunk = [0_u8; 4096];

    loop {
        let size = stream.read(&mut chunk).await?;
        if size == 0 {
            break;
        }

        data.extend_from_slice(&chunk[..size]);
        if data.len() > max_request_size {
            bail!("控制消息过大: {} bytes", data.len());
        }

        match serde_json::from_slice::<Value>(&data) {
            Ok(value) => return Ok(value),
            Err(err) if err.is_eof() => continue,
            Err(err) => return Err(err.into()),
        }
    }

    if data.is_empty() {
        bail!("空请求体");
    }

    Ok(serde_json::from_slice::<Value>(&data)?)
}

pub async fn write_json_response<T: Serialize>(stream: &mut TcpStream, response: &T) -> Result<()> {
    let output = serde_json::to_vec(response)?;
    stream.write_all(&output).await?;
    Ok(())
}

pub async fn write_json_request<T: Serialize>(stream: &mut TcpStream, request: &T) -> Result<()> {
    let output = serde_json::to_vec(request)?;
    stream.write_all(&output).await?;
    Ok(())
}

pub async fn read_json_response<T: serde::de::DeserializeOwned>(stream: &mut TcpStream) -> Result<T> {
    let mut data = Vec::with_capacity(16 * 1024);
    let mut chunk = [0_u8; 4096];

    loop {
        let size = stream.read(&mut chunk).await?;
        if size == 0 {
            break;
        }

        data.extend_from_slice(&chunk[..size]);

        match serde_json::from_slice::<T>(&data) {
            Ok(value) => return Ok(value),
            Err(err) if err.is_eof() => continue,
            Err(err) => return Err(err.into()),
        }
    }

    Ok(serde_json::from_slice::<T>(&data)?)
}
