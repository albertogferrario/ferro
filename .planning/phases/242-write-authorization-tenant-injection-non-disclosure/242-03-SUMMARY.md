---
phase: 242
plan: 03
slug: write-authorization-tenant-injection-non-disclosure
subsystem: ferro-mcp-server::write / app::controllers::mcp
tags: [security, authorization, mcp, crud, write-path]
requirements: [CRUD-05]

dependency_graph:
  requires: [Plan 01 (derive_crud_plan tenant_column derivation), Plan 02 (execute_crud_plan tenant injection)]
  provides: [McpContext.write_authorized field, fail-closed write-ability enforcement in handle_write_call, host Gate evaluation for write tools]
  affects: [ferro-mcp-server write path, app MCP endpoint, all write tool callers]

tech_stack:
  added: []
  patterns:
    - "Dedicated Option<bool> field on McpContext for write-ability authorization (D-01/D-02)"
    - "Fail-closed early-return in handle_write_call before CRUD prefix loop and confirmation routing"
    - "Host evaluates Gate::authorize_for on mcp_write_ability; strips verb/confirm prefixes to resolve service"
    - "write_authorized: None for read tools (list_); Some(true/false) for write tools"
    - "Existing dispatch tests updated to write_authorized: Some(true) — auth gate tested separately"

key_files:
  modified:
    - ferro-mcp-server/src/renderer.rs
    - ferro-mcp-server/src/write_dispatch.rs
    - ferro-mcp-server/src/jsonrpc.rs
    - app/src/controllers/mcp.rs
    - app/src/tests/mcp_write_dispatch.rs

decisions:
  - "D-02 upheld: write_authorized is a dedicated field, NOT stored in evaluated_guards (visibility filter ≠ auth gate)"
  - "Fail-closed placement: check runs BEFORE CRUD prefix loop and confirmation routing so it uniformly covers create_/update_/delete_ AND request_confirm_/confirm_"
  - "Host is policy owner: ferro-mcp-server carries the pre-evaluated boolean; no live Gate call inside the kernel crate"
  - "Prefix stripping order: request_confirm_ → confirm_ → then trim_start_matches for create_/update_/delete_ (handles nested delete_ in confirm prefix)"
  - "Existing dispatch tests (ferro-mcp-server + app) updated to write_authorized: Some(true) — they test dispatch, not the auth gate"

metrics:
  duration_seconds: 1410
  completed_date: "2026-06-24"
  tasks_completed: 5
  files_modified: 5
---

# Phase 242 Plan 03: Write-Ability Authorization Gate Summary

`McpContext` now carries a dedicated `write_authorized: Option<bool>` field (D-01/D-02). `handle_write_call` enforces it fail-closed before any CRUD or confirmation routing. The host (`app/src/controllers/mcp.rs`) evaluates `Gate::authorize_for` against `mcp_write_ability` for write tools (stripping verb + confirmation prefixes) and populates the field. Full workspace gate green.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add write_authorized field to McpContext | f5693615 | ferro-mcp-server/src/renderer.rs |
| 2 | Fail-closed write-ability enforcement in handle_write_call | b6937b6b | ferro-mcp-server/src/write_dispatch.rs |
| 3 | Host wiring — evaluate mcp_write_ability via Gate for write tools | a466b678 | app/src/controllers/mcp.rs |
| 4 | Framing tests — Gate-deny, scope-deny verification, happy path | 6039d071 | ferro-mcp-server/src/write_dispatch.rs, ferro-mcp-server/src/jsonrpc.rs |
| 5 | Full workspace gate (fmt + clippy + test) | 678175b8 | app/src/controllers/mcp.rs, ferro-mcp-server/src/write_dispatch.rs, app/src/tests/mcp_write_dispatch.rs |

## What Was Built

### Task 1: McpContext.write_authorized field

Added `pub write_authorized: Option<bool>` to `McpContext` in `ferro-mcp-server/src/renderer.rs` after the existing `scope` field. The struct-level doc comment was updated to describe the field's semantics. Because `#[derive(Default)]` is already on the struct, all existing `McpContext::default()` and `McpContext { .. ..Default::default() }` call sites automatically receive `write_authorized: None` — no call site changes required for Task 1.

The field is explicitly documented as separate from `evaluated_guards` (D-02): the guard map is a visibility filter; `write_authorized` is an authorization gate.

### Task 2: Fail-closed enforcement in handle_write_call

