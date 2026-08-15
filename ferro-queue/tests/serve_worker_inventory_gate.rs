//! Regression test for Phase 249.2 — serve worker inventory-registration gate (OFFLOAD-05).
//!
//! Guards the seam between the serve boot gate (`framework/src/app.rs:488`,
//! `!no_worker && Queue::has_registered_jobs()`) and `#[offload]`-derived inventory
//! registration. A pure-offload app registers jobs ONLY via the `JobRegistrarEntry`
//! inventory; before the fix `has_registered_jobs()` inspected only the manual
//! `JOB_REGISTRARS` Vec and returned `false`, so serve spawned no worker.
//!
//! IMPORTANT: `inventory::submit!` is GLOBAL to this test binary. The submit below makes
//! `has_registered_jobs()` return `true` for the ENTIRE binary, permanently. This file
//! therefore contains ONLY the positive (inventory-present) assertions; it must NOT assert
//! an empty-inventory / `has_registered_jobs() == false` case — that cannot hold here.

use async_trait::async_trait;
use ferro_queue::{Error, Job, JobRegistrarEntry, Queue, WorkerLoop};
use serde::{Deserialize, Serialize};

/// A job registered ONLY through the compile-time inventory — never via `Queue::register`.
#[derive(Serialize, Deserialize)]
struct ServeGateInventoryJob;

#[async_trait]
impl Job for ServeGateInventoryJob {
    fn name(&self) -> &'static str {
        "ServeGateInventoryJob"
    }

    async fn handle(&self) -> Result<(), Error> {
        Ok(())
    }
}

// Simulate what `#[offload]` emits: self-register via inventory, zero bootstrap code.
inventory::submit! {
    JobRegistrarEntry {
        register: |w: &mut WorkerLoop| { w.register::<ServeGateInventoryJob>(); },
        name: "ServeGateInventoryJob",
        queue: None,
    }
}

/// Criterion 1: the predicate is true from the inventory alone (no manual registration).
/// Criterion 3: reproduce the EXACT serve-boot spawn decision so a future regression that
/// reverts the predicate to `JOB_REGISTRARS`-only fails here.
#[test]
fn serve_gate_spawns_worker_for_inventory_only_app() {
    // No `Queue::register::<ServeGateInventoryJob>()` anywhere — inventory is the only source.

    // Criterion 1: predicate consults the inventory.
    assert!(
        Queue::has_registered_jobs(),
        "has_registered_jobs() must be true from the JobRegistrarEntry inventory alone \
         (no manual Queue::register call)"
    );

    // Criterion 2/3: the serve gate is `!no_worker && has_registered_jobs()`
    // (framework/src/app.rs:488). Reproduce that exact boolean for both no_worker states.
    let has = Queue::has_registered_jobs();

    // Default `serve` (no_worker = false) → gate is true → worker spawns.
    let no_worker = false;
    assert!(
        !no_worker && has,
        "default serve path (no --no-worker) must decide to spawn the in-process worker \
         for an inventory-only app"
    );

    // `serve --no-worker` (no_worker = true) → gate is false → skipped, even with jobs.
    // The negated form mirrors the exact gate expression from app.rs:488 for clarity.
    // `no_worker || !has` is the boolean equivalent; allow the explicit form.
    let no_worker = true;
    #[allow(clippy::nonminimal_bool)]
    let gate_skips = !(!no_worker && has);
    assert!(
        gate_skips,
        "serve --no-worker must still skip the in-process worker even when jobs are registered"
    );
}
