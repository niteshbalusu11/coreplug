use futures::stream::SplitSink;

use std::sync::Arc;

use tokio_tungstenite::tungstenite::Message;

use futures::{stream::StreamExt, SinkExt};
use log::{error, info};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{accept_async, tungstenite, WebSocketStream};

lazy_static::lazy_static! {
    static ref WEBSOCKET_CLIENT_CONNECTED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

pub struct ConfiguredConnection {
    address: std::net::SocketAddr,
    writer: SplitSink<WebSocketStream<TcpStream>, Message>,
}

pub async fn start_websocket_server(
    configured_connection_sender: mpsc::UnboundedSender<ConfiguredConnection>,
    configured_connection_receiver: mpsc::UnboundedReceiver<ConfiguredConnection>,
    event_receiver: mpsc::UnboundedReceiver<serde_json::Value>,
) {
    match start_websocket_server_inner(
        configured_connection_sender,
        Arc::new(Mutex::new(configured_connection_receiver)), // Wrap in Arc<Mutex<_>>
        Arc::new(Mutex::new(event_receiver)),
    )
    .await
    {
        Ok(()) => (),
        Err(error) => {
            error!("{}", error);
        }
    }
}

async fn start_websocket_server_inner(
    configured_connection_sender: mpsc::UnboundedSender<ConfiguredConnection>,
    configured_connection_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ConfiguredConnection>>>, // Change the type to Arc<Mutex<_>>
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,
) -> Result<(), anyhow::Error> {
    let listener = TcpListener::bind("0.0.0.0:3012").await?;

    while let Ok((stream, _)) = listener.accept().await {
        // Check if there is an active WebSocket client connected
        let websocket_client_connected = *WEBSOCKET_CLIENT_CONNECTED.lock().await;

        if websocket_client_connected {
            // Reject the new connection
            log::warn!(
                "Rejecting new connection. An existing WebSocket client is already connected."
            );

            let message = tungstenite::Message::Text(
                serde_json::json!({
                    "type": "error",
                    "message": "Only one client is allowed to connect at a time."
                })
                .to_string(),
            );

            let mut stream = accept_async(stream).await?;
            stream.send(message).await?;

            continue;
        }

        let configured_connection_sender = configured_connection_sender.clone();

        tokio::spawn(async move {
            // Set the flag to indicate an active WebSocket client is now connected
            *WEBSOCKET_CLIENT_CONNECTED.lock().await = true;

            if let Err(err) = handle_connection(stream, configured_connection_sender).await {
                log::error!("Error handling connection: {:?}", err);
            }
        });

        tokio::spawn(broadcast_task(
            configured_connection_receiver.clone(),
            event_receiver.clone(),
        ));
    }

    Ok(())
}

async fn handle_connection(
    raw_stream: TcpStream,
    configured_connection_sender: mpsc::UnboundedSender<ConfiguredConnection>,
) -> Result<(), anyhow::Error> {
    let address = raw_stream.peer_addr()?;
    let ws_stream = accept_async(raw_stream).await?;

    let (writer, mut reader) = ws_stream.split();

    tokio::spawn(async move {
        Ok(while let Some(Ok(message)) = reader.next().await {
            if message.is_text() {
                let message = message.to_string();
                info!("Received message from {}: {}", address, message);
            }

            *WEBSOCKET_CLIENT_CONNECTED.lock().await = false;
            anyhow::bail!("Received unexpected message from {}", address);
        })
    });

    let configured_connection = ConfiguredConnection { address, writer };

    // config logic
    // put whichever flags to know which events to send in the COnfiguredConnection struct..
    // parse from message

    match configured_connection_sender.send(configured_connection) {
        Ok(()) => (),
        Err(_error) => {
            *WEBSOCKET_CLIENT_CONNECTED.lock().await = false;
            anyhow::bail!("Failed to send configured connection to boadcast task, broadcast task must have died.");
        }
    }

    Ok(())
}

pub async fn broadcast_task(
    configured_connection_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ConfiguredConnection>>>, // Change the type to Arc<Mutex<_>>
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,
) {
    match broadcast_task_inner(configured_connection_receiver, event_receiver).await {
        Ok(()) => (),
        Err(error) => {
            error!("{}", error);
        }
    }
}

pub async fn broadcast_task_inner(
    configured_connection_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ConfiguredConnection>>>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,
) -> anyhow::Result<()> {
    let mut conf_connection: Option<ConfiguredConnection> = None;
    let mut configured_connection_receiver = configured_connection_receiver.lock().await;
    let mut event_receiver = event_receiver.lock().await;

    loop {
        tokio::select! {
            configured_connection = configured_connection_receiver.recv() => {
                let configured_connection = match configured_connection {
                    Some(configured_connection) => {
                        info!("New websocket connection from {}", configured_connection.address);
                        configured_connection
                    },
                    None => return Ok(()),
                };
                conf_connection = Some(configured_connection);
            }


            event = event_receiver.recv() => {
                let event = match event {
                    Some(event) => event.to_string(),
                    None => return Ok(()),
                };

                if let Some(conf_connection) = conf_connection.as_mut() {
                    match conf_connection.writer.send(Message::Text(event.clone())).await {
                        Ok(_) => (),
                        Err(error) => {
                            *WEBSOCKET_CLIENT_CONNECTED.lock().await = false;
                            error!("Failed to send to web socket (might just be closed): {}", error);
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}
