use anyhow::{Context, Result};
use futures_util::{Sink, SinkExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message,
};

pub type ClientWsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn connect_ws(ws_url: &str) -> Result<ClientWsStream> {
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .with_context(|| format!("连接教师端失败: {}", ws_url))?;
    Ok(ws_stream)
}

pub fn new_text_channel() -> (UnboundedSender<String>, UnboundedReceiver<String>) {
    mpsc::unbounded_channel::<String>()
}

pub async fn run_text_writer_loop<S>(mut sink: S, mut rx: UnboundedReceiver<String>) -> Result<()>
where
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    while let Some(text) = rx.recv().await {
        sink.send(Message::Text(text.into())).await?;
    }

    Ok(())
}
