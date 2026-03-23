use crate::Error;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::AbortHandle;
use tokio::time::sleep;

/// TTL for deduplication entries. Covers all Meta retry windows.
const DEDUP_TTL: Duration = Duration::from_secs(300);

/// Pluggable store for wamid-based message deduplication.
///
/// Implementations must be thread-safe. The `check_and_insert` method is
/// the single entry point: it returns `true` if the wamid was already seen
/// (duplicate) and `false` if it is new.
#[async_trait]
pub trait DeduplicationStore: Send + Sync {
    /// Checks whether `wamid` has been seen before and records it if not.
    ///
    /// Returns `true` if the wamid is a duplicate (already seen).
    /// Returns `false` if it is new (first time seen).
    async fn check_and_insert(&self, wamid: &str) -> Result<bool, Error>;
}

/// In-memory deduplication store backed by a [`DashMap`].
///
/// Each entry carries an [`AbortHandle`] for its 5-minute TTL expiry task.
/// DashMap guards are **never** held across `.await` points.
pub struct InMemoryDeduplicationStore {
    inner: Arc<DashMap<String, AbortHandle>>,
}

impl InMemoryDeduplicationStore {
    /// Create a new store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryDeduplicationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DeduplicationStore for InMemoryDeduplicationStore {
    async fn check_and_insert(&self, wamid: &str) -> Result<bool, Error> {
        // Check existence without holding guard across .await
        if self.inner.contains_key(wamid) {
            tracing::debug!("duplicate wamid: {wamid}");
            return Ok(true);
        }

        // Not a duplicate — insert with TTL timer
        let store = Arc::clone(&self.inner);
        let key_owned = wamid.to_string();

        let abort_handle = tokio::spawn(async move {
            sleep(DEDUP_TTL).await;
            store.remove(&key_owned);
        })
        .abort_handle();

        self.inner.insert(wamid.to_string(), abort_handle);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dedup_first_insert() {
        let store = InMemoryDeduplicationStore::new();
        let result = store.check_and_insert("wamid.123").await.unwrap();
        assert!(!result, "first insert must return false (not duplicate)");
    }

    #[tokio::test]
    async fn dedup_duplicate() {
        let store = InMemoryDeduplicationStore::new();
        store.check_and_insert("wamid.abc").await.unwrap();
        let result = store.check_and_insert("wamid.abc").await.unwrap();
        assert!(
            result,
            "second insert of same wamid must return true (duplicate)"
        );
    }

    #[tokio::test]
    async fn dedup_different_wamids() {
        let store = InMemoryDeduplicationStore::new();
        let r1 = store.check_and_insert("wamid.001").await.unwrap();
        let r2 = store.check_and_insert("wamid.002").await.unwrap();
        assert!(!r1, "first wamid must not be duplicate");
        assert!(!r2, "second different wamid must not be duplicate");
    }

    #[tokio::test(start_paused = true)]
    async fn dedup_ttl_expiry() {
        let store = InMemoryDeduplicationStore::new();
        store.check_and_insert("wamid.ttl").await.unwrap();

        // Yield so the spawned TTL task registers its sleep timer
        tokio::task::yield_now().await;

        // Not expired yet
        let is_dup = store.check_and_insert("wamid.ttl").await.unwrap();
        assert!(is_dup, "should still be a duplicate before TTL");

        // Remove the second insert's timer too (it also spawned one)
        tokio::task::yield_now().await;

        // Advance past the TTL
        tokio::time::advance(Duration::from_secs(301)).await;

        // Allow spawned tasks to complete
        for _ in 0..5 {
            tokio::task::yield_now().await;
        }

        // Should no longer be a duplicate
        let after_expiry = store.check_and_insert("wamid.ttl").await.unwrap();
        assert!(
            !after_expiry,
            "after TTL expiry must return false (not duplicate)"
        );
    }
}
