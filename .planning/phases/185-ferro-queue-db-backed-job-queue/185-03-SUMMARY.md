---
phase: 185-ferro-queue-db-backed-job-queue
plan: "03"
subsystem: ferro-queue
tags: [queue, sea-orm, worker, sqlite, postgres, panic-isolation, sigterm, tenant-scope]
dependency_graph:
  requires:
    - phase: 185-02
      provides: [db::claim, db::reaper, db::enqueue, db::delete_job, db::fail_job, db::release_job, db::requeue_claimed_by, Queue global]
  provides:
    - WorkerLoop with reaper/claim/spawn cycle and panic isolation (D-11)
    - SIGTERM + Ctrl-C graceful shutdown with semaphore drain + requeue_claimed_by (D-10)
    - DB-backed dispatch via db::enqueue — all three free functions preserved (D-16)
    - Zero redis/QueueConnection references in ferro-queue/src
  affects: [ferro-queue, framework, ferro-mcp]
tech-stack:
  added: [tokio signal feature]
  patterns:
    - AssertUnwindSafe + FutureExt::catch_unwind for async panic isolation
    - AtomicBool shutdown flag shared between SIGTERM handler and programmatic shutdown()
    - JobHandler closure captures retry_delay(attempt) at registration time
    - Semaphore acquired inside spawned task (not in sync caller) for correct permit lifetime
key-files:
  created: []
  modified:
    - ferro-queue/src/worker.rs
    - ferro-queue/src/dispatcher.rs
    - ferro-queue/src/lib.rs
    - ferro-queue/Cargo.toml
  deleted:
    - ferro-queue/src/queue.rs
key-decisions:
  - "JobHandler closure signature extended to (String, u32) -> (Result<(), Error>, Duration) — captures retry_delay(attempt) at call time, not at registration. Keeps the closure approach under 30 lines and honours per-job override without a separate registry closure."
  - "Panic default jitter delay is a free fn applying the same formula as Job::retry_delay default — used when catch_unwind catches a panic (no job instance available to call retry_delay on)"
  - "tokio signal feature added to ferro-queue/Cargo.toml — SIGTERM handler requires tokio::signal::unix which is feature-gated"
  - "pub type Worker = WorkerLoop kept for API continuity — external callers using Worker continue to compile without changes"
requirements-completed: [QUEUE-F-01, QUEUE-F-03]
duration: 325s
completed: "2026-06-07"
---

# Phase 185 Plan 03: WorkerLoop + DB Dispatcher Summary

**DB-backed WorkerLoop with reaper/claim/spawn cycle, catch_unwind panic isolation, SIGTERM graceful shutdown with drain+requeue, and dispatcher wired to db::enqueue — ferro-queue is now fully Redis-free.**

## Performance

- **Duration:** ~325s
- **Started:** 2026-06-07T17:50:37Z
- **Completed:** 2026-06-07T17:56:00Z
- **Tasks:** 2
- **Files modified:** 4 modified, 1 deleted

## Accomplishments

- `WorkerLoop` replaces the old `Worker` struct: runs a `reaper→claim→spawn` cycle, acquires a semaphore permit inside each spawned task, and wraps every handler in `AssertUnwindSafe(...).catch_unwind()` so panics count as failed attempts instead of killing the loop (D-11, T-185-03)
- SIGTERM and Ctrl-C both set a shared `Arc<AtomicBool>` shutdown flag; `WorkerLoop::shutdown()` sets the same flag so programmatic and signal-based shutdown share one path; on shutdown the loop drains in-flight jobs via `semaphore.acquire_many` then calls `db::requeue_claimed_by` (D-10)
- `dispatcher::dispatch_to_queue` stub replaced with a real `db::enqueue` call preserving queue, tenant_id, idempotency_key, delay, and job_type/max_retries (T-185-09); `register_tenant_capture_hook` and all three free functions unchanged (D-16, D-17)
- `queue.rs` (Redis `QueueConnection`, `ConnectionManager`, `BRPOP`/`LPUSH`) deleted entirely; zero redis/QueueConnection references remain in `ferro-queue/src`

## Task Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | WorkerLoop — reaper/claim cycle, panic isolation, SIGTERM shutdown | 36a481ff | ferro-queue/src/worker.rs, ferro-queue/src/lib.rs, ferro-queue/Cargo.toml |
| 2 | DB-backed dispatcher + delete queue.rs + lib.rs re-exports | 99f14884 | ferro-queue/src/dispatcher.rs, ferro-queue/src/lib.rs, ferro-queue/src/queue.rs (deleted) |

## Files Created/Modified

