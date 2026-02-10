//! The main broadcaster for managing channels and sending messages.

use crate::channel::{ChannelInfo, ChannelType, PresenceMember};
use crate::config::BroadcastConfig;
use crate::message::{BroadcastMessage, ServerMessage};
use crate::Error;
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// A connected client.
pub struct Client {
    /// Unique socket ID.
    pub socket_id: String,
    /// Sender to push messages to this client.
    pub sender: mpsc::Sender<ServerMessage>,
    /// Channels this client is subscribed to.
    pub channels: Vec<String>,
}

/// Shared state for the broadcaster.
struct BroadcasterInner {
    /// Connected clients by socket ID.
    clients: DashMap<String, Client>,
    /// Channels by name.
    channels: DashMap<String, ChannelInfo>,
    /// Optional authorization callback.
    authorizer: Option<Arc<dyn ChannelAuthorizer>>,
    /// Configuration.
    config: BroadcastConfig,
}

/// The broadcaster manages channels and client connections.
#[derive(Clone)]
pub struct Broadcaster {
    inner: Arc<BroadcasterInner>,
}

impl Broadcaster {
    /// Create a new broadcaster with default configuration.
    pub fn new() -> Self {
        Self::with_config(BroadcastConfig::default())
    }

    /// Create a new broadcaster with the given configuration.
    pub fn with_config(config: BroadcastConfig) -> Self {
        Self {
            inner: Arc::new(BroadcasterInner {
                clients: DashMap::new(),
                channels: DashMap::new(),
                authorizer: None,
                config,
            }),
        }
    }

