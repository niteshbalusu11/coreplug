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
