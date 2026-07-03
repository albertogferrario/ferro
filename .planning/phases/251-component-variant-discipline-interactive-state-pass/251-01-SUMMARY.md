---
phase: 251-component-variant-discipline-interactive-state-pass
plan: 01
subsystem: ui
tags: [json-ui, serde, schemars, strum, design-system, enums]

# Dependency graph
requires:
  - phase: 250-token-vocabulary-v2-default-theme-refresh
    provides: token vocabulary v2 (--color-ring, motion tiers) that Plan 02 will consume; this plan only establishes the enum vocabulary
provides:
  - Shared canonical `Variant` (primary/secondary/outline/ghost/destructive), `Tone` (neutral/success/warning/destructive), `Size` (sm/md/lg), `CardAppearance` (bordered/elevated) enums in ferro-json-ui/src/component.rs
  - All seven per-component weight/status/size enums deleted (ButtonVariant, AlertVariant, BadgeVariant, ToastVariant, CardVariant, ActionCardVariant, old 4-value Size) plus action-level DialogVariant/NotifyVariant (OQ-1 normalized to `tone: Tone`)
  - Props renames — Alert/Toast/Badge/ActionCard `variant`→`tone`, Card `variant`→`appearance`, StatCard gains `tone` (OQ-2), MediaCardGrid `badge_variant_key`→`badge_tone_key`, DataTable badge cell `{tone, label}`
  - Facade re-exports (`ferro::{Variant, Tone, Size, CardAppearance}`), toast JS/SSR lockstep on `data-toast-tone`, VariantArray-driven wire-format guard, retired-value rejection tests
affects: [251-02 interactive-state pass, 251-03 drift guard + catalog/mcp sweep, 251-04 migration docs, 252 design lint, gestiscilo-232 adoption]

# Tech tracking
tech-stack:
  added: [strum::VariantArray derive on the three canonical enums]
  patterns: [one canonical enum per prop axis (variant=weight, tone=status, size=scale), typed Tone parse for data-driven row values with neutral fallback, VariantArray-iterated strum-serde wire-format guards]

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/action.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/render/data.rs
    - ferro-json-ui/src/runtime/toasts.rs
    - ferro-json-ui/src/runtime/mod.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-json-ui/src/projection/component_map.rs
    - ferro-json-ui/src/loader.rs
    - ferro-json-ui/src/lib.rs
    - framework/src/lib.rs
    - app/src/views/login.json
    - app/src/views/login_confirm.json
    - ferro-mcp/src/tools/json_ui_validate_spec.rs

key-decisions:
  - "Neutral badge treatment: OUTLINED (`border border-border text-text`) — reuses the old Outline look; avoids inventing a filled neutral token"
  - "Alert neutral tint: `bg-surface border-border text-text` — plan suggested `bg-muted` but no `--color-muted` token exists in the ferro-theme vocabulary; `bg-surface` is the existing neutral surface family"
  - "Toast neutral keeps the old Info classes (bg-primary/70 SSR, bg-primary JS) — zero visual change; the class/motion pass is Plan 02's scope"
  - "badge_tone_for collapses ALL field meanings to Tone::Neutral (D-09) — semantic tones are a per-value concern the schema layer cannot infer"
  - "MediaCardGrid parses row tone through the typed Tone enum with neutral fallback and routes through badge_inline_html — MCG and Badge cannot drift (T-251-01)"

patterns-established:
  - "One word, one meaning: `variant` is always visual weight, `tone` always status color, `size` always sm/md/lg — enforced by shared enums, not convention"
  - "Wire-format guards iterate strum::VariantArray VARIANTS so variant omission is structurally impossible"
  - "Retired wire values are proven rejected by negative serde tests (D-12: no aliases, clean break)"

requirements-completed: [DS-03]

# Metrics
duration: 26min
completed: 2026-07-03
---

# Phase 251 Plan 01: Canonical Variant/Tone/Size Vocabulary Summary

**Single canonical `Variant`/`Tone`/`Size`/`CardAppearance` vocabulary replaces nine per-component enums across ferro-json-ui, the framework facade, projection emitters, toast JS/SSR lockstep, and sample-app specs — retired values now fail at serde parse.**

## Performance

- **Duration:** ~26 min
- **Started:** 2026-07-03T12:16:30Z
- **Completed:** 2026-07-03T12:42:00Z
- **Tasks:** 3/3
- **Files modified:** 15

## Accomplishments

