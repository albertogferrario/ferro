---
phase: 35-cli-seed-command
plan: 02
subsystem: cli
tags: [clap, cli, database, namespace, mcp]

# Dependency graph
requires:
  - phase: 35-01
    provides: db:seed CLI command pattern
provides:
  - Unified db: namespace for all database CLI commands
  - Consistent command naming across CLI, MCP, docs, and app binary
affects: [cli-reference, mcp-tools, app-binary-template]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "All database commands use db: namespace prefix"

key-files:
  created: []
  modified:
    - ferro-cli/src/commands/db_migrate.rs
    - ferro-cli/src/commands/db_rollback.rs
    - ferro-cli/src/commands/db_status.rs
    - ferro-cli/src/commands/db_fresh.rs
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
    - ferro-cli/src/templates/files/backend/main.rs.tpl
    - ferro-mcp/src/tools/list_commands.rs
    - ferro-mcp/src/tools/diagnose_error.rs
    - framework/src/app.rs
    - app/src/main.rs
    - docs/src/reference/cli.md
    - docs/src/features/database.md
    - docs/src/getting-started/quickstart.md
    - docs/src/upgrading/migration-guide.md
    - README.md
    - AGENTS.md

key-decisions:
  - "Rename enum variants from MigrateX to DbX for consistency with command names"

# Metrics
duration: 8min
completed: 2026-02-09
---

# Phase 35 Plan 02: Database CLI Command Normalization Summary

**Unified all database commands under db: namespace -- migrate -> db:migrate, migrate:rollback -> db:rollback, migrate:status -> db:status, migrate:fresh -> db:fresh**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-09
- **Completed:** 2026-02-09
- **Tasks:** 2
- **Files modified:** 17

## Accomplishments
- Renamed all migrate CLI command files to db_ convention (db_migrate.rs, db_rollback.rs, db_status.rs, db_fresh.rs)
- Updated CLI, framework Application builder, app binary, and scaffold template to use db: namespace
- Updated MCP tools (list_commands, diagnose_error) with normalized command names
- Updated all documentation (CLI reference, database guide, quickstart, migration guide, README, AGENTS.md)

## Task Commits

Each task was committed atomically:

1. **Task 1: Rename CLI command files and update command registrations** - `5518b26` (feat)
2. **Task 2: Update MCP, docs, and reference files** - `94560e5` (docs)

## Files Created/Modified
- `ferro-cli/src/commands/db_migrate.rs` - Renamed from migrate.rs, delegates to db:migrate
- `ferro-cli/src/commands/db_rollback.rs` - Renamed from migrate_rollback.rs, delegates to db:rollback
- `ferro-cli/src/commands/db_status.rs` - Renamed from migrate_status.rs, delegates to db:status
- `ferro-cli/src/commands/db_fresh.rs` - Renamed from migrate_fresh.rs, delegates to db:fresh
- `ferro-cli/src/commands/mod.rs` - Updated module declarations
- `ferro-cli/src/main.rs` - Updated enum variants and match arms
- `ferro-cli/src/templates/files/backend/main.rs.tpl` - Updated scaffold template
- `framework/src/app.rs` - Updated Application command definitions
- `app/src/main.rs` - Updated sample app binary
- `ferro-mcp/src/tools/list_commands.rs` - Updated command names
- `ferro-mcp/src/tools/diagnose_error.rs` - Updated error suggestion
- `docs/src/reference/cli.md` - Updated command references and summary table
- `docs/src/features/database.md` - Updated migration command examples
- `docs/src/getting-started/quickstart.md` - Updated migration command
- `docs/src/upgrading/migration-guide.md` - Updated CLI examples
- `README.md` - Updated CLI reference
- `AGENTS.md` - Updated database command examples

## Decisions Made
- Renamed enum variants from Migrate/MigrateX to DbMigrate/DbX for consistency with the db: command names
- make:migration scaffolding command left unchanged (it's under make: namespace, not a database operation)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated framework/src/app.rs Application command definitions**
- **Found during:** Task 2 (MCP and docs update)
- **Issue:** framework/src/app.rs contained the old migrate:* command names in the Application builder's clap definitions, which would cause the app binary to still use old names when built with Application::new()
- **Fix:** Updated command names, enum variants, match arms, and doc comments in framework/src/app.rs
- **Files modified:** framework/src/app.rs
- **Verification:** cargo build, cargo clippy, cargo test all pass
- **Committed in:** 94560e5 (Task 2 commit)

**2. [Rule 3 - Blocking] Updated app/src/main.rs sample application**
- **Found during:** Task 1 (CLI command rename)
- **Issue:** The sample application binary used old migrate:* command names and would fail to compile consistently with the CLI
- **Fix:** Updated command names, enum variants, match arms, doc comments, and error message references in app/src/main.rs
- **Files modified:** app/src/main.rs
- **Verification:** cargo build -p app passes, no stale references
- **Committed in:** 5518b26 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes necessary for complete namespace consistency. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 35 complete (both plans executed)
- All database commands consistently use db: namespace
- Ready for Phase 36 (Gitignore Generated Types)

---
*Phase: 35-cli-seed-command*
*Completed: 2026-02-09*
