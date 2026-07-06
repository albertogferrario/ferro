---
phase: 176-json-ui-v2-runtime-patches-booking-staff-field-test
plan: 01
subsystem: ui
tags: json-ui, components, card, badge, subtitle, catalog, serde, schemars

# Dependency graph
requires:
  - phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test
    provides: Card component template and test infrastructure patterns
provides:
  - CardProps with badge and subtitle optional string slots
  - render_card HTML emission for badge (flex title-row wrapper with Secondary chrome) and subtitle (muted-text paragraph)
  - Catalog description string naming the new slots
  - docs/src/json-ui/components.md Card section with badge + subtitle prop table rows and worked example
affects:
  - 176-02 (shares containers.rs test infrastructure)
  - ferro-mcp json_ui_catalog (auto-reflects updated CardProps schema)
  - gestiscilo-it booking/staff kanban cards

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Additive CardProps extension — new Option<String> fields with skip_serializing_if; no breaking change to existing specs
    - Conditional title-row wrapper: badge-present emits flex wrapper + h3 + span; badge-absent emits bare h3 (existing test invariant preserved)
    - html_escape applied to all new prop slots (same pattern as title and description) — XSS mitigation uniform across all Card text slots

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/catalog.rs
    - docs/src/json-ui/components.md

key-decisions:
  - "F7+F8 shipped as one combined plan — both extend the same CardProps struct, render_card template, catalog entry, and docs section; splitting would yield adjacent commits modifying identical lines with no review benefit"
  - "Badge slot uses flex title-row wrapper (justify-between) rather than stacking below title — keeps title and badge co-planar, matches consumer kanban card visual intent"
  - "Subtitle uses mt-0.5 (4px) spacing to pair visually with the title; description retains mt-1 (8px) to visually separate from the title block"
  - "No BUILTIN_TYPES count change — Card was already registered; only its props change"

patterns-established:
  - "html_escape invariant: every Card text slot (title, description, subtitle, badge) is wrapped with html_escape — uniform XSS discipline, future refactors break all slots together if violated"
  - "Conditional slot emission pattern: if let Some(ref field) = props.field { html.push_str(...html_escape(field)...) } — same shape for both new slots"

requirements-completed: []

# Metrics
duration: pre-executed (commits already present at executor start)
completed: 2026-05-21
---

# Phase 176 Plan 01: Card badge + subtitle slots (F7+F8) Summary

**CardProps extended with `badge: Option<String>` and `subtitle: Option<String>` slots — render_card emits Badge-styled span in flex title-row for badge and muted paragraph for subtitle, both html_escaped; eleven new tests pin slot semantics and serde round-trips**

## Performance

- **Duration:** pre-executed (all four task commits already on master at executor start)
- **Started:** 2026-05-21T00:00:00Z
- **Completed:** 2026-05-21T00:00:00Z
- **Tasks:** 5 (Tasks 1+2 as one atomic commit, Tasks 3, 4, 5 each separate)
- **Files modified:** 4

## Accomplishments

- `CardProps` gains `badge: Option<String>` (field order: title → description → subtitle → badge → max_width → footer → variant) with `#[serde(default, skip_serializing_if = "Option::is_none")]` on both new fields
- `render_card` emits a `flex items-start justify-between gap-2` title-row wrapper containing the h3 + a `bg-secondary/10 text-secondary-foreground shrink-0` Badge span when `props.badge` is `Some`; falls back to the bare `<h3>` when absent (existing tests unaffected)
- `render_card` emits a `<p class="mt-0.5 text-sm text-text-muted">` subtitle paragraph between title and description when `props.subtitle` is `Some`
- Eleven new tests: four serde round-trip/omit-empty tests, two schemars schema-includes tests, five render tests (including combined slot-order assertion and positive/negative cases for each new slot)
- Three pre-existing positional `CardProps` constructors augmented with `subtitle: None, badge: None` to maintain compile-time correctness
- Catalog description updated; docs Card section gains prop table rows and worked example

## Task Commits

1. **Tasks 1+2 (atomic): CardProps fields + serde tests + mechanical fixup** - `b3f35e03` (feat)
2. **Task 3: render_card badge + subtitle slots + five render tests** - `14a9c22e` (feat)
3. **Task 4: Catalog description string** - `e7372289` (chore)
4. **Task 5: Docs — Card prop table + worked example** - `9aba2e8c` (docs)

## Files Created/Modified

- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/component.rs` — `CardProps` struct with two new optional fields; six new serde/schema tests; three existing tests augmented with `subtitle: None, badge: None`
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/render/containers.rs` — `render_card` conditional badge + subtitle emission; five new render tests
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/catalog.rs` — Card entry description string updated
- `/Users/alberto/repositories/albertogferrario/ferro/docs/src/json-ui/components.md` — Card section prop table and worked example

## Decisions Made

- F7+F8 combined into one plan: the file-set overlap is total (same struct, same render function, same catalog entry, same docs section). Splitting would yield two plans modifying the same lines in adjacent commits with no review benefit.
- Badge positioned right-of-title via `flex justify-between` wrapper rather than below description — co-planar layout matches consumer kanban card visual intent ("countdown badge" sits alongside the title, not below it).
- `mt-0.5` for subtitle vs `mt-1` for description: subtitle pairs visually with the title (4px); description separates from the title block (8px).

## Deviations from Plan

None — plan executed exactly as written. All four commits match the plan's task structure: Tasks 1+2 atomic (struct + fixups), Task 3 render, Task 4 catalog, Task 5 docs.

## Issues Encountered

None.

## Known Stubs

None — all new slots are fully wired. `badge` and `subtitle` render verbatim text from `CardProps` via `html_escape`; no placeholder data.

## Threat Flags

No new threat surface beyond what the plan's threat model covers. Both new slots (T-176-01-01, T-176-01-02) are mitigated via `html_escape` in the same pattern as the existing `title` and `description` slots.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Plan 176-02 (F9: Grid.visible reproduction + audit) is independent; no ordering dependency on this plan.
- Consumer (gestiscilo-it) can rebuild against the patched local-path ferro to close F7 + F8 UAT items.
- The ferro-mcp `json_ui_catalog` tool automatically reflects the updated `CardProps` schema with `badge` and `subtitle` on its next invocation (no MCP edit needed).

## Self-Check: PASSED

- FOUND: ferro-json-ui/src/component.rs
- FOUND: ferro-json-ui/src/render/containers.rs
- FOUND: ferro-json-ui/src/catalog.rs
- FOUND: docs/src/json-ui/components.md
- FOUND commit: b3f35e03 (feat: CardProps + tests)
- FOUND commit: 14a9c22e (feat: render_card + render tests)
- FOUND commit: e7372289 (chore: catalog description)
- FOUND commit: 9aba2e8c (docs: components.md)

---
*Phase: 176-json-ui-v2-runtime-patches-booking-staff-field-test*
*Completed: 2026-05-21*
