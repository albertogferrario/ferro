---
phase: 251-component-variant-discipline-interactive-state-pass
plan: 04
subsystem: ui
tags: [json-ui, docs, migration-table, design-system, tailwind, ferro-base-css]

# Dependency graph
requires:
  - phase: 251-component-variant-discipline-interactive-state-pass (plan 01)
    provides: canonical Variant/Tone/Size/CardAppearance enums + documented visual deltas (migration-table content source)
  - phase: 251-component-variant-discipline-interactive-state-pass (plan 02)
    provides: interactive-state class pass + the confirmed list of new emitted literals the regen must surface
  - phase: 251-component-variant-discipline-interactive-state-pass (plan 03)
    provides: D-19 drift guard + canonical agent surface (prompt/docs consistency target)
provides:
  - Public `Component vocabulary migration` table (D-17) in docs/src/json-ui/components.md — the gestiscilo Phase 232 adoption reference
  - Docs "Shared Enum Values" collapsed to exactly variant/tone/size; component-scoped enums split into their own section
  - Regenerated ferro-base.css covering every emitted interactive/tone class (border-l-success, disabled:pointer-events-none, peer-focus:ring-ring/30, focus-visible:ring-inset newly surfaced)
  - Full CI-exact workspace gate green (fmt + clippy --all --all-targets --all-features + test --all-features)
affects: [252 design lint, 253 MCP surface + publish, gestiscilo-232 adoption]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Docs enum vocabulary mirrors the agent prompt surface: variant = weight, tone = status, size = sm/md/lg — one word, one meaning"

key-files:
  created: []
  modified:
    - docs/src/json-ui/components.md
    - docs/src/json-ui/actions.md
    - docs/src/json-ui/forms.md
    - ferro-json-ui/assets/ferro-base.css

key-decisions:
  - "input.css unchanged — safelist audit confirmed every emitted class is a complete source literal (dynamic-construction grep 0 hits); no @source inline additions needed"
  - "Retired classes (duration-150/300, ring-primary family) still appear as dead utility definitions in ferro-base.css — leaked from NEGATIVE test assertions the Tailwind scanner cannot distinguish from emitted classes; left as-is (unreferenced, ~200 bytes) rather than string-splitting valuable guard assertions"
  - "Visual checkpoint auto-approved under the --auto chain with HTML/CSS-level evidence (no Chrome MCP tools in executor session); no defect found at the markup/class level"

requirements-completed: [DS-03, DS-04]

# Metrics
duration: ~41min (excl. background test-gate wait)
completed: 2026-07-03
---

# Phase 251 Plan 04: Migration Docs + ferro-base.css Regen + Phase Gate Summary

**Public docs now carry the D-17 old→new migration table and a canonical three-enum vocabulary section, ferro-base.css is regenerated with every newly-emitted interactive/tone class, and the full CI-exact workspace gate is green — closing Phase 251.**

## Performance

- **Duration:** ~41 min active (13:41:42Z → 14:22:55Z)
- **Tasks:** 3/3 (Task 3 checkpoint auto-approved under the auto chain; see Deviations)
- **Files modified:** 4

## Accomplishments

