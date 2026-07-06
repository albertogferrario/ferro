---
phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test
plan: 01
subsystem: ferro-json-ui
tags: [json-ui, spec, depth-limit, diagnostics, tdd]
dependency_graph:
  requires: []
  provides: [depth-8-spec-acceptance, depth-limit-diagnostic-split]
  affects: [ferro-json-ui/src/spec.rs, ferro-json-ui/src/render/mod.rs]
tech_stack:
  added: []
  patterns: [tdd-red-green, constant-rename, diagnostic-split]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/spec.rs
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/tests/reject.rs
    - ferro-json-ui/tests/fixtures/reject/six_level_nesting.json
decisions:
  - "MAX_NESTING_DEPTH = 16 per D-F1-depth lock; consumer evidence is depth-8, limit provides 2x headroom"
  - "Walker tripwire uses 'depth limit exceeded at depth N (max=M)' per D-F1-diagnostic lock; cycle detector untouched"
  - "Integration test fixture six_level_nesting.json updated in-place to 17-level chain; test name kept for runner stability"
metrics:
  duration: ~13 minutes
  completed: 2026-05-20
  tasks_completed: 2
  files_modified: 4
---

# Phase 175 Plan 01: F1 — Depth Limit Raise + Diagnostic Split Summary

**One-liner:** `MAX_NESTING_DEPTH` raised from 5 to 16 with the walker tripwire diagnostic split from cycle detection — depth-8 consumer specs (staff-detail tree) now parse, validate, and render without node stripping.

## What Was Built

Two production changes and six new/renamed tests (TDD red→green):

**spec.rs:**
- `MAX_NESTING_DEPTH: usize = 5` → `16`
- Docstring updated to reference Phase 175 gestiscilo-it staff-detail evidence (depth 8; limit at 16 provides headroom)

**render/mod.rs:**
- Walker tripwire diagnostic rewritten: `"cycle guard tripped at depth {depth}"` → `"depth limit exceeded at depth {depth} (max={MAX_NESTING_DEPTH})"`
- Diagnostic comment updated to clarify this is distinct from cycle detection

**tests/reject.rs + tests/fixtures/reject/six_level_nesting.json:**
- Integration test fixture updated from 6-level to 17-level chain (6 levels now accepted with MAX=16)
- Assertion updated: `max=5, found>5` → `max=16, found=17`

## Tests Added / Renamed

| Test | File | Change | Result |
|------|------|--------|--------|
| `from_json_rejects_depth_17` | spec.rs | renamed from `from_json_rejects_six_level_nesting`; 17-deep chain | green |
| `from_json_accepts_depth_8` | spec.rs | new; consumer evidence fixture (8-level staff-detail tree) | green |
| `cycle_detector_only_on_revisit` | spec.rs | new; A→B→A asserts `SpecError::Cycle` not depth tripwire | green |
| `nested_builder_accepts_depth_sixteen` | spec.rs | renamed from `nested_builder_accepts_depth_five`; 16-level accept boundary | green |
| `nested_builder_rejects_depth_seventeen` | spec.rs | renamed from `nested_builder_rejects_depth_six`; 17-level reject boundary | green |
| `walker_depth_tripwire_relative` | render/mod.rs | renamed from `walker_cycle_tripwire_fires_at_depth_4`; asserts "depth limit exceeded" | green |
| `walker_depth_tripwire` | render/mod.rs | new; asserts "depth limit exceeded" + "max=16" + NOT "cycle" | green |

## Commits