Removed the `let _ = ctx;` suppression at the top of `handle_write_call` (ctx is now genuinely read). Immediately after `let tool_name = …` (line 121), inserted the fail-closed write-ability gate:

```rust
if ctx.write_authorized != Some(true) {
    return json!({ "error": { "code": -32603, "message": "authorization: write ability denied" } });
}
```

Placement (line 124) is before the `#[cfg(feature = "confirmation")]` prefix routing (lines 136+) and before the `for prefix in ["create_", "update_", "delete_"]` CRUD loop (line 178). This means the check uniformly covers:
- `create_<svc>` / `update_<svc>` / `delete_<svc>` (CRUD verb tools)
- `request_confirm_delete_<svc>` / `confirm_delete_<svc>` (confirmation flow)
- All `ActionDef`-based write tools

The deny envelope matches the existing tenant fail-closed shape: bare `{ "error": { "code": -32603, "message": "..." } }` — not wrapped in `"result":`.

### Task 3: Host wiring

`app/src/controllers/mcp.rs` now computes `write_authorized: Option<bool>` before building `McpContext`:

**Read tools** (`list_` prefix): `write_authorized = None` — the field is unused on the read path.

**Write tools** (all other names): strips `request_confirm_` / `confirm_` / `create_` / `update_` / `delete_` prefixes to resolve the owning service name, then:
1. If service not found or not `mcp_exposed`: `Some(false)` (fail-closed)
2. If `mcp_write_ability` is `None`: `Some(false)` (fail-closed; CRUD-07 rejects this at boot)
3. If ability is `Some(ability)`: load the concrete `User` via `User::find_by_id(user_id)`, call `Gate::authorize_for(&user, ability, None)`, map `Ok → Some(true)`, `Err → Some(false)`

The stale write-skip comment (lines 250-264, which previously documented why Gate was skipped for write tools) was replaced with the Phase 242 authorization boundary description.

The `McpContext` build now includes `write_authorized`:
```rust
let ctx = McpContext {
    tenant_id,
    scope: key_scope,
    write_authorized,
    ..Default::default()
};
```

The result: two `Gate::authorize_for` call sites in `mcp.rs` — one for read tools (`mcp_ability`), one for write tools (`mcp_write_ability`).

### Task 4: Framing tests

Three new `#[tokio::test]` functions added to `write_dispatch::confirmation_tests` (require `--features confirmation`):

- **`write_authorized_none_denies`**: `write_authorized: None` → `-32603` with message containing `"write ability denied"`, no `"result"` key, executor NOT called.
- **`write_authorized_false_denies`**: `write_authorized: Some(false)` → same deny envelope, executor NOT called.
- **`write_authorized_true_proceeds`**: `write_authorized: Some(true)` → gate does NOT fire, response does not contain `"write ability denied"`, request reaches CRUD dispatch.

