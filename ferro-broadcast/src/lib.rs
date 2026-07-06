//! # Ferro Broadcast
//!
//! WebSocket broadcasting and real-time channels for the Ferro framework.
//!
//! Provides a Laravel Echo-inspired broadcasting system with support for:
//! - Public channels (anyone can subscribe)
//! - Private channels (require authorization)
//! - Presence channels (track online users)
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_broadcast::{Broadcast, Broadcaster};
//! use std::sync::Arc;
//!
//! // Create a broadcaster
//! let broadcaster = Arc::new(Broadcaster::new());
//!
//! // Broadcast to a channel
//! Broadcast::new(broadcaster.clone())
//!     .channel("orders.1")
//!     .event("OrderUpdated")
//!     .data(&order)
//!     .send()
//!     .await?;
//! ```
//!
//! ## Channel Types
//!
//! Channels are determined by their name prefix:
//! - `orders` - Public channel
//! - `private-orders.1` - Private channel (requires auth)
//! - `presence-chat.1` - Presence channel (tracks members)
//!
//! ## Authorization
//!
//! For private and presence channels, implement the `ChannelAuthorizer` trait:
//!
//! ```rust,ignore
//! use ferro_broadcast::{AuthData, ChannelAuthorizer};
//!
//! struct MyAuthorizer;
//!
//! #[async_trait::async_trait]
//! impl ChannelAuthorizer for MyAuthorizer {
//!     async fn authorize(&self, data: &AuthData) -> bool {
//!         // Check if user can access this channel
//!         verify_access(&data.channel, &data.auth_token)
//!     }
//! }
//! ```

mod broadcast;
mod broadcaster;
mod channel;
mod config;
mod error;
mod message;

pub use broadcast::{Broadcast, BroadcastBuilder};
pub use broadcaster::{AuthData, Broadcaster, ChannelAuthorizer, Client};
pub use channel::{ChannelInfo, ChannelType, PresenceMember};
pub use config::BroadcastConfig;
pub use error::Error;
pub use message::{BroadcastMessage, ClientMessage, ServerMessage};

/// Re-export async_trait for convenience.
pub use async_trait::async_trait;
