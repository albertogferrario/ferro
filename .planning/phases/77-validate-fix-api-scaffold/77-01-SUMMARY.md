---
phase: 77-validate-fix-api-scaffold
plan: 01
subsystem: api
tags: [code-generation, templates, cli, mcp, documentation]

# Dependency graph
requires:
  - phase: 76-default-api-scaffold
    provides: make_api CLI, MCP code_templates, API documentation
provides:
  - Correct DB::connection() usage (sync, no .await) in all generated templates
  - Correct ResourceCollection typing (Vec<Resource> not Vec<Value>) in index handlers
affects: [77-validate-fix-api-scaffold]

# Tech tracking
tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified:
    - ferro-cli/src/commands/make_api.rs
    - ferro-mcp/src/tools/code_templates.rs
    - docs/src/features/api.md

key-decisions:
  - "DB::connection() errors mapped with map_err for consistent error handling in templates"

patterns-established: []

# Metrics
duration: 8min
completed: 2026-02-28
---

# Phase 77 Plan 01: Fix Generated Code Template Bugs

**Fixed .await on sync DB::connection() and Vec<serde_json::Value> type mismatch in all three code generation sources**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-28
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Removed `.await` on sync `DB::connection()` in 4 locations across make_api.rs, code_templates.rs, and api.md
- Replaced `Vec<serde_json::Value>` with typed `Vec<{Entity}Resource>` in index handler templates (make_api.rs and code_templates.rs)
- All three code generation sources now produce compilable Rust code

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix DB::connection().await in all templates and docs** - `ab0e1e8` (fix)
2. **Task 2: Fix ResourceCollection type mismatch in index handler templates** - `7b6d2cc` (fix)

## Files Created/Modified
- `ferro-cli/src/commands/make_api.rs` - Fixed index handler template (DB::connection and Resource typing) and ApiKeyProvider template (DB::connection)
- `ferro-mcp/src/tools/code_templates.rs` - Fixed api_controller template (DB::connection and Resource typing)
- `docs/src/features/api.md` - Fixed relationship example (DB::connection)

## Decisions Made
- DB::connection() errors use map_err for consistent error handling pattern rather than unwrap or expect

## Deviations from Plan
None - plan executed exactly as written

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Template bugs fixed, ready for Plan 02 (test coverage) and Plan 03 (integration testing)

---
*Phase: 77-validate-fix-api-scaffold*
*Completed: 2026-02-28*
