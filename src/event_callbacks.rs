use crate::plugin_state::PluginState;
use anyhow::Error;
use cln_plugin::Plugin;
use std::sync::Arc;

pub async fn channel_opened(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: channel_opened {:?}", v);
    p.state().handle_event("channel_opened", v);
    Ok(())
}

pub async fn channel_open_failed(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: channel_open_failed {}", v);
    p.state().handle_event("channel_open_failed", v);
    Ok(())
}

pub async fn channel_state_changed(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: channel_state_changed {}", v);
    p.state().handle_event("channel_state_changed", v);
    Ok(())
}

pub async fn connect(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: connect {}", v);
    p.state().handle_event("connect", v);
    Ok(())
}

pub async fn disconnect(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: disconnect {}", v);
    p.state().handle_event("disconnect", v);
    Ok(())
}

pub async fn invoice_payment(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: invoice_payment {}", v);
    p.state().handle_event("invoice_payment", v);
    Ok(())
}

pub async fn invoice_creation(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: invoice_creation {}", v);
    p.state().handle_event("invoice_creation", v);
    Ok(())
}

pub async fn warning(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: warning {}", v);
    p.state().send_message(v);
    Ok(())
}

pub async fn forward_event(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: forward_event {}", v);
    p.state().handle_event("forward_event", v);
    Ok(())
}

pub async fn sendpay_success(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: sendpay_success {}", v);
    p.state().handle_event("sendpay_success", v);
    Ok(())
}

pub async fn sendpay_failure(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: sendpay_failure {}", v);
    p.state().handle_event("sendpay_failure", v);
    Ok(())
}

pub async fn coin_movement(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: coin_movement {}", v);
    p.state().handle_event("coin_movement", v);
    Ok(())
}

pub async fn balance_snapshot(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: balance_snapshot {}", v);
    p.state().handle_event("balance_snapshot", v);
    Ok(())
}

pub async fn block_added(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: block_added {}", v);
    p.state().handle_event("block_added", v);
    Ok(())
}

pub async fn openchannel_peer_sigs(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: openchannel_peer_sigs {}", v);
    p.state().handle_event("openchannel_peer_sigs", v);
    Ok(())
}
