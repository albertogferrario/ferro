pub mod events;
pub mod store;

use crate::error::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::time::Duration;

pub use events::ConfirmationExpired;
pub use store::InMemoryConfirmationStore;

/// Metadata about a pending action waiting for confirmation.
#[derive(Debug, Clone, Serialize)]
pub struct PendingActionInfo {
    /// The confirmation key.
    pub key: String,
    /// When the action was first queued for confirmation.
    pub created_at: DateTime<Utc>,
}

/// Store for managing pending confirmation actions.
///
/// Stores type-erased [`serde_json::Value`] payloads keyed by a string.
/// Each entry has a TTL; when it expires, a [`ConfirmationExpired`] event is
/// dispatched via [`ferro_events::dispatch_sync`].
///
/// # Example
///
/// ```rust,ignore
/// use ferro_ai::{InMemoryConfirmationStore, ConfirmationStore};
/// use std::time::Duration;
///
/// let store = InMemoryConfirmationStore::new(Duration::from_secs(60));
/// let payload = serde_json::json!({"action": "delete_user", "user_id": 42});
///
/// store.request_confirmation("confirm-delete-42", payload, Duration::from_secs(60)).await?;
/// let confirmed = store.confirm("confirm-delete-42").await?;
/// // confirmed = Some(serde_json::Value)
/// ```
#[async_trait]
pub trait ConfirmationStore: Send + Sync {
    /// Store a payload under the given key, replacing any existing entry.
    ///
    /// Starts a TTL timer. When the timer expires, the entry is removed and
    /// a [`ConfirmationExpired`] event is dispatched.
    async fn request_confirmation(
        &self,
        key: &str,
        payload: serde_json::Value,
        ttl: Duration,
    ) -> Result<(), Error>;

    /// Confirm the action identified by `key`.
    ///
    /// Removes the entry and aborts its TTL timer.
    /// Returns the stored payload, or `None` if the key does not exist.
    async fn confirm(&self, key: &str) -> Result<Option<serde_json::Value>, Error>;

    /// Reject the action identified by `key`.
    ///
    /// Removes the entry and aborts its TTL timer without dispatching an
    /// expiry event.
    ///
    /// Returns `true` if the entry existed, `false` if it was not found.
    async fn reject(&self, key: &str) -> Result<bool, Error>;

    /// Retrieve the stored payload without removing the entry.
    ///
    /// Returns `None` if the key does not exist or has already expired.
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, Error>;

    /// List all pending (not yet confirmed, rejected, or expired) entries.
    async fn list_pending(&self) -> Result<Vec<PendingActionInfo>, Error>;
}
