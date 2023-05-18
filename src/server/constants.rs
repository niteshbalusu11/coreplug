pub static EVENTS_LIST: &'static [&'static str] = &[
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

// TODO: Add values
pub static HOOKS_LIST: &'static [&'static str] = &[];
