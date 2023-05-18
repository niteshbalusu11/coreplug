use futures::stream::SplitStream;

use crate::plugin_state::HookCallbackMessage;
use crate::server::constants::EVENTS_LIST;
use crate::AbortTaskOnDrop;
use anyhow::{bail, Context};
use futures::{future, stream::StreamExt, SinkExt};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, tungstenite, WebSocketStream};

pub struct ServerState {
    websocket_message_sender_watch_sender: watch::Sender<Option<mpsc::UnboundedSender<Message>>>,
    hook_callback_receiver: mpsc::UnboundedReceiver<HookCallbackMessage>,
    listener: TcpListener,
    hook_response_channels: HashMap<usize, oneshot::Sender<serde_json::Value>>,
    connection_reader: Option<SplitStream<WebSocketStream<TcpStream>>>,
    listened_events_watch_sender: watch::Sender<Vec<&'static str>>,
    client_write_task: Option<AbortTaskOnDrop>,
}

impl ServerState {
    pub async fn new(
        websocket_message_sender_watch_sender: watch::Sender<
            Option<mpsc::UnboundedSender<Message>>,
        >,
        hook_callback_receiver: mpsc::UnboundedReceiver<HookCallbackMessage>,
        listened_events_watch_sender: watch::Sender<Vec<&'static str>>,
    ) -> anyhow::Result<Self> {
        let listener = TcpListener::bind("0.0.0.0:3012").await?;
        let hook_response_channels = HashMap::<usize, oneshot::Sender<serde_json::Value>>::new();
        let connection_reader: Option<SplitStream<WebSocketStream<TcpStream>>> = None;

        Ok(Self {
            websocket_message_sender_watch_sender,
            hook_callback_receiver,
            listener,
            hook_response_channels,
            connection_reader,
            listened_events_watch_sender,
            client_write_task: None,
        })
    }

    pub async fn handle_client_message(
        &mut self,
        message: Option<Result<Message, tungstenite::Error>>,
    ) -> anyhow::Result<()> {
        let Some(message) = message else {
            self.connection_reader = None;

            // No more messages from client, must have disconnected
            if let Err(_error) = self.websocket_message_sender_watch_sender.send(None) {
                bail!("Failed to send new websocket message sender to plugin, plugin must have died");
            }

            // Abort the write task
            self.client_write_task = None;

            return Ok(());
        };

        let message = match message {
            Ok(message) => message,
            Err(error) => {
                // Error receiving message from client
                error!(
                    "Error receiving message from remote connection - terminated it: {}",
                    error
                );

                if let Err(_error) = self.websocket_message_sender_watch_sender.send(None) {
                    bail!("Failed to send new websocket message sender to plugin, plugin must have died");
                }

                return Ok(());
            }
        };

        #[derive(Serialize, Deserialize)]
        enum ClientMessage {
            SetEventSubscriptions {
                events: Vec<String>,
            },
            HookResponse {
                id: usize,
                response: serde_json::Value,
            },
        }

        let Message::Text(message) = message else {
            error!("Received non-text message from websocket");
            return Ok(());
        };

        match serde_json::from_str::<ClientMessage>(&message)? {
            ClientMessage::SetEventSubscriptions { events } => {
                let events = events
                    .into_iter()
                    .map(|event| {
                        match EVENTS_LIST
                            .iter()
                            .find(|events_list_event| event.as_str() == **events_list_event)
                        {
                            Some(event_list_event) => Ok(*event_list_event),
                            None => bail!("Invalid event: {}", event.as_str()),
                        }
                    })
                    .collect::<anyhow::Result<Vec<&'static str>>>()?;

                info!("Setting event subscriptions: {:?}", events);

                if let Err(_error) = self.listened_events_watch_sender.send(events) {
                    bail!("Failed to send new set of events to listen to - plugin must have died");
                }
            }
            ClientMessage::HookResponse { id, response } => {
                let Some(hook_response_channel) = self.hook_response_channels.remove(&id) else {
                    // should probably tell the client they did a bad thing here
                    error!("Received hook response for invalid id: {}", id);
                    return Ok(());
                };

                match hook_response_channel.send(response) {
                    Ok(()) => (),
                    Err(_error) => {
                        error!("Received hook response for valid id '{}', but the hook callback had already moved on", id);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn handle_new_client_connection(
        &mut self,
        new_connection: tokio::io::Result<(TcpStream, SocketAddr)>,
    ) -> anyhow::Result<()> {
        // let address = raw_stream.peer_addr()?;
        let (tcp_stream, _) = new_connection?;
        let mut ws_stream = accept_async(tcp_stream).await?;
        if self.connection_reader.is_some() {
            // We already have a connection, reject the new one
            if let Err(error) = ws_stream.close(None).await {
                error!("Error rejecting (closing) new client connection while a client is already connected: {}", error);
            }
        }
        let (mut writer, reader) = StreamExt::split(ws_stream);
        self.connection_reader = Some(reader);

        let (connection_writer_message_sender, mut connection_writer_message_receiver) =
            mpsc::unbounded_channel();

        self.client_write_task = Some(
            tokio::spawn(async move {
                while let Some(connection_writer_message) =
                    connection_writer_message_receiver.recv().await
                {
                    match writer.send(connection_writer_message).await {
                        Ok(_) => (),
                        Err(error) => {
                            error!("Error writing message to connection: {}", error);
                            return;
                        }
                    }
                }

                info!("Client disconnected");
            })
            .into(),
        );

        if let Err(_error) = self
            .websocket_message_sender_watch_sender
            .send(Some(connection_writer_message_sender))
        {
            bail!("Failed to send new websocket message sender to plugin state - plugin must have died");
        }

        Ok(())
    }

    pub async fn handle_hook_callback_message(
        &mut self,
        hook_callback_message: Option<HookCallbackMessage>,
    ) -> anyhow::Result<()> {
        let hook_callback_message = hook_callback_message
            .context("Stopped getting hook messages, plugin might have died.")?;
        match hook_callback_message {
            HookCallbackMessage::AddCallback {
                id,
                response_channel,
            } => {
                self.hook_response_channels.insert(id, response_channel);
            }
            HookCallbackMessage::RemoveCallback { id } => {
                self.hook_response_channels.remove(&id);
            }
        }

        Ok(())
    }

    pub async fn run_task_inner(mut self) -> anyhow::Result<()> {
        loop {
            // We might not have a connection - if we don't, return a future which never completes.
            // One of the other futures will occur instead, such as the one to receive a new client connection.
            let receive_connection_message_future = async {
                match self.connection_reader.as_mut() {
                    Some(connection_reader) => connection_reader.next().await,
                    None => future::pending().await,
                }
            };

            tokio::select! {
                client_message = receive_connection_message_future => {
                    self.handle_client_message(client_message).await?;
                }

                new_client_connection = self.listener.accept() => {
                    self.handle_new_client_connection(new_client_connection).await?;
                }

                hook_callback_message = self.hook_callback_receiver.recv() => {
                    self.handle_hook_callback_message(hook_callback_message).await?;
                }
            }
        }
    }

    pub async fn run_task(self) {
        match self.run_task_inner().await {
            Ok(()) => (),
            Err(error) => {
                error!("{}", error);
            }
        }
    }
}
