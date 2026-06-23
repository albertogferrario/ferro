# Phase 242: Write Authorization, Tenant Injection & Non-Disclosure — Research

**Researched:** 2026-06-24
**Domain:** ferro-mcp-server write path / ferro-projections / framework::write kernel
**Confidence:** HIGH (all claims verified against actual source files in this session)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Enforce `.mcp_write_ability` via a dedicated, fail-closed authorization signal
  carried into the write path, checked in `ferro-mcp-server` (`handle_write_call`) BEFORE
  `dispatch_write`. A `None`/absent ability result denies.
- **D-02:** Do NOT reuse `McpContext.evaluated_guards` for the write-ability check.
  `renderer.rs:210` documents that the guard map is a *visibility* filter, NOT an auth gate.
- **D-03:** (Research question, now resolved — see below) Concrete carrier shape for the
  write-authorization signal in `McpContext`.
- **D-04:** `derive_crud_plan` fills `tenant_column: Some(TenantColumn { column })` from
  `svc.tenant_column` when the projection declares one.
- **D-05:** `execute_crud_plan` binds the runtime `tenant_id` when `tenant_column` is `Some`:
  Create → append `(tenant_column, tenant_id)` to INSERT. Update/Delete → add
  `AND <tenant_column> = ?` to the WHERE predicate.
- **D-06:** Runtime `tenant_id` is never stored in the serializable `CrudPlan`.
- **D-07:** Defense-in-depth: executor injects tenant column from context; derive path never
  copies it from agent inputs (write-excluded).
- **D-08:** Cross-tenant / soft-deleted targets → 0 rows → existing `WriteError::RecordNotFound`
  → `error_kind: "not_found"`. No new error kinds.
- **D-09:** Authorization denial (scope-deny, Gate-deny) = explicit error. Target
  non-disclosure (cross-tenant / soft-deleted row) = opaque not-found. Keep separate.
- **D-10:** No new validation code. Add a boot-time test asserting `ServiceDef::validate()`
  rejects CRUD-verb-without-`mcp_write_ability`.

### Claude's Discretion

- Exact test-fixture layout, choice between one `write_authorized: Option<bool>` boolean vs
  an ability-keyed map (D-03, now resolved by research — see below), SQL placeholder/dialect
  details (follow existing `execute_crud_plan` dual-dialect pattern), and the precise
  `McpContext` field name.

### Deferred Ideas (OUT OF SCOPE)

- App flip, e2e drive, structured-envelope regression-guard extension, catalog/docs
  (Phase 243, already roadmapped).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRUD-05 | `create`/`update`/`delete` require `read_write` scope + `.mcp_write_ability` Gate; `tenant_id` injected server-side; cross-tenant/soft-deleted targets indistinguishable from "not found" | D-01..D-09 cover all three sub-requirements; see Architecture Patterns below for exact wiring points |
| CRUD-07 | `ServiceDef::validate()` fails fast when any CRUD verb is enabled without `mcp_write_ability` | Rule ships at `service.rs:504-510`; Phase 242 adds a boot-time test to verify it (D-10) |
</phase_requirements>

---

## Summary

Phase 242 closes the safety envelope on the CRUD write path (Phases 239-241). Three
capabilities are net-new; the rest reuses shipped infrastructure.

The scope gate (`jsonrpc.rs:77`) already enforces that a `read` key cannot call any write
tool — that half of SC#1 is done. Phase 242 adds the write-ability Gate half: a pre-evaluated
`write_authorized: Option<bool>` field on `McpContext` that `handle_write_call` checks
fail-closed before the CRUD prefix loop dispatches. The host populates this field by calling
`Gate::authorize_for(&user, ability, None)` for the service's `mcp_write_ability` ability,
mirroring exactly how the read path populates the Gate check for `mcp_ability`.

Tenant injection requires two changes: (1) `derive_crud_plan` must set `tenant_column:
Some(TenantColumn { column })` from `svc.tenant_column` instead of always `None`, and (2)
`execute_crud_plan` must bind the `_tenant_id` parameter (currently ignored) into the SQL —
appending it to the INSERT column list for Create, and adding `AND <tenant_column> = ?` to
the WHERE predicate for Update and Delete. The dual-dialect (`?` / `$N`) pattern is already
established for soft-delete; the tenant predicate follows the same pattern.

