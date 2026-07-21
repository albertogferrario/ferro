//! End-to-end tests for the `#[memoize]` attribute macro, exercised from the
//! `ferro-rs` crate (the consumer).  A proc-macro crate cannot invoke its own
//! macro in unit tests, so these tests live here where `MEMO_STORE` and
//! `with_memo_scope` are in-crate reachable.
//!
//! Coverage:
//! - SC-1: body runs at most once per (callsite, args); distinct args recompute
//! - SC-2: concurrent callers for the same key coalesce onto one computation
//! - LIVE-01: `#[memoize]` on an impl method with `&self` excludes `self` from the key
//! - D-02: out-of-scope call runs un-memoized without panic
//! - D-04: `Result`-returning fn caches its `Err` for the request

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::memo::{memo_scope, with_memo_scope};
use crate::memoize;

// ── SC-1: hit + miss ──────────────────────────────────────────────────────────

static COUNTER_SC1: AtomicUsize = AtomicUsize::new(0);

#[memoize]
async fn load_sc1(id: u32) -> u32 {
    COUNTER_SC1.fetch_add(1, Ordering::SeqCst);
    id * 2
}

#[tokio::test]
async fn memoize_runs_body_once_same_args() {
    COUNTER_SC1.store(0, Ordering::SeqCst);

    let store = memo_scope();
    let (a, b, c) = with_memo_scope(store, async {
        let a = load_sc1(1).await;
        let b = load_sc1(1).await; // same id → hits cache
        let c = load_sc1(2).await; // different id → recomputes
        (a, b, c)
    })
    .await;

    assert_eq!(a, 2);
    assert_eq!(b, 2); // same result as a
    assert_eq!(c, 4);
    // Body ran twice: once for id=1, once for id=2.
    assert_eq!(COUNTER_SC1.load(Ordering::SeqCst), 2);
}

// ── SC-2: concurrent callers coalesce ─────────────────────────────────────────

static COUNTER_SC2: AtomicUsize = AtomicUsize::new(0);

#[memoize]
async fn load_sc2(id: u32) -> u32 {
    COUNTER_SC2.fetch_add(1, Ordering::SeqCst);
    id * 3
}

#[tokio::test]
async fn concurrent_callers_coalesce() {
    COUNTER_SC2.store(0, Ordering::SeqCst);

    let store = memo_scope();
    let (r1, r2) = with_memo_scope(store, async { tokio::join!(load_sc2(1), load_sc2(1)) }).await;

    assert_eq!(r1, 3);
    assert_eq!(r2, 3);
    // Body ran exactly once for id=1.
    assert_eq!(COUNTER_SC2.load(Ordering::SeqCst), 1);
}

// ── LIVE-01: #[memoize] on impl method — &self excluded from key ──────────────

static COUNTER_METHOD: AtomicUsize = AtomicUsize::new(0);

struct ProductLoader;

impl ProductLoader {
    #[memoize]
    async fn load(&self, id: u32) -> u32 {
        COUNTER_METHOD.fetch_add(1, Ordering::SeqCst);
        id * 10
    }
}

#[tokio::test]
async fn service_method_memoized() {
    COUNTER_METHOD.store(0, Ordering::SeqCst);

    let loader_a = Arc::new(ProductLoader);
    let loader_b = Arc::new(ProductLoader); // different instance, same id

    let store = memo_scope();
    let (r1, r2) = with_memo_scope(store, async {
        let r1 = loader_a.load(7).await;
        // Second call with same id but DIFFERENT &self instance — &self is
        // excluded from the key, so this must hit the cache.
        let r2 = loader_b.load(7).await;
        (r1, r2)
    })
    .await;

    assert_eq!(r1, 70);
    assert_eq!(r2, 70); // same result
                        // Body ran exactly once — &self exclusion confirmed.
    assert_eq!(COUNTER_METHOD.load(Ordering::SeqCst), 1);
}

// ── D-02: out-of-scope runs un-memoized, no panic ────────────────────────────

static COUNTER_D02: AtomicUsize = AtomicUsize::new(0);

#[memoize]
async fn load_d02(id: u32) -> u32 {
    COUNTER_D02.fetch_add(1, Ordering::SeqCst);
    id + 100
}

#[tokio::test]
async fn out_of_scope_is_noop() {
    COUNTER_D02.store(0, Ordering::SeqCst);

    // No MEMO_STORE scope active — must not panic and must return correct value.
    let r1 = load_d02(5).await;
    let r2 = load_d02(5).await;

    assert_eq!(r1, 105);
    assert_eq!(r2, 105);
    // Body ran twice (no caching without a store).
    assert_eq!(COUNTER_D02.load(Ordering::SeqCst), 2);
}

// ── D-04: Result-returning fn caches its Err ─────────────────────────────────

static COUNTER_D04: AtomicUsize = AtomicUsize::new(0);

#[memoize]
async fn maybe(id: u32) -> Result<u32, String> {
    COUNTER_D04.fetch_add(1, Ordering::SeqCst);
    if id == 0 {
        Err(format!("boom-{id}"))
    } else {
        Ok(id * 2)
    }
}

#[tokio::test]
async fn err_is_cached() {
    COUNTER_D04.store(0, Ordering::SeqCst);

    let store = memo_scope();
    let (r1, r2) = with_memo_scope(store, async {
        let r1 = maybe(0).await;
        let r2 = maybe(0).await; // same key → Err must be served from cache
        (r1, r2)
    })
    .await;

    assert!(r1.is_err());
    assert_eq!(r1.unwrap_err(), "boom-0");
    assert!(r2.is_err());
    assert_eq!(r2.unwrap_err(), "boom-0");
    // Body ran exactly once — Err was cached.
    assert_eq!(COUNTER_D04.load(Ordering::SeqCst), 1);
}
