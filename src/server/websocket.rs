use futures::stream::SplitStream;

use crate::plugin_state::HookCallbackMessage;
use crate::server::constants::{EVENTS_LIST, HOOKS_LIST};
use crate::AbortTaskOnDrop;
use anyhow::{bail, Context};
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::{future, stream::StreamExt, SinkExt};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::watch;

pub struct ServerState {
    websocket_message_sender_watch_sender: watch::Sender<Option<mpsc::UnboundedSender<Message>>>,
    hook_callback_receiver: mpsc::UnboundedReceiver<HookCallbackMessage>,
    _axum_server_task: AbortTaskOnDrop,
    websocket_receiver: mpsc::UnboundedReceiver<WebSocket>,
    hook_response_channels: HashMap<usize, oneshot::Sender<serde_json::Value>>,
    connection_reader: Option<SplitStream<WebSocket>>,
    listened_events_watch_sender: watch::Sender<Vec<&'static str>>,
    listened_hooks_watch_sender: watch::Sender<Vec<&'static str>>,
    client_write_task: Option<AbortTaskOnDrop>,
}

impl ServerState {
    pub async fn new(
        websocket_message_sender_watch_sender: watch::Sender<
            Option<mpsc::UnboundedSender<Message>>,
        >,
        hook_callback_receiver: mpsc::UnboundedReceiver<HookCallbackMessage>,
        listened_events_watch_sender: watch::Sender<Vec<&'static str>>,
        listened_hooks_watch_sender: watch::Sender<Vec<&'static str>>,
    ) -> anyhow::Result<Self> {
        let (websocket_sender, websocket_receiver) = mpsc::unbounded_channel();

        async fn websocket_handler(
            ws: WebSocketUpgrade,
            axum::extract::State(sender): axum::extract::State<mpsc::UnboundedSender<WebSocket>>,
        ) -> impl IntoResponse {
            ws.on_upgrade(|socket| async move {
                if let Err(_error) = sender.send(socket) {
                    error!("Failed to send websocket to server from Axum callback, server must have died");
                }
            })
        }

        let listen_address = "0.0.0.0:3012";

        let axum_server_task_handle =
            tokio::spawn(
                axum::Server::bind(&listen_address.parse().with_context(|| {
                    format!("Failed to parse listen address: {}", listen_address)
                })?)
                .serve(
                    Router::new()
                        .route("/websocket", get(websocket_handler))
                        .with_state(websocket_sender)
                        .into_make_service(),
                ),
            );

        Ok(Self {
            websocket_message_sender_watch_sender,
            hook_callback_receiver,
            _axum_server_task: axum_server_task_handle.into(),
            websocket_receiver,
            hook_response_channels: HashMap::new(),
            connection_reader: None,
            listened_events_watch_sender,
            listened_hooks_watch_sender,
            client_write_task: None,
        })
    }

    pub async fn handle_client_message(
        &mut self,
        message: Option<Result<Message, axum::Error>>,
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
            SetHookSubscriptions {
                hooks: Vec<String>,
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
            ClientMessage::SetHookSubscriptions { hooks } => {
                let hooks = hooks
                    .into_iter()
                    .map(|hook| {
                        match HOOKS_LIST
                            .iter()
                            .find(|hooks_list_hook| hook.as_str() == **hooks_list_hook)
                        {
                            Some(hooks_list_hook) => Ok(*hooks_list_hook),
                            None => bail!("Invalid hook: {}", hook.as_str()),
                        }
                    })
                    .collect::<anyhow::Result<Vec<&'static str>>>()?;

                info!("Setting hook subscriptions: {:?}", hooks);

                if let Err(_error) = self.listened_hooks_watch_sender.send(hooks) {
                    bail!("Failed to send new set of hooks to listen to - plugin must have died");
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
        websocket: Option<WebSocket>,
    ) -> anyhow::Result<()> {
        let Some(websocket) = websocket else {
            bail!("New websocket connection sender seems to have died, server must have died");
        };

        if self.connection_reader.is_some() {
            // We already have a connection, reject the new one
            if let Err(error) = websocket.close().await {
                bail!("Error rejecting (closing) new client connection while a client is already connected: {}", error);
            }

            return Ok(());
        }

        let (mut writer, reader) = StreamExt::split(websocket);
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

                new_client_connection = self.websocket_receiver.recv() => {
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
