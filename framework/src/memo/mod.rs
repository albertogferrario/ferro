//! Request-scoped memoization store.
//!
//! A [`MemoStore`] is created fresh per HTTP request and held in the
//! `MEMO_STORE` task-local. Any async function annotated with
//! `#[memoize]` reads the ambient store via [`current_memo_store()`];
//! outside a request context the function runs normally with no caching
//! (graceful no-op, D-02).

use futures::future::{BoxFuture, FutureExt, Shared};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ── Task-local ──────────────────────────────────────────────────────────────

tokio::task_local! {
    /// Task-local slot that holds the per-request memo store.
    ///
    /// Entered once per request in `server.rs`; absent in background jobs,
    /// queue workers, and tests that do not explicitly scope it.
    pub(crate) static MEMO_STORE: Arc<MemoStore>;
}

// ── Public types ─────────────────────────────────────────────────────────────

/// Type-erased awaitable slot stored in a [`MemoStore`] entry.
///
/// A `Shared` future whose output is an `Arc<dyn Any + Send + Sync>`.
/// Multiple concurrent callers can await the same slot; the wrapped future
/// runs exactly once.
pub type MemoSlot = Shared<BoxFuture<'static, Arc<dyn Any + Send + Sync>>>;

/// Key for a memoized call: callsite identity plus a hash of the arguments.
///
/// `callsite` is the [`std::any::TypeId`] of a per-macro-expansion
/// zero-sized marker type, ensuring distinct call sites never share slots
/// even when argument hashes collide.
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct MemoKey {
    callsite: std::any::TypeId,
    args_hash: u64,
}

impl MemoKey {
    /// Construct a key for the given callsite marker and hashable arguments.
    ///
    /// `Marker` is a zero-sized type minted by `#[memoize]` at each
    /// expansion site. `A` is the tuple of non-receiver arguments; it must
    /// implement [`std::hash::Hash`].
    pub fn new<Marker: 'static, A: std::hash::Hash>(args: &A) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        let mut h = DefaultHasher::new();
        args.hash(&mut h);
        Self {
            callsite: std::any::TypeId::of::<Marker>(),
            args_hash: h.finish(),
        }
    }
}

/// Per-request memoization store.
///
/// Holds a map from [`MemoKey`] to a [`MemoSlot`] (a shared awaitable
/// future). The first caller for a key inserts a pending future via
/// [`get_or_insert`](MemoStore::get_or_insert); subsequent callers —
/// including concurrent ones — receive a clone of the same slot and await
/// the same underlying computation.
///
/// Created fresh at the start of each HTTP request and dropped when the
/// request's task-local scope exits. No entries survive across requests.
pub struct MemoStore {
    entries: Mutex<HashMap<MemoKey, MemoSlot>>,
}

impl MemoStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Return the cached slot for `key`, inserting one built by `make_fut`
    /// if none exists.
    ///
    /// The mutex guard is released **before** the slot is returned to the
    /// caller, so the caller can `.await` the slot without holding any lock.
    /// `make_fut` is called at most once per key per store lifetime.
    pub fn get_or_insert(
        &self,
        key: MemoKey,
        make_fut: impl FnOnce() -> BoxFuture<'static, Arc<dyn Any + Send + Sync>>,
    ) -> MemoSlot {
        let slot = {
            let mut map = self.entries.lock().unwrap();
            map.entry(key)
                .or_insert_with(|| make_fut().shared())
                .clone()
        }; // MutexGuard dropped here — safe to .await outside the lock
        slot
    }
}

impl Default for MemoStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── Scope helpers ─────────────────────────────────────────────────────────────

/// Return the current request's memo store, if inside a request context.
///
/// Returns `None` outside a request scope (background jobs, queue workers,
/// tests that do not enter a `MEMO_STORE.scope`). Never panics.
pub fn current_memo_store() -> Option<Arc<MemoStore>> {
    MEMO_STORE.try_with(|s| s.clone()).ok()
}

/// Create a fresh `Arc<MemoStore>` for a new request scope.
// Used by server.rs to enter the per-request scope.
#[allow(dead_code)]
pub(crate) fn memo_scope() -> Arc<MemoStore> {
    Arc::new(MemoStore::new())
}

/// Run `f` within a `MEMO_STORE` scope backed by `store`.
// Used by server.rs to enter the per-request scope.
#[allow(dead_code)]
pub(crate) async fn with_memo_scope<F, R>(store: Arc<MemoStore>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    MEMO_STORE.scope(store, f).await
}

// ── Macro-level tests (in-crate, uses #[memoize] via crate::memoize) ─────────

