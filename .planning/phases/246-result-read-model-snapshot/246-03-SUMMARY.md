---
phase: 246-result-read-model-snapshot
plan: "03"
subsystem: ferro-queue
tags: [offload, handle-key, queue, migration, dispatcher]
dependency_graph:
  requires: [246-01, 246-02]
  provides: [handle_key_on_jobs_table, handle_key_on_job_row, with_handle_key_builder, offload_mint_before_dispatch]
  affects: [ferro-queue]
tech_stack:
  added: []
  patterns: [nullable-column-mirrors-tenant_id, builder-consuming-mut-self, extract-before-move]
key_files:
  created: []
  modified:
    - ferro-queue/src/migration.rs
    - ferro-queue/src/db.rs
    - ferro-queue/src/dispatcher.rs
    - ferro-queue/src/offload.rs
    - ferro-queue/tests/shutdown.rs
    - ferro-queue/tests/race_claim_sqlite.rs
    - ferro-queue/tests/race_claim_postgres.rs
decisions:
  - "handle_key extracted to local variable before self.job move in dispatch_to_queue (extract-before-move pattern)"
  - "handle_key: Option<String> in PendingDispatch (not Option<&str>) to allow clone before the move"
  - "Amended CreateJobsTable migration rather than creating AddHandleKeyToJobs (pre-production, safe)"
metrics:
  duration: "~15 minutes"
  completed: "2026-08-13"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 7
requirements: [OFFLOAD-03]
---

# Phase 246 Plan 03: Handle Key Propagation to Worker — Summary

Thread the offload handle key from the caller through the enqueue→claim boundary so the worker can write the result snapshot under the correct key. Mirrors the existing `tenant_id: Option<i64>` propagation path exactly across all four crate files.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add handle_key to jobs schema + JobRow + parse_job_row | e51c8400 | migration.rs, db.rs |
| 2 | Thread handle_key through enqueue + both claim SELECT arms | 0686d426 | db.rs, shutdown.rs, race_claim_sqlite.rs, race_claim_postgres.rs |
| 3 | with_handle_key() on PendingDispatch + mint-before-dispatch in offload() | cd5f17d0 | dispatcher.rs, offload.rs |

## Final enqueue Signature

```rust
#[allow(clippy::too_many_arguments)]
pub async fn enqueue(
    conn: &DatabaseConnection,
    queue: &str,
    job_type: &str,
    payload: &str,
    max_retries: u32,
    idempotency_key: Option<&str>,
    tenant_id: Option<i64>,
    handle_key: Option<&str>,   // NEW — position after tenant_id
    available_at: DateTime<Utc>,
) -> Result<(), Error>
```

`handle_key` sits immediately after `tenant_id`, matching the column order in the `jobs` table and the `JobRow` struct.

## JobRow.handle_key Field

```rust
/// Optional offload handle key (UUID string). Present for jobs dispatched via Offloadable::offload().
pub handle_key: Option<String>,
```

Parsed in `parse_job_row` via `try_get_by::<Option<String>, _>("handle_key")`, mirroring the `tenant_id` parse. Available to `spawn_job` in the worker after claim.

## with_handle_key Builder

```rust
/// Attach an offload handle key so the worker can persist the result under it.
pub fn with_handle_key(mut self, key: String) -> Self {
    self.handle_key = Some(key);
    self
}
```

Mirrors `for_tenant()` exactly. Called in `offload()` as `.with_handle_key(key.as_str().to_string())` before the dispatch.

## Payload Unchanged (D-05 Preserved)

The serializable `Job` struct payload remains exactly the method's non-`self` parameters. The handle key travels exclusively as a `handle_key` column on the `jobs` table row — it is never added to the JSON payload. The `OffloadSerializable` bound on `Output` is unaffected.

## Note for Plan 04

