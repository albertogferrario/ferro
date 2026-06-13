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
use ferro_events::{global_dispatcher, Event};
use std::sync::Arc;

/// Register a cache-invalidation listener for events of type `E`.
///
/// When an event of type `E` is dispatched, `key_fn` is invoked with the event
/// to compute the set of tags to flush. Each tag is flushed independently via
/// [`Cache::tags`] + [`crate::TaggedCache::flush`]. Per-tag flush failures are
/// logged and swallowed.
///
/// Multiple invalidators may be registered for the same event type — all run;
/// order between them is unspecified.
///
/// # Parameters
///
/// - `cache`: an `Arc<Cache>` whose store will be used for tag flushing. The
///   `Arc` is cloned into the closure so the cache outlives the listener
///   registration.
/// - `key_fn`: a closure `Fn(&E) -> Vec<String>` returning the tags to flush.
///   Returning an empty `Vec` is a no-op (and skips the per-tag flush calls).
///
/// # Example
///
/// See the [module-level documentation](self) for a complete wiring example.
pub fn register_invalidator<E, F>(cache: Arc<Cache>, key_fn: F)
where
    E: Event,
    F: Fn(&E) -> Vec<String> + Send + Sync + 'static,
{
    let key_fn = Arc::new(key_fn);
    global_dispatcher().on::<E, _, _>(move |event: E| {
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
            .put("availability:foo", &"slot-grid-blob", Duration::from_secs(60))
            .await
            .unwrap();
        assert!(
            cache.tags(&["business:1:product:7"]).has("availability:foo").await.unwrap(),
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

        EvtFlushNonMatching { product: 99 }.dispatch().await.unwrap();

        assert!(
            cache.tags(&["business:1:product:7"]).has("a").await.unwrap(),
            "unrelated tag must survive"
        );
        assert!(
            !cache.tags(&["business:1:product:99"]).has("b").await.unwrap(),
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
}
