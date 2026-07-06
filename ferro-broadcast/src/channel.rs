//! Channel types and management.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

/// Channel type based on prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    /// Public channel - anyone can subscribe.
    Public,
    /// Private channel - requires authorization.
    Private,
    /// Presence channel - tracks online users.
    Presence,
}

impl ChannelType {
    /// Determine channel type from channel name.
    pub fn from_name(name: &str) -> Self {
        if name.starts_with("private-") {
            Self::Private
        } else if name.starts_with("presence-") {
            Self::Presence
        } else {
            Self::Public
        }
    }

    /// Check if this channel type requires authorization.
    pub fn requires_auth(&self) -> bool {
        matches!(self, Self::Private | Self::Presence)
    }
}

/// Information about a channel.
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    /// Channel name.
    pub name: String,
    /// Channel type.
    pub channel_type: ChannelType,
    /// Connected client socket IDs.
    pub subscribers: HashSet<String>,
    /// Presence members (for presence channels).
    pub members: Vec<PresenceMember>,
}

impl ChannelInfo {
    /// Create a new channel.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let channel_type = ChannelType::from_name(&name);
        Self {
            name,
            channel_type,
            subscribers: HashSet::new(),
            members: Vec::new(),
        }
    }

    /// Check if channel has any subscribers.
    pub fn is_empty(&self) -> bool {
        self.subscribers.is_empty()
    }

    /// Get subscriber count.
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    /// Add a subscriber.
    pub fn add_subscriber(&mut self, socket_id: String) -> bool {
        self.subscribers.insert(socket_id)
    }

    /// Remove a subscriber.
    pub fn remove_subscriber(&mut self, socket_id: &str) -> bool {
        self.subscribers.remove(socket_id)
    }

    /// Add a presence member.
    pub fn add_member(&mut self, member: PresenceMember) {
        // Remove existing member with same user_id if present
        self.members.retain(|m| m.user_id != member.user_id);
        self.members.push(member);
    }

    /// Remove a presence member by socket ID.
    pub fn remove_member(&mut self, socket_id: &str) -> Option<PresenceMember> {
        if let Some(idx) = self.members.iter().position(|m| m.socket_id == socket_id) {
            Some(self.members.remove(idx))
        } else {
            None
        }
    }

    /// Get presence members.
    pub fn get_members(&self) -> &[PresenceMember] {
        &self.members
    }
}

/// A member in a presence channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceMember {
    /// The socket ID.
    pub socket_id: String,
    /// The user ID.
    pub user_id: String,
    /// Additional user information.
    pub user_info: Value,
}

impl PresenceMember {
    /// Create a new presence member.
    pub fn new(socket_id: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self {
            socket_id: socket_id.into(),
            user_id: user_id.into(),
            user_info: Value::Null,
        }
    }

    /// Set user info.
    pub fn with_info(mut self, info: impl Serialize) -> Self {
        self.user_info = serde_json::to_value(info).unwrap_or(Value::Null);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type_from_name() {
        assert_eq!(ChannelType::from_name("orders"), ChannelType::Public);
        assert_eq!(
            ChannelType::from_name("private-orders.1"),
            ChannelType::Private
        );
        assert_eq!(
            ChannelType::from_name("presence-chat.1"),
            ChannelType::Presence
        );
    }

    #[test]
    fn test_channel_requires_auth() {
        assert!(!ChannelType::Public.requires_auth());
        assert!(ChannelType::Private.requires_auth());
        assert!(ChannelType::Presence.requires_auth());
    }

    #[test]
    fn test_channel_info() {
        let mut channel = ChannelInfo::new("private-orders.1");
        assert_eq!(channel.channel_type, ChannelType::Private);
        assert!(channel.is_empty());

        channel.add_subscriber("socket_1".into());
        assert!(!channel.is_empty());
        assert_eq!(channel.subscriber_count(), 1);

        channel.remove_subscriber("socket_1");
        assert!(channel.is_empty());
    }

    #[test]
    fn test_presence_members() {
        let mut channel = ChannelInfo::new("presence-chat.1");
        let member = PresenceMember::new("socket_1", "user_1")
            .with_info(serde_json::json!({"name": "Alice"}));

        channel.add_subscriber("socket_1".into());
        channel.add_member(member);

        assert_eq!(channel.get_members().len(), 1);
        assert_eq!(channel.get_members()[0].user_id, "user_1");
    }
}
