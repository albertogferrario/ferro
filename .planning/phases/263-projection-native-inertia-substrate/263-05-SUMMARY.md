---
phase: 263-projection-native-inertia-substrate
plan: "05"
subsystem: app-tests
tags: [tests, parity, single-source, tenant-scoping, verification, subst-05]
dependency_graph:
  requires: [263-01, 263-02, 263-03, 263-04]
  provides: [SUBST-05]
  affects: [app/src/tests/]
tech_stack:
  added: []
  patterns:
    - "sorted-Vec set comparison for order-independent action-name equality"
    - "framework::projection_read::dispatch called directly (not via MCP surface)"
    - "dispatch_write(.., 'web') as the Inertia write channel assertion"
key_files:
  created:
    - app/src/tests/permitted_actions_parity.rs
    - app/src/tests/data_tenant_scoping.rs
  modified:
    - app/src/tests/single_source.rs
    - app/src/tests/mod.rs
    - app/src/tests/permitted_actions_parity.rs  # cargo fmt
    - ferro-mcp-server/src/renderer.rs  # cargo fmt
    - ferro-projections/src/lib.rs  # cargo fmt
    - ferro-projections/src/schema_contract.rs  # cargo fmt
    - ferro-projections/tests/schema_contract.rs  # cargo fmt
    - framework/src/inertia/projection.rs  # cargo fmt
    - framework/src/lib.rs  # cargo fmt
    - framework/src/projection_read.rs  # cargo fmt
decisions:
  - "data_tenant_scoping is a NEW file (not a crud_e2e.rs extension): crud_e2e.rs drives the MCP surface via handle_tools_call, not framework::projection_read::dispatch — the dispatch-level tenant test requires a separate file to use the framework path"
  - "fmt drift from prior-wave 263-01..04 fixed in this commit via cargo fmt --all; only whitespace/line-wrap changes, no logic"
metrics:
  duration: "~5 minutes"
  completed: "2026-07-27T14:34:52Z"
  tasks: 3
  files: 10
requirements: [SUBST-05]
---

# Phase 263 Plan 05: Parity Tests (SUBST-05) Summary

Single-source parity proven across three axes: permitted-actions visibility,
data tenant isolation, and write-kernel channel reuse.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | permitted-actions parity test | 4b7bf5a7 | permitted_actions_parity.rs, mod.rs |
| 2 | data tenant-scoping test | 813fbe83 | data_tenant_scoping.rs, mod.rs |
| 3 | Inertia write-parity extension + fmt | c882132f | single_source.rs, fmt fixes |

## Parity / Scoping Test Assertions

### Task 1 — permitted_actions_parity.rs (SUBST-02/05)

**`permitted_actions_matches_mcp_tools_list`**
- With `evaluated_guards = {"is_manager": false}`:
  - `ferro::permitted_actions(&service, &guards)` excludes `"approve"` and includes `"submit"`.
  - `render_exposed_tools(&[service], &ctx)` (same guards) — write tools (non-`list_`) exclude `"approve"` and include `"submit"`.
  - The two sorted action-name Vecs are `assert_eq!` equal.

**`state_change_updates_both_surfaces_identically`**
- Guard `false` → `approve` hidden on BOTH surfaces; guard absent → `approve` visible on BOTH.
- Each surface's deny/allow sorted sets are `assert_eq!` equal to the other's.
- The deny set and allow set are `assert_ne!` — proving the guard flip actually changes output.

### Task 2 — data_tenant_scoping.rs (SUBST-03 / T-263-15)

**Decision:** NEW file created. `crud_e2e.rs` drives `handle_tools_call` (MCP surface), not `framework::projection_read::dispatch`. Extending it would not test the framework dispatch path.

**`data_tenant_scoping`**
- `dispatch(&service, json!({}), 10, 0, &db, Some(1))` returns exactly 2 rows, all with `tenant_id = 1`.

