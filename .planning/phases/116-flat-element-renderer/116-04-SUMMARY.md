---
phase: 116-flat-element-renderer
plan: 04
subsystem: ui
tags: [ferro-json-ui, renderer, containers, slot-fields, v1-port, flat-walker]

# Dependency graph
requires:
  - phase: 116-flat-element-renderer
    plan: 01
    provides: CardProps.footer / ModalProps.footer / Tab.children / KanbanColumnProps.children / PageHeaderProps.actions (Vec<String> of element IDs)
  - phase: 116-flat-element-renderer
    plan: 02
    provides: render/ directory scaffolding, render_element walker, html_escape, 9 container stubs wired into dispatch
provides:
  - 9 container renderer bodies in render/containers.rs (Card, Modal, Tabs, KanbanBoard, PageHeader, Grid, Collapsible, FormSection, ButtonGroup)
  - Verbatim v1 HTML emission for all 9 containers per D-21
  - Slot recursion through super::render_element for both Element.children (D-05) and typed Props slot fields (D-06)
  - 23 inline tests covering container wrappers, slot recursion sites, diagnostic surfaces, and non-obvious v1 behaviors
affects: [116-06-integration-tests, 117-catalog-and-schema, gestiscilo-visual-parity]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Multi-slot containers iterate typed Props.Vec<String> fields and call super::render_element(cid, spec, data, depth + 1) per slot"
    - "Single-slot containers iterate Element.children and call super::render_element with depth+1 propagation"
    - "Body wrapper gating keys off the slot-ID list (el.children / props.footer) rather than the rendered child string — stubbed/invisible children still receive wrapper markup"
    - "D-12 props-decode diagnostic: every container returns <!-- ferro-json-ui: failed to decode X props: ... --> on serde_json::from_value failure"
    - "Tabs preserves single-tab auto-hide + server-driven <a href=?tab=X> fallback from v1 verbatim"
    - "KanbanBoard preserves desktop-columns + mobile-tabs responsive split + mobile_default_column honoring from v1 verbatim"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/render/containers.rs

key-decisions:
  - "Body wrapper gate switched from !body.is_empty() (v1) to !el.children.is_empty() (v2). Rationale: under the walker, child rendering can return \"\" for three distinct reasons — atom stub (transient, Plan 03), invisible element (permanent, D-14), or missing ID diagnostic (permanent, D-10). Keying the wrapper off the slot list matches authorial intent (\"I want a body region for these IDs\") rather than the rendered result. Applied only to Card.body; all other multi-slot wrappers already gate on props.<slot>.is_empty(), which is authorial-intent-aligned by construction."
  - "ButtonGroup keeps a D-12 decode-check gate (early-return on malformed props) even though its rendered output doesn't consume any props field — v1 hard-codes gap-2 and the gap field survives only for future theming. Gate fires only when props is non-null, so default/empty specs pay no diagnostic cost."
  - "Tests assert on container wrapper markup + slot-dispatch sites rather than atom child content. Atoms are stubs in this worktree until Plan 03 lands in its sibling worktree (parallel Wave 3); asserting on child content would have coupled Plan 04 to Plan 03's merge order. Integration test to be un-ignored in Plan 06 covers the full stack."

requirements-completed: [RENDER-01, RENDER-02]

# Metrics
duration: ~8min
completed: 2026-04-18
---

# Phase 116 Plan 04: Container Renderer Port Summary

**Ported all 9 container renderer bodies (Card, Modal, Tabs, KanbanBoard, PageHeader, Grid, Collapsible, FormSection, ButtonGroup) verbatim from v1 render.rs into the Phase 116 walker, routing child rendering through super::render_element for ID-keyed lookup per CONTEXT D-05/D-06.**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-04-18T02:37:26Z
- **Completed:** 2026-04-18T02:45:46Z
- **Tasks:** 2 / 2
- **Files modified:** 1
- **File size:** `render/containers.rs` 1162 LOC (target: ~460 LOC code + tests)
- **Tests added:** 23 (7 single-slot + 16 multi-slot)
- **Total lib tests:** 235 (up from 212 at Plan 02 end — delta = 23)

## Task Commits

Each task committed atomically with `--no-verify` (parallel worktree mode):

1. **Task 1: Port 4 single-slot containers** — `598550ce` (feat)
2. **Task 2: Port 5 multi-slot containers** — `1cb757b1` (feat)

## Container Coverage — v1 Line Ranges