Non-disclosure follows for free: adding `AND <tenant_column> = ?` makes a cross-tenant row
produce 0 affected rows, which maps to the existing `WriteError::RecordNotFound` →
`error_kind: "not_found"` path. No new code paths or error kinds are needed.

**Primary recommendation:** Three scoped code changes, each with a single clear insertion
point. The most complex is `execute_crud_plan` (framework write kernel) — it requires
placeholder-numbering care for both SQLite and Postgres.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Scope gate (read-key rejects write) | `ferro-mcp-server` framing | — | Already shipped at `jsonrpc.rs:77`; no new code |
| Write-ability Gate evaluation | Host (consumer app `controllers/mcp.rs`) | — | Policy lives in the app; host runs `Gate::authorize_for`, stores result in `McpContext.write_authorized` |
| Write-ability Gate enforcement | `ferro-mcp-server` framing | — | `handle_write_call` checks `ctx.write_authorized` fail-closed before dispatch |
| Tenant column injection (SQL) | `framework::write` kernel (`execute_crud_plan`) | `ferro-projections` (`derive_crud_plan`) | Plan carries column name; kernel binds the runtime value |
| Tenant column derivation | `ferro-projections` (`derive_crud_plan`) | — | Reads `svc.tenant_column`, sets `TenantColumn { column }` |
| Non-disclosure | SQL predicate in `execute_crud_plan` | — | 0-row outcome → existing `RecordNotFound` path; no extra framing needed |
| CRUD-07 validate() test | `ferro-projections` test module | — | Verifies the rule that shipped in `5cb17d60` |

---

## Standard Stack

No new dependencies. All capabilities use existing crates already in the workspace.

| Crate | Current Role | Phase 242 Extension |
|-------|-------------|---------------------|
| `ferro-projections` | `CrudPlan`, `TenantColumn`, `derive_crud_plan`, `ServiceDef::validate()` | Fill `tenant_column` from `svc.tenant_column` in `derive_crud_plan`; add test for `validate()` rule |
| `framework` (`ferro-rs`) | `execute_crud_plan`, `dispatch_write`, `WriteError::RecordNotFound` | Wire `_tenant_id` into SQL; add `AND <tenant_column> = ?` predicate |
| `ferro-mcp-server` | `handle_write_call`, `McpContext`, scope gate | Add `write_authorized: Option<bool>` to `McpContext`; check it fail-closed before CRUD loop |
| App (`app/src/controllers/mcp.rs`) | Builds `McpContext`, calls `Gate::authorize_for` for read tools | Add `Gate::authorize_for` for write tools; populate `ctx.write_authorized` |

---

## Architecture Patterns

### System Architecture Diagram

```
[Agent] ──tools/call──► [host: controllers/mcp.rs]
                              │
                              │ 1. validate bearer + extract tenant_id, scope
                              │ 2. for WRITE tools: Gate::authorize_for(user, svc.mcp_write_ability)
                              │    → write_authorized: Some(true/false)
                              │ 3. build McpContext { tenant_id, scope, write_authorized }
                              ▼
                    [ferro-mcp-server: jsonrpc.rs handle_tools_call]
                              │
                              │ 4. scope gate (already shipped): read key → error
                              │ 5. route to handle_write_call (if is_write_tool)
                              ▼
                    [ferro-mcp-server: write_dispatch.rs handle_write_call]
                              │
                              │ 6. check ctx.write_authorized fail-closed (NEW D-01)
                              │    None/false → Auth error, return early
                              │ 7. CRUD prefix loop: find svc by tool name
                              │ 8. derive_crud_plan(svc, verb, args)  ← fills tenant_column (NEW D-04)
                              │ 9. dispatch_write(..., Some(&plan))
                              ▼
                    [framework::write::dispatch_write]
                              │
                              │ 10. guard re-eval (existing)
                              │ 11. idempotency check (existing)
                              │ 12. confirmation seam (existing)
                              │ 13. execute_crud_plan(plan, tenant_id, db)  ← binds tenant (NEW D-05)
                              │     Create: INSERT … (col_list + tenant_col) VALUES (… + tenant_id)
                              │     Update: UPDATE … WHERE id=? AND deleted_at IS NULL AND tenant_id=?
                              │     Delete: UPDATE … SET deleted_at=now WHERE id=? AND deleted_at IS NULL AND tenant_id=?
                              │     → 0 rows affected → WriteError::RecordNotFound (D-08 non-disclosure)
                              │ 14. idempotency store + audit (existing)
                              ▼
                    [ferro-mcp-server: write_dispatch.rs]
                              │ 15. map WriteError::RecordNotFound → error_kind:"not_found" (existing path)
                              ▼
                    [Agent receives structured envelope]
```

