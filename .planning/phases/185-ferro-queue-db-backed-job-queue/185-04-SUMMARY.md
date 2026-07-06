---
phase: 185-ferro-queue-db-backed-job-queue
plan: "04"
subsystem: framework + ferro-queue + ferro-mcp
tags: [queue, framework-integration, worker-autostart, debug-endpoints, mcp-fix]
dependency_graph:
  requires:
    - phase: 185-02
      provides: [Queue global, db free functions, QueueStats, FailedJobInfo]
    - phase: 185-03
      provides: [WorkerLoop, WorkerConfig, from_registry, has_registered_jobs]
  provides:
    - ferro::queue namespaced module (D-02, one control surface)
    - Queue::register<J> / has_registered_jobs / apply_registrars global registrar
    - WorkerLoop::from_registry(config)
    - WorkerLoop auto-start in Application::run server path (D-09)
    - debug endpoints reading jobs table via DB-backed free functions (D-18)
    - ferro-mcp job_history reading failed jobs from jobs WHERE status='failed'
  affects:
    - framework/src/lib.rs
    - framework/src/app.rs
    - framework/src/debug/mod.rs
    - ferro-queue/src/db.rs
    - ferro-queue/src/worker.rs
    - ferro-mcp/src/tools/job_history.rs
tech_stack:
  added: []
  patterns:
    - Global Mutex<Vec<RegisterFn>> for pre-start job-type registration
    - is_initialized() guard prevents double-init in run_server_internal
    - WorkerLoop::from_registry applies registrars then returns a ready loop
    - tokio::spawn for non-blocking WorkerLoop in server boot path
key_files:
  created: []
  modified:
    - framework/src/lib.rs
    - framework/src/app.rs
    - framework/src/debug/mod.rs
    - ferro-queue/src/db.rs
    - ferro-queue/src/worker.rs
    - ferro-mcp/src/tools/job_history.rs
decisions:
  - "Global JOB_REGISTRARS Mutex<Vec<RegisterFn>> lives in db.rs alongside Queue — Queue::register<J>/has_registered_jobs/apply_registrars are inherent methods on Queue, consistent with Queue::init/connection/is_initialized already there. WorkerLoop::from_registry(config) in worker.rs calls apply_registrars. This avoids a new file and keeps the two halves of the Queue API co-located."
  - "QueueStats derives Default — enables unwrap_or_default() in debug endpoints; the zero value (empty queues, 0 failed) is the correct fallback when no stats are available."
  - "debug/mod.rs uses QueueConfig::from_env().default_queue to determine the queue name — consistent with how the worker reads its queue list; no hardcoded string."
metrics:
  duration: "352s"
  completed: "2026-06-07"
  tasks_completed: 2
  files_changed: 6
---

# Phase 185 Plan 04: Framework Integration + MCP Fix Summary

**Namespaced `ferro::queue` module, WorkerLoop auto-start in `Application::run`, DB-backed debug endpoints, and corrected ferro-mcp failed-jobs query — the queue now "just works" with a single binary.**

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Namespaced ferro::queue module + WorkerLoop auto-start in server boot | e097af77 | framework/src/lib.rs, framework/src/app.rs, ferro-queue/src/db.rs, ferro-queue/src/worker.rs, framework/src/debug/mod.rs |
| 2 | Debug endpoints over DB + ferro-mcp failed-jobs query fix | 2246b0cc | ferro-mcp/src/tools/job_history.rs |
| — | cargo fmt formatting | 45de48ae | ferro-queue/src/db.rs, framework/src/lib.rs |

## Decisions Made

1. **Global JOB_REGISTRARS in db.rs alongside Queue** — `Queue::register<J>`, `Queue::has_registered_jobs`, and `Queue::apply_registrars` are inherent methods on the existing `Queue` struct in `db.rs`. `WorkerLoop::from_registry(config)` in `worker.rs` calls `Queue::apply_registrars`. This keeps the job-registry API co-located with the connection API, consistent with `Queue::init/connection/is_initialized`. The consumer pattern is: call `ferro::queue::Queue::register::<MyJob>()` in bootstrap, the framework spawns the loop automatically.

