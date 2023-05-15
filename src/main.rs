mod events;
mod server;

extern crate serde_json;
use cln_plugin::Builder;

use events::event_functions::{
    balance_snapshot, block_added, channel_open_failed, channel_opened, channel_state_changed,
    coin_movement, connect, disconnect, forward_event, invoice_creation, invoice_payment,
    openchannel_peer_sigs, sendpay_failure, sendpay_success, shutdown, warning,
};

use server::websocket::start_websocket_server;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct PluginState {
    sender: mpsc::Sender<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (sender, receiver) = mpsc::channel(100);

    let state = PluginState { sender };

    tokio::spawn(async move { start_websocket_server(receiver).await });

    if let Some(plugin) = Builder::new(tokio::io::stdin(), tokio::io::stdout())
        .dynamic()
        .subscribe("connect", connect)
        .subscribe("disconnect", disconnect)
        // .subscribe("channel_opened", channel_opened)
        // .subscribe("channel_open_failed", channel_open_failed)
        // .subscribe("channel_state_changed", channel_state_changed)
        // .subscribe("invoice_payment", invoice_payment)
        // .subscribe("invoice_creation", invoice_creation)
        // .subscribe("warning", warning)
        // .subscribe("forward_event", forward_event)
        // .subscribe("sendpay_success", sendpay_success)
        // .subscribe("sendpay_failure", sendpay_failure)
        // .subscribe("coin_movement", coin_movement)
        // .subscribe("balance_snapshot", balance_snapshot)
        // .subscribe("block_added", block_added)
        // .subscribe("openchannel_peer_sigs", openchannel_peer_sigs)
        // .subscribe("shutdown", shutdown)
        .start(state)
        .await?
    {
        let plug_res = plugin.join().await;

        plug_res
    } else {
        Ok(())
    }
}

// use std::collections::HashSet;
// use std::sync::Arc;

// use futures::{SinkExt, StreamExt};
// use tokio::net::{TcpListener, TcpStream};
// use tokio::sync::{mpsc, Mutex};
// use tokio::task;

// use tokio_tungstenite::tungstenite::Message;
// use tokio_tungstenite::{accept_async, tungstenite};

// #[tokio::main]
// async fn main() -> Result<(), anyhow::Error> {
//     let (tx, rx) = mpsc::channel::<serde_json::Value>(100);

//     let listener = TcpListener::bind("0.0.0.0:3013").await?;
//     let addr = listener.local_addr()?;
//     println!("Listening on {}", addr);

//     let clients = Arc::new(Mutex::new(HashSet::new()));

//     loop {
//         let (stream, _) = listener.accept().await?;
//         let clients = clients.clone();
//         let tx = tx.clone();

//         task::spawn(async move {
//             if let Err(e) = handle_connection(stream, clients, tx).await {
//                 eprintln!("Error: {}", e);
//             }
//         });
//     }
// }

// async fn handle_connection(
//     raw_stream: TcpStream,
//     clients: Arc<Mutex<HashSet<std::net::SocketAddr>>>,
//     tx: mpsc::Sender<serde_json::Value>,
// ) -> Result<(), anyhow::Error> {
//     let addr = raw_stream.peer_addr()?;
//     let ws_stream = accept_async(raw_stream).await?;
//     println!("New WebSocket connection: {}", addr);

//     let (write, mut read) = ws_stream.split();

//     let write = Arc::new(Mutex::new(write)); // Wrap write in an Arc and Mutex

//     let write_clone = write.clone(); // Clone the Arc

//     clients.lock().await.insert(addr);

//     while let Some(Ok(message)) = read.next().await {
//         if message.is_text() {
//             let message = message.to_text().unwrap();
//             println!("Received a message from {}: {}", addr, message);

//             // Send the message to all connected clients
//             let clients = clients.lock().await;
//             for client in clients.iter() {
//                 println!("Sending message to {:?} {:?}", addr, client);

//                 let message = Message::Text(message.to_string());
//                 let mut write = write_clone.lock().await; // Lock the Mutex to access write
//                 write.send(message).await?;
//             }
//         }
//     }

//     clients.lock().await.remove(&addr);

//     Ok(())
// }