- `ferro-queue/src/worker.rs` — Complete rewrite: `WorkerLoop` with `catch_unwind` panic isolation, `AtomicBool` shutdown, `reaper→claim→spawn` cycle, `requeue_claimed_by` on shutdown, `pub type Worker = WorkerLoop` alias
- `ferro-queue/src/dispatcher.rs` — `dispatch_to_queue` stub replaced with `db::enqueue`; `#[allow(dead_code)]` removed from `captured_tenant_id`; Redis doc comments removed
- `ferro-queue/src/lib.rs` — Uncommented `mod worker`; added `pub use worker::...` re-exports; updated crate-level doc example to show `Queue::init(conn)` (no Redis URL); removed `mod queue`
- `ferro-queue/Cargo.toml` — Added `signal` to tokio features
- `ferro-queue/src/queue.rs` — Deleted (Redis backend, 635 lines)

## Decisions Made

1. **JobHandler returns `(Result<(), Error>, Duration)`** — The closure captures `job.retry_delay(attempt)` before `job.handle()` consumes the job. This is the simplest correct approach: the registry-closure computes the delay at invocation time so per-job `retry_delay` overrides are honoured without a separate registry data structure. Stays under 30 lines as specified.

2. **Panic default jitter uses `default_jitter_delay` free fn** — When `catch_unwind` catches a panic, the job instance no longer exists, so `retry_delay` cannot be called. A free `fn default_jitter_delay(attempt: u32) -> Duration` applies the same formula as the `Job::retry_delay` default (full jitter, base 5s, cap 15min). Documented in SUMMARY as the plan specified.

3. **tokio `signal` feature added** — `tokio::signal::unix` is feature-gated. The existing Cargo.toml had `["sync", "rt", "time", "macros"]`; `signal` was missing. Added as a Rule 3 deviation.

4. **`pub type Worker = WorkerLoop`** — Kept for API continuity. Any external caller (framework, gestiscilo) using `Worker` continues to compile without changes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added `signal` feature to tokio dependency**
- **Found during:** Task 1 compilation
- **Issue:** `tokio::signal::unix` is feature-gated behind `tokio/signal`. The feature was absent from `ferro-queue/Cargo.toml`, causing three `failed to resolve: could not find 'signal' in 'tokio'` errors.
- **Fix:** Added `"signal"` to the tokio features list in `ferro-queue/Cargo.toml`.
- **Files modified:** `ferro-queue/Cargo.toml`
- **Verification:** `cargo build -p ferro-queue` clean after change
- **Committed in:** 36a481ff (Task 1 commit)

**2. [Rule 1 - Bug] Removed unused `claimed_any` flag from run loop**
- **Found during:** Task 1 clippy pass
- **Issue:** `claimed_any = true` was set then immediately followed by `continue 'outer`, so the `if !claimed_any` block at the bottom was always reached with `claimed_any = false`. `cargo build` emitted `unused_assignments` warning; clippy -D warnings would fail.
- **Fix:** Removed `claimed_any` variable entirely; the idle sleep is unconditionally at the bottom of the loop body (only reached when no queues yielded a job, since `continue 'outer` skips it when a job is claimed).
- **Files modified:** `ferro-queue/src/worker.rs`
- **Verification:** `cargo clippy -p ferro-queue --all-targets -- -D warnings` clean
- **Committed in:** 36a481ff (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both fixes were necessary for compilation and clippy compliance. No scope creep.

## Test Results

```
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

New tests added in Task 1:
- `worker::tests::test_worker_loop_new` — WorkerLoop can be constructed without a connection argument
- `worker::tests::test_shutdown_sets_flag` — `WorkerLoop::shutdown()` sets the AtomicBool
- `worker::tests::test_worker_config_visibility_timeout_default` — visibility_timeout default is 300s
- `worker::tests::test_default_jitter_delay_bounds` — jitter stays in bounds across attempts 0, 3, 30

Previous tests from Plans 01-02 (config, dispatcher, db, migration, job, worker scope) all pass unchanged.

## Known Stubs

None. All functions fully implemented.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries introduced.

Mitigations verified present:
- **T-185-03** (panic in handle() killing loop): `AssertUnwindSafe(...).catch_unwind()` wraps handler — `grep -q 'catch_unwind' ferro-queue/src/worker.rs` PASS
- **T-185-08** (cross-tenant execution): `tenant_id` from `JobRow` drives `scope.with_scope(id, fut)` before handler runs — preserved from Plan 01 tenant scope pattern
- **T-185-09** (dispatch sink integrity): `dispatch_to_queue` calls `db::enqueue` with parameterized binding (T-185-01 mitigated in Plan 02)

## Self-Check: PASSED

- ferro-queue/src/worker.rs: FOUND
- ferro-queue/src/dispatcher.rs: FOUND
- ferro-queue/src/lib.rs: FOUND
- ferro-queue/src/queue.rs: DELETED (confirmed)
- Commit 36a481ff (Task 1): FOUND
- Commit 99f14884 (Task 2): FOUND
- `grep -q 'catch_unwind' ferro-queue/src/worker.rs`: PASS
- `grep -q 'db::enqueue' ferro-queue/src/dispatcher.rs`: PASS
- `grep -ri 'redis\|QueueConnection' ferro-queue/src/`: zero matches PASS
- `cargo test -p ferro-queue --lib`: 44 passed PASS
