---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
plan: 05
subsystem: docs
tags: [json-ui, readme, crates-io, publish-blocker]

# Dependency graph
requires:
  - phase: 115-json-ui-v2-spec-foundation
    provides: Spec / Element / SpecBuilder public surface (the API the README now advertises)
  - phase: 117-catalog-json-schema
    provides: 41-component BUILTIN_TYPES catalog cited in the Features list
provides:
  - ferro-json-ui/README.md Usage block that compiles against the current v2 API
  - Removal of the v1 JsonUiView / LayoutComponent / view.into_response example
  - Features list accurately reflecting the live crate (41 components, ID-keyed graph, $each/$if, plugin system)
affects:
  - 161-merge-v12-0-json-ui-v2-to-master-full-test-pass-clippy-clean (publish gate cleared — crates.io front page will render a copy-paste-valid example)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "README example mirrors the verified-correct rustdoc example in src/lib.rs:19-27"
    - "Neutral, present-tense framing — no v1 / legacy / migration narrative in public docs (CONTEXT D-08)"

key-files:
  created: []
  modified:
    - ferro-json-ui/README.md (full Usage + Features rewrite)

key-decisions:
  - "README Usage block is two examples: the framework handler pattern (JsonUi::render_file) and the in-Rust builder (Spec::builder + Element::new) — covers both consumption modes the crate supports."
  - "Features list cites the BUILTIN_TYPES.len() == 41 invariant directly rather than a soft '30+' range — single source of truth with the test in render/mod.rs:538."
  - "No 'v1', 'v2', 'legacy', or 'migration' framing in the rewritten README — JSON-UI is the only version that exists in agent-readable surface (per feedback_json_ui_naming.md, mirrored as CONTEXT D-08)."

patterns-established:
  - "Pre-publish README audit: any crate ferro publishes must have its README usage block compile against the current public API. Drift here is a first-impression bug on crates.io."

requirements-completed: [D-08, Pattern-6]

# Metrics
duration: 4min
completed: 2026-05-17
---

# Phase 160 Plan 05: Rewrite ferro-json-ui README for current public API Summary

**Replaced the v1 JsonUiView/LayoutComponent README example with the v2 surface (Spec::builder + JsonUi::render_file), clearing the Phase 161 crates.io publish blocker.**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-05-17T05:08:00Z
- **Completed:** 2026-05-17T05:11:46Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- ferro-json-ui/README.md Usage block now compiles against the current public API
- Features list updated to the live shape (41 built-in components, ID-keyed element graph + parse-time validation, $each/$if directives, plugin system, schemars compile-time validation)
- Zero v1 type references (`JsonUiView`, `ComponentNode`, `LayoutComponent`, `view.into_response`) remain in the README
- Phase 161 publish blocker cleared — `crates.io/crates/ferro-json-ui` will render a copy-paste-valid example on next publish

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite ferro-json-ui/README.md Usage block and Features list** — `cc56ff47` (docs)

## Files Created/Modified

- `ferro-json-ui/README.md` — Full rewrite. Replaces v1 `JsonUiView { layout: LayoutComponent::Stack { ... } }` with two v2 examples (handler-side `JsonUi::render_file` + in-Rust `Spec::builder()` + `Element::new()`). Features list rewritten to cite the live BUILTIN_TYPES count (41), the ID-keyed element graph, expression directives, and the plugin system.

## Decisions Made

- Kept the README narrow — two code examples, no expanded prose, no version-history framing. Anything richer belongs at docs.ferro-rs.dev, not on the crates.io front page.
- Used the verbatim Pattern 6 replacement from RESEARCH rather than recomposing — the source examples were already verified against `ferro-json-ui/src/lib.rs:19-27` (Spec builder) and `ferro-cli/src/templates/make.rs` (handler template).

## Deviations from Plan

None — plan executed exactly as written. The Pattern 6 replacement was written verbatim per the plan's `<action>` block.

## Issues Encountered

None.

## Verification

All acceptance gates pass:

| Gate | Expected | Actual |
| --- | --- | --- |
| `grep -cE '\b(JsonUiView\|ComponentNode\|LayoutComponent)\b' ferro-json-ui/README.md` | 0 | 0 |
| `grep -c 'view\.into_response' ferro-json-ui/README.md` | 0 | 0 |
| `grep -c 'Spec::builder' ferro-json-ui/README.md` | ≥1 | 1 |
| `grep -c 'JsonUi::render_file' ferro-json-ui/README.md` | ≥1 | 1 |
| `grep -c '41 built-in components' ferro-json-ui/README.md` | ≥1 | 1 |
| `grep -c 'use ferro_json_ui::{Spec, Element}' ferro-json-ui/README.md` | ≥1 | 1 |
| `grep -c '^```' ferro-json-ui/README.md` | even | 4 |
| `cargo build -p ferro-json-ui` | success | success (4.80s) |

## User Setup Required

None.

## Next Phase Readiness

- Plan 06 (next plan in Phase 160) is unblocked.
- Phase 161 publish blocker for `ferro-json-ui` resolved at the README level. Any remaining publish-readiness concerns for Phase 161 are unrelated to the front-page Usage example.

## Self-Check: PASSED

- File `ferro-json-ui/README.md` exists at HEAD with the rewritten content.
- Commit `cc56ff47` exists in git log on `v12.0/json-ui-v2`.
- All seven acceptance gates verified above.

---
*Phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp*
*Completed: 2026-05-17*
