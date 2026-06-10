---
phase: 195-close-the-loop-by-default
plan: "04"
subsystem: ferro-mcp
tags: [mcp, projection, checkpoint, ambient, introspection]
completed: "2026-06-10T00:49:33Z"

dependency_graph:
  requires: [195-01, 195-03]
  provides: [CHK-08-surface]
  affects: [ferro-mcp/src/tools/projection_coverage.rs, ferro-mcp/src/tools/application_info.rs, ferro-mcp/src/service.rs]

tech_stack:
  patterns:
    - read_ambient_status called from two consumer tools (cache-only, stale-ok)
    - sub-struct rollup pattern mirroring ClaudeCodeSkillsStatus

key_files:
  modified:
    - ferro-mcp/src/tools/projection_coverage.rs
    - ferro-mcp/src/tools/application_info.rs
    - ferro-mcp/src/service.rs
    - ferro-mcp/src/tools/checkpoint_projection.rs

decisions:
  - "checkpoint_status keyed on projection function name (proj.name), not model name — Pitfall 4"
  - "read_ambient_status is pub(crate) called via crate::tools::checkpoint_projection:: path"
  - "removed #[allow(dead_code)] from read_ambient_status once callers were added"
  - "total_projections == clean + failing + unverified enforced by test invariant"

metrics:
  duration_minutes: 25
  tasks_completed: 3
  tasks_total: 3
  files_modified: 4
  tests_added: 5
  tests_total: 299
---

# Phase 195 Plan 04: Ambient Checkpoint Status Surface Summary

Surface cached verification debt in two read-only MCP introspection tools: per-model `checkpoint_status` in `projection_coverage` and a `projection_checkpoint` rollup in `application_info`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | projection_coverage.checkpoint_status | c56ef537 | projection_coverage.rs |
| 2 | application_info.projection_checkpoint rollup | c36bcc57 | application_info.rs |
| 3 | Update tool descriptions in service.rs | 0ccc8083 | service.rs, checkpoint_projection.rs |

## What Was Built

### Task 1 — `ModelCoverage.checkpoint_status`

Added `checkpoint_status: String` to `ModelCoverage`. In the matched branch (projection exists), populated by calling `crate::tools::checkpoint_projection::read_ambient_status(project_root, &proj.name).to_string()` — keyed on the projection function name (e.g. `booking_service`), not the model name. In the else branch (no projection), always `"unverified"`.

Three new tests cover the three possible states: cache file with `"failing"`, missing cache file, and no projection at all.

### Task 2 — `ProjectionCheckpointSummary` + `ApplicationInfo.projection_checkpoint`

Added `ProjectionCheckpointSummary { total_projections, clean, failing, unverified }` struct (mirroring `ClaudeCodeSkillsStatus` derive pattern). Added `projection_checkpoint: ProjectionCheckpointSummary` field to `ApplicationInfo`. Added `check_projection_checkpoint` helper that iterates `list_projections::execute(project_root, None)` and tallies `read_ambient_status` per projection function name.

Two new tests: mixed-state rollup (1 clean + 1 failing + 1 unverified = total 3) and empty project (all zeros). The invariant `total_projections == clean + failing + unverified` is asserted directly.

### Task 3 — Tool description updates

Updated `application_info` and `projection_coverage` MCP tool descriptions in `service.rs` to document the new fields. Removed `#[allow(dead_code)]` from `read_ambient_status` in `checkpoint_projection.rs` since it now has two callers.

## Deviations from Plan

**1. [Rule 2 - Missing critical cleanup] Removed `#[allow(dead_code)]` from `read_ambient_status`**
- **Found during:** Task 3
- **Issue:** The attribute comment said "Plans 03/04 add the callers" — Plan 04 adds both callers, so the allow was no longer needed and would be misleading.
- **Fix:** Removed the `#[allow(dead_code)]` attribute in the same commit as the tool description updates.
- **Files modified:** `ferro-mcp/src/tools/checkpoint_projection.rs`
- **Commit:** 0ccc8083

No other deviations. Plan executed as specified.

## Verification

- `grep -n "checkpoint_status: String" ferro-mcp/src/tools/projection_coverage.rs` → line 39
- `grep -n "read_ambient_status" ferro-mcp/src/tools/projection_coverage.rs` → line 104 (keyed on `&proj.name`)
- `grep -c "run_for" ferro-mcp/src/tools/projection_coverage.rs` → 0
- `grep -n "pub struct ProjectionCheckpointSummary" ferro-mcp/src/tools/application_info.rs` → line 58
- `grep -n "pub projection_checkpoint: ProjectionCheckpointSummary" ferro-mcp/src/tools/application_info.rs` → line 23
- `grep -c "run_for" ferro-mcp/src/tools/application_info.rs` → 0
- `grep -n "checkpoint_status" ferro-mcp/src/service.rs` → line 1630 (description)
- `grep -n "projection_checkpoint" ferro-mcp/src/service.rs` → line 408 (description)
- `cargo test -p ferro-mcp` → 299 passed
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings` → clean

## Self-Check: PASSED
