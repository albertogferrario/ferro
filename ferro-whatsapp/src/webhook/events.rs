use crate::client::WhatsApp;
use crate::message::{DeliveryStatus, SenderIdentity};
use ferro_events::Event;

/// Ferro event emitted when a WhatsApp text message is received.
///
/// Dispatched by [`ProcessWhatsAppWebhook`] for each inbound message in
/// the Meta webhook payload. Attach a [`ferro_events::Listener`] to handle it.
#[derive(Debug, Clone)]
pub struct WhatsAppTextReceived {
    /// WhatsApp message ID (`wamid.xxx`). Use for delivery status correlation.
    pub wamid: String,
    /// Sender identity resolved via [`WhatsAppConfig::is_owner`].
    pub sender_identity: SenderIdentity,
    /// Text body of the message.
    pub text: String,
    /// Unix timestamp of the message.
    pub timestamp: i64,
    /// Raw Meta webhook message object for custom field access.
    pub raw: serde_json::Value,
}

impl Event for WhatsAppTextReceived {
    fn name(&self) -> &'static str {
        "whatsapp.message.received"
    }
}

/// Ferro event emitted when a WhatsApp delivery status update is received.
///
/// Dispatched by [`ProcessWhatsAppWebhook`] for each status entry in
/// the Meta webhook payload.
#[derive(Debug, Clone)]
pub struct WhatsAppStatusUpdate {
    /// WhatsApp message ID (`wamid.xxx`) this status refers to.
    pub wamid: String,
    /// Delivery status from Meta.
    pub status: DeliveryStatus,
    /// Unix timestamp of the status update.
    pub timestamp: i64,
}

impl Event for WhatsAppStatusUpdate {
    fn name(&self) -> &'static str {
        "whatsapp.status.update"
    }
}

/// Background job that processes a WhatsApp webhook payload asynchronously.
///
/// Webhook handlers dispatch this job immediately after HMAC verification,
/// returning HTTP 200 to Meta without blocking on event processing.
/// The job parses the Meta JSON envelope and dispatches typed ferro-events.
///
/// ## Deduplication
///
/// Deduplication is handled at the application layer. Wire an
/// [`InMemoryDeduplicationStore`][crate::InMemoryDeduplicationStore] (or custom
/// [`DeduplicationStore`][crate::DeduplicationStore]) into your webhook handler
/// before dispatching this job to avoid processing duplicate `wamid`s.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessWhatsAppWebhook {
    /// The raw JSON body received from Meta's webhook POST.
    pub payload_json: String,
}

#[ferro_queue::async_trait]
impl ferro_queue::Job for ProcessWhatsAppWebhook {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        let v: serde_json::Value = serde_json::from_str(&self.payload_json)
            .map_err(|e| ferro_queue::Error::custom(format!("invalid webhook JSON: {e}")))?;

        let value = &v["entry"][0]["changes"][0]["value"];
        let is_owner = &WhatsApp::config().is_owner;

        for event in parse_text_messages(value, is_owner.as_ref()) {
            event.dispatch_sync();
        }

        for event in parse_status_updates(value) {
            event.dispatch_sync();
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "ProcessWhatsAppWebhook"
    }
}

fn parse_text_messages(
    value: &serde_json::Value,
    is_owner: &dyn Fn(&str) -> bool,
) -> Vec<WhatsAppTextReceived> {
    let Some(messages) = value["messages"].as_array() else {
        return vec![];
    };

    messages
        .iter()
        .filter_map(|msg| {
            let wamid = msg["id"].as_str()?.to_string();
            let from_phone = msg["from"].as_str()?.to_string();
            let timestamp: i64 = msg["timestamp"].as_str()?.parse().ok()?;
            let text = msg["text"]["body"].as_str()?.to_string();
            let sender_identity = resolve_identity(&from_phone, is_owner);
            Some(WhatsAppTextReceived {
                wamid,
                sender_identity,
                text,
                timestamp,
                raw: msg.clone(),
            })
        })
        .collect()
}