`job_row.handle_key: Option<String>` is now available in `spawn_job` (via `let handle_key = job_row.handle_key.clone()`) for the result write-back. The success path can call `::ferro::offload::persist_result(key, value, conn)` and the terminal-error path can call `::ferro::offload::persist_error(key, msg, conn)` — both from the `spawn_job` closure after the handler future resolves, bypassing `Job::failed()` (which is not called in the real async worker path per RESEARCH §RQ4).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated enqueue call sites in test helpers**

- **Found during:** Task 2/3 compile run
- **Issue:** `ferro-queue/tests/shutdown.rs`, `race_claim_sqlite.rs`, `race_claim_postgres.rs`, and `db.rs` inline tests all called `enqueue()` with 8 arguments after the signature was extended to 9. The compiler caught all four sites.
- **Fix:** Added `None` as the `handle_key` argument in every existing call site (no handle key for manually dispatched test jobs).
- **Files modified:** ferro-queue/tests/shutdown.rs, tests/race_claim_sqlite.rs, tests/race_claim_postgres.rs, src/db.rs (inline test)
- **Commit:** 0686d426

**2. [Rule 3 - Blocking] Extract-before-move for handle_key in dispatch_to_queue**

- **Found during:** Task 3 implementation
- **Issue:** `self.job` is consumed for `payload` and `job_type` via `serde_json::to_string(&self.job)` and `self.job.name()`. A direct `self.handle_key.as_deref()` call after those moves would fail the borrow checker.
- **Fix:** Added `let handle_key = self.handle_key.clone();` before the job consumption, then passed `handle_key.as_deref()` to `enqueue`. This is a standard extract-before-move pattern; the plan's acceptance criterion (`self.handle_key.as_deref()`) describes the intent, which the local variable achieves identically.
- **Files modified:** ferro-queue/src/dispatcher.rs
- **Commit:** cd5f17d0

## Test Evidence

```
cargo test -p ferro-queue

running 51 tests
test dispatcher::tests::test_with_handle_key_sets_field ... ok
test offload::tests::handle_key_is_uuid_v4 ... ok
test offload::tests::handle_round_trips_with_non_serializable_t ... ok
test db::tests::idempotency_dedup ... ok
test db::tests::claim_returns_pending_job ... ok
test migration::tests::migration_creates_jobs_table ... ok
... (45 more tests, all ok)

test result: ok. 51 passed; 0 failed; 0 ignored

Running tests/offload_round_trip.rs
test offload_round_trip_sync_mode ... ok
test offload_result_err_maps_to_job_failure ... ok
test offload_job_auto_registers_via_inventory ... ok
test result: ok. 3 passed; 0 failed

Running tests/race_claim_sqlite.rs
test two_workers_claim_each_job_exactly_once ... ok
test result: ok. 1 passed; 0 failed

Running tests/shutdown.rs
test graceful_shutdown_requeues_claimed_jobs ... ok
test result: ok. 1 passed; 0 failed
```

`cargo clippy -p ferro-queue --all-targets -- -D warnings`: clean (no warnings).

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundary crossings introduced. The `handle_key` column is an internal `jobs` table field; `enqueue`/`claim` use `Statement::from_sql_and_values` (parameterized) — no string-concatenated SQL. T-246-tamper mitigation preserved exactly.

## Self-Check: PASSED

- [x] `ferro-queue/src/migration.rs` — HandleKey in enum + ColumnDef (2 occurrences)
- [x] `ferro-queue/src/db.rs` — pub handle_key: Option<String> field, parse, enqueue param, 2 INSERT arms, 2 claim SELECTs (12 occurrences)
- [x] `ferro-queue/src/dispatcher.rs` — handle_key field, None init, with_handle_key(), local extract + as_deref() call, test
- [x] `ferro-queue/src/offload.rs` — with_handle_key(key.as_str().to_string()) before dispatch
- [x] Commits e51c8400, 0686d426, cd5f17d0 exist in git log
- [x] `cargo test -p ferro-queue` green (56 tests total across all test binaries)
- [x] No ferro-projection dependency added to ferro-queue Cargo.toml
