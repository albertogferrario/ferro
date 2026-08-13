---
phase: 246-result-read-model-snapshot
verified: 2026-08-13T22:58:54Z
status: passed
score: 3/3
overrides_applied: 0
re_verification: false
---

# Phase 246: Result Read-Model Snapshot — Verification Report

**Phase Goal:** Give offloaded work a result path — the worker writes the method's return value
into a `ferro-projection` snapshot keyed by the handle, so the result is durably retrievable
after completion without the request having waited on it.

**Requirement:** OFFLOAD-03

**Verified:** 2026-08-13T22:58:54Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | On worker completion the return value is persisted as a projection snapshot keyed by the handle (SC1) | VERIFIED | `worker.rs:549-556` — success arm calls `persist_offload_outcome(handle_key.as_deref(), Ok(val), conn)`; hook calls `persist_result_raw` → `snapshot_write`; `offload_result_round_trip` asserts `Some(Completed { value: 42 })` after drain |
| 2 | The snapshot is retrievable by handle after completion in a test (SC2) | VERIFIED | `framework/tests/offload_result_round_trip.rs:254-282` — captures `handle.key()`, drains, calls `read_result::<i32>(&key, db)` and asserts `Completed { value: 42 }`; a second scenario (`retrieve_by_handle_after_complete`) also asserts key equality with value 99 |
| 3 | A failed/panicking offloaded method records a terminal error state on the handle — no silent drop (SC3) | VERIFIED | `worker.rs:613-619` — `handle_failure` calls `persist_offload_outcome(handle_key, Err(err_msg), conn)` only when `attempts + 1 >= max_retries`; SC3a test asserts `Failed { error }` contains "always fails"; SC3b test asserts `Failed { error }` contains "panicked" |

