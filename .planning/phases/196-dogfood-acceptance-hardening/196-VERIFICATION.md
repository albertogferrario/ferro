---
phase: 196-dogfood-acceptance-hardening
verified: 2026-06-10T10:00:00Z
status: passed
score: 4/4
overrides_applied: 0
---

# Phase 196: Dogfood Acceptance + Hardening Verification Report

**Phase Goal:** The checkpoint earns its place by finding a real seam defect in a real project. The synthetic app catalog must include at least one deliberately poisoned projection (a field with no backing migration column). The live consumer must produce at least one finding (fail or warn on any seam). Any wrapper seam that produces zero findings across all dogfood inputs is demoted to reporting `not_checked` by default rather than shipped active. `next_steps` is capped to 5 entries.

**Verified:** 2026-06-10T10:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Poisoned projection produces `status: fail` with the field→column seam naming exactly the planted dangling field and no other field | VERIFIED | `poisoned_projection_dangling_field_acceptance` test exists at line 1944 of `checkpoint_projection.rs`; asserts `status == Fail`, `findings.len() == 1`, `findings[0].subject == "phantom_col"`, and that `"id"` is not flagged |
| 2 | Running the checkpoint against a live consumer (`app/`) produces at least one finding | VERIFIED | `dogfood_app_projections` `#[tokio::test]` at line 2000 asserts `total_findings > 0`; `196-ACCEPTANCE.md` records GO verdict with total of 20 findings; genuine defect driver is seam 3 (`action_to_route`) producing 4 real findings from unregistered actions |
| 3 | `next_steps` contains at most 5 entries in any verdict | VERIFIED | `const MAX_NEXT_STEPS: usize = 5` at line 703; cap guard at line 734 uses `result.len() == MAX_NEXT_STEPS`; test `next_steps_cap_at_five` at line 1332 asserts 7 findings → exactly 5 entries |
| 4 | Any wrapper seam with zero findings across all dogfood inputs is demoted to `not_checked`-by-default and documented | VERIFIED | `props_to_contract_seam` at line 579 unconditionally returns `SeamStatus::NotChecked` with `source: "validate_contracts"` and `reason: "unproven_against_real_inputs"`; documented in `service.rs` lines 1610-1613 (`**Seam coverage:**` paragraph) and in `docs/src/agents/checkpoint-projection.md` seam-coverage table at line 120; seams 2, 3 remain active (field_to_column proven by fixture; action_to_route proven by 4 live findings); seams 1, 4 remain active (produced findings in live run) |