**`tenant_isolation_symmetric`**
- `dispatch(&service, json!({}), 10, 0, &db, Some(2))` returns exactly 2 rows, all with `tenant_id = 2`.

**`cross_tenant_id_not_found`** (T-263-15 security regression pin)
- `dispatch(&service, json!({"id": 3}), 10, 0, &db, Some(1))` — id=3 belongs to tenant 2 — returns `rows.is_empty()`. No data disclosure.

### Task 3 — single_source.rs extension (SUBST-05b / T-263-16)

**`single_source_inertia_reuses_web_channel`** (added after existing two tests)
- `drive_visual("submit", json!({"id": 2}), 1, &db)` — calls `dispatch_write(.., "web")`, which is the channel `visual_action::handle` and the Inertia `POST /{service}/{action}` route invoke.
- `drive_mcp("submit", json!({"id": 1}), 1, &db)` — calls `dispatch_write(.., "mcp")`.
- Both persisted `to_state` are `assert_eq!` equal to the derived `"submitted"`.
- `web_audit.first().action == "web.action.submit"` and `mcp_audit.first().action == "mcp.action.submit"` — audit channel tag is the ONLY divergence.

## Scoped Test Results

```
cargo test -p app permitted_actions_parity
running 2 tests
test tests::permitted_actions_parity::tests::permitted_actions_matches_mcp_tools_list ... ok
test tests::permitted_actions_parity::tests::state_change_updates_both_surfaces_identically ... ok
test result: ok. 2 passed; 0 failed

cargo test -p app data_tenant_scoping
running 3 tests
test tests::data_tenant_scoping::tests::cross_tenant_id_not_found ... ok
test tests::data_tenant_scoping::tests::data_tenant_scoping ... ok
test tests::data_tenant_scoping::tests::tenant_isolation_symmetric ... ok
test result: ok. 3 passed; 0 failed

cargo test -p app single_source
running 4 tests
test tests::single_source::tests::single_source_guard_rejects_both ... ok
test tests::single_source::tests::single_source_inertia_reuses_web_channel ... ok
test tests::single_source::tests::single_source_both_channels ... ok
test tests::crud_e2e::tests::crud_mcp_visual_single_source_parity ... ok
test result: ok. 4 passed; 0 failed
```

## Full CI-Exact Gate

`cargo fmt --all -- --check`: exit 0 (clean after fixing whitespace drift from prior waves 263-01..04)

Full CI-exact gate (clippy --all --all-targets -D warnings + test --all-features): DEFERRED to orchestrator post-clean rebuild (disk-managed).

## Deviations from Plan

### Auto-applied

**1. [Rule 2 - Missing] cargo fmt drift from prior waves**
- Found during: Task 3 (fmt --check revealed drift in 7 files from 263-01..04 commits)
- Fix: `cargo fmt --all` applied; whitespace/line-wrap only, no logic changes
- Files: ferro-projections/src/lib.rs, schema_contract.rs, tests/schema_contract.rs, framework/src/lib.rs, projection_read.rs, inertia/projection.rs, ferro-mcp-server/src/renderer.rs
- Commit: c882132f (bundled with Task 3)

**2. [Rule 3 - Blocking] `framework::projection_read` import path**
- Found during: Task 2 compilation
- Issue: App Cargo.toml imports framework as `ferro = { package = "ferro-rs" }` — the import alias is `ferro`, not `framework`
- Fix: Changed `use framework::projection_read::dispatch` → `use ferro::projection_read::dispatch`
- Files: data_tenant_scoping.rs
- Commit: 813fbe83

## Known Stubs

None — all tests assert real behavior against real data; no placeholder logic.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes introduced. All changes are test files.

## Self-Check: PASSED

```bash
[ -f "app/src/tests/permitted_actions_parity.rs" ] → FOUND
[ -f "app/src/tests/data_tenant_scoping.rs" ] → FOUND
git log --oneline | grep 4b7bf5a7 → FOUND
git log --oneline | grep 813fbe83 → FOUND
git log --oneline | grep c882132f → FOUND
```
