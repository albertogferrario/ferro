---
phase: 98-ferro-json-ui-stable-release
plan: "03"
subsystem: ui
tags: [ferro-json-ui, schemars, json-schema, api-surface, visibility]

# Dependency graph
requires:
  - phase: 98-01
    provides: StatCardProps, ChecklistProps, ToastProps, NotificationDropdownProps, SidebarProps, HeaderProps, ToastVariant
  - phase: 98-02
    provides: DashboardLayout, DashboardLayoutConfig, runtime.rs module

provides:
  - schemars JsonSchema derives on all public leaf component and action types
  - Restricted internal helpers (resolve_path, resolve_path_string, collect_plugin_types) to pub(crate)
  - Stable public API surface with layout internals removed from user-facing exports
  - Framework re-exports updated with Plan 01 + Plan 02 types
  - serde_json re-export removed from ferro-json-ui
  - Experimental disclaimer removed from crate docs

affects: [ferro-mcp, ferro-projections, app, 98-04, 98-05]

# Tech tracking
tech-stack:
  added: [schemars = { version = "1", features = ["derive"] } in ferro-json-ui]
  patterns:
    - JsonSchema derives on leaf types only; types containing Component via flatten are excluded (Component has custom serde)
    - pub(crate) for internal render/data pipeline helpers
    - Framework re-exports as single import surface for users

key-files:
  created: []
  modified:
    - ferro-json-ui/Cargo.toml
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/action.rs
    - ferro-json-ui/src/visibility.rs
    - ferro-json-ui/src/config.rs
    - ferro-json-ui/src/data.rs
    - ferro-json-ui/src/render.rs
    - ferro-json-ui/src/lib.rs
    - framework/src/lib.rs

key-decisions:
  - "JsonSchema skipped on Component/PluginProps (custom serde) AND ComponentNode/CardProps/FormProps/ModalProps/Tab/TabsProps/JsonUiView (transitively contain Component via flatten — schemars requires T: JsonSchema for all fields)"
  - "resolve_path and resolve_path_string doctests removed when demoted to pub(crate) — external API examples must not reference crate-private items"
  - "AppLayout/AuthLayout/DefaultLayout remain pub in layout.rs for framework use but removed from lib.rs top-level re-exports — users select layouts by name string not struct"

patterns-established:
  - "JsonSchema on leaf types: derive on all enums and structs that do NOT transitively contain Component"
  - "pub(crate) for internal helpers that only serve the render pipeline"
  - "Framework re-exports are the single authoritative public API; ferro-json-ui re-exports are implementation details"

requirements-completed: [API-01, API-02, API-03, API-04]

# Metrics
duration: 17min
completed: "2026-03-11"
---

# Phase 98 Plan 03: API Surface Audit Summary

**schemars JsonSchema derives on 40+ public types, internal helpers demoted to pub(crate), framework re-exports updated for Plans 01/02, stable API documentation**

## Performance

- **Duration:** 17 min
- **Started:** 2026-03-11T16:32:08Z
- **Completed:** 2026-03-11T16:49:00Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Added schemars 1.x to ferro-json-ui with JsonSchema derives on all public leaf types (40+ types across action, visibility, config, component modules)
- Demoted resolve_path, resolve_path_string (data.rs), and collect_plugin_types (render.rs) to pub(crate)
- Removed layout internals (AppLayout, AuthLayout, DefaultLayout, navigation, sidebar, footer, global_registry) from user-facing exports
- Removed serde_json re-export from ferro-json-ui
- Updated experimental disclaimer to stable API description
- Updated framework/src/lib.rs re-exports: removed 14 internal items, added 13 Plan 01/02 types

## Task Commits

Each task was committed atomically:

1. **Task 1: Add schemars dependency and JsonSchema derives** - `2726b4f` (feat)
2. **Task 2: Audit visibility and update framework re-exports** - `f1108c8` (feat)

**Plan metadata:** (in this commit via docs command)