- Three shared enums (`Variant`, `Tone`, `Size`) + `CardAppearance` are now the only weight/status/size vocabulary in the crate; the compiler drove completeness — all nine old enums deleted with zero aliases (D-02/D-12).
- OQ-1 normalized: `ConfirmDialog.tone` and `ActionOutcome::Notify.tone` use the shared `Tone` (`default→neutral`, `danger→destructive`, `info→neutral`, `error→destructive`); Notify's absent-tone default stays `success` via a serde default fn — Plan 03's D-19 guard gets a zero-exclusion transitive walk.
- Toast JS/SSR lockstep renamed in one task: `data-toast-tone` attribute, `VARIANT_CLASSES` keys `neutral/success/warning/destructive`, `toast.tone || 'neutral'` fallback; runtime guard asserts the retired attribute never resurfaces.
- Wire-format guard now iterates `strum::VariantArray::VARIANTS` — the pre-existing BadgeVariant::Warning omission class of bug is structurally impossible; negative tests pin `xs`/`default`/`link`/`info`/`error` as parse errors.
- `cargo build --workspace` green; `cargo test -p ferro-json-ui` 616+ tests green (also with `--all-features`); crate clippy `--all-targets --all-features -D warnings` clean; fmt clean.

## Task Commits

1. **Task 1: Define shared canonical enums + rename all ferro-json-ui consumers** - `0f4fbe94` (feat)
2. **Task 2: Ripple through facade, sample specs, runtime guard** - `c2876297` (feat)
3. **Task 3: Canonical enum guards (VariantArray, retired values, defaults)** - `46c537c3` (test)

## Documented Visual Deltas (feed the Plan 04 migration table)

| Surface | Old | New | Delta |
|---------|-----|-----|-------|
| Badge `default`/`secondary`/`outline` | filled primary/secondary tint, outline | `tone: neutral` = **outlined** `border border-border text-text` | one consistent neutral treatment (D-09) |
| Alert `info` | `bg-primary/10 border-primary text-primary` | `tone: neutral` = `bg-surface border-border text-text` | primary tint → muted surface tint |
| ActionCard `default` | `border-l-primary` | `tone: neutral` = `border-l-border` | primary accent → plain border; `success` = `border-l-success` is a NEW arm |
| Relationship button | `ButtonVariant::Link` (underline) | `Variant::Ghost` | underline-link style removed framework-wide (D-07) |
| Button `size: xs` | `px-2 py-1 text-xs` | retired; `sm` = `px-3 py-1.5 text-sm` | xs consumers migrate to sm |
| Avatar `size: xs` | `h-6 w-6 text-xs` | retired; `sm` = `h-8 w-8 text-sm` | xs consumers migrate to sm |
| Toast `info` | `bg-primary/70` | `tone: neutral`, same classes | rename only, zero visual change |

## OQ Resolutions

- **OQ-1 (normalize action variants):** DONE — `DialogVariant`/`NotifyVariant` deleted; both fields are `tone: Tone`.
- **OQ-2 (StatCard/CalendarCell tone):** StatCard gained `#[serde(default)] tone: Tone` (schema + wire only; see Known Stubs). **CalendarCell was audit-assessed and SKIPPED** — it has no natural cell-level status axis distinct from its existing `closed` (availability hatch) and `dot_colors` (per-event color) props; adding `tone` would duplicate an existing control surface.
- **OQ-3 (CalendarCell `dot_colors`):** Out of scope — raw Tailwind class strings in row data are not a variant/tone/size prop. **Note for Plan 04 migration docs / backlog:** `dot_colors: Vec<String>` still accepts raw Tailwind color classes (e.g. `bg-blue-500`) and bypasses the semantic token vocabulary.

## Decisions Made

