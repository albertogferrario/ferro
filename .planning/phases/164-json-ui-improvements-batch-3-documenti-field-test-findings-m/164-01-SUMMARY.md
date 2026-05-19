---
phase: 164
plan: 01
subsystem: ferro-json-ui
tags: [spec, depth, validation, docs, tdd]
dependency_graph:
  requires: []
  provides: [MAX_NESTING_DEPTH=5, depth-5-spec-valid, depth-6-spec-rejected]
  affects: [ferro-json-ui/src/spec.rs, docs/src/json-ui/spec-construction.md]
tech_stack:
  added: []
  patterns: [TDD RED/GREEN, const-bump, integration-test fixture]
key_files:
  created:
    - ferro-json-ui/tests/fixtures/reject/six_level_nesting.json
  modified:
    - ferro-json-ui/src/spec.rs
    - ferro-json-ui/tests/reject.rs
    - docs/src/json-ui/spec-construction.md
decisions:
  - MAX_NESTING_DEPTH raised from 3 to 5 to unblock depth-4 dashboard specs (D-14 / F4)
  - from_json_rejects_four_level_nesting renamed to from_json_rejects_six_level_nesting; fixture updated to 6-node chain
  - nested_builder_flattens_two_levels renamed to nested_builder_accepts_depth_three
  - New Nesting depth limit section added to spec-construction.md
metrics:
  duration: "~20 min"
  completed: "2026-05-17"
  tasks: 2
  files: 4
---

# Phase 164 Plan 01: Raise MAX_NESTING_DEPTH 3 → 5 Summary

**One-liner:** `MAX_NESTING_DEPTH` raised from 3 to 5 in `ferro-json-ui/src/spec.rs`, unblocking depth-4 dashboard specs (root → grid → card → badge) and adding one level of headroom.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Bump MAX_NESTING_DEPTH and rewrite depth tests | `512a91e8` (RED), `32c51766` (GREEN) | `ferro-json-ui/src/spec.rs` |
| 2 | Update spec-construction.md to document depth 5 | `f90c2870` | `docs/src/json-ui/spec-construction.md`, `ferro-json-ui/tests/reject.rs`, `ferro-json-ui/tests/fixtures/reject/six_level_nesting.json` |

## Changes

### ferro-json-ui/src/spec.rs

- `pub const MAX_NESTING_DEPTH: usize = 3` → `5`
- Docstring updated: references dashboard nesting (root → grid → card → row → atom), removes stale 115-CONTEXT D-09 reference
- `nested_builder_flattens_two_levels` renamed to `nested_builder_accepts_depth_three` with comment updated to "well within MAX_NESTING_DEPTH=5"
- New test `nested_builder_accepts_depth_five`: 5-level spec builds successfully
- New test `nested_builder_rejects_depth_six`: 6-level spec fails with `SpecError::DepthExceeded`
- `from_json_rejects_four_level_nesting` renamed to `from_json_rejects_six_level_nesting`: assertion updated to `max=5`, `found>5`, uses 6-node fixture

### ferro-json-ui/tests/reject.rs + fixture

- `reject_four_level_nesting` → `reject_six_level_nesting`
- New fixture `six_level_nesting.json`: root → A → B → C → D → E (6 nodes, depth 6)
- Old `four_level_nesting.json` left in place (still referenced by history; now a valid spec that passes)

### docs/src/json-ui/spec-construction.md

- New section "Nesting depth limit" added between "Composition rules" and "Migration from v1"
- Documents `MAX_NESTING_DEPTH = 5`, `SpecError::DepthExceeded`, and the flattening escape hatch

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `from_json_rejects_four_level_nesting` test now passes spuriously**
- **Found during:** Task 1 GREEN phase — `cargo test --all-features` run
- **Issue:** Integration test in `ferro-json-ui/tests/reject.rs` used a 4-level fixture and asserted `max=3`. With `MAX_NESTING_DEPTH=5`, a 4-level spec is valid — the test would have passed incorrectly.
- **Fix:** Renamed test to `reject_six_level_nesting`, created `six_level_nesting.json` fixture with 6 nodes, updated assertions to `max=5` and `found>5`.
- **Files modified:** `ferro-json-ui/tests/reject.rs`, `ferro-json-ui/tests/fixtures/reject/six_level_nesting.json`
- **Commit:** `f90c2870`

**2. [Rule 3 - Blocking] Disk full during `cargo test --all-features`**
- **Found during:** Task 2 verification
- **Issue:** `errno=28` (ENOSPC) during linking; `/dev/disk3s5` had 251 MB free.
- **Fix:** `cargo clean` freed ~6 GB; tests passed on retry.
- **Impact:** No code change needed; build environment issue.

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED — failing test committed | `512a91e8` | PASS — `nested_builder_accepts_depth_five` failed with `DepthExceeded { max: 3, found: 4 }` |
| GREEN — implementation committed | `32c51766` | PASS — all 8 nested_builder tests pass |
| REFACTOR | N/A | No refactor needed |

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. The depth limit change is a structural constant; raising it from 3 to 5 is a 2-unit increase with bounded recursion in `validate_depth` — stack growth is constant in practice (single-digit nesting adds bytes to multi-MB handler stacks). Per T-164-01-01 in the plan's threat model: accepted.

## Known Stubs

None — this plan makes no data-flow changes; the constant change is complete and unconditional.

## Self-Check

- [x] `grep -c 'pub const MAX_NESTING_DEPTH: usize = 5' ferro-json-ui/src/spec.rs` = 1
- [x] `grep -c 'pub const MAX_NESTING_DEPTH: usize = 3' ferro-json-ui/src/spec.rs` = 0
- [x] `grep -c 'fn nested_builder_accepts_depth_five' ferro-json-ui/src/spec.rs` = 1
- [x] `grep -c 'fn nested_builder_rejects_depth_six' ferro-json-ui/src/spec.rs` = 1
- [x] `cargo test -p ferro-json-ui --lib` exits 0 — 453 passed
- [x] `cargo test -p ferro-json-ui` exits 0 — all integration + doc tests pass
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy --all --all-targets -- -D warnings` clean
- [x] `mdbook build` (docs/) clean

## Self-Check: PASSED
