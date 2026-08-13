---
phase: 246-result-read-model-snapshot
reviewed: 2026-08-14T00:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - ferro-projection/src/direct.rs
  - ferro-projection/src/lib.rs
  - framework/src/offload.rs
  - framework/src/lib.rs
  - framework/src/app.rs
  - framework/Cargo.toml
  - ferro-queue/src/offload.rs
  - ferro-queue/src/dispatcher.rs
  - ferro-queue/src/db.rs
  - ferro-queue/src/migration.rs
  - ferro-queue/src/job.rs
  - ferro-queue/src/worker.rs
  - ferro-queue/src/lib.rs
  - ferro-macros/src/offload.rs
  - framework/tests/offload_result_round_trip.rs
findings:
  critical: 0
  high: 0
  medium: 2
  low: 3
  total: 5
status: clean
---

# Phase 246: Code Review Report

**Reviewed:** 2026-08-14
**Depth:** standard
**Files Reviewed:** 15
**Status:** clean (no critical or high-severity issues)

## Summary

Phase 246 adds a read-model snapshot path for offloaded job results: a direct upsert/read API in `ferro-projection` (`direct.rs`), a typed envelope layer in `framework/src/offload.rs` (`OffloadResult<T>`, `persist_result`/`persist_error`/`persist_result_raw`/`read_result`, `register_offload_hooks`), and write-back in the worker (`spawn_job` success path, `handle_failure` terminal path, panic path). The injection boundary is maintained correctly — `ferro-queue` carries no `ferro-projection` import; the hook is injected through `OffloadResultHook` / `OnceLock`. Design invariants D-11 (no upward dependency), D-07 (envelope shape), and the non-fatal persistence contract (T-246-05) are all satisfied. The two bugs noted in the prompt as already fixed in commit `f84124a4` (`App::make` → `.expect()`, serde_json routed through `::ferro::serde_json::*`) are confirmed correct in the current source.

Two medium findings and three low findings are noted below. None blocks shipping.

## Warnings

### WR-01: Sync-mode `offload()` silently drops the result — no snapshot is written

**File:** `ferro-queue/src/dispatcher.rs:145–176`

**Issue:** When `QUEUE_CONNECTION=sync`, `Offloadable::offload()` calls `PendingDispatch::dispatch()` which routes to `dispatch_immediately()`. That path calls `self.job.handle()` (not `handle_with_value()`) and ignores the `handle_key`. The offload persistence hook is therefore never invoked. A caller holding an `OffloadHandle` will receive `Ok(None)` from `read_result` indefinitely — identical to the "job not yet started" state — with no indication that the result was simply discarded. This violates the caller's reasonable expectation that `offload()` always eventually populates the result envelope.

The framework already emits a `WARNING` to stderr in `app.rs:427–434` when jobs are registered and sync mode is active, but that warning covers the general queue-not-running case, not the specific offload-result-is-lost case.

**Fix:** Either extend `dispatch_immediately` to call `handle_with_value()`, capture the returned `Option<Value>`, and invoke `persist_offload_outcome` — analogous to the worker's success path (lines 547–557 of `worker.rs`) — or add an explicit `tracing::warn!` inside `dispatch_immediately` when `self.handle_key.is_some()` to surface the silent-loss contract clearly:

```rust
// In dispatch_immediately, before calling handle():
if self.handle_key.is_some() {
    tracing::warn!(
        job = %job_name,
        "sync-mode dispatch ignores offload handle_key — no snapshot will be written; \
         use QUEUE_CONNECTION=db to persist offload results"
    );
}
```

### WR-02: Reaper-parked jobs never receive a failed envelope

**File:** `ferro-queue/src/db.rs:493–510` (`reaper` step 2) and `db.rs:727–733` (`reap_startup_claims`)

**Issue:** When the visibility timeout expires and the row is parked as `failed` by the reaper (or by the startup orphan reap), no offload result envelope is written. A caller polling `read_result` for such a job will observe `Ok(None)` indefinitely, indistinguishable from a pending job. The worker's `handle_failure` path does write an envelope for terminal errors (D-09 comment in the code is correct for the handler path), but the reaper bypasses the hook entirely — it operates at the SQL layer without access to `handle_key` routing.

