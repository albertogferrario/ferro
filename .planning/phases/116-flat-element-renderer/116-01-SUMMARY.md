---
phase: 116-flat-element-renderer
plan: 01
subsystem: ui
tags: [ferro-json-ui, spec-v2, visibility, slot-fields, component-catalog, schemars]

# Dependency graph
requires:
  - phase: 115-spec-v2-data-structures
    provides: Spec/Element v2 shape, typed *Props structs (v1 slot fields stripped), VisibilityCondition/VisibilityOperator (no evaluate method)
provides:
  - CardProps.footer, ModalProps.footer, Tab.children, KanbanColumnProps.children, PageHeaderProps.actions (Vec<String> of element IDs)
  - Visibility::evaluate(&Value) -> bool with 11-operator coverage (Exists, NotExists, Eq, NotEq, Gt, Lt, Gte, Lte, Contains, NotEmpty, Empty)
  - evaluate_condition + numeric_cmp private helpers (infallible predicate contract)
  - COMPONENT_CATALOG entries updated to match v2 Props shape (Card, Modal, Tabs, Form, PageHeader new, KanbanBoard new)
affects: [116-02-flat-walker, 116-04-multi-slot-containers, 116-06-integration-tests, 117-catalog-and-schema]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Slot-as-typed-Vec<String>-field on *Props (over generic Element.children) for multi-slot containers"
    - "Infallible visibility predicate: malformed conditions and missing paths all resolve to false, no panics, no Result"
    - "Missing-path semantics per A1: Eq=false (no value to compare), NotEq=true (no value, so not-equal-to-anything)"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/visibility.rs
    - ferro-json-ui/src/lib.rs

key-decisions:
  - "Visibility::evaluate is infallible — malformed inputs degrade to false rather than return Result, matching CONTEXT D-13 and the SDUI presentation-layer-only V4 constraint"
  - "NotEmpty/Empty treat numbers and booleans as non-empty (scalar values are 'present data'), matching Phase 116 RESEARCH edge-case decision"
  - "Numeric comparators (Gt/Lt/Gte/Lte) return false for non-numeric or missing values — strict type requirement, no string coercion"
  - "Tab gets a children: Vec<String> slot field — Tab is nested inside TabsProps.tabs (not a top-level Element) so its children cannot come from Element.children"
  - "PageHeader and KanbanBoard get new COMPONENT_CATALOG entries inline (HIGH-1 recommendation (a)) rather than deferring to Phase 117 — small cost, keeps ferro-mcp catalog accurate for Plans 02–06"

patterns-established:
  - "Multi-slot container: Vec<String> of IDs in Props field (never an enum, never a generic map)"
  - "Visibility rustdoc documents every edge case (missing path per operator) — not just happy path"
  - "COMPONENT_CATALOG entries use the phrase 'Vec<String> of element IDs' to signal slot semantics (vs. ordinary Vec<String> data fields)"

requirements-completed: [RENDER-01, RENDER-03]

# Metrics
duration: ~15min
completed: 2026-04-18
---

# Phase 116 Plan 01: Wave 1 Renderer Prerequisites Summary

**Re-added 5 multi-slot Vec<String> fields to *Props structs, implemented Visibility::evaluate covering 11 operators with an infallible predicate contract, and updated COMPONENT_CATALOG to match the v2 Props shape.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-18T02:00:55Z
- **Completed:** 2026-04-18T02:16:11Z
- **Tasks:** 3 / 3
- **Files modified:** 3

## Accomplishments

- Five slot fields re-added: `CardProps.footer`, `ModalProps.footer`, `Tab.children`, `KanbanColumnProps.children`, `PageHeaderProps.actions` — all serde-defaulted (`#[serde(default, skip_serializing_if = "Vec::is_empty")]`), all backward-compatible with Phase 115 round-trip and reject suites.
- `Visibility::evaluate(&Value) -> bool` implemented with full 11-operator coverage (Exists, NotExists, Eq, NotEq, Gt, Lt, Gte, Lte, Contains, NotEmpty, Empty) plus And/Or/Not composition; infallible, no panics, no `Result`.
- `evaluate_condition` and `numeric_cmp` private helpers factor the operator dispatch cleanly.
- `COMPONENT_CATALOG` updated for 5 entries: Card, Modal, Tabs rewritten; PageHeader + KanbanBoard added; Form.fields obsolete entry removed (Form fields come from `Element.children` in v2).
- Test count: component.rs gained 3 tests (round-trip, Tab children, empty-footer skip), visibility.rs gained 13 tests (one per operator plus compound And/Or/Not and edge cases).
- Wave 1 gate passes: `cargo test -p ferro-json-ui --lib` → 205/205 pass; `cargo test -p ferro-json-ui --tests` → 19/19 pass (8 round-trip + 11 reject); `cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings` → clean.

## Task Commits

Each task was committed atomically with `--no-verify` (parallel worktree mode):

1. **Task 1: Re-add 5 slot fields to component.rs** — `44136973` (feat)
2. **Task 2: Implement Visibility::evaluate with 11-operator coverage** — `3006be33` (feat)
3. **Task 3: Update COMPONENT_CATALOG for 5 slot-bearing components** — `d6e3ec33` (docs)