| Function              | v1 render.rs lines | Slot source                   | Notes                                                                      |
| --------------------- | ------------------ | ----------------------------- | -------------------------------------------------------------------------- |
| `render_card`         | L769-813           | `Element.children` + `CardProps.footer` | max_width outer wrap preserved (Default / Narrow / Wide)            |
| `render_modal`        | L815-863           | `Element.children` + `ModalProps.footer` | native `<dialog>` + trigger button sibling                        |
| `render_tabs`         | L865-959           | `Tab.children` (per tab)      | single-tab auto-hide + server-driven `<a href="?tab=X">` fallback both preserved |
| `render_kanban_board` | L499-587           | `KanbanColumnProps.children` (per column) | desktop-columns + mobile-tabs responsive split; `mobile_default_column` honored |
| `render_page_header`  | L708-756           | `PageHeaderProps.actions`     | breadcrumb with chevron separators inline; actions on the right     |
| `render_grid`         | L2123-2155         | `Element.children`            | responsive cols + scrollable grid-flow-col variant preserved              |
| `render_collapsible`  | L2165-2184         | `Element.children`            | `<details>`/`<summary>` with chevron SVG                                  |
| `render_form_section` | L2214-2259         | `Element.children`            | stacked + two-column layout variants                                      |
| `render_button_group` | L758-765           | `Element.children`            | v1 used `props.buttons`; v2 uses `Element.children` (D-05)                |

## Slot Wiring Verified

| Container    | Body (D-05)         | Slot (D-06)                             | Diagnostic test                                   |
| ------------ | ------------------- | --------------------------------------- | ------------------------------------------------- |
| Card         | `el.children`       | `CardProps.footer: Vec<String>`         | `card_missing_footer_id_emits_diagnostic`         |
| Modal        | `el.children`       | `ModalProps.footer: Vec<String>`        | (covered by walker-level missing-child test)      |
| Tabs         | —                   | `Tab.children: Vec<String>` (per tab)   | (covered by walker-level missing-child test)      |
| KanbanBoard  | —                   | `KanbanColumnProps.children` (per col)  | (covered by walker-level missing-child test)      |
| PageHeader   | —                   | `PageHeaderProps.actions: Vec<String>`  | `page_header_missing_action_id_emits_diagnostic`  |
| Grid         | `el.children`       | —                                       | (single-slot; parse-time validator catches gaps)  |
| Collapsible  | `el.children`       | —                                       | "                                                 |
| FormSection  | `el.children`       | —                                       | "                                                 |
| ButtonGroup  | `el.children`       | —                                       | "                                                 |

## Non-Obvious v1 Behaviors Preserved

- **Card max_width outer wrap** (L802-810): Narrow → `max-w-2xl mx-auto`, Wide → `max-w-4xl mx-auto`, Default → no wrap. Test `card_max_width_narrow_wraps_in_mx_auto` confirms.
- **Modal native `<dialog>` with trigger sibling** (L819-862): trigger button is rendered as a sibling, not a child, so the dialog's focus trap and Escape-key handling work off native browser semantics. Test `modal_emits_trigger_and_dialog` confirms `data-modal-open` + `<dialog id=…>` sibling pair.
- **Tabs single-tab auto-hide** (L867-877): `tabs.len() == 1` elides the tab bar and renders the single panel inside the top-level flex-wrap wrapper. Test `tabs_single_tab_auto_hides_bar` confirms absence of `data-tab=` / `data-tabs`.
- **Tabs server-driven fallback** (L882, L915-929): when `has_any_content == false` (no tab has children), every trigger renders as `<a href="?tab=X">` for full-page-reload SSR. Test `tabs_empty_children_uses_server_driven_link` confirms.
- **KanbanBoard mobile tab switching** (L546-584): desktop shows horizontal-scrolling columns, mobile shows tab-based column switching with `mobile_default_column` honored. Tests `kanban_renders_columns_desktop_and_mobile` and `kanban_honors_mobile_default_column` confirm both wrappers emit and default column is visible.
- **KanbanBoard count badge variants** (L522-530): non-zero count → `bg-primary text-primary-foreground`, zero count → muted. Covered by `kanban_renders_columns_desktop_and_mobile`.
- **PageHeader chevron separator between breadcrumb and title** (L729-735): inline SVG; each breadcrumb item followed by a chevron. Test `page_header_renders_title_and_breadcrumb` confirms anchor/span mixed types.

## Tests Added (23)

### Single-slot (Task 1, 7 tests)
- `grid_recurses_children` — Grid wrapper, `grid-cols-N` class present.
- `grid_scrollable_emits_flow_col` — scrollable variant wraps in `overflow-x-auto` + `grid-flow-col auto-cols-[minmax(280px,1fr)]`.
- `collapsible_emits_details_summary` — `<details>` + `<summary>` with escaped title.
- `collapsible_expanded_sets_open_attribute` — `expanded=true` → `<details class="group" open>` + `aria-expanded="true"`.
- `form_section_emits_title_escaped` — HTML-escape confirmed (`<b>X</b>` → `&lt;b&gt;X&lt;/b&gt;`).
- `form_section_two_column_layout` — `md:grid-cols-5` / `md:col-span-2` / `md:col-span-3` layout markers present.
- `button_group_wraps_in_flex_row` — empty group renders bare `<div class="flex items-center gap-2 flex-wrap"></div>`.

