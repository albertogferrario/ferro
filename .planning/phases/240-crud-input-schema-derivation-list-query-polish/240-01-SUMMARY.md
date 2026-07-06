---
phase: 240-crud-input-schema-derivation-list-query-polish
plan: "01"
subsystem: projections
tags: [ferro-projections, service-def, schema-derivation, write-boundary, crud]

requires:
  - phase: 239-soft-delete-data-model-deleted-at-migration
    provides: "is_server_injected_field predicate (Gate A substrate)"

provides:
  - "ServiceDef::is_write_excluded_field(&self, field: &FieldDef, exclude_sm_status: bool) -> bool"
  - "Table test is_write_excluded_field_gates covering all five gates"

affects:
  - "240-02 — schema builders consume is_write_excluded_field directly"
  - "ferro-mcp-server — build_create/update_input_schema will call into this predicate"

tech-stack:
  added: []
  patterns:
    - "Predicate composition: is_write_excluded_field delegates Gate A to is_server_injected_field; all subsequent gates are additive"
    - "SM-conditional Status gate: exclude_sm_status bool is the caller's responsibility (service.state_machine.is_some()), keeping the predicate pure"

key-files:
  created: []
  modified:
    - ferro-projections/src/service.rs

key-decisions:
  - "exclude_sm_status is a caller-supplied bool, not read internally from self.state_machine — keeps predicate pure and reusable regardless of SM presence at call site"
  - "Gate order is load-bearing: server-injected → UpdatedAt → Sensitive → is_list → SM-Status; matches D-03/D-04/D-05/D-07 spec order"

patterns-established:
  - "TDD with RED commit (failing test) → GREEN commit (method) within same plan task"
  - "Schema-only predicates in ferro-projections stay renderer-agnostic; output crates (ferro-mcp-server) call them at schema-build time"

requirements-completed: [CRUD-01, CRUD-02]

duration: 2min
completed: "2026-06-23"
---

# Phase 240 Plan 01: Write-Exclusion Predicate Summary

**`ServiceDef::is_write_excluded_field` — five-gate predicate composing `is_server_injected_field` with UpdatedAt/Sensitive/list/SM-Status exclusions, the single shared source of truth for which declared fields are agent-writable**

## Performance

- **Duration:** 2 min
- **Started:** 2026-06-23T17:17:04Z
- **Completed:** 2026-06-23T17:19:22Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- `ServiceDef::is_write_excluded_field` added to `ferro-projections/src/service.rs` immediately after `is_server_injected_field`
- Five gates implemented in spec order: Gate A (server-injected via delegation), Gate B (UpdatedAt), Gate C (Sensitive), Gate D (is_list), Gate E (Status under SM)
- Table test `is_write_excluded_field_gates` covers 9 cases including the SM-conditional Status pair, the tenant-column path, and a writable FreeText field under both SM flags
- `cargo fmt` + `cargo clippy -p ferro-projections --all-targets -- -D warnings` both exit 0
- Full `cargo test -p ferro-projections` suite: 277 tests green, 0 regressions

## Task Commits

1. **Task 1: Add ServiceDef::is_write_excluded_field predicate + table test** - `43a5d516` (feat)

## Files Created/Modified

- `ferro-projections/src/service.rs` — `is_write_excluded_field` method (lines 248–276) + `is_write_excluded_field_gates` table test

## Decisions Made

- `exclude_sm_status` is a caller parameter rather than reading `self.state_machine.is_some()` directly inside the method. This keeps the predicate pure and decoupled from the SM presence at call time — callers decide the SM context (consistent with the plan specification).
- Gate order follows D-03/D-04/D-05/D-07 spec exactly; changing order would affect short-circuit behavior for overlapping cases (e.g., a Sensitive list field — caught by Gate C before Gate D).

## Deviations from Plan

None — plan executed exactly as written. The TDD cycle (RED compile failure confirmed → GREEN test pass) proceeded without incident. Rustfmt required expanding compact struct-literal syntax in the test array, which `cargo fmt --all` resolved automatically before commit.

## Issues Encountered

- `cargo fmt --all -- --check` failed on the compact one-line struct literal syntax used in the table test cases. Applied `cargo fmt --all` to auto-expand to rustfmt's preferred multi-line form before committing. No logic change.

## Next Phase Readiness

- `is_write_excluded_field` is `pub` and callable from `ferro-mcp-server` crate boundary — ready for Plan 02 schema builders (`build_create_input_schema`, `build_update_input_schema`) to consume it.
- The predicate is the single source of truth; both create and update schema builders will call `service.is_write_excluded_field(field, service.state_machine.is_some())` — no drift possible between them.

---
*Phase: 240-crud-input-schema-derivation-list-query-polish*
*Completed: 2026-06-23*
