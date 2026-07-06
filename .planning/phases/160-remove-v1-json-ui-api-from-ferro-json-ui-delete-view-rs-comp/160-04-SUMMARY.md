---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
plan: 04
subsystem: testing
tags: [ferro-mcp, json-ui-inspect, test-fixtures, v1-removal]

# Dependency graph
requires:
  - phase: 160-01
    provides: deletion of view.rs renderer (v1 framework path removed)
  - phase: 160-02
    provides: MCP code_templates v1 category dropped
  - phase: 160-03
    provides: scaffolder copy/template v1 references neutralized
provides:
  - test_ignores_non_json_files fixture uses neutral identifiers (no v1 framing)
  - regression coverage for "scanner skips non-.json files in views dir" preserved
affects: [160-05, 160-06, 160-07, 160-08, 160-09, 160-10, 161]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Test-fixture rename pattern: when the test asserts a still-valid behavior, rename the v1-framing fixture identifiers instead of deleting the test (RESEARCH Pattern 4)"

key-files:
  created: []
  modified:
    - ferro-mcp/src/tools/json_ui_inspect.rs

key-decisions:
  - "[160-04] Applied PLAN's stricter rewrite over RESEARCH Pattern 4: replaced all three v1 strings (old_view.rs, // old v1 file, pub mod old;) rather than only the first two — RESEARCH's draft still kept `pub mod old;` in mod.rs, but the PLAN's acceptance criteria explicitly require `grep -c 'pub mod old'` to return 0"

patterns-established:
  - "Test-fixture neutralization: replace v1-framing decorative identifiers with subject-neutral equivalents (stale_artifact.rs, // non-JSON artifact, pub mod stale_artifact;) while preserving the behavioral assertion verbatim"

requirements-completed: [D-06, Pattern-4]

# Metrics
duration: 1m 15s
completed: 2026-05-17
---

# Phase 160 Plan 04: Rename v1-Framing Test Fixture Identifiers Summary

**`test_ignores_non_json_files` fixture renamed in-place from v1-coded names (`old_view.rs`, `// old v1 file`, `pub mod old;`) to neutral identifiers (`stale_artifact.rs`, `// non-JSON artifact`, `pub mod stale_artifact;`); scanner-ignores-non-JSON behavior assertion preserved unchanged.**

## Performance

- **Duration:** 1m 15s
- **Started:** 2026-05-17T05:07:03Z
- **Completed:** 2026-05-17T05:08:18Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Eliminated all three v1-framing strings (`old_view.rs`, `// old v1 file`, `pub mod old;`) from the `ferro-mcp` test suite
- Preserved the behavioral assertion (`result.total == 0`) and the test function name (`test_ignores_non_json_files`)
- Verified the full `json_ui_inspect` test module (9 tests) still passes with neutral fixture names

## Task Commits

1. **Task 1: Rename test fixture filenames to neutral names** — `e47a9afb` (test)

## Files Created/Modified
- `ferro-mcp/src/tools/json_ui_inspect.rs` — 2 line replacements inside `test_ignores_non_json_files`; no other test, no production code, no module structure touched

## Decisions Made

- Applied the PLAN's rewrite (`pub mod stale_artifact;`) rather than the literal RESEARCH Pattern 4 draft (which still contained `pub mod old;`). The PLAN's acceptance grep gates explicitly require zero matches for `pub mod old`, and the PLAN takes precedence as the source of truth for execution. RESEARCH's intent was clearly neutralization; the PLAN-level edit fulfills that intent more completely.

## Deviations from Plan

None — plan executed exactly as written. The PLAN's action block, acceptance criteria, and verification all matched the implemented edit.

## Issues Encountered

None.

## User Setup Required

None — pure test-fixture rename, no external service configuration.

## Next Phase Readiness

- D-06 (test fixture audit) is fully discharged for plan 160-04's surface
- Plans 160-05 through 160-10 may now proceed (they operate on disjoint surfaces: scaffolder templates, MCP responses, public docs, README content, changelog entry, final verification)
- No blockers introduced; ferro-mcp still builds clippy-clean with `-D warnings`

## Self-Check: PASSED

- File exists: FOUND: `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp/src/tools/json_ui_inspect.rs` (modified)
- Commit exists: FOUND: `e47a9afb` (`git log --oneline | grep e47a9afb`)
- Grep gate: 0 matches for `old_view.rs|old v1 file|pub mod old` in ferro-mcp/src/tools/json_ui_inspect.rs
- Test execution: `cargo test -p ferro-mcp --all-features --lib json_ui_inspect::tests::test_ignores_non_json_files` exited 0
- Module-wide test execution: all 9 `json_ui_inspect::tests::*` pass
- Format/lint: `cargo fmt --all -- --check` clean; `cargo clippy -p ferro-mcp --all-targets -- -D warnings` clean

---
*Phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp*
*Completed: 2026-05-17*