| Hash | Type | Description |
|------|------|-------------|
| `46782ec0` | test | add depth-boundary and diagnostic split tests (red state) |
| `0fa84020` | feat | raise MAX_NESTING_DEPTH to 16 and rewrite walker diagnostic |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Integration test `reject_six_level_nesting` broke when MAX=16 was set**
- **Found during:** Task 2 (running `cargo test -p ferro-json-ui`)
- **Issue:** `tests/reject.rs:reject_six_level_nesting` used a 6-level fixture and asserted `max=5`. After raising the constant, the 6-level spec correctly parsed (no longer exceeds limit), causing the test to panic on `Ok(...)`.
- **Fix:** Updated `tests/fixtures/reject/six_level_nesting.json` to a 17-level chain and updated `tests/reject.rs` assertions to `max=16, found=17`. Fixture file name kept unchanged for test runner stability.
- **Files modified:** `ferro-json-ui/tests/reject.rs`, `ferro-json-ui/tests/fixtures/reject/six_level_nesting.json`
- **Commit:** `0fa84020`

**2. [Rule 1 - Bug] `cargo fmt` rejected multi-line assert! calls in Task 1 tests**
- **Found during:** Pre-commit fmt check after Task 2
- **Issue:** Two `assert!(...)` blocks in `walker_depth_tripwire_relative` and `walker_depth_tripwire` were formatted as multi-line but rustfmt wanted single-line (they fit within 100 chars).
- **Fix:** Reformatted two `assert!` calls to single-line form.
- **Files modified:** `ferro-json-ui/src/render/mod.rs`
- **Commit:** `0fa84020` (same commit as the main Task 2 change)

## Verification Results

```
cargo test -p ferro-json-ui
  running 544 tests
  test result: ok. 544 passed; 0 failed

cargo test --all-features
  All workspace crates: 0 FAILED

cargo fmt --all -- --check      → clean (exit 0)
cargo clippy --all --all-targets -- -D warnings → clean (exit 0)
```

### Acceptance Criteria Check

- `grep -q 'MAX_NESTING_DEPTH: usize = 16' ferro-json-ui/src/spec.rs` → PASS
- `grep -q 'depth limit exceeded at depth' ferro-json-ui/src/render/mod.rs` → PASS
- `! grep -q 'cycle guard tripped at depth' ferro-json-ui/src/render/mod.rs` → PASS
- `grep -q 'max={MAX_NESTING_DEPTH}' ferro-json-ui/src/render/mod.rs` → PASS
- `grep -rn 'MAX_NESTING_DEPTH = 5' ferro-json-ui/` → empty (PASS)
- `cargo test -p ferro-json-ui from_json_accepts_depth_8` → exit 0 (PASS)
- `cargo test -p ferro-json-ui walker_depth_tripwire` → exit 0 (PASS)

## Must-Have Verification

- Depth-8 spec (`from_json_accepts_depth_8`) parses and validates without `DepthExceeded`: **PASS**
- Depth-17 spec is rejected with `SpecError::DepthExceeded { max: 16, found: 17 }`: **PASS**
- Walker tripwire emits `"depth limit exceeded at depth N (max=16)"` — never `"cycle guard tripped"`: **PASS**
- Cycle detector emits `SpecError::Cycle { path }` only on real revisit (`cycle_detector_only_on_revisit`): **PASS**
- `BUILTIN_TYPES.len() == 42` invariant preserved (F1 does not touch the catalog): **PASS**

## Impact

- **F4 (Plan 175-05) unblocked:** `Switch` at depth 8 will now reach its dispatch arm. The consumer's staff-detail tree renderer no longer strips children past depth 6.
- **Diagnostic legibility:** Future depth failures in hand-mutated specs are immediately distinguishable from cycle detection. No more `"cycle guard"` false attribution.

## Known Stubs

None. All changes are production code backed by passing tests.

## Threat Flags

None. The depth constant change is a relaxation of an existing DoS guard within the parameters established by T-175-01-01 (max=16, bounded stack cost). The diagnostic renaming (T-175-01-03) leaks only static information already present in a published constant.

## Self-Check: PASSED

- `ferro-json-ui/src/spec.rs` — modified, contains `MAX_NESTING_DEPTH: usize = 16`
- `ferro-json-ui/src/render/mod.rs` — modified, contains `depth limit exceeded at depth`
- Commit `46782ec0` exists: `git log --oneline | grep 46782ec0` → found
- Commit `0fa84020` exists: `git log --oneline | grep 0fa84020` → found
