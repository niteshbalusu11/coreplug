use crate::plugin_state::PluginState;
use anyhow::Error;
use cln_plugin::Plugin;
use std::sync::Arc;

pub async fn channel_opened(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: channel_opened {:?}", v);

    let v = serde_json::json!({
        "type": "channel_opened",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn channel_open_failed(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: channel_open_failed {}", v);

    let v = serde_json::json!({
        "type": "channel_open_failed",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn channel_state_changed(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: channel_state_changed {}", v);

    let v = serde_json::json!({
        "type": "channel_state_changed",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn connect(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: connect {}", v);

    let v = serde_json::json!({
        "type": "connect",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn disconnect(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: disconnect {}", v);

    let v = serde_json::json!({
        "type": "disconnect",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn invoice_payment(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: invoice_payment {}", v);

    let v = serde_json::json!({
        "type": "invoice_payment",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn invoice_creation(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: invoice_creation {}", v);

    let v = serde_json::json!({
        "type": "invoice_creation",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn warning(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: warning {}", v);
    p.state().send_message(v);

    Ok(())
}

pub async fn forward_event(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: forward_event {}", v);

    let v = serde_json::json!({
        "type": "forward_event",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn sendpay_success(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: sendpay_success {}", v);

    let v = serde_json::json!({
        "type": "sendpay_success",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn sendpay_failure(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: sendpay_failure {}", v);

    let v = serde_json::json!({
        "type": "sendpay_failure",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn coin_movement(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: coin_movement {}", v);

    let v = serde_json::json!({
        "type": "coin_movement",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn balance_snapshot(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: balance_snapshot {}", v);

    let v = serde_json::json!({
        "type": "balance_snapshot",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn block_added(p: Plugin<Arc<PluginState>>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: block_added {}", v);

    let v = serde_json::json!({
        "type": "block_added",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}

pub async fn openchannel_peer_sigs(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: openchannel_peer_sigs {}", v);

    let v = serde_json::json!({
        "type": "openchannel_peer_sigs",
        "data": v
    });

    p.state().send_message(v);

    Ok(())
}
