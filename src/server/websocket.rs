use std::{collections::HashSet, sync::Arc};

use futures::{stream::StreamExt, SinkExt};
use log::{error, info};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{accept_async, tungstenite};

pub async fn start_websocket_server(
    receiver: mpsc::Receiver<serde_json::Value>,
) -> Result<(), anyhow::Error> {
    let listener = TcpListener::bind("0.0.0.0:3012").await?;
    let clients = Arc::new(Mutex::new(HashSet::new()));

    let rx = Arc::new(tokio::sync::Mutex::new(receiver));

    while let Ok((stream, _)) = listener.accept().await {
        let cloned_rx = Arc::clone(&rx);
        let clients = clients.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, cloned_rx, clients).await {
                log::error!("Error handling connection: {:?}", err);
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    raw_stream: TcpStream,
    rx: Arc<tokio::sync::Mutex<mpsc::Receiver<serde_json::Value>>>,
    clients: Arc<Mutex<HashSet<std::net::SocketAddr>>>,
) -> Result<(), anyhow::Error> {
    let addr = raw_stream.peer_addr()?;
    let ws_stream = accept_async(raw_stream).await?;
    clients.lock().await.insert(addr);

    let (write, mut read) = ws_stream.split();

    let write = Arc::new(tokio::sync::Mutex::new(write)); // Wrap write in an Arc and Mutex

    let write_clone = write.clone(); // Clone the Arc

    while let Some(Ok(message)) = read.next().await {
        if message.is_text() && message.to_string() == "something else" {
            let mut rx = rx.lock().await;
            while let Some(message) = rx.recv().await {
                // Send the message to the WebSocket client
                let message = tungstenite::Message::Text(message.to_string());

                let mut write = write_clone.lock().await; // Lock the Mutex to access write
                let send_result = write.send(message).await;
                if let Err(e) = send_result {
                    error!("Failed to send message to WebSocket client: {}", e);
                    rx.close();
                    break;
                }
            }
        }
    }

    clients.lock().await.remove(&addr);

    Ok(())
}

// Function to get the list of connected clients
fn get_connected_clients(clients: &HashSet<std::net::SocketAddr>) -> Vec<String> {
    clients
        .iter()
        .map(|addr| addr.to_string())
        .collect::<Vec<String>>()
}
