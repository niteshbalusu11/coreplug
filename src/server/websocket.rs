use futures::stream::SplitSink;

use std::sync::Arc;

use tokio_tungstenite::tungstenite::Message;

use crate::HookCallbackMessage;
use anyhow::Context;
use futures::stream::SplitStream;
use futures::{stream::StreamExt, SinkExt};
use log::{error, info};
use std::collections::HashMap;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{accept_async, tungstenite, WebSocketStream};

use crate::server::validate::{validate_event, validate_message};

use super::constants::WEBSOCKET_CLIENT_CONNECTED;

pub struct ConfiguredConnection {
    address: std::net::SocketAddr,
    writer: SplitSink<WebSocketStream<TcpStream>, Message>,
}

// pub async fn start_websocket_server(
//     configured_connection_sender: mpsc::UnboundedSender<ConfiguredConnection>,
//     configured_connection_receiver: mpsc::UnboundedReceiver<ConfiguredConnection>,
//     event_receiver: mpsc::UnboundedReceiver<serde_json::Value>,
// ) {
//     match start_websocket_server_inner(
//         configured_connection_sender,
//         Arc::new(Mutex::new(configured_connection_receiver)), // Wrap in Arc<Mutex<_>>
//         Arc::new(Mutex::new(event_receiver)),
//     )
//     .await
//     {
//         Ok(()) => (),
//         Err(error) => {
//             error!("{}", error);
//         }
//     }
// }

// async fn start_websocket_server_inner(
//     configured_connection_sender: mpsc::UnboundedSender<ConfiguredConnection>,
//     configured_connection_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ConfiguredConnection>>>, // Change the type to Arc<Mutex<_>>
//     event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,
// ) -> Result<(), anyhow::Error> {
//     let listener = TcpListener::bind("0.0.0.0:3012").await?;

//     while let Ok((stream, _)) = listener.accept().await {
//         // Check if there is an active WebSocket client connected
//         let websocket_client_connected = *WEBSOCKET_CLIENT_CONNECTED.lock().await;

//         if websocket_client_connected {
//             // Reject the new connection
//             log::warn!(
//                 "Rejecting new connection. An existing WebSocket client is already connected."
//             );

//             let message = tungstenite::Message::Text(
//                 serde_json::json!({
//                     "type": "error",
//                     "message": "Only one client is allowed to connect at a time."
//                 })
//                 .to_string(),
//             );

//             let mut stream = accept_async(stream).await?;
//             stream.send(message).await?;

//             continue;
//         }

//         let configured_connection_sender = configured_connection_sender.clone();

//         tokio::spawn(async move {
//             // Set the flag to indicate an active WebSocket client is now connected
//             *WEBSOCKET_CLIENT_CONNECTED.lock().await = true;

//             if let Err(err) = handle_connection(stream, configured_connection_sender).await {
//                 log::error!("Error handling connection: {:?}", err);
//             }
//         });

//         // tokio::spawn(broadcast_task(
//         //     configured_connection_receiver.clone(),
//         //     event_receiver.clone(),
//         // ));
//     }

//     Ok(())
// }

// async fn handle_connection(
//     raw_stream: TcpStream,
//     configured_connection_sender: mpsc::UnboundedSender<ConfiguredConnection>,
// ) -> Result<(), anyhow::Error> {
//     let address = raw_stream.peer_addr()?;

//     let ws_stream = accept_async(raw_stream).await?;

//     let (writer, mut reader) = ws_stream.split();

//     tokio::spawn(async move {
//         Ok(while let Some(Ok(message)) = reader.next().await {
//             if message.is_text() {
//                 let message = message.to_string();

//                 if let Err(err) = validate_message(message).await {
//                     log::error!("Received invalid message from {:?}", err);
//                 }
//             }

//             *WEBSOCKET_CLIENT_CONNECTED.lock().await = false;
//             anyhow::bail!("Received unexpected message from {}", address);
//         })
//     });

//     let configured_connection = ConfiguredConnection { address, writer };

//     // config logic
//     // put whichever flags to know which events to send in the COnfiguredConnection struct..
//     // parse from message

//     match configured_connection_sender.send(configured_connection) {
//         Ok(()) => (),
//         Err(_error) => {
//             *WEBSOCKET_CLIENT_CONNECTED.lock().await = false;
//             anyhow::bail!("Failed to send configured connection to boadcast task, broadcast task must have died.");
//         }
//     }

//     Ok(())
// }

pub async fn websocket_connection_reader_task(
    websocket_message_sender_watch_sender: watch::Sender<Option<mpsc::UnboundedSender<Message>>>,
    hook_callback_receiver: mpsc::UnboundedReceiver<HookCallbackMessage>,
) {
    match websocket_connection_reader_task_inner(
        websocket_message_sender_watch_sender,
        hook_callback_receiver,
    )
    .await
    {
        Ok(()) => (),
        Err(error) => {
            error!("{}", error);
        }
    }
}

