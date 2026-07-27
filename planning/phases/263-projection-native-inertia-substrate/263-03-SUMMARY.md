---
phase: 263-projection-native-inertia-substrate
plan: "03"
subsystem: framework/projection_read
tags: [framework, data-query, tenant-scoping, mcp, relocation, refactor]
dependency_graph:
  requires: [263-02]
  provides: [framework::projection_read, SUBST-03-data-core]
  affects: [ferro-mcp-server/dispatch, ferro-mcp-server/schema, framework/lib]
tech_stack:
  added: [framework::projection_read, ProjectionReadError, ProjectionReadResult, DispatchResult]
  patterns: [thin-delegation-wrapper, re-export-for-back-compat, feature-gated-module]
key_files:
  created:
    - framework/src/projection_read.rs
  modified:
    - framework/src/lib.rs
    - ferro-mcp-server/src/dispatch.rs
    - ferro-mcp-server/src/schema.rs
decisions:
  - schema.rs re-export (not delete-and-repoint): multiple callers in schema.rs itself make re-export the minimal diff
  - ProjectionReadError defined locally in projection_read.rs (thiserror); maps 1:1 back to crate::Error in the wrapper
  - Tests: match arms required `other => panic!()` arm since ProjectionReadError is a non-exhaustive-by-code two-variant enum
metrics:
  duration_seconds: 437
  completed_date: "2026-07-27"
  tasks_completed: 2
  files_modified: 4
requirements: [SUBST-03]
---

# Phase 263 Plan 03: projection_read Relocation Summary

Tenant-scoped data query (`dispatch` + all pure helpers) relocated from `ferro-mcp-server` into `framework::projection_read`. `ferro-mcp-server::dispatch` is now a thin delegation wrapper. The query is now reachable by the Inertia wave (Plan 04) without any `ferro-inertia → ferro-mcp-server` edge.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create framework::projection_read | a6e4c078 | framework/src/projection_read.rs, framework/src/lib.rs |
| 2 | Reduce ferro-mcp-server::dispatch to thin wrapper | fe551550 | ferro-mcp-server/src/dispatch.rs, ferro-mcp-server/src/schema.rs |

## Relocated Helpers

All moved verbatim from `ferro-mcp-server/src/dispatch.rs` into `framework/src/projection_read.rs`:

| Helper | Type | Description |
|--------|------|-------------|
| `MAX_LIMIT` | const `u64 = 100` | Hard cap on rows per request; enforced regardless of caller |
| `MAX_OFFSET` | const `u64 = i64::MAX as u64` | Guard against u64→i64 wrap on OFFSET |
| `placeholder()` | fn | Backend-aware SQL parameter placeholder (`$N` for Postgres, `?` otherwise) |
| `json_to_sea_value()` | fn | Maps `serde_json::Value` to `sea_orm::Value` for bound parameters |
| `split_op_key()` | fn | Splits `"field__op"` on the last `__` (rfind); enables `total__gt` syntax |
| `rows_to_json()` | fn | Converts `Vec<sea_orm::QueryResult>` to `Vec<serde_json::Value>` |
| `is_filter_field()` | fn (from schema.rs) | 5-gate equality-filter allowlist for FieldDef |
| `is_range_filter_field()` | fn (from schema.rs) | 4-gate + DataType range-filter allowlist for FieldDef |

`dispatch` itself also moved, with return type changed from `crate::Result<DispatchResult>` to `ProjectionReadResult<DispatchResult>`.

## ProjectionReadError Variants

```rust
pub enum ProjectionReadError {
    InvalidFilter(String),   // unknown field, bad op, empty __in array, missing tenant context
    Database(String),        // sea-orm query failure
}
pub type ProjectionReadResult<T> = Result<T, ProjectionReadError>;
```

Maps 1:1 back to `ferro-mcp-server::Error::InvalidFilter` / `Error::Database` in the thin wrapper.

## schema.rs Decision: Re-export (not delete-and-repoint)

`ferro-mcp-server/src/schema.rs` had **multiple callers** of both `is_filter_field` and `is_range_filter_field` throughout the file (lines 121, 133, 156, plus test assertions). Re-exporting from `ferro_rs::projection_read` required a two-line change vs. updating ~10 call sites. The re-export preserves the MCP surface unchanged with the smallest diff.

```rust
// schema.rs — before (69 lines of function bodies)
pub fn is_filter_field(field: &FieldDef) -> bool { ... }
pub fn is_range_filter_field(field: &FieldDef) -> bool { ... }

// schema.rs — after (2 lines)
pub use ferro_rs::projection_read::{is_filter_field, is_range_filter_field};
```

## cargo tree No-New-Cycle Evidence

```
ferro-mcp-server
├── ferro-rs v0.2.102 (framework)       ← existing acyclic edge, unchanged
```

The `ferro-mcp-server → ferro-rs (framework)` edge was already present before this plan. No new dependency edge was added. `ferro-inertia` (Plan 04) will depend on `ferro-rs::projection_read` via the same existing `framework` edge — no cycle possible.

## Test Results

| Scope | Command | Result |
|-------|---------|--------|
| framework projection_read | `cargo test -p ferro-rs --features projections projection_read` | 13/13 pass |
| ferro-mcp-server full suite | `cargo test -p ferro-mcp-server` | 64/64 pass (45 unit + 19 integration) |

The 13 tenant-scoping tests (`tenant_scoping`, `tenant_isolation`, `tenant_fail_closed`, `non_tenant_unscoped`, plus filter/sort/range/in/soft-delete tests) now live in `framework::projection_read` and travel with the code.

## Deviations from Plan

**1. [Rule 1 - Bug] Test match arms required exhaustive coverage**
- **Found during:** Task 1 first compile
- **Issue:** The original `ferro-mcp-server` tests used `crate::Error` which is matched with a wildcard (`other => panic!`). The relocated tests in `framework::projection_read` used single-arm `match` on `ProjectionReadError`, which Rust rejected as non-exhaustive (5 errors).
- **Fix:** Added `other => panic!("expected InvalidFilter, got: {other:?}")` to all 5 single-variant test match arms.
- **Files modified:** `framework/src/projection_read.rs`
- **Commit:** a6e4c078

**2. [Rule 1 - Bug] Crate name is `ferro-rs` / `ferro_rs`, not `framework`**
- **Found during:** Task 2 first compile
- **Issue:** The plan's code excerpts used `framework::projection_read` but the published crate name is `ferro-rs` (Rust path `ferro_rs`). Applying the excerpts verbatim caused 5 unresolved-crate errors.
- **Fix:** Changed all `framework::projection_read` → `ferro_rs::projection_read` in `dispatch.rs`, `schema.rs`, and `lib.rs`.
- **Files modified:** `ferro-mcp-server/src/dispatch.rs`, `ferro-mcp-server/src/schema.rs`
- **Commit:** fe551550

## Known Stubs

None. All code paths are complete and tested.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. This is a pure relocation — the tenant-scoping, filter allowlisting, and MAX_LIMIT=100 cap are preserved verbatim. The existing T-263-06/07/08/09 mitigations remain intact (grep-verified by acceptance criteria).

## Self-Check: PASSED

| Item | Status |
|------|--------|
| framework/src/projection_read.rs | FOUND |
| 263-03-SUMMARY.md | FOUND |
| commit a6e4c078 (Task 1) | FOUND |
| commit fe551550 (Task 2) | FOUND |
