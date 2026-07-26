---
phase: 260-live-reactive-fragment
plan: 03
subsystem: ui
tags: [ferro-json-ui, javascript-runtime, websocket, live-fragment, sc3]

# Dependency graph
requires:
  - phase: 260-02
    provides: LiveFragmentProps struct and render_live_fragment renderer in ferro-json-ui
provides:
  - "ferro-json-ui/src/runtime/live_fragment.rs with setupLiveFragments SOURCE const"
  - "live_fragment::SOURCE wired into FERRO_RUNTIME_JS assembly"
  - "setupLiveFragments registered in ferroRuntime() dispatcher"
  - "SC3 static proof: no WebAssembly, no useState, no eval in the client runtime"
affects: [260-04, 262-mcp-catalog-docs-publish]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "pub(super) const SOURCE raw-string pattern for per-concern JS setup functions"
    - "One shared WebSocket per page for all live fragments (D-03)"
    - "msg.event === 'fragment' filter to ignore raw delta events from ferro-broadcast"

key-files:
  created:
    - ferro-json-ui/src/runtime/live_fragment.rs
  modified:
    - ferro-json-ui/src/runtime/mod.rs

key-decisions:
  - "D-03: one shared /_ferro/ws socket for all fragments on page; no per-fragment socket"
  - "SC3: client runtime has no WebAssembly, no client-side state, no eval — asserted statically by live_fragment_runtime_no_wasm_no_state test"
  - "Reconnect strategy deferred to a future phase (mirrors sse.rs posture)"

patterns-established:
  - "SC3 inline test pattern: assert absence of WebAssembly/useState/eval in SOURCE const"
  - "WS URL constructed from location.host + fixed path, no body attribute (unlike SSE data-sse-url)"

requirements-completed: [LIVE-02]

# Metrics
duration: 3min
completed: 2026-07-26
---

# Phase 260 Plan 03: Live reactive fragment client runtime Summary

**setupLiveFragments JS runtime: one shared /_ferro/ws socket, per-channel Subscribe, innerHTML swap on fragment event — no WASM, no client state (SC3 statically proven)**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-07-26T15:49:16Z
- **Completed:** 2026-07-26T15:51:49Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Created `ferro-json-ui/src/runtime/live_fragment.rs` with `setupLiveFragments` SOURCE const: scans `[data-live-fragment]`, opens one shared `/_ferro/ws` WebSocket, sends `Subscribe` per channel on open, swaps `innerHTML` on `msg.event === 'fragment'`
- SC3 no-WASM/no-state test (`live_fragment_runtime_no_wasm_no_state`) statically asserts the SOURCE has no `WebAssembly`, no `useState`, no `eval(`
- Wired `live_fragment::SOURCE` into `FERRO_RUNTIME_JS` at all four required sites in `runtime/mod.rs`: mod decl, push_str, dispatcher array, both enumerating tests
- All 19 runtime tests pass: 18 in `runtime::tests` + 1 SC3 in `runtime::live_fragment::tests`; `bundle_is_single_iife` passes; `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` clean

## Task Commits

1. **Task 1: Create runtime/live_fragment.rs SOURCE const + SC3 no-WASM test** - `3d59ac2d` (feat)
2. **Task 2: Wire setupLiveFragments into runtime/mod.rs (4 sites) + update enumerating tests** - `23438647` (feat)

## Files Created/Modified

- `ferro-json-ui/src/runtime/live_fragment.rs` — `pub(super) const SOURCE` raw string with `setupLiveFragments` JS function + SC3 assertion test
- `ferro-json-ui/src/runtime/mod.rs` — mod decl, `push_str(live_fragment::SOURCE)`, dispatcher array entry, both enumerating test arrays updated

## Decisions Made

None beyond what D-03 specifies — plan executed exactly as written. The WS URL is constructed from `location.host + '/_ferro/ws'` (no body attribute) matching the pattern note in the plan interfaces block.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Known Stubs

None — this plan delivers the complete client runtime. The `error` handler intentionally has no reconnect logic (deferred by design, documented in source comment).

## Next Phase Readiness

- Plan 03 complete: the client transport consumer half of the LiveFragment killer feature is in place
- Plan 04 (ferro-projection hook seam + integration test) can now proceed — it is the server-side complement
- Phase 262 owns `generation_context`, ferro-mcp mirror count bump, and `docs/src` coverage

---
*Phase: 260-live-reactive-fragment*
*Completed: 2026-07-26*
