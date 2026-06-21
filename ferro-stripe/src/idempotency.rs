//! Idempotency primitives for Stripe webhook processing.
//!
//! Stripe retries webhooks, so every handler must be idempotent. Deduplicate
//! on the Stripe event id (`evt_xxx`) using a persistent log. The trait
//! below is the contract; the consuming app ships the impl backed by its DB.
//!
//! ## Recommended SQL schema
//!
//! ```sql
//! CREATE TABLE stripe_processed_events (
//!   event_id TEXT PRIMARY KEY,
//!   event_type TEXT NOT NULL,
//!   received_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
//! );
//! ```
//!
//! Ferro does not ship this migration. Applications own the table.
//! The `PRIMARY KEY` on `event_id` is the idempotency fence: the app's
//! `try_mark_processed` impl SHOULD perform a conditional `INSERT` and
//! return `Ok(true)` when a row was inserted, `Ok(false)` when the row
//! already existed (unique-constraint violation).
//!
//! [`MemoryProcessedLog`] is provided for tests and single-process dev
//! scenarios. Production systems MUST implement [`ProcessedEventLog`]
//! against a shared database — in-process state does not survive restarts
//! or protect against horizontal scaling.

use crate::Error;
use async_trait::async_trait;

/// Records Stripe webhook events that have been processed, so that
/// retries from Stripe do not cause duplicate side effects.
///
/// Implementations SHOULD be backed by a durable store with a unique
/// constraint on `event_id`. See the module-level doc for the
/// recommended SQL schema.
#[async_trait]
pub trait ProcessedEventLog: Send + Sync {
    /// Attempts to mark `event_id` as processed.
    ///
    /// Returns `Ok(true)` when this call is the first to record the id
    /// (the handler SHOULD proceed with side effects).
    /// Returns `Ok(false)` when the id was already present
    /// (the handler MUST skip side effects — this is a retry).
    async fn try_mark_processed(&self, event_id: &str) -> Result<bool, Error>;

    /// Removes `event_id` from the processed set so a later delivery is
    /// reprocessed.
    ///
    /// Call this when a handler marked an event processed but then failed
    /// transiently *before* its side effects committed: un-marking lets Stripe's
    /// retry re-run the handler instead of short-circuiting on the idempotency
    /// fast-path (which would otherwise drop the side effect permanently).
    /// Idempotent — un-marking an absent id is a no-op.
    async fn unmark(&self, event_id: &str) -> Result<(), Error>;
}

/// In-memory [`ProcessedEventLog`] backed by [`dashmap::DashMap`].
///
/// Intended for tests and single-process development. State is lost on
/// process restart and is not shared across processes. Use a DB-backed
/// impl in production.
pub struct MemoryProcessedLog {
    seen: dashmap::DashMap<String, ()>,
}

impl MemoryProcessedLog {
    /// Creates an empty in-memory log.
    pub fn new() -> Self {
        Self {
            seen: dashmap::DashMap::new(),
        }
    }
}

impl Default for MemoryProcessedLog {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProcessedEventLog for MemoryProcessedLog {
    async fn try_mark_processed(&self, event_id: &str) -> Result<bool, Error> {
        // DashMap::insert returns None when the key was absent (first time,
        // so we return Ok(true)) and Some(()) when the key was already
        // present (already seen, so Ok(false)). DashMap shard locking
        // makes this atomic per key across concurrent callers.
        Ok(self.seen.insert(event_id.to_string(), ()).is_none())
    }

    async fn unmark(&self, event_id: &str) -> Result<(), Error> {
        // Idempotent: removing an absent key is a no-op.
        self.seen.remove(event_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_log_true_then_false() {
        let log = MemoryProcessedLog::new();
        assert!(
            log.try_mark_processed("evt_001").await.unwrap(),
            "first call with a new id must return Ok(true)"
        );
        assert!(
            !log.try_mark_processed("evt_001").await.unwrap(),
            "second call with the same id must return Ok(false)"
        );
        assert!(
            log.try_mark_processed("evt_002").await.unwrap(),
            "different id must return Ok(true) even after an earlier id was seen"
        );
    }

    #[tokio::test]
    async fn memory_log_concurrent_insert_applies_once() {
        use std::sync::Arc;
        let log = Arc::new(MemoryProcessedLog::new());
        let log2 = Arc::clone(&log);

        let t1 = tokio::spawn(async move { log.try_mark_processed("evt_race_001").await });
        let t2 = tokio::spawn(async move { log2.try_mark_processed("evt_race_001").await });

        let (r1, r2) = tokio::join!(t1, t2);
        let v1 = r1.unwrap().unwrap();
        let v2 = r2.unwrap().unwrap();
        assert_ne!(
            v1,
            v2,
            "concurrent inserts must apply exactly once: one Ok(true), one Ok(false), got ({v1}, {v2})"
        );
    }
}
