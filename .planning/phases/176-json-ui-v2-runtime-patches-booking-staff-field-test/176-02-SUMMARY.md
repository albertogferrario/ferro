---
phase: 176-json-ui-v2-runtime-patches-booking-staff-field-test
plan: 02
subsystem: ui
tags: json-ui, grid, visibility, regression-test, documentation

# Dependency graph
requires:
  - phase: 176-json-ui-v2-runtime-patches-booking-staff-field-test
    plan: 01
    provides: Card badge+subtitle (F7+F8); shares containers.rs test infrastructure
provides:
  - Three regression tests pinning Grid element-level visibility (Outcome A — no-repro)
  - docs/src/json-ui/components.md Grid section with Visibility subsection
affects:
  - gestiscilo-it booking/staff chip-strip Grid (consumer should re-test with patched runtime)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Element-level visibility test pattern — use render_spec_to_html (not bare render_grid) so the walker's visibility check at mod.rs:155-160 is exercised
    - Non-root hidden element asserted by substring ABSENCE (no diagnostic comment for non-root — empty string returned into parent body)
    - Consumer spec reproduction pattern — mirror the exact JSON spec shape in a Rust test using build_spec + Element builder

key-files:
  created: []
  modified:
    - ferro-json-ui/src/render/containers.rs
    - docs/src/json-ui/components.md

key-decisions:
  - "Outcome A: all three reproduction tests passed green on first run — F9 closes as could-not-reproduce against current ferro master"
  - "Task 3 SKIPPED: no production code change needed; visibility evaluator architecture verified correct by both RESEARCH trace and live test execution"
  - "Docs clarification added regardless of outcome — element-level visibility semantics were not documented in the Grid section; the subsection closes CONTEXT F9 'audit which v2 components support visible'"

requirements-completed: []

# Metrics
duration: 6 minutes
completed: 2026-05-21
---

# Phase 176 Plan 02: Grid visibility reproduction tests + docs (F9) Summary

**F9 closes as Outcome A (no-repro): three Grid visibility regression tests pass green on first run against current ferro master; visibility evaluator architecture is correct; consumer chip-strip Grid should render correctly when rebuilt against patched ferro runtime**

## Outcome

**Outcome A — could not reproduce against current ferro master.**

All three Task 1 reproduction tests (`grid_renders_when_visible_true`, `grid_hidden_when_visible_false`, `grid_visible_consumer_reproduction`) passed green on their first run. The consumer's reported F9 symptom (Grid absent from DOM despite `data.has_staff = true`) cannot be reproduced from the code path traced in RESEARCH.

Task 3 (conditional production code fix) was SKIPPED per plan instructions.

The plan ships as: regression tests (Task 1) + docs clarification (Task 2).

## Performance

- **Duration:** 6 minutes
- **Started:** 2026-05-20T22:26:05Z
- **Completed:** 2026-05-21T00:32:00Z
- **Tasks:** 2 executed (Task 3 skipped — Outcome A)
- **Files modified:** 2

## Accomplishments

- Three regression tests added to `ferro-json-ui/src/render/containers.rs` immediately after `grid_scrollable_emits_flow_col`:
  - `grid_renders_when_visible_true` — minimal Grid with `Eq(true)` condition renders when `data.flag=true`
  - `grid_hidden_when_visible_false` — same spec omits the Grid entirely when `data.flag=false`
  - `grid_visible_consumer_reproduction` — mirrors the consumer's chip-strip spec shape: root Grid containing inner `staff_chips_row` Grid gated on `/has_staff`; both the visible-true and visible-false branches asserted by grid-div count
- `docs/src/json-ui/components.md` Grid section gains a `#### Visibility` subsection documenting:
  - `visible` is element-level, not a `GridProps` prop
  - Entire subtree absent from rendered DOM when condition is false (no `hidden` attribute, no empty wrapper)
  - Consumer chip-strip example (`/has_staff` gate) as a worked code snippet
  - Explicit universality statement: identical semantics apply to Card, Form, Button, Badge, and all plugin components

## Task Commits

1. **Task 1: Grid visibility reproduction tests** — `28b2eb58` (test)
2. **Task 2: Grid Visibility docs subsection + fmt** — `727755b3` (docs)

