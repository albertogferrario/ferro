use serde::{Deserialize, Serialize};

/// Outbound WhatsApp message variant.
///
/// Build a message and pass it to [`WhatsApp::send`](crate::WhatsApp::send).
#[derive(Debug, Clone)]
pub enum Message {
    /// Plain text message.
    Text {
        /// Message body text.
        body: String,
    },
    /// Template message.
    ///
    /// Templates must be pre-approved by Meta before use.
    /// Each element in `parameters` must be a typed parameter object per Meta spec,
    /// e.g. `{"type": "text", "text": "value"}` or `{"type": "currency", ...}`.
    Template {
        /// Template name as registered in Meta Business Manager.
        name: String,
        /// Language code, e.g. `"en_US"` or `"it"`.
        language: String,
        /// Typed parameter objects for template variable substitution.
        parameters: Vec<serde_json::Value>,
    },
}

impl Message {
    /// Serializes the message into the Meta Cloud API JSON payload format.
    ///
    /// The `to` field is the recipient phone number in E.164 format without `+`.
    pub fn to_api_payload(&self, to: &str) -> serde_json::Value {
        match self {
            Message::Text { body } => serde_json::json!({
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": to,
                "type": "text",
                "text": { "body": body }
            }),
            Message::Template {
                name,
                language,
                parameters,
            } => serde_json::json!({
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": to,
                "type": "template",
                "template": {
                    "name": name,
                    "language": { "code": language },
                    "components": parameters
                }
            }),
        }
    }
}

/// Result returned by a successful [`WhatsApp::send`](crate::WhatsApp::send) call.
#[derive(Debug, Clone)]
pub struct SendResult {
    /// WhatsApp message ID returned by the Meta API.
    ///
    /// Use this ID to correlate outbound messages with delivery status webhooks.
    pub wamid: String,
}

/// Identifies whether a WhatsApp message was sent by the business owner or a customer.
///
/// The phone number is in E.164 format without `+` prefix (as delivered by Meta).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "phone")]
#[serde(rename_all = "snake_case")]
pub enum SenderIdentity {
    /// Message sent by the business owner.
    Owner(String),
    /// Message sent by a customer.
    Customer(String),
}

impl SenderIdentity {
    /// Returns the phone number regardless of identity type.
    pub fn phone(&self) -> &str {
        match self {
            SenderIdentity::Owner(p) | SenderIdentity::Customer(p) => p,
        }
    }
}

/// WhatsApp message delivery status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    /// Message sent to Meta's servers.
    Sent,
    /// Message delivered to the recipient's device.
    Delivered,
    /// Message read by the recipient.
    Read,
    /// Message delivery failed.
    Failed,
    /// Unknown status string from Meta.
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_message_produces_correct_meta_api_json() {
        let msg = Message::Text {
            body: "Hello World".into(),
        };
        let payload = msg.to_api_payload("393401234567");

        assert_eq!(payload["messaging_product"], "whatsapp");
        assert_eq!(payload["recipient_type"], "individual");
        assert_eq!(payload["to"], "393401234567");
        assert_eq!(payload["type"], "text");
        assert_eq!(payload["text"]["body"], "Hello World");
    }

    #[test]
    fn template_message_produces_correct_meta_api_json() {
        let params = vec![
            serde_json::json!({"type": "text", "text": "Alberto"}),
            serde_json::json!({"type": "text", "text": "123456"}),
        ];
        let msg = Message::Template {
            name: "order_confirmation".into(),
            language: "en_US".into(),
            parameters: params.clone(),
        };
        let payload = msg.to_api_payload("393401234567");

        assert_eq!(payload["messaging_product"], "whatsapp");
        assert_eq!(payload["recipient_type"], "individual");
        assert_eq!(payload["to"], "393401234567");
        assert_eq!(payload["type"], "template");
        assert_eq!(payload["template"]["name"], "order_confirmation");
        assert_eq!(payload["template"]["language"]["code"], "en_US");
        assert_eq!(payload["template"]["components"], serde_json::json!(params));
    }

    #[test]
    fn sender_identity_owner_carries_phone_number() {
        let identity = SenderIdentity::Owner("393401234567".into());
        assert_eq!(identity.phone(), "393401234567");
        assert!(matches!(identity, SenderIdentity::Owner(_)));
    }

    #[test]
    fn sender_identity_customer_carries_phone_number() {
        let identity = SenderIdentity::Customer("393409999999".into());
        assert_eq!(identity.phone(), "393409999999");
        assert!(matches!(identity, SenderIdentity::Customer(_)));
    }

    #[test]
    fn delivery_status_deserializes_known_variants() {
        assert_eq!(
            serde_json::from_str::<DeliveryStatus>("\"sent\"").unwrap(),
            DeliveryStatus::Sent
        );
        assert_eq!(
            serde_json::from_str::<DeliveryStatus>("\"delivered\"").unwrap(),
            DeliveryStatus::Delivered
        );
        assert_eq!(
            serde_json::from_str::<DeliveryStatus>("\"read\"").unwrap(),
            DeliveryStatus::Read
        );
        assert_eq!(
            serde_json::from_str::<DeliveryStatus>("\"failed\"").unwrap(),
            DeliveryStatus::Failed
        );
    }

    #[test]
    fn delivery_status_deserializes_unknown_variant() {
        let status: DeliveryStatus = serde_json::from_str("\"expired\"").unwrap();
        assert_eq!(status, DeliveryStatus::Unknown);
    }
}