    /// Set the channel authorizer.
    pub fn with_authorizer<A: ChannelAuthorizer + 'static>(self, authorizer: A) -> Self {
        Self {
            inner: Arc::new(BroadcasterInner {
                clients: DashMap::new(),
                channels: DashMap::new(),
                authorizer: Some(Arc::new(authorizer)),
                config: self.inner.config.clone(),
            }),
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &BroadcastConfig {
        &self.inner.config
    }

    /// Register a new client connection.
    pub fn add_client(&self, socket_id: String, sender: mpsc::Sender<ServerMessage>) {
        info!(socket_id = %socket_id, "Client connected");
        self.inner.clients.insert(
            socket_id.clone(),
            Client {
                socket_id,
                sender,
                channels: Vec::new(),
            },
        );
    }

    /// Remove a client and clean up their subscriptions.
    pub fn remove_client(&self, socket_id: &str) {
        if let Some((_, client)) = self.inner.clients.remove(socket_id) {
            info!(socket_id = %socket_id, "Client disconnected");

            // Remove from all subscribed channels
            for channel_name in &client.channels {
                self.unsubscribe_internal(socket_id, channel_name);
            }
        }
    }

    /// Subscribe a client to a channel.
    pub async fn subscribe(
        &self,
        socket_id: &str,
        channel_name: &str,
        auth: Option<&str>,
        member_info: Option<PresenceMember>,
    ) -> Result<(), Error> {
        let channel_type = ChannelType::from_name(channel_name);
        let config = &self.inner.config;

        // Check max channels limit
        if config.max_channels > 0
            && !self.inner.channels.contains_key(channel_name)
            && self.inner.channels.len() >= config.max_channels
        {
            warn!(channel = %channel_name, max = config.max_channels, "Max channels limit reached");
            return Err(Error::ChannelFull);
        }

        // Check authorization for private/presence channels
        if channel_type.requires_auth() {
            if let Some(authorizer) = &self.inner.authorizer {
                let auth_data = AuthData {
                    socket_id: socket_id.to_string(),
                    channel: channel_name.to_string(),
                    auth_token: auth.map(|s| s.to_string()),
                };
                if !authorizer.authorize(&auth_data).await {
                    warn!(socket_id = %socket_id, channel = %channel_name, "Authorization failed");
                    return Err(Error::unauthorized("Channel authorization failed"));
                }
            } else if auth.is_none() {
                return Err(Error::unauthorized("Authorization required"));
            }
        }

        // Get or create channel
        let mut channel = self
            .inner
            .channels
            .entry(channel_name.to_string())
            .or_insert_with(|| ChannelInfo::new(channel_name));

        // Check max subscribers limit
        if config.max_subscribers_per_channel > 0
            && channel.subscriber_count() >= config.max_subscribers_per_channel
        {
            warn!(
                channel = %channel_name,
                max = config.max_subscribers_per_channel,
                "Max subscribers per channel limit reached"
            );
            return Err(Error::ChannelFull);
        }

        // Add subscriber
        channel.add_subscriber(socket_id.to_string());

        // For presence channels, add member info
        if channel_type == ChannelType::Presence {
            if let Some(member) = member_info {
                channel.add_member(member.clone());

                // Notify other members
                let msg = ServerMessage::MemberAdded {
                    channel: channel_name.to_string(),
                    user_id: member.user_id.clone(),
                    user_info: member.user_info.clone(),
                };
                drop(channel); // Release lock before broadcasting
                self.send_to_channel_except(channel_name, socket_id, &msg)
                    .await;
            }
        } else {
            drop(channel);
        }

        // Update client's channel list
        if let Some(mut client) = self.inner.clients.get_mut(socket_id) {
            if !client.channels.contains(&channel_name.to_string()) {
                client.channels.push(channel_name.to_string());
            }
        }

        debug!(socket_id = %socket_id, channel = %channel_name, "Subscribed to channel");
        Ok(())
    }

    /// Unsubscribe a client from a channel.
    pub async fn unsubscribe(&self, socket_id: &str, channel_name: &str) {
        self.unsubscribe_internal(socket_id, channel_name);
    }

    fn unsubscribe_internal(&self, socket_id: &str, channel_name: &str) {
        // Remove from channel
        if let Some(mut channel) = self.inner.channels.get_mut(channel_name) {
            channel.remove_subscriber(socket_id);

            // For presence channels, notify about member leaving
            if channel.channel_type == ChannelType::Presence {
                if let Some(member) = channel.remove_member(socket_id) {
                    let msg = ServerMessage::MemberRemoved {
                        channel: channel_name.to_string(),
                        user_id: member.user_id,
                    };
                    // We can't await here, so we'll spawn a task
                    let channel_name = channel_name.to_string();
                    let broadcaster = self.clone();
                    tokio::spawn(async move {
                        broadcaster.send_to_channel(&channel_name, &msg).await;
                    });
                }
            }

            // Clean up empty channels
            if channel.is_empty() {
                drop(channel);
                self.inner.channels.remove(channel_name);
            }
        }

        // Update client's channel list
        if let Some(mut client) = self.inner.clients.get_mut(socket_id) {
            client.channels.retain(|c| c != channel_name);
        }

        debug!(socket_id = %socket_id, channel = %channel_name, "Unsubscribed from channel");
    }

    /// Broadcast a message to a channel.
    pub async fn broadcast<T: Serialize>(
        &self,
        channel: &str,
        event: &str,
        data: T,
    ) -> Result<(), Error> {
        let msg = BroadcastMessage::new(channel, event, data);
        let server_msg = ServerMessage::Event(msg);
        self.send_to_channel(channel, &server_msg).await;
        Ok(())
    }

    /// Forward a client event (whisper) to other subscribers of the same channel.
    ///
    /// Used for client-to-client messaging such as typing indicators and cursor positions.
    /// Only works when `allow_client_events` is enabled in configuration.
    /// The sender is excluded from receiving the whispered message.
    pub async fn whisper(
        &self,
        socket_id: &str,
        channel_name: &str,
        event: &str,
        data: serde_json::Value,
    ) -> Result<(), Error> {
        if !self.inner.config.allow_client_events {
            return Err(Error::Other("Client events are not allowed".into()));
        }

        // Verify client is subscribed to the channel
        let channel = self
            .inner
            .channels
            .get(channel_name)
            .ok_or_else(|| Error::ChannelNotFound(channel_name.to_string()))?;
        if !channel.subscribers.contains(socket_id) {
            return Err(Error::ClientNotConnected(format!(
                "Client {} is not subscribed to {}",
                socket_id, channel_name
            )));
        }
        drop(channel); // Release DashMap guard before await

        let msg = BroadcastMessage::with_data(channel_name, event, data);
        let server_msg = ServerMessage::Event(msg);
        self.send_to_channel_except(channel_name, socket_id, &server_msg)
            .await;

        Ok(())
    }

    /// Broadcast to a channel, excluding a specific client.
    pub async fn broadcast_except<T: Serialize>(
        &self,
        channel: &str,
        event: &str,
        data: T,
        except_socket_id: &str,
    ) -> Result<(), Error> {
        let msg = BroadcastMessage::new(channel, event, data);
        let server_msg = ServerMessage::Event(msg);
        self.send_to_channel_except(channel, except_socket_id, &server_msg)
            .await;
        Ok(())
    }

    /// Send a message to all subscribers of a channel.
    async fn send_to_channel(&self, channel_name: &str, msg: &ServerMessage) {
        if let Some(channel) = self.inner.channels.get(channel_name) {
            for socket_id in channel.subscribers.iter() {
                self.send_to_client(socket_id, msg.clone()).await;
            }
        }
    }

    /// Send a message to all subscribers except one.
    async fn send_to_channel_except(
        &self,
        channel_name: &str,
        except_socket_id: &str,
        msg: &ServerMessage,
    ) {
        if let Some(channel) = self.inner.channels.get(channel_name) {
            for socket_id in channel.subscribers.iter() {
                if socket_id.as_str() != except_socket_id {
                    self.send_to_client(socket_id, msg.clone()).await;
                }
            }
        }
    }

    /// Send a message to a specific client.
    async fn send_to_client(&self, socket_id: &str, msg: ServerMessage) {
        if let Some(client) = self.inner.clients.get(socket_id) {
            if let Err(e) = client.sender.send(msg).await {
                warn!(socket_id = %socket_id, error = %e, "Failed to send message to client");
            }
        }
    }

    /// Check if a client would be authorized for a channel.
    ///
    /// Returns true if:
    /// - Channel is public (no auth needed)
    /// - Channel is private/presence AND authorizer returns true
    ///
    /// Returns false if:
    /// - Channel is private/presence AND no authorizer registered
    /// - Channel is private/presence AND authorizer denies access
    ///
    /// Used by the broadcasting auth HTTP endpoint to validate authorization
    /// without subscribing the client.
    pub async fn check_auth(&self, auth_data: &AuthData) -> bool {
        let channel_type = ChannelType::from_name(&auth_data.channel);
        if !channel_type.requires_auth() {
            return true;
        }
        if let Some(authorizer) = &self.inner.authorizer {
            authorizer.authorize(auth_data).await
        } else {
            false
        }
    }

    /// Get channel info.
    pub fn get_channel(&self, name: &str) -> Option<ChannelInfo> {
        self.inner.channels.get(name).map(|c| c.clone())
    }

    /// Get number of connected clients.
    pub fn client_count(&self) -> usize {
        self.inner.clients.len()
    }

    /// Get number of active channels.
    pub fn channel_count(&self) -> usize {
        self.inner.channels.len()
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// Authorization data for private/presence channels.
#[derive(Debug, Clone)]
pub struct AuthData {
    /// The socket ID requesting access.
    pub socket_id: String,
    /// The channel name.
    pub channel: String,
    /// Optional auth token from the client.
    pub auth_token: Option<String>,
}

/// Trait for authorizing channel access.
#[async_trait::async_trait]
pub trait ChannelAuthorizer: Send + Sync {
    /// Check if access should be granted.
    async fn authorize(&self, data: &AuthData) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcaster_basic() {
        let broadcaster = Broadcaster::new();
        let (tx, _rx) = mpsc::channel(32);

        broadcaster.add_client("socket_1".into(), tx);
        assert_eq!(broadcaster.client_count(), 1);

        broadcaster.remove_client("socket_1");
        assert_eq!(broadcaster.client_count(), 0);
    }

    #[tokio::test]
    async fn test_subscribe_public_channel() {
        let broadcaster = Broadcaster::new();
        let (tx, _rx) = mpsc::channel(32);

        broadcaster.add_client("socket_1".into(), tx);
        broadcaster
            .subscribe("socket_1", "orders", None, None)
            .await
            .unwrap();

        assert_eq!(broadcaster.channel_count(), 1);
        let channel = broadcaster.get_channel("orders").unwrap();
        assert_eq!(channel.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn test_subscribe_private_requires_auth() {
        let broadcaster = Broadcaster::new();
        let (tx, _rx) = mpsc::channel(32);

        broadcaster.add_client("socket_1".into(), tx);
        let result = broadcaster
            .subscribe("socket_1", "private-orders.1", None, None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_whisper_forwards_to_others() {
        let broadcaster = Broadcaster::new();

        let (tx1, mut rx1) = mpsc::channel(32);
        let (tx2, mut rx2) = mpsc::channel(32);

        broadcaster.add_client("socket_1".into(), tx1);
        broadcaster.add_client("socket_2".into(), tx2);

        // Subscribe both to public channel
        broadcaster
            .subscribe("socket_1", "chat", None, None)
            .await
            .unwrap();
        broadcaster
            .subscribe("socket_2", "chat", None, None)
            .await
            .unwrap();

        // Client 1 whispers
        broadcaster
            .whisper(
                "socket_1",
                "chat",
                "typing",
                serde_json::json!({"user": "alice"}),
            )
            .await
            .unwrap();

        // Client 2 receives the whisper
        let msg = rx2.try_recv().unwrap();
        match msg {
            ServerMessage::Event(broadcast_msg) => {
                assert_eq!(broadcast_msg.event, "typing");
                assert_eq!(broadcast_msg.channel, "chat");
                assert_eq!(broadcast_msg.data, serde_json::json!({"user": "alice"}));
            }
            other => panic!("Expected Event, got {:?}", other),
        }

        // Client 1 does NOT receive it
        assert!(rx1.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_whisper_rejected_when_disabled() {
        let config = BroadcastConfig::new().allow_client_events(false);
        let broadcaster = Broadcaster::with_config(config);

        let (tx, _rx) = mpsc::channel(32);
        broadcaster.add_client("socket_1".into(), tx);
        broadcaster
            .subscribe("socket_1", "chat", None, None)
            .await
            .unwrap();

        let result = broadcaster
            .whisper(
                "socket_1",
                "chat",
                "typing",
                serde_json::json!({}),
            )
            .await;

        assert!(result.is_err());
    }

    struct MockAuthorizer {
        allowed_channels: Vec<String>,
    }

    #[async_trait::async_trait]
    impl ChannelAuthorizer for MockAuthorizer {
        async fn authorize(&self, data: &AuthData) -> bool {
            self.allowed_channels.contains(&data.channel)
        }
    }

    #[tokio::test]
    async fn test_check_auth_public_channel_always_authorized() {
        let broadcaster = Broadcaster::new();
        let auth_data = AuthData {
            socket_id: "socket_1".to_string(),
            channel: "orders".to_string(),
            auth_token: None,
        };
        assert!(broadcaster.check_auth(&auth_data).await);
    }

    #[tokio::test]
    async fn test_check_auth_public_channel_authorized_without_authorizer() {
        // Even with no authorizer, public channels pass
        let broadcaster = Broadcaster::new();
        let auth_data = AuthData {
            socket_id: "socket_1".to_string(),
            channel: "chat".to_string(),
            auth_token: Some("user_42".to_string()),
        };
        assert!(broadcaster.check_auth(&auth_data).await);
    }

    #[tokio::test]
    async fn test_check_auth_private_channel_denied_without_authorizer() {
        let broadcaster = Broadcaster::new();
        let auth_data = AuthData {
            socket_id: "socket_1".to_string(),
            channel: "private-orders".to_string(),
            auth_token: Some("user_42".to_string()),
        };
        assert!(!broadcaster.check_auth(&auth_data).await);
    }

    #[tokio::test]
    async fn test_check_auth_private_channel_allowed_by_authorizer() {
        let authorizer = MockAuthorizer {
            allowed_channels: vec!["private-orders".to_string()],
        };
        let broadcaster = Broadcaster::new().with_authorizer(authorizer);
        let auth_data = AuthData {
            socket_id: "socket_1".to_string(),
            channel: "private-orders".to_string(),
            auth_token: Some("user_42".to_string()),
        };
        assert!(broadcaster.check_auth(&auth_data).await);
    }

    #[tokio::test]
    async fn test_check_auth_private_channel_denied_by_authorizer() {
        let authorizer = MockAuthorizer {
            allowed_channels: vec!["private-orders".to_string()],
        };
        let broadcaster = Broadcaster::new().with_authorizer(authorizer);
        let auth_data = AuthData {
            socket_id: "socket_1".to_string(),
            channel: "private-admin".to_string(),
            auth_token: Some("user_42".to_string()),
        };
        assert!(!broadcaster.check_auth(&auth_data).await);
    }

    #[tokio::test]
    async fn test_check_auth_presence_channel_denied_without_authorizer() {
        let broadcaster = Broadcaster::new();
        let auth_data = AuthData {
            socket_id: "socket_1".to_string(),
            channel: "presence-chat".to_string(),
            auth_token: Some("user_42".to_string()),
        };
        assert!(!broadcaster.check_auth(&auth_data).await);
    }

    #[tokio::test]
    async fn test_check_auth_presence_channel_allowed_by_authorizer() {
        let authorizer = MockAuthorizer {
            allowed_channels: vec!["presence-chat".to_string()],
        };
        let broadcaster = Broadcaster::new().with_authorizer(authorizer);
        let auth_data = AuthData {
            socket_id: "socket_1".to_string(),
            channel: "presence-chat".to_string(),
            auth_token: Some("user_42".to_string()),
        };
        assert!(broadcaster.check_auth(&auth_data).await);
    }

    #[tokio::test]
    async fn test_whisper_rejected_when_not_subscribed() {
        let broadcaster = Broadcaster::new();

        let (tx1, _rx1) = mpsc::channel(32);
        let (tx2, _rx2) = mpsc::channel(32);

        broadcaster.add_client("socket_1".into(), tx1);
        broadcaster.add_client("socket_2".into(), tx2);

        // Only socket_2 subscribes
        broadcaster
            .subscribe("socket_2", "chat", None, None)
            .await
            .unwrap();

        // socket_1 tries to whisper without subscribing
        let result = broadcaster
            .whisper(
                "socket_1",
                "chat",
                "typing",
                serde_json::json!({}),
            )
            .await;

        assert!(result.is_err());
    }
}
