---
phase: 23-json-ui-schema
plan: 01
subsystem: ui
tags: [json-ui, sdui, serde, server-driven-ui, component-catalog]

# Dependency graph
requires:
  - phase: v2.1
    provides: inertia crate pattern for framework-agnostic UI crates
provides:
  - ferro-json-ui crate with 10 typed component variants
  - Action declarations with confirm dialogs and outcomes
  - Visibility rules with AND/OR/NOT composition
  - JsonUiView container with builder API and JSON parsing
  - JsonUiConfig for rendering configuration
affects: [24-component-catalog, 25-data-binding, 26-action-system, 28-html-renderer]

# Tech tracking
tech-stack:
  added: [ferro-json-ui]
  patterns: [tagged-serde-enum, component-node-wrapper, untagged-visibility-composition]

key-files:
  created:
    - ferro-json-ui/Cargo.toml
    - ferro-json-ui/src/lib.rs
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/action.rs
    - ferro-json-ui/src/visibility.rs
    - ferro-json-ui/src/view.rs
    - ferro-json-ui/src/config.rs
  modified:
    - Cargo.toml

key-decisions:
  - "Serde tagged enum for Component (type field in JSON)"
  - "Serde untagged enum for Visibility (clean and/or/not JSON syntax)"
  - "ComponentNode wraps Component with key, action, visibility via serde flatten"
  - "ActionOutcome uses tagged enum with type field for redirect/show_errors/refresh/notify"
  - "HttpMethod serializes as UPPERCASE (GET, POST, etc.)"

patterns-established:
  - "Component catalog: serde tag=type enum with props structs"
  - "ComponentNode: shared fields (key, action, visibility) wrapping components via flatten"
  - "Builder pattern for JsonUiView and JsonUiConfig"

# Metrics
duration: 5min
completed: 2026-02-09
---

# Phase 23 Plan 01: Core JSON-UI Schema Types Summary

**ferro-json-ui crate with 10 component types, action system, visibility rules, and view container using serde tagged enums**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-09T06:06:21Z
- **Completed:** 2026-02-09T06:11:00Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Created ferro-json-ui crate with workspace integration
- Defined 10 component types (Card, Table, Form, Button, Input, Select, Alert, Badge, Modal, Text) with typed props
- Built action system with handler references, confirm dialogs, and outcome variants
- Implemented visibility rules with AND/OR/NOT logical composition
- Added JsonUiView builder API with JSON parsing and serialization
- 34 tests (31 unit + 3 doc-tests) all passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Create ferro-json-ui crate with component catalog and action/visibility types** - `49b2446` (feat)
2. **Task 2: Add view container, config, and schema validation with tests** - `2babf20` (feat)

## Files Created/Modified
- `Cargo.toml` - Added ferro-json-ui to workspace members
- `ferro-json-ui/Cargo.toml` - Crate manifest with serde/serde_json dependencies
- `ferro-json-ui/src/lib.rs` - Public API exports and crate documentation
- `ferro-json-ui/src/component.rs` - 10 component types with tagged serde enum and ComponentNode wrapper
- `ferro-json-ui/src/action.rs` - Action, ConfirmDialog, ActionOutcome, HttpMethod types
- `ferro-json-ui/src/visibility.rs` - Visibility enum with AND/OR/NOT composition
- `ferro-json-ui/src/view.rs` - JsonUiView container with builder and JSON methods
- `ferro-json-ui/src/config.rs` - JsonUiConfig with builder pattern

## Decisions Made
- Used `#[serde(tag = "type")]` for Component enum to produce `{"type": "Card", ...}` format
- Used `#[serde(untagged)]` for Visibility enum to allow clean `{"and": [...]}` syntax
- Placed compound variants (And/Or/Not) before Condition in Visibility enum for correct untagged deserialization priority
- Used `#[serde(flatten)]` on ComponentNode to merge component props into the node object
- ActionOutcome uses `#[serde(tag = "type", rename_all = "snake_case")]` for typed variants
- HttpMethod uses `#[serde(rename_all = "UPPERCASE")]` for standard HTTP method format
- Followed ferro-inertia pattern: same Cargo.toml structure, serde-only dependencies

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Core schema types ready for Phase 24 (Component Catalog) to implement HTML rendering
- All types are Serialize + Deserialize, ready for use in handlers and responses
- No blockers or concerns

---
*Phase: 23-json-ui-schema*
*Completed: 2026-02-09*
