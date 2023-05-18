use crate::abort_tasks_on_drop::AbortTaskOnDrop;
use crate::server::websocket::ServerState;
use anyhow::bail;
use log::error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio_tungstenite::tungstenite;

pub struct PluginState {
    pub websocket_message_sender_watch_receiver:
        watch::Receiver<Option<mpsc::UnboundedSender<tungstenite::Message>>>,
    listened_events_watch_receiver: watch::Receiver<Vec<&'static str>>,
    listened_hooks_watch_receiver: watch::Receiver<Vec<&'static str>>,
    hook_callback_sender: mpsc::UnboundedSender<HookCallbackMessage>,
    next_hook_id: AtomicUsize,
    _server_task_handle: AbortTaskOnDrop,
}

impl PluginState {
    pub async fn new() -> anyhow::Result<PluginState> {
        let (websocket_message_sender_watch_sender, websocket_message_sender_watch_receiver) =
            watch::channel(Option::<mpsc::UnboundedSender<tungstenite::Message>>::None);
        let (listened_events_watch_sender, listened_events_watch_receiver) =
            watch::channel(Vec::new());
        let (listened_hooks_watch_sender, listened_hooks_watch_receiver) =
            watch::channel(Vec::new());
        let (hook_callback_sender, hook_callback_receiver) = mpsc::unbounded_channel();

        let server_task_handle = tokio::spawn(
            ServerState::new(
                websocket_message_sender_watch_sender,
                hook_callback_receiver,
                listened_events_watch_sender,
                listened_hooks_watch_sender,
            )
            .await?
            .run_task(),
        );

        let plugin_state = PluginState {
            websocket_message_sender_watch_receiver,
            hook_callback_sender,
            listened_events_watch_receiver,
            listened_hooks_watch_receiver,
            next_hook_id: AtomicUsize::new(1),
            _server_task_handle: server_task_handle.into(),
        };

        Ok(plugin_state)
    }

    pub fn is_event_subscribed(&self, event: &'static str) -> bool {
        self.listened_events_watch_receiver
            .borrow()
            .contains(&event)
    }

    pub fn is_hook_subscribed(&self, hook: &'static str) -> bool {
        self.listened_hooks_watch_receiver.borrow().contains(&hook)
    }

    /// Sends a message to the connected websocket, if there is one.
    pub fn send_message(&self, message: serde_json::Value) {
        if let Some(sender) = self
            .websocket_message_sender_watch_receiver
            .borrow()
            .as_ref()
        {
            match sender.send(tungstenite::Message::Text(message.to_string())) {
                Ok(()) => (),
                Err(_error) => {
                    error!("Failed to send message to web socket - the receiver must have dropped");
                }
            }
        }
    }

    pub async fn send_hook_message_and_await_response(
        &self,
        message: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let id = self.next_hook_id.fetch_add(1, Ordering::SeqCst); // not sure if ordering is correct
        let (response_channel, response_receiver) = oneshot::channel();

        // First set up our ability to receive a response for the hook
        if let Err(_error) = self
            .hook_callback_sender
            .send(HookCallbackMessage::AddCallback {
                id,
                response_channel,
            })
        {
            bail!("Failed to send hook callback message to add hook, server must have died");
        }

        // Then send the message to the client
        self.send_message(serde_json::json!({
            "type": "hook",
            "id": id,
            "message": message,
        }));

        // Then await the client's response, with a timeout - if it times out, make sure to remove the hook response callback, otherwise if the client never responds, the response channel will effectively leak!
        let Ok(response) = tokio::time::timeout(Duration::from_secs(10), response_receiver).await else {
            if let Err(_error) = self
                .hook_callback_sender
                .send(HookCallbackMessage::RemoveCallback { id })
            {
                bail!("Failed to send hook callback message to remove hook, server must have died");
            }

            bail!("Client failed to respond within timeout to hook #{}", id);
        };

        // Check if we got a response, or if we got told the sender was dropped
        let Ok(response) = response else {
            bail!("Hook response channel was closed while it was still being waited on for hook callback #{} - server may have died", id);
        };

        Ok(response)
    }
}

pub enum HookCallbackMessage {
    AddCallback {
        id: usize,
        response_channel: oneshot::Sender<serde_json::Value>,
    },
    RemoveCallback {
        id: usize,
    },
}