### Recommended Project Structure

No new files. All changes are surgical additions to existing files:

```
ferro-projections/src/executor.rs      — derive_crud_plan: fill tenant_column
ferro-projections/src/service.rs       — no code change; test module gains a validate() test
framework/src/write/mod.rs             — execute_crud_plan: bind tenant_id
ferro-mcp-server/src/renderer.rs       — McpContext: add write_authorized field
ferro-mcp-server/src/write_dispatch.rs — handle_write_call: check write_authorized
app/src/controllers/mcp.rs             — tools/call write path: run Gate + populate write_authorized
```

---

## Resolved Question: D-03 — Concrete Carrier Shape

**Question from CONTEXT.md:** Should `McpContext` carry `write_authorized: Option<bool>` or
an ability-keyed map?

**Answer (HIGH confidence, verified against `app/src/controllers/mcp.rs` and
`ferro-mcp-server/src/renderer.rs`):**

Use `write_authorized: Option<bool>`. Rationale:

1. **One write ability per call.** The CRUD prefix loop in `handle_write_call` resolves one
   `ServiceDef` per call, which has one `mcp_write_ability` string. There is no fan-out
   across multiple ability names at dispatch time.

2. **Mirror the read-path pattern.** The read-tool path (`controllers/mcp.rs:294-306`) already
   runs `Gate::authorize_for(&user, ability, None)` and maps the result to a single allow/deny
   before building `McpContext`. The host is the policy owner; `ferro-mcp-server` only
   enforces the pre-evaluated result. A boolean mirrors this pattern exactly.

3. **No Gate dependency in ferro-mcp-server.** `ferro-mcp-server` must not call the Gate live
   (`Gate` requires a full `User`, which the MCP server crate does not have). A boolean
   carrier keeps the crate free of a policy/auth dependency.

4. **Ability-keyed map is over-engineering.** There is currently no case where multiple
   write abilities need to be pre-evaluated for a single `tools/call`. Adding a map would
   create a control surface that is never exercised and duplicates the reads map
   (`evaluated_guards`) semantics.

**Concrete field:**
```rust
// ferro-mcp-server/src/renderer.rs — McpContext (lines 17-22 today)
pub struct McpContext {
    pub tenant_id: Option<i64>,
    pub evaluated_guards: HashMap<String, bool>,
    pub scope: Option<String>,
    /// Pre-evaluated write-ability authorization result.
    /// `Some(true)` = Gate passed; `Some(false)` = Gate denied; `None` = not checked
    /// (read-only tools, OAuth JWT path). Checked fail-closed in handle_write_call.
    pub write_authorized: Option<bool>,
}
```

**Host population site** (`controllers/mcp.rs`, write-tool branch, before building `McpContext`):
```rust
// After resolving the service (same lookup as the CRUD prefix loop does):
let write_authorized: Option<bool> = if is_write_tool {
    // Locate the owning service by stripping the CRUD prefix.
    // None if the service is not found or not CRUD-enabled — fail-closed.
    let svc_name = ["create_", "update_", "delete_"]
        .iter()
        .find_map(|pfx| tool_name.strip_prefix(pfx));
    match svc_name.and_then(|n| services.iter().find(|s| s.mcp_exposed && s.name == n)) {
        Some(svc) => match svc.mcp_write_ability.as_deref() {
            Some(ability) => Some(Gate::authorize_for(&user, ability, None).is_ok()),
            None => Some(false),  // fail-closed: no declared ability → deny
        },
        None => Some(false),      // no service found → deny (will 32601 in dispatch)
    }
} else {
    None
};
```

**Enforcement site** (`write_dispatch.rs:handle_write_call`, before the CRUD prefix loop):
```rust
// D-01: fail-closed write-ability Gate check. Must run BEFORE the CRUD prefix loop
// and BEFORE find_action — an unauthorized agent must not learn which tools exist.
// Scope check already ran in handle_tools_call (jsonrpc.rs:77).
if ctx.write_authorized != Some(true) {
    return json!({ "error": { "code": -32603, "message":
        crate::Error::Auth("write ability not authorized".to_string()).to_string()
    }});
}
```

