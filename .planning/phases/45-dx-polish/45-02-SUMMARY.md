---
phase: 45-dx-polish
plan: 02
subsystem: cli, mcp
tags: [mcp, cli, documentation, list_commands, make:auth, make:resource]

requires:
  - phase: 39
    provides: make:auth CLI command
  - phase: 41
    provides: make:resource CLI command
  - phase: 44
    provides: broadcasting commands
provides:
  - Complete MCP command list (40 commands matching CLI)
  - CLI reference documentation for make:auth and make:resource
affects: [46-mcp-cli-updates]

tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified:
    - ferro-mcp/src/tools/list_commands.rs
    - docs/src/reference/cli.md

key-decisions:
  - "Added `clean` command to MCP list (8 instead of 7) to match actual CLI"

patterns-established: []

duration: 5min
completed: 2026-02-10
---

# Phase 45 Plan 02: MCP + CLI Docs Completeness Summary

**MCP list_commands updated to 40 commands (was 32), CLI reference docs expanded with make:auth and make:resource sections**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-10T12:09:20Z
- **Completed:** 2026-02-10T12:13:51Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- MCP list_commands now returns all 40 CLI commands (added 8 missing)
- CLI reference docs include make:auth section with options, generated files, and ALTER TABLE approach
- CLI reference docs include make:resource section with options, generated code, and field attributes
- Command Summary table updated with both new entries

## Task Commits

Each task was committed atomically:

1. **Task 1: Add 8 missing commands to MCP list_commands** - `d3eb8a9` (feat)
2. **Task 2: Add make:auth and make:resource to CLI reference docs** - `d8eb4de` (docs)

## Files Created/Modified

- `ferro-mcp/src/tools/list_commands.rs` -- Added make:auth, make:json-view, make:resource, db:seed, do:init, claude:install, clean, validate:contracts
- `docs/src/reference/cli.md` -- Added detailed sections for make:auth and make:resource, updated Command Summary table

## Decisions Made

- Added `clean` command to MCP list beyond the 7 specified in plan, bringing total to 40 matching the actual CLI command count

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added `clean` command to MCP list_commands**
- **Found during:** Task 1 (MCP command list update)
- **Issue:** Plan specified 7 missing commands (total 39), but actual CLI has 40 commands -- `clean` was also missing from MCP
- **Fix:** Added `clean` entry to the static command list
- **Files modified:** ferro-mcp/src/tools/list_commands.rs
- **Verification:** Command count is 40, matching ferro-cli/src/main.rs
- **Committed in:** d3eb8a9 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Ensures complete parity between CLI and MCP. No scope creep.

## Issues Encountered

None.

## User Setup Required

None -- no external service configuration required.

## Next Phase Readiness

- MCP list_commands complete with all 40 CLI commands
- CLI reference docs complete with all make: commands documented
- Ready for remaining Phase 45 plans or Phase 46

---
*Phase: 45-dx-polish*
*Completed: 2026-02-10*
