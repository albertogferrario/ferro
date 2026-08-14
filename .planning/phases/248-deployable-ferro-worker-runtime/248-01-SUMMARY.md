---
phase: 248-deployable-ferro-worker-runtime
plan: "01"
subsystem: ferro-queue / framework / app
tags: [worker-runtime, boot-factoring, queue-routing, WR-01, D-05, D-07, wave-1]
dependency_graph:
  requires:
    - 248-00 (test scaffolds: worker_runtime.rs, worker_boot.rs)
  provides:
    - ferro-queue/src/db.rs (registered_queue_names() + JobRegistrarEntry.queue)
    - framework/src/app.rs (run_common_boot + run_worker + WR-01 attach + spawn_in_process_worker)
    - framework/src/lib.rs (run_worker + run_common_boot re-exports)
    - app/src/bootstrap.rs (transport-ownership comment)
    - framework/tests/worker_boot.rs (D-07 un-stubbed, real boot step driven)
  affects:
    - ferro-macros (offload.rs: JobRegistrarEntry emission gains queue: None)
    - framework/tests/offload_delta_broadcast.rs (JobRegistrarEntry literals)
    - framework/tests/offload_result_round_trip.rs (JobRegistrarEntry literals)
    - ferro-queue/tests/offload_round_trip.rs (JobRegistrarEntry literal)
tech_stack:
  added: []
  patterns:
    - "run_common_boot(bootstrap_fn, no_worker) shared boot seam (no-duplicate-control-surface)"
    - "#[cfg(feature = \"redis-transport\")] / #[cfg(not(...))] feature-flag pair for WR-01/D-07"
    - "App::singleton overwrite-by-TypeId for transport-attached Broadcaster replacement"
    - "Queue::is_initialized() guard to skip DB init in tests that pre-initialise Queue"
    - "Free-standing module-level run_worker / run_common_boot wrappers delegating to Application::<NoMigrator>"
key_files:
  created: []
  modified:
    - ferro-queue/src/db.rs
    - framework/src/app.rs
    - framework/src/lib.rs
    - app/src/bootstrap.rs
    - framework/tests/worker_boot.rs
    - ferro-macros/src/offload.rs
    - ferro-queue/tests/offload_round_trip.rs
    - framework/tests/offload_delta_broadcast.rs
    - framework/tests/offload_result_round_trip.rs
decisions:
  - "run_common_boot exposed as #[doc(hidden)] pub on Application<M> plus free-standing wrapper at module level; both delegated via Application::<NoMigrator> to avoid adding the generic parameter to the public API"
  - "D-07 test drives run_common_boot by pre-initialising Queue with a temp SQLite NamedTempFile so the get_database_connection() DB step is skipped (Queue::is_initialized() guard)"
  - "No JobRegistrarEntry literals outside ferro-macros emission required a declared queue (all set queue: None); Plan 02 replaces the macro emission with the parsed declared_queue"
  - "None-broadcaster fallback (register_offload_hooks()) left intact per RESEARCH critical constraint; Phase 249.1 removes it"
  - "SC#4 grep false-positive: container/testing.rs:58 matches 'keda' case-insensitively inside 'FakeDatabase'; pre-existing, not introduced by this plan"
metrics:
  duration_seconds: ~700
  completed_date: "2026-08-14"
  tasks_completed: 3
  files_created: 0
  files_modified: 9
---

# Phase 248 Plan 01: Framework Boot Factoring + Worker Entry Point Summary

## One-liner

Factored the framework boot path into a single shared `run_common_boot` seam, wired
WR-01 Redis transport attachment under the feature flag, and exposed `run_worker` as a
public entry point — satisfying D-05, D-06, D-07 and providing the foundation for Plan 03.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | JobRegistrarEntry.queue field + registered_queue_names() | ad826f12 | ferro-queue/src/db.rs, 5 literal sites |
| 2 | Factor run_common_boot + add run_worker + WR-01 | 5570b74d | framework/src/app.rs, framework/src/lib.rs |
| 3 | Bootstrap annotation + D-07 test un-stub | 97057467 | app/src/bootstrap.rs, framework/tests/worker_boot.rs |

## Exact Signatures

### `run_common_boot`

```rust
// framework/src/app.rs — on Application<M> (generic impl)
#[doc(hidden)]
pub async fn run_common_boot(bootstrap_fn: Option<BootstrapFn>, no_worker: bool)

// module-level free function (re-exported as ferro::run_common_boot)
#[doc(hidden)]
pub async fn run_common_boot(bootstrap_fn: Option<BootstrapFn>, no_worker: bool)
// delegates to Application::<NoMigrator>::run_common_boot(bootstrap_fn, no_worker)
```

Re-export in `framework/src/lib.rs`:
```rust
pub use app::{run_common_boot, run_worker, Application};
```

### `run_worker`

