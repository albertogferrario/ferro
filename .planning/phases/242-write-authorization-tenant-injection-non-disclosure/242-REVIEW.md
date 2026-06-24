---
phase: 242-write-authorization-tenant-injection-non-disclosure
reviewed: 2026-06-24T00:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - ferro-projections/src/executor.rs
  - ferro-projections/src/service.rs
  - framework/src/write/mod.rs
  - ferro-mcp-server/src/renderer.rs
  - ferro-mcp-server/src/write_dispatch.rs
  - app/src/controllers/mcp.rs
findings:
  critical: 2
  warning: 2
  info: 1
  total: 5
status: fixes_applied
---

# Phase 242: Code Review Report

**Reviewed:** 2026-06-24T00:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 242 closes the CRUD write-safety envelope: tenant column derivation in `executor.rs`, SQL tenant-binding in `framework/src/write/mod.rs`, the dedicated `write_authorized` signal in `renderer.rs`, fail-closed enforcement in `write_dispatch.rs`, and Gate evaluation in `app/src/controllers/mcp.rs`.

The SQL parameter-binding arithmetic is correct (tenant placeholder index and value position agree), all values are bound rather than interpolated, and the non-disclosure invariant (cross-tenant yields `RecordNotFound`) is properly implemented. The `write_authorized` fail-closed check is the first thing `handle_write_call` executes, before confirmation and CRUD routing.

Two critical issues are present:

1. The service-name resolver in `app/src/controllers/mcp.rs` uses chained `trim_start_matches` instead of `strip_prefix`. Because `trim_start_matches` strips the pattern greedily and the three calls are chained on the same string, transition-action tools (non-CRUD verbs like `"submit"`, `"approve"`) never resolve to a service, causing every transition-action write to be denied with `write_authorized = Some(false)`. CRUD verb tools resolve correctly because a single verb prefix is consumed and the remainder is the service name. This blocks the Phase 242 write path for all pre-existing action tools.

2. The SQLite post-INSERT SELECT in `execute_crud_plan` is not scoped to the inserting tenant's `tenant_id`. In a multi-tenant concurrent environment with a shared SQLite connection, `last_insert_rowid()` is per-connection but the subsequent `SELECT * FROM {table} WHERE id = ?` has no tenant predicate. The row with `last_insert_rowid()` is always the one just inserted (same connection), so the returned data is correct — but the SELECT does not verify the inserted row belongs to the calling tenant. If `last_insert_rowid()` somehow returns a stale value (connection re-use edge cases in certain pool configurations), a different tenant's row could be returned to the caller. The Phase 242 Pitfall 5 fix was correctly applied to the post-UPDATE SELECT but was not applied to the post-INSERT SELECT.

---

## Critical Issues

### CR-01: Transition-action write tools always denied — `trim_start_matches` misuse in service-name resolver

**File:** `app/src/controllers/mcp.rs:324-330`

**Issue:** The `write_authorized` block resolves a service name from the write-tool name by first stripping optional confirmation prefixes (`strip_prefix`), then chaining three `trim_start_matches` calls:

```rust
let svc_name = tool_name
    .strip_prefix("request_confirm_")
    .or_else(|| tool_name.strip_prefix("confirm_"))
    .unwrap_or(tool_name)
    .trim_start_matches("create_")
    .trim_start_matches("update_")
    .trim_start_matches("delete_");
```

`trim_start_matches` removes all leading occurrences of the pattern, unlike `strip_prefix` which removes exactly one. The three calls are chained on the result of the previous. For a CRUD verb tool like `"create_order"`, the first call strips `"create_"` to yield `"order"` and the remaining two are no-ops — correct. But for a transition-action tool like `"submit"` or `"approve_on_order"`, none of the three patterns match, so `svc_name` ends up as `"submit"` or `"approve_on_order"`. `services.iter().find(|s| s.name == svc_name)` then finds no service, returning `None`, which maps to `write_authorized = Some(false)`. The fail-closed check in `handle_write_call` (`ctx.write_authorized != Some(true)`) then denies all transition-action writes unconditionally. The sample `app` crate's `order` projection has `mcp_write_ability` for CRUD verbs, but transition actions (expose via `service_def()`) will be blocked at this gate.

**Fix:** Use `strip_prefix` with a match on the verb that actually appears, replacing the chained `trim_start_matches` chain with a proper single-verb strip. The service name for action tools must be looked up differently — either by iterating services to find which one owns an action with that tool name, or by recognizing that action tools are not CRUD tools and should resolve the owning service via `find_action`:

```rust
let write_authorized: Option<bool> = if tool_name.starts_with("list_") {
    None
} else {
    // Strip optional confirmation prefix first.
    let base = tool_name
        .strip_prefix("request_confirm_")
        .or_else(|| tool_name.strip_prefix("confirm_"))
        .unwrap_or(tool_name);

    // Try CRUD verb prefixes (exactly one strip).
    let svc_name = base
        .strip_prefix("create_")
        .or_else(|| base.strip_prefix("update_"))
        .or_else(|| base.strip_prefix("delete_"))
        .unwrap_or(base); // fallback: bare action name — use find_action to resolve service

    // Prefer CRUD service lookup; fall back to action-based service lookup.
    let svc = services
        .iter()
        .find(|s| s.mcp_exposed && s.name == svc_name)
        .or_else(|| {
            // For bare action tools, find the service that owns the action.
            services.iter().find(|s| {
                s.mcp_exposed && s.actions.iter().any(|a| {
                    a.name == base || format!("{}_on_{}", a.name, s.name) == base
                })
            })
        });

    match svc {
        Some(svc) => match svc.mcp_write_ability.as_deref() {
            Some(ability) => {
                let user = /* … load user … */;
                Some(ferro::authorization::Gate::authorize_for(&user, ability, None).is_ok())
            }
            None => Some(false),
        },
        None => Some(false),
    }
};
```

