mod abort_tasks_on_drop;
mod event_callbacks;
mod plugin_state;
mod server;

use crate::abort_tasks_on_drop::AbortTaskOnDrop;
use crate::plugin_state::PluginState;
use cln_plugin::Builder;
use event_callbacks::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let plugin_state = Arc::new(PluginState::new().await?);

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

#[cfg(test)]
mod test {
    use super::*;
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;

    #[tokio::test]
    pub async fn test_websocket_clean_connection_and_disconnection() -> anyhow::Result<()> {
        let mut plugin_state = PluginState::new().await?;

        {
            let (ws_stream, _) = connect_async("ws://127.0.0.1:3012").await?;
            let (mut write, _read) = StreamExt::split(ws_stream);

            // Wait for websocket message sender to become available
            tokio::time::timeout(
                Duration::from_secs(1),
                plugin_state
                    .websocket_message_sender_watch_receiver
                    .wait_for(|value| value.is_some()),
            )
            .await??;

            write.close().await?;
        }

        // Wait for websocket message sender to become unavailable, since the connection closed.
        tokio::time::timeout(
            Duration::from_secs(1),
            plugin_state
                .websocket_message_sender_watch_receiver
                .wait_for(|value| value.is_none()),
        )
        .await??;

        Ok(())
    }
}
