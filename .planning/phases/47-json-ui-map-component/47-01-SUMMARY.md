---
phase: 47-json-ui-map-component
plan: 01
subsystem: ui
tags: [json-ui, plugin, registry, layout, assets]

# Dependency graph
requires: []
provides:
  - JsonUiPlugin trait for custom interactive components
  - PluginRegistry with global access (OnceLock + RwLock)
  - Asset type with SRI integrity support
  - collect_plugin_assets function with URL deduplication
  - LayoutContext.scripts field for JS injection before </body>
affects: [47-02, 47-03, 47-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Plugin registry pattern: OnceLock<RwLock<PluginRegistry>> mirrors LayoutRegistry"
    - "with_plugin closure API avoids RwLock lifetime issues"
    - "Asset deduplication via HashSet on URL"

key-files:
  created:
    - ferro-json-ui/src/plugin.rs
  modified:
    - ferro-json-ui/src/lib.rs
    - ferro-json-ui/src/layout.rs
    - framework/src/json_ui/mod.rs

key-decisions:
  - "Used with_plugin closure pattern instead of returning guard reference to avoid RwLock lifetime complexity"
  - "PluginRegistry starts empty (no built-in plugins) unlike LayoutRegistry which ships with 3 defaults"

patterns-established:
  - "Plugin registration: register_plugin() convenience function for global registry"
  - "Asset collection: collect_plugin_assets accepts type names, returns deduplicated CSS/JS/init"

# Metrics
duration: 8min
completed: 2026-02-10
---

# Phase 47 Plan 01: Plugin System Foundation Summary

**JsonUiPlugin trait, PluginRegistry with global OnceLock access, Asset type with SRI, and LayoutContext.scripts field for JS injection**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-10T10:13:44Z
- **Completed:** 2026-02-10T10:21:56Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- JsonUiPlugin trait with component_type, props_schema, render, css_assets, js_assets, init_script methods
- PluginRegistry following LayoutRegistry pattern (OnceLock + RwLock + HashMap)
- Asset type with URL, SRI integrity, and crossorigin attributes (builder pattern)
- collect_plugin_assets function deduplicates CSS/JS by URL using HashSet
- LayoutContext extended with scripts field, base_document injects before </body>
- All 3 built-in layouts (Default, App, Auth) pass scripts through to base_document

## Task Commits

Each task was committed atomically:

1. **Task 1: Create plugin module with JsonUiPlugin trait and PluginRegistry** - `a3a4d62` (feat)
2. **Task 2: Extend LayoutContext with scripts field and update base_document** - `350b236` (feat)

## Files Created/Modified
- `ferro-json-ui/src/plugin.rs` - Plugin trait, Asset type, PluginRegistry, global access, collect_plugin_assets
- `ferro-json-ui/src/lib.rs` - Added plugin module and re-exports
- `ferro-json-ui/src/layout.rs` - Added scripts field to LayoutContext, updated base_document and all 3 layouts
- `framework/src/json_ui/mod.rs` - Updated build_response to construct LayoutContext with empty scripts

## Decisions Made
- Used `with_plugin` closure pattern for global plugin lookup instead of returning a guard that derefs to the plugin. The Deref approach hit lifetime issues where the borrow of `&self` couldn't outlive the guard. The closure pattern is simpler and avoids the issue entirely.
- PluginRegistry starts empty (unlike LayoutRegistry which ships with 3 built-in layouts). Plugins are registered at app startup; the Map plugin will be the first built-in plugin in Plan 03.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plugin system foundation complete, ready for Plan 02 (Component enum Plugin variant)
- LayoutContext.scripts field ready to be wired to actual plugin asset collection
- All existing tests pass without behavioral changes (scripts defaults to empty string)

---
*Phase: 47-json-ui-map-component*
*Completed: 2026-02-10*
