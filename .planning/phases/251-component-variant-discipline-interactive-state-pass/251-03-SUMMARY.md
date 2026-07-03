---
phase: 251-component-variant-discipline-interactive-state-pass
plan: 03
subsystem: ui
tags: [json-ui, design-system, drift-guard, json-schema, ferro-mcp, agent-surface]

# Dependency graph
requires:
  - phase: 251-component-variant-discipline-interactive-state-pass (plan 01)
    provides: canonical Variant/Tone/Size enums, OQ-1 action-level tone normalization (zero-exclusion walk possible)
  - phase: 251-component-variant-discipline-interactive-state-pass (plan 02)
    provides: interactive base constants (no schema impact; wave-2 dependency only)
provides:
  - D-19 schema-walking drift guard `variant_tone_size_enum_sets_drift_guard` in ferro-json-ui/src/catalog.rs — any catalog property named variant/tone/size with a non-canonical value set is a red test
  - Catalog prompt prop docs inline $ref'd enum values (canonical Variant/Tone/Size surfaced to agents instead of `<see schema>`)
  - Canonical tone wording in BUILTIN_SPECS prose (Badge/Alert/ActionCard)
  - ferro-mcp agent surface free of retired vocabulary (code_templates, validate_spec fixtures, ACTION_API prose)
affects: [251-04 migration docs, 252 design lint, gestiscilo-232 adoption]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Schema-walking drift guard: transitive $ref-resolving walker (visited-set cycle guard) over the assembled full_schema, asserting canonical value sets relationally with a non-vacuity counter"
    - "Prompt prop docs resolve local $defs enum refs — enum-typed fields render values inline, non-enum refs keep the <see schema> fallback"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/catalog.rs
    - ferro-mcp/src/tools/code_templates.rs
    - ferro-mcp/src/tools/json_ui_validate_spec.rs
    - ferro-mcp/src/tools/json_ui_catalog.rs

key-decisions:
  - "Guard walks BOTH every oneOf props subtree AND every root $defs entry directly — the strongest no-exclusions form (OQ-1 made action-level fields conform, so orphaned/hoisted defs are covered too)"
  - "Optional render_field_type $ref resolution ADDED — prompt prop docs now inline canonical enum values; prompt budget bumped 11 KB → 12 KB following the file's dated-comment bump convention"
  - "ConfirmDialog documented as `tone: Tone (neutral|destructive)` per plan (the confirm/confirm_danger builders produce exactly those two tones)"

patterns-established:
  - "Non-vacuity assertion on walkers: `checked >= 10` proves the traversal reaches properties, so a structural schema change cannot silently turn the guard into a no-op"

requirements-completed: [DS-03]

# Metrics
duration: 15min
completed: 2026-07-03
---

# Phase 251 Plan 03: D-19 Drift Guard + Agent-Surface Sweep Summary

**A transitive $ref-resolving schema walker now proves every catalog `variant`/`tone`/`size` property equals the canonical set (a future `size: xs` is a red test), catalog prompt docs inline the canonical enum values for agents, and the ferro-mcp surface carries zero retired vocabulary.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-07-03T13:23:15Z
- **Completed:** 2026-07-03T13:38:26Z
- **Tasks:** 3/3
- **Files modified:** 4

## Accomplishments

- `variant_tone_size_enum_sets_drift_guard` (catalog.rs tests): (1) asserts `$defs/Variant|Tone|Size` equal the canonical arrays in serde declaration order; (2) walks every `$defs/Element/oneOf` props subtree transitively, resolving `$ref` against the root `$defs` with a visited-set; (3) walks every root `$defs` entry directly — no exclusions, so `ActionItem.variant`, `ConfirmDialog.tone`, and `ActionOutcome::Notify.tone` inside `$defs/Action` are all asserted. The extractor handles all schemars 1.x shapes (`enum` array, `anyOf[].const`, `Option`-null unwrap, `$ref` hop) so the guard cannot be silently defeated; a `checked >= 10` counter proves non-vacuity.
- Failure mode verified during development: perturbing the expected `size` set produced `schema property 'size' carries a non-canonical value set ["sm", "md", "lg"] (canonical: [...])` — the message names the offending property and both sets.
- Plan-recommended optional extension shipped: `render_field_type` resolves `#/$defs/<Name>` refs against the component schema's local `$defs` and inlines plain-string-enum values, so the agent prompt shows `variant (Option<primary|secondary|outline|ghost|destructive>)` instead of `<see schema>`. Non-enum refs keep the fallback. A new `prompt_inlines_canonical_enum_values` test pins the behavior.
- BUILTIN_SPECS prose canonicalized: Badge "tone-styled status label", Alert "neutral / success / warning / destructive tones", ActionCard "tone-colored left border". Button/Avatar/FormSection/Input wording kept (generic prop-name usage, per plan).
- ferro-mcp surface swept: `list_view` template Button `"variant": "primary"`; validate_spec positive fixture `{"tone": "neutral"}`; ACTION_API prose `Notify { message, tone: Tone (neutral|success|warning|destructive) }` and `ConfirmDialog { ..., tone: Tone (neutral|destructive) }`. The 47-count mirror and expected-names list untouched.
- Gates green per commit: `cargo test -p ferro-json-ui` 658 tests, `cargo test -p ferro-mcp` 303+ tests, crate-scoped clippy `--all-targets --all-features -D warnings`, `cargo fmt --all -- --check`.