This is a scoping issue: the reaper operates in `ferro-queue/src/db.rs`, which intentionally cannot call the `OffloadResultHook` (`ferro-queue` has no access to the static connection or the hook's DB parameter at the SQL level). A clean fix requires the reaper to surface the affected `handle_key` values so the caller can write envelopes. An interim acceptable mitigation is to document this gap explicitly in `db::reaper`'s doc comment.

**Fix (Phase 247 scope is acceptable; document gap now):** Add to `db::reaper`'s doc comment:

```rust
/// Note: jobs parked as failed here do NOT receive an offload result envelope.
/// Callers using `read_result` on such handles will observe `Ok(None)`.
/// Phase 247 must either: (a) have the reaper return reaped `handle_key` values
/// for the caller to write envelopes, or (b) introduce a pending-marker snapshot
/// that distinguishes "not done" from "timed out silently".
```

## Info

### IN-01: `drain_for_test` busy-wait is fragile for slow CI environments

**File:** `ferro-queue/src/worker.rs:394–433`

**Issue:** `drain_for_test` waits for three consecutive idle rounds with `sleep_duration` between each. In the E2E harness, `sleep_duration` is 10 ms, so the drain exits ~30 ms after the last claim. The harness then adds a hardcoded 200 ms sleep (`framework/tests/offload_result_round_trip.rs:107`). On a loaded CI runner or under memory pressure the spawned job tasks (which write snapshots) may not have completed within 200 ms, causing SC1/SC2/SC3a/SC3b to see `Ok(None)`.

This is not a logic bug (CI is green), but the timing is not bounds-guaranteed. A more reliable pattern would be to poll `read_result` with a timeout rather than sleeping a fixed duration.

**Fix (low priority):** Consider replacing the fixed sleep with a loop that polls until `read_result` returns `Some` or a timeout elapses. This makes test intent explicit and eliminates implicit timing coupling.

### IN-02: `version = 1` is never updated even on repeated writes

**File:** `ferro-projection/src/direct.rs:54`, `direct.rs:58–65`

**Issue:** The `OnConflict` clause updates `State` and `UpdatedAt` but omits `Version`. The doc comment on line 16–18 explains this is intentional (one-shot, no event fold). The concern is that if `snapshot_read` returns a stale cached result from a layer above (e.g., a future caching layer), the immutable version field provides no signal that the value changed. This is a design note rather than a current bug.

**Fix:** No action required in Phase 246. When Phase 247 introduces a pending marker or any read-caching layer, revisit whether the version field needs to increment on update.

### IN-03: `OffloadResultHook` type takes `&'static DatabaseConnection`

**File:** `ferro-queue/src/dispatcher.rs:30–34`

**Issue:** The hook signature requires a `&'static DatabaseConnection`. This works because `Queue::connection()` returns `&'static DatabaseConnection` (from the `OnceLock`). However, the `'static` bound appears in the public `OffloadResultHook` type alias, which means any alternative hook implementation (e.g., in tests) must also provide a static connection. The E2E harness works around this correctly using `Queue::init` + `Queue::connection()`, but the constraint is not documented on the type alias.

**Fix (doc-only):** Add a note to the `OffloadResultHook` type alias that the connection must be `'static`, which in practice means the hook must use `Queue::connection()` or an equivalent static.

---

## Design Invariant Verification

| Invariant | Status |
|-----------|--------|
| D-11: `ferro-queue` imports neither `framework` nor `ferro-projection` | Confirmed — `ferro-queue/Cargo.toml` lists no such dependency; grep of all `use` statements in `ferro-queue/src/` shows only `crate::`, `async_trait`, `serde`, `sea_orm`, `chrono`, `uuid`, `rand`, `futures`, `tracing`, and `inventory`. |
| Terminal error from `spawn_job`/`handle_failure`, not `Job::failed()` | Confirmed — `handle_failure` calls `persist_offload_outcome` on the terminal branch; no `failed()` override is emitted by the macro (`ferro-macros/src/offload.rs` emits `handle_with_value` only). |
| Value capture in `handle_with_value`, not `handle()` | Confirmed — `worker.rs` calls `handle_with_value()` via the registered `JobHandler`; `handle()` is separate and called only in sync mode. |
| Persistence failure is non-fatal | Confirmed — `register_offload_hooks` in `framework/src/offload.rs:183–197` logs `tracing::warn!` on `Err` and returns `()`; the hook never propagates the error upward. |
| `handle_key` is a DB column, not a payload field | Confirmed — carried in `PendingDispatch.handle_key`, written to `jobs.handle_key` column via `enqueue()`, parsed back in `parse_job_row()`. No payload field. |
| `OffloadHandle::key()` == worker write key | Confirmed — `HandleKey` is minted in `Offloadable::offload()` and passed to `with_handle_key()`; the same UUID string is carried in `job_row.handle_key` and written by `persist_offload_outcome`. |
| `App::make` returns `Option`, not `Result` | Confirmed — macro emits `.expect(#expect_msg)` in both `handle()` and `handle_with_value()`. |
| `serde_json` routed through `::ferro::serde_json::*` | Confirmed — `ferro-macros/src/offload.rs:287,293,298,303` uses `::ferro::serde_json::to_value`. |

## Security Surface Notes (from 246-RESEARCH §Security Domain)

Three deferred items are in scope for awareness:

1. **Error-string disclosure** (`format!("{e}")` in `handle_failure`, worker.rs:564) — the full `Display` of the job error is stored in the `failed` envelope. In Phase 246 the envelope is retrieved in-process only. Phase 247 must sanitize before any client-facing exposure. Severity for Phase 246: **low** (no client exposure path yet).

2. **Unbounded snapshot growth** — `projection_snapshots` rows accumulate with no TTL. Deferred to a housekeeping phase. Not a correctness issue for 246.

3. **Handle-key access control** — any code with a DB connection can call `read_result` with any UUID and retrieve the envelope. Deferred to Phase 247. Not exploitable in the current in-process-only retrieval surface.

None of these constitute a critical or high issue for Phase 246.

---

_Reviewed: 2026-08-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
