---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
plan: 02
subsystem: mcp
tags: [ferro-mcp, code_templates, v1-deletion, surface-reduction]

# Dependency graph
requires:
  - phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
    provides: "v1 type surface already absent from production (verified in Plan 01)"
  - phase: 164-json-ui-improvements-batch-3-documenti-field-test-findings-m
    provides: "gestiscilo (sole consumer) fully migrated to v2 — no v1 codebases remain"
provides:
  - "ferro-mcp `code_templates` tool no longer returns the `migration_v1_to_v2` category"
  - "`migration_v1_to_v2_templates()` function removed (7 template literals deleted)"
  - "`code_templates_returns_migration_patterns` integration test removed"
affects: [160-03, 160-04, 160-05, 160-06, 160-07, 160-08, 160-09, 160-10, 161]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Surface-area reduction in MCP tool registry: registration + function + test deleted as one coordinated diff (no orphaned comment, no green-test artifact)"

key-files:
  created: []
  modified:
    - ferro-mcp/src/tools/code_templates.rs

key-decisions:
  - "Coordinated three-site deletion in a single commit per RESEARCH.md Pitfall 2 (avoid orphaned `// v1 → v2 migration patterns` comment between commits)"
  - "True test deletion over count update — leaving 'expected at least 0 migration templates' as a green-test artifact is itself noise per CONTEXT.md `<specifics>`"

patterns-established:
  - "MCP code_templates category deletion: drop the `templates.extend(*_templates())` call + its comment in `build_templates()`, drop the producer function, drop the integration test asserting on that category — one commit, three sites"

requirements-completed: [D-04, Pattern-3]

# Metrics
duration: 3min
completed: 2026-05-17
---

# Phase 160 Plan 02: Delete migration_v1_to_v2 template category from ferro-mcp Summary

**ferro-mcp `code_templates` tool no longer advertises a v1→v2 migration category; 230 lines (registration + 7-template function + integration test) removed in a single coordinated diff.**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-05-17T04:57:14Z
- **Completed:** 2026-05-17T04:59:53Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Deleted the `// v1 → v2 migration patterns` comment + `templates.extend(migration_v1_to_v2_templates());` registration in `build_templates()` (lines 78-79 of the pre-edit file).
- Deleted the entire `fn migration_v1_to_v2_templates() -> Vec<CodeTemplate>` body — 7 template literals (`render_file_migration`, `card_children_flat_map`, `datatable_row_actions_interpolation`, `inline_view_edit_pattern`, `checkbox_list_data_driven`, `variant_strum_round_trip`, `verify_action_mcp`).
- Deleted the `code_templates_returns_migration_patterns` integration test (would have failed after the function removal anyway, but kept as a true deletion rather than a count update).
- Verified remaining template categories (handler, controller, model, migration, middleware, validation, json_view, rate_limiting, broadcasting, api) all still present and tested by `test_all_categories_present`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Delete migration_v1_to_v2 registration, function, and test in code_templates.rs** — `e9d4a996` (chore)

## Files Created/Modified
- `ferro-mcp/src/tools/code_templates.rs` — three coordinated deletions; net `-230` lines, no additions

## Decisions Made
- **Single coordinated diff for the three sites** (Pattern 3 from RESEARCH.md): the registration comment, the function body, and the integration test are co-dependent — splitting them produces either an orphaned comment, a dead-code clippy warning, or a failing test mid-sequence. One diff, one commit, no intermediate broken state.
- **No count update on the test** (CONTEXT.md `<specifics>`): per the user-naming constraint, no migration story belongs in agent-readable surface; an assertion of "≥ 0 migration_v1_to_v2 templates" is itself a v1-shaped artifact.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- The `migration_v1_to_v2` literal is now absent from `ferro-mcp/src/tools/code_templates.rs` — the post-deletion grep gate from D-10 will pass for this file.
- Plan 03 (`application_info::scan_json_ui_specs` rewrite per D-05) is the next site in the ferro-mcp tree; no dependency on this plan's output, can run in any order against Plan 02.
- The remaining 9 plans in Phase 160 can proceed; no consumer of `migration_v1_to_v2_templates()` existed outside the deleted test, so no downstream breakage.

## Self-Check: PASSED

- File exists: `ferro-mcp/src/tools/code_templates.rs` — FOUND
- Commit exists: `e9d4a996` — FOUND in `git log`
- Acceptance gate: `grep -c migration_v1_to_v2 ferro-mcp/src/tools/code_templates.rs` returns 0 — PASS
- `cargo fmt --all -- --check` — PASS
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings` — PASS
- `cargo test -p ferro-mcp --all-features --lib code_templates` — PASS (6 tests passed, 0 failed, down from 7 by design)

---
*Phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp*
*Completed: 2026-05-17*
