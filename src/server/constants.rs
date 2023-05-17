use std::sync::Arc;

use tokio::sync::Mutex;

pub static EVENTS_LIST: &[&str] = &[
    "connect",
    "disconnect",
    "channel_opened",
    "channel_open_failed",
    "channel_state_changed",
    "invoice_payment",
    "invoice_creation",
    "warning",
    "forward_event",
    "sendpay_success",
    "sendpay_failure",
    "coin_movement",
    "balance_snapshot",
    "block_added",
    "openchannel_peer_sigs",
];

lazy_static::lazy_static! {
    pub static ref WEBSOCKET_CLIENT_CONNECTED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    pub static ref WEBSOCKET_CLIENT_EVENT_SUBSCRIPTIONS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}