- **D-17 migration table shipped** (`docs/src/json-ui/components.md` § "Component vocabulary migration"): every renamed prop and value old → new (Button/ActionGroup `default→primary`, `link→ghost`; `xs→sm`/`default→md` sizes; Alert/Toast/Badge/ActionCard `variant→tone` with value maps; Card `variant→appearance`; DataTable badge cell `{tone,label}`; MediaCardGrid `badge_variant_key→badge_tone_key`; ConfirmDialog/Notify `variant→tone`), plus the behavior/visual delta notes from the Plan 01/02 SUMMARYs (neutral badge outlined, Alert neutral on `bg-surface`, ActionCard `border-l-border`, link-style removal) and the OQ-3 `dot_colors` raw-Tailwind note.
- **Docs enum section collapsed to exactly three enums** — `variant` (weight) / `tone` (status) / `size` (sm/md/lg) — with component-scoped enums (`card_appearance`, `column_format`, `gap_size`, …) split into their own "Component-Specific Enum Values" section. Grep sweep: zero `button_variant`/`badge_variant`/`alert_variant`/`toast_variant`/`card_variant`/`"xs"`/`ring-primary` mentions in docs/src/json-ui outside the migration table.
- **Pre-existing doc drift fixed in the same sweep:** `column_format` gains the missing `badge`/`image`/`icon` formats (+ a `{tone,label}` badge-cell shape note in DataTable), GapSize's nonexistent `"xs"` removed, StatCard's `tone` prop documented, Card badge pill prose de-jargonized. actions.md ConfirmDialog/Notify migrated to `tone`; forms.md Alert builder example now `.prop("tone", "destructive")`. mdBook builds clean.
- **ferro-base.css regenerated once, after all class changes (D-04):** 64,388 bytes. Smoke greps (minified selectors escape `:` as `\:`): `ring-ring` 1, `duration-fast/base/slow` 1 each, `ease-base` 1, `border-l-success` 1 (NEW), `disabled\:pointer-events-none` 1 (NEW), `peer-focus\:ring-ring` 1 (NEW), `focus-visible\:ring-inset` 1 (NEW), `border-l-border` 1, StatCard accents `text-success/warning/destructive` 1 each. Dynamic-construction grep (`format!("...bg-{`) = 0 hits; input.css safelist already complete — no changes.
- **Full CI-exact gate green:** `cargo fmt --all -- --check` exit 0; `cargo clippy --all --all-targets --all-features -- -D warnings` finished clean (4m33s); `cargo test --all-features` completed with all-ok results (disk checked first: 38 GiB free, target/ pre-cleaned). The named gate tests re-confirmed explicitly on the warm build: `variant_tone_size_enum_sets_drift_guard` 1/1 ok, ferro-mcp `json_ui` suite 47/47 ok.
- **Schema churn discarded per the audit finding:** after the full test run, `docs/protocol/schemas/{protocol,service-def}.json` showed diffs — inspected and confirmed to be v16.3 CRUD-surface export churn (`creatable`/`updatable`/`deletable`/`mcp_write_ability`/`table`/`soft_delete_column`), zero variant/tone/size content → `git checkout docs/protocol/schemas/` (not folded into phase commits).

## Task Commits

1. **Task 1: Docs enum section + migration table + stale-value sweep** - `44292814` (docs)
2. **Task 2: Regenerate ferro-base.css, safelist verified complete** - `dee32d58` (chore)
3. **Task 3: Full CI-exact gate + visual evidence pass** - no code changes (gate + evidence only)

## Visual Checkpoint Evidence (auto-approved under --auto chain)

No Chrome MCP tools were available in the executor session, so the light+dark screenshot pass was replaced with served-HTML/CSS class-level verification against the running sample app (`target/debug/app` on :8090, started with orchestrator authorization, stopped after):

- `GET /auth/login` 200 (~40 KB): canonical classes present in markup — `focus-visible:ring-ring` (1), `duration-fast` (4), `ease-base` (2), `disabled:pointer-events-none` (1), elevated Card `shadow-md`. Retired classes all 0: `focus-visible:ring-primary`, `duration-150`, `duration-300`, `data-toast-variant`, `motion-reduce:transition-none`.
- **Light + dark both defined:** injected theme styles carry `--color-ring: oklch(55% 0.2 250)` (light) and `oklch(65% 0.18 250)` (dark).
- Served `/_ferro/ferro-base.css?v=0.2.83` is byte-identical to the regenerated file (64,388 bytes) and contains all required utilities.
- Error path exercised (`POST /auth/login` with unknown email → 422 re-render): `ring-destructive` on the errored input, error message rendered, submit button `variant: primary`, zero `ring-primary`.

