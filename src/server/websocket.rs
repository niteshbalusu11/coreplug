use std::sync::{Arc, Mutex};

use futures::{stream::StreamExt, SinkExt};
use log::{error, info};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_tungstenite::accept_async;

pub async fn start_websocket_server(
    receiver: mpsc::Receiver<serde_json::Value>,
) -> Result<(), anyhow::Error> {
    let listener = TcpListener::bind("0.0.0.0:3012").await?;

    let rx = Arc::new(tokio::sync::Mutex::new(receiver));

    while let Ok((stream, _)) = listener.accept().await {
        let cloned_rx = Arc::clone(&rx);

        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, cloned_rx).await {
                log::error!("Error handling connection: {:?}", err);
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    raw_stream: TcpStream,
    rx: Arc<tokio::sync::Mutex<mpsc::Receiver<serde_json::Value>>>,
) -> Result<(), anyhow::Error> {
    let addr = raw_stream.peer_addr()?;
    let ws_stream = accept_async(raw_stream).await?;
    info!("New WebSocket connection: {}", addr);

    tokio::spawn(async move {
        let mut rx = rx.lock().await;

        while let Some(message) = rx.recv().await {
            info!("SENDING = {}", message);
        }
    });

    let (mut write, mut read) = ws_stream.split();

    while let Some(Ok(message)) = read.next().await {
        if message.is_text() || message.is_binary() {
            write.send(message).await?;
        }
    }

    // Peer disconnected
    if let Err(err) = write.close().await {
        error!("Error closing WebSocket connection for {}: {:?}", addr, err);
    } else {
        info!("WebSocket connection closed: {}", addr);
    }

    Ok(())
}
