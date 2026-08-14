---
phase: 247-read-model-delta-broadcast-streaming
plan: "01"
subsystem: framework/offload
tags: [offload, queue, projection, broadcast, serde, pending, redaction]
completed: "2026-08-14"
duration: "513s"

dependency_graph:
  requires:
    - "246-result-read-model-snapshot (snapshot_write, snapshot_read, ProjectionError, ProjectionKey)"
    - "ferro_projection::CreateProjectionSnapshotsTable (migration)"
  provides:
    - "OffloadResult::Pending variant (serde tag: {\"status\":\"pending\"})"
    - "persist_pending() — framework::offload::persist_pending"
    - "read_result_redacted() — framework::offload::read_result_redacted"
  affects:
    - "247-02 (broadcast hook consumes OFFLOAD_BROADCASTER static, Pending variant)"
    - "247-03 (integration tests consume persist_pending, read_result_redacted)"

tech_stack:
  added: []
  patterns:
    - "Internally-tagged serde enum extended with a fieldless Pending variant (backward-compatible)"
    - "Redact-at-read-back pattern: raw error in snapshot, fixed marker at the client boundary"
    - "Non-fatal async helper: returns ProjectionError, callers tracing::warn! and continue"

key_files:
  created: []
  modified:
    - "framework/src/offload.rs"
    - "framework/src/lib.rs"
    - "framework/tests/offload_result_round_trip.rs"

decisions:
  - "Pending variant is fieldless — no value field means it cannot smuggle data through the serde boundary (T-247-input-validation mitigation)"
  - "read_result_redacted delegates to read_result then redacts the Failed arm in a single match; no new DB path"
  - "Fixed marker string is \"terminal error\" (non-sensitive, opaque, consistent with D-05)"
  - "persist_pending placed between persist_error and read_result in source order (symmetric with persist_error in shape)"
  - "Integration test exhaustive-match arms updated for Pending (Rule 1 auto-fix — new variant broke existing matches)"

metrics:
  duration: "513s"
  completed: "2026-08-14"
  tasks_completed: 3
  files_modified: 3
---

# Phase 247 Plan 01: Pending Variant, persist_pending, and read_result_redacted Summary

Three data-layer primitives added to `framework/src/offload.rs` that Plans 02 and 03 depend on: `OffloadResult::Pending` (D-07), `persist_pending` (D-07), and `read_result_redacted` (D-05/D-10). All three provable in isolation by unit tests; downstream plans build on a verified foundation.

## What Was Built

### `OffloadResult::Pending` (Task 1)

Added a third variant to the `OffloadResult<T>` enum immediately after `Failed`:

```rust
Pending,   // fieldless — serde-tags as {"status":"pending"}
```

The enum already carries `#[serde(tag = "status", rename_all = "snake_case")]`, so the new variant serializes as `{"status":"pending"}` with no fields. Existing completed/failed rows are unaffected: their tags still match their arms.

Module doc envelope-shape block updated to include `{ "status": "pending" }`. Security notes updated to reference `read_result_redacted` as the Phase 247 client-facing path.

### `persist_pending` (Task 2)

```rust
pub async fn persist_pending(
    handle_key: &str,
    db: &DatabaseConnection,
) -> Result<(), ProjectionError>
```

Writes `serde_json::json!({ "status": "pending" })` via `snapshot_write` at `(OFFLOAD_PROJECTION_NAME, handle_key)`. Mirrors `persist_error` in shape. Non-fatal contract: returns `ProjectionError`, callers log and continue. Resolves the Phase 246 D-08 deferred unknown-handle-vs-not-done ambiguity: `read_result` now returns `None` for an unknown handle and `Some(Pending)` for a pending one.

`read_result` doc updated to describe the `None` vs `Some(Pending)` distinction.

### `read_result_redacted` (Task 3)

```rust
pub async fn read_result_redacted<T: OffloadSerializable>(
    handle_key: &str,
    db: &DatabaseConnection,
) -> Result<Option<OffloadResult<T>>, ProjectionError>
```

Delegates to `read_result`, then redacts the `Failed` arm by replacing the raw error with the fixed marker `"terminal error"`. Completed values and the `Pending` marker pass through unchanged. No new DB path. This is the client-safe read-back for the subscribe → read-back → await pattern (D-09); the raw error remains in the snapshot and worker logs for authorized/server-side access via `read_result`.

`framework/src/lib.rs` offload module doc updated to list `persist_pending` and `read_result_redacted` alongside the existing public surface.

## Tests Added

| Test | File | What it proves |
|------|------|----------------|
| `offload_result_pending_round_trip` | `framework/src/offload.rs` | `{"status":"pending"}` → `OffloadResult::Pending`; existing completed/failed envelopes unaffected (A3) |
| `offload_pending_round_trip` | `framework/src/offload.rs` | `persist_pending("k1")` → `read_result` = `Some(Pending)`; unknown handle = `None` (D-07 distinction) |
| `read_result_redacted_hides_error` | `framework/src/offload.rs` | Failed: `error == "terminal error"` AND `error != "sensitive-secret-value"`; Completed: value preserved; Pending: passes through; None: unknown handle |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Exhaustive match arms in integration test broken by new Pending variant**

- **Found during:** Task 1 (first compile after adding the variant)
- **Issue:** `framework/tests/offload_result_round_trip.rs` had three exhaustive `match result` arms (SC1, SC3a, SC3b) with no arm for `OffloadResult::Pending`. Adding the variant caused `E0004` (non-exhaustive patterns) at all three sites.
- **Fix:** Added `OffloadResult::Pending => panic!("SC1/SC3a/SC3b: expected ..., got Pending")` to each match arm.
- **Files modified:** `framework/tests/offload_result_round_trip.rs`
- **Commit:** `8ebbfd56`

No other deviations. All three tasks executed exactly as specified.

## Threat Surface Scan

The plan's `<threat_model>` covers both relevant threats and both mitigations are implemented:

| Threat ID | Disposition | Status |
|-----------|-------------|--------|
| T-247-info-disclosure | mitigate | `read_result_redacted` never returns the raw error; `read_result_redacted_hides_error` asserts `error != "sensitive-secret-value"` |
| T-247-input-validation | mitigate | `Pending` is fieldless — no new deserialization surface; same `serde_json::from_value` strict path |

No new threat surface introduced beyond what the plan's threat model covers.

## Known Stubs

None. All three primitives are fully wired: `persist_pending` writes a real row, `read_result` reads it back as `Some(Pending)`, `read_result_redacted` correctly redacts. No placeholder data or hardcoded returns flow to any rendering surface.

## Self-Check: PASSED

| Item | Result |
|------|--------|
| `framework/src/offload.rs` exists | FOUND |
| `framework/src/lib.rs` exists | FOUND |
| `framework/tests/offload_result_round_trip.rs` exists | FOUND |
| Commit `8ebbfd56` (Task 1) | FOUND |
| Commit `2bce5354` (Task 2) | FOUND |
| Commit `1d48bb34` (Task 3) | FOUND |
| `Pending,` inside `OffloadResult` enum (line 70) | FOUND |
| `pub async fn persist_pending` (line 142) | FOUND |
| `pub async fn read_result_redacted` (line 195) | FOUND |
| `offload_result_pending_round_trip` test (line 332) | FOUND |
| `offload_pending_round_trip` test (line 385) | FOUND |
| `read_result_redacted_hides_error` test (line 413) | FOUND |
