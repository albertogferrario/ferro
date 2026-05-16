---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
plan: 03
subsystem: ui
tags: [rust, ferro-json-ui, json-ui, svg, tailwind]

# Dependency graph
requires:
  - phase: 162-01
    provides: CheckboxList component in component.rs and render/form.rs
  - phase: 162-02
    provides: DataTable URL placeholder interpolation in render/data.rs
provides:
  - SwitchProps.compact optional bool field — CSS scale-75/origin-left toggle
  - ImageProps.inline_svg optional String field — verbatim SVG emission path
  - ImageProps::inline_svg(svg, alt) factory method with safety rustdoc
affects:
  - gestiscilo settings.rs (6 SwitchProps.compact consumer sites now compile)
  - gestiscilo bar-chart SVG sites (1 ImageProps::inline_svg consumer site now compiles)
  - 162-05 (MCP catalog update — no new built-in component, no count change needed)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Optional bool CSS toggle: compact: Option<bool> + conditional class suffix in render function"
    - "Inline SVG early-return: if let Some(ref svg) = props.inline_svg { return format!(...) } before <img> path"
    - "Server-only trust boundary documented via rustdoc Safety section on the field and factory"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/form.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/projection/component_map.rs

key-decisions:
  - "D-16: compact is Option<bool> not bool — serde default skips serialization when None, backward compat preserved"
  - "D-17: parallel field inline_svg: Option<String> rather than ImageSource enum — avoids wire-format break on existing specs using src"
  - "alt text HTML-escaped in aria-label; SVG body emitted verbatim — trust boundary is server construction"

patterns-established:
  - "CSS-class toggles on optional bools: compute suffix string before format!, append to existing class list"
  - "Early-return render branches: check optional alternate render path before main path, return immediately"

requirements-completed: []

# Metrics
duration: 15min
completed: 2026-05-16
---

# Phase 162 Plan 03: SwitchProps.compact + ImageProps.inline_svg Summary

**Re-added two blast-radius props (D-16/D-17): SwitchProps.compact emits scale-75/origin-left CSS, ImageProps.inline_svg emits verbatim SVG in an aria-labelled div with no img tag**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-16T00:00:00Z
- **Completed:** 2026-05-16T00:15:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- `SwitchProps.compact: Option<bool>` field added; `render_switch` emits `scale-75 origin-left` on the outermost wrapper div only when `compact == Some(true)`
- `ImageProps.inline_svg: Option<String>` field added with safety rustdoc; `render_image` early-returns `<div aria-label="{alt}">{svg}</div>` bypassing the `<img>` path
- `ImageProps::inline_svg(svg, alt)` factory method added; round-trips correctly through serde
- `build_switch_props` in `projection/component_map.rs` updated with `compact: None` (Rule 3 blocking fix)
- 4 new tests: compact CSS class presence/absence, inline SVG no-img, alt XSS escape, factory serde roundtrip

## Task Commits

1. **Task 1: SwitchProps.compact + render_switch toggle** - `1e092a3f` (feat)
2. **Task 2: ImageProps.inline_svg + factory + render branch** - `96aa0520` (feat)

## Files Created/Modified
- `ferro-json-ui/src/component.rs` — compact field on SwitchProps, inline_svg field + impl block on ImageProps
- `ferro-json-ui/src/render/form.rs` — compact_class computation + format! in render_switch, switch_compact_adds_scale_class test
- `ferro-json-ui/src/render/atoms.rs` — inline_svg early-return branch in render_image, 2 inline_svg tests
- `ferro-json-ui/src/projection/component_map.rs` — compact: None added to build_switch_props initializer

## Decisions Made
- Used parallel field `inline_svg: Option<String>` rather than restoring the `ImageSource` enum — avoids breaking the wire format for existing specs that use `"src": "..."`. The factory `ImageProps::inline_svg(svg, alt)` sets `src: ""` so callers using the factory get the correct behavior without touching `src`.
- `compact` appended as a suffix to the outermost `space-y-1` div class rather than a separate wrapper — matches the existing render_switch string-building style.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added compact: None to build_switch_props initializer**
- **Found during:** Task 1 (GREEN phase compile)
- **Issue:** `projection/component_map.rs` line 258 constructs `SwitchProps` with a struct literal — adding a new field without a default causes a compile error on all struct-literal sites.
- **Fix:** Added `compact: None` to the existing struct initializer in `build_switch_props`.
- **Files modified:** `ferro-json-ui/src/projection/component_map.rs`
- **Verification:** `cargo test -p ferro-json-ui switch_compact_adds_scale_class` passes after fix
- **Committed in:** `1e092a3f` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking — struct literal update)
**Impact on plan:** Required for compilation. No scope creep.

## Issues Encountered
- rustfmt rejected a long single-line assert! in the `image_inline_svg_escapes_alt_text` test. Reformatted to multi-line `assert!(...)` block before the Task 2 commit.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Both D-16 and D-17 blast-radius regressions are closed. The 6 gestiscilo settings.rs sites using `compact` and the 1 bar-chart SVG site using `ImageProps::inline_svg` will compile against the updated ferro path.
- No catalog count change — neither `compact` nor `inline_svg` introduces a new built-in component type.
- Plan 162-04 (auth layout card removal, D-05) and subsequent plans can proceed independently.

---
*Phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec*
*Completed: 2026-05-16*

## Self-Check: PASSED

Files exist:
- ferro-json-ui/src/component.rs — FOUND (modified with compact + inline_svg)
- ferro-json-ui/src/render/form.rs — FOUND (modified with compact_class)
- ferro-json-ui/src/render/atoms.rs — FOUND (modified with inline_svg branch)
- ferro-json-ui/src/projection/component_map.rs — FOUND (modified with compact: None)

Commits exist:
- 1e092a3f — FOUND (Task 1: SwitchProps.compact)
- 96aa0520 — FOUND (Task 2: ImageProps.inline_svg)