## Files Created/Modified

- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/render/containers.rs` — three new tests (`grid_renders_when_visible_true`, `grid_hidden_when_visible_false`, `grid_visible_consumer_reproduction`); cargo fmt applied
- `/Users/alberto/repositories/albertogferrario/ferro/docs/src/json-ui/components.md` — Grid section gains `#### Visibility` subsection

## Decisions Made

- Outcome A confirmed by execution: all three tests green on first run. The consumer's F9 report is most likely attributable to testing against a stale local ferro checkout (RESEARCH §"Possible real causes" item 1), or possibly consumer-side authoring confusion between `has_staff_widget` and `has_staff` paths (item 5). The consumer should rebuild against the current ferro local-path dependency and re-run their UAT.
- No production code change: the visibility evaluator at `render/mod.rs:155-160` correctly evaluates every element type identically before component dispatch; Grid is no special case.
- The `#### Visibility` docs subsection is unconditional — it ships regardless of outcome because the "audit which v2 components support visible" criterion (CONTEXT F9) requires documenting the union as "all v2 components".

## Deviations from Plan

**[Rule 2 - Missing critical step] cargo fmt applied in Task 2 commit**

The three test lines in `grid_visible_consumer_reproduction` produced two `cargo fmt --check` diffs (long method-chain and long let binding). Applied `cargo fmt -p ferro-json-ui` before the Task 2 commit. The formatted output matches rustfmt's line-length conventions; no behavioral change.

Otherwise — plan executed exactly as written. Outcome A materialized as predicted by RESEARCH §"Critical Pre-Planning Finding".

## F9 No-Repro Finding

**Verbatim test output (Task 1 run):**

```
running 3 tests
test render::containers::tests::grid_hidden_when_visible_false ... ok
test render::containers::tests::grid_renders_when_visible_true ... ok
test render::containers::tests::grid_visible_consumer_reproduction ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 543 filtered out; finished in 0.00s
```

**Why the consumer's symptom is real but not reproducible here:**

The RESEARCH trace (mod.rs:155-160 → visibility.rs:evaluate → data.rs:resolve_path) is architecturally correct. `render_element` checks `el.visible` BEFORE component dispatch — Grid is no different from Card, Badge, Button, or any other type. The consumer's `data.has_staff = true` case would have `resolve_path(data, "/has_staff") = Some(&Value::Bool(true))`, `operator Eq`, `value Some(Bool(true))` → evaluates `true` → Grid renders. The three reproduction tests confirm this directly.

Most likely consumer-side causes (per RESEARCH §"Possible real causes"):
1. The consumer's local-path ferro checkout was at a stale commit when the UAT was run (item 1 — highest probability)
2. The consumer's chrome-mcp snapshot captured the DOM before the full React hydration / Inertia page transition completed (item 2)

**Consumer action required:** rebuild against the current ferro local-path dependency and re-run the per-staff chip strip UAT (Bug R4 in gestiscilo-it `.planning/phases/152-booking-staff-binding/152-UI-FINDINGS.md`).

## Issues Encountered

None.

## Known Stubs

None.

## Threat Flags

No new threat surface introduced. Test fixtures use constant prop values; no user-supplied data flows through the test render path.

## User Setup Required

None.

## Next Phase Readiness

- Phase 176 is now complete (both plans executed: 176-01 F7+F8 card slots, 176-02 F9 Grid visibility regression + docs).
- Consumer (gestiscilo-it) should rebuild against the patched local-path ferro and re-run the booking↔staff binding β UAT (chrome-mcp snapshot).
- Next: Push master + publish v12.0 release; then begin v12.1 Form Validation DX (Phases 137-139) or v12.1 AI milestone (Phase 165: LlmClient Trait).

## Self-Check: PASSED

- FOUND: ferro-json-ui/src/render/containers.rs
- FOUND: docs/src/json-ui/components.md
- FOUND commit: 28b2eb58 (test: Grid visibility reproduction tests)
- FOUND commit: 727755b3 (docs: Grid Visibility subsection + fmt)

---
*Phase: 176-json-ui-v2-runtime-patches-booking-staff-field-test*
*Completed: 2026-05-21*