A human pixel-level pass (focus-ring on tab, hover/transition smoothness, dark-mode contrast) remains worthwhile at the Phase 253 pre-publish review, but no defect is detectable at the markup/class level.

## Deviations from Plan

**1. [Checkpoint disposition] Task 3 `checkpoint:human-verify` auto-approved**
- The run is inside an `--auto` chain (orchestrator-confirmed); Chrome MCP tools were unavailable in the executor session. Per the orchestrator's instruction, the checkpoint was treated as auto-approved after recording the HTML/CSS-level evidence above. No visual defect found; nothing blocked.

**2. [Observation, intentionally not fixed] Retired utility definitions leak into ferro-base.css from negative test assertions**
- **Found during:** Task 2 smoke greps
- **Issue:** `duration-150`, `duration-300`, `ring-primary`, `peer-focus:ring-primary/30` appear as utility *definitions* in the regenerated CSS. Source: negative test assertions (e.g. `assert!(!html.contains("focus-visible:ring-primary"))` in `ferro-json-ui/src/render/atoms.rs`, `runtime/mod.rs`) — the Tailwind `@source` scanner reads test code and cannot distinguish a negative assertion from an emitted class.
- **Disposition:** Left as-is. The definitions are dead (nothing in rendered markup references them — verified against the served pages), cost ~200 bytes, and the guard assertions are permanent regression value. String-splitting the assertion literals to hide them from the scanner would trade test readability for cosmetics. Candidate cleanup note for Phase 252 if the design lint grows a CSS-hygiene rule.

---

**Total deviations:** 1 checkpoint disposition + 1 documented observation. All plan scope landed as written; `input.css` needed no changes.

## D-18 Skip (recorded per plan output spec)

ferro-cli was audited for retired json-ui vocabulary: its only spec-adjacent `variant` occurrences are the shadcn React templates `ferro-cli/src/templates/files/frontend/src/{pages/Settings,layouts/AuthLayout}.tsx.tpl` — a different vocabulary (shadcn's own `variant` prop on React components, not json-ui specs). All other `variant` hits in ferro-cli Rust sources are Rust-language prose ("enum variant", "invariant"). **Audited and intentionally left unchanged.**

## Pointer for gestiscilo Phase 232

The consumer migration reference is `docs/src/json-ui/components.md` § **"Component vocabulary migration"** — every old→new prop/value row plus the expected visual deltas (neutral badge outlined, Alert info→neutral surface tint, ActionCard neutral border, link→ghost buttons) and the OQ-3 `dot_colors` raw-Tailwind caveat.

## Known Stubs

None.

## Issues Encountered

- First background `cargo test --all-features` run completed but its output redirect captured nothing (session backgrounding quirk) — re-ran in foreground with captured output rather than accepting unverifiable evidence.
- The plan's shorthand smoke grep (`grep -c "ring-ring\|..."`) undercounts on the minified CSS because class selectors escape `:` as `\:`; per-class escaped greps used instead (all present).

## User Setup Required

None.

## Next Phase Readiness

- Phase 251 is complete: 4/4 plans. Canonical vocabulary + interactive-state quality bar shipped end-to-end (enums → classes → drift guard → agent surface → docs → stylesheet), full workspace gate green.
- Phase 252 (design lint) can build on: the D-19 drift guard, the stale-prop detection handoff (Plan 03 note), the OQ-3 `dot_colors` lint candidate, and the negative-assertion CSS-leak observation above.
- No publish this phase — single publish at Phase 253 (friction-loop release cadence).

---
*Phase: 251-component-variant-discipline-interactive-state-pass*
*Completed: 2026-07-03*

## Self-Check: PASSED

All 4 claimed modified files + SUMMARY exist on disk; commits 44292814, dee32d58 verified in git log; migration table and ring-ring regen artifact confirmed present.