- Neutral badge = outlined treatment (`border border-border text-text`) — Claude's-discretion call under D-09; reuses the established Outline look rather than inventing a filled neutral.
- `badge_tone_for` returns `Tone::Neutral` for every field meaning — the projection layer cannot infer per-value semantics, so status/category/boolean badges all get the neutral pill.
- MediaCardGrid routes its footer badge through `badge_inline_html` after a typed `Tone` parse (invalid → neutral fallback, value never interpolated) — lockstep by construction.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Alert neutral uses `bg-surface`, not the plan-specified `bg-muted`**
- **Found during:** Task 1 (render_alert arms)
- **Issue:** The plan prescribed `bg-muted border-border text-text` for Alert neutral, but no `--color-muted` token exists in the ferro-theme vocabulary or the Tailwind bridge (`input.css`) — `bg-muted` would generate no utility and render unstyled.
- **Fix:** Used `bg-surface border-border text-text` (the existing neutral surface token family per D-16's "surface family" direction).
- **Files modified:** ferro-json-ui/src/render/atoms.rs
- **Verification:** grep of input.css token list; render tests green.
- **Committed in:** `0f4fbe94`

**2. [Rule 1 - Bug] ferro-mcp validate_spec test broken by the Alert prop rename**
- **Found during:** Task 2 (workspace verification)
- **Issue:** `reports_catalog_error_on_bad_variant` used `{"variant": ""}` on Alert expecting a catalog enum error; after the field rename, `variant` is an unknown prop (ignored by serde/schema) so no error fired — test failed. ferro-mcp is outside this plan's files_modified, but the breakage was directly caused by the rename.
- **Fix:** Minimal fixture rename `"variant": ""` → `"tone": ""` (same shape/intent, exactly what PATTERNS §13 prescribes). The broader ferro-mcp prose sweep stays with Plan 03.
- **Files modified:** ferro-mcp/src/tools/json_ui_validate_spec.rs
- **Verification:** `cargo test -p ferro-mcp json_ui` 47/47 green.
- **Committed in:** `c2876297`

**3. [Rule 3 - Blocking] projection builder StatCardProps initializer missing new `tone` field**
- **Found during:** Task 1 (clippy `--all-features` — the `projections` feature is non-default, so plain build/test missed it)
- **Issue:** `build_stat_card` in projection/builder.rs did not compile after StatCard gained `tone`.
- **Fix:** Initialize `tone: Tone::Neutral` (reproduces today's look per OQ-2).
- **Files modified:** ferro-json-ui/src/projection/builder.rs
- **Verification:** clippy/tests with `--all-features` green.
- **Committed in:** `0f4fbe94`

**4. [Task-boundary adjustment] Mechanical test-module updates folded into Task 1**
- **Issue:** The plan sequenced test rewrites into Task 3, leaving Tasks 1-2 commits with non-compiling `#[cfg(test)]` code — violating the project's "tests green before every commit" gate.
- **Fix:** Task 1 carried the mechanical rewrites (strum test hand-list, `card_variant_tests`→`card_appearance_tests`, action.rs assertions, render fixture renames) so every commit is test-green; Task 3 delivered the NEW guards (VariantArray iteration, retired-value negatives, defaults). Strict TDD RED commits were skipped for the same reason — a RED state cannot compile in Rust for a type-deletion refactor.

---

**Total deviations:** 4 (2× Rule 1, 1× Rule 3, 1 task-boundary adjustment)
**Impact on plan:** All fixes necessary for correctness or the per-commit quality gate. No scope creep — ferro-mcp touch was one fixture line.

## Known Stubs

| Stub | File | Reason / Resolution |
|------|------|---------------------|
| StatCard `tone` is wire/schema-only — the renderer does not yet apply the value/icon accent | ferro-json-ui/src/render/atoms.rs (render_stat_card) | Plan-prescribed (OQ-2: "No other StatCard change"). `neutral` default reproduces today's look. Plan 02 (interactive-state/class pass) applies the accent. |

## Handoffs to Plans 02/03/04 (intentionally untouched, per plan scope)

- **Plan 02:** `duration-150`/`duration-300` motion literals, `focus-visible:ring-primary` (→ `ring-ring`), disabled-treatment consolidation, shared class constants, StatCard tone accent, `ferro-base.css` regen.
- **Plan 03:** catalog.rs prose still says "info / success / warning / error variants" (Alert), "Small variant-styled label." (Badge); ferro-mcp `json_ui_catalog.rs:277` ACTION_API prose still names `NotifyVariant`; `code_templates` still emits `"variant": "default"`; D-19 schema-walking guard.
- **Plan 03 nuance (unknown-prop posture):** retired VALUES on canonical prop names hard-fail at serde (`size: xs`, `variant: link`, `tone: info`), but retired prop NAMES on renamed fields (e.g. Alert `"variant": "info"`) are silently ignored — props structs do not `deny_unknown_fields` and catalog schemas do not set `additionalProperties: false`. The D-19 guard/lint should decide whether stale-prop detection is in its scope.
- **Plan 04:** migration table content in "Documented Visual Deltas" + OQ-3 `dot_colors` note above; docs/src/json-ui "Shared Enum Values" section still documents the old sets.

## Issues Encountered

None beyond the deviations above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Compiling workspace on the canonical vocabulary; enums re-exported through `ferro::`.
- Plan 02 (interactive-state pass) and Plan 03 (drift guard) can consume `Variant`/`Tone`/`Size`/`CardAppearance` verbatim — the `<interfaces>` contract shipped unchanged except the documented `bg-muted`→`bg-surface` class substitution (enum bodies identical).
- Workspace-wide `cargo test` beyond ferro-json-ui/ferro-mcp not run this plan (phase gate runs it after Plans 02/03); build is green everywhere.

---
*Phase: 251-component-variant-discipline-interactive-state-pass*
*Completed: 2026-07-03*

## Self-Check: PASSED

All 16 claimed files exist on disk; commits 0f4fbe94, c2876297, 46c537c3 verified in git log.
