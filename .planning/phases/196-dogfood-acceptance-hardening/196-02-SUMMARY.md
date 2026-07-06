---
phase: 196-dogfood-acceptance-hardening
plan: "02"
subsystem: ferro-mcp
tags: [test, acceptance, checkpoint, seam-2, tdd]
dependency_graph:
  requires: [196-01]
  provides: [poisoned-fixture-acceptance-test]
  affects: [ferro-mcp/src/tools/checkpoint_projection.rs]
tech_stack:
  added: []
  patterns: [tempdir-fixture, direct-seam-invocation, four-assertion-acceptance]
key_files:
  created: []
  modified:
    - ferro-mcp/src/tools/checkpoint_projection.rs
decisions:
  - "Single-word struct name 'Dangling'/'dangling' required for seam-2 lowercase matcher to match"
  - "DataType::String used for phantom_col to avoid D-06 warn path firing before column check"
metrics:
  duration: "100s"
  completed: "2026-06-10"
  tasks_completed: 1
  files_modified: 1
requirements: [CHK-10]
---

# Phase 196 Plan 02: Poisoned Fixture Acceptance Test Summary

Poisoned projection acceptance test: one planted dangling field proves seam 2 catches real defects.

## What Was Built

Added `poisoned_projection_dangling_field_acceptance` to the `mod tests` block in
`ferro-mcp/src/tools/checkpoint_projection.rs`. The test constructs a tempdir fixture
with a `dangling` service that declares `id` (has a backing column) and `phantom_col`
(no backing column), invokes `field_to_column_seam` directly, and asserts four exact
conditions (SC-1):

1. `status == SeamStatus::Fail`
2. `findings.len() == 1`
3. `findings[0].subject == "phantom_col"`
4. No finding with `subject == "id"` (negative assertion)

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1    | 7ddaefb0 | test(196-02): add poisoned_projection_dangling_field_acceptance fixture |

## Deviations from Plan

None — plan executed exactly as written. The test passed on the first run (expected:
seam 2 already existed from Phase 194; this is an acceptance lock, not new behavior).
Formatting fix applied: `assert_eq!(result.status, ...)` was a single-line call exceeding
the rustfmt line limit — expanded to multi-line form before commit.

## Known Stubs

None.

## Threat Flags

None introduced. Test operates on a tempdir scoped to the test process; no production
path or shared state is written. Consistent with T-196-02 accept disposition.

## Self-Check: PASSED

- `fn poisoned_projection_dangling_field_acceptance` exists: confirmed (grep returns 1)
- Commit 7ddaefb0 exists: confirmed
- Test passes: `test result: ok. 1 passed; 0 failed`
- No file deletions in commit
