---
phase: 77-validate-fix-api-scaffold
plan: 03
subsystem: cli
tags: [code-generation, templates, cli, integration-testing, singularize]

# Dependency graph
requires:
  - phase: 77-validate-fix-api-scaffold
    provides: Plan 01 template bug fixes (DB::connection, Resource typing)
provides:
  - Compilable make:api output for real models
  - Correct singular model naming from plural entity files
  - Module import path separation (module_name vs model name)
  - resources/mod.rs and requests/mod.rs generation
  - 32 template validation regression tests
affects: [77-validate-fix-api-scaffold]

# Tech tracking
tech-stack:
  added: []
  patterns: [singularize-entity-stems, module-name-separation, owned-from-impl]

key-files:
  created: []
  modified:
    - ferro-cli/src/commands/make_api.rs
    - ferro-mcp/src/tools/code_templates.rs

key-decisions:
  - "Added singularize() function to derive singular model names from plural entity file stems"
  - "Separated module_name (for import paths) from model name (for type names) in ModelInfo"
  - "Generate From<Model> (owned) instead of From<&Model> to match ApiResource derive macro"
  - "Use into_iter() instead of iter() for owned model-to-resource conversion"
  - "Generate resources/mod.rs and requests/mod.rs to complete module wiring"

patterns-established:
  - "module_name field: entity import paths must use actual module names, not derived singular names"
  - "Owned From impl: generated resources use From<Model> for compatibility with ApiResource derive"

# Metrics
duration: 15min
completed: 2026-02-28
---

# Plan 03: End-to-end validation of make:api generated code with 32 regression tests

**Ran make:api against real app models, fixed 5 compilation issues, and added 32 template validation tests**

## Performance

- **Duration:** 15 min
- **Started:** 2026-02-28
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Validated make:api generates compilable code against real Ferro models (User, Todo)
- Fixed plural model naming: entity file stems (users.rs, todos.rs) now correctly singularize to User, Todo
- Added module_name field to ModelInfo for correct `crate::models::` import paths
- Generated resources/mod.rs and requests/mod.rs with module declarations
- Changed From<&Model> to From<Model> for ApiResource derive macro compatibility
- Added 32 regression tests covering all template helper functions

## Task Commits

Each task was committed atomically:

1. **Task 1: Run make:api and compile generated code** - `b39338a` (fix)
2. **Task 2: Add make:api template validation test** - `d5a85b3` (test)

## Files Created/Modified
- `ferro-cli/src/commands/make_api.rs` - Fixed model naming, added module_name, singularize(), mod.rs generation, 32 tests
- `ferro-mcp/src/tools/code_templates.rs` - Fixed iter() to into_iter() and from(&ref) to from(owned)

## Decisions Made
- Entity file stems singularized using best-effort English rules (ies->y, ses->s, etc.)
- module_name stored separately from model name since entities/ files use plural names but model types should be singular
- From<Model> (owned) chosen over From<&Model> (reference) to match existing ApiResource derive macro behavior
- resources/mod.rs and requests/mod.rs generation added to complete the module wiring

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Plural model names from entity file stems**
- **Found during:** Task 1 (initial make:api --all run)
- **Issue:** Entity files use plural names (users.rs, todos.rs), producing model names Users/Todos instead of User/Todo
- **Fix:** Added singularize() function and applied to entity file stems
- **Files modified:** ferro-cli/src/commands/make_api.rs
- **Verification:** `ferro make:api Todo --yes` correctly generates Todo, TodoResource, CreateTodoRequest
- **Committed in:** b39338a (Task 1 commit)

**2. [Rule 3 - Blocking] Wrong module import paths after singularization**
- **Found during:** Task 1 (compilation attempt)
- **Issue:** `crate::models::todo` doesn't exist - actual module is `crate::models::todos`
- **Fix:** Added module_name field to ModelInfo, storing actual file stem for import paths
- **Files modified:** ferro-cli/src/commands/make_api.rs
- **Verification:** cargo check -p app succeeds with correct imports
- **Committed in:** b39338a (Task 1 commit)

**3. [Rule 3 - Blocking] From<&Model> vs From<Model> incompatibility**
- **Found during:** Task 1 (compilation attempt)
- **Issue:** Generated controller calls `from(&ref)` but existing ApiResource macro generates `From<Model>` (owned)
- **Fix:** Changed templates to use `From<Model>` (owned) and `into_iter()` instead of `iter()`
- **Files modified:** ferro-cli/src/commands/make_api.rs, ferro-mcp/src/tools/code_templates.rs
- **Verification:** cargo check -p app compiles without errors
- **Committed in:** b39338a (Task 1 commit)

**4. [Rule 3 - Blocking] Missing resources/mod.rs and requests/mod.rs generation**
- **Found during:** Task 1 (compilation setup)
- **Issue:** Generator creates resource/request files but doesn't create or update mod.rs in those directories
- **Fix:** Added generate_resources_mod() and generate_requests_mod() functions
- **Files modified:** ferro-cli/src/commands/make_api.rs
- **Verification:** Generated mod.rs files include all model module declarations
- **Committed in:** b39338a (Task 1 commit)

---

**Total deviations:** 4 auto-fixed (4 blocking)
**Impact on plan:** All fixes necessary for generated code to compile. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 77 complete: all 3 plans executed
- make:api generates compilable code for real models
- 75 total tests across plans 02 and 03 (43 CRUD + 32 template validation)
- Ready for next milestone

---
*Phase: 77-validate-fix-api-scaffold*
*Completed: 2026-02-28*