**Note on the Gate principal:** `Gate::authorize_for` takes `&U: Authenticatable`. In the app
the host already resolves the concrete `User` from the DB for read-tool Gate checks
(`controllers/mcp.rs:282-290`). The same user-resolution code is reused for write tools. The
MCP principal is `tenant_id` (from the JWT `sub` claim); the corresponding `User` is fetched
by `User::find_by_id(user_id)`. The Gate receives the full `User`, same as the web surface.

---

## Tenant Injection — Exact SQL Changes

### derive_crud_plan (ferro-projections/src/executor.rs)

**Current state (lines 247, 272, 289):** all three `CrudPlan` variants hardcode `tenant_column: None`.

**Phase 242 change:** replace `None` with the actual column when `svc.tenant_column` is set:

```rust
// At the bottom of CrudVerb::Create arm (line 244):
tenant_column: svc.tenant_column.as_ref().map(|col| TenantColumn { column: col.clone() }),

// At the bottom of CrudVerb::Update arm (line 266-273):
tenant_column: svc.tenant_column.as_ref().map(|col| TenantColumn { column: col.clone() }),

// At the bottom of CrudVerb::Delete arm (line 284-290):
tenant_column: svc.tenant_column.as_ref().map(|col| TenantColumn { column: col.clone() }),
```

No other changes to `derive_crud_plan`. The `TenantColumn` struct already exists (lines 133-136).

### execute_crud_plan (framework/src/write/mod.rs)

**Current state:** all three match arms destructure `tenant_column: _` (lines 290, 373, 431),
ignoring it entirely. `_tenant_id` parameter is also ignored.

**Phase 242 changes per verb:**

**Create (lines 287-363):**
The current pattern builds `col_names` from `columns`, then appends `"created_at"`, then
builds `ph_parts` matching the column count, then appends `now_expr`.

Insert the tenant column after `created_at`:
```rust
// After the created_at push (current line ~300-301):
if let Some(ref tc) = tenant_column {
    col_names.push(tc.column.clone());
    ph_parts.push(placeholder(backend, col_names.len()));  // index after created_at
    // created_at is a literal (not a bound param), so the count for placeholder()
    // must track only bound values. Tenant_id comes after all col values.
}
// ... then build values vec, THEN push tenant_id:
if tenant_column.is_some() {
    values.push(sea_orm::Value::BigInt(Some(tenant_id)));
}
```

**Placeholder numbering detail:** `created_at` is a SQL literal expression (`now_expr`), not
a bound parameter, so it does not consume a placeholder index. The placeholder for `tenant_id`
is `placeholder(backend, columns.len() + 1)` (1-based: columns are `$1...$N`, tenant_id is
`$N+1`). For Postgres this yields `$N+1`; for SQLite it is `?`. The current code uses
`(1..=columns.len()).map(|i| placeholder(backend, i))` — just extend this by one for the
tenant column.

**Update (lines 366-424):**
Current SQL: `UPDATE {table} SET {set_sql} WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL`
Phase 242 SQL: append `AND {tenant_column} = {tenant_ph}` to the WHERE clause.

```rust
// After building the existing sql string:
let sql = if let Some(ref tc) = tenant_column {
    let tenant_ph = placeholder(backend, patch.len() + 2);  // id is patch.len()+1
    format!(
        "UPDATE {table} SET {set_sql} WHERE {id_column} = {id_ph} \
         AND {soft_delete_column} IS NULL AND {tc_col} = {tenant_ph}",
        tc_col = tc.column
    )
} else {
    format!("UPDATE {table} SET {set_sql} WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL")
};
// Append tenant_id to values vec after id_value:
if tenant_column.is_some() {
    values.push(sea_orm::Value::BigInt(Some(tenant_id)));
}
```

The post-update SELECT (lines 408-423) must also add the tenant predicate for correctness
(guard against a concurrent cross-tenant update race). Mirror the same pattern.

**Delete (lines 427-453):**
Current SQL: `UPDATE {table} SET {soft_delete_column} = {now_expr} WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL`
Phase 242 SQL: append `AND {tenant_column} = {tenant_ph}`.