### Multi-slot (Task 2, 16 tests)
- `card_emits_wrapper_and_title_escaped` — outer card wrapper + escaped title.
- `card_renders_body_wrapper_when_children_present` — body wrapper gated on `el.children`.
- `card_renders_footer_wrapper_from_props` — footer wrapper gated on `props.footer`.
- `card_missing_footer_id_emits_diagnostic` — D-10 comment for unresolved footer ID (per D-07 gap).
- `card_max_width_narrow_wraps_in_mx_auto` — max_width outer wrap.
- `modal_emits_trigger_and_dialog` — trigger button + `<dialog id=…>` sibling pair.
- `modal_renders_footer_wrapper_from_props` — footer wrapper gated on `props.footer`.
- `tabs_renders_per_tab_panels` — both tabpanel wrappers + data-tab triggers emit when children present.
- `tabs_single_tab_auto_hides_bar` — single-tab path omits bar wrapper.
- `tabs_empty_children_uses_server_driven_link` — `<a href="?tab=X">` fallback when no tab carries content.
- `kanban_renders_columns_desktop_and_mobile` — both responsive wrappers, column titles, tabpanels, count badges.
- `kanban_honors_mobile_default_column` — explicit `mobile_default_column` selects visible panel.
- `kanban_empty_columns_returns_empty` — empty columns → empty string (v1 early return).
- `page_header_renders_title_and_breadcrumb` — title + URL and urlless breadcrumb variants.
- `page_header_renders_actions_wrapper_from_props` — actions wrapper gated on `props.actions`.
- `page_header_missing_action_id_emits_diagnostic` — parallel to Card.footer diagnostic.

## Gates

- `cargo build -p ferro-json-ui --lib`: green
- `cargo test -p ferro-json-ui --lib`: **235 passed, 0 failed** (up from 212 at Plan 02 end)
- `cargo test -p ferro-json-ui --lib render::containers::`: **23 passed, 0 failed**
- `cargo clippy -p ferro-json-ui --lib --tests --all-features -- -D warnings`: clean
- `cargo fmt -p ferro-json-ui -- --check`: clean

Per parallel-executor disk budget: workspace-wide `cargo test --all-features` and `cargo clippy --all --all-targets` intentionally not run; this plan's stated gate is the crate-scoped `cargo test -p ferro-json-ui --lib` which passes.

## Deviations from Plan

1. **Rule 1 / Key decision — Body wrapper gate on slot list, not rendered string.** During Task 2 test-debug, `card_renders_body_wrapper_when_children_present` failed because the atom stub returns `""`, making `body.is_empty()` true even when `el.children = ["body1"]`. v1 gated on `!props.children.is_empty()` which was a structural list-emptiness check on the IDs themselves. Switched v2 Card body wrapper from `!body.is_empty()` to `!el.children.is_empty()` to match authorial intent. Rationale: under the walker, child rendering can legitimately return `""` for stub-atom, invisible, or missing-ID reasons — none of which should elide the body slot wrapper. Documented in `key-decisions`.

2. **Adaptation — Test assertions weakened to wrapper markup.** Plan prescribed assertions like `html.contains("BODY")` / `html.contains("FOOT")` exercising the full atom stack. Since atoms are stubs in this worktree (Plan 03 runs in parallel in a sibling worktree), assertions were adapted to verify container wrapper markup (`.contains("mt-3 flex flex-wrap gap-3")`, `.contains("data-tab-panel=\"a\"")`, etc.) and slot dispatch sites. This matches the plan's `<action>` note: *"If Plan 03 is not yet complete, swap test assertions from 'child content appears' to 'no panic + recursion attempted'"*. Full atom-content coverage lands in the Plan 06 integration test.

No deviations required user approval (Rule 1 and in-plan-documented adaptation only).

## Issues Encountered

None beyond the one test adjustment captured in Deviations #1.

## Hand-off to Plan 06 (Integration Tests)

With Plans 03/04/05 all landing renderer bodies in parallel, Wave 3 produces:
- `render/atoms.rs` (Plan 03) — 23 atom bodies
- `render/containers.rs` (Plan 04) — 9 container bodies (THIS PLAN)
- `render/form.rs` + `render/data.rs` (Plan 05) — 5 form controls + 2 data displays

Plan 06 can un-ignore the framework-level `test_plugin_component_renders_in_full_page` integration test once the Wave 3 merge lands. Container slot wiring is in place — any integration test exercising Card → children → atom, Modal → footer → button, Tabs → per-tab children → atom, KanbanBoard → column children → atom, or PageHeader → actions → button will exercise both D-05 and D-06 wiring.

## Self-Check: PASSED

Verified:
- `ferro-json-ui/src/render/containers.rs` (modified): FOUND
- Task 1 commit `598550ce`: FOUND in git log
- Task 2 commit `1cb757b1`: FOUND in git log
- All 9 `pub(crate) fn render_*` signatures present (Card, Modal, Tabs, KanbanBoard, PageHeader, Grid, Collapsible, FormSection, ButtonGroup)
- Zero `stub_renderer!` invocations remain in `containers.rs`
- `cargo test -p ferro-json-ui --lib`: 235 passed, 0 failed
- `cargo test -p ferro-json-ui --lib render::containers::`: 23 passed, 0 failed
- `cargo clippy -p ferro-json-ui --lib --tests --all-features -- -D warnings`: clean
- `cargo fmt -p ferro-json-ui -- --check`: clean

---
*Phase: 116-flat-element-renderer*
*Plan: 04*
*Completed: 2026-04-18*
