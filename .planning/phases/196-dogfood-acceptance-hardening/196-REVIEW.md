---
phase: 196-dogfood-acceptance-hardening
reviewed: 2026-06-10T00:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - ferro-mcp/src/tools/checkpoint_projection.rs
  - ferro-mcp/src/service.rs
  - docs/src/agents/checkpoint-projection.md
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 196: Code Review Report

**Reviewed:** 2026-06-10
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found

## Summary

Phase 196 makes four concrete changes: reduces `next_steps` cap 10→5, adds a single-dangling-field acceptance test, adds a dogfood live run against `app/`, renames the cap test, and demotes `props_to_contract_seam` to a permanent `not_checked` stub. The demotion is structurally correct — it preserves the coverage-honesty invariant by returning `not_checked` rather than a vacuous pass. The `validate_contracts` import was already absent from the `use super::{...}` block, so no stale import remains. The implementation is generally clean; four issues are called out below.

## Warnings

### WR-01: Cascade gate passes seam 5 when seam 4 is `not_checked` — logically unsound

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:621-630`

**Issue:** `decide_seam5` returns `None` (i.e., "run seam 5") when `seam4_status == NotChecked`. This means seam 5 is allowed to run even when seam 4 was itself skipped due to a prerequisite being absent. The cascade rule documented in the code comment (line 204-207) states seam 5 skips only when seam 1 or seam 4 *failed*, not when seam 4 is `not_checked`. In the Phase 196 world this is moot because `props_to_contract_seam` is unconditionally demoted to `not_checked`, so the gate decision never affects an observable outcome. However, if seam 5 is ever re-activated, the gate will silently allow seam 5 to run after a prerequisite-absent seam 4, violating the stated cascade contract and the CHK-03 coverage-honesty invariant.

The test `decide_seam5_pure` at line 1835 asserts `decide_seam5(&SeamStatus::Pass, &SeamStatus::NotChecked) == None`, confirming the behavior is tested and intentional — but the test comment does not explain *why* `NotChecked` on seam 4 permits seam 5 to run. This is a latent correctness risk, not a present defect.

**Fix:** Either extend the gate to skip seam 5 when seam 4 is `not_checked`:
```rust
fn decide_seam5(seam1_status: &SeamStatus, seam4_status: &SeamStatus) -> Option<&'static str> {
    if *seam1_status == SeamStatus::Fail {
        Some("seam_1_failed")
    } else if *seam4_status == SeamStatus::Fail || *seam4_status == SeamStatus::NotChecked {
        Some("seam_4_not_runnable")
    } else {
        None
    }
}
```
Or add a comment to `decide_seam5` explicitly documenting that `NotChecked` on seam 4 is a conscious decision to let seam 5 run independently, so future re-activation does not silently inherit bad cascade semantics.

---

### WR-02: `dogfood_app_projections` SC-2 gate relies on name-collision artifacts, not field-level findings

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:1979-2093`

**Issue:** The test comment at lines 1982-1992 documents that seam 2 (`field_to_column`) returns `not_checked` for every `app/` projection because all entities export `pub struct Model` (so model name resolution fails), and the expected driver for `total_findings > 0` is seam 3 (`action_to_route`) on unregistered action names. The test additionally documents that seams 1 and 4 produce findings via name-collision artifacts (the file stem `"feedback_form"` resolves to a different function than the actual `service_def` export).

The SC-2 gate (`assert!(total_findings > 0)`) therefore passes primarily because of structural mismatches between the test harness (file-stem routing) and the `app/` projection convention (all export `service_def`), not because the checkpoint finds real projection defects. This means the gate proves "some seam fires" rather than "the dogfood seams catch real problems." If the `app/` projections are refactored to export unique function names (matching the file stems), the seam 3 action-based findings may still hold the gate, but the character of the evidence changes.

This is not a bug that needs an immediate fix, but the assertion needs a stronger comment distinguishing structural/harness artifacts from genuine field-level findings, so future test maintainers do not treat a passing SC-2 gate as evidence that `field_to_column` is working against live data.

**Fix:** Add a comment above the `assert!(total_findings > 0)` clarifying which seams are expected to contribute findings and why:
```rust
// SC-2 gate: at least one finding required across all dogfood seams.
// Expected contributors:
//   - seam 3 (action_to_route): unregistered actions in feedback_form/order projections
//   - seams 1/4: name-collision artifacts (file stem != exported fn name "service_def")
// Seam 2 (field_to_column) contributes zero findings (model name resolution fails
// because app/ entities export `pub struct Model` rather than a per-entity struct name).
// Treat this gate as an integration smoke test, not a field-coverage proof.
assert!(
    total_findings > 0,
    ...
```

## Info

### IN-01: Doc example shows `projection_well_formed` as `not_checked` with `reason: "unproven_against_real_inputs"` — misleading

**File:** `docs/src/agents/checkpoint-projection.md:42-47`

**Issue:** The verdict shape example (lines 24-51) shows `projection_well_formed` with `status: "not_checked"` and `reason: "unproven_against_real_inputs"`. That reason string belongs to the `props_to_contract` demotion (Phase 196), not to `projection_well_formed`, which is an active seam that returns `not_checked` only when `validate_projection` is unavailable (with `reason: "validate_projection_unavailable"`). A reader following the example to interpret real tool output will misread `projection_well_formed` as demoted when it is actually active.

**Fix:** Update the example to reflect a realistic output — `projection_well_formed` should appear as `"status": "pass"` or `"status": "not_checked"` with `"reason": "validate_projection_unavailable"`, not with the `props_to_contract` demotion reason. Alternatively, change the second seam in the example to `props_to_contract` to accurately illustrate the demoted seam.

---

### IN-02: `_project_root` and `_service_name` unused parameters in `props_to_contract_seam` are permanent dead API surface

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:579`

**Issue:** `props_to_contract_seam` now takes `_project_root: &Path` and `_service_name: &str` but ignores both unconditionally. As a demoted stub, it will never use them. The parameters are still accepted so the call site in `run_for` (line 217) does not need to change, but they add noise and will cause clippy to flag unused-variable warnings unless the leading underscore suppresses them (it does in Rust, so no lint failure). The long-term concern is that the function signature implies the seam is configurable by project root and service name, which is false.

**Fix:** No immediate change required (underscore prefix is idiomatic and clippy-clean). If `props_to_contract` is ever re-activated, remove the underscores and restore the logic. If it is permanently removed, delete the parameters. For now, a brief comment on the signature noting the parameters are retained for call-site compatibility is sufficient:

```rust
// Parameters retained for call-site compatibility with run_for; unused while demoted.
fn props_to_contract_seam(_project_root: &Path, _service_name: &str) -> SeamResult {
```

---

_Reviewed: 2026-06-10_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
