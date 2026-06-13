//! # Ferro Cache
//!
//! Caching with tags for the Ferro framework.
//!
//! Provides a unified caching API with support for:
//! - Multiple backends (Redis, in-memory)
//! - Cache tags for bulk invalidation
//! - Remember pattern for lazy caching
//! - TTL (time-to-live) support
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_cache::{Cache, CacheConfig};
//! use std::time::Duration;
//!
//! // Create cache
//! let cache = Cache::memory();
//!
//! // Store a value
//! cache.put("user:1", &user, Duration::from_secs(3600)).await?;
//!
//! // Get a value
//! let user: User = cache.get("user:1").await?;
//!
//! // Remember pattern - get from cache or compute
//! let users = cache.remember("users:active", Duration::from_secs(3600), || async {
//!     User::where_active().all().await
//! }).await?;
//! ```
//!
//! ## Cache Tags
//!
//! Tags allow bulk invalidation of related cache entries:
//!
//! ```rust,ignore
//! // Store with tags
//! cache.tags(&["users", "admins"])
//!     .put("user:1", &admin, Duration::from_secs(3600))
//!     .await?;
//!
//! // Flush all entries with a tag
//! cache.tags(&["users"]).flush().await?;
//! ```

mod cache;
mod error;
mod invalidator;
mod stores;
mod tagged;

pub use cache::{Cache, CacheConfig, CacheStore};
pub use error::Error;
pub use invalidator::{register_invalidator, register_invalidator_on};
pub use tagged::TaggedCache;

#[cfg(feature = "memory")]
pub use stores::MemoryStore;

#[cfg(feature = "redis-backend")]
pub use stores::RedisStore;

/// Re-export for convenience.
pub use async_trait::async_trait;
pub use serde;