pub async fn websocket_connection_reader_task_inner(
    mut websocket_message_sender_watch_sender: watch::Sender<
        Option<mpsc::UnboundedSender<Message>>,
    >,
    mut hook_callback_receiver: mpsc::UnboundedReceiver<HookCallbackMessage>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:3012").await?;
    let mut hook_response_channel = HashMap::<usize, oneshot::Sender<serde_json::Value>>::new();
    let mut connection_reader = None;

    loop {
        // If we have a new value for connection_reader, set it.
        // if let Some(next_connection_reader) = next_connection_reader.take() {
        //     connection_reader = next_connection_reader;
        // }

        let hook_callback_message = if let Some(connection_reader) = connection_reader.as_ref() {
            tokio::select! {
                // Handle new connection
                new_connection = listener.accept() => {
                    // let address = raw_stream.peer_addr()?;
                    let (raw_stream, _) = new_connection?;
                    let ws_stream = accept_async(raw_stream).await?;
                    if connection_reader.is_some() {
                        // We already have a connection, reject the new one
                        ws_stream.close(None);
                    }
                    let (mut writer, mut reader) = ws_stream.split();
                    connection_reader = Some(reader);

                    let (connection_writer_message_sender, connection_writer_message_receiver) = mpsc::unbounded_channel();
                    tokio::spawn(async move {
                        while let Some(connection_writer_message) = connection_writer_message_receiver.recv().await {
                            match writer.send(connection_writer_message).await {
                                Ok(_) => (),
                                Err(error) => error!("Error writing message to connection: {}", error),
                            }
                        }
                    });

                    websocket_message_sender_watch_sender.send(Some(connection_writer_message_sender));
                }

                // Handle changes to the set of hooks which exist
                hook_callback_message = hook_callback_receiver.recv() => {
                    let hook_callback_message = hook_callback_message.context("Stopped getting hook messages, plugin might have died.")?;
                    match hook_callback_message {
                        HookCallbackMessage::AddCallback { id, response_channel } => {
                            hook_response_channel.insert(id, response_channel);
                        },
                        HookCallbackMessage::RemoveCallback { id } => {
                            hook_response_channel.remove(&id);
                        },
                    }
                }

                // Handle connection responses
                message = connection_reader.next() => {
                    let Some(message) = message else {
                        websocket_message_sender_watch_sender.send(None);
                        continue;
                    };

                    // handle messages
                    // for example, if the message is a hook response, look up the hook in the hashmap
                    // if hookmessage ...
                    let hook_response_channel = hook_response_channel.remove(0 /* id from response message here instead of 0 */);
                    let Some(hook_response_channel) = hook_response_channel else {
                        // maybe tell the client they did a bad thing, id not found for response, fail, etc.
                        continue;
                    };

                    hook_response_channel.send() // <-- in send send the json message to respond to the hook with, from the client response
                }
            }
        } else {
            // tokio::select! {
            //     connection_changed = connection_reader_watch_receiver.changed() => {
            //         connection_changed?;
            //         continue;
            //     }
            // }
        };

        // configured_connection_receiver.recv()

        // tokio::select! {

        // }
    }
    // let mut conf_connection: Option<ConfiguredConnection> = None;
    // let mut configured_connection_receiver = configured_connection_receiver.lock().await;
    // let mut event_receiver = event_receiver.lock().await;

    // tokio::select! {
    //     configured_connection = configured_connection_receiver.recv() => {
    //         let configured_connection = match configured_connection {
    //             Some(configured_connection) => {
    //                 info!("New websocket connection from {}", configured_connection.address);
    //                 configured_connection
    //             },
    //             None => return Ok(()),
    //         };
    //         conf_connection = Some(configured_connection);
    //     }

    //     event = event_receiver.recv() => {
    //         let event = match event {
    //             Some(event) => event,
    //             None => return Ok(()),
    //         };

    //         if !validate_event(event.clone()).await {
    //             return Ok(());
    //         }

    //         if let Some(conf_connection) = conf_connection.as_mut() {
    //             match conf_connection.writer.send(Message::Text(event.to_string().clone())).await {
    //                 Ok(_) => (),
    //                 Err(error) => {
    //                     *WEBSOCKET_CLIENT_CONNECTED.lock().await = false;
    //                     error!("Failed to send to web socket (might just be closed): {}", error);
    //                     return Ok(());
    //                 }
    //             }
    //         }
    //     }
    // }
    //     }
}

// pub async fn broadcast_task(
//     configured_connection_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ConfiguredConnection>>>, // Change the type to Arc<Mutex<_>>
//     event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,
// ) {
//     match broadcast_task_inner(configured_connection_receiver, event_receiver).await {
//         Ok(()) => (),
//         Err(error) => {
//             error!("{}", error);
//         }
//     }
// }

// pub async fn broadcast_task_inner(
//     configured_connection_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ConfiguredConnection>>>,
//     event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,
// ) -> anyhow::Result<()> {
//     let mut conf_connection: Option<ConfiguredConnection> = None;
//     let mut configured_connection_receiver = configured_connection_receiver.lock().await;
//     let mut event_receiver = event_receiver.lock().await;

//     loop {
//         tokio::select! {
//             configured_connection = configured_connection_receiver.recv() => {
//                 let configured_connection = match configured_connection {
//                     Some(configured_connection) => {
//                         info!("New websocket connection from {}", configured_connection.address);
//                         configured_connection
//                     },
//                     None => return Ok(()),
//                 };
//                 conf_connection = Some(configured_connection);
//             }

//             event = event_receiver.recv() => {
//                 let event = match event {
//                     Some(event) => event,
//                     None => return Ok(()),
//                 };

//                 if !validate_event(event.clone()).await {
//                     return Ok(());
//                 }

//                 if let Some(conf_connection) = conf_connection.as_mut() {
//                     match conf_connection.writer.send(Message::Text(event.to_string().clone())).await {
//                         Ok(_) => (),
//                         Err(error) => {
//                             *WEBSOCKET_CLIENT_CONNECTED.lock().await = false;
//                             error!("Failed to send to web socket (might just be closed): {}", error);
//                             return Ok(());
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
