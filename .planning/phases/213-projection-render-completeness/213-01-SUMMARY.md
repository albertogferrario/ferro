---
phase: 213-projection-render-completeness
plan: 01
subsystem: ferro-json-ui/projection
tags: [json-ui, projection, render, actions, gap-b]
requirements: [GAP-B]

dependency_graph:
  requires: []
  provides:
    - emit_actions_placeholder emits DropdownMenu from service.actions
    - emit_datatable_root.row_actions populated from service.actions
    - Wave 0 test fixtures (service_with_actions, service_with_state_machine, service_with_money_field)
  affects:
    - ferro-json-ui Browse/Focus/Process/Track projections (actions now wired)

tech_stack:
  added: []
  patterns:
    - TDD RED/GREEN on private emit functions via mod tests
    - serde_json::to_value(TypedProps) + element_with_props pattern
    - /{service.name}/{action.name} action-route convention (documented in fn docstring)
    - /{service.name}/{row_key}/{action.name} DataTable row-action URL template

key_files:
  modified:
    - ferro-json-ui/src/projection/builder.rs

decisions:
  - emit_actions_placeholder emits a single DropdownMenu (not per-action Buttons) — matches gestiscilo row-action pattern and minimizes element count
  - row_key set to "id" when actions are non-empty — conventional PK name; consumers with non-id PKs override at the route level
  - service_with_state_machine and service_with_money_field annotated #[allow(dead_code)] — reserved fixtures for plans 02/03, not consumed yet
  - RED tests committed in same commit as fixtures (Task 1) to satisfy clippy -D dead-code on service_with_actions

metrics:
  duration: 354s
  completed: "2026-06-12"
  tasks: 2
  files: 1
---

# Phase 213 Plan 01: Gap B — Actions Slot + DataTable Row Actions Summary

Gap B wired in a single file edit. Every Browse/Focus/Process/Track projection that declares `ServiceDef.actions` now emits usable action affordances — a DropdownMenu for card/kanban contexts and `row_actions` for DataTable rows. This is the highest-leverage gap: it lifts migrated pages from read-only display to management-capable UI.

## What Was Built

**`emit_actions_placeholder` (Focus / Process / Track — `actions` slot):**
- Replaced no-op stub with a real implementation
- Iterates `service.actions: Vec<ActionDef>` → `Vec<DropdownMenuAction>`
- Emits a single `DropdownMenu` element with `menu_id: "actions_{service.name}"`, `trigger_label: "Actions"`
- URL convention: `POST /{service.name}/{action.name}` (action-route contract documented in fn docstring)
- Early-return when `service.actions.is_empty()` — no element emitted for action-free services

**`emit_datatable_root` (Browse / Track — row actions):**
- Populates `DataTableProps.row_actions` from `service.actions`
- URL template: `/{service.name}/{row_key}/{action.name}` — the DataTable renderer substitutes `{row_key}` per row
- Sets `row_key: Some("id")` when actions are non-empty (conventional PK)
- `row_actions: None` and `row_key: None` preserved when service has no actions

**Wave 0 test fixtures (`mod tests`):**
- `service_with_actions()` — "staff" service with view/edit/delete actions
- `service_with_state_machine()` — "order" service with 3-state lifecycle (reserved for plan 02)
- `service_with_money_field()` — "statistics" service with Money field (reserved for plan 03)

**Render tests (GREEN):**
- `actions_slot_emits_dropdown_from_service_actions` — asserts DropdownMenu emitted, items.len() == service.actions.len(), items[0].label == "View"
- `datatable_root_has_row_actions_from_service_actions` — asserts row_actions is Some(_) with len == service.actions.len()

## Verification Results

| Gate | Result |
|------|--------|
| `cargo test -p ferro-json-ui --lib --features projections -- projection::builder` | 12/12 PASS |
| `actions_slot_emits_dropdown_from_service_actions` | PASS (GREEN) |
| `datatable_root_has_row_actions_from_service_actions` | PASS (GREEN) |
| `from_service_def_validates` | PASS (frozen) |
| `statcard_metadata_is_orphan_element` | PASS (frozen) |
| `cargo test -p ferro-projections --test catalog` | 22/22 PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all --all-targets -- -D warnings` | PASS |
| `git diff --stat` (only ferro-json-ui modified) | PASS |

## Deviations from Plan

**1. [Rule 1 - Bug] RED tests committed in Task 1 rather than a separate commit**
- **Found during:** Task 1 clippy gate — `service_with_actions` would be dead code without a caller
- **Issue:** Clippy `-D warnings` treats unused test helper functions as errors; Task 1 fixtures had no callers until Task 2 tests were written
- **Fix:** Added the two RED-phase test stubs (`actions_slot_emits_dropdown_from_service_actions`, `datatable_root_has_row_actions_from_service_actions`) in the Task 1 commit alongside the fixtures. They fail at that point (RED gate confirmed: 2 FAILED, 10 PASSED). Task 2 commit brings them to GREEN.
- **Files modified:** `ferro-json-ui/src/projection/builder.rs`
- **TDD gate compliance:** RED gate confirmed before GREEN implementation. Gate sequence preserved.

**2. [Rule 2 - Missing critical functionality] `#[allow(dead_code)]` on two deferred fixtures**
- `service_with_state_machine` and `service_with_money_field` reserved for plans 02/03 have no callers yet
- Added `#[allow(dead_code)]` with a comment naming the target plan — correct annotation for intentionally-reserved test helpers

## Known Stubs

None. The two fixtures annotated `#[allow(dead_code)]` are reserved infrastructure for plans 02/03, not stubs blocking this plan's goal. Gap B is fully wired.

## Threat Flags

None. Changes are contained to `emit_actions_placeholder` and `emit_datatable_root` within `ferro-json-ui`. No new network endpoints, auth paths, file access, or schema changes introduced. The action-route convention (`/{service.name}/{action.name}`) is documented — authorization enforcement remains the consumer's route-level responsibility (T-213-01/02/03 accepted per plan threat model).

## Self-Check: PASSED

- `ferro-json-ui/src/projection/builder.rs` — FOUND
- commit `1246c7b6` (test: Wave 0 fixtures + RED) — FOUND
- commit `c4d0d14b` (feat: GREEN implementation) — FOUND