## Files Created/Modified

- `ferro-json-ui/src/component.rs` — 5 slot fields added to 5 structs; 3 new tests appended to `schema_smoke_tests` module
- `ferro-json-ui/src/visibility.rs` — `Visibility::evaluate` + `evaluate_condition` + `numeric_cmp` implementations; 13 new tests appended to `tests` module
- `ferro-json-ui/src/lib.rs` — `COMPONENT_CATALOG` entries for Card, Modal, Tabs, Form updated; PageHeader and KanbanBoard entries added

## Visibility::evaluate Semantics (per operator)

| Operator   | Returns true when                                                                                          | Missing path                    |
| ---------- | ---------------------------------------------------------------------------------------------------------- | ------------------------------- |
| Exists     | `resolve_path(data, &path)` is `Some(v)` and `!v.is_null()`                                                | false                           |
| NotExists  | Inverse of Exists                                                                                          | true                            |
| Eq         | resolved value equals target (per `serde_json::Value::eq`)                                                 | false (no value to compare)     |
| NotEq      | resolved value does not equal target                                                                       | true (nothing is equal to x)    |
| Gt/Gte     | both values numeric and resolved > (or ≥) target                                                           | false (non-numeric treated as false) |
| Lt/Lte     | both values numeric and resolved < (or ≤) target                                                           | false                           |
| Contains   | String: substring match. Array: membership match (`arr.iter().any(&v| v == target)`)                      | false                           |
| NotEmpty   | Non-empty String/Array/Object, OR any Number/Bool (scalars are always "not empty")                         | false                           |
| Empty      | Empty String/Array/Object, OR null/missing. Numbers/booleans are never empty.                              | true                            |

Compound (`And`, `Or`, `Not`): `.all()`, `.any()`, and `!inner.evaluate()` respectively. Empty `And` → `true` (vacuous truth); empty `Or` → `false`.

## COMPONENT_CATALOG Entries Updated

| Component   | Change                                                                                                                  |
| ----------- | ----------------------------------------------------------------------------------------------------------------------- |
| Card        | Replaced obsolete `children (Vec<String>)` entry with `max_width` + `footer (Vec<String> of element IDs)`; note that body comes from Element.children |
| Modal       | Replaced obsolete `children` entry with `id`, `footer (Vec<String> of element IDs)`; note that body comes from Element.children                        |
| Tabs        | Clarified `Tab.children` contents are element IDs (`Vec<String> of element IDs`)                                        |
| Form        | Dropped obsolete `fields (Vec<String>)` — Form fields now come from Element.children in v2                              |
| PageHeader  | **New entry**: title, breadcrumb, actions (Vec<String> of element IDs)                                                  |
| KanbanBoard | **New entry**: columns with embedded `KanbanColumnProps {id, title, count, children: Vec<String> of element IDs}`       |

## Decisions Made

None beyond the plan — plan executed as written. The three edge-case decisions listed in `key-decisions` frontmatter were pre-specified in RESEARCH and CONTEXT; this plan implemented them.

## Deviations from Plan

None — plan executed exactly as written.

Test module in `component.rs` is named `schema_smoke_tests` (not `tests`) — the plan's language "in the `#[cfg(test)] mod tests` section" was interpreted as "in the existing test module of `component.rs`." The 3 new tests were appended to `schema_smoke_tests` since that is the only `#[cfg(test)]` module in that file. This is not a deviation from intent — tests run under `cargo test -p ferro-json-ui --lib component::` as specified.

## Issues Encountered

None.

## User Setup Required

None — this plan modifies internal data structures and predicate logic only; no external services, no new env vars.

## Next Phase Readiness

Wave 1 contract satisfied. Downstream Phase 116 plans can now rely on:

- **Plan 02 (flat walker):** `Visibility::evaluate(&Value) -> bool` is present and covers all 11 operators per D-13.
- **Plan 04 (multi-slot containers):** CardProps.footer, ModalProps.footer, Tab.children, KanbanColumnProps.children, PageHeaderProps.actions are present with Vec<String> shape per D-06.
- **Plan 06 (integration tests):** COMPONENT_CATALOG matches the Props shape — ferro-mcp catalog consistency is preserved between Phase 116 and Phase 117.

No blockers. Phase 115's round_trip and reject suites remain green — the slot field additions are serde-default backward-compatible per RESEARCH lines 428–437.

## Self-Check: PASSED

Verified:
- `ferro-json-ui/src/component.rs` (modified): FOUND
- `ferro-json-ui/src/visibility.rs` (modified): FOUND
- `ferro-json-ui/src/lib.rs` (modified): FOUND
- Task 1 commit `44136973`: FOUND in git log
- Task 2 commit `3006be33`: FOUND in git log
- Task 3 commit `d6e3ec33`: FOUND in git log
- `cargo test -p ferro-json-ui --lib`: 205 passed, 0 failed
- `cargo test -p ferro-json-ui --tests`: 19 passed, 0 failed
- `cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings`: clean

---
*Phase: 116-flat-element-renderer*
*Plan: 01*
*Completed: 2026-04-18*
