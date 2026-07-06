---
phase: 108-p0-accuracy-fixes
plan: 02
subsystem: docs
tags: [mdbook, documentation, accuracy, cli, storage, readme]

# Dependency graph
requires: []
provides:
  - CLI reference docs with real example logic replacing all TODO stubs
  - Storage docs accurately presenting S3 as a shipped feature
  - README with accurate JSON-UI shipped status and correct crate badge
affects: [109-cli-mcp-documentation, 112-agent-first-readme]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - CLI generator examples use tracing::info! for action/listener/job/task stubs
    - Migration examples use SeaORM DeriveIden enum pattern with Item generic type

key-files:
  created: []
  modified:
    - docs/src/reference/cli.md
    - docs/src/features/storage.md
    - README.md

key-decisions:
  - "Migration example adds Item Iden enum to show full SeaORM table creation pattern"
  - "Middleware stub at line 212 left unchanged — deferred to Phase 113 per plan"
  - "ferro-rs.dev domain links left unchanged — active website domain, not the renamed crate"
  - "README Roadmap section removed; JSON-UI promoted to a top-level shipped feature section"

patterns-established:
  - "Generic type names (Item, Resource) in generated code examples"

requirements-completed: [ACC-02, ACC-03, ACC-04, ACC-05]

# Metrics
duration: 12min
completed: 2026-03-26
---

# Phase 108 Plan 02: P0 Accuracy Fixes (Docs) Summary

**CLI reference examples now use real logic (tracing + SeaORM patterns), S3 marked shipped, and README presents JSON-UI as a delivered feature with a corrected crate badge**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-26T01:10:00Z
- **Completed:** 2026-03-26T01:22:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Replaced 8 in-scope TODO stubs in cli.md with minimal real logic (tracing::info! calls and a SeaORM table creation example)
- Corrected storage.md S3 driver note from "coming soon" to "Enable the s3 feature"
- Updated README crates.io badge from ferro-rs to ferro (crate was renamed)
- Removed "Work in Progress" Roadmap heading from README; JSON-UI presented as shipped

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace CLI TODO stubs with real example logic** - `a032d231` (docs)
2. **Task 2: Fix storage S3 status + README accuracy audit** - `614944b0` (docs)

## Files Created/Modified

- `docs/src/reference/cli.md` - Replaced 8 TODO stubs with real example logic; middleware stub left for Phase 113
- `docs/src/features/storage.md` - S3 driver note corrected to reflect shipped status
- `README.md` - Crates.io badge fixed (ferro-rs → ferro); JSON-UI presented as shipped feature

## Decisions Made

- Migration example adds an `Item` Iden enum (Table, Id, Name, CreatedAt) to show the full SeaORM table creation pattern — the only stub that required structural additions beyond a single tracing call
- The middleware stub at cli.md line 212 was intentionally left unchanged per plan scope — deferred to Phase 113
- `ferro-rs.dev` and `docs.ferro-rs.dev` domain links in README were left unchanged — these are the active website domain names, distinct from the `ferro-rs` crate name (which was what the badge incorrectly referenced)
- The README "Roadmap" heading was removed entirely (nothing else was in progress to list); JSON-UI moved to its own top-level section as a shipped feature

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ACC-02, ACC-03, ACC-04, ACC-05 requirements satisfied
- Phase 109 (CLI/MCP documentation) can proceed — cli.md is now stub-free
- Phase 112 (agent-first README rewrite) can proceed — README is now factually accurate

---
*Phase: 108-p0-accuracy-fixes*
*Completed: 2026-03-26*
