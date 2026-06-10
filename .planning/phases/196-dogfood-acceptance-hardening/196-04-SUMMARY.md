---
phase: 196-dogfood-acceptance-hardening
plan: "04"
subsystem: ferro-mcp
tags: [checkpoint, seam-demotion, coverage-honesty, documentation]
dependency_graph:
  requires: [196-03]
  provides: [sc4-seam-demotion-documented]
  affects: [ferro-mcp, docs]
tech_stack:
  added: []
  patterns: [not_checked-by-default, coverage-honesty-invariant]
key_files:
  modified:
    - ferro-mcp/src/tools/checkpoint_projection.rs
    - ferro-mcp/src/service.rs
    - docs/src/agents/checkpoint-projection.md
decisions:
  - "Demote only seam 5 (props_to_contract): the sole zero-finding seam across both dogfood inputs per 196-ACCEPTANCE.md"
  - "Seams 1 and 4 stay active: findings were collision artifacts but still real findings in the tally"
  - "Remove unused validate_contracts import after demotion to satisfy clippy -D warnings"
metrics:
  duration: "~19 minutes"
  completed: "2026-06-10T02:14:03Z"
  tasks_completed: 3
  files_modified: 3
---

# Phase 196 Plan 04: Evidence-Driven Seam Demotion Summary

Evidence-driven demotion of `props_to_contract` from active dispatch to `not_checked`-by-default, with SC-4-compliant documentation in both the MCP tool description and the agent doc.

## What Was Done

### Seam demotion decision (from 196-ACCEPTANCE.md)

| Seam | Poisoned Fixture Findings | app/ Live Findings | Total | Decision |
|------|--------------------------|-------------------|-------|----------|
| 1 `projection_well_formed` | 0 | 8 | 8 | STAYS ACTIVE (found findings) |
| 2 `field_to_column` | 1 | 0 | 1 | EXEMPT (proven by poisoned fixture) |
| 3 `action_to_route` | 0 | 4 | 4 | STAYS ACTIVE (found findings) |
| 4 `rendered_view` | 0 | 8 | 8 | STAYS ACTIVE (found findings) |
| 5 `props_to_contract` | 0 | 0 | 0 | DEMOTED to `not_checked`-by-default |

Seams 1 and 4 produced findings via the name-collision path (file-stem resolution vs function-name resolution). The ACCEPTANCE.md tally counts them as findings regardless of the underlying cause. Only seam 5 had zero findings across both inputs.

### Task 1: Code change (`checkpoint_projection.rs`)

`props_to_contract_seam` body replaced with a `not_checked` literal return:
- `status: SeamStatus::NotChecked`
- `source: "validate_contracts"` (SC-4: delegating validator, never `"checkpoint"`)
- `reason: Some("unproven_against_real_inputs")`
- Parameters prefixed with `_` to satisfy clippy
- `validate_contracts` module import removed (no longer used — clippy `-D warnings`)
- `seam5_source_provenance` test updated to expect `"unproven_against_real_inputs"` reason

### Task 2: Documentation

**`ferro-mcp/src/service.rs`**: appended `**Seam coverage:**` paragraph to the `checkpoint_projection` tool description naming `props_to_contract` as `not_checked`-by-default with rationale.

**`docs/src/agents/checkpoint-projection.md`**:
- Replaced stale `"not_implemented_phase_195"` reason in the example JSON with `"unproven_against_real_inputs"`
- Added seam-coverage table documenting all 5 seams' default outcomes with per-seam evidence

### Task 3: Full gate

`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` — all green.

## Phase 196 Success Criteria Confirmation

| Criterion | Test | Status |
|-----------|------|--------|
| SC-1 | `poisoned_projection_dangling_field_acceptance` | PASS |
| SC-2 | `dogfood_app_projections` (total_findings=20 > 0) | PASS |
| SC-3 | `next_steps_cap_at_five` (cap=5) | PASS |
| SC-4 | Demoted seam documented in service.rs + agent doc | PASS |

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | `23c82690` | feat(196-04): demote props_to_contract seam to not_checked-by-default |
| 2 | `4dff0c44` | docs(196-04): document not_checked-by-default seam in service.rs and agent doc |
| 3 | `d9cd3ce8` | style(196-04): fix rustfmt formatting in seam5 test assertion |

## Deviations from Plan

**[Rule 1 - Bug] Unused import after demotion**
- **Found during:** Task 1
- **Issue:** Removing `validate_contracts` call from `props_to_contract_seam` left the `validate_contracts` module import unused; clippy `-D warnings` would fail
- **Fix:** Removed `validate_contracts` from the `use super::{...}` import list
- **Files modified:** `ferro-mcp/src/tools/checkpoint_projection.rs`
- **Commit:** `23c82690`

**[Rule 1 - Bug] rustfmt formatting**
- **Found during:** Task 3 (fmt check)
- **Issue:** Multi-line `assert_eq!` in the updated `seam5_source_provenance` test did not match rustfmt's single-line format
- **Fix:** Collapsed to single-line form
- **Files modified:** `ferro-mcp/src/tools/checkpoint_projection.rs`
- **Commit:** `d9cd3ce8`

## Known Stubs

None. The `props_to_contract` demotion is intentional and documented — it is not a stub, it is a coverage-honesty declaration. The seam will be re-activated in a future phase once real consumer evidence is gathered.

## Threat Flags

None. This change makes verdicts more honest (reports `not_checked` instead of vacuous `pass`), narrowing the tool's claims rather than expanding attack surface.

## Self-Check: PASSED

- `ferro-mcp/src/tools/checkpoint_projection.rs` exists and contains `unproven_against_real_inputs` ✓
- `ferro-mcp/src/service.rs` contains `not_checked` in tool description ✓
- `docs/src/agents/checkpoint-projection.md` contains `not_checked` (13 occurrences), no `not_implemented_phase_195` ✓
- Commits `23c82690`, `4dff0c44`, `d9cd3ce8` exist in git log ✓
- All 301 ferro-mcp tests pass ✓
