---
phase: 119
plan: "01"
subsystem: ferro-json-ui
tags: [spec, merge, data, tdd]
dependency_graph:
  requires: []
  provides: [Spec::merge_data]
  affects: [ferro-json-ui/src/spec.rs]
tech_stack:
  added: []
  patterns: [consuming-builder, serde_json::Value::as_object_mut, debug_assert]
key_files:
  modified:
    - ferro-json-ui/src/spec.rs
decisions:
  - "Handler keys override spec.data keys on collision (D-04)"
  - "Value::Null spec.data initialized to empty Object before merge (Pitfall 4 fix)"
  - "Non-Object handler_data silently ignored in production; debug_assert fires in dev"
metrics:
  duration: ~5min
  completed: 2026-04-21
  tasks_completed: 1
  files_modified: 1
---

# Phase 119 Plan 01: Spec::merge_data — Summary

`Spec::merge_data` consuming builder: shallow top-level merge of handler-provided `Value::Object` into `spec.data`, handler keys winning on collision, with `Value::Null` data initialization guard.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add Spec::merge_data with consuming-builder semantics | 6430b178 | ferro-json-ui/src/spec.rs |

## Implementation

**Location:** `ferro-json-ui/src/spec.rs` lines 137–167, inside the `impl Spec` block (between `builder()` and `from_json()`).

**Method signature:** `pub fn merge_data(mut self, handler_data: serde_json::Value) -> Self`

**Key implementation details:**
- Uses `handler_data.as_object()` guard — only Objects trigger the merge
- Initializes `self.data = Value::Object(Map::new())` when `self.data.is_null()` before calling `as_object_mut()` (Pitfall 4 fix from RESEARCH.md)
- Iterates handler object keys with `k.clone(), v.clone()` into the data map
- `debug_assert!` fires in dev builds for non-Null/non-Object input; no panic in production
- No new `use` statements added — `Map` and `Value` already imported at line 19

## Test Results

4/4 tests passing in `spec::tests`:

| Test | Validates |
|------|-----------|
| `merge_data_handler_wins` | Handler key `b` overrides spec key `b`; new key `c` added; untouched key `a` preserved |
| `merge_data_ignores_non_object` | `Value::Null` handler_data leaves spec.data unchanged; no panic |
| `merge_data_initializes_null_data` | `Value::Null` spec.data is promoted to `{"k":"v"}` after merge |
| `merge_data_empty_handler_no_op` | Empty `{}` handler_data leaves spec.data unchanged |

Full suite: 377 tests passed, 0 failed.

## Quality Gates

- `cargo fmt --all -- --check`: PASS
- `cargo clippy -p ferro-json-ui --all-targets -- -D warnings`: PASS (zero warnings)
- `cargo test -p ferro-json-ui --lib`: 377 passed, 0 failed

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. `merge_data` is a complete pure-transform method with no stubs or TODOs.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The method is a pure in-memory value transform. Threat register reviewed (T-119-01-01 through T-119-01-03) — all mitigations in-scope for this plan are addressed by downstream HTML escaping (already in place per RESEARCH.md §Security Domain, not in scope for this plan).

## Self-Check

### Files

- [x] `ferro-json-ui/src/spec.rs` contains `pub fn merge_data` — FOUND
- [x] `grep "merge_data_handler_wins"` returns match — FOUND (line 778)
- [x] `grep "merge_data_initializes_null_data"` returns match — FOUND (line 805)

### Commits

- [x] 6430b178 — feat(119-01): add Spec::merge_data consuming builder method — FOUND

## Self-Check: PASSED