fn parse_status_updates(value: &serde_json::Value) -> Vec<WhatsAppStatusUpdate> {
    let Some(statuses) = value["statuses"].as_array() else {
        return vec![];
    };

    statuses
        .iter()
        .filter_map(|status| {
            let wamid = status["id"].as_str()?.to_string();
            let status_str = status["status"].as_str()?;
            let timestamp: i64 = status["timestamp"].as_str()?.parse().ok()?;
            let delivery_status: DeliveryStatus =
                serde_json::from_value(serde_json::Value::String(status_str.to_string()))
                    .unwrap_or(DeliveryStatus::Unknown);
            Some(WhatsAppStatusUpdate {
                wamid,
                status: delivery_status,
                timestamp,
            })
        })
        .collect()
}

fn resolve_identity(phone: &str, is_owner: &dyn Fn(&str) -> bool) -> SenderIdentity {
    if is_owner(phone) {
        SenderIdentity::Owner(phone.to_string())
    } else {
        SenderIdentity::Customer(phone.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_queue::Job;

    fn sample_text_webhook(
        wamid: &str,
        from: &str,
        text: &str,
        timestamp: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "entry": [{
                "changes": [{
                    "value": {
                        "messages": [{
                            "id": wamid,
                            "from": from,
                            "timestamp": timestamp,
                            "type": "text",
                            "text": { "body": text }
                        }]
                    }
                }]
            }]
        })
    }

    fn sample_status_webhook(wamid: &str, status: &str, timestamp: &str) -> serde_json::Value {
        serde_json::json!({
            "entry": [{
                "changes": [{
                    "value": {
                        "statuses": [{
                            "id": wamid,
                            "status": status,
                            "timestamp": timestamp
                        }]
                    }
                }]
            }]
        })
    }

    #[test]
    fn text_received_event_name() {
        let event = WhatsAppTextReceived {
            wamid: "wamid.001".into(),
            sender_identity: SenderIdentity::Customer("393401234567".into()),
            text: "hello".into(),
            timestamp: 1700000000,
            raw: serde_json::Value::Null,
        };
        assert_eq!(event.name(), "whatsapp.message.received");
    }

    #[test]
    fn status_update_event_name() {
        let event = WhatsAppStatusUpdate {
            wamid: "wamid.001".into(),
            status: DeliveryStatus::Delivered,
            timestamp: 1700000001,
        };
        assert_eq!(event.name(), "whatsapp.status.update");
    }

    #[test]
    fn process_job_name() {
        let job = ProcessWhatsAppWebhook {
            payload_json: "{}".to_string(),
        };
        assert_eq!(job.name(), "ProcessWhatsAppWebhook");
    }

    #[test]
    fn sender_identity_owner() {
        let identity = resolve_identity("393401234567", &|phone| phone == "393401234567");
        assert!(
            matches!(identity, SenderIdentity::Owner(_)),
            "is_owner=true must produce Owner"
        );
        assert_eq!(identity.phone(), "393401234567");
    }

    #[test]
    fn sender_identity_customer() {
        let identity = resolve_identity("393409999999", &|_| false);
        assert!(
            matches!(identity, SenderIdentity::Customer(_)),
            "is_owner=false must produce Customer (safe default)"
        );
        assert_eq!(identity.phone(), "393409999999");
    }

    #[test]
    fn parse_text_message() {
        let payload = sample_text_webhook("wamid.abc", "393401234567", "Ciao!", "1700000000");
        let value = &payload["entry"][0]["changes"][0]["value"];
        let events = parse_text_messages(value, &|_| false);

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.wamid, "wamid.abc");
        assert_eq!(event.text, "Ciao!");
        assert_eq!(event.timestamp, 1700000000);
        assert!(matches!(event.sender_identity, SenderIdentity::Customer(_)));
    }

    #[test]
    fn parse_status_update() {
        let payload = sample_status_webhook("wamid.xyz", "delivered", "1700000001");
        let value = &payload["entry"][0]["changes"][0]["value"];
        let events = parse_status_updates(value);

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.wamid, "wamid.xyz");
        assert_eq!(event.timestamp, 1700000001);
        assert!(matches!(event.status, DeliveryStatus::Delivered));
    }

    // Compile-time check: all event types are Clone + Send + Sync
    fn _assert_clone_send_sync<T: Clone + Send + Sync>() {}

    #[test]
    fn events_are_clone_send_sync() {
        _assert_clone_send_sync::<WhatsAppTextReceived>();
        _assert_clone_send_sync::<WhatsAppStatusUpdate>();
        _assert_clone_send_sync::<ProcessWhatsAppWebhook>();
    }
}