2. **QueueStats derives Default** — `unwrap_or_default()` in the debug endpoint `handle_queue_stats` requires `QueueStats: Default`. The zero value (`queues: vec![], total_failed: 0`) is the correct fallback when the DB is unreachable or returns an error. Added `Default` to `QueueStats` derive — no behavioral change to the stats query itself.

3. **debug/mod.rs uses `QueueConfig::from_env().default_queue`** — The old Redis handler called `conn.config().default_queue` on a `QueueConnection`. The new handler uses `ferro_queue::QueueConfig::from_env().default_queue` to read `QUEUE_DEFAULT` env var (defaulting to `"default"`). Consistent with how `WorkerConfig::default()` also defaults to the "default" queue.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] debug/mod.rs Redis method calls blocked framework build during Task 1**
- **Found during:** Task 1 — `cargo build -p ferro-rs` failed with 6 errors on `conn.config()`, `conn.get_pending_jobs()`, etc.
- **Issue:** The existing `handle_queue_jobs` and `handle_queue_stats` called Redis-backend methods on `QueueConnection`, but `Queue::connection()` now returns `&DatabaseConnection`. The debug handler fixes were planned for Task 2, but the framework won't compile without them.
- **Fix:** Applied the Task 2 debug/mod.rs changes during Task 1 (included in Task 1 commit e097af77). Task 2 commit 2246b0cc contains only the ferro-mcp change.
- **Files modified:** `framework/src/debug/mod.rs`
- **Commit:** e097af77

**2. [Rule 1 - Bug] QueueStats missing Default derive**
- **Found during:** Task 1 build after debug/mod.rs fix — `unwrap_or_default()` on `Result<QueueStats, _>` failed with "Default not implemented for QueueStats"
- **Fix:** Added `Default` to `#[derive(...)]` on `QueueStats` in `ferro-queue/src/db.rs`
- **Files modified:** `ferro-queue/src/db.rs`
- **Commit:** e097af77

## Test Results

```
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All previous ferro-queue tests pass unchanged. No new tests added (integration with framework boot path is manual-only per VALIDATION.md).

## Known Stubs

None. All functions fully implemented.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| T-185-10 mitigated | framework/src/debug/mod.rs | `is_debug_enabled()` (403) and `Queue::is_initialized()` (503) guards preserved verbatim |
| T-185-11 accepted | ferro-mcp/src/tools/job_history.rs | Queue filter string-interpolation pattern unchanged (pre-existing; MCP reads local admin DB) |
| T-185-12 mitigated | framework/src/debug/mod.rs, ferro-mcp | Debug queries use LIMIT 100; MCP uses tool-arg limit |

## Self-Check: PASSED

- framework/src/lib.rs: FOUND — `pub mod queue { pub use ferro_queue::... }`
- framework/src/app.rs: FOUND — Queue::init + WorkerLoop::from_registry spawn in run_server_internal
- framework/src/debug/mod.rs: FOUND — DB-backed calls, guards preserved
- ferro-queue/src/db.rs: FOUND — JOB_REGISTRARS, Queue::register, has_registered_jobs, apply_registrars
- ferro-queue/src/worker.rs: FOUND — WorkerLoop::from_registry
- ferro-mcp/src/tools/job_history.rs: FOUND — FROM jobs WHERE status='failed', try_get_by("error")
- Commit e097af77 (Task 1): FOUND
- Commit 2246b0cc (Task 2): FOUND
- Commit 45de48ae (fmt): FOUND
- `cargo build -p ferro-rs`: PASS
- `cargo build -p ferro-mcp`: PASS
- `cargo test -p ferro-queue --lib`: 44 passed PASS
- `cargo fmt --all -- --check`: PASS
- `cargo clippy -p ferro-queue -p ferro-rs --all-targets -- -D warnings`: PASS
