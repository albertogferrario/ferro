---
phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
plan: 09
subsystem: ui
tags: [json-ui, docs, directives, mdbook, expressions]

# Dependency graph
requires:
  - phase: 162-json-ui-component-friction-points-and-api-surface-fixes
    provides: spec-validator framework that 163 directives plug into
provides:
  - docs/src/json-ui/spec-construction.md (four-quadrant decision rubric)
  - docs/src/json-ui/expressions.md $each and $if sections
  - SUMMARY.md nav entry linking spec-construction.md before expressions.md
  - Namespace-split lock-in (element-level $each/$if vs prop-level $data/$template)
  - Operator-name lock-in (eq canonical, no equals alias)
affects: [163-04-validation, 163-05-each-expansion, 163-06-if-expansion, 163-10-mcp-catalog]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Decision-rubric documentation pattern: data-shape -> construction strategy mapping"
    - "Cross-reference between rubric (spec-construction.md) and directive reference (expressions.md)"

key-files:
  created:
    - docs/src/json-ui/spec-construction.md
  modified:
    - docs/src/json-ui/expressions.md
    - docs/src/SUMMARY.md

key-decisions:
  - "Rubric ordering: rubric page placed before expressions.md in SUMMARY nav so callers read the data-shape decision before the directive reference."
  - "Voice: neutral architectural framing throughout — no marketing language, no first-person commitments, no strategic positioning."
  - "Operator-name lock: documented `eq` as the canonical wire syntax; explicitly stated that `equals` is not accepted (locks planner decision #1)."
  - "Namespace-split lock: documented why no $template element exists — element-level templating is covered by $each, and the $template keyword is occupied by the prop-level interpolation directive (D-05)."

patterns-established:
  - "Cross-reference pattern: rubric pages link to reference pages via section anchors (./expressions.md#each, ./expressions.md#if)"
  - "Validation-error naming visible in user-facing docs: the same names emitted by SpecError (EachPathNotArray, EachAsReservedName, NestedEach, MismatchedEach, IfPathMissing) appear in the docs verbatim so users can map error messages to the documented rules"

requirements-completed: []

# Metrics
duration: ~15 min
completed: 2026-05-16
---

# Phase 163 Plan 09: Decision-rubric documentation (D-05, D-08) Summary

**Four-quadrant decision rubric for spec construction (Static / `$each` / `$if` / `SpecBuilder`) and `$each` / `$if` directive reference in expressions.md, locking the `eq` operator name and the element-level vs prop-level namespace split.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-16T17:09:00Z (approx)
- **Completed:** 2026-05-16T17:24:29Z
- **Tasks:** 2
- **Files modified:** 3 (1 created, 2 modified)

## Accomplishments

- New `docs/src/json-ui/spec-construction.md` mapping data shape to construction strategy across four quadrants, with one worked example per quadrant.
- Locked the namespace split in user-facing docs: element-level directives (`$each`, `$if`) appear as keys on element objects; prop-level directives (`$data`, `$template`) appear inside `props` values; no `$template` element type exists (D-05).
- Extended `docs/src/json-ui/expressions.md` with full `$each` and `$if` directive reference: fields, expansion order, validation errors (`EachPathNotArray`, `EachAsReservedName`, `NestedEach`, `MismatchedEach`, `IfPathMissing`), correlated-children rule, composition with each other, and limitations.
- Locked operator naming: documented `eq` as the canonical wire syntax explicitly (no `equals` alias accepted).

## Task Commits

Each task was committed atomically:

1. **Task 1: Create spec-construction.md + SUMMARY.md nav update** — `127e7aa4` (docs)
2. **Task 2: Extend expressions.md with $each and $if sections** — `09c86a2a` (docs)

## Files Created/Modified

- `docs/src/json-ui/spec-construction.md` (created) — Four-quadrant decision rubric with worked examples for each quadrant; namespace split documentation; composition rules; migration-from-v1 pointer; concrete decision examples.
- `docs/src/json-ui/expressions.md` (modified) — Appended `## $each` and `## $if` sections after the existing infallible-semantics section. The pre-existing `$data` / `$template` sections are unchanged.
- `docs/src/SUMMARY.md` (modified) — Added `- [Spec construction](./json-ui/spec-construction.md)` entry between `Plugins` and `Expressions` so the rubric is read before the directive reference.

## Decisions Made

- **Rubric ordering in nav.** Placed `spec-construction.md` immediately before `expressions.md` rather than after — the rubric tells users which directive to reach for, so it belongs before the directive reference, not after it. Matches the read-first intent of D-08.
- **Cross-link with anchors.** Linked the rubric to specific sections of expressions.md (`./expressions.md#each`, `./expressions.md#if`) rather than the page top, so users land on the relevant subsection.
- **Operator-name framing.** Made the `eq` lock explicit in bold in the predicate-syntax subsection, rather than burying it in the operator-name list. Future readers should not have to infer that `equals` is rejected.
- **Voice.** Held the neutral architectural voice throughout. The rubric is framed as "pick by data shape, not precedent" — a tool for choosing among options, not as strategic positioning.

## Deviations from Plan

None — plan executed as written. Both tasks completed without scope expansion or unplanned fixes.

## Issues Encountered

- The plan's task 1 specified `SpecBuilder::element_nested` in the worked example for the heterogeneous-runtime quadrant. That API surface is the D-06 design target but is not yet present in `ferro-json-ui/src/spec.rs` (the existing `SpecBuilder` exposes `element(id, ElementBuilder)` but not `element_nested`). The doc was written using `element_nested` as the plan directed; when Plan 06 (SpecBuilder ergonomic layer) ships the API, the example will be backed by real code. No deviation logged because this is documentation-ahead-of-implementation by design — the rubric is the surface-of-record that the implementation phases align to.

## User Setup Required

None — no external service configuration required. Documentation-only change.

## Next Phase Readiness

- Plans 04, 05, 06, 10 in Phase 163 can reference these docs as the user-facing surface contract. The validation error names (`EachPathNotArray`, `EachAsReservedName`, `NestedEach`, `MismatchedEach`, `IfPathMissing`) and the operator-name lock (`eq` canonical) are now public commitments that implementation plans must honor.
- No blockers.

## Self-Check: PASSED

- `docs/src/json-ui/spec-construction.md` — FOUND (176 insertions, all four rubric quadrants present, namespace split documented, composition rules and decision examples present).
- `docs/src/json-ui/expressions.md` — FOUND (110 insertions, `## $each` and `## $if` sections present at end of file).
- `docs/src/SUMMARY.md` — FOUND (spec-construction.md link present at line 59).
- Commit `127e7aa4` — FOUND in git log.
- Commit `09c86a2a` — FOUND in git log.
- `cargo doc --no-deps -p ferro-json-ui` — exits 0 (pre-existing rustdoc warning in `lib.rs:13` is unrelated to plan-09 changes and out of scope per scope-boundary rule).
- Voice scan — no marketing phrases, no first-person personal-name pronouns.

---
*Phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test*
*Plan: 09*
*Completed: 2026-05-16*
