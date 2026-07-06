---
phase: 75-fix-generate-types-circular-import
plan: 01
subsystem: cli
tags: [typescript, codegen, circular-import, ferro-cli, ferro-mcp]

requires:
  - phase: 22.4-22.9
    provides: generate-types command with shared.ts import/re-export logic
provides:
  - Self-contained inertia-props.ts output (no shared.ts imports/re-exports)
  - Inline JsonValue and ValidationErrors utility types in both CLI and MCP
  - Removed --no-reexports CLI flag
affects: [generate-types, ferro-mcp, inertia-props]

tech-stack:
  added: []
  patterns:
    - "Self-contained generated TypeScript files (no cross-file imports from generated code)"

key-files:
  created: []
  modified:
    - ferro-cli/src/commands/generate_types.rs
    - ferro-cli/src/main.rs
    - ferro-mcp/src/tools/generate_types.rs

key-decisions:
  - "Keep parse_shared_types for resolve_nested_types filtering (avoids regenerating user-defined types)"
  - "MCP type converters now emit JsonValue/ValidationErrors aliases instead of inlining unknown/Record<string, string[]>"

patterns-established:
  - "Generated TypeScript files are fully self-contained with inline utility types"

duration: 12min
completed: 2026-02-27
---

# Phase 75: Fix Generate Types Circular Import Summary

**Self-contained inertia-props.ts output eliminates TS2440/TS2484 circular dependency with shared.ts**

## Performance

- **Duration:** 12 min
- **Started:** 2026-02-27
- **Completed:** 2026-02-27
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Removed `import type { ... } from './shared'` and `export type { ... } from './shared'` from both CLI and MCP generate-types output
- Added inline `JsonValue` and `ValidationErrors` utility type definitions to MCP output (CLI already had them)
- Updated MCP type converters to emit `JsonValue`/`ValidationErrors` aliases instead of `unknown`/`Record<string, string[]>`
- Removed `--no-reexports` CLI flag (no longer needed)
- Removed dead code (`collect_referenced_types` in MCP, `generate_typescript_with_options`/`generate_typescript_with_imports`/`generate_types_to_file_with_options` in CLI)

## Task Commits

Each task was committed atomically:

1. **Task 1: Make CLI generate-types output self-contained** - `3135cb7` (fix)
2. **Task 2: Make MCP generate-types output self-contained** - `2cc5874` (fix)
3. **Task 3: Remove --no-reexports CLI flag** - included in `3135cb7` (required for compilation)

## Files Created/Modified
- `ferro-cli/src/commands/generate_types.rs` - Removed shared.ts import/re-export logic, simplified function signatures, updated tests
- `ferro-cli/src/main.rs` - Removed --no-reexports CLI flag from GenerateTypes command
- `ferro-mcp/src/tools/generate_types.rs` - Removed shared.ts import/re-export logic, added inline utility types, updated type converters

## Decisions Made
- Kept `parse_shared_types` in both CLI and MCP since `resolve_nested_types` uses it to avoid regenerating types the user already defined in `shared.ts`
- MCP type converters updated to emit `JsonValue` and `ValidationErrors` aliases (matching the inline type definitions) instead of raw `unknown` and `Record<string, string[]>`

## Deviations from Plan

### Auto-fixed Issues

**1. [Task 3 merged into Task 1] main.rs change required for compilation**
- **Found during:** Task 1 (CLI generate-types refactoring)
- **Issue:** Removing `no_reexports` from `run()` requires updating `main.rs` simultaneously for compilation
- **Fix:** Included main.rs changes in Task 1 commit
- **Files modified:** ferro-cli/src/main.rs
- **Verification:** cargo test --all-features passes
- **Committed in:** 3135cb7 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Task 3 merged into Task 1 for compilation)
**Impact on plan:** No scope creep. Task 3 became a verification-only step.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Generated `inertia-props.ts` is fully self-contained
- No circular import possible between generated and user-authored type files
- All 1036+ tests pass across the workspace

---
*Phase: 75-fix-generate-types-circular-import*
*Completed: 2026-02-27*
