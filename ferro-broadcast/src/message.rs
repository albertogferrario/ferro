//! Broadcast message types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

/// A message that can be broadcast to channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    /// The event name.
    pub event: String,
    /// The channel name.
    pub channel: String,
    /// The message data.
    pub data: Value,
}

impl BroadcastMessage {
    /// Create a new broadcast message.
    pub fn new(channel: impl Into<String>, event: impl Into<String>, data: impl Serialize) -> Self {
        Self {
            channel: channel.into(),
            event: event.into(),
            data: serde_json::to_value(data).unwrap_or(Value::Null),
        }
    }

    /// Create with raw JSON data.
    pub fn with_data(channel: impl Into<String>, event: impl Into<String>, data: Value) -> Self {
        Self {
            channel: channel.into(),
            event: event.into(),
            data,
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert to a WebSocket text message.
    pub fn to_ws_message(&self) -> Result<WsMessage, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(WsMessage::Text(json.into()))
    }
}

/// A client-to-server message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to a channel.
    Subscribe {
        channel: String,
        #[serde(default)]
        auth: Option<String>,
        /// Optional member data for presence channels (e.g. `{"user_id": "42", "user_info": {"name": "Alice"}}`).
        #[serde(default)]
        channel_data: Option<Value>,
    },
    /// Unsubscribe from a channel.
    Unsubscribe { channel: String },
    /// Send a message to a channel (client events).
    Whisper {
        channel: String,
        event: String,
        data: Value,
    },
    /// Ping to keep connection alive.
    Ping,
}

impl ClientMessage {
    /// Parse a client message from WebSocket text payload.
    pub fn from_ws_text(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// A server-to-client message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Connection established.
    Connected { socket_id: String },
    /// Subscription successful.
    Subscribed { channel: String },
    /// Subscription failed.
    SubscriptionError { channel: String, error: String },
    /// Unsubscribed from channel.
    Unsubscribed { channel: String },
    /// Broadcast event.
    Event(BroadcastMessage),
    /// Member joined (presence channels).
    MemberAdded {
        channel: String,
        user_id: String,
        user_info: Value,
    },
    /// Member left (presence channels).
    MemberRemoved { channel: String, user_id: String },
    /// Pong response.
    Pong,
    /// Error message.
    Error { message: String },
}

impl ServerMessage {
    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert to a WebSocket text message.
    pub fn to_ws_message(&self) -> Result<WsMessage, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(WsMessage::Text(json.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcast_message() {
        let msg = BroadcastMessage::new("orders.1", "OrderUpdated", serde_json::json!({"id": 1}));
        assert_eq!(msg.channel, "orders.1");
        assert_eq!(msg.event, "OrderUpdated");
    }

    #[test]
    fn test_client_message_serialize() {
        let msg = ClientMessage::Subscribe {
            channel: "private-orders.1".into(),
            auth: Some("auth_token".into()),
            channel_data: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("private-orders.1"));
    }

    #[test]
    fn test_subscribe_with_channel_data() {
        let json = r#"{"type":"subscribe","channel":"presence-nearby","auth":"ok","channel_data":{"user_id":"42","user_info":{"name":"Alice"}}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Subscribe {
                channel,
                auth,
                channel_data,
            } => {
                assert_eq!(channel, "presence-nearby");
                assert_eq!(auth.unwrap(), "ok");
                let data = channel_data.unwrap();
                assert_eq!(data["user_id"], "42");
                assert_eq!(data["user_info"]["name"], "Alice");
            }
            _ => panic!("Expected Subscribe"),
        }
    }

    #[test]
    fn test_subscribe_without_channel_data() {
        let json = r#"{"type":"subscribe","channel":"presence-nearby","auth":"ok"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Subscribe { channel_data, .. } => {
                assert!(channel_data.is_none());
            }
            _ => panic!("Expected Subscribe"),
        }
    }

    #[test]
    fn test_server_message_serialize() {
        let msg = ServerMessage::Connected {
            socket_id: "abc123".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("connected"));
        assert!(json.contains("abc123"));
    }
}
