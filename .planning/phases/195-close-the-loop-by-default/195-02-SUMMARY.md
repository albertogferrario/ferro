---
phase: 195-close-the-loop-by-default
plan: 02
subsystem: ferro-mcp
tags: [mcp, projection, checkpoint, seams, validators, cascade]
requirements: [CHK-09]

dependency_graph:
  requires: [195-01]
  provides: [checkpoint-seam-dispatch, seam-cascade, sc4-guard]
  affects: [ferro-mcp/src/tools/checkpoint_projection.rs]

tech_stack:
  added: []
  patterns:
    - per-seam dispatch functions normalizing heterogeneous validator output to Finding
    - two-source rule for seam 4 (render_projection vs json_ui_validate_spec)
    - cascade gate helpers (decide_seam4/decide_seam5) as pure #[cfg(test)] functions
    - make_not_checked constructor to avoid cascade repetition

key_files:
  modified:
    - ferro-mcp/src/tools/checkpoint_projection.rs

decisions:
  - title: "decide_seam4/decide_seam5 as #[cfg(test)] pure helpers"
    rationale: >
      The cascade decision logic is trivial (one or two comparisons) and only needed
      in tests for deterministic verification. Marking #[cfg(test)] avoids dead_code
      warnings without adding a #[allow] suppression.
  - title: "action_to_route_seam takes Option<&ServiceDef>"
    rationale: >
      service_def reconstruction can fail; passing None is the correct not-checked path
      rather than propagating an error that would mask the seam's own outcome.

metrics:
  duration_minutes: 7
  completed_date: "2026-06-10"
  tasks_completed: 3
  files_modified: 1
---

# Phase 195 Plan 02: Wire Four Wrapper Seams to Validators Summary

Replace four `NotChecked` stubs in `run_for` with cascade-aware dispatch to existing validators,
normalizing each validator's heterogeneous output into `Finding` at the module boundary, with
per-seam `source` provenance enforced by the SC-4 guard test.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Seam 1 + seam 3 dispatch + normalization | a5968534 | checkpoint_projection.rs |
| 2 | Seam 4 + seam 5 dispatch + normalization | a5968534 | checkpoint_projection.rs |
| 3 | Cascade wiring + SC-4 guard + provenance tests | a5968534 | checkpoint_projection.rs |
| fix | Test fixture provenance update | e0b9c88a | checkpoint_projection.rs |

## What Was Built

Four private dispatch functions added to `checkpoint_projection.rs`:

- `projection_well_formed_seam(project_root, name)` — calls `validate_projection::execute_single`,
  normalizes `errors[]` → `Finding` (Fail), `warnings[]` → `Finding` (Warn), `source: "validate_projection"`
- `action_to_route_seam(service, routes)` — calls `json_ui_verify_action::find_handler` per action,
  routes pre-loaded once via `list_routes::execute(project_root).await`; `source: "json_ui_verify_action"`
- `rendered_view_seam(project_root, name)` — calls `render_projection::execute` then
  `json_ui_validate_spec::execute`; two-source rule: render failure → `"render_projection"`,
  spec findings → `"json_ui_validate_spec"`
- `props_to_contract_seam(project_root, service_name)` — calls `validate_contracts::execute`
  with substring filter; routes-file-missing → `not_checked("routes_file_missing")`;
  `source: "validate_contracts"`

Cascade wiring in `run_for` (D-06):
- Seams 1, 2, 3 always run
- Seam 4: `not_checked("seam_1_failed")` if seam 1 failed
- Seam 5: `not_checked("seam_1_failed")` or `not_checked("seam_4_failed")` per cascade

## Tests Added (38 total, all green)

New Phase 195 tests:
- `seam1_source_provenance` — asserts `source == "validate_projection"`
- `seam3_source_provenance` — asserts `source == "json_ui_verify_action"` on both not_checked and pass paths
- `seam4_source_provenance` — asserts `source == "render_projection"` on render fail, `source == "json_ui_validate_spec"` on spec-validate path
- `seam5_source_provenance` — asserts `source == "validate_contracts"` and `status == NotChecked` with `reason == "routes_file_missing"`
- `cascade_seams_2_3_independent` — verifies seam 3 runs regardless of seam 1 status
- `cascade_seam1_fail` — verifies seams 4 and 5 get `not_checked("seam_1_failed")`
- `cascade_seam4_fail` — verifies seam 5 gets `not_checked("seam_4_failed")`
- `decide_seam4_pure` / `decide_seam5_pure` — unit tests for pure cascade decision helpers
- `sc4_no_checkpoint_source_on_wrapper_seams` — mechanical SC-4 guard asserting no wrapper seam carries `source: "checkpoint"`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Non-exhaustive match in action_to_route_seam**
- **Found during:** Initial build
- **Issue:** Combined `None | Some(_) if ...` guard in a match arm produced E0004 non-exhaustive patterns
- **Fix:** Replaced with two sequential `match`/`if` checks (None arm then empty-actions check)
- **Files modified:** checkpoint_projection.rs
- **Commit:** a5968534

**2. [Rule 2 - Missing functionality] Test fixture used stale Phase-194 stub values**
- **Found during:** Post-commit stub scan
- **Issue:** `verdict_summary_shape` test fixture had `source: "checkpoint"` and `reason: "not_implemented_phase_195"` on the `action_to_route` entry — misleading after Phase 195 wires the seam
- **Fix:** Updated to `source: "json_ui_verify_action"` and `reason: "route_list_unavailable"`
- **Files modified:** checkpoint_projection.rs
- **Commit:** e0b9c88a

## Known Stubs

None. All four wrapper seams now dispatch to real validators. `source == "checkpoint"` appears only on `field_to_column` (owned by checkpoint itself, per SC-4 invariant).

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries beyond those documented in the plan's threat model (T-195-04 through T-195-06).

## Self-Check: PASSED

- `ferro-mcp/src/tools/checkpoint_projection.rs` — modified file exists
- Commit `a5968534` — verified via `git log`
- Commit `e0b9c88a` — verified via `git log`
- `cargo test -p ferro-mcp checkpoint_projection` — 38 passed, 0 failed
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
