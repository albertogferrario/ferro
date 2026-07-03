---
phase: 251-component-variant-discipline-interactive-state-pass
plan: 02
subsystem: ui
tags: [json-ui, design-system, focus-ring, motion-tokens, interactive-states, tailwind]

# Dependency graph
requires:
  - phase: 250-token-vocabulary-v2-default-theme-refresh
    provides: "--color-ring token + ring-ring utility, duration-fast/base/slow @utility classes, ease-base, 0.01ms reduced-motion collapse"
  - phase: 251-component-variant-discipline-interactive-state-pass (plan 01)
    provides: canonical Variant/Tone/Size enums; StatCard `tone` prop (wire/schema-only handoff)
provides:
  - Shared interactive-base class constants in ferro-json-ui/src/render/classes.rs (FOCUS_RING, MOTION_FAST, MOTION_BASE, DISABLED_BASE, INTERACTIVE_BASE) with a composition drift-guard test
  - Every interactive render site emits focus-visible:ring-ring (token), fast/base-tier motion, and hover; controls carry the uniform disabled treatment
  - Zero raw duration-150/duration-300/motion-reduce:transition-none/focus-visible:ring-primary in render/layout/runtime code
  - Toast dismissal via transitionend + 500ms fallback (OQ-5); toast/tab JS in lockstep with SSR class literals
  - StatCard tone renderer accent (Plan 01 Known Stub resolved)
affects: [251-03 drift guard + catalog/mcp sweep, 251-04 migration docs + ferro-base.css regen, 252 design lint]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Shared pub(crate) const class fragments composed via format!/concat! — every emitted class stays a complete literal in crate source (Tailwind @source scanner contract)"
    - "Composition drift guard: INTERACTIVE_BASE == MOTION_FAST + ' ' + FOCUS_RING asserted in tests"
    - "Anchored-control disabled contract: skip the anchor wrap, emit aria-disabled + literal pointer-events-none opacity-50"

key-files:
  created:
    - ferro-json-ui/src/render/classes.rs
  modified:
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/render/data.rs
    - ferro-json-ui/src/render/form.rs
    - ferro-json-ui/src/layout.rs
    - ferro-json-ui/src/runtime/toasts.rs
    - ferro-json-ui/src/runtime/mod.rs

key-decisions:
  - "SegmentedControl segments use an INSET ring (focus-visible:ring-inset, no offset) — the cluster container is overflow-hidden and an offset ring would be clipped (D-14 compact-control discretion)"
  - "StatCard tone accent = value text color + icon color (text-success/warning/destructive); neutral emits exactly today's markup (text-text value, uncolored icon span)"
  - "Collapsible chevron keeps transition-transform but at duration-base ease-base (base tier — disclosure reveal, not a hover)"
  - "Compact icon buttons (notification toggles, kebab triggers, sidebar toggle) gained rounded-md alongside the ring so the focus ring hugs a rounded shape"
  - "OQ-4 non-addition: modal and dropdown open/close remain unanimated — no enter/leave animation was invented (decorative, out of scope)"

metrics:
  duration: 18min
  completed: 2026-07-03

requirements-completed: [DS-04]
---

# Phase 251 Plan 02: Interactive-State Pass Summary

**Every interactive JSON-UI render site now sources one shared set of class constants — `focus-visible:ring-ring` token ring, `duration-fast`/`duration-base` + `ease-base` motion tiers, uniform `disabled:opacity-50 disabled:pointer-events-none` — with the toast runtime dismissing via `transitionend` in lockstep with the SSR literals.**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-07-03T13:00:37Z
- **Completed:** 2026-07-03T13:18:00Z
- **Tasks:** 3/3
- **Files modified:** 9 (1 created)

## Accomplishments