```rust
// id_ph is placeholder(backend, 1); tenant_ph is placeholder(backend, 2)
let sql = if let Some(ref tc) = tenant_column {
    let tenant_ph = placeholder(backend, 2);
    format!(
        "UPDATE {table} SET {soft_delete_column} = {now_expr} \
         WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL \
         AND {tc_col} = {tenant_ph}",
        tc_col = tc.column
    )
} else {
    format!("UPDATE {table} SET {soft_delete_column} = {now_expr} WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL")
};
if tenant_column.is_some() {
    values.push(sea_orm::Value::BigInt(Some(tenant_id)));
}
```

**Non-disclosure:** When `tenant_column = Some` and the row belongs to a different tenant, the
`AND <tenant_column> = ?` predicate yields 0 rows affected → `exec_result.rows_affected() == 0`
→ `return Err(WriteError::RecordNotFound)`. This is the existing check at lines 400-402 and
447-449. No new code path.

---

## CRUD-07 Validation — Existing Rule Location

`ServiceDef::validate()` is at `ferro-projections/src/service.rs:499`. The CRUD-07 rule is at
lines 502-510:

```rust
// service.rs:502-510 (shipped in 5cb17d60)
if (self.creatable || self.updatable || self.deletable) && self.mcp_write_ability.is_none() {
    return Err(crate::Error::Validation(format!(
        "projection '{}' enables create/update/delete but declares no mcp_write_ability",
        self.name
    )));
}
```

**Existing validate() tests are at `service.rs:1106-1540`** (large block of `#[test]` functions
within `mod tests`). The Phase 242 boot-time test must mirror this pattern.

