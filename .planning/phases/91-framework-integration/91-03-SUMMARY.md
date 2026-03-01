---
phase: 91-framework-integration
plan: 03
subsystem: mcp
tags: [mcp, service-projections, introspection, json-ui, intent-derivation]

# Dependency graph
requires:
  - phase: 84-service-def
    provides: ServiceDef, DataType, FieldMeaning, builder API
  - phase: 89-intent-graph-generation
    provides: derive_intents() function, IntentScore types
  - phase: 90-renderer-trait
    provides: JsonUiRenderer, Renderer trait, RenderContext, RenderMode
provides:
  - list_projections MCP tool for discovering ServiceDef functions
  - inspect_projection MCP tool for parsing field/relationship/action structure
  - render_projection MCP tool for reconstructing ServiceDef and rendering JSON-UI
affects: [93-field-test]

# Tech tracking
tech-stack:
  added: [ferro-projections dependency in ferro-mcp]
  patterns: [source-scanning regex pattern for ServiceDef builder calls, ServiceDef reconstruction from source]

key-files:
  created: [ferro-mcp/src/tools/list_projections.rs, ferro-mcp/src/tools/inspect_projection.rs, ferro-mcp/src/tools/render_projection.rs]
  modified: [ferro-mcp/Cargo.toml, ferro-mcp/src/tools/mod.rs, ferro-mcp/src/service.rs]

key-decisions:
  - "Source-scanning approach using regex (matching json_ui_inspect pattern) rather than runtime ServiceDef evaluation"
  - "render_projection reconstructs ServiceDef from parsed source, then calls derive_intents() and JsonUiRenderer.render()"
  - "InspectResult enum with untagged serde for Found/NotFound variants"
  - "All field types (field, optional_field, read_only_field, write_only_field) parsed with correct readable/writable flags"

patterns-established:
  - "Projection source scanning: regex-based extraction of ServiceDef builder calls from src/projections/*.rs"
  - "ServiceDef reconstruction: parse source -> build ServiceDef programmatically -> derive intents -> render"

# Metrics
duration: 15min
completed: 2026-03-01
---

# Phase 91 Plan 03: MCP Projection Introspection Tools

**Three MCP tools (list_projections, inspect_projection, render_projection) enable AI agents to discover, inspect, and render ServiceDef projections**

## Performance

- **Duration:** 15 min
- **Started:** 2026-03-01
- **Completed:** 2026-03-01
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Added `ferro-projections` dependency to ferro-mcp for access to ServiceDef, derive_intents, and JsonUiRenderer
- Created `list_projections` tool scanning `src/projections/` for ServiceDef functions with name/file/service_name/display_name extraction
- Created `inspect_projection` tool parsing fields (with readable/writable flags), relationships, actions, state machine presence, and intent hints from source
- Created `render_projection` tool that reconstructs ServiceDef from source, derives intents, and renders JSON-UI output via JsonUiRenderer
- Registered all 3 tools in service.rs with param structs and descriptive tool annotations
- 17 new tests covering serialization, parsing, reconstruction, mode parsing, and error handling

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ferro-projections dependency and create list_projections + inspect_projection tools** - `a8ada75` (feat)
2. **Task 2: Create render_projection tool and register all 3 tools in service.rs** - `2415777` (feat)

## Files Created/Modified
- `ferro-mcp/Cargo.toml` - Added ferro-projections dependency
- `ferro-mcp/src/tools/list_projections.rs` - Source scanner discovering ServiceDef functions in src/projections/
- `ferro-mcp/src/tools/inspect_projection.rs` - Deep parser extracting fields, relationships, actions, state machine, intent hints
- `ferro-mcp/src/tools/render_projection.rs` - ServiceDef reconstructor + intent deriver + JSON-UI renderer
- `ferro-mcp/src/tools/mod.rs` - Module declarations for 3 new tools
- `ferro-mcp/src/service.rs` - 3 param structs and 3 tool handlers with descriptions

## Decisions Made
- Source-scanning via regex matches the established `json_ui_inspect.rs` pattern for consistency
- `render_projection` reconstructs ServiceDef from parsed source rather than trying to execute Rust code dynamically
- `InspectResult` uses `#[serde(untagged)]` enum for clean JSON output (Found returns detail, NotFound returns error with available list)
- State machine reconstruction uses pre-collected final state set to avoid regex compilation in loops

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Clippy caught regex creation inside a loop (state machine parsing) - restructured to pre-collect final states into a HashSet
- Type inference ambiguity on HashSet::contains with &str vs String - resolved by converting to owned String before lookup

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 3 projection MCP tools are operational and tested (17 tests, 147 total ferro-mcp tests)
- Ready for Phase 93 field test and any further MCP integration work
- `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` all pass

---
*Phase: 91-framework-integration*
*Completed: 2026-03-01*
