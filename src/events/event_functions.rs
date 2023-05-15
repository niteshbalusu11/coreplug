use anyhow::Error;
use cln_plugin::Plugin;

use crate::PluginState;

pub async fn channel_opened(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: channel_opened {:?}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn channel_open_failed(
    p: Plugin<PluginState>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: channel_open_failed {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn channel_state_changed(
    p: Plugin<PluginState>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: channel_state_changed {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn connect(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: connect {}", v);

    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn disconnect(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: disconnect {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn invoice_payment(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: invoice_payment {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn invoice_creation(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: invoice_creation {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn warning(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: warning {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn forward_event(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: forward_event {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn sendpay_success(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: sendpay_success {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn sendpay_failure(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: sendpay_failure {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn coin_movement(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: coin_movement {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn balance_snapshot(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: balance_snapshot {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn block_added(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: block_added {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn openchannel_peer_sigs(
    p: Plugin<PluginState>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("CorePlug: openchannel_peer_sigs {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}

pub async fn shutdown(p: Plugin<PluginState>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: shutdown {}", v);
    tokio::spawn(async move { p.state().sender.send(v).await });

    Ok(())
}