```rust
// framework/src/app.rs — on Application<M>
pub async fn run_worker(bootstrap_fn: Option<BootstrapFn>, queues: Vec<String>)

// module-level free function (re-exported as ferro::run_worker)
pub async fn run_worker(bootstrap_fn: Option<BootstrapFn>, queues: Vec<String>)
// delegates to Application::<NoMigrator>::run_worker(bootstrap_fn, queues)
```

## How the D-07 Test Drives the Boot Step

The `transport_url_no_feature_warns` scenario in `framework/tests/worker_boot.rs` drives
`ferro::run_common_boot(None, /*no_worker=*/true)` directly. Since `run_common_boot`
internally calls `get_database_connection()` only when `!Queue::is_initialized()`, the
test pre-initialises the Queue with a `tempfile::NamedTempFile` SQLite database and runs
the `CreateJobsTable` migration before calling `run_common_boot`. This lets the DB step
be skipped and the test proceeds straight to the broadcaster logic, where the D-07
`#[cfg(not(feature = "redis-transport"))]` branch fires and emits `tracing::warn!`.
The test asserts: (a) no panic, and (b) `App::get::<Broadcaster>()` is `Some` afterwards.

## JobRegistrarEntry Literals Outside the Macro

Six hand-written `JobRegistrarEntry { ... }` literals existed outside the `#[offload]`
macro emission:

| File | Count | Fix |
|------|-------|-----|
| framework/tests/offload_delta_broadcast.rs | 2 | Added `queue: None` |
| framework/tests/offload_result_round_trip.rs | 3 | Added `queue: None` |
| ferro-queue/tests/offload_round_trip.rs | 1 | Added `queue: None` |

The macro emission in `ferro-macros/src/offload.rs` was also updated to emit `queue: None`
(Plan 02 replaces this with the parsed `declared_queue` value once `#[offload(queue = "name")]`
parsing is wired).

## None-Broadcaster Fallback Intact

The `else { crate::offload::register_offload_hooks() }` branch in `run_common_boot` is
preserved exactly as the RESEARCH critical constraint requires. It is the valid path for
headless worker-only deployments where no `Broadcaster` is registered. Phase 249.1
removes it in the convergence sweep.

The branch location in `run_common_boot` (framework/src/app.rs):
```rust
} else {
    // None-broadcaster fallback: valid for headless worker-only deployments.
    // Phase 249.1 convergence sweep removes this path.
    crate::offload::register_offload_hooks();
}
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Free-standing module-level wrappers for run_worker / run_common_boot**

- **Found during:** Task 2 — `pub use app::{run_worker, Application}` failed because
  `run_worker` is an associated function on `Application<M>`, not a module-level item.
- **Fix:** Added free-standing `pub async fn run_worker` and `pub async fn run_common_boot`
  at the bottom of `framework/src/app.rs` that delegate to
  `Application::<NoMigrator>::run_worker/run_common_boot`. This avoids exposing the generic
  parameter and gives callers a clean `ferro::run_worker(...)` call site.
- **Files modified:** framework/src/app.rs
- **Commit:** 5570b74d

## Known Stubs

None — all stubs from Plan 00 have been resolved in this plan.

The `queue_unknown_arg.stderr` trybuild fixture and the `queue_arg.rs` pass fixture remain
as stubs from Plan 00, but those are Plan 02's responsibility (macro attribute parsing).

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust
boundaries introduced. The WR-01 path reads `BROADCAST_REDIS_URL` from the environment
(operator-controlled, same trust boundary as `DATABASE_URL`) and opens an outbound Redis
connection at boot. Both vectors were in the plan's `<threat_model>` (T-248-01-01,
T-248-01-02). The Redis URL is never logged (only `error = %e` + static message).

## Self-Check: PASSED

```
[ -f "ferro-queue/src/db.rs" ]                                    → FOUND
[ -f "framework/src/app.rs" ]                                     → FOUND
[ -f "framework/src/lib.rs" ]                                     → FOUND
[ -f "app/src/bootstrap.rs" ]                                     → FOUND
[ -f "framework/tests/worker_boot.rs" ]                           → FOUND

git log → ad826f12, 5570b74d, 97057467 all present
grep "pub queue: Option<&'static str>" ferro-queue/src/db.rs      → FOUND
grep "pub fn registered_queue_names()" ferro-queue/src/db.rs      → FOUND
grep "async fn run_common_boot" framework/src/app.rs              → FOUND
grep "pub async fn run_worker" framework/src/app.rs               → FOUND
grep "with_transport" framework/src/app.rs                        → FOUND
grep "run_worker" framework/src/lib.rs                            → FOUND

cargo build -p ferro-rs                                           → OK
cargo build -p ferro-rs --features redis-transport                → OK
cargo test -p ferro-queue                                         → ok. 1 passed
cargo test -p ferro-rs --test worker_boot                         → ok. 1 passed
cargo fmt --all -- --check                                        → clean
cargo clippy --all --all-targets -- -D warnings                   → clean (0 warnings)
```
