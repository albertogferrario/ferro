//! Event-driven cache invalidation.
//!
//! Bridges [`ferro_events`] and the tagged-cache surface so a consumer can
//! declare once at boot — "when event `E` fires, flush these tags" — instead
//! of writing per-app `impl Listener<E>` glue that knows about the cache.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_cache::{Cache, register_invalidator};
//! use ferro_events::Event;
//! use std::sync::Arc;
//!
//! #[derive(Clone)]
//! struct BookingCreated {
//!     business_id: i64,
//!     product_id: i64,
//! }
//!
//! impl Event for BookingCreated {
//!     fn name(&self) -> &'static str { "BookingCreated" }
//! }
//!
//! # async fn wire_up() {
//! let cache = Arc::new(Cache::memory());
//!
//! // One line at boot — every BookingCreated dispatch flushes the
//! // matching business:N:product:M tag.
//! register_invalidator::<BookingCreated, _>(
//!     cache.clone(),
//!     |e| vec![format!("business:{}:product:{}", e.business_id, e.product_id)],
//! );
//!
//! // Later, somewhere else in the app:
//! // BookingCreated { business_id: 1, product_id: 7 }.dispatch().await?;
//! //   → ferro-events delivers the event
//! //   → this invalidator runs
//! //   → cache.tags(&["business:1:product:7"]).flush() runs
//! //   → the next read recomputes
//! # }
//! ```
//!
//! # Failure semantics
//!
//! Listener failures (cache store unavailable, serialization mismatch, …) are
//! logged via `tracing::warn!` and swallowed: the original
//! [`ferro_events::EventDispatcher::dispatch`] call **does not** propagate the
//! error. A degraded cache must not brick the write path that fired the event.

use crate::cache::Cache;
use ferro_events::{global_dispatcher, Event, EventDispatcher};
use std::sync::Arc;

/// Register a cache-invalidation listener on an arbitrary
/// [`EventDispatcher`].
///
/// When an event of type `E` is dispatched through `dispatcher`, `key_fn`
/// is invoked with the event to compute the set of tags to flush. Each
/// tag is flushed via [`Cache::tags`] + [`crate::TaggedCache::flush`];
/// per-tag flush failures are `tracing::warn!`'d and swallowed at the
/// dispatcher boundary so a degraded cache cannot brick the write path
/// that fired the event.
///
/// Multiple invalidators may be registered for the same event type on the
/// same dispatcher — all run; order between them is unspecified.
///
/// Prefer [`register_invalidator`] for the common case of registering on
/// the process-wide [`global_dispatcher`]. Use this overload when the
/// consumer holds a non-global dispatcher (isolated per-tenant context,
/// per-test fixture, embedded library inside a larger app, …).
///
/// # Parameters
///
/// - `dispatcher`: the dispatcher to register the listener on.
/// - `cache`: an `Arc<Cache>` whose store backs the tag flushing.
/// - `key_fn`: a closure `Fn(&E) -> Vec<String>` returning the tags to
///   flush. Returning an empty `Vec` is a no-op (and skips the per-tag
///   flush calls).
pub fn register_invalidator_on<E, F>(dispatcher: &EventDispatcher, cache: Arc<Cache>, key_fn: F)
where
    E: Event,
    F: Fn(&E) -> Vec<String> + Send + Sync + 'static,
{
    let key_fn = Arc::new(key_fn);
    dispatcher.on::<E, _, _>(move |event: E| {
        let cache = cache.clone();
        let key_fn = Arc::clone(&key_fn);
        async move {
            let tags = key_fn(&event);
            for tag in tags {
                if let Err(e) = cache.tags(&[tag.as_str()]).flush().await {
                    tracing::warn!(
                        error = %e,
                        tag = %tag,
                        "ferro-cache invalidator: tag flush failed"
                    );
                }
            }
            // Always succeed at the dispatcher boundary — a failed flush must
            // not propagate back to the write path that fired the event.
            Ok(())
        }
    });
}

/// Register a cache-invalidation listener on the process-wide
/// [`global_dispatcher`].
///
/// Convenience wrapper around [`register_invalidator_on`]; see that
/// function for the full behavioural contract. This is the right entry
/// point for app-boot wiring where events are dispatched via the
/// ergonomic `event.dispatch().await` Laravel-style API.
///
/// # Example
///
/// See the [crate-level documentation](crate) for a complete wiring example.
pub fn register_invalidator<E, F>(cache: Arc<Cache>, key_fn: F)
where
    E: Event,
    F: Fn(&E) -> Vec<String> + Send + Sync + 'static,
{
    register_invalidator_on::<E, F>(global_dispatcher(), cache, key_fn);
}

#[cfg(all(test, feature = "memory"))]
mod tests {
    use super::*;
    use crate::Cache;
    use ferro_events::Event;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    // Each test uses a unique event type so the global dispatcher's
    // per-TypeId listener registry does not bleed state between tests.

