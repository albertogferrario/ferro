//! Fluent API for broadcasting messages.

use crate::broadcaster::Broadcaster;
use crate::Error;
use serde::Serialize;
use std::sync::Arc;

/// A fluent builder for broadcasting messages.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_broadcast::Broadcast;
///
/// Broadcast::channel("orders.1")
///     .event("OrderUpdated")
///     .data(&order)
///     .send()
///     .await?;
/// ```
pub struct Broadcast {
    broadcaster: Arc<Broadcaster>,
}

impl Broadcast {
    /// Create a new Broadcast with the given broadcaster.
    pub fn new(broadcaster: Arc<Broadcaster>) -> Self {
        Self { broadcaster }
    }

    /// Start building a broadcast to a channel.
    pub fn channel(&self, name: impl Into<String>) -> BroadcastBuilder {
        BroadcastBuilder {
            broadcaster: self.broadcaster.clone(),
            channel: name.into(),
            event: None,
            data: None,
            except: None,
        }
    }

    /// Get the underlying broadcaster.
    pub fn broadcaster(&self) -> &Arc<Broadcaster> {
        &self.broadcaster
    }
}

/// Builder for constructing a broadcast message.
pub struct BroadcastBuilder {
    broadcaster: Arc<Broadcaster>,
    channel: String,
    event: Option<String>,
    data: Option<serde_json::Value>,
    except: Option<String>,
}

impl BroadcastBuilder {
    /// Set the event name.
    pub fn event(mut self, name: impl Into<String>) -> Self {
        self.event = Some(name.into());
        self
    }

    /// Set the data payload.
    pub fn data<T: Serialize>(mut self, data: T) -> Self {
        self.data = serde_json::to_value(data).ok();
        self
    }

    /// Exclude a specific client from receiving the broadcast.
    pub fn except(mut self, socket_id: impl Into<String>) -> Self {
        self.except = Some(socket_id.into());
        self
    }

    /// Send the broadcast.
    pub async fn send(self) -> Result<(), Error> {
        let event = self.event.unwrap_or_else(|| "message".into());
        let data = self.data.unwrap_or(serde_json::Value::Null);

        if let Some(except) = self.except {
            self.broadcaster
                .broadcast_except(&self.channel, &event, data, &except)
                .await
        } else {
            self.broadcaster
                .broadcast(&self.channel, &event, data)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcast_builder() {
        let broadcaster = Arc::new(Broadcaster::new());
        let broadcast = Broadcast::new(broadcaster);

        // This won't send to anyone since no clients, but it should not error
        let result = broadcast
            .channel("orders.1")
            .event("OrderUpdated")
            .data(serde_json::json!({"id": 1}))
            .send()
            .await;

        assert!(result.is_ok());
    }
}
