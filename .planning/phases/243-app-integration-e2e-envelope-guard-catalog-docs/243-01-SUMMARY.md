---
phase: 243-app-integration-e2e-envelope-guard-catalog-docs
plan: "01"
subsystem: app-projections
tags: [crud, projections, service-def, mcp, crud-07]
dependency_graph:
  requires: [Phase 239 deleted_at migration, Phase 241 derive_crud_plan, Phase 242 write-authz gate]
  provides: [order projection CRUD opt-in, CRUD-07 boot-validate pin]
  affects: [app/src/projections/order.rs, ferro-mcp-server CRUD tool surface for order]
tech_stack:
  added: []
  patterns: [ServiceDef CRUD opt-in (four additive builder calls), validate() boot contract test]
key_files:
  modified:
    - app/src/projections/order.rs
decisions:
  - "D-04: .mcp_write_ability('manage-orders') + .creatable/.updatable/.deletable are additive; read ability and StateMachine unchanged"
  - "D-05: status/id/created_at/tenant_id exclusions handled by derive_crud_plan — no change needed in order.rs"
  - "CRUD-07: validate() passes because mcp_write_ability is present alongside write flags; pinned by boot-validate test"
metrics:
  duration_seconds: 164
  completed_date: "2026-06-24"
  tasks_completed: 2
  files_modified: 1
---

# Phase 243 Plan 01: Order Projection CRUD Flip Summary

**One-liner:** Flip the app's order `ServiceDef` from read-only to CRUD with four additive builder calls, and pin the CRUD-07 boot-validate contract with a unit test.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Flip order projection to CRUD | 054efddd | app/src/projections/order.rs |
| 2 | Boot-validate assertion (CRUD-07) | 956686e5 | app/src/projections/order.rs |

## What Was Built

### Task 1 — Order projection CRUD opt-in

Added four builder calls to `app/src/projections/order.rs` immediately after `.mcp_ability("view-orders")` and before `.display_name("Order")`:

```rust
.mcp_write_ability("manage-orders") // write gate: scopes create_/update_/delete_ tools
.creatable(true) // derives create_order tool (CRUD-01)
.updatable(true) // derives update_order tool (CRUD-02)
.deletable(true) // derives delete_order tool, confirmation-gated (CRUD-03)
```

The flip is strictly additive: the existing read ability (`view-orders`), tenant column, StateMachine (`order_lifecycle`), guards, and transition actions are all unchanged. No CRUD verb names were added to `.action(...)`.

### Task 2 — Boot-validate test (TDD GREEN)

Added `#[cfg(test)] mod tests` block with `order_projection_validates_after_crud_flip` that:
- Asserts `svc.creatable`, `svc.updatable`, `svc.deletable` are all `true` after the flip
- Calls `svc.validate().expect(...)` — passes because `mcp_write_ability` is present alongside the write flags (CRUD-07)

## Verification

- `cargo build -p app` — exits 0
- `cargo test -p app order_projection_validates_after_crud_flip` — 1 passed
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean, 0 warnings

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. The projection flip is complete and produces real derived tools. No placeholder data.

## Threat Flags

No new network endpoints, auth paths, or schema changes introduced. The CRUD tool surface widening is gated by the existing `mcp_write_ability("manage-orders")` write gate (T-243-01 per plan threat model). The `status`/`tenant_id` exclusion guarantee (T-243-02) is upheld by `derive_crud_plan` which reads the StateMachine and `tenant_column` that were kept unchanged.

## Self-Check: PASSED

- `app/src/projections/order.rs` — FOUND (modified, 4 additions + 19 lines test block)
- Commit `054efddd` — FOUND (`feat(243-01): flip order projection to CRUD`)
- Commit `956686e5` — FOUND (`test(243-01): add boot-validate assertion`)
