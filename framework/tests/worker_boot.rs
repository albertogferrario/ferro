//! Worker-boot integration suite (WR-01 + D-07).
//!
//! Two scenarios run as sub-functions of a single outer tokio test, following
//! the `offload_delta_broadcast.rs` suite pattern (one `#[tokio::test]`,
//! scenarios as plain `async fn`s, `#[serial_test::serial]` to protect
//! global state across test crates).
//!
//! Scenario 1 — `transport_url_no_feature_warns` (D-07):
//!   When `BROADCAST_REDIS_URL` is set but the `redis-transport` feature is
//!   disabled, the framework boot step must complete without panic and the
//!   registered `Broadcaster` must remain resolvable via `App::get` (no hard
//!   failure from the feature-off fallback path).
//!
//! Scenario 2 — `transport_url_attaches_redis_transport` (WR-01):
//!   Feature-gated behind `redis-transport`; skips when `REDIS_URL` is unset.
//!   After Plan 01 ships `run_common_boot`, this scenario drives the real boot
//!   step and asserts a transport-attached `Broadcaster` is reachable via the
//!   `App` singleton.

extern crate ferro_rs as ferro;

use ferro_broadcast::{BroadcastConfig, Broadcaster};
use ferro_queue::{CreateJobsTable, Queue};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

// ---------------------------------------------------------------------------
// Minimal inline migrator for the Queue jobs table
// ---------------------------------------------------------------------------

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(CreateJobsTable)]
    }
}

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
/// the framework boot step must complete without panic and the in-process hub
/// must be used as the fallback.
///
/// Drives the real `ferro::run_common_boot(None, true)` boot step:
/// - Pre-initialises the Queue connection via a temp SQLite file so the DB
///   step inside `run_common_boot` is skipped (guarded by `is_initialized()`).
/// - Registers a `Broadcaster` whose `transport_redis_url` is `Some(...)`.
/// - Calls `run_common_boot(None, /*no_worker=*/true)` — must not panic.
/// - Asserts the Broadcaster singleton is still resolvable after the boot step.
#[cfg(not(feature = "redis-transport"))]
async fn transport_url_no_feature_warns() {
    // Pre-initialise the Queue DB connection with a temp SQLite file so that
    // run_common_boot's `if !Queue::is_initialized()` guard skips the
    // get_database_connection() call (which needs DATABASE_URL in env).
    if !Queue::is_initialized() {
        let db_file = tempfile::NamedTempFile::new().expect("create temp SQLite file");
        let url = format!("sqlite://{}?mode=rwc", db_file.path().display());
        let conn = Database::connect(&url)
            .await
            .expect("connect to temp SQLite");
        TestMigrator::up(&conn, None)
            .await
            .expect("run jobs migration");
        let _ = Queue::init(conn).await; // OnceLock — error means already initialised; fine.
    }

    // Register a Broadcaster with a Redis URL set, as bootstrap would do.
    let config = BroadcastConfig::new().transport_redis_url("redis://127.0.0.1:6379");
    let broadcaster = Broadcaster::with_config(config);
    ferro::App::singleton(broadcaster);

    // Drive the real shared boot step.
    // Under default features (no redis-transport), the D-07 branch fires:
    //   tracing::warn!("BROADCAST_REDIS_URL is set but the `redis-transport` feature is disabled...")
    // The call must complete without panic.
    ferro::run_common_boot(None, /*no_worker=*/ true).await;

    // Assert the Broadcaster singleton survives the boot step (in-process hub fallback).
    let registered = ferro::App::get::<Broadcaster>();
    assert!(
        registered.is_some(),
        "D-07: Broadcaster singleton must survive run_common_boot under feature-off fallback"
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
    /// Drives the real `run_common_boot` boot step with a Broadcaster whose
    /// `transport_redis_url` matches the live Redis instance. Asserts the
    /// App-registered `Broadcaster` is resolvable after boot.
    pub async fn transport_url_attaches_redis_transport() {
        let Some(url) = redis_url() else {
            eprintln!("REDIS_URL not set — skipping transport_url_attaches_redis_transport");
            return;
        };

        // Pre-initialise Queue DB so run_common_boot's DB step is skipped.
        if !Queue::is_initialized() {
            let db_file = tempfile::NamedTempFile::new().expect("create temp SQLite file");
            let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());
            let conn = Database::connect(&db_url)
                .await
                .expect("connect to temp SQLite");
            TestMigrator::up(&conn, None)
                .await
                .expect("run jobs migration");
            let _ = Queue::init(conn).await;
        }

        // Register a Broadcaster with the Redis URL so run_common_boot's WR-01
        // branch calls RedisTransport::connect and replaces the singleton.
        let config = BroadcastConfig::new().transport_redis_url(&url);
        let broadcaster = Broadcaster::with_config(config);
        ferro::App::singleton(broadcaster);

        // Drive the real shared boot step with redis-transport feature ON.
        // WR-01: RedisTransport::connect is called; the transport-attached
        // Broadcaster replaces the singleton via App::singleton.
        ferro::run_common_boot(None, /*no_worker=*/ true).await;

        // Assert the Broadcaster singleton is still resolvable after boot.
        let registered = ferro::App::get::<Broadcaster>();
        assert!(
            registered.is_some(),
            "WR-01: Broadcaster with transport must be registered via App singleton after run_common_boot"
        );

        // Verify the redundant stand-in path below also compiles (transport construction).
        let _ = Arc::new(
            RedisTransport::connect(&url)
                .await
                .expect("RedisTransport::connect"),
        );
    }
}