- `render/classes.rs` is the single source of truth for the interactive base: `FOCUS_RING`, `MOTION_FAST`, `MOTION_BASE`, `DISABLED_BASE`, `INTERACTIVE_BASE`; a test pins `INTERACTIVE_BASE == MOTION_FAST + " " + FOCUS_RING` so the fragments cannot drift apart. The three seed duplicates (Button base in atoms.rs, `button_variant_classes` in containers.rs, layout sidebar nav) are compositions of it.
- All 22 `focus-visible:ring-primary` sites migrated to `ring-ring`; all `duration-150`/`duration-300` replaced by `duration-fast`/`duration-base` + `ease-base`; every `motion-reduce:transition-none` deleted (Phase 250's 0.01ms collapse owns reduced motion — D-15). Final grep: zero hits in non-test render/layout/runtime code across ferro-json-ui and framework.
- Every ✗ gap from the RESEARCH inventory filled: EmptyState CTA, Checklist dismiss, NotificationDropdown trigger, notification item links, Header logout, CalendarCell (ring on the navigating anchor), ActionCard, ProductTile ±, Modal close, Collapsible summary, ActionGroup kebab + menu items, SegmentedControl segments, SidebarLayout nav, data-table cell links / mobile cards / row-action kebab / dropdown menu items, layout sidebar toggle / notification toggles / logout / nav helpers.
- Disabled uniformity (D-16): form controls swapped `disabled:cursor-not-allowed` for `disabled:pointer-events-none`; Button carries `DISABLED_BASE` in its base, and a disabled GET-action Button is no longer anchor-wrapped — it renders the bare button with `aria-disabled="true"` + literal `pointer-events-none opacity-50` (closes RESEARCH Pitfall 3's still-navigates hole).
- Toast: SSR fade is `MOTION_BASE` (`transition-opacity duration-base ease-base`); JS emits the identical literal and dismissal switched from `setTimeout(300)` to a `transitionend` listener with a 500ms fallback timer (OQ-5) — no stuck nodes under themable durations or reduced motion. Runtime guard now rejects `duration-300`/`duration-150` and requires `duration-base`/`transitionend` in the bundle.
- Tab JS lockstep verified unchanged: the toggled literals (`border-primary`/`border-transparent`/`text-primary font-semibold`/`text-text-muted hover:text-text`) were not touched by the SSR class pass.
- StatCard `tone` renderer accent shipped (Plan 01 Known Stub): tone tints the value text and icon (`text-success`/`text-warning`/`text-destructive`); `neutral` reproduces the previous markup byte-for-byte.
- `cargo test -p ferro-json-ui` 628+28 tests green; crate clippy `--all-targets --all-features -D warnings` clean; fmt clean.

## Task Commits

1. **Task 1: Shared interactive-base constants + seed dedupe** - `48d5331b` (feat)
2. **Task 2: Per-file interactive pass + toast/tab lockstep** - `e27d8c9e` (feat)
3. **Task 3: Token-class test assertions on previously-missing states** - `26376378` (test)

## Ring / Hover / Motion Choices (recorded per plan output spec)

| Family | Ring | Hover | Motion tier |
|--------|------|-------|-------------|
| Buttons, links, nav items, menu items, CTAs | `ring-2` + `ring-offset-2` (`FOCUS_RING`) | existing per-variant / `hover:bg-surface` family | fast |
| SegmentedControl segments | `ring-2` + **`ring-inset`** (offset would clip in the `overflow-hidden` cluster) | `hover:bg-surface hover:text-text` (existing) | fast |
| Form controls (input/textarea/select/checkbox) | non-error `FOCUS_RING`; error stays `ring-destructive` | n/a | fast |
| Switch | non-error `peer-focus:ring-ring/30`; error stays `peer-focus:ring-destructive/30` | n/a | (after:transition-all unchanged) |
| Toast fade | n/a | n/a | base (`MOTION_BASE`) |
| Collapsible chevron | (summary carries `INTERACTIVE_BASE`) | `hover:bg-card` (existing) | base (`transition-transform duration-base ease-base`) |
| Compact icon buttons (toggles, kebabs) | `FOCUS_RING` + added `rounded-md` | `hover:text-text` / `hover:bg-surface` (existing) | fast |

## New/changed class literals Plan 04's ferro-base.css regen must surface

`focus-visible:ring-ring`, `focus-visible:ring-inset`, `peer-focus:ring-ring/30`, `duration-fast`, `duration-base`, `ease-base`, `disabled:opacity-50`, `disabled:pointer-events-none`, `pointer-events-none`, `opacity-50`, `transition-transform` (chevron, now token-timed), StatCard accents `text-success`/`text-warning`/`text-destructive` (existing token family), `border-l-success` (Plan 01 ActionCard arm). All appear as complete literals in crate source (scanner contract held; no dynamic class construction introduced).

## OQ Resolutions

- **OQ-4:** Interpreted DS-04 as "existing transitions use tokens + every component meets hover/focus/disabled". Modal and dropdown open/close have NO animation today and none was added — explicit non-addition.
- **OQ-5:** Toast dismissal is `transitionend`-driven with a 500ms fallback timeout; reduced motion still fires the event (0.01ms collapse, not `none`).

## Deviations from Plan

### Auto-fixed / adjusted

**1. [Task-boundary adjustment] INT-07 test flip folded into Task 1**
- **Issue:** The plan sequenced the INT-07 layout assertion flip into Task 3, but Task 1's seed collapse breaks it immediately — violating the tests-green-per-commit gate.
- **Fix:** Flipped `ring-primary`→`ring-ring`, `duration-150`→`duration-fast` in Task 1's commit (same adjustment Plan 01 documented). Task 3 delivered the NEW assertions.
- **Committed in:** `48d5331b`

**2. [Rule 3 - Blocking] Temporary `#[allow(dead_code)]` between Tasks 1 and 2**
- **Issue:** Task 1 defines five constants but only consumes `INTERACTIVE_BASE`; crate clippy (`-D warnings`) rejects the four unconsumed constants, blocking the Task 1 commit gate.
- **Fix:** Per-const `#[allow(dead_code)]` in Task 1, removed in Task 2 when every constant gained lib consumers. Also added the composition drift-guard test in classes.rs (permanent value, not a workaround).
- **Committed in:** `48d5331b` (added), `e27d8c9e` (removed)

**3. [Plan-vs-tree drift] form.rs positive `ring-primary` test assertions did not exist**
- **Issue:** The plan (from pre-251-01 line numbers) expected form.rs tests at :941-942/:1121-1122 pinning `focus-visible:ring-primary` to flip. Post-Plan-01, only destructive-ring (error) assertions exist — nothing to flip.
- **Fix:** Preserved the error-ring assertions unchanged; added NEW positive non-error assertions (`ring-ring`, `disabled:pointer-events-none`) instead, and repaired the now-vacuous negative `!contains("peer-focus:ring-primary/30")` → `!contains("peer-focus:ring-ring/30")` so the error-pill exclusion stays meaningful.
- **Committed in:** `26376378`

**4. [Rule 2 - Missing critical] layout disabled nav arm migrated to `pointer-events-none`**
- **Found during:** Task 1 (seed collapse touched the adjacent disabled arm)
- **Issue:** `cursor-not-allowed` on the aria-disabled sidebar nav span contradicts the D-16 uniform treatment this plan establishes.
- **Fix:** `opacity-50 pointer-events-none select-none` in both layout.rs and the atoms.rs Sidebar duplicate.
- **Committed in:** `48d5331b`, `e27d8c9e`

---

**Total deviations:** 4 (1 task-boundary, 1 Rule 3, 1 plan-vs-tree drift, 1 Rule 2)
**Impact on plan:** None on scope — all class transformations landed exactly as specified; test-surface work was reshaped to match the actual post-251-01 tree.

## Known Stubs

None. The Plan 01 StatCard `tone` stub is resolved by this plan.

## Threat Model Compliance

- T-251-02 (toast/tab JS class toggling): JS toggles only fixed class-string literals; `dismissToast`'s `transitionend` handler removes the node and renders no attribute-derived markup. Verified in review of `runtime/toasts.rs`/`tabs.rs`.
- T-251-03 (labels in migrated sites): every `html_escape` call preserved — edits touched class attributes only.
- T-251-05 (toast DoS under reduced motion): 500ms fallback timer guarantees node removal even if `transitionend` is missed.

## Issues Encountered

- Two new test fixtures needed wire-format corrections (`HttpMethod` is uppercase `"GET"` on the wire; `ProductTileProps.product_id` is a required string) — fixed inline during Task 3.

## User Setup Required

None.

## Next Phase Readiness

- Plan 03 (drift guard + catalog/mcp sweep) can rely on: zero old motion/ring vocabulary anywhere in render/layout/runtime code; the runtime guard already rejects `duration-150`/`duration-300`/`data-toast-variant`.
- Plan 04 must regenerate `ferro-base.css` (D-04) — the new-literal list above is the smoke-check target (`grep -c "focus-visible:ring-ring\|ring-inset\|duration-fast" ferro-json-ui/assets/ferro-base.css` after regen).
- Workspace-wide `cargo test` beyond ferro-json-ui not run this plan (phase gate after Plans 02/03); no other crate pins the retired class strings (grep-verified across framework/app/ferro-mcp/ferro-cli).

---
*Phase: 251-component-variant-discipline-interactive-state-pass*
*Completed: 2026-07-03*

## Self-Check: PASSED

All 10 claimed files exist on disk; commits 48d5331b, e27d8c9e, 26376378 verified in git log.
