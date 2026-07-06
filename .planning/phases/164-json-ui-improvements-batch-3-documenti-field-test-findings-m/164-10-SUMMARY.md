---
phase: 164
plan: 10
subsystem: docs/json-ui
tags: [documentation, json-ui, components, expressions, migration]
dependency_graph:
  requires: [164-01, 164-03, 164-04, 164-05, 164-06, 164-07, 164-08, 164-09]
  provides: [complete-v12-json-ui-docs]
  affects: [docs/src/json-ui/]
tech_stack:
  added: []
  patterns: [mdbook, neutral-doc-voice]
key_files:
  modified:
    - docs/src/json-ui/components.md
    - docs/src/json-ui/expressions.md
    - docs/src/json-ui/migration-v1-to-v2.md
    - docs/src/json-ui/plugins.md
    - docs/src/json-ui/spec-construction.md
decisions:
  - "D-06 paper audit: 2 minor gaps fixed inline (render() data param undocumented; init_script() per-page-once semantics undocumented) — no escalation to Plan 11 BLOCKER"
  - "ImaginaryWidget excluded from coverage check (test fixture only, not in user-facing catalog)"
metrics:
  duration: ~40 minutes
  completed: 2026-05-17
  tasks_completed: 5
  files_modified: 5
---

# Phase 164 Plan 10: JSON-UI Documentation Pass Summary

Documentation pass closing the v12.0 ferro-side JSON-UI surface across five doc files.

## What was built

### components.md

New and updated sections:

| Anchor heading | Change | Requirement |
|----------------|--------|-------------|
| `#### Variant` (under Card) | Bordered/Elevated variant table with class details | D-18, Plan 05 |
| `#### Dynamic source via data_path` (under Image) | data_path override of src | D-15, Plan 03 |
| `### CalendarCell` | New full component section (was missing) | D-08 coverage |
| `#### Dynamic items via data_path` (under DescriptionList) | data_path override of items | D-15, Plan 03 |
| `### CheckboxList` | New full component section (was missing) | D-08 coverage |
| `#### actions — lax acceptance` (under PageHeader) | Lax forms table (null/""/[]/[string]) | D-19/F6, Plan 08 |
| `#### Dynamic columns via data_path` (under KanbanBoard) | data_path override + cross-link to expressions.md | D-13a, Plan 06 |
| `### RawHtml` | New component section with trust boundary call-out | D-17a, Plan 03 |
| Component Overview table | Added CalendarCell, CheckboxList, RawHtml, KanbanColumn categories | D-08 |

### spec-construction.md

| Anchor heading | Change | Requirement |
|----------------|--------|-------------|
| `## Spec.title binding` | TitleBinding: literal string or `{"$data": "/path"}`, fallback behavior, both JSON examples | D-12, Plan 04 |

MAX_NESTING_DEPTH already reads `5` (confirmed — Plan 01 Task 2 handled this).

### migration-v1-to-v2.md

| Change | Requirement |
|--------|-------------|
| `## Cheat sheet` — 10-row table at top of file | D-09 |
| Fixed stale "Depth is limited to 3 levels" — corrected to MAX_NESTING_DEPTH=5 | Rule 1 (bug fix) |

Cheat sheet covers: render_file, flat elements, Plugin→RawHtml/registered, HTTP method uppercase, DetailForm, validation errors, codemod, auth Card elevated, visible/$if, KanbanBoard.data_path.

### expressions.md

| Change | Requirement |
|--------|-------------|
| `### Example: kanban cards from a data array` — comparison table + full JSON spec + handler data | D-13b |
| Updated `spec.title` in "Where Expressions Apply" to note it accepts `$data` binding | D-12 accuracy |

### plugins.md

| Change | Requirement |
|--------|-------------|
| `## When to use RawHtml instead` — added near top | D-06, D-17a |
| Updated built-in count: 40 → 41 | accuracy |
| D-06 paper audit gaps fixed inline (see below) | D-06 |

## D-06 Plugin Paper Audit

Walked through authoring three plugins against the current `plugins.md`:

**(a) Stripe payment status widget** — Gap found: `render(props, data)` second argument not documented; a Stripe plugin needs it to read per-request Stripe account IDs. Fixed: added comment in `ChartPlugin.render()` example explaining both parameters.

**(b) WhatsApp connection-status flow** — Same gap as (a). Also found: `init_script()` per-page-once semantics not documented (critical for multi-instance pages). Fixed: added comment block after `init_script()` implementation.

**(c) Chart renderer** — Covered by existing `ChartPlugin` example. No additional gaps.

**Audit conclusion:** 2 minor gaps fixed inline. No escalation to Plan 11 BLOCKER.

## Per-component coverage grep result

```
0 MISSING DOC lines
```

All 41 built-in components have a matching `### ComponentName` section in `docs/src/json-ui/components.md` or an adjacent doc file. `ImaginaryWidget` (test fixture only, not in user-facing catalog) was excluded from the check.

Component count confirmed: BUILTIN_TYPES has 41 entries (test at `ferro-json-ui/src/render/mod.rs:532` asserts this).

## mdbook build

Clean on every task commit. Final: `INFO HTML book written to .../docs/book`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Stale depth-3 mention in migration guide**
- **Found during:** Task 2
- **Issue:** `migration-v1-to-v2.md` section 2 said "Depth is limited to 3 levels" — stale since Plan 01 raised MAX_NESTING_DEPTH to 5.
- **Fix:** Updated to "Depth is limited to 5 levels (MAX_NESTING_DEPTH)".
- **Files modified:** `docs/src/json-ui/migration-v1-to-v2.md`
- **Commit:** cf6a807f

**2. [Rule 2 - Missing critical functionality] CalendarCell and CheckboxList undocumented**
- **Found during:** Task 5 coverage sweep
- **Issue:** Two built-in components (CalendarCell, CheckboxList) had no `### ComponentName` section in components.md. CheckboxList was mentioned only in the migration guide; CalendarCell was entirely absent from docs.
- **Fix:** Added full component sections for both in Task 1 (pre-emptively, before the coverage sweep confirmed the gap).
- **Files modified:** `docs/src/json-ui/components.md`
- **Commit:** 3d30f137

**3. [Rule 1 - Bug] spec.title described as literal-only in expressions.md**
- **Found during:** Task 3
- **Issue:** The "Where Expressions Apply" section listed `spec.title` as "literal string" — inaccurate since Plan 04 added TitleBinding support.
- **Fix:** Updated the bullet to reflect that `spec.title` accepts either a literal or a `{"$data": "/path"}` binding, with cross-link to spec-construction.md.
- **Files modified:** `docs/src/json-ui/expressions.md`
- **Commit:** 515c8905

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 3d30f137 | docs(164-10): document new component fields, variants, and Spec.title binding (D-08) |
| 2 | cf6a807f | docs(164-10): add v1→v2 cheat sheet table to migration guide (D-09) |
| 3 | 515c8905 | docs(164-10): add $each-for-kanban example and update title-binding note in expressions.md (D-13b) |
| 4 | 63529b33 | docs(164-10): cross-reference RawHtml, reaffirm Plugin-dispatch ban, fix D-06 audit gaps |
| 5 | — | No commit (coverage grep + build verification only; no files modified) |

## Self-Check

Files exist:
- docs/src/json-ui/components.md ✓
- docs/src/json-ui/expressions.md ✓
- docs/src/json-ui/migration-v1-to-v2.md ✓
- docs/src/json-ui/plugins.md ✓
- docs/src/json-ui/spec-construction.md ✓

Commits exist: 3d30f137, cf6a807f, 515c8905, 63529b33 ✓

## Self-Check: PASSED
