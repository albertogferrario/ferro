---
phase: 260-live-reactive-fragment
plan: 02
subsystem: ui
tags: [ferro-json-ui, live-fragment, server-push, projection, render]

# Dependency graph
requires:
  - phase: 260-01
    provides: fragment_hook seam on ProjectionRuntime (ferro-projection)
provides:
  - LiveFragmentProps struct in ferro-json-ui/src/component.rs (Serialize/Deserialize/JsonSchema, no Eq)
  - render_live_fragment fn in ferro-json-ui/src/render/containers.rs (first-paint + absent-snapshot)
affects:
  - 260-03 (client runtime — reads data-live-fragment / data-channel emitted here)
  - 260-04 (dispatch arm wires render/mod.rs to this function + catalog lockstep)
  - 262 (ferro-mcp mirror count + generation_context + docs)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "LiveFragment render: decode props → deserialize child Spec → render_spec_to_html(child, data) → wrap in data-live-fragment container"
    - "html_escape on both projection and key before interpolation into data-channel attribute (T-260-05)"
    - "D-04 absent-snapshot compliance: render_live_fragment never reads DB; caller passes {} when snapshot absent"
    - "D-05 binding engine reuse: render_spec_to_html serves both first-paint and delta re-render, no parallel syntax"
    - "#[allow(dead_code)] on render_live_fragment until Plan 04 wires the dispatch arm (Plan 02 boundary)"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/containers.rs

key-decisions:
  - "No Eq on LiveFragmentProps: serde_json::Value is PartialEq but not Eq — derive block is Debug/Clone/Serialize/Deserialize/JsonSchema only"
  - "LiveFragmentProps not re-exported from lib.rs: StreamTextProps (closest analog) is also not re-exported; internal render use only"
  - "#[allow(dead_code)] applied to render_live_fragment: test-module usage does not suppress the dead_code lint from the lib perspective; Plan 04 removes the allow when dispatch arm lands"
  - "Tests use build_spec helper + spec.elements.get() pattern (not el.clone()): ElementBuilder is not Clone, adapting from the plan's pseudo-code to the actual API"

patterns-established:
  - "render_live_fragment: props-decode error → HTML comment; template-parse error → HTML comment; no panic path"

requirements-completed: [LIVE-02]

# Metrics
duration: 5min
completed: 2026-07-26
---

# Phase 260 Plan 02: LiveFragment Renderer Summary

**`LiveFragmentProps` struct + `render_live_fragment` function: first-paint server render of a child template against a per-key projection snapshot, wrapped in `<div data-live-fragment data-channel="projection.{name}.{key}">` with html-escaped attribute segments**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-07-26T15:41:33Z
- **Completed:** 2026-07-26T15:46:04Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `LiveFragmentProps` added to `ferro-json-ui/src/component.rs`: `projection` + `key` String fields with `serde(default)`, required `template: serde_json::Value`, derived `Debug/Clone/Serialize/Deserialize/JsonSchema` (no `Eq` — `Value` is not `Eq`)
- `render_live_fragment` implemented in `ferro-json-ui/src/render/containers.rs`: decodes props, deserializes child `Spec` from `props.template`, renders child against `data` via `super::render_spec_to_html`, wraps in `data-live-fragment` container with html-escaped channel attribute
- Two unit tests: first-paint (container marker + channel + child text) and absent-snapshot D-04 (empty `{}` renders container without error comment)
- `cargo test -p ferro-json-ui render_live_fragment`: 2/2 pass
- `cargo clippy -p ferro-json-ui --all-targets -- -D warnings`: clean

## Task Commits

1. **Task 1: Add LiveFragmentProps to component.rs** — `b71e8fb1` (feat)
2. **Task 2: Implement render_live_fragment + tests** — `8d0856b2` (feat)

**Plan metadata:** _(to be committed with SUMMARY)_

## Files Created/Modified

- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/component.rs` — Added `LiveFragmentProps` struct after `StreamTextProps`
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/render/containers.rs` — Added `LiveFragmentProps` to import block, added `render_live_fragment` function + 2 unit tests

## Decisions Made

- **No `Eq` on `LiveFragmentProps`**: `serde_json::Value` is `PartialEq` but not `Eq`; the derive block follows the PATTERNS.md note exactly.
- **No lib.rs re-export**: `StreamTextProps` (closest analog) is not in the `pub use component::{...}` list; `LiveFragmentProps` follows the same pattern — internal render use only.
- **`#[allow(dead_code)]`**: Test-module references do not suppress the Rust dead_code lint from the lib perspective. Adding the attribute with a note that Plan 04 removes it when the dispatch arm lands is the correct clean fix for this plan boundary.
- **Test API adaptation**: The plan's pseudo-code used `el.clone()` on an `ElementBuilder`, which is not `Clone`. Tests use the existing `build_spec` + `spec.elements.get("root").unwrap()` idiom consistent with all other container tests.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `#[allow(dead_code)]` added to `render_live_fragment`**
- **Found during:** Task 2 (clippy gate)
- **Issue:** Plan expected test-module usage of `render_live_fragment` to satisfy `dead_code` lint, but `#[cfg(test)]` code does not suppress `dead_code` from the lib perspective. `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` emitted `error: function is never used: render_live_fragment`.
- **Fix:** Added `#[allow(dead_code)]` with a doc comment noting Plan 04 removes it when the dispatch arm is wired.
- **Files modified:** `ferro-json-ui/src/render/containers.rs`
- **Verification:** Clippy clean after adding the allow.
- **Committed in:** `8d0856b2` (Task 2 commit)

**2. [Rule 1 - Bug] Test API adapted from pseudo-code**
- **Found during:** Task 2 (writing tests)
- **Issue:** Plan test pseudo-code used `el.clone()` on `ElementBuilder`, which does not implement `Clone`. Passing a cloned `ElementBuilder` to both `Spec::builder().element()` and `render_live_fragment` is not possible.
- **Fix:** Followed the existing container test idiom: `build_spec(vec![...])` then `spec.elements.get("root").unwrap()` to obtain `&Element`. This is how every other render test in the file works.
- **Files modified:** `ferro-json-ui/src/render/containers.rs` (tests only)
- **Verification:** Both tests compile and pass.
- **Committed in:** `8d0856b2` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 — plan pseudo-code issues)
**Impact on plan:** Both fixes necessary for compilation and correctness. No scope creep.

## Issues Encountered

None beyond the two auto-fixed deviations above.

## Known Stubs

None. `render_live_fragment` is fully wired to `render_spec_to_html`; no placeholder values.

## Threat Flags

No new trust boundaries beyond those in the plan's threat model (T-260-04, T-260-05, T-260-06 — all mitigated as specified).

## Next Phase Readiness

- Plan 03 (client runtime JS — `setupLiveFragments` in `ferro-json-ui/src/runtime/`) can proceed: the `data-live-fragment` / `data-channel` container shape is now fixed.
- Plan 04 (dispatch arm + catalog lockstep) can proceed: `render_live_fragment` signature is `pub(crate) fn render_live_fragment(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String` — matches the planned dispatch arm exactly. Plan 04 must also remove the `#[allow(dead_code)]` attribute.
- `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` must remain clean through Plans 03 and 04.

## Self-Check

---
*Phase: 260-live-reactive-fragment*
*Completed: 2026-07-26*
