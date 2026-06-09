---
phase: 194-core-checkpoint-tool
plan: "03"
subsystem: ferro-mcp
tags: [checkpoint, mcp-tool, aggregation, cache, service-registration, docs]
dependency_graph:
  requires: [194-01, 194-02]
  provides: [checkpoint_projection_mcp_tool, aggregate_status_tests, aggregate_next_steps_tests, cache_tests, checkpoint_docs]
  affects: [ferro-mcp/src/service.rs, ferro-mcp/src/tools/checkpoint_projection.rs, docs/src/agents/checkpoint-projection.md, docs/src/SUMMARY.md]
tech_stack:
  added: []
  patterns: [serde_flatten_on_verdict_in_cache, uninlined_format_args_in_tests, fixed_timestamp_injection_for_deterministic_tests]
key_files:
  created:
    - docs/src/agents/checkpoint-projection.md
  modified:
    - ferro-mcp/src/tools/checkpoint_projection.rs
    - ferro-mcp/src/service.rs
    - docs/src/SUMMARY.md
decisions:
  - "CacheEntry needed #[serde(flatten)] on the verdict field so status/projection/seams/next_steps appear at the top level alongside ambient_status and checked_at (D-11 spec)."
  - "All aggregate_status tests confirm CHK-03: not_checked never raises to fail, and all-not_checked resolves to pass."
  - "write_cache_direct tests the cache shape directly (bypassing inspect_projection routing); cache_write tests the run_for code path accepting either Ok or a not-found Err."
  - "Docs placed under docs/src/agents/ (new section) rather than docs/src/features/ — agent-facing tools have a different audience than framework features."
metrics:
  duration: "309s"
  completed: "2026-06-10"
  tasks_completed: 3
  files_created: 1
  files_modified: 3
---

# Phase 194 Plan 03: Tool Registration, Cache Tests, and Docs Summary

CHK-06 ranking/dedup/cap tests, D-11 cache shape fix and tests, T-194-01 traversal guard test, `checkpoint_projection` MCP tool registration in `service.rs`, and neutral architectural docs describing the verdict shape, field→column seam, coverage honesty, and the read-only contract.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | aggregate_status + aggregate_next_steps tests (CHK-03/CHK-06) | 085ec256 | ferro-mcp/src/tools/checkpoint_projection.rs |
| 2 | CacheEntry flatten fix + cache/run_for tests (D-11/T-194-01/CHK-01) | 397ec677 | ferro-mcp/src/tools/checkpoint_projection.rs |
| 3 | MCP tool registration + docs | 88607b83 | ferro-mcp/src/service.rs, docs/src/agents/checkpoint-projection.md, docs/src/SUMMARY.md |

## What Was Built

### Tests added to checkpoint_projection.rs (23 total, up from 12)

**Task 1 — aggregate_status (4 tests):**
- `aggregate_status_fail_wins_over_not_checked`: Fail + NotChecked → Fail
- `aggregate_status_warn_not_checked`: Warn + NotChecked → Warn
- `aggregate_status_pass_not_checked`: Pass + NotChecked → Pass (not_checked does not suppress Pass)
- `aggregate_status_all_not_checked_is_pass`: all NotChecked → Pass (CHK-03: never raises to Fail)

**Task 1 — aggregate_next_steps (3 tests):**
- `next_steps_ranked_deduped`: failures before warnings; seam-order within rank; D-10 format string
- `next_steps_dedup`: identical (subject, fix) across seams → one entry
- `next_steps_cap_at_10`: 12 distinct findings → exactly 10 entries

**Task 2 — cache and run_for (4 tests):**
- `write_cache_direct`: status/ambient_status/checked_at/projection at top level in JSON (validates flatten fix)
- `cache_write`: run_for writes cache file; validates required keys exist
- `cache_rejects_traversal`: `"../evil"` returns Err; no file written (T-194-01)
- `run_for_full_verdict`: Ok verdict has required shape keys; Err has meaningful message (CHK-01)