Scope-deny test (SC#1 first half) already existed: `read_scope_key_rejected_on_write_tool_name` in `tests/mcp_tenant_isolation.rs` asserts a `read`-scoped key on any write tool returns `-32603` with `"scope insufficient"`.

**Existing dispatch tests updated**: `write_tool_result_parses_as_valid_mcp_content`, `crud_result_structured_envelope`, `delete_bare_call_returns_confirmation_required` (write_dispatch.rs), `crud_tool_call_nti_parses_as_valid_mcp_content`, `crud_nti_not_returned_when_verb_flag_disabled` (jsonrpc.rs) — all updated to `write_authorized: Some(true)` because they test dispatch behavior, not the auth gate.

### Task 5: Full workspace gate

- `cargo fmt --all -- --check`: exit 0 (two formatting fixes: long `match` in mcp.rs, long `assert!` in write_dispatch.rs test)
- `cargo clippy --all --all-targets -- -D warnings`: exit 0, no warnings
- `cargo test --all-features`: exit 0, all suites green

One additional fix during Task 5: `app/src/tests/mcp_write_dispatch.rs` — `call_write_tool` helper and the idempotency test inline `McpContext` both needed `write_authorized: Some(true)` (same pattern as ferro-mcp-server). The `cross_tenant_write_denied` test would otherwise receive `-32603` from the new auth gate rather than the BOLA denial it tests.

## Verification

```
# Three new write_authorized framing tests
cargo test -p ferro-mcp-server --features confirmation write_authorized
# 3 passed: write_authorized_none_denies, write_authorized_false_denies, write_authorized_true_proceeds

# Scope-deny test (SC#1 first half) — existing, passing
cargo test -p ferro-mcp-server read_scope_key_rejected_on_write_tool_name
# 1 passed

# App builds with two Gate::authorize_for sites
cargo build -p app
# Finished (exit 0)

# Order audit: write_authorized check precedes CRUD loop
grep -n "write_authorized != Some(true)\|for prefix in" ferro-mcp-server/src/write_dispatch.rs
# 124: write_authorized check
# 178: for prefix in [...] loop  (124 < 178 ✓)

# Coherence: write_authorized check does NOT consult evaluated_guards
grep -n "evaluated_guards" ferro-mcp-server/src/write_dispatch.rs
# write_dispatch.rs has no evaluated_guards reference (✓)

# Full workspace gate
cargo fmt --all -- --check  # exit 0
cargo clippy --all --all-targets -- -D warnings  # exit 0
cargo test --all-features  # exit 0, all suites green
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Existing dispatch tests broken by new write_authorized gate**

- **Found during:** Task 4 (running tests) and Task 5 (full suite)
- **Issue:** Ten existing tests in `ferro-mcp-server/src/write_dispatch.rs`, two in `ferro-mcp-server/src/jsonrpc.rs`, and two in `app/src/tests/mcp_write_dispatch.rs` used `McpContext::default()` (write_authorized=None). After Task 2 inserted the fail-closed check, those tests received -32603 from the auth gate instead of reaching the dispatch behavior they were testing.
- **Fix:** Updated all affected test contexts to `write_authorized: Some(true)` with a comment clarifying they test dispatch, not auth. The new auth-gate tests (`write_authorized_none_denies`, `write_authorized_false_denies`, `write_authorized_true_proceeds`) explicitly test the None/false/true paths.
- **Files modified:** `ferro-mcp-server/src/write_dispatch.rs`, `ferro-mcp-server/src/jsonrpc.rs`, `app/src/tests/mcp_write_dispatch.rs`
- **Commits:** 6039d071 (ferro-mcp-server tests), 678175b8 (app tests)

**2. [Rule 1 - Bug] rustfmt formatting on mcp.rs match arm and write_dispatch.rs assert**

- **Found during:** Task 5 (cargo fmt --all -- --check)
- **Issue:** Two lines exceeded rustfmt's line width: a long `match services.iter().find(...)` expression in mcp.rs and a long `assert!` condition in write_dispatch.rs.
- **Fix:** Applied the rustfmt-preferred line breaks to both.
- **Commits:** 678175b8

## Known Stubs

None. `write_authorized` is fully evaluated by the host Gate for all write tools. The fail-closed check is unconditional. Plans 01 and 02 (derive_crud_plan + execute_crud_plan) are complete; Plan 04 (validate() CRUD-07 boot test) is the final piece.

## Threat Flags

No new threat surface introduced. This plan closes the T-242-01 mitigation:

- T-242-01 (Elevation of Privilege): `ctx.write_authorized != Some(true)` deny runs before any service lookup or CRUD dispatch. An unauthorized agent cannot learn which write tools exist.
- T-242-03 (Tampering via evaluated_guards reuse): `write_authorized` is a dedicated field, not stored in `evaluated_guards`. Verified: `grep -n "evaluated_guards" ferro-mcp-server/src/write_dispatch.rs` returns no results in the write path.
- T-242-01 (Information Disclosure via deny envelope): deny returns generic `"authorization: write ability denied"` — no service name, row, or ability name disclosed.

## Self-Check: PASSED

- `ferro-mcp-server/src/renderer.rs` — modified, present; `grep -c "pub write_authorized: Option<bool>"` = 1
- `ferro-mcp-server/src/write_dispatch.rs` — modified, present; `grep -c "write_authorized != Some(true)"` = 1; `grep -c "let _ = ctx;"` = 0
- `app/src/controllers/mcp.rs` — modified, present; `grep -c "write_authorized"` = 7; `grep -c "Gate::authorize_for(&user, ability, None)"` = 2
- Commit `f5693615` exists (Task 1)
- Commit `b6937b6b` exists (Task 2)
- Commit `a466b678` exists (Task 3)
- Commit `6039d071` exists (Task 4)
- Commit `678175b8` exists (Task 5)
- `cargo fmt --all -- --check` = exit 0
- `cargo clippy --all --all-targets -- -D warnings` = exit 0
- `cargo test --all-features` = exit 0, 0 failed across all suites
