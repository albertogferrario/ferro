---
phase: 29-layout-system
plan: 01
subsystem: ui
tags: [layout, registry, html, tailwind, partials, ssr]

# Dependency graph
requires:
  - phase: 28-html-renderer
    provides: render_to_html component renderer and html_escape function
provides:
  - Layout trait and LayoutContext for page wrapping
  - LayoutRegistry with global access for named layout lookup
  - DefaultLayout, AppLayout, AuthLayout built-in layouts
  - NavItem, SidebarSection types with navigation/sidebar/footer partials
affects: [29-02-framework-integration, 30-cli-scaffolding]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "OnceLock<RwLock<T>> global registry pattern (matches route name registration)"
    - "base_document helper to avoid HTML boilerplate duplication across layouts"
    - "Partial functions returning String for composable layout sections"

key-files:
  created:
    - ferro-json-ui/src/layout.rs
  modified:
    - ferro-json-ui/src/lib.rs
    - ferro-json-ui/src/render.rs

key-decisions:
  - "All layouts, partials, and registry in single layout.rs module (not separate partial.rs)"
  - "html_escape made pub(crate) in render.rs for cross-module reuse"
  - "AppLayout uses empty partials by default, users create custom Layout impls with real data"

patterns-established:
  - "Layout trait: Send + Sync with render(&self, ctx: &LayoutContext) -> String"
  - "LayoutRegistry: register/render/has with fallback to default on missing name"
  - "Partial functions: navigation(), sidebar(), footer() returning HTML String"

# Metrics
duration: 3min
completed: 2026-02-09
---

# Phase 29 Plan 01: Layout Trait, Registry, Default Layouts, and Partials Summary

**Layout trait with OnceLock registry, three built-in layouts (default/app/auth), and composable partial functions for navigation, sidebar, and footer**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-09T08:41:01Z
- **Completed:** 2026-02-09T08:44:01Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Layout trait (`Send + Sync`) with LayoutContext struct containing title, content, head, body_class, view_json, data_json
- Three built-in layouts: DefaultLayout (minimal HTML shell), AppLayout (dashboard with nav + sidebar), AuthLayout (centered card)
- base_document helper eliminates HTML boilerplate duplication across all layouts
- LayoutRegistry with register/render/has and automatic fallback to default
- Global registry via `OnceLock<RwLock<LayoutRegistry>>` with register_layout/render_layout convenience functions
- NavItem and SidebarSection types with navigation(), sidebar(), footer() partial functions
- All user strings escaped via html_escape for XSS prevention
- 27 unit tests covering layouts, registry, partials, escaping, and global functions

## Task Commits

Each task was committed atomically:

1. **Task 1+2: Layout module with traits, registry, layouts, and partials** - `bf12108` (feat)

**Note:** Tasks 1 and 2 were implemented together in a single commit because the partials (Task 2) are tightly coupled with the layouts (Task 1) -- AppLayout directly calls navigation() and sidebar(). Splitting into two commits would have required a broken intermediate state.

## Files Created/Modified
- `ferro-json-ui/src/layout.rs` - Layout trait, LayoutContext, LayoutRegistry, DefaultLayout, AppLayout, AuthLayout, partials, global registry, 27 tests
- `ferro-json-ui/src/lib.rs` - Added `pub mod layout` and re-exports for all public types
- `ferro-json-ui/src/render.rs` - Changed html_escape from `fn` to `pub(crate) fn` for cross-module use

## Decisions Made
- Combined partials (Task 2) into layout.rs alongside layouts (Task 1) rather than separate partial.rs -- single module is simpler and the research doc's suggestion of separate files adds unnecessary indirection for 3 small functions
- Made html_escape `pub(crate)` instead of duplicating the function -- follows DRY, research doc explicitly advised against hand-rolling HTML escaping
- AppLayout renders empty navigation/sidebar by default -- users create custom Layout impls that call partials with real NavItem/SidebarSection data

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Layout infrastructure complete in ferro-json-ui
- Ready for Plan 02 to integrate layout system into framework render pipeline
- LayoutRegistry, LayoutContext, and all three layouts available for framework import

---
*Phase: 29-layout-system*
*Completed: 2026-02-09*
