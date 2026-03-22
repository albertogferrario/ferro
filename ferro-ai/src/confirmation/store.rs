use super::{ConfirmationExpired, ConfirmationStore, PendingActionInfo};
use crate::error::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use ferro_events::dispatch_sync;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::AbortHandle;
use tokio::time::sleep;

/// Internal storage entry for a pending confirmation action.
struct StoredAction {
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
    abort_handle: AbortHandle,
}

/// In-memory confirmation store backed by a [`DashMap`].
///
/// Each entry carries an [`AbortHandle`] for its TTL expiry task, so confirming
/// or rejecting an action immediately cancels the background timer.
///
/// # Concurrency
///
/// `DashMap` guards are **never** held across `.await` points. In `list_pending`,
/// the entries are collected into a `Vec` while holding the shard locks, and the
/// guard is dropped before returning.
pub struct InMemoryConfirmationStore {
    inner: Arc<DashMap<String, StoredAction>>,
}

impl InMemoryConfirmationStore {
    /// Create a new store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryConfirmationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConfirmationStore for InMemoryConfirmationStore {
    async fn request_confirmation(
        &self,
        key: &str,
        payload: serde_json::Value,
        ttl: Duration,
    ) -> Result<(), Error> {
        // Abort any existing TTL task for this key before inserting the new entry.
        if let Some((_, old)) = self.inner.remove(key) {
            old.abort_handle.abort();
        }

        let store = Arc::clone(&self.inner);
        let key_owned = key.to_string();

        let abort_handle = tokio::spawn(async move {
            sleep(ttl).await;
            if store.remove(&key_owned).is_some() {
                dispatch_sync(ConfirmationExpired {
                    key: key_owned,
                    expired_at: Utc::now(),
                });
            }
        })
        .abort_handle();

        self.inner.insert(
            key.to_string(),
            StoredAction {
                payload,
                created_at: Utc::now(),
                abort_handle,
            },
        );

        Ok(())
    }

    async fn confirm(&self, key: &str) -> Result<Option<serde_json::Value>, Error> {
        match self.inner.remove(key) {
            Some((_, entry)) => {
                entry.abort_handle.abort();
                Ok(Some(entry.payload))
            }
            None => Ok(None),
        }
    }

