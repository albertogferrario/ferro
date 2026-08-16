//! Round-trip integration tests for the `#[offload]`-derived job substrate.
//!
//! Tests run in `QUEUE_CONNECTION=sync` mode so no database is needed.
//! All tests are marked `#[serial_test::serial]` because they mutate the
//! `QUEUE_CONNECTION` environment variable.
//!
//! - Test A (OFFLOAD-01-d): sync-mode dispatch runs `handle()`.
//! - Test B (OFFLOAD-01-e): a `handle()` that returns `Err` surfaces as a
//!   dispatch failure.
//! - Test C (OFFLOAD-01-f): a Job submitted via `inventory::submit!` is picked
//!   up by `WorkerLoop::from_registry` with zero manual `Queue::register` call.

use std::sync::atomic::{AtomicBool, Ordering};

use ferro_queue::{async_trait, dispatch, Error, Job, JobRegistrarEntry, WorkerConfig, WorkerLoop};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Test A — sync-mode dispatch runs handle()
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RanJob {
    value: i32,
}

static JOB_RAN: AtomicBool = AtomicBool::new(false);

#[async_trait]
impl Job for RanJob {
    fn name(&self) -> &'static str {
        "RanJob"
    }

    async fn handle(&self) -> Result<(), Error> {
        JOB_RAN.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
#[serial_test::serial]
async fn offload_round_trip_sync_mode() {
    std::env::set_var("QUEUE_CONNECTION", "sync");
    JOB_RAN.store(false, Ordering::SeqCst);
    dispatch(RanJob { value: 42 }).await.unwrap();
    assert!(
        JOB_RAN.load(Ordering::SeqCst),
        "handle() must have run in sync mode"
    );
}

// ---------------------------------------------------------------------------
// Test B — Result-error maps to a dispatch failure (OFFLOAD-01-e)
//
// Sync-mode dispatch_immediately() propagates Err(e) from handle() directly
// (dispatcher.rs lines 117-127: the Err arm calls job.failed() then returns Err(e)).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FailingJob;

#[async_trait]
impl Job for FailingJob {
    fn name(&self) -> &'static str {
        "FailingJob"
    }

    async fn handle(&self) -> Result<(), Error> {
        // Mirrors the derived Result<T,E> branch: map a method Err(e) to job_failed.
        let simulated: Result<(), String> = Err("boom".to_string());
        simulated
            .map(|_| ())
            .map_err(|e| Error::job_failed("FailingJob", e))
    }
}

#[tokio::test]
#[serial_test::serial]
async fn offload_result_err_maps_to_job_failure() {
    std::env::set_var("QUEUE_CONNECTION", "sync");
    let res = dispatch(FailingJob).await;
    assert!(
        res.is_err(),
        "Err(e) from handle() must surface as a dispatch failure in sync mode"
    );
    assert!(
        format!("{}", res.unwrap_err()).contains("boom"),
        "Error message must contain the original failure reason"
    );
}

// ---------------------------------------------------------------------------
// Test C — inventory auto-registration picked up by from_registry (OFFLOAD-01-f)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InventoryJob;

#[async_trait]
impl Job for InventoryJob {
    fn name(&self) -> &'static str {
        "InventoryJob"
    }

    async fn handle(&self) -> Result<(), Error> {
        Ok(())
    }
}

// Simulate what #[offload] emits: self-register via inventory, zero bootstrap code.
inventory::submit! {
    JobRegistrarEntry {
        register: |w: &mut WorkerLoop| { w.register::<InventoryJob>(); },
        name: "InventoryJob",
        queue: None,
    }
}

#[tokio::test]
#[serial_test::serial]
async fn offload_job_auto_registers_via_inventory() {
    // No manual Queue::register::<InventoryJob>() call anywhere in this test.
    let worker = WorkerLoop::from_registry(WorkerConfig::default());
    assert!(
        worker.registered_job_count() >= 1,
        "InventoryJob must be auto-registered by from_registry via the inventory path"
    );
}

// ---------------------------------------------------------------------------
// Test D — WR-02: two successive from_registry calls register each job once (SC#2)
//
// Each from_registry starts with a fresh HashMap; HashMap::insert is per-key
// idempotent, so re-running the same registrar overwrites with an identical handler
// and leaves the count unchanged. This test asserts that invariant explicitly.
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial_test::serial]
async fn double_from_registry_does_not_double_register() {
    // InventoryJob is already submitted to inventory (Test C in this file).
    // Two WorkerLoop instances each start with a fresh HashMap; HashMap::insert
    // is per-key idempotent so re-running the same registrar overwrites with an
    // identical handler, leaving count unchanged.
    let w1 = WorkerLoop::from_registry(WorkerConfig::default());
    let count1 = w1.registered_job_count();

    let w2 = WorkerLoop::from_registry(WorkerConfig::default());
    let count2 = w2.registered_job_count();

    assert_eq!(
        count1, count2,
        "second from_registry must not double-register: expected {count1}, got {count2}"
    );
    assert!(count1 >= 1, "at least InventoryJob must be registered");
}