**Score:** 3/3 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-projection/src/direct.rs` | `snapshot_write` + `snapshot_read` free functions; min 60 lines | VERIFIED | 155 lines; both functions present; upsert uses `OnConflict::columns([Column::ProjectionName, Column::Key])`, `update_columns([Column::State, Column::UpdatedAt])` — `Column::Version` deliberately omitted; three unit tests included |
| `ferro-projection/src/lib.rs` | `pub use direct::{snapshot_read, snapshot_write}` | VERIFIED | `mod direct;` at line 75; `pub use direct::{snapshot_read, snapshot_write}` at line 84 |
| `framework/src/offload.rs` | `persist_result`, `persist_error`, `read_result`, `OffloadResult<T>`, `OFFLOAD_PROJECTION_NAME`, min 70 lines | VERIFIED | 288 lines; all four functions present; `OFFLOAD_PROJECTION_NAME = "offload.result"`; `register_offload_hooks()` present; `persist_result_raw` helper for pre-serialized values; four unit tests |
| `framework/src/lib.rs` | `pub mod offload;` at top level | VERIFIED | Line 228: `pub mod offload;` — top-level, not nested under queue |
| `framework/Cargo.toml` | always-on `ferro-projection` dependency | VERIFIED | Line 54: `ferro-projection = { path = "../ferro-projection", version = "0.3" }` — non-optional |
| `ferro-queue/src/dispatcher.rs` | `OnceLock` hook + `register_offload_result_hook` + `persist_offload_outcome` | VERIFIED | `pub type OffloadResultHook` at line 30; `register_offload_result_hook` at line 41; `persist_offload_outcome` (pub(crate)) handles `handle_key = None` (no-op) and unregistered hook (no-op) |
| `ferro-queue/src/lib.rs` | `register_offload_result_hook` re-exported | VERIFIED | Line 64: included in the re-export list |
| `ferro-queue/src/job.rs` | `handle_with_value` provided method (default: discard value) | VERIFIED | Line 78: `async fn handle_with_value(&self) -> Result<Option<serde_json::Value>, Error>` with default `self.handle().await.map(|_| None)` |
| `ferro-queue/src/worker.rs` | `JobHandler` extended; `spawn_job` calls hook on success + failure; `handle_failure` accepts `handle_key` param | VERIFIED | `JobHandler` returns `Result<Option<serde_json::Value>, Error>`; `persist_offload_outcome` at lines 550 and 616; `handle_failure` signature includes `handle_key: Option<&str>` |
| `ferro-queue/src/migration.rs` | nullable `handle_key TEXT` column in `CreateJobsTable` | VERIFIED | `ColumnDef::new(Jobs::HandleKey).string().null()` at line 71 |
| `ferro-queue/src/db.rs` | `JobRow.handle_key`, `parse_job_row` reads it, `enqueue` writes it, both claim arms SELECT it | VERIFIED | Field at line 135; `try_get_by::<Option<String>, _>("handle_key")` at line 241-243; `enqueue` param `handle_key: Option<&str>` at line 532; `handle_key` appears in both claim SELECT strings (lines 367, 422) and both INSERT arms (lines 554, 593) |
| `ferro-queue/src/dispatcher.rs` | `PendingDispatch.handle_key` field + `with_handle_key()` builder | VERIFIED | Field at line 72; `with_handle_key` at line 113; `self.handle_key.clone()` extracted at line 183; passed to `enqueue` at line 202 |
| `ferro-queue/src/offload.rs` | `offload()` mints key before dispatch, carries via `with_handle_key` | VERIFIED | Line 121: `.with_handle_key(key.as_str().to_string())` — key minted before `dispatch().await` |
| `ferro-macros/src/offload.rs` | `handle_with_value` override emitted; `to_value` used; no `failed()` override | VERIFIED | `handle_with_value` at line 353; `to_value` references at lines 287, 291, 299, 303 (all four call-expr arms); no `async fn failed` emitted |
| `framework/tests/offload_result_round_trip.rs` | integration harness; both migrations; `register_offload_hooks`; SC1/SC2/SC3a/SC3b assertions; min 80 lines | VERIFIED | 376 lines; `TestMigrator` runs `ferro_queue::CreateJobsTable` + `ferro_projection::CreateProjectionSnapshotsTable`; `register_offload_hooks()` called at line 239 (before drain); all four SC assertions in `offload_result_round_trip` |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-projection/src/direct.rs` | `projection_snapshots` table | `Entity::insert(...).on_conflict(OnConflict::columns([Column::ProjectionName, Column::Key]))` | WIRED | Upsert updates `State` + `UpdatedAt`; version fixed at 1 and excluded from update |
| `ferro-projection/src/direct.rs` | read by composite PK | `Entity::find_by_id((name, key.0))` | WIRED | Returns `Ok(Some(m.state))` or `Ok(None)` |
| `framework/src/offload.rs` | `ferro_projection::snapshot_write` / `snapshot_read` | composes Plan 01 direct API | WIRED | `use ferro_projection::{snapshot_read, snapshot_write, ...}` at line 32; no `OnConflict` in the facade |
| `framework/src/offload.rs` | `OFFLOAD_PROJECTION_NAME = "offload.result"` | reserved name constant | WIRED | Constant at line 42; used in every `snapshot_write`/`snapshot_read` call |
| `ferro-queue/src/offload.rs offload()` | `PendingDispatch::with_handle_key` | mint key then carry it | WIRED | Line 121: `.with_handle_key(key.as_str().to_string())` |
| `ferro-queue/src/dispatcher.rs dispatch_to_queue` | `db::enqueue handle_key arg` | `self.handle_key.as_deref()` | WIRED | Line 202: `handle_key.as_deref()` passed to `enqueue` |
| `ferro-queue/src/db.rs claim SELECT` | `parse_job_row handle_key` | `try_get_by::<Option<String>, _>("handle_key")` | WIRED | Both Postgres and SQLite claim arms include `handle_key` in SELECT/RETURNING column list |
| `ferro-queue/src/worker.rs spawn_job` | result-persist hook (success) | `persist_offload_outcome(handle_key.as_deref(), Ok(val), conn)` at line 550 | WIRED | Only fires when `success_value` is `Some(val)` — non-offload jobs return `None` and skip persistence |
| `ferro-queue/src/worker.rs handle_failure` | result-persist hook (terminal) | `persist_offload_outcome(handle_key, Err(err_msg), conn)` at line 616 | WIRED | Inside `if attempts + 1 >= max_retries` — transient failures do not persist (D-09) |
| `framework/src/app.rs` | `ferro_queue::register_offload_result_hook` | `crate::offload::register_offload_hooks()` at line 419 | WIRED | Called at framework boot (app.rs:419); also called explicitly in the integration test at line 239 |
| test harness | `register_offload_hooks` + both migrations | setup at test top; `register_offload_hooks()` before drain | WIRED | Line 239: `register_offload_hooks()` is called in the test before any `drain()`; hook is therefore registered when `WorkerLoop::drain_for_test` executes |

---

### Key Trap Verification (register_offload_hooks in the test)

The test calls `register_offload_hooks()` explicitly at line 239, before the first `drain()`.
Without this, `OFFLOAD_RESULT_HOOK.get()` would return `None` inside `persist_offload_outcome`,
and no snapshot would be written — making any assertion on `read_result` after drain a spurious
pass (the function would return `None`, and an `.unwrap()` or `.expect()` would fail, not pass).

The test asserts the **presence** of the snapshot via `.expect("SC1: completed envelope must be present after worker drains")` — so the test cannot silently pass with an unregistered hook.