    #[derive(Clone)]
    struct EvtFlushSingle {
        product: i64,
    }
    impl Event for EvtFlushSingle {
        fn name(&self) -> &'static str {
            "EvtFlushSingle"
        }
    }

    #[tokio::test]
    async fn flushes_matching_tag() {
        let cache = Arc::new(Cache::memory());

        cache
            .tags(&["business:1:product:7"])
            .put(
                "availability:foo",
                &"slot-grid-blob",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        assert!(
            cache
                .tags(&["business:1:product:7"])
                .has("availability:foo")
                .await
                .unwrap(),
            "precondition: entry exists before invalidator runs"
        );

        register_invalidator::<EvtFlushSingle, _>(cache.clone(), |e| {
            vec![format!("business:1:product:{}", e.product)]
        });

        EvtFlushSingle { product: 7 }.dispatch().await.unwrap();

        assert!(
            !cache
                .tags(&["business:1:product:7"])
                .has("availability:foo")
                .await
                .unwrap(),
            "entry should be evicted after matching event"
        );
    }

    #[derive(Clone)]
    struct EvtFlushNonMatching {
        product: i64,
    }
    impl Event for EvtFlushNonMatching {
        fn name(&self) -> &'static str {
            "EvtFlushNonMatching"
        }
    }

    #[tokio::test]
    async fn does_not_flush_unrelated_tags() {
        let cache = Arc::new(Cache::memory());

        cache
            .tags(&["business:1:product:7"])
            .put("a", &"kept", Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .tags(&["business:1:product:99"])
            .put("b", &"evicted", Duration::from_secs(60))
            .await
            .unwrap();

        register_invalidator::<EvtFlushNonMatching, _>(cache.clone(), |e| {
            vec![format!("business:1:product:{}", e.product)]
        });

        EvtFlushNonMatching { product: 99 }
            .dispatch()
            .await
            .unwrap();

        assert!(
            cache
                .tags(&["business:1:product:7"])
                .has("a")
                .await
                .unwrap(),
            "unrelated tag must survive"
        );
        assert!(
            !cache
                .tags(&["business:1:product:99"])
                .has("b")
                .await
                .unwrap(),
            "matching tag must be evicted"
        );
    }

    #[derive(Clone)]
    struct EvtMultiInvalidator;
    impl Event for EvtMultiInvalidator {
        fn name(&self) -> &'static str {
            "EvtMultiInvalidator"
        }
    }

    #[tokio::test]
    async fn all_registered_invalidators_run() {
        let cache = Arc::new(Cache::memory());

        // Two distinct tags carrying two distinct entries.
        cache
            .tags(&["scope:a"])
            .put("k", &"va", Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .tags(&["scope:b"])
            .put("k", &"vb", Duration::from_secs(60))
            .await
            .unwrap();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_a = Arc::clone(&calls);
        let calls_b = Arc::clone(&calls);

        register_invalidator::<EvtMultiInvalidator, _>(cache.clone(), move |_e| {
            calls_a.fetch_add(1, Ordering::SeqCst);
            vec!["scope:a".to_string()]
        });
        register_invalidator::<EvtMultiInvalidator, _>(cache.clone(), move |_e| {
            calls_b.fetch_add(1, Ordering::SeqCst);
            vec!["scope:b".to_string()]
        });

        EvtMultiInvalidator.dispatch().await.unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 2, "both key_fns should run");
        assert!(!cache.tags(&["scope:a"]).has("k").await.unwrap());
        assert!(!cache.tags(&["scope:b"]).has("k").await.unwrap());
    }

    #[derive(Clone)]
    struct EvtEmptyTags;
    impl Event for EvtEmptyTags {
        fn name(&self) -> &'static str {
            "EvtEmptyTags"
        }
    }

    #[tokio::test]
    async fn empty_tag_set_is_a_noop() {
        let cache = Arc::new(Cache::memory());
        cache
            .tags(&["t"])
            .put("k", &"v", Duration::from_secs(60))
            .await
            .unwrap();

        register_invalidator::<EvtEmptyTags, _>(cache.clone(), |_e| Vec::new());

        EvtEmptyTags.dispatch().await.unwrap();

        assert!(
            cache.tags(&["t"]).has("k").await.unwrap(),
            "empty tag list must not flush anything"
        );
    }

    #[derive(Clone)]
    struct EvtLocalDispatcher {
        product: i64,
    }
    impl Event for EvtLocalDispatcher {
        fn name(&self) -> &'static str {
            "EvtLocalDispatcher"
        }
    }

    #[tokio::test]
    async fn register_invalidator_on_arbitrary_dispatcher() {
        use ferro_events::EventDispatcher;

        // Two isolated dispatchers — only the one we wire the invalidator
        // to should see the flush; the other must be untouched.
        let wired_dispatcher = EventDispatcher::new();
        let untouched_dispatcher = EventDispatcher::new();

        let cache = Arc::new(Cache::memory());
        cache
            .tags(&["business:1:product:7"])
            .put("k", &"v", Duration::from_secs(60))
            .await
            .unwrap();

        register_invalidator_on::<EvtLocalDispatcher, _>(&wired_dispatcher, cache.clone(), |e| {
            vec![format!("business:1:product:{}", e.product)]
        });

        // Dispatching through the OTHER dispatcher must not flush.
        untouched_dispatcher
            .dispatch(EvtLocalDispatcher { product: 7 })
            .await
            .unwrap();
        assert!(
            cache
                .tags(&["business:1:product:7"])
                .has("k")
                .await
                .unwrap(),
            "untouched dispatcher must not trigger the invalidator"
        );

        // Dispatching through the WIRED dispatcher must flush.
        wired_dispatcher
            .dispatch(EvtLocalDispatcher { product: 7 })
            .await
            .unwrap();
        assert!(
            !cache
                .tags(&["business:1:product:7"])
                .has("k")
                .await
                .unwrap(),
            "wired dispatcher must trigger the invalidator"
        );
    }
}
