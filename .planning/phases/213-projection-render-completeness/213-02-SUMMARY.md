---
phase: 213-projection-render-completeness
plan: "02"
subsystem: ferro-json-ui/projection
tags: [json-ui, projection, render, kanban, state-machine, tdd]
dependency_graph:
  requires: [213-01]
  provides: [GAP-A]
  affects: [ferro-json-ui]
tech_stack:
  added: []
  patterns: [state-machine-to-kanban-columns, data_path-binding, ctx-threading]
key_files:
  modified:
    - ferro-json-ui/src/projection/builder.rs
decisions:
  - "emit_kanban_root derives one KanbanColumnProps per StateDef from service.state_machine.states; data_path bound to /data/{name}/columns"
  - "build_display_spec gains ctx: &VisualContext parameter to thread into emit_kanban_root"
  - "ctx.current_state maps to mobile_default_column as active-column approximation (Risk 3 option a)"
  - "static columns in KanbanBoardProps serve as schema fallback when data_path fails to resolve"
metrics:
  duration: "~15 minutes"
  completed: "2026-06-12T20:51:16Z"
  tasks_completed: 1
  tasks_total: 1
  files_changed: 1
---

# Phase 213 Plan 02: Gap A — Kanban State-Machine Columns Summary

Gap A implementation: `emit_kanban_root` derives kanban columns from `ServiceDef.state_machine` and sets `data_path` to bind runtime card data.

## What Was Built

`emit_kanban_root` previously emitted a single placeholder `KanbanColumnProps` (id="default", title=service display name) with no `data_path`, regardless of whether the service had a state machine. Process-intent services (e.g. Orders with draft/submitted/done lifecycle) rendered as a single "Order / 0" card instead of status columns.

The function now:

1. Iterates `service.state_machine.states` when present, emitting one `KanbanColumnProps` per `StateDef` (id = state name, title = display_name or name fallback).
2. Sets `KanbanBoardProps.data_path` to `/data/{service.name}/columns` so the renderer binds runtime column data (counts + card children) from handler output.
3. Falls back to the single-placeholder-column path with `data_path: None` when `state_machine` is `None`.
4. Accepts `ctx: &VisualContext` and maps `ctx.current_state` to `mobile_default_column` for active-column highlighting on mobile.

`build_display_spec` gained a `ctx: &VisualContext` parameter threaded from `from_service_def_with_catalog` (the only call site). Card, DataTable, and StatCard arms ignore the new parameter.

## TDD Gate Compliance

RED: Tests added with `emit_kanban_root(&service, &ctx)` — compile-failed because the old signature took only `service`. Confirmed compilation error before implementing.

GREEN: Implemented `emit_kanban_root(service, ctx)` with state-machine column derivation. Both tests pass.

## Tests

| Test | Result |
|------|--------|
| `kanban_root_derives_columns_from_state_machine` | PASS |
| `kanban_root_fallback_when_no_state_machine` | PASS |
| All 14 builder tests (including 12 pre-existing) | PASS |
| `cargo test -p ferro-projections --test catalog` (22 tests) | PASS |

## Acceptance Criteria Verification

- [x] `kanban_root_derives_columns_from_state_machine` exits 0
- [x] `kanban_root_fallback_when_no_state_machine` exits 0
- [x] `grep "state-machine awareness is a deferred idea"` returns 0 — phrase removed; replaced with accurate doc comment
- [x] `service.state_machine` is read in `emit_kanban_root`
- [x] `/columns` suffix present in `data_path` format string
- [x] Full builder test suite exits 0 — depth invariant preserved (no KanbanBoard children added)
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy --all --all-targets -- -D warnings` clean

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None introduced by this plan. The `mobile_default_column` feature is minimal (maps `ctx.current_state` directly) — this is the documented approximation per Risk 3 option a.

## Threat Flags

None. Column ids/titles derive from declared `StateMachine.states` (not user input). `data_path` is a fixed literal derived from `service.name`. No new trust boundaries introduced.

## Self-Check: PASSED

- `ferro-json-ui/src/projection/builder.rs` — confirmed modified (1 file, +92/-17)
- Commit `18e188a7` — confirmed present in git log
- No unintended file deletions