    async fn reject(&self, key: &str) -> Result<bool, Error> {
        match self.inner.remove(key) {
            Some((_, entry)) => {
                entry.abort_handle.abort();
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, Error> {
        // Clone payload while guard is held, then drop guard before return.
        let payload = self.inner.get(key).map(|entry| entry.payload.clone());
        Ok(payload)
    }

    async fn list_pending(&self) -> Result<Vec<PendingActionInfo>, Error> {
        // Collect into Vec while holding shard locks, drop all guards before returning.
        let pending: Vec<PendingActionInfo> = self
            .inner
            .iter()
            .map(|entry| PendingActionInfo {
                key: entry.key().clone(),
                created_at: entry.value().created_at,
            })
            .collect();
        Ok(pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> InMemoryConfirmationStore {
        InMemoryConfirmationStore::new()
    }

    fn payload(val: &str) -> serde_json::Value {
        serde_json::json!({"action": val})
    }

    // --- Basic CRUD ---

    #[tokio::test]
    async fn test_request_and_get() {
        let s = store();
        let p = payload("delete");
        s.request_confirmation("k1", p.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        let result = s.get("k1").await.unwrap();
        assert_eq!(result, Some(p));
    }

    #[tokio::test]
    async fn test_confirm_returns_payload_and_removes_entry() {
        let s = store();
        let p = payload("publish");
        s.request_confirmation("k2", p.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        let confirmed = s.confirm("k2").await.unwrap();
        assert_eq!(confirmed, Some(p));
        // Entry must be gone after confirm
        assert_eq!(s.get("k2").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_confirm_nonexistent_returns_none() {
        let s = store();
        let result = s.confirm("no-such-key").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_reject_removes_entry_and_returns_true() {
        let s = store();
        s.request_confirmation("k3", payload("send"), Duration::from_secs(60))
            .await
            .unwrap();
        let rejected = s.reject("k3").await.unwrap();
        assert!(rejected);
        assert_eq!(s.get("k3").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_reject_nonexistent_returns_false() {
        let s = store();
        let result = s.reject("ghost").await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_list_pending_returns_all_entries() {
        let s = store();
        s.request_confirmation("a", payload("alpha"), Duration::from_secs(60))
            .await
            .unwrap();
        s.request_confirmation("b", payload("beta"), Duration::from_secs(60))
            .await
            .unwrap();

        let mut pending = s.list_pending().await.unwrap();
        pending.sort_by_key(|p| p.key.clone());
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].key, "a");
        assert_eq!(pending[1].key, "b");
    }

    #[tokio::test]
    async fn test_overwrite_replaces_payload() {
        let s = store();
        s.request_confirmation("k4", payload("first"), Duration::from_secs(60))
            .await
            .unwrap();
        s.request_confirmation("k4", payload("second"), Duration::from_secs(60))
            .await
            .unwrap();
        let result = s.get("k4").await.unwrap();
        assert_eq!(result, Some(payload("second")));
        // Still only one pending entry
        assert_eq!(s.list_pending().await.unwrap().len(), 1);
    }

    // --- TTL expiry ---

    /// Yield to the tokio scheduler so spawned tasks can register timers before a time advance.
    ///
    /// Spawned tasks (e.g. TTL expiry tasks) are not polled until the current task yields.
    /// Calling this after `request_confirmation` lets the spawned task run until it
    /// registers its `sleep` timer with the runtime. After `tokio::time::advance`, the
    /// timer fires and further yields allow the task to complete.
    async fn yield_to_register_timer() {
        tokio::task::yield_now().await;
    }

    /// Yield after a time advance so woken tasks can complete their post-timer work.
    async fn yield_after_advance() {
        for _ in 0..5 {
            tokio::task::yield_now().await;
        }
    }

    #[tokio::test(start_paused = true)]
    async fn test_entry_removed_after_ttl_expires() {
        let s = store();
        s.request_confirmation("ttl-key", payload("expire-me"), Duration::from_millis(100))
            .await
            .unwrap();
        // Yield so the spawned TTL task registers its sleep timer.
        yield_to_register_timer().await;
        // Not expired yet
        assert!(s.get("ttl-key").await.unwrap().is_some());

        tokio::time::advance(Duration::from_millis(150)).await;
        yield_after_advance().await;

        assert_eq!(s.get("ttl-key").await.unwrap(), None);
    }

    #[tokio::test(start_paused = true)]
    async fn test_confirm_before_ttl_prevents_expiry() {
        let s = store();
        s.request_confirmation(
            "confirm-early",
            payload("confirm"),
            Duration::from_millis(100),
        )
        .await
        .unwrap();
        yield_to_register_timer().await;
        let result = s.confirm("confirm-early").await.unwrap();
        assert!(result.is_some());

        // Advance past TTL — the task should have been aborted, no panic
        tokio::time::advance(Duration::from_millis(200)).await;
        yield_after_advance().await;

        // Key should remain absent (was removed by confirm, not by expiry)
        assert_eq!(s.get("confirm-early").await.unwrap(), None);
    }

    #[tokio::test(start_paused = true)]
    async fn test_reject_before_ttl_prevents_expiry() {
        let s = store();
        s.request_confirmation(
            "reject-early",
            payload("reject"),
            Duration::from_millis(100),
        )
        .await
        .unwrap();
        yield_to_register_timer().await;
        let rejected = s.reject("reject-early").await.unwrap();
        assert!(rejected);

        tokio::time::advance(Duration::from_millis(200)).await;
        yield_after_advance().await;

        assert_eq!(s.get("reject-early").await.unwrap(), None);
    }

    #[tokio::test(start_paused = true)]
    async fn test_independent_ttls() {
        let s = store();
        s.request_confirmation("short", payload("s"), Duration::from_millis(100))
            .await
            .unwrap();
        s.request_confirmation("long", payload("l"), Duration::from_millis(200))
            .await
            .unwrap();
        // Let both spawned tasks register their timers.
        yield_to_register_timer().await;
        yield_to_register_timer().await;

        // Advance past short TTL only
        tokio::time::advance(Duration::from_millis(150)).await;
        yield_after_advance().await;

        assert_eq!(
            s.get("short").await.unwrap(),
            None,
            "short should be expired"
        );
        assert!(
            s.get("long").await.unwrap().is_some(),
            "long should still be alive"
        );

        // Advance past long TTL too
        tokio::time::advance(Duration::from_millis(100)).await;
        yield_after_advance().await;

        assert_eq!(
            s.get("long").await.unwrap(),
            None,
            "long should now be expired"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn test_overwrite_cancels_old_ttl_timer() {
        let s = store();
        // Register with a short TTL
        s.request_confirmation("over-key", payload("original"), Duration::from_millis(100))
            .await
            .unwrap();
        // Yield to let the original task register its timer.
        yield_to_register_timer().await;
        // Overwrite with a longer TTL immediately — aborts the old timer.
        s.request_confirmation(
            "over-key",
            payload("replacement"),
            Duration::from_millis(300),
        )
        .await
        .unwrap();
        // Yield to let the replacement task register its timer.
        yield_to_register_timer().await;

        // Advance past the original (short) TTL — the old timer should be dead.
        tokio::time::advance(Duration::from_millis(150)).await;
        yield_after_advance().await;

        // The new payload should still be there (old timer was aborted).
        assert_eq!(
            s.get("over-key").await.unwrap(),
            Some(payload("replacement")),
            "overwritten entry should still be alive under new TTL"
        );

        // Advance past the new TTL.
        tokio::time::advance(Duration::from_millis(200)).await;
        yield_after_advance().await;

        assert_eq!(
            s.get("over-key").await.unwrap(),
            None,
            "new TTL should now expire"
        );
    }
}
