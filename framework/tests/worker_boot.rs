//! Worker-boot integration suite (WR-01 + D-07).
//!
//! Two scenarios run as sub-functions of a single outer tokio test, following
//! the `offload_delta_broadcast.rs` suite pattern (one `#[tokio::test]`,
//! scenarios as plain `async fn`s, `#[serial_test::serial]` to protect
//! global state across test crates).
//!
//! Scenario 1 — `transport_url_no_feature_warns` (D-07):
//!   When `BROADCAST_REDIS_URL` is set but the `redis-transport` feature is
//!   disabled, the framework boot step must complete without panic and emit a
//!   `tracing::warn!` rather than hard-failing.  This scenario stubs the
//!   actual `run_common_boot` call (not yet introduced at Wave 0) with a
//!   `// TODO(plan-01)` marker — Plan 01 must un-stub it once the symbol
//!   exists.
//!
//! Scenario 2 — `transport_url_attaches_redis_transport` (WR-01):
//!   Feature-gated behind `redis-transport`; skips when `REDIS_URL` is unset.
//!   After Plan 01 ships `run_common_boot`, this scenario drives the real boot
//!   step and asserts a transport-attached `Broadcaster` is reachable via the
//!   `App` singleton.

extern crate ferro_rs as ferro;

use ferro_broadcast::{BroadcastConfig, Broadcaster};

// ---------------------------------------------------------------------------
// Outer suite — one tokio::test, two scenario sub-functions
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial_test::serial]
async fn worker_boot_suite() {
    transport_url_no_feature_warns().await;

    #[cfg(feature = "redis-transport")]
    redis_tests::transport_url_attaches_redis_transport().await;
}

// ---------------------------------------------------------------------------
// Scenario 1: D-07 — feature off + URL set → warn!, no panic
// ---------------------------------------------------------------------------

/// D-07: with `redis-transport` feature OFF and `BROADCAST_REDIS_URL` set,
/// the framework must emit a warning and fall back to the in-process hub
/// (no hard failure).
///
/// At Wave 0 the `run_common_boot` symbol does not exist yet; the boot
/// invocation is stubbed with `assert!(true)`.
///
/// TODO(plan-01): replace the `assert!(true)` stub with:
///   `ferro::App::run_common_boot(None, true).await;`
/// (or whichever public entry point Plan 01 exposes for the shared boot step)
/// once the symbol is available.  Plan 02 regenerates `queue_unknown_arg.stderr`
/// via `TRYBUILD=overwrite cargo test -p ferro-macros --test offload_macro`.
#[cfg(not(feature = "redis-transport"))]
async fn transport_url_no_feature_warns() {
    // Register a Broadcaster with a Redis URL set, as bootstrap would do.
    let config = BroadcastConfig::new().transport_redis_url("redis://127.0.0.1:6379");
    let broadcaster = Broadcaster::with_config(config);
    ferro::App::singleton(broadcaster);

    // TODO(plan-01): call `framework::run_common_boot(None, /*no_worker=*/true).await`
    // and verify that tracing::warn! fires (initialize tracing_subscriber in Plan 01
    // once it is added as a dev-dependency, or use the `tracing-test` crate).
    // here once Plan 01 exposes the symbol.  The assertion below is a
    // compile-and-run placeholder so this scenario registers in `--list`
    // at Wave 0 and Plan 01 can un-stub it without structural changes.
    assert!(
        true,
        "D-07 stub: replace with run_common_boot call in Plan 01"
    );

    // Confirm that the Broadcaster was registered (sanity check that the
    // App singleton path we will exercise in Plan 01 is wired correctly now).
    let registered = ferro::App::get::<Broadcaster>();
    assert!(
        registered.is_some(),
        "D-07: Broadcaster singleton must survive the boot step"
    );
}

// Stub for when redis-transport IS enabled (the real scenario lives in redis_tests).
#[cfg(feature = "redis-transport")]
async fn transport_url_no_feature_warns() {
    // Under the redis-transport feature the D-07 warning path is inactive.
    // The WR-01 scenario in redis_tests covers the feature-on branch.
}

// ---------------------------------------------------------------------------
// Scenario 2 (WR-01): feature-gated, skips without REDIS_URL
// ---------------------------------------------------------------------------

#[cfg(feature = "redis-transport")]
mod redis_tests {
    use super::*;
    use ferro_broadcast::transport::redis::RedisTransport;
    use std::sync::Arc;

    fn redis_url() -> Option<String> {
        std::env::var("REDIS_URL").ok().filter(|s| !s.is_empty())
    }

    /// WR-01: when `redis-transport` feature is ON and `REDIS_URL` is set,
    /// the framework boot step must construct and attach a `RedisTransport`
    /// to the registered `Broadcaster`.
    ///
    /// At Wave 0 this scenario exercises only the infrastructure building
    /// blocks (Broadcaster construction + App registration + InMemoryTransport
    /// as a stand-in).  Plan 01 must wire the actual `run_common_boot` call
    /// to make this a meaningful end-to-end boot assertion.
    ///
    /// TODO(plan-01): replace the InMemoryTransport stand-in with a real
    /// `run_common_boot` invocation once Plan 01 ships the symbol.
    pub async fn transport_url_attaches_redis_transport() {
        let Some(url) = redis_url() else {
            eprintln!("REDIS_URL not set — skipping transport_url_attaches_redis_transport");
            return;
        };

        // TODO(plan-01): call `framework::run_common_boot(None, true).await`
        // after registering the Broadcaster with the Redis URL, and then assert
        // `App::get::<Broadcaster>().unwrap().config().transport_redis_url.is_some()`.

        // Stand-in: construct a transport-attached Broadcaster directly to
        // prove the wiring compiles and the App singleton path is functional.
        let transport = Arc::new(
            RedisTransport::connect(&url)
                .await
                .expect("RedisTransport::connect"),
        );
        let broadcaster = Arc::new(Broadcaster::new().with_transport(transport));
        ferro::App::singleton((*broadcaster).clone());

        let registered = ferro::App::get::<Broadcaster>();
        assert!(
            registered.is_some(),
            "WR-01: Broadcaster with transport must be registered via App singleton"
        );
    }
}