---

### CR-02: Post-INSERT SELECT not scoped to tenant — untenanted data disclosure risk on SQLite

**File:** `framework/src/write/mod.rs:363-378`

**Issue:** After a successful `INSERT`, the SQLite path calls `last_insert_rowid()` then fetches the full inserted record:

```rust
let select_sql = format!("SELECT * FROM {table} WHERE id = ?");
let select_stmt = Statement::from_sql_and_values(
    backend,
    &select_sql,
    vec![sea_orm::Value::BigInt(Some(inserted_id))],
);
```

This SELECT has no `AND {tenant_column} = ?` predicate, even when `tenant_column` is `Some`. The Phase 242 security comment (line 272: T-242-03, "Pitfall 5") was correctly applied to the post-UPDATE SELECT (lines 443-460) but was not applied here. In a single-connection SQLite scenario, `last_insert_rowid()` is per-connection and reliable. However, if a connection pool returns a connection on which a concurrent INSERT by another tenant has run (possible with certain pool configurations), the `last_insert_rowid()` would return the other tenant's `id`, and the subsequent untenanted SELECT would return the other tenant's row to the caller. The tenant's own row is still inserted correctly — the vulnerability is only in the data returned to the caller after insert.

**Fix:** Add the tenant predicate to the post-INSERT SELECT when `tenant_column` is `Some`, mirroring the post-UPDATE SELECT pattern:

```rust
let (select_sql, select_values) = if let Some(ref tc) = tenant_column {
    let t_ph = placeholder(backend, 2);
    (
        format!(
            "SELECT * FROM {table} WHERE id = ? AND {tc_col} = {t_ph}",
            tc_col = tc.column
        ),
        vec![
            sea_orm::Value::BigInt(Some(inserted_id)),
            sea_orm::Value::BigInt(Some(tenant_id)),
        ],
    )
} else {
    (
        format!("SELECT * FROM {table} WHERE id = ?"),
        vec![sea_orm::Value::BigInt(Some(inserted_id))],
    )
};
```

---

## Warnings

### WR-01: Denial audit not written for CRUD-verb guard failures

**File:** `ferro-mcp-server/src/write_dispatch.rs:336-349`

**Issue:** When a guard fails (`WriteError::GuardFailed`) on the transition-action path, `handle_write_call` writes a forensic audit entry (lines 342-349). The CRUD verb dispatch block (lines 178-273) has no equivalent denial audit. CRUD verbs currently carry no action-level preconditions so no guard evaluation occurs on that path today. However, as guard support on CRUD verbs is added (the comment at line 436 explicitly marks this as a "Phase 242 extension point"), the absence of a denial audit will silently break the forensic trail invariant (D-05). The gap is latent now but would become live with the first CRUD guard.

**Fix:** Extract the denial audit into a shared helper and call it from both the action path and the CRUD path whenever `WriteError::GuardFailed` is returned, so the invariant is structurally guaranteed rather than contingent on remembering to add audit calls at each new call site.

---

### WR-02: `validate_crud_verb_without_write_ability` test in `service.rs` uses `mcp_write_ability` naming inconsistency in message assertion

**File:** `ferro-projections/src/service.rs:2307`

**Issue:** The test `validate_rejects_crud_verb_without_write_ability` asserts `err.to_string().contains("mcp_write_ability")` (line 2307). This is correct and matches the error message at line 506. However, the test name says `validate_crud_verb_without_write_ability` while the preceding three tests in the same group use `validate_rejects_creatable_without_write_ability` etc. (lines 2021-2036). Those older tests only call `.unwrap_err()` and assert the variant, not the message content. If the error message at line 506 is changed (e.g., renamed field), only the newer consolidated test will catch the regression — the three older tests would pass regardless of message content. This creates an asymmetric test coverage gap.

**Fix:** Either add a message-content assertion to the three older tests as well, or remove the redundant older tests in favour of the consolidated one, keeping the message assertion intact.

---

## Info

### IN-01: `write_tool_error_result` response shape differs from `make_tool_deny_response` for write-path auth errors

**File:** `ferro-mcp-server/src/write_dispatch.rs:61-72` and `app/src/controllers/mcp.rs:166-178`

**Issue:** The `write_authorized` failure path in `handle_write_call` (lines 124-131) returns a JSON-RPC error envelope (`{"error": {"code": -32603, "message": "..."}}`) rather than a tool-error result (`{"result": {"isError": true, "content": [...]}}`) used by `write_tool_error_result`. The `make_tool_deny_response` helper in the app controller also produces `{"result": {"isError": true, ...}}` (the D-09 shape). The inconsistency means that the write-ability denial arrives at the JSON-RPC transport layer differently from all other write-path denials, requiring clients to handle two distinct error shapes. The current tests confirm the `-32603` shape, so changing this would be a breaking test change — but it is worth flagging as a shape inconsistency to address intentionally.

**Fix:** If uniform error shape is desired, replace the inline `json!({"error": ...})` return with a `write_tool_error_result` call. If the transport-level error is intentional (to distinguish authorization from application errors), document the decision in a comment alongside the check.

---

_Reviewed: 2026-06-24T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
