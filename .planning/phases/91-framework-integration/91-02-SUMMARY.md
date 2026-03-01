---
phase: 91-framework-integration
plan: 02
subsystem: cli
tags: [cli, scaffolding, service-projections, code-generation]

# Dependency graph
requires:
  - phase: 84-service-def
    provides: ServiceDef, DataType, FieldMeaning types
provides:
  - ferro make:projection CLI command for scaffolding ServiceDef modules
  - Generates src/projections/{name}.rs with builder function template
  - Manages mod.rs creation and append
affects: [91-framework-integration, 93-field-test]

# Tech tracking
tech-stack:
  added: []
  patterns: [make_* CLI scaffolding pattern extended to projections]

key-files:
  created: [ferro-cli/src/commands/make_projection.rs]
  modified: [ferro-cli/src/commands/mod.rs, ferro-cli/src/main.rs]

key-decisions:
  - "Template uses ferro::{...} imports (not ferro_projections) since generated code targets user projects with framework dependency"
  - "Directory auto-creation (like make_json_view) rather than requiring pre-existing directory (like make_controller)"
  - "generate_in_dir helper marked #[cfg(test)] to avoid dead_code warning under -D warnings"

patterns-established:
  - "Projection scaffolding: src/projections/{name}.rs with {name}_service() -> ServiceDef function"

# Metrics
duration: 8min
completed: 2026-03-01
---

# Phase 91 Plan 02: make:projection CLI Command

**`ferro make:projection` scaffolds ServiceDef module files with builder function template and mod.rs management**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-01
- **Completed:** 2026-03-01
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Created `make_projection.rs` command following established make_* pattern (directory creation, mod.rs management, case conversion)
- Generated template includes `ServiceDef::new()` with `.display_name()`, `.field()` for id, and commented field examples
- Registered `make:projection` command in CLI with proper clap attributes
- Added 4 unit tests covering template generation, file system operations, and mod.rs deduplication

## Task Commits

Each task was committed atomically:

1. **Task 1: Create make_projection CLI command** - `5036984` (feat)
2. **Task 2: Register make:projection command and add tests** - `c1f426e` (feat)

## Files Created/Modified
- `ferro-cli/src/commands/make_projection.rs` - CLI command with execute(), template generation, case conversion, mod.rs management, 4 unit tests
- `ferro-cli/src/commands/mod.rs` - Added `pub mod make_projection;`
- `ferro-cli/src/main.rs` - Added MakeProjection variant and dispatch match arm

## Decisions Made
- Used `ferro::{...}` in template imports (targets user Ferro projects, not the workspace itself)
- Auto-creates `src/projections/` directory like `make_json_view` rather than requiring it to exist
- `generate_in_dir` helper gated with `#[cfg(test)]` to satisfy clippy `-D warnings`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `ferro make:projection <name>` is operational and tested
- Ready for Phase 91 Plan 03 (MCP integration) and Phase 93 field test

---
*Phase: 91-framework-integration*
*Completed: 2026-03-01*
