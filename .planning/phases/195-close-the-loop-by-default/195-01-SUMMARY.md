---
phase: 195-close-the-loop-by-default
plan: 01
subsystem: ferro-mcp
tags: [mcp, projection, checkpoint, async]
requirements: [CHK-07, CHK-08, CHK-09]

dependency_graph:
  requires: []
  provides:
    - async run_for/execute in checkpoint_projection.rs
    - canonical seam names (projection_well_formed, action_to_route, rendered_view, props_to_contract)
    - VerdictSummary type + Verdict::summary()
    - read_ambient_status cache-read helper
    - service.rs checkpoint handler awaits async execute
  affects:
    - ferro-mcp/src/tools/checkpoint_projection.rs
    - ferro-mcp/src/service.rs
    - docs/src/agents/checkpoint-projection.md

tech_stack:
  patterns:
    - tokio::test for async test functions
    - pub(crate) with #[allow(dead_code)] for forward-declared API consumed by later plans

key_files:
  modified:
    - ferro-mcp/src/tools/checkpoint_projection.rs
    - ferro-mcp/src/service.rs
    - docs/src/agents/checkpoint-projection.md

decisions:
  - run_for and execute made async in same commit as seam rename to avoid intermediate broken state
  - read_ambient_status uses tolerant serde_json::Value parse so any cache corruption returns unverified (T-195-01)
  - VerdictSummary excludes not_checked seams from fail_seams/warn_seams per SC-1 signal-to-noise constraint
  - #[allow(dead_code)] on read_ambient_status: Plans 03/04 add callers; forward-declared API is intentional
  - service.rs Task 3 committed with Task 1 (both required for compilation — async execute + await at call site)

metrics:
  duration: 377s
  completed_date: "2026-06-10"
  tasks: 3
  files_modified: 3
---

# Phase 195 Plan 01: Async Foundation + Canonical Seam Names Summary

Async `run_for`/`execute` with canonical seam vocabulary, `VerdictSummary` type, and `read_ambient_status` cache helper — the shared primitives all subsequent Phase 195 plans depend on.

## What Was Built

**Task 1 — Async conversion + seam-name reconciliation (source, tests, docs)**

- `run_for` and `execute` converted from sync to async (`pub(crate) async fn` / `pub async fn`)
- Four Phase-194 stub seam names reconciled to canonical design-spec vocabulary:
  - `schema_load` → `projection_well_formed`
  - `field_type_compat` → `action_to_route`
  - `action_binding` → `rendered_view`
  - `render_target` → `props_to_contract`
- Six tests using old names updated (`aggregate_status_*`, `next_steps_*`)
- Three tests calling `run_for` directly converted to `#[tokio::test] async fn`
- New `seam_names_canonical` test asserts the exact canonical seam set
- `service.rs` checkpoint handler: `.await` added to `execute` call
- Docs updated: seam example JSON uses `projection_well_formed`; `source` table removes "always checkpoint" caveat

**Task 2 — VerdictSummary + Verdict::summary() + read_ambient_status**

- `VerdictSummary` struct added to public output-contract block: `status`, `fail_seams`, `warn_seams`, `next_steps` (no raw `seams` array — SC-1)
- `Verdict::summary()` method: filters fail/warn seams, excludes `not_checked` entries
- `pub(crate) fn read_ambient_status`: reads `.ferro/checkpoints/{name}.json`, returns `"clean"` / `"failing"` / `"unverified"` with tolerant fallback
- Three new unit tests: `verdict_summary_shape`, `ambient_missing_unverified`, `ambient_read_clean`

**Task 3 — service.rs await (committed with Task 1)**

- `checkpoint_projection::execute(...).await` in the MCP handler (already `pub async fn`)

## Test Results

28 tests passing in `cargo test -p ferro-mcp checkpoint_projection`:
- All 25 Phase-194 tests still green after async + rename
- 3 new Task 2 tests passing

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Task 3 committed with Task 1**
- **Found during:** Task 1 compile
- **Issue:** Making `execute` async causes immediate compile error in `service.rs` — the two changes must be in the same compilable state
- **Fix:** Included `service.rs` `.await` change in the Task 1 commit instead of a separate Task 3 commit
- **Files modified:** `ferro-mcp/src/service.rs`
- **Commit:** `a7e0ca9b`

**2. [Rule 2 - Missing functionality] #[allow(dead_code)] on read_ambient_status**
- **Found during:** Task 2 clippy gate
- **Issue:** `pub(crate)` function with no callers yet (Plans 03/04 add them) triggers `-D warnings` dead_code lint
- **Fix:** Added `#[allow(dead_code)]` with explanatory comment pointing to Plans 03/04
- **Files modified:** `ferro-mcp/src/tools/checkpoint_projection.rs`
- **Commit:** `a7c5f049`

**3. [Rule 1 - Bug] Removed forbidden-names set from seam_names_canonical test**
- **Found during:** Task 1 grep gate
- **Issue:** Test had a `forbidden` HashSet containing the old seam names as string literals, which triggered the grep gate
- **Fix:** Removed the redundant `forbidden` set — the `assert_eq!(seam_names, expected)` already enforces canonical names exclusively; the grep gate is the authoritative check for source literals
- **Files modified:** `ferro-mcp/src/tools/checkpoint_projection.rs`
- **Commit:** `a7e0ca9b`

## Verification Gates Passed

- `grep -rn "schema_load\|field_type_compat\|action_binding\|render_target" ferro-mcp/src/ docs/` → no matches (CLEAN)
- `grep -n "pub async fn execute" ferro-mcp/src/tools/checkpoint_projection.rs` → line 148
- `grep -n "pub(crate) async fn run_for" ferro-mcp/src/tools/checkpoint_projection.rs` → line 155
- `grep -n "pub struct VerdictSummary" ferro-mcp/src/tools/checkpoint_projection.rs` → line 77
- `grep -n "pub fn summary(&self) -> VerdictSummary"` → line 96
- `grep -n "pub(crate) fn read_ambient_status"` → line 504
- `grep -n "checkpoint_projection::execute.*\.await" ferro-mcp/src/service.rs` → line 1608
- `cargo fmt --all -- --check` → clean
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings` → clean
- `cargo test -p ferro-mcp checkpoint_projection` → 28/28 passing

## Commits

| Hash | Message |
|------|---------|
| `a7e0ca9b` | feat(195-01): async run_for/execute + canonical seam names + service.rs await |
| `a7c5f049` | feat(195-01): add VerdictSummary + Verdict::summary() + read_ambient_status |

## Self-Check: PASSED

Files exist:
- `ferro-mcp/src/tools/checkpoint_projection.rs` — FOUND
- `ferro-mcp/src/service.rs` — FOUND
- `docs/src/agents/checkpoint-projection.md` — FOUND

Commits exist:
- `a7e0ca9b` — FOUND
- `a7c5f049` — FOUND