**Test pattern:**
```rust
// In ferro-projections/src/service.rs, mod tests:
#[test]
fn validate_rejects_crud_verb_without_write_ability() {
    // creatable with no mcp_write_ability → Err
    let svc = ServiceDef::new("order")
        .mcp_exposed(true)
        .creatable(true)
        .field("id", DataType::Integer, FieldMeaning::Identifier);
    let err = svc.validate().unwrap_err();
    assert!(matches!(err, crate::Error::Validation(_)));
    assert!(err.to_string().contains("mcp_write_ability"));

    // With mcp_write_ability → Ok
    let svc_ok = ServiceDef::new("order")
        .mcp_exposed(true)
        .creatable(true)
        .mcp_write_ability("manage-orders")
        .field("id", DataType::Integer, FieldMeaning::Identifier);
    assert!(svc_ok.validate().is_ok());
}
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Dual-dialect SQL placeholder | Hand-write `?`/`$N` branching | `placeholder(backend, index)` helper at `framework/src/write/mod.rs:184` | Already tested for both dialects; missed indices = data corruption |
| Tenant predicate SQL | String interpolation of tenant value | `Statement::from_sql_and_values` with bound `sea_orm::Value::BigInt(Some(tenant_id))` | All values must be bound parameters; values from agent must never reach SQL (T-241-05) |
| Cross-tenant denial | New error variant or response envelope | Add `AND <tenant_column> = ?` predicate → existing `RecordNotFound` path | A new error variant discloses that a cross-tenant row exists (D-08) |
| Write-ability check in dispatcher | Call `Gate` live inside `ferro-mcp-server` | Pre-evaluate in host, carry as `McpContext.write_authorized` | Gate requires full User; ferro-mcp-server must not pull auth/ORM deps |

**Key insight:** The entire non-disclosure mechanism is the SQL predicate. Adding `AND
tenant_column = ?` to WHERE makes a foreign-tenant row unaddressable at the DB level — no
separate denial code path is needed or should be added. Adding a separate path would itself
leak the row's existence.

---

## Common Pitfalls

### Pitfall 1: Placeholder index off-by-one for Create
**What goes wrong:** The `created_at` column is a SQL literal expression (`datetime('now')`
or `NOW()`), not a bound parameter. If the planner counts it as a placeholder slot, the tenant
column gets index `N+2` instead of `N+1`, silently binding wrong values to wrong columns.
**Why it happens:** The `ph_parts` vector and `values` vector grow independently; only
`values` drives actual parameter binding.
**How to avoid:** Count placeholder indices from `columns.len()` only (not from
`ph_parts.len()`). `created_at` is appended to `ph_parts` as a literal string, not via
`placeholder(backend, i)`.
**Warning signs:** SQLite INSERT succeeds but `created_at` contains the tenant_id value;
or Postgres returns a type mismatch error.

### Pitfall 2: write_authorized check placed after CRUD loop
**What goes wrong:** The CRUD prefix loop at `write_dispatch.rs:167-262` resolves a service
and builds a plan before the write-ability check runs. An unauthorized agent learns which
services exist.
**Why it happens:** The natural code flow is "find the service, then authorize". But
information about which services have which verbs enabled is itself the tool surface.
**How to avoid:** Check `ctx.write_authorized != Some(true)` at the TOP of `handle_write_call`,
before any service lookup or loop, immediately after `let _ = ctx;` line 118. The scope check
already enforces "can this key call ANY write tool" at `handle_tools_call` in `jsonrpc.rs:77`
— the write-ability check adds "is this key allowed to call write tools on THIS service".

### Pitfall 3: Conflating scope gate with write-ability Gate
**What goes wrong:** Treating the existing `key_scope == "read"` check as sufficient write
authorization, skipping the write-ability Gate.
**Why it happens:** The scope gate IS already enforcing that a read key cannot call write
tools. But scope only says "this key class is allowed to write at all" — not "this key has
permission for this specific projection's write ability".
**How to avoid:** Both checks are required. Scope gate = key class permission. Write-ability
Gate = projection-level policy permission. They are orthogonal.

### Pitfall 4: Storing write_authorized inside evaluated_guards
**What goes wrong:** Adding `write_authorized` as a special key in the `evaluated_guards`
HashMap instead of a dedicated field.
**Why it happens:** `evaluated_guards` is already there and holds boolean values. It looks
like a natural container.
**How to avoid:** D-02 is explicit: `evaluated_guards` is a visibility filter (which tools
appear in `tools/list`), NOT an authorization gate. Mixing them erodes the security boundary
and violates the "no duplicate control surface" convention.

### Pitfall 5: Cross-dialect post-Update SELECT missing tenant predicate
**What goes wrong:** The `UPDATE` adds the tenant predicate but the subsequent `SELECT *
FROM {table} WHERE id = ?` (lines 408-423) omits it. On a race where a concurrent soft-delete
or cross-tenant reassignment happens between UPDATE and SELECT, the SELECT returns wrong data.
**How to avoid:** Add `AND <soft_delete_column> IS NULL` (already present) AND `AND
<tenant_column> = ?` to the post-update SELECT as well. This is defense-in-depth; the UPDATE
already enforced both, but the SELECT should match.

---

## Code Examples

All examples verified against actual source code in this session.

### Current McpContext (renderer.rs:17-22)
```rust
// [VERIFIED: direct read of ferro-mcp-server/src/renderer.rs]
#[derive(Debug, Clone, Default)]
pub struct McpContext {
    pub tenant_id: Option<i64>,
    pub evaluated_guards: HashMap<String, bool>,
    pub scope: Option<String>,
    // Phase 242: add write_authorized: Option<bool> here
}
```

### Scope gate (jsonrpc.rs:75-86) — already shipped, no change needed
```rust
// [VERIFIED: direct read of ferro-mcp-server/src/jsonrpc.rs]
let is_write_tool = !tool_name.starts_with("list_");
let key_scope = ctx.scope.as_deref().unwrap_or("read_write");
if is_write_tool && key_scope == "read" {
    return json!({ "error": { "code": -32603, "message": ... } });
}
```

### Phase 242 marker comment in handle_write_call (write_dispatch.rs:423-426)
```rust
// [VERIFIED: direct read of ferro-mcp-server/src/write_dispatch.rs]
// Phase 242: the synthesized CRUD delete verb has no preconditions, so this
// loop is a correct no-op. Phase 242 wires mcp_write_ability / per-record
// guards as preconditions here; this loop is the extension point.
let crud_guards: Vec<String> = vec![]; // Phase 242 populates from svc preconditions
```
Note: Phase 242's Gate check goes BEFORE `handle_write_call` returns to this loop, not
inside it. The comment refers to the mcp_write_ability enforcement upstream.

### placeholder() helper (framework/src/write/mod.rs:184-189)
```rust
// [VERIFIED: direct read of framework/src/write/mod.rs]
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}
```

### validate() CRUD-07 rule (ferro-projections/src/service.rs:502-510)
```rust
// [VERIFIED: direct read of ferro-projections/src/service.rs]
if (self.creatable || self.updatable || self.deletable) && self.mcp_write_ability.is_none() {
    return Err(crate::Error::Validation(format!(
        "projection '{}' enables create/update/delete but declares no mcp_write_ability",
        self.name
    )));
}
```

### Read-tool Gate check (app/src/controllers/mcp.rs:292-315) — mirror for writes
```rust
// [VERIFIED: direct read of app/src/controllers/mcp.rs]
let ability = match service.mcp_ability.as_deref() {
    Some(a) => a,
    None => { return Ok(HttpResponse::json(make_tool_deny_response(...))); }
};
match ferro::authorization::Gate::authorize_for(&user, ability, None) {
    Ok(()) => {}
    Err(_) => { return Ok(HttpResponse::json(make_tool_deny_response(...))); }
}
```

### Current execute_crud_plan Create arm (framework/src/write/mod.rs:287-363) — key pattern
```rust
// [VERIFIED: direct read of framework/src/write/mod.rs]
// Phase 241: tenant_column: _ destructured and ignored
CrudPlan::Create { table, columns, tenant_column: _ } => {
    let mut col_names: Vec<String> = columns.iter().map(|(c, _)| c.clone()).collect();
    col_names.push("created_at".to_string());           // literal, not bound

    let mut ph_parts: Vec<String> = (1..=columns.len())
        .map(|i| placeholder(backend, i))
        .collect();
    ph_parts.push(now_expr.to_string());                // literal expression

    let values: Vec<sea_orm::Value> =
        columns.iter().map(|(_, v)| json_to_sea_value(v)).collect();
    // Phase 242: push tenant col to col_names + placeholder + values
}
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[test]` + `#[tokio::test]` (cargo test) |
| Config file | `Cargo.toml` (workspace member features, `[dev-dependencies]`) |
| Quick run command | `cargo test -p ferro-projections -p ferro-mcp-server -p framework --all-features 2>&1 \| tail -30` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| CRUD-05 | Scope-deny: `read`-scope key on write tool → explicit auth error | unit / framing | `cargo test -p ferro-mcp-server scope_deny` | ✅ covered by `jsonrpc.rs:77` (shipped) |
| CRUD-05 | Gate-deny: `write_authorized=Some(false)` → auth error before dispatch | unit / framing | `cargo test -p ferro-mcp-server write_ability_gate_deny` | ❌ Wave 0 (new) |
| CRUD-05 | Gate-pass: `write_authorized=Some(true)` → dispatch proceeds | unit / framing | `cargo test -p ferro-mcp-server write_ability_gate_pass` | ❌ Wave 0 (new) |
| CRUD-05 | Tenant injection Create: INSERT contains tenant_column with ctx.tenant_id value | sqlite-in-memory | `cargo test -p framework crud_create_injects_tenant` | ❌ Wave 0 (new) |
| CRUD-05 | Tenant injection Update: WHERE includes AND tenant_col = tid | sqlite-in-memory | `cargo test -p framework crud_update_tenant_predicate` | ❌ Wave 0 (new) |
| CRUD-05 | Tenant injection Delete: WHERE includes AND tenant_col = tid | sqlite-in-memory | `cargo test -p framework crud_delete_tenant_predicate` | ❌ Wave 0 (new) |
| CRUD-05 | Cross-tenant non-disclosure Update: row owned by tenant 2, tenant 1 gets not_found | sqlite-in-memory | `cargo test -p framework crud_cross_tenant_update_not_found` | ❌ Wave 0 (new) |
| CRUD-05 | Cross-tenant non-disclosure Delete: same as update | sqlite-in-memory | `cargo test -p framework crud_cross_tenant_delete_not_found` | ❌ Wave 0 (new) |
| CRUD-05 | Soft-deleted non-disclosure: deleted row → same not_found envelope (existing test extended) | sqlite-in-memory | `cargo test -p framework crud_update_soft_deleted_not_found` | ✅ shipped (`framework/src/write/mod.rs:1420`) |
| CRUD-05 | Tenant column absent from create/update schema (is_server_injected_field) | unit | `cargo test -p ferro-projections is_server_injected` | ✅ shipped (Phase 239/240) |
| CRUD-07 | validate() rejects CRUD verb without mcp_write_ability | unit | `cargo test -p ferro-projections validate_rejects_crud_verb_without_write_ability` | ❌ Wave 0 (new, verifying shipped rule) |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-projections -p ferro-mcp-server -p framework --all-features 2>&1 | tail -30`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps (tests that must be written before or alongside implementation)
- [ ] `ferro-projections/src/executor.rs` — `validate_rejects_crud_verb_without_write_ability`
- [ ] `framework/src/write/mod.rs` — `crud_create_injects_tenant`, `crud_update_tenant_predicate`, `crud_delete_tenant_predicate`, `crud_cross_tenant_update_not_found`, `crud_cross_tenant_delete_not_found`
- [ ] `ferro-mcp-server/src/write_dispatch.rs` — `write_ability_gate_deny`, `write_ability_gate_pass`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V4 Access Control | yes | `write_authorized` field + fail-closed enforcement; scope gate; Gate::authorize_for |
| V5 Input Validation | yes | Tenant column excluded from agent inputs via `is_server_injected_field` / `is_write_excluded_field` |
| V2 Authentication | yes (existing) | Bearer validation + `tenant_id` from JWT; not changed in Phase 242 |
| V3 Session Management | no | MCP is stateless bearer |
| V6 Cryptography | no | No new crypto |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Tenant escalation: agent supplies `tenant_id` in create input | Elevation of privilege | `is_server_injected_field` excludes tenant column from write schemas (Phase 239, shipped) |
| Cross-tenant update: agent supplies valid id from another tenant | Elevation of privilege | `AND <tenant_column> = ?` predicate → 0 rows → not_found (D-08) |
| Cross-tenant info disclosure via distinct error | Information disclosure | D-09: both "not found" and "wrong tenant" return identical `error_kind: "not_found"` envelope |
| Gate bypass: agent calls write tool after scope passes | Elevation of privilege | D-01: fail-closed `write_authorized` check before dispatch; None → deny |
| Guard bypass: agent calls write tool skipping tools/list visibility filter | Elevation of privilege | Existing: live guard re-eval in `dispatch_write` (not `evaluated_guards`) |

---

## Open Questions

None. All CONTEXT.md open questions (D-03) are resolved by this research.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies; all changes are code/SQL in Rust workspace)

---

## Assumptions Log

All claims in this research were verified by direct source reading in this session.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | (none) | — | — |

**All claims verified:** no user confirmation needed before planning.

---

## Sources

### Primary (HIGH confidence)
- `ferro-projections/src/executor.rs` — full file read; `CrudPlan`, `TenantColumn`, `derive_crud_plan` (lines 130-293), all test fixtures
- `ferro-projections/src/service.rs` — read lines 1-300 + grep; `ServiceDef`, `validate()` (lines 499-510), `is_server_injected_field`, `is_write_excluded_field`, `tenant_column`, `mcp_write_ability`
- `framework/src/write/mod.rs` — full file read; `execute_crud_plan` (lines 272-455), `dispatch_write` (lines 600-743), `placeholder()` helper (lines 184-189), `WriteError::RecordNotFound`, all existing test patterns
- `ferro-mcp-server/src/renderer.rs` — full file read; `McpContext` struct (lines 17-22), visibility-not-auth warning (line 210)
- `ferro-mcp-server/src/jsonrpc.rs` — full file read; scope gate (lines 75-86), `handle_tools_call` flow
- `ferro-mcp-server/src/write_dispatch.rs` — full file read; `handle_write_call` CRUD loop (lines 157-262), Phase 242 marker (lines 423-426), `WriteError::RecordNotFound` mapping (lines 240-244)
- `app/src/controllers/mcp.rs` — read lines 200-349; host `McpContext` construction, `Gate::authorize_for` for read tools, `write_authorized` gap
- `framework/src/authorization/gate.rs` — full file read; `Gate::authorize_for`, `Authenticatable`, gate registry

### Secondary
- `.planning/phases/242-write-authorization-tenant-injection-non-disclosure/242-CONTEXT.md` — decisions D-01..D-10

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new crates; verified against actual imports
- Architecture: HIGH — all wiring points verified against source; no assumed line numbers
- Pitfalls: HIGH — derived from actual code patterns and the explicit D-02 warning in renderer.rs
- SQL patterns: HIGH — `placeholder()`, `json_to_sea_value()`, and dual-dialect patterns all read directly

**Research date:** 2026-06-24
**Valid until:** 2026-07-24 (stable codebase; no external ecosystem changes)
