use anyhow::Ok;
use log::info;
use serde::{Deserialize, Serialize};

use crate::server::constants::{EVENTS_LIST, WEBSOCKET_CLIENT_EVENT_SUBSCRIPTIONS};

#[derive(Serialize, Deserialize)]
struct EventSubscriptionMessage {
    event_type: String,
    events: Vec<String>,
}

pub async fn validate_message(message: String) -> Result<(), anyhow::Error> {
    let data = serde_json::from_str::<EventSubscriptionMessage>(&message)?;

    if data.event_type == "subscribe_events" {
        // Check if the events vector are valid comparing against the constant list of events
        for event in &data.events {
            if !EVENTS_LIST.contains(&event.as_str()) {
                return Err(anyhow::anyhow!("Invalid event: {}", event));
            }
        }

        for event in data.events {
            WEBSOCKET_CLIENT_EVENT_SUBSCRIPTIONS
                .lock()
                .await
                .push(event);
        }
    }

    info!(
        "Event subscriptions: {:?}",
        WEBSOCKET_CLIENT_EVENT_SUBSCRIPTIONS.lock().await
    );

    Ok(())
}

pub async fn validate_event(event: serde_json::Value) -> bool {
    info!("Validating event: {:?}", event["type"]);
    info!("Events list: {:?}", EVENTS_LIST);
    info!(
        "Event subscriptions: {:?}",
        WEBSOCKET_CLIENT_EVENT_SUBSCRIPTIONS.lock().await
    );
    // Check if event type is in the list of events
    match event["type"].as_str() {
        Some(event_type) => {
            if !EVENTS_LIST.contains(&event_type) {
                return false;
            }

            if !WEBSOCKET_CLIENT_EVENT_SUBSCRIPTIONS
                .lock()
                .await
                .contains(&event_type.to_string())
            {
                return false;
            }
        }
        None => return false,
    }

    return true;
}
