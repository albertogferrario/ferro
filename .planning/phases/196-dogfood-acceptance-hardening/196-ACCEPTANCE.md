# Phase 196: Projection Checkpoint — Acceptance Report

**Date:** 2026-06-10
**Checkpoint version under test:** commit `5d1512ba` (ferro-mcp v0.2.50)
**Plan:** 196-03 (SC-2 / D-03 / CHK-10)

---

## Inputs

Two dogfood inputs were exercised:

### Input 1: Poisoned Synthetic Fixture (Plan 196-02)

A `#[test]` fixture in `ferro-mcp/src/tools/checkpoint_projection.rs`
(`poisoned_projection_dangling_field_acceptance`). The fixture constructs a
temporary projection with exactly one field (`phantom_col`) absent from the
backing model's column set. Exercises seam 2 (`field_to_column`), which is
the seam that cannot fire on the `app/` live consumer due to SeaORM naming
conventions.

- Commit: `7ddaefb0`
- Result: seam 2 `Fail`, 1 finding, subject = `"phantom_col"`, no other field flagged (SC-1 satisfied).

### Input 2: In-Repo `app/` Live Consumer (Plan 196-03)

The `dogfood_app_projections` `#[tokio::test]` in
`ferro-mcp/src/tools/checkpoint_projection.rs`. Exercises all 8 projection
files in `app/src/projections/` (excluding `mod.rs`):

| File | Service Name |
|------|--------------|
| `api_key.rs` | `api_key` |
| `feedback_form.rs` | `feedback_form` |
| `order.rs` | `order` |
| `product.rs` | `product` |
| `revenue_dashboard.rs` | `revenue_dashboard` |
| `sales_analytics.rs` | `sales_analytics` |
| `todo.rs` | `todo` |
| `user.rs` | `user` |

Seam functions were called directly per file (not via `run_for`) because all 8
files export `pub fn service_def() -> ServiceDef`, which causes a name collision
in `list_projections`.

---

## Per-Seam Finding Tally

### Input 1: Poisoned Fixture

| Seam | Canonical Name | Findings |
|------|---------------|----------|
| 2 | `field_to_column` | 1 (planted `phantom_col`) |
| 1 | `projection_well_formed` | 0 |
| 3 | `action_to_route` | 0 |
| 4 | `rendered_view` | 0 |
| 5 | `props_to_contract` | 0 |

### Input 2: `app/` Live Consumer (8 projections, summed)

| Seam | Canonical Name | Findings | Notes |
|------|---------------|----------|-------|
| 1 | `projection_well_formed` | 8 | `NotChecked` path — `inspect_projection` matches by function name; all `app/` files export `service_def`, so per-file lookup by file stem returns `not_found` (1 finding each) |
| 2 | `field_to_column` | 0 | `NotChecked` — SeaORM entities define `pub struct Model`; `list_models` returns name `"Model"` for all, service_name→model name match fails |
| 3 | `action_to_route` | 4 | `Fail` — `feedback_form` declares action `submit_feedback`; `order` declares `submit`, `approve`, `ship`; none registered in `app/src/routes.rs` |
| 4 | `rendered_view` | 8 | `Fail` — `render_projection::execute` also resolves by function name; per-file stem lookup returns `not_found` (1 finding each) |
| 5 | `props_to_contract` | 0 | `NotChecked` — no `app/` routes match the service name substrings |

**Total findings across live consumer run:** 20

### Combined Tally (both inputs)

| Seam | Canonical Name | Total Findings |
|------|---------------|----------------|
| 1 | `projection_well_formed` | 8 |
| 2 | `field_to_column` | 1 |
| 3 | `action_to_route` | 4 |
| 4 | `rendered_view` | 8 |
| 5 | `props_to_contract` | 0 |

---

## Per-Seam Demotion Candidates (D-04 Input)

Wrapper seams with zero findings across all dogfood inputs:

| Seam | Canonical Name | Zero Findings? | Reason |
|------|---------------|----------------|--------|
| 5 | `props_to_contract` | YES | No `app/` routes match service name substrings; poisoned fixture has no contract mismatch. Candidate for `not_checked`-by-default. |

**Seam 2 (`field_to_column`) is exempt from demotion** — the poisoned fixture
(Input 1) produced 1 finding, proving seam 2 fires on real defects.

Note: Seams 1 and 4 produced findings in the live consumer run, but these
findings arose from the name-collision limitation (file-stem vs function-name
resolution) rather than genuine structural defects. Plan 04 will evaluate
whether to treat these as real findings for demotion purposes.

---

## Verdict

**GO**

The live consumer run produced 20 findings across all `app/` projections
(`total_findings = 20 > 0`). The SC-2 machine-checkable assertion
(`assert!(total_findings > 0, ...)` in `dogfood_app_projections`) passed.

The primary driver of real structural defects is **seam 3 (`action_to_route`)**:
4 actions declared by `feedback_form` (`submit_feedback`) and `order` (`submit`,
`approve`, `ship`) have no corresponding registered route in `app/src/routes.rs`.
This is a genuine seam defect in a real application — not a test artifact.

CHK-10 is satisfied: the checkpoint surfaces at least one real seam defect in a
real project. Plan 04 (seam demotion) proceeds on the evidence recorded above.