#[cfg(test)]
mod macro_tests;

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ── hit: same key, body runs once ─────────────────────────────────────

    #[tokio::test]
    async fn hit_body_runs_once_for_same_key() {
        let counter = Arc::new(AtomicUsize::new(0));
        let store = Arc::new(MemoStore::new());

        // Unique marker for this call site.
        struct Marker;
        let key = MemoKey::new::<Marker, _>(&42u32);

        let c1 = counter.clone();
        let slot1 = store.get_or_insert(key, move || {
            Box::pin(async move {
                c1.fetch_add(1, Ordering::SeqCst);
                Arc::new(99u32) as Arc<dyn Any + Send + Sync>
            })
        });

        let c2 = counter.clone();
        let slot2 = store.get_or_insert(key, move || {
            // This closure must NOT be called because the key already exists.
            Box::pin(async move {
                c2.fetch_add(100, Ordering::SeqCst);
                Arc::new(0u32) as Arc<dyn Any + Send + Sync>
            })
        });

        let a1 = slot1.await;
        let a2 = slot2.await;

        assert_eq!(*a1.downcast_ref::<u32>().unwrap(), 99);
        assert_eq!(*a2.downcast_ref::<u32>().unwrap(), 99);
        // Body ran exactly once.
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    // ── miss: distinct keys each run their own body ────────────────────────

    #[tokio::test]
    async fn miss_distinct_keys_each_run_body() {
        let counter = Arc::new(AtomicUsize::new(0));
        let store = Arc::new(MemoStore::new());

        struct MarkerA;
        struct MarkerB;
        let key_a = MemoKey::new::<MarkerA, _>(&1u32);
        let key_b = MemoKey::new::<MarkerB, _>(&1u32); // same arg, different callsite

        let ca = counter.clone();
        let sa = store.get_or_insert(key_a, move || {
            Box::pin(async move {
                ca.fetch_add(1, Ordering::SeqCst);
                Arc::new(10u32) as Arc<dyn Any + Send + Sync>
            })
        });

        let cb = counter.clone();
        let sb = store.get_or_insert(key_b, move || {
            Box::pin(async move {
                cb.fetch_add(1, Ordering::SeqCst);
                Arc::new(20u32) as Arc<dyn Any + Send + Sync>
            })
        });

        let va = sa.await;
        let vb = sb.await;

        assert_eq!(*va.downcast_ref::<u32>().unwrap(), 10);
        assert_eq!(*vb.downcast_ref::<u32>().unwrap(), 20);
        // Both bodies ran.
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    // ── coalesce: concurrent awaits on the same key run the body once ──────

    #[tokio::test]
    async fn coalesce_concurrent_callers_run_body_once() {
        let counter = Arc::new(AtomicUsize::new(0));
        let store = Arc::new(MemoStore::new());

        struct Marker;
        let key = MemoKey::new::<Marker, _>(&7u32);

        let c1 = counter.clone();
        let slot1 = store.get_or_insert(key, move || {
            Box::pin(async move {
                c1.fetch_add(1, Ordering::SeqCst);
                Arc::new(42u32) as Arc<dyn Any + Send + Sync>
            })
        });

        let slot2 = store.get_or_insert(key, || {
            // Must not be called — slot already exists.
            Box::pin(async move { Arc::new(0u32) as Arc<dyn Any + Send + Sync> })
        });

        // Drive both concurrently.
        let (r1, r2) = tokio::join!(slot1, slot2);

        assert_eq!(*r1.downcast_ref::<u32>().unwrap(), 42);
        assert_eq!(*r2.downcast_ref::<u32>().unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    // ── out-of-scope: no panic, returns None ─────────────────────────────

    #[test]
    fn out_of_scope_returns_none_without_panic() {
        // No MEMO_STORE scope active — must not panic.
        let result = current_memo_store();
        assert!(result.is_none());
    }

    // ── Err cached: Result-returning future stores the Err for second caller

    #[tokio::test]
    async fn err_cached_result_returning_future() {
        let counter = Arc::new(AtomicUsize::new(0));
        let store = Arc::new(MemoStore::new());

        struct Marker;
        let key = MemoKey::new::<Marker, _>(&0u32);

        let c1 = counter.clone();
        let slot1 = store.get_or_insert(key, move || {
            Box::pin(async move {
                c1.fetch_add(1, Ordering::SeqCst);
                // Store a Result::Err as the full cached value.
                let result: Result<u32, String> = Err("boom".to_string());
                Arc::new(result) as Arc<dyn Any + Send + Sync>
            })
        });

        let slot2 = store.get_or_insert(key, || {
            Box::pin(async move { Arc::new(Ok::<u32, String>(0)) as Arc<dyn Any + Send + Sync> })
        });

        let a1 = slot1.await;
        let a2 = slot2.await;

        let r1 = a1.downcast_ref::<Result<u32, String>>().unwrap();
        let r2 = a2.downcast_ref::<Result<u32, String>>().unwrap();

        assert!(r1.is_err());
        assert_eq!(r1.as_ref().unwrap_err(), "boom");
        assert!(r2.is_err());
        assert_eq!(r2.as_ref().unwrap_err(), "boom");
        // Body ran exactly once.
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    // ── drop: a fresh store has no entries from a prior scope ─────────────

    #[tokio::test]
    async fn dropped_store_has_no_prior_entries() {
        struct Marker;
        let key = MemoKey::new::<Marker, _>(&5u32);

        // First scope — populate an entry.
        let store1 = Arc::new(MemoStore::new());
        {
            let slot = store1.get_or_insert(key, || {
                Box::pin(async move { Arc::new(123u32) as Arc<dyn Any + Send + Sync> })
            });
            let _ = slot.await;
        }
        // Verify it is in store1.
        {
            let map = store1.entries.lock().unwrap();
            assert!(map.contains_key(&key));
        }

        // Second scope — a fresh store must not see the prior entry.
        let store2 = Arc::new(MemoStore::new());
        {
            let map = store2.entries.lock().unwrap();
            assert!(!map.contains_key(&key));
        }
    }

    // ── with_memo_scope helper wires current_memo_store() correctly ────────

    #[tokio::test]
    async fn with_scope_makes_current_memo_store_return_some() {
        let store = memo_scope();
        let result = with_memo_scope(store, async { current_memo_store() }).await;
        assert!(result.is_some());
    }
}