## Files Created/Modified
- `ferro-json-ui/Cargo.toml` - Added schemars = { version = "1", features = ["derive"] }
- `ferro-json-ui/src/component.rs` - JsonSchema on 36 leaf types; 6 types skipped with comment (Component, PluginProps, ComponentNode, CardProps, FormProps, ModalProps, Tab, TabsProps)
- `ferro-json-ui/src/action.rs` - JsonSchema on Action, ActionOutcome, ConfirmDialog, DialogVariant, HttpMethod, NotifyVariant
- `ferro-json-ui/src/visibility.rs` - JsonSchema on Visibility, VisibilityCondition, VisibilityOperator
- `ferro-json-ui/src/config.rs` - JsonSchema on JsonUiConfig
- `ferro-json-ui/src/data.rs` - resolve_path, resolve_path_string demoted to pub(crate); doctests removed
- `ferro-json-ui/src/render.rs` - collect_plugin_types demoted to pub(crate)
- `ferro-json-ui/src/lib.rs` - Removed internal re-exports, stable doc comment, removed serde_json re-export
- `framework/src/lib.rs` - Removed internal items, added Plan 01/02 types

## Decisions Made

- **JsonSchema exclusions propagate transitively**: `Component` has custom serde → cannot derive JsonSchema → any type containing `Vec<ComponentNode>` or `Component` directly/indirectly cannot derive JsonSchema. This excludes ComponentNode, CardProps, FormProps, ModalProps, Tab, TabsProps, and JsonUiView. Leaf types (simple props, enums) all get JsonSchema.
- **Doctests removed with pub(crate) demotion**: The `resolve_path` and `resolve_path_string` functions had doctests that referenced `ferro_json_ui::resolve_path`. Once demoted to pub(crate), these become invalid external references. Removed them; internal unit tests in the same file still cover the functions.
- **AppLayout/AuthLayout/DefaultLayout strategy**: Remain pub in layout.rs (needed for `impl Layout for AppLayout` pattern), but not re-exported at the crate root. Framework code accesses via `ferro_json_ui::layout::AppLayout`. Users interact with layouts by string name only.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed JsonSchema from ComponentNode and component types containing ComponentNode**
- **Found during:** Task 1 (JsonSchema derive implementation)
- **Issue:** Plan specified JsonSchema on ComponentNode, but ComponentNode has `#[serde(flatten)] pub component: Component`. Schemars requires `T: JsonSchema` for flattened fields, and Component has custom serde (cannot derive JsonSchema). This cascades to any type containing `Vec<ComponentNode>`: CardProps, FormProps, ModalProps, Tab, TabsProps, and JsonUiView.
- **Fix:** Removed JsonSchema from these 8 types, added `// JsonSchema skipped: ...` comments explaining why
- **Files modified:** ferro-json-ui/src/component.rs, ferro-json-ui/src/view.rs
- **Verification:** Compilation succeeds; 317 unit tests pass
- **Committed in:** 2726b4f (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - compilation bug from derive constraint)
**Impact on plan:** JsonSchema coverage is on all structurally derivable types. The excluded types (ComponentNode, CardProps, etc.) wrap Component which has intentional custom serde — this limitation is structural, not a gap.

## Issues Encountered

- Disk space (460GB partition at 100% capacity) caused build failures during full `cargo build --all`. Resolved by cleaning async-stripe build artifacts. Tests run on specific crates (ferro-json-ui, ferro-rs, ferro-mcp) passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 98-04 (docs update) can reference the stable API surface defined in this plan
- Phase 98-05 (release packaging) has the complete re-export surface to validate
- ferro-mcp with_plugin access confirmed working
- All 317 unit tests + 5 doc tests pass

## Self-Check: PASSED

- FOUND: .planning/phases/98-ferro-json-ui-stable-release/98-03-SUMMARY.md
- FOUND: schemars in ferro-json-ui/Cargo.toml
- FOUND: commit 2726b4f (Task 1)
- FOUND: commit f1108c8 (Task 2)
- FOUND: commit 16ed712 (metadata)

---
*Phase: 98-ferro-json-ui-stable-release*
*Completed: 2026-03-11*
