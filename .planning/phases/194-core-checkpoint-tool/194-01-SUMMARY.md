---
phase: 194-core-checkpoint-tool
plan: "01"
subsystem: ferro-mcp
tags: [checkpoint, mcp-tool, output-types, seam2, validation]
dependency_graph:
  requires: []
  provides: [checkpoint_projection_module, output_contract_types, validate_name, fixture_helpers]
  affects: [ferro-mcp/src/tools/mod.rs]
tech_stack:
  added: []
  patterns: [snake_case_serde_enum, pub_crate_inner_fn, allow_dead_code_on_forward_fixtures]
key_files:
  created:
    - ferro-mcp/src/tools/checkpoint_projection.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - Cargo.lock
decisions:
  - "Implemented full seam 2 logic (field_to_column_seam), aggregation, and cache write in the same file as the output types — cleaner than splitting across tasks since the contract was already locked."
  - "Added #[allow(dead_code)] to project_with_projection and add_model fixture helpers to silence clippy until Plans 02/03 consume them."
metrics:
  duration: "235s"
  completed: "2026-06-09"
  tasks_completed: 2
  files_created: 1
  files_modified: 2
---

# Phase 194 Plan 01: Checkpoint Projection Foundation Summary

Four public output-contract types, path-traversal-safe name validation, full seam 2 field-to-column logic with stub seams 1/3/4/5, verdict aggregation, cache write, and three contract tests — all in one new module registered in mod.rs.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create module with public output types and name validation | 5e3d508c | ferro-mcp/src/tools/checkpoint_projection.rs, ferro-mcp/src/tools/mod.rs |
| 2 | Add test scaffold with fixture helpers and contract-shape + not_checked tests | 96320c2e | ferro-mcp/src/tools/checkpoint_projection.rs |

## What Was Built

`ferro-mcp/src/tools/checkpoint_projection.rs` contains:

- **Four public output types** (D-07 locked shape): `Finding`, `SeamStatus`, `SeamResult`, `Verdict`
- **`SeamStatus` four-variant enum** with `#[serde(rename_all = "snake_case")]` — wire values `pass`, `warn`, `fail`, `not_checked`
- **`validate_name`** — path-traversal guard accepting only `[a-zA-Z0-9_-]`
- **`execute` / `run_for`** — public entry point + testable timestamp-injected inner function
- **`field_to_column_seam`** — seam 2: reconstruction, D-06 completeness check, model resolution, field presence loop
- **`count_column_backed_builders`** — comment-stripped invocation count for all four D-05 builders
- **`aggregate_status` / `aggregate_next_steps`** — D-09/D-10 aggregation with fail>warn>pass and cap-10 dedup
- **`write_cache`** — D-11 `.ferro/checkpoints/{name}.json` write with `ambient_status` and `checked_at`
- **Stubs for seams 1/3/4/5** with `not_checked` and `reason: "not_implemented_phase_195"`
- **`#[cfg(test)]` module** with 3 contract tests and 2 reusable fixture helpers

`ferro-mcp/src/tools/mod.rs`: one `pub mod checkpoint_projection;` line added alphabetically.

## Verification

- `cargo build -p ferro-mcp` exits 0
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings` exits 0
- `cargo test -p ferro-mcp checkpoint_projection` — 3 passed, 0 failed:
  - `verdict_shape` (CHK-01)
  - `seamstatus_wire` (CHK-03)
  - `name_validation` (T-194-01 path traversal)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Implemented seam 2 and aggregation logic in Task 1**
- **Found during:** Task 1 implementation
- **Issue:** Plan structured Task 1 as types-only and Task 2 as tests-only, but the module needed implementation to compile cleanly and serve as a useful foundation.
- **Fix:** Wrote the full `run_for`, `field_to_column_seam`, `aggregate_*`, and `write_cache` implementations in Task 1 alongside the types, then Task 2 added only the `#[allow(dead_code)]` suppressors on the fixture helpers.
- **Files modified:** ferro-mcp/src/tools/checkpoint_projection.rs
- **Commit:** 5e3d508c

**2. [Rule 1 - Bug] dead_code clippy errors on forward-looking fixture helpers**
- **Found during:** Task 2 verification
- **Issue:** `project_with_projection` and `add_model` are reusable by Plans 02/03 but not called in Plan 01, causing `-D dead_code` clippy failures.
- **Fix:** Added `#[allow(dead_code)]` on both helpers — correct pattern for intentionally forward-looking test fixtures.
- **Files modified:** ferro-mcp/src/tools/checkpoint_projection.rs
- **Commit:** 96320c2e

## Known Stubs

- Seams 1, 3, 4, 5 are stub `SeamResult` entries with `status: NotChecked` and `reason: "not_implemented_phase_195"`. These are intentional — Phase 195 fills in the real logic.

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary changes. Path traversal threat T-194-01 is mitigated by `validate_name` (tested).

## Self-Check: PASSED
