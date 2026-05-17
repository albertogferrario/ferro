---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
plan: 07
subsystem: docs
tags: [json-ui, projections, docs, mdbook, visual-context]

# Dependency graph
requires:
  - phase: 160
    provides: "Source-of-truth rustdoc at ferro-json-ui/src/projection/mod.rs:79-97 (VisualContext, Spec.schema/elements, ferro-json-ui/v2)"
provides:
  - "docs/src/features/projections.md Quick Start example now compiles against the actual v2 public API"
  - "Eliminated triple-stale references (v1 schema, RenderContext, json[\"components\"]) in the Minimal example"
affects: [160-08, 160-09, 160-10, 161]

# Tech tracking
tech-stack:
  added: []
  patterns: ["docs-rustdoc parity: hand-written feature docs mirror the verified source rustdoc shape exactly"]

key-files:
  created: []
  modified:
    - docs/src/features/projections.md

key-decisions:
  - "[160-07] Scope discipline: rewrote only the Quick Start Minimal example block per CONTEXT D-07 / RESEARCH Pattern 5 §(d); the later 'Rendering' section's RenderContext/RenderMode prose is out of scope for this plan"
  - "[160-07] Shape-parity over paraphrase: ported the rustdoc example structure verbatim (result/spec split, spec.schema / spec.elements comments) so future divergence is mechanically visible to diff tools"

patterns-established:
  - "Triple-stale rewrite: when a single code block carries multiple drift markers (wrong type, wrong schema, wrong field name), replace the whole block in one edit rather than chaining three substitutions"

requirements-completed: [D-07, Pattern-5]

# Metrics
duration: 53s
completed: 2026-05-17
---

# Phase 160 Plan 07: Sync projections.md Minimal Example to Source Rustdoc Summary

**Quick Start example in docs/src/features/projections.md rewritten to mirror ferro-json-ui/src/projection/mod.rs:79-97 — VisualContext replaces RenderContext, spec.schema/spec.elements replace json["$schema"]/json["components"], and the v1 schema string is gone.**

## Performance

- **Duration:** 53s
- **Started:** 2026-05-17T05:18:36Z
- **Completed:** 2026-05-17T05:19:29Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Imported the correct public type `VisualContext` (verified at `ferro-json-ui/src/lib.rs:94`) in the docs example.
- Replaced the stale `json["$schema"] == "ferro-json-ui/v1"` and `json["components"]` comments with the actual `Spec` shape: `spec.schema == "ferro-json-ui/v2"` and `spec.elements` (flat ID-keyed map) + `spec.root` (root element name).
- The Quick Start now matches the source-of-truth rustdoc word-for-word in shape, so future API renames will surface as a clippy/doctest failure on the source side first.

## Task Commits

Each task was committed atomically:

1. **Task 1: Sync projections.md minimal example to match projection/mod.rs rustdoc** — `6df1516b` (docs)

## Files Created/Modified

- `docs/src/features/projections.md` — Rewrote the import line and the example tail in the Quick Start block (lines 23-43) to mirror the rustdoc at `ferro-json-ui/src/projection/mod.rs:79-97`.

## Decisions Made

- **Scope discipline:** the file's later "Rendering" section (around lines 178-219) still uses `RenderContext` / `RenderMode` and `let json = renderer.render(...)`. Per CONTEXT D-07 and the plan's `<action>` block, that section is explicitly out of scope for this plan — only the Quick Start Minimal example was the rewrite target. The later section is presumably handled by a sibling plan (or carried into a follow-up phase) and was not silently expanded into.
- **Shape parity over paraphrase:** the replacement reproduces the rustdoc's `let result = ...; let spec = result.expect(...)` two-line split rather than collapsing to `let spec = renderer.render(...).expect(...)`. Matching the rustdoc shape exactly makes any future drift mechanically visible in diff review.

## Deviations from Plan

None — plan executed exactly as written. All seven acceptance criteria (3 negative greps + 4 positive greps) pass on first execution.

## Acceptance Verification

```
grep -c 'ferro-json-ui/v1'   docs/src/features/projections.md → 0
grep -c 'RenderContext::default'  docs/src/features/projections.md → 0
grep -c 'json\["components"\]'    docs/src/features/projections.md → 0
grep -c 'VisualContext'       docs/src/features/projections.md → 2
grep -c 'ferro-json-ui/v2'    docs/src/features/projections.md → 1
grep -c 'spec.elements'       docs/src/features/projections.md → 1
grep -c 'spec.schema'         docs/src/features/projections.md → 1
grep -c 'VisualContext::default' docs/src/features/projections.md → 1
```

All gates green.

## Issues Encountered

None.

## Known Stubs

None — the rewrite is a docs-only correctness fix; no placeholder UI, no mock data, no hardcoded empties introduced.

## Next Phase Readiness

- Quick Start example is now safe to reference from agent docs and from intro/onboarding pages.
- Follow-up consideration for sibling/later plans in Phase 160: the "Rendering" section (around lines 178-219) and the "Complete Example" (around lines 222-269) and the Reference table (lines 271-290) still describe `RenderContext { intent_index, current_state, mode, templates }` as the public type. If the v1 API surface has been fully removed from `ferro-json-ui` (per the broader Phase 160 objective), those sections will need the same treatment in a subsequent plan. This plan deliberately did not expand scope.

## Self-Check: PASSED

- `docs/src/features/projections.md` exists and contains the rewritten Quick Start example (verified via Read).
- Commit `6df1516b` exists in `git log` (verified via `git rev-parse --short HEAD`).
- All 7 acceptance grep gates pass (verified above).

---
*Phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp*
*Plan: 07*
*Completed: 2026-05-17*
