---
phase: 242
plan: 04
subsystem: ferro-projections
tags: [auth, test, crud, validate, boot-time]
requirements: [CRUD-07]

dependency_graph:
  requires: []
  provides: [validate_rejects_crud_verb_without_write_ability test]
  affects: [ferro-projections/src/service.rs mod tests]

tech_stack:
  added: []
  patterns: [boot-time validate() regression pinning, mirror existing validate_catches_* test pattern]

key_files:
  created: []
  modified:
    - ferro-projections/src/service.rs

decisions:
  - D-10: validate() function unchanged (test-only per plan constraint)
  - TDD note: rule shipped before test (5cb17d60); RED phase inapplicable — test written directly against shipped rule and passed green immediately

metrics:
  duration_seconds: 65
  completed_date: "2026-06-23"
  tasks_completed: 1
  files_changed: 1
---

# Phase 242 Plan 04: CRUD-07 boot-time validate() test Summary

Boot-time regression test pinning the CRUD-07 fail-fast rule (`ServiceDef::validate()` rejects write verbs without `mcp_write_ability`, `service.rs:502-510`, shipped in `5cb17d60`).

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Add boot-time validate() write-ability test | ed12c64a | ferro-projections/src/service.rs |

## Decisions Made

- **D-10 honored**: `validate()` function body is unchanged. The plan adds only a test to `mod tests`.
- **TDD note**: The implementation (the rule at `service.rs:502-510`) shipped prior to this plan in commit `5cb17d60`. There is no RED→GREEN sequence here — the test was written against the already-shipped rule and passed immediately. Documented as a deviation from the standard TDD gate sequence.

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written.

### TDD Gate Compliance

The plan has `tdd="true"` but the rule under test shipped before this test plan. There is no RED commit (the test could not fail — the implementation was already correct). This is expected and documented in the plan itself ("verify the SHIPPED CRUD-07 rule — no new validation code"). The GREEN gate is satisfied: `cargo test -p ferro-projections validate_rejects_crud_verb_without_write_ability` exits 0.

## Test Coverage

The test `validate_rejects_crud_verb_without_write_ability` asserts:

1. `.creatable(true)` without `.mcp_write_ability` → `Err(Error::Validation(_))` with message containing `"mcp_write_ability"`
2. `.updatable(true)` without `.mcp_write_ability` → `is_err()`
3. `.deletable(true)` without `.mcp_write_ability` → `is_err()`
4. `.creatable(true)` with `.mcp_write_ability("manage-orders")` → `is_ok()`

## Known Stubs

None.

## Threat Flags

None. This plan adds a test with no new network endpoints, auth paths, file access patterns, or schema changes.

## Self-Check: PASSED

- `ferro-projections/src/service.rs` exists and contains `fn validate_rejects_crud_verb_without_write_ability`
- Commit `ed12c64a` exists in git log
- `validate()` function body at lines 499-510 is unchanged