## Task Commits

1. **Task 1: D-19 schema-walking canonical enum-set drift guard (+ prompt $ref inlining)** - `28cc881e` (test)
2. **Task 2: Canonical tone wording in catalog BUILTIN_SPECS prose** - `b9ac504d` (docs)
3. **Task 3: Canonical vocabulary in ferro-mcp agent-facing surfaces** - `37e1137f` (fix)

## Plan Output Spec Answers

- **Optional `render_field_type` $ref resolution:** ADDED. Prop docs in `Catalog::prompt()` now show inline enum values for every field referencing a local plain-string-enum `$defs` entry (canonical Variant/Tone/Size and other unit enums like IconPosition). Per-variant-doc'd enums (anyOf-of-const shape) intentionally keep `<see schema>`.
- **Guard $ref-resolution boundary:** the full assembled schema with NO exclusions — every oneOf props subtree plus every root `$defs` entry, `$ref`-resolved transitively with a visited-set. Possible because OQ-1 (Plan 01) normalized the action-level fields to the shared `Tone`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Prompt size budget bumped 11 KB → 12 KB**
- **Found during:** Task 1 (optional $ref-enum inlining)
- **Issue:** Inlining enum values grew `prompt()` to 11,351 bytes, 87 bytes over the 11 KB budget — `prompt_under_size_budget` failed.
- **Fix:** Bumped the budget to 12 KB with the file's established dated-comment convention (fourth bump; prior: 8→9→10→11 KB). The growth is deliberate agent-surface content (canonical values visible in prop docs), not bloat.
- **Files modified:** ferro-json-ui/src/catalog.rs
- **Committed in:** `28cc881e`

**2. [Rule 2 - Missing critical] ACTION_API `.confirm`/`.confirm_danger` wording also updated**
- **Found during:** Task 3
- **Issue:** Lines adjacent to the plan's target (`.confirm(title) -> Self (default dialog)` / `(danger dialog)`) still used the retired DialogVariant vocabulary — stale agent-facing text is a bug per D-18.
- **Fix:** "(neutral dialog)" / "(destructive dialog)".
- **Files modified:** ferro-mcp/src/tools/json_ui_catalog.rs
- **Committed in:** `37e1137f`

**3. [Minor addition] `prompt_inlines_canonical_enum_values` test**
- New assertion pinning the optional feature (prompt must contain the three canonical value strings) — cheap permanent drift protection in the established string-containment guard style.
- **Committed in:** `28cc881e`

---

**Total deviations:** 3 (1 Rule 3, 1 Rule 2, 1 minor test addition)
**Impact on plan:** None on scope — all three tasks landed as specified; the budget bump is the accounted cost of the plan-recommended optional feature.

## Handoff Notes (Plan 04 / Phase 252)

- **Plan 01's unknown-prop posture nuance stands:** retired prop NAMES (e.g. Alert `"variant"`) are still serde-ignored, not rejected — the D-19 guard asserts value SETS on canonically-named properties; stale-prop detection was NOT added here (props structs do not `deny_unknown_fields`). Phase 252's design lint is the natural home for stale-prop diagnostics.
- The prompt now surfaces canonical values inline — Plan 04's docs sweep should keep `docs/src/json-ui/components.md` "Shared Enum Values" consistent with what agents see in the prompt.

## Known Stubs

None. The `<see schema>` fallback for non-enum `$ref`s is by design, not a stub.

## Threat Model Compliance

- **T-251-06 (out-of-vocabulary enum injection):** mitigated — the guard makes silent schema drift a build failure; `Catalog::validate` continues to reject unknown values at spec-load (verified by the pre-existing `reports_catalog_error_on_bad_variant` red-path test, now on `tone`).
- **T-251-07 (stale template steering agents to rejected values):** mitigated — zero retired-value mentions remain in `ferro-mcp/src/tools` (grep-verified 0 hits).
- No new security-relevant surface introduced (test + prose + template changes only).

## Issues Encountered

None beyond the deviations above.

## User Setup Required

None.

## Next Phase Readiness

- Plan 04 (migration docs + ferro-base.css regen) is unblocked; no schema or class changes were made in this plan, so the regen target list from Plan 02 is unchanged.
- Phase-gate workspace-wide `cargo test --all-features` deferred to the phase close per Plans 01/02 convention; crate-scoped suites (ferro-json-ui, ferro-mcp) and crate clippy are green.

---
*Phase: 251-component-variant-discipline-interactive-state-pass*
*Completed: 2026-07-03*

## Self-Check: PASSED

All 4 claimed modified files + SUMMARY exist on disk; commits 28cc881e, b9ac504d, 37e1137f verified in git log.
