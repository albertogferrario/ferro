# Phase 195: Close the Loop by Default - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.

**Date:** 2026-06-10
**Phase:** 195-close-the-loop-by-default
**Mode:** --auto (recommended defaults auto-selected)
**Areas discussed:** Seam-naming reconciliation, Wrapper dispatch+normalization, Seam cascade, Inline verdict summary format, Ambient status surfacing, Stale-ok cache read

---

## Seam-naming reconciliation (discrepancy from Phase 194)

| Option | Description | Selected |
|--------|-------------|----------|
| Fix to canonical design-spec names | Rename 194's `schema_load`/`field_type_compat`/`action_binding`/`render_target` to `projection_well_formed`/`action_to_route`/`rendered_view`/`props_to_contract` and wire validators | ✓ |
| Keep 194 names | Wire validators under the existing (semantically wrong) names | |

**Choice:** Fix (recommended) — names are part of the output contract; settle before Phase 196 documents them. Update stubs, tests, and docs in the same phase. Per project "fix discrepancies, never work around" policy.

---

## Wrapper seam dispatch + normalization (CHK-09)

| Option | Description | Selected |
|--------|-------------|----------|
| Dispatch to existing validators + per-seam normalization | seam1→validate_projection, seam3→json_ui_verify_action, seam4→render_projection+json_ui_validate_spec, seam5→validate_contracts; translate to `Finding` at boundary | ✓ |
| Reimplement checks in checkpoint | Inline the validation logic | |

**Choice:** Dispatch + normalize (recommended, mandated by CHK-09). Research flag: `json_ui_verify_action` exposes only `find_handler` — entry point TBD by researcher.

---

## Seam cascade

| Option | Description | Selected |
|--------|-------------|----------|
| Locked cascade from STATE.md | seam1 fail→4,5 not_checked; seam4 fail→5 not_checked; seams 2,3 independent | ✓ |

**Choice:** Locked cascade (recommended). Coverage-honesty invariant preserved.

---

## Inline verdict summary format (CHK-07)

| Option | Description | Selected |
|--------|-------------|----------|
| Compact summary (status + fail/warn seams + next_steps) | `VerdictSummary`, no five-seam not_checked noise | ✓ |
| Full Verdict embed | Raw seams array inline | |

**Choice:** Compact summary (recommended) — SC-1 forbids five empty `not_checked` entries. Research flag: json_ui_generate anchor projection TBD.

---

## Ambient status surfacing (CHK-08)

| Option | Description | Selected |
|--------|-------------|----------|
| Add fields read from cache | `ModelCoverage.checkpoint_status` + `ApplicationInfo.projection_checkpoint` summary | ✓ |
| Separate new tool | A dedicated status tool | |

**Choice:** Extend existing tools (recommended, mandated by CHK-08). Keyed by projection function name; missing cache → "unverified".

---

## Stale-ok cache read

| Option | Description | Selected |
|--------|-------------|----------|
| Read cache only, no recompute | Ambient consumers never call run_for | ✓ |
| Always-fresh recompute | Recompute on every ambient query | |

**Choice:** Stale-ok read (recommended, locked freshness decision) — recompute I/O cost unacceptable for frequently-called `application_info`; inline hook keeps cache fresh.

## Claude's Discretion

- VerdictSummary field layout; seam-3 entry strategy; shared cache-read helper location.

## Deferred Ideas

- Phase 196: dogfood acceptance, poisoned fixture, zero-finding-seam demotion, next_steps cap 10→5.
- Phase 194 IN-02 polish (DataType in D-06 warn subject).
