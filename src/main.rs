mod events;
mod server;

extern crate serde_json;
use std::sync::Arc;

use cln_plugin::Builder;

use events::event_functions::{
    balance_snapshot, block_added, channel_open_failed, channel_opened, channel_state_changed,
    coin_movement, connect, disconnect, forward_event, invoice_creation, invoice_payment,
    openchannel_peer_sigs, sendpay_failure, sendpay_success, warning,
};

use server::websocket::websocket_connection_reader_task;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio_tungstenite::tungstenite;

#[derive(Clone)]
pub struct PluginState {
    /// To write to this, use tungstenite::Message::Text( your string ) or whatever
    /// eg.                 serde_json::json!({
    ///     "type": "error",
    ///     "message": "Only one client is allowed to connect at a time."
    /// })
    /// .to_string()
    websocket_message_sender_watch_receiver:
        watch::Receiver<Option<mpsc::UnboundedSender<tungstenite::Message>>>,
    hook_callback_sender: mpsc::UnboundedSender<HookCallbackMessage>,
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (websocket_message_sender_watch_sender, websocket_message_sender_watch_receiver) =
        watch::channel(Option::<mpsc::UnboundedSender<tungstenite::Message>>::None);
    let (hook_callback_sender, hook_callback_receiver) = mpsc::unbounded_channel();
    // let (configured_connection_sender, configured_connection_receiver) = mpsc::unbounded_channel();

    let plugin_state = Arc::new(PluginState {
        websocket_message_sender_watch_receiver,
        hook_callback_sender,
    });

    tokio::spawn(websocket_connection_reader_task(
        websocket_message_sender_watch_sender,
        hook_callback_receiver,
    ));

    if let Some(plugin) = Builder::new(tokio::io::stdin(), tokio::io::stdout())
        .dynamic()
        .subscribe("connect", connect)
        .subscribe("disconnect", disconnect)
        .subscribe("channel_opened", channel_opened)
        .subscribe("channel_open_failed", channel_open_failed)
        .subscribe("channel_state_changed", channel_state_changed)
        .subscribe("invoice_payment", invoice_payment)
        .subscribe("invoice_creation", invoice_creation)
        .subscribe("warning", warning)
        .subscribe("forward_event", forward_event)
        .subscribe("sendpay_success", sendpay_success)
        .subscribe("sendpay_failure", sendpay_failure)
        .subscribe("coin_movement", coin_movement)
        .subscribe("balance_snapshot", balance_snapshot)
        .subscribe("block_added", block_added)
        .subscribe("openchannel_peer_sigs", openchannel_peer_sigs)
        .start(plugin_state)
        .await?
    {
        let plug_res = plugin.join().await;

        plug_res
    } else {
        Ok(())
    }
}
