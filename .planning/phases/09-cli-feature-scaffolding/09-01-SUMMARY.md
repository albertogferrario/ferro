---
phase: 09-cli-feature-scaffolding
plan: 01
subsystem: cli, scaffolding
tags: [cli, scaffolding, testing, factories, api]

# Dependency graph
requires:
  - phase: 08-mcp-generation-hints
    provides: generation_context tool, code_templates tool
provides:
  - --with-tests flag generating controller test stubs
  - --with-factory flag generating factory with field definitions
  - --auto-routes flag with interactive confirmation
  - --api flag for JSON-only scaffolds
affects: [developer-experience, test-generation, api-development]

# Tech tracking
tech-stack:
  added: [dialoguer (interactive prompts)]
  patterns: [conditional template selection, route injection, test generation]

key-files:
  created: []
  modified:
    - cancer-cli/src/commands/make_scaffold.rs
    - cancer-cli/src/templates/mod.rs
    - cancer-cli/src/main.rs

key-decisions:
  - "Use dialoguer::Confirm for interactive route registration prompts"
  - "API controller template uses json_response! macro throughout"
  - "Test stubs use standard assert!() rather than expect! macro"
  - "Factory fields generated from scaffold field definitions"

patterns-established:
  - "Conditional template selection based on --api flag"
  - "Route insertion via string manipulation (not AST parsing)"
  - "Test file creation in src/tests/ with mod.rs auto-update"
  - "Factory template with Fake::* values based on field types"

# Metrics
duration: 35min
completed: 2026-01-15
---

# Phase 9: CLI Feature Scaffolding Summary

**Four new flags for make:scaffold command: --with-tests, --with-factory, --auto-routes, --api**

## Performance

- **Duration:** 35 min
- **Started:** 2026-01-15
- **Completed:** 2026-01-15
- **Tasks:** 4
- **Files modified:** 3

## Accomplishments
- `--with-tests` flag generates controller test file with 5 CRUD test stubs (index, show, store, update, destroy)
- `--with-factory` flag generates factory with field types matching scaffold definition using Fake::* values
- `--auto-routes` flag automatically registers routes in src/routes.rs with interactive confirmation (--yes to skip)
- `--api` flag generates API-only scaffold with JSON responses, skipping Inertia pages
- All 88 tests pass, no clippy warnings

## Task Commits

Each task was committed atomically:

1. **Task 1: --with-tests flag** - `607ab29` (feat)
2. **Task 2: --with-factory flag** - `2f2c33c` (feat)
3. **Task 3: --auto-routes flag** - `10b23f4` (feat)
4. **Task 4: --api flag** - `1c033ab` (feat)

## Files Modified

- `cancer-cli/src/main.rs` - Added 4 new flags to MakeScaffold command with CLI documentation
- `cancer-cli/src/commands/make_scaffold.rs` - Implemented test generation, factory generation, route registration, and API controller logic
- `cancer-cli/src/templates/mod.rs` - Added scaffold_test_template, scaffold_factory_template, and api_controller_template functions

## New Features

### --with-tests
Generates `src/tests/{snake_name}_controller_test.rs` with:
- test_{plural}_index (GET list)
- test_{plural}_show (GET single)
- test_{plural}_store (POST create)
- test_{plural}_update (PUT)
- test_{plural}_destroy (DELETE)

### --with-factory
Generates `src/factories/{snake_name}_factory.rs` with:
- Factory struct with fields matching scaffold definition
- Fake::* value generation based on field types (string -> word, text -> sentence, bool -> boolean, etc.)
- FactoryTraits stub for named traits

### --auto-routes
- Reads src/routes.rs and injects `resource!("{plural}", controllers::{snake}_controller)` entry
- Adds use statement if not present
- Interactive confirmation via dialoguer (--yes skips prompt)
- Falls back to print_route_instructions() if declined

### --api
- Uses api_controller_template instead of Inertia controller template
- Returns JSON responses with standardized format:
  - index: `{"data": [...], "meta": {"total": n}}`
  - show: `{"data": {...}}`
  - store/update: `{"data": {...}, "message": "..."}`
  - destroy: `{"message": "Deleted successfully"}`
- Skips Inertia page generation
- Updates success message to indicate API-only mode

## Decisions Made

- **dialoguer for prompts**: Used dialoguer::Confirm for interactive route registration to maintain consistency with Rust CLI patterns.
- **String manipulation over AST**: Route injection uses simple string matching rather than Rust AST parsing for simplicity and reliability.
- **Standard assertions in tests**: Used assert!() with TestResponse methods rather than expect! macro for simpler test stubs.
- **Field type mapping**: Factory generation maps scaffold field types to appropriate Fake::* methods.

## Deviations from Plan

None - all 4 tasks executed as specified.

## Issues Encountered

- Variable scoping issue in Task 3 required moving `updated_content` declaration inside the if block.
- Resolved quickly with standard Rust scoping patterns.

## User Setup Required

None - all features work out of the box.

## CLI Help

```
cancer make:scaffold [OPTIONS] <NAME> [FIELDS]...

Options:
      --with-tests    Generate test file with CRUD test stubs
      --with-factory  Generate factory with field definitions
      --auto-routes   Automatically register routes in src/routes.rs
  -y, --yes           Skip confirmation prompt for auto-routes (for CI/automation)
      --api           Generate API-only scaffold (JSON responses, no Inertia pages)
```

## Next Phase Readiness
- CLI scaffold enhancements complete
- All flags documented in --help
- Ready for Phase 10: CLI Smart Defaults

---
*Phase: 09-cli-feature-scaffolding*
*Completed: 2026-01-15*