**Score:** 4/4 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp/src/tools/checkpoint_projection.rs` | `MAX_NEXT_STEPS: usize = 5`, cap guard, `next_steps_cap_at_five` test, `poisoned_projection_dangling_field_acceptance` test, `dogfood_app_projections` test, `props_to_contract_seam` returning `NotChecked` | VERIFIED | All six items confirmed present at expected lines |
| `ferro-mcp/src/service.rs` | `**Seam coverage:**` paragraph in tool description naming `props_to_contract` as `not_checked`-by-default | VERIFIED | Lines 1610-1613 contain the required paragraph |
| `docs/src/agents/checkpoint-projection.md` | `capped at 5` (not 10), seam-coverage table, no `not_implemented_phase_195`, `props_to_contract` row showing `not_checked` by default | VERIFIED | Line 61: `capped at 5`; line 125: `capped at 5 entries`; line 120: `props_to_contract` row; no stale stub string found |
| `.planning/phases/196-dogfood-acceptance-hardening/196-ACCEPTANCE.md` | Per-seam finding tally + GO verdict | VERIFIED | File exists; records GO verdict; per-seam tally table present; `action_to_route` identified as primary genuine defect driver |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `aggregate_next_steps` | `MAX_NEXT_STEPS` | `result.len() == MAX_NEXT_STEPS` cap guard | VERIFIED | Line 734: `if result.len() == MAX_NEXT_STEPS { break; }` |
| `poisoned_projection_dangling_field_acceptance` | `field_to_column_seam` | direct seam invocation | VERIFIED | Line 1959: `field_to_column_seam(tmp.path(), "dangling", &None, proj_src)` |
| `dogfood_app_projections` | `action_to_route_seam` / all five seam fns | direct per-file calls (NOT `run_for`) | VERIFIED | Lines 2070-2075 call each of the five seam functions directly; no `run_for(` present in test body |
| `196-ACCEPTANCE.md` tally | `props_to_contract_seam` demotion | zero-finding evidence | VERIFIED | ACCEPTANCE.md records 0 findings for `props_to_contract` across both inputs; code demotion follows |
| `props_to_contract_seam` source | `"validate_contracts"` (never `"checkpoint"`) | SC-4 source field | VERIFIED | Line 583: `source: "validate_contracts".to_string()` |

---

## Data-Flow Trace (Level 4)

Not applicable — the primary artifacts are Rust tests and documentation, not dynamic data-rendering components. The test assertions constitute the data-flow proof: inputs (synthetic fixture / live `app/`) flow through seam functions and the assertion at line 2100 confirms the live run produces real findings.

---

## Behavioral Spot-Checks

Cannot run `cargo test` without a server/build environment, but static code analysis confirms all assertions are machine-checkable and non-trivial:

| Behavior | Evidence | Status |
|----------|----------|--------|
| Poisoned fixture fires seam 2 with exactly one finding | Four distinct assertions at lines 1961-1982 | VERIFIED (static) |
| Live dogfood gate asserts `total_findings > 0` | `assert!(total_findings > 0, ...)` at line 2100 | VERIFIED (static) |
| Cap enforced at 5 in `next_steps_cap_at_five` | `assert_eq!(steps.len(), 5, ...)` at line 1339 | VERIFIED (static) |
| `props_to_contract_seam` always returns `NotChecked` | Unconditional literal return at lines 580-587 | VERIFIED (static) |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CHK-10 | 196-01 through 196-04 | Checkpoint run across synthetic catalog with deliberately poisoned projection + live consumer; surfaces at least one real seam defect | SATISFIED | Poisoned fixture proves field→column seam; live run (20 findings, 4 genuine seam-3 defects) proves real-world detection; ACCEPTANCE.md records GO verdict |

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `docs/src/agents/checkpoint-projection.md` | 43 | `"reason": "unproven_against_real_inputs"` used in `projection_well_formed` example slot | INFO | The doc example shows `projection_well_formed` with the `props_to_contract` demotion reason. REVIEW.md flagged this as IN-01; commit `aa783341` addressed it by updating the second seam in the example to `props_to_contract`. This now correctly reflects the demoted seam, not an active one. No remaining concern. |
| `ferro-mcp/src/tools/checkpoint_projection.rs` | 579 | `_project_root` and `_service_name` unused parameters in `props_to_contract_seam` | INFO | Underscore-prefixed parameters are idiomatic and clippy-clean. Retained for call-site compatibility with `run_for`. REVIEW.md flagged as IN-02 with no immediate fix required. |

No blocker or warning-severity anti-patterns remain. The `decide_seam5` cascade gate concern (REVIEW WR-01) is a latent issue when seam 5 is ever re-activated — not a current defect since `props_to_contract_seam` is unconditionally demoted.

---

## Human Verification Required

None. All four success criteria are machine-checkable:
- SC-1: test assertions on exact finding set
- SC-2: `assert!(total_findings > 0)` with ACCEPTANCE.md recording the actual tally
- SC-3: `assert_eq!(steps.len(), 5)` against 7 inputs
- SC-4: unconditional `not_checked` literal return + documented in two public surfaces

---

## Gaps Summary

No gaps. All four success criteria are verified against the actual codebase:

- SC-1 (`poisoned_projection_dangling_field_acceptance`): test exists with all four required assertions — status == Fail, findings.len() == 1, subject == "phantom_col", id not in findings.
- SC-2 (`dogfood_app_projections`): test exists asserting `total_findings > 0`; `196-ACCEPTANCE.md` records GO with per-seam tally; seam 3 (`action_to_route`) is confirmed as the genuine defect driver (4 real findings).
- SC-3 (`next_steps` cap): `const MAX_NEXT_STEPS: usize = 5` present; cap guard uses it; `next_steps_cap_at_five` tests 7 findings → 5 entries; no stale "== 10" / "cap 10" / "10 entries" found in code or docs.
- SC-4 (seam demotion): only `props_to_contract` was demoted (zero findings across both inputs); seams 1, 2, 3, 4 remain active; demotion documented in `service.rs` and `checkpoint-projection.md`; no `not_implemented_phase_195` string remains.

The code review (196-REVIEW.md) fix commit `aa783341` corrected the doc example seam attribution (IN-01) and added inline comments documenting the seam-5 cascade and SC-2 provenance (WR-02). Both addressed.

---

_Verified: 2026-06-10T10:00:00Z_
_Verifier: Claude (gsd-verifier)_