### service.rs changes

- `CheckpointProjectionParams { name: String }` struct added after `ValidateProjectionParams`
- `checkpoint_projection` handler registered via `#[tool]` in the `#[tool_router]` impl block
- Delegates to `tools::checkpoint_projection::execute`; error arm uses `e.replace('"', "\\\"")` for JSON-safe embedding

### docs/src/agents/checkpoint-projection.md

New page covering:
- When to use the tool
- Verdict shape table (status, seams, next_steps, Finding fields)
- SeamStatus value meanings including `not_checked`
- The field→column seam: what it checks, what it does not (presence only, not type compat)
- Coverage honesty: `not_checked` ≠ `pass`; listed with reason in every verdict
- Aggregate status logic (fail > warn > pass; not_checked never raises to fail)
- next_steps assembly (rank, seam-order, dedup, cap 10, D-10 format)
- Status cache: ambient_status derivation, checked_at, purpose
- Read-only contract

### docs/src/SUMMARY.md

New `# Agents` section added before `# Reference`, containing `[checkpoint_projection](agents/checkpoint-projection.md)`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] CacheEntry missing #[serde(flatten)] on verdict field**
- **Found during:** Task 2 test run — `write_cache_direct` and `cache_write` failed because `status` was nested under `"verdict"` key instead of being a top-level key
- **Issue:** The plan's Task 2 action spec (and D-11) require `status`, `ambient_status`, and `checked_at` at the same JSON level. The existing `CacheEntry` struct lacked `#[serde(flatten)]`, producing `{"verdict": {"status": ...}, "ambient_status": ...}` instead of `{"status": ..., "ambient_status": ...}`.
- **Fix:** Added `#[serde(flatten)]` to the `verdict: &'a Verdict` field in `CacheEntry`
- **Files modified:** ferro-mcp/src/tools/checkpoint_projection.rs
- **Commit:** 397ec677

**2. [Rule 1 - Bug] clippy uninlined_format_args in 3 test assertions**
- **Found during:** Task 3 clippy gate
- **Issue:** Three `assert!` macro calls used `"message: {:?}", var` instead of `"message: {var:?}"` — clippy `-D warnings` treats this as an error
- **Fix:** Inlined the format variables in all three assertions
- **Files modified:** ferro-mcp/src/tools/checkpoint_projection.rs
- **Commit:** 88607b83 (bundled with Task 3)

### Already Implemented in Earlier Waves

Per the prior-wave-note: `aggregate_status`, `aggregate_next_steps`, `write_cache`, `run_for`, and `execute` were all implemented in Wave 1 (Plan 01) as a Rule 2 deviation. This wave's net-new deliverables were:
1. The test battery for those functions (Tasks 1-2)
2. The MCP tool registration (Task 3)
3. The docs (Task 3)

## Verification

- `cargo build -p ferro-mcp` exits 0
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings` exits 0
- `cargo test -p ferro-mcp -- checkpoint_projection` — 23 passed, 0 failed
- `grep 'name = "checkpoint_projection"' ferro-mcp/src/service.rs` — found at line 1591
- `grep "struct CheckpointProjectionParams" ferro-mcp/src/service.rs` — found at line 327
- `grep "checkpoint_projection::execute" ferro-mcp/src/service.rs` — found at line 1606
- `test -f docs/src/agents/checkpoint-projection.md` — exists
- `grep "not_checked" docs/src/agents/checkpoint-projection.md` — 10 occurrences
- `grep "checkpoint-projection" docs/src/SUMMARY.md` — found

## Known Stubs

Seams 1, 3, 4, 5 remain intentional stubs (`not_implemented_phase_195`) from Plan 01. These are by design — Phase 195 fills them in.

## Threat Flags

None — no new network endpoints, auth paths, or external trust-boundary changes. The MCP tool registration exposes an existing read-only code path. Path traversal threat T-194-01 is mitigated and tested.

## Self-Check: PASSED
