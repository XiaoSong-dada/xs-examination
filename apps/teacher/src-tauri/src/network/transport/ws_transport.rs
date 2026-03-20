use anyhow::Result;
use futures_util::{Sink, SinkExt};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;

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
