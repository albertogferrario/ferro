---
phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test
plan: 02
subsystem: ui
tags: [json-ui, runtime, javascript, tabs, url-params]

requires:
  - phase: 175-01
    provides: F1/F2 runtime patches (dismissible buttons, field-value expression)

provides:
  - initTabFromUrl(container, triggers, panels) JS function in the tabs IIFE
  - ?tab=<name> URL-driven tab activation at DOMContentLoaded with no flash
  - runtime_contains_init_tab_from_url assertion test in ferro-json-ui

affects: [ferro-json-ui runtime consumers, gestiscilo staff domain tab navigation]

tech-stack:
  added: []
  patterns:
    - "URL-driven tab init via URLSearchParams mirrors initToastFromUrl pattern — synthetic handler call reuses existing DOM toggle logic"
    - "Red-state test first: write assertion test before implementation, confirm failure, then implement"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/runtime/tabs.rs
    - ferro-json-ui/src/runtime/mod.rs

key-decisions:
  - "D-F3-tabs honored: client-side IIFE only — no server-side conditional panel rendering"
  - "initTabFromUrl validates ?tab= against server-rendered [data-tab] triggers before acting (returns early on no-match — T-175-02-01 mitigation)"
  - "Synthetic event object passed to makeTabHandler reuses existing toggle logic with no DOM logic duplication"

patterns-established:
  - "URL-param-driven boot init: URLSearchParams.get() → validate against rendered DOM → invoke existing handler — same shape as initToastFromUrl"

requirements-completed: []

duration: 6min
completed: 2026-05-20
---

# Phase 175 Plan 02: initTabFromUrl — URL-Driven Tab Activation at Boot Summary

**tabs IIFE extended with initTabFromUrl: reads ?tab=<name> via URLSearchParams at DOMContentLoaded, validates against server-rendered triggers, activates matching tab via synthetic makeTabHandler call — no flash, no server roundtrip**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-05-20T18:31:10Z
- **Completed:** 2026-05-20T18:37:07Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Extended `setupTabs` IIFE with `initTabFromUrl(container, triggers, panels)` called per container after click-handler wiring
- `?tab=<name>` URL param now selects the matching tab at DOMContentLoaded — no flash of the server-rendered default
- Existing click-handler behavior (`makeTabHandler`) fully preserved; no DOM logic duplicated
- Red-state assertion test added first (`runtime_contains_init_tab_from_url`) and confirmed failing before implementation
- Full pre-commit suite (fmt + clippy + all tests) green

## Task Commits

1. **Task 1: Add red-state test asserting initTabFromUrl is in FERRO_RUNTIME_JS** - `0fea1be2` (test)
2. **Task 2: Add initTabFromUrl to tabs.rs SOURCE and call it from initTabContainer** - `789563f2` (feat)

**Plan metadata:** committed below (docs)

## Files Created/Modified

- `ferro-json-ui/src/runtime/tabs.rs` — added `initTabFromUrl(container, triggers, panels)` function and call from `initTabContainer`
- `ferro-json-ui/src/runtime/mod.rs` — added `runtime_contains_init_tab_from_url` assertion test

## Decisions Made

- D-F3-tabs from CONTEXT.md honored: client-side IIFE, no server changes. The `render_tabs` hidden-class emission is unchanged.
- `initTabFromUrl` validates the URL value against server-rendered `[data-tab]` triggers before toggling anything. If `?tab=<arbitrary>` has no matching trigger, the function returns early — this is the T-175-02-01 mitigation (no DOM injection, no eval, no innerHTML).
- Synthetic call to `makeTabHandler` with a minimal `currentTarget` shim reuses all existing toggle logic. Single source of truth for tab state changes (per D-F3-tabs).

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- F3 closed. `ferro-json-ui` runtime bundle now honors `?tab=<name>` at boot.
- Gestiscilo staff domain tab navigation (friction finding F3) is resolved by this patch.
- Plans 175-03 through 175-06 can proceed.

## Known Stubs

None — `initTabFromUrl` is fully wired. The function reads live `window.location.search`, validates against real DOM, and delegates to the existing handler.

## Threat Flags

None — all trust boundaries addressed inline per the plan's threat model (T-175-02-01 validated, T-175-02-02 and T-175-02-03 accepted).

## Self-Check: PASSED

- `ferro-json-ui/src/runtime/tabs.rs` exists and contains `function initTabFromUrl` and `initTabFromUrl(container, triggers, panels)` call site
- `ferro-json-ui/src/runtime/mod.rs` exists and contains `fn runtime_contains_init_tab_from_url`
- Commits `0fea1be2` and `789563f2` verified in git log

---
*Phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test*
*Completed: 2026-05-20*
