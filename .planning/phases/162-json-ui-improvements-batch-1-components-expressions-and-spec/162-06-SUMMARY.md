---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
plan: "06"
subsystem: ui
tags: [json-ui, layout, auth, html, tailwind]

requires:
  - phase: 162-05
    provides: preceding wave-3 plan (triple-lockstep reconciliation)

provides:
  - AuthLayout renders structural centering only; card chrome removed (D-05)
  - auth_layout_centers_content test asserts card wrapper absent

affects:
  - gestiscilo auth specs (must declare Card as root component)
  - 162-07 (full suite gate runs after this plan)

tech-stack:
  added: []
  patterns:
    - "Layout = structural only; specs own their own card chrome (D-05 convention)"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/layout.rs

key-decisions:
  - "D-05: Remove bg-card rounded-lg shadow-md p-8 wrapper from AuthLayout; layout is structural only (centering + max-width); consumer specs must declare Card as root"
  - "D-06 (no-op): No Fragment/Group borderless container added; decision is a deletion, not an addition"

patterns-established:
  - "AuthLayout pattern: min-h-screen flex items-center justify-center / w-full max-w-md / {content} — no card chrome"

requirements-completed: []

duration: 8min
completed: 2026-05-16
---

# Phase 162 Plan 06: AuthLayout Card Wrapper Removal (D-05) Summary

**AuthLayout card wrapper removed: layout is now structural only (centering + max-width), specs must declare their own Card root (D-05)**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-05-16T00:00:00Z
- **Completed:** 2026-05-16T00:08:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Deleted the `<div class="bg-card rounded-lg shadow-md p-8">` wrapper from `AuthLayout::render`, closing the double-card friction surfaced by gestiscilo auth pages migration
- AuthLayout retains structural centering (`min-h-screen flex items-center justify-center`) and max-width container (`w-full max-w-md`); no card chrome
- Updated `auth_layout_centers_content` test: asserts structural wrappers present and card chrome absent
- Updated module-level doc comment and `AuthLayout` struct doc to reflect new behaviour
- D-06 honoured implicitly: no Fragment/Group container added (deletion-only plan)

## Task Commits

1. **Task 1: Remove card wrapper from AuthLayout.render and update test assertion** - `392b0191` (feat)

**Plan metadata:** (committed with SUMMARY below)

## Files Created/Modified

- `ferro-json-ui/src/layout.rs` - Removed `bg-card rounded-lg shadow-md p-8` div from `AuthLayout::render`; updated `auth_layout_centers_content` test assertions; updated doc comments

## Decisions Made

- D-05 implemented as a pure deletion: three lines of HTML removed from the format string, one test assertion inverted. No new abstraction introduced.
- D-06 confirmed no-op: no `Fragment`/`Group` container added.

## Deviations from Plan

None — plan executed exactly as written. The formatting fix (rustfmt expanded a single-line `assert!` into multi-line) was applied automatically by `cargo fmt` and is not a behavioural deviation.

## Issues Encountered

None. `cargo fmt`, `cargo clippy -p ferro-json-ui --all-targets -- -D warnings`, and `cargo test -p ferro-json-ui --all-features` all passed on first attempt.

## Known Stubs

None.

## Threat Flags

None. Structural HTML deletion only; no new network endpoints or auth paths introduced.

## User Setup Required

None.

## Next Phase Readiness

- 162-07 (full wave-3 suite gate) can run immediately; `cargo test -p ferro-json-ui --all-features` already passes

---
*Phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec*
*Completed: 2026-05-16*
