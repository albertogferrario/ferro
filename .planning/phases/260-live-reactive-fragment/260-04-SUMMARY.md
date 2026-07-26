---
phase: 260-live-reactive-fragment
plan: "04"
subsystem: ferro-json-ui
tags: [live-fragment, catalog-lockstep, dispatch, builtin, end-to-end, sc1, sc4, d-06]
dependency_graph:
  requires:
    - phase: 260-01
      provides: with_fragment_renderer hook seam (ferro-projection)
    - phase: 260-02
      provides: render_live_fragment + LiveFragmentProps (ferro-json-ui)
    - phase: 260-03
      provides: setupLiveFragments client runtime JS (ferro-json-ui)
  provides:
    - LiveFragment registered in BUILTIN_TYPES (ferro-json-ui render dispatch)
    - LiveFragment BUILTIN_SPECS entry + schema_for!(LiveFragmentProps) (ferro-json-ui catalog)
    - Pinned count asserts BUILTIN_TYPES.len() == 53
    - render_spec_to_html dispatches LiveFragment end-to-end (SC1 proven via public API)
    - SC4 one-binding-pattern proven + absence check (no reconciliation code)
  affects:
    - 262 (ferro-mcp mirror count + generation_context + docs — deferred per D-06)
tech_stack:
  added: []
  patterns:
    - "D-06 lockstep: BUILTIN_TYPES + dispatch arm + BUILTIN_SPECS + pinned count must all move in one wave"
    - "TDD GREEN: dispatch arm wired first (Tasks 1+2), tests prove end-to-end (Task 3)"
    - "SC4 absence check via include_str! + assert!(!src.contains('reconcile'))"
    - "#[allow(dead_code)] removed from render_live_fragment — now reachable via dispatch"
key_files:
  modified:
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/catalog.rs
    - ferro-json-ui/src/runtime/live_fragment.rs
decisions:
  - "Removed #[allow(dead_code)] from render_live_fragment as planned — dispatch arm now live"
  - "13 KB prompt budget not exceeded by LiveFragment addition — no budget bump needed"
  - "fmt fixup committed separately for clarity (pre-existing drift from Plans 02/03)"
metrics:
  duration: ~15min
  completed: "2026-07-26"
  tasks: 3
  files: 4
requirements: [LIVE-02]
---

# Phase 260 Plan 04: Catalog Lockstep + End-to-End Proof Summary

**`LiveFragment` registered as a builtin: BUILTIN_TYPES + dispatch arm + BUILTIN_SPECS + pinned count 52→53, with end-to-end render proof via the public API and SC4 one-binding-pattern absence check**

## What Was Built

- **`BUILTIN_TYPES` entry** — `"LiveFragment"` added after `"SelectionPanel"` in `ferro-json-ui/src/render/mod.rs` (count: 53).
- **Dispatch arm** — `"LiveFragment" => containers::render_live_fragment(el, spec, data, depth)` added in `render_element` match, immediately after the `SelectionPanel` arm.
- **`#[allow(dead_code)]` removed** from `render_live_fragment` in `containers.rs` — the function is now reachable via the dispatch arm; the allow was Plan 02's temporary boundary marker.
- **`LiveFragmentProps` import** added to the `use crate::component::{...}` block in `catalog.rs` (alphabetical: after `KanbanBoardProps`).
- **`BUILTIN_SPECS` tuple** for `LiveFragment` added after `SelectionPanel` in `catalog.rs`, with a structural-noun description (no domain nouns per catalog-vocabulary convention) and `schema_for!(LiveFragmentProps)`.
- **Pinned count bumped 52→53** in `builtin_types_count_drift_guard` test; history comment updated with `→ 53 (LiveFragment)`.
- **Prompt budget** — adding `LiveFragment` did NOT exceed the 13 KB limit; no bump needed. All prompt tests pass: `prompt_mentions_every_builtin`, `prompt_under_size_budget`, `prompt_is_deterministic`.
- **End-to-end integration test** — `live_fragment_end_to_end_first_paint_and_delta_use_one_render_path` in `render/mod.rs`: builds a `LiveFragment` host spec via `Spec::builder`, calls `render_spec_to_html` (public API), asserts `data-live-fragment` container, `data-channel="projection.inventory.dashboard.warehouse-a"` attribute, child text rendered; then proves D-05 single render path by asserting `delta_html_a == delta_html_b` for two identical calls.
- **SC4 absence test** — `live_fragment_ships_one_binding_pattern_no_list_reconciliation` in `render/mod.rs`: uses `include_str!("containers.rs")` to assert no `"reconcile"` or `"keyed_diff"` strings exist in the LiveFragment renderer (v17.0 one-binding-pattern invariant).
- **Fmt fixup** — three files (`containers.rs`, `render/mod.rs`, `runtime/live_fragment.rs`) had pre-existing formatting drift from Plans 02/03; `cargo fmt --all` applied in a separate commit.