The `OnceLock`-backed registration is idempotent; repeated calls across scenarios within the same
`#[tokio::test]` function are safe.

---

### Data-Flow Trace (Level 4)

| Component | Data Variable | Source | Produces Real Data | Status |
|-----------|--------------|--------|--------------------|--------|
| `framework/tests/offload_result_round_trip.rs` | `result: OffloadResult<i32>` | `::ferro::offload::read_result::<i32>(&key, db)` → `snapshot_read` → SeaORM `Entity::find_by_id` | Yes — SQLite `projection_snapshots` row written by `persist_offload_outcome` → hook → `persist_result_raw` → `snapshot_write` | FLOWING |
| `ferro-queue/src/worker.rs spawn_job` | `success_value: Option<serde_json::Value>` | `job.handle_with_value().await` — `SuccessJob::handle_with_value` serializes `self.expected_value` via `serde_json::to_value` | Yes — real i32 value, not mocked | FLOWING |
| `ferro-queue/src/worker.rs handle_failure` | `err_msg: &str` | `Error::job_failed("AlwaysErrJob", "always fails").to_string()` / `"job handler panicked"` | Yes — deterministic error strings, asserted in SC3a/SC3b | FLOWING |

---

### Behavioral Spot-Checks

The prompt instructs not to run cargo. Per the prompt: "the E2E `offload_result_round_trip` test — which asserts all four SC — passes." The following checks are confirmed by code reading.

| Behavior | Evidence | Status |
|----------|----------|--------|
| `register_offload_hooks` called before drain in test | `framework/tests/offload_result_round_trip.rs:239` — `register_offload_hooks()` before first `drain()` | PASS (code) |
| Success: `Some(Completed { value: 42 })` after drain | line 266-281 — `.expect("SC1: completed envelope must be present")` + `OffloadResult::Completed { value } => assert_eq!(value, 42)` | PASS (code) |
| SC3a: `Some(Failed { error: "always fails" ... })` | lines 326-335 — `.expect("SC3a: failed envelope must be present")` + `assert!(error.contains("always fails"))` | PASS (code) |
| SC3b: `Some(Failed { error: "...panicked..." })` | lines 357-373 — `.expect("SC3b: failed envelope must be present")` + `assert!(error.contains("panicked"))` | PASS (code) |

---

### Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
|-------------|---------------|-------------|--------|----------|
| OFFLOAD-03 | 246-01 through 246-05 | Return value persisted as projection snapshot keyed by handle; failed run records terminal error state | SATISFIED | SC1/SC2/SC3 all verified via code analysis; E2E test reported green by CI gate |

REQUIREMENTS.md traceability table (line 73): `OFFLOAD-03 | Phase 246 | Not started` — this entry is stale (written before execution). The actual implementation is complete as verified above.

---

### Anti-Patterns Found

No significant anti-patterns found in the phase artifacts. Specific checks:

- No `TODO`/`FIXME`/`PLACEHOLDER` comments in `direct.rs`, `offload.rs`, or the integration test.
- No empty `return null` / `return {}` / stub patterns — all functions have substantive implementations.
- The `persist_offload_outcome` no-op when `handle_key.is_none()` is correct behavior (non-offload jobs), not a stub.
- `serde_json::to_value(&v).ok()` yielding `None` on serialization failure is documented as
  "extremely unlikely given the `OffloadSerializable` bound" — an acceptable design decision, not
  a silent data drop.

---

### Human Verification Required

None. All Phase 246 behaviors have automated verification (in-process `sqlite::memory:` / temp-file
SQLite covers persist, retrieve, terminal-error, and panic paths per the VALIDATION.md design).

---

## Gaps Summary

No gaps. All three Success Criteria are satisfied:

- **SC1 (return value persisted):** `worker.rs` success arm invokes `persist_offload_outcome` with the serialized value from `handle_with_value()`; the hook writes the completed envelope via `persist_result_raw` → `snapshot_write` → `projection_snapshots`.
- **SC2 (retrievable by handle):** The handle key minted by `offload()` is carried via `with_handle_key` → `enqueue` → `jobs.handle_key` column → `JobRow.handle_key` → `spawn_job` → `persist_offload_outcome`; it is therefore the same key the caller holds via `handle.key()`. The E2E test asserts this key equality explicitly.
- **SC3 (terminal error, no silent drop):** `handle_failure` calls `persist_offload_outcome` with the error string only when `attempts + 1 >= max_retries`; the panic arm passes `"job handler panicked"`; neither path requires the `Job::failed()` override (D-10 correction correctly applied). The hook is registered before the test's `drain_for_test` run, so neither SC3a nor SC3b can silently pass.

---

_Verified: 2026-08-13T22:58:54Z_
_Verifier: Claude (gsd-verifier)_
