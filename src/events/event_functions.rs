use anyhow::Error;
use cln_plugin::Plugin;

use crate::PluginState;

pub async fn channel_opened(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: channel_opened {:?}", v);
    Ok(())
}

pub async fn channel_open_failed(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: channel_open_failed {}", v);
    Ok(())
}

pub async fn channel_state_changed(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: channel_state_changed {}", v);
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

pub async fn invoice_payment(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: invoice_payment {}", v);
    Ok(())
}

pub async fn invoice_creation(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: invoice_creation {}", v);
    Ok(())
}

pub async fn warning(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: warning {}", v);
    Ok(())
}

pub async fn forward_event(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: forward_event {}", v);
    Ok(())
}

pub async fn sendpay_success(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: sendpay_success {}", v);
    Ok(())
}

pub async fn sendpay_failure(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: sendpay_failure {}", v);
    Ok(())
}

pub async fn coin_movement(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: coin_movement {}", v);
    Ok(())
}

pub async fn balance_snapshot(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: balance_snapshot {}", v);
    Ok(())
}

pub async fn block_added(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: block_added {}", v);
    Ok(())
}

pub async fn openchannel_peer_sigs(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: openchannel_peer_sigs {}", v);
    Ok(())
}

pub async fn shutdown(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::info!("CorePlug: shutdown {}", v);
    Ok(())
}