## Task Commits

1. **Task 1: BUILTIN_TYPES + dispatch arm** — `fd6b3b0c` (feat)
2. **Task 2: catalog lockstep 52→53** — `f7e2fd6f` (feat)
3. **Task 3: end-to-end + SC4 tests** — `9e2d4586` (test)
4. **Fmt fixup** — `d670b2d3` (style)

## CI Gate Results

All three commands run ONE AT A TIME (thermal constraint respected):

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | CLEAN (exit 0) |
| `cargo clippy --all --all-targets -- -D warnings` | CLEAN (exit 0) |
| `cargo test -p ferro-json-ui` | 766 lib tests + 5 doc-tests — 0 failed |

## Files Created/Modified

- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/render/mod.rs` — Added `"LiveFragment"` to BUILTIN_TYPES, dispatch arm, two integration tests (Tasks 1+3)
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/render/containers.rs` — Removed `#[allow(dead_code)]` from `render_live_fragment` (Task 1)
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/catalog.rs` — `LiveFragmentProps` import + BUILTIN_SPECS tuple + count 52→53 (Task 2)
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/runtime/live_fragment.rs` — Fmt-only fix (pre-existing drift from Plan 03)

## Decisions Made

- **`#[allow(dead_code)]` removed on dispatch wiring** — Plan 02 added it as a Plan-04-boundary marker; removing it in Task 1 is the expected clean completion per the Plan 02 SUMMARY note.
- **13 KB budget not exceeded** — LiveFragment's schema is compact (`projection`, `key`, `template` fields); no budget bump needed. The history comment was NOT updated with a "52 components" → "53 components" note because the surrounding test text did not contain that exact string.
- **Fmt fixup as a separate commit** — The fmt drift was pre-existing from Plans 02/03 test code. Committing it separately keeps the task commits focused.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] cargo fmt failures in containers.rs + mod.rs + live_fragment.rs**
- **Found during:** Final CI gate (`cargo fmt --all -- --check`)
- **Issue:** Three files had formatting drift from Plans 02/03 that triggered fmt failures in the Plan 04 CI gate.
- **Fix:** `cargo fmt --all` applied; changes committed as `style(260-04)` separately from task commits.
- **Files modified:** `ferro-json-ui/src/render/containers.rs`, `ferro-json-ui/src/render/mod.rs`, `ferro-json-ui/src/runtime/live_fragment.rs`
- **Commit:** `d670b2d3`

## Known Stubs

None. The D-06 lockstep is complete: `LiveFragment` is in BUILTIN_TYPES, has a dispatch arm, has a BUILTIN_SPECS entry, and the pinned count is 53. The `ferro-mcp` mirror count and `generation_context` remain deferred to Phase 262 per D-06 — that is intentional, not a stub.

## Threat Flags

No new trust boundaries introduced. T-260-09 (XSS via delta re-render) is mitigated by the single-render-path proof in Task 3 (`delta_html_a == delta_html_b`): both first-paint and delta HTML go through the identical `render_spec_to_html` escaping path. T-260-11 (catalog drift) is structurally enforced by the now-passing lockstep guards.

## Self-Check: PASSED

| Item | Result |
|------|--------|
| `ferro-json-ui/src/render/mod.rs` — `"LiveFragment"` in BUILTIN_TYPES | FOUND |
| `ferro-json-ui/src/render/mod.rs` — dispatch arm `"LiveFragment" => containers::render_live_fragment` | FOUND |
| `ferro-json-ui/src/catalog.rs` — `LiveFragmentProps` import | FOUND |
| `ferro-json-ui/src/catalog.rs` — `schema_for!(LiveFragmentProps)` | FOUND |
| `ferro-json-ui/src/catalog.rs` — `assert_eq!(crate::render::BUILTIN_TYPES.len(), 53)` | FOUND |
| `ferro-json-ui/src/catalog.rs` — old count `52` (should be 0 occurrences) | 0 |
| Commit fd6b3b0c (Task 1) | FOUND |
| Commit f7e2fd6f (Task 2) | FOUND |
| Commit 9e2d4586 (Task 3) | FOUND |
| Commit d670b2d3 (fmt fixup) | FOUND |
| `cargo test -p ferro-json-ui` — 766 passed, 0 failed | PASSED |
| `cargo fmt --all -- --check` | CLEAN |
| `cargo clippy --all --all-targets -- -D warnings` | CLEAN |
| `ferro-mcp` mirror NOT touched | CONFIRMED |
| `ferro-base.css` NOT regenerated | CONFIRMED |
