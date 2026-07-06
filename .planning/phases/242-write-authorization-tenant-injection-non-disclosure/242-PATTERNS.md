# Phase 242: Write Authorization, Tenant Injection & Non-Disclosure — Pattern Map

**Mapped:** 2026-06-24
**Files analyzed:** 6 (all modifications to existing files; no new files)
**Analogs found:** 6 / 6

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `ferro-projections/src/executor.rs` | schema-only derivation | transform | existing `CrudVerb::Update` / `CrudVerb::Delete` arms within same file | exact |
| `framework/src/write/mod.rs` | framework kernel, SQL | CRUD | existing soft-delete predicate + `placeholder()` already in same file | exact |
| `ferro-mcp-server/src/renderer.rs` | channel adapter, struct | request-response | existing `tenant_id: Option<i64>` and `scope: Option<String>` fields in same struct | exact |
| `ferro-mcp-server/src/write_dispatch.rs` | channel adapter, dispatch | request-response | existing `tenant_id` fail-closed check (lines 173-179) + jsonrpc.rs scope gate | exact |
| `ferro-projections/src/service.rs` (test only) | test | unit | existing `validate_catches_*` test functions in same `mod tests` block | exact |
| `app/src/controllers/mcp.rs` | host app, controller | request-response | existing `Gate::authorize_for` read-tool block (lines 265-316) | exact |

---

## Pattern Assignments

### `ferro-projections/src/executor.rs` — fill `tenant_column` from `svc.tenant_column`

**Analog:** lines 244–291 of the same file — the three `CrudPlan` variant construction sites.

**Current pattern (the three `tenant_column: None` sites to replace):**

Lines 244–248 (Create arm):
```rust
Ok(CrudPlan::Create {
    table,
    columns,
    tenant_column: None,
})
```

Lines 266–273 (Update arm):
```rust
Ok(CrudPlan::Update {
    table,
    id_column: "id".into(),
    id_value,
    patch,
    soft_delete_column: svc.resolved_soft_delete_column().to_string(),
    tenant_column: None,
})
```

Lines 284–290 (Delete arm):
```rust
Ok(CrudPlan::Delete {
    table,
    id_column: "id".into(),
    id_value,
    soft_delete_column: svc.resolved_soft_delete_column().to_string(),
    tenant_column: None,
})
```

**Pattern to apply — replace all three `tenant_column: None` with:**
```rust
tenant_column: svc.tenant_column.as_ref().map(|col| TenantColumn { column: col.clone() }),
```

The field access `svc.tenant_column` mirrors exactly how `svc.resolved_soft_delete_column()` and `svc.state_machine` are read in adjacent lines. The `TenantColumn` struct is defined at lines 133–136 of the same file:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TenantColumn {
    pub column: String,
}
```

**Test analog** (lines 518–519, 591, 614): the existing `assert_eq!(tenant_column, &None)` assertions in `derive_crud_plan_create`, `derive_crud_plan_update`, `derive_crud_plan_delete` become the counter-assertions for the `Some` case in new tests.

---

### `framework/src/write/mod.rs` — bind `tenant_id` in `execute_crud_plan`

**Analog A — `placeholder()` helper (lines 184–189, same file):**
```rust
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}
```
Every tenant placeholder must go through this helper. Never write `?` or `$N` by hand.

**Analog B — existing soft-delete predicate in the Update arm (lines 380–395, same file):**
```rust
let set_clauses: Vec<String> = patch
    .iter()
    .enumerate()
    .map(|(i, (col, _))| format!("{col} = {}", placeholder(backend, i + 1)))
    .collect();
let set_sql = set_clauses.join(", ");
let id_ph = placeholder(backend, patch.len() + 1);
let sql = format!(
    "UPDATE {table} SET {set_sql} WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL"
);
let mut values: Vec<sea_orm::Value> =
    patch.iter().map(|(_, v)| json_to_sea_value(v)).collect();
values.push(json_to_sea_value(id_value));
```

**Analog C — existing soft-delete check (0 rows → RecordNotFound) at lines 400–403:**
```rust
if exec_result.rows_affected() == 0 {
    return Err(WriteError::RecordNotFound);
}
```
The tenant predicate reuses this exact path for non-disclosure (D-08). No new error variant.

**Analog D — post-UPDATE SELECT guard (lines 408–412):**
```rust
let id_ph2 = placeholder(backend, 1);
let select_sql = format!(
    "SELECT * FROM {table} WHERE {id_column} = {id_ph2} AND {soft_delete_column} IS NULL"
);
```
The tenant predicate must be added to this SELECT as well (Pitfall 5).

**Pattern to apply for each CrudPlan arm:**

**Create arm** — change destructuring and extend column/placeholder/value lists:
```rust
CrudPlan::Create {
    table,
    columns,
    tenant_column,    // was: tenant_column: _
} => {
    let mut col_names: Vec<String> = columns.iter().map(|(c, _)| c.clone()).collect();
    col_names.push("created_at".to_string());
    // Phase 242: tenant column after created_at in col_names
    if let Some(ref tc) = tenant_column {
        col_names.push(tc.column.clone());
    }

    let mut ph_parts: Vec<String> = (1..=columns.len())
        .map(|i| placeholder(backend, i))
        .collect();
    ph_parts.push(now_expr.to_string()); // literal, NOT a bound param — does NOT consume a slot
    // Phase 242: tenant placeholder index = columns.len() + 1
    // (created_at is a literal above, so it does not shift the bound-value index)
    if tenant_column.is_some() {
        ph_parts.push(placeholder(backend, columns.len() + 1));
    }

    let mut values: Vec<sea_orm::Value> =
        columns.iter().map(|(_, v)| json_to_sea_value(v)).collect();
    // Phase 242: tenant_id comes last in bound values
    if tenant_column.is_some() {
        values.push(sea_orm::Value::BigInt(Some(tenant_id)));  // tenant_id param, not _tenant_id
    }
    // ... rest of match arm unchanged
```

**CRITICAL placeholder numbering:** `created_at` is pushed to `ph_parts` as `now_expr.to_string()` — a SQL literal, not a bound parameter. It does not consume an index slot. The tenant column bound placeholder is therefore `placeholder(backend, columns.len() + 1)`, not `columns.len() + 2`.

**Update arm** — change destructuring, extend WHERE clause and values:
```rust
CrudPlan::Update {
    table,
    id_column,
    id_value,
    patch,
    soft_delete_column,
    tenant_column,    // was: tenant_column: _
} => {
    // ... existing set_clauses + id_ph ...
    let sql = if let Some(ref tc) = tenant_column {
        let tenant_ph = placeholder(backend, patch.len() + 2); // id = patch.len()+1, tenant = patch.len()+2
        format!(
            "UPDATE {table} SET {set_sql} WHERE {id_column} = {id_ph} \
             AND {soft_delete_column} IS NULL AND {tc_col} = {tenant_ph}",
            tc_col = tc.column
        )
    } else {
        format!("UPDATE {table} SET {set_sql} WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL")
    };

    let mut values: Vec<sea_orm::Value> =
        patch.iter().map(|(_, v)| json_to_sea_value(v)).collect();
    values.push(json_to_sea_value(id_value));
    // Phase 242: tenant_id after id_value
    if tenant_column.is_some() {
        values.push(sea_orm::Value::BigInt(Some(tenant_id)));
    }
    // ... exec + rows_affected() == 0 check (unchanged) ...

    // Post-UPDATE SELECT — also add tenant predicate (Pitfall 5):
    let id_ph2 = placeholder(backend, 1);
    let select_sql = if let Some(ref tc) = tenant_column {
        let t_ph2 = placeholder(backend, 2);
        format!(
            "SELECT * FROM {table} WHERE {id_column} = {id_ph2} \
             AND {soft_delete_column} IS NULL AND {tc_col} = {t_ph2}",
            tc_col = tc.column
        )
    } else {
        format!("SELECT * FROM {table} WHERE {id_column} = {id_ph2} AND {soft_delete_column} IS NULL")
    };
    let mut select_values = vec![json_to_sea_value(id_value)];
    if tenant_column.is_some() {
        select_values.push(sea_orm::Value::BigInt(Some(tenant_id)));
    }
```

**Delete arm** — change destructuring, extend WHERE clause and values:
```rust
CrudPlan::Delete {
    table,
    id_column,
    id_value,
    soft_delete_column,
    tenant_column,    // was: tenant_column: _
} => {
    let id_ph = placeholder(backend, 1);
    let sql = if let Some(ref tc) = tenant_column {
        let tenant_ph = placeholder(backend, 2);
        format!(
            "UPDATE {table} SET {soft_delete_column} = {now_expr} \
             WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL \
             AND {tc_col} = {tenant_ph}",
            tc_col = tc.column
        )
    } else {
        format!(
            "UPDATE {table} SET {soft_delete_column} = {now_expr} \
             WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL"
        )
    };
    let mut stmt_values = vec![json_to_sea_value(id_value)];
    if tenant_column.is_some() {
        stmt_values.push(sea_orm::Value::BigInt(Some(tenant_id)));
    }
    // ... rows_affected() == 0 → RecordNotFound (unchanged) ...
```

**Function signature change required:** rename `_tenant_id` to `tenant_id` in `execute_crud_plan` (line 274):
```rust
// Before:
async fn execute_crud_plan(plan: &CrudPlan, _tenant_id: i64, db: &DatabaseConnection) -> WriteResult<Value>
// After:
async fn execute_crud_plan(plan: &CrudPlan, tenant_id: i64, db: &DatabaseConnection) -> WriteResult<Value>
```

---

### `ferro-mcp-server/src/renderer.rs` — add `write_authorized` to `McpContext`

**Analog:** existing fields in `McpContext` at lines 17–22:
```rust
#[derive(Debug, Clone, Default)]
pub struct McpContext {
    pub tenant_id: Option<i64>,
    pub evaluated_guards: HashMap<String, bool>,
    pub scope: Option<String>,
}
```

**Pattern to apply — add one field after `scope`:**
```rust
#[derive(Debug, Clone, Default)]
pub struct McpContext {
    pub tenant_id: Option<i64>,
    pub evaluated_guards: HashMap<String, bool>,
    pub scope: Option<String>,
    /// Pre-evaluated write-ability authorization result.
    /// `Some(true)` = Gate passed; `Some(false)` = Gate denied; `None` = not evaluated
    /// (read-only tools, OAuth JWT path). Checked fail-closed in `handle_write_call`.
    /// NOT stored in `evaluated_guards` — that map is a visibility filter (see line 210).
    pub write_authorized: Option<bool>,
}
```

`#[derive(Default)]` already on the struct, so new field defaulting to `None` costs nothing. All existing construction sites using `..Default::default()` or `McpContext::default()` automatically get `write_authorized: None` — no update needed at those sites.

**Construction sites that must explicitly set `write_authorized`:** only `app/src/controllers/mcp.rs` (the host). All test sites using `McpContext::default()` in renderer tests are unaffected.

---

### `ferro-mcp-server/src/write_dispatch.rs` — fail-closed Gate check in `handle_write_call`

**Analog A — existing tenant_id fail-closed check (lines 173–179, same file):**
```rust
// Fail-closed: writes require an authenticated tenant.
let tid = match tenant_id {
    Some(t) => t,
    None => {
        return json!({ "error": { "code": -32603, "message": "auth: tenant required" } });
    }
};
```
Mirror this pattern — same early-return style, same `json!` macro envelope.

**Analog B — scope gate in `jsonrpc.rs` lines 75–86:**
```rust
let is_write_tool = !tool_name.starts_with("list_");
let key_scope = ctx.scope.as_deref().unwrap_or("read_write");
if is_write_tool && key_scope == "read" {
    return json!({
        "error": {
            "code": -32603,
            "message": crate::Error::Auth(
                "scope insufficient: read key cannot call write tools".to_string()
            ).to_string()
        }
    });
}
```
The write-ability Gate check is the SECOND auth layer after the scope gate. Same error code (-32603), same envelope shape. Location: top of `handle_write_call`, after `let _ = ctx;` at line 118, before the CRUD prefix loop at line 167.

**Pattern to apply — insert after line 118 (`let _ = ctx;`):**
```rust
// D-01: fail-closed write-ability Gate check.
// Must precede every service lookup — an unauthorized caller must not learn which
// services or verbs exist. Scope check already ran in handle_tools_call (jsonrpc.rs:77).
if ctx.write_authorized != Some(true) {
    return json!({
        "error": {
            "code": -32603,
            "message": crate::Error::Auth("write ability not authorized".to_string()).to_string()
        }
    });
}
```

**Remove the `let _ = ctx;` line** — `ctx` is now read, not suppressed.

**Error enum reference:** `crate::Error::Auth(String)` — same variant used in the scope gate at jsonrpc.rs line 82. Verify `Error::Auth` exists in `ferro-mcp-server/src/lib.rs` error enum.

---

### `ferro-projections/src/service.rs` — CRUD-07 boot-time test (test module only)

**Analog:** validate test block at lines 1105–1172, same file. Exact test shape to mirror:

```rust
#[test]
fn validate_catches_undefined_action_precondition() {
    let service = ServiceDef::new("order")
        .guard(GuardDef::new("has_items"))
        .action(ActionDef::new("submit").precondition("nonexistent_guard"));

    let result = service.validate();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("nonexistent_guard"));
    assert!(err.contains("submit"));
}
```

**Pattern to apply — add to `mod tests` in `service.rs`:**
```rust
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

    // updatable with no mcp_write_ability → Err
    let svc_u = ServiceDef::new("order")
        .mcp_exposed(true)
        .updatable(true)
        .field("id", DataType::Integer, FieldMeaning::Identifier);
    assert!(svc_u.validate().is_err());

    // deletable with no mcp_write_ability → Err
    let svc_d = ServiceDef::new("order")
        .mcp_exposed(true)
        .deletable(true)
        .field("id", DataType::Integer, FieldMeaning::Identifier);
    assert!(svc_d.validate().is_err());

    // With mcp_write_ability → Ok
    let svc_ok = ServiceDef::new("order")
        .mcp_exposed(true)
        .creatable(true)
        .mcp_write_ability("manage-orders")
        .field("id", DataType::Integer, FieldMeaning::Identifier);
    assert!(svc_ok.validate().is_ok());
}
```

The rule being tested is at lines 502–510 of `service.rs` — no new code required there.

---

### `app/src/controllers/mcp.rs` — populate `write_authorized` via `Gate::authorize_for`

**Analog:** the read-tool Gate block at lines 265–316 of the same file:
```rust
if let Some(service_name) = tool_name.strip_prefix("list_") {
    let service = match services.iter().find(|s| s.name == service_name && s.mcp_exposed) {
        Some(s) => s,
        None => { return Ok(HttpResponse::json(...)); }
    };

    let user = crate::models::users::User::find_by_id(user_id).await...?;

    let ability = match service.mcp_ability.as_deref() {
        Some(a) => a,
        None => { return Ok(HttpResponse::json(make_tool_deny_response(...))); }
    };

    match ferro::authorization::Gate::authorize_for(&user, ability, None) {
        Ok(()) => {}
        Err(_) => { return Ok(HttpResponse::json(make_tool_deny_response(...))); }
    }
}
```

**McpContext construction site (lines 320–325)** — the site to extend:
```rust
// Current:
let ctx = McpContext {
    tenant_id,
    scope: key_scope,
    ..Default::default()
};

// Phase 242: add write_authorized field
let ctx = McpContext {
    tenant_id,
    scope: key_scope,
    write_authorized: write_authorized_for_tool(tool_name, &services, user_id, &id).await?,
    ..Default::default()
};
```

**Pattern for the write-ability evaluation** — insert as a helper or inline before `McpContext` construction:
```rust
// Mirrors the read-tool Gate pattern but for write verbs (create_/update_/delete_).
// Returns Some(true) if Gate passes, Some(false) if denied/absent, None if not a write tool.
let write_authorized: Option<bool> = if !tool_name.starts_with("list_") {
    // Locate the owning service by stripping the CRUD prefix.
    let svc_name = ["create_", "update_", "delete_"]
        .iter()
        .find_map(|pfx| tool_name.strip_prefix(pfx));
    match svc_name.and_then(|n| services.iter().find(|s| s.mcp_exposed && s.name == n)) {
        Some(svc) => {
            // User resolution mirrors the read-tool path (Pitfall 7: Gate needs full User).
            let user = crate::models::users::User::find_by_id(user_id).await...?;
            match svc.mcp_write_ability.as_deref() {
                Some(ability) => {
                    Some(ferro::authorization::Gate::authorize_for(&user, ability, None).is_ok())
                }
                None => Some(false), // fail-closed: no declared ability → deny
            }
        }
        None => Some(false), // no service found → deny (will -32601 in dispatch)
    }
} else {
    None // read tools: write_authorized field unused
};
```

**Important:** the comment at lines 257–264 documents why write tools previously skipped Gate. Phase 242 fills that gap. Remove or update that comment block when wiring this.

---

## Shared Patterns

### Dual-dialect SQL placeholder
**Source:** `framework/src/write/mod.rs` lines 184–189
**Apply to:** all new WHERE clause and INSERT column additions in `execute_crud_plan`
```rust
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}
```
Never write `?` or `$N` by hand. Every new parameter needs its index tracked against the bound-values vec, not the ph_parts vec (created_at is a literal and does not consume an index).

### Fail-closed early return
**Source:** `ferro-mcp-server/src/write_dispatch.rs` lines 173–179 (tenant_id check)
**Apply to:** the new `write_authorized` check in `handle_write_call`
```rust
let tid = match tenant_id {
    Some(t) => t,
    None => {
        return json!({ "error": { "code": -32603, "message": "auth: tenant required" } });
    }
};
```
Same envelope shape: top-level `error.code` + `error.message`. Not wrapped in `"result":`.

### RecordNotFound non-disclosure
**Source:** `framework/src/write/mod.rs` lines 400–403 and 447–449
**Apply to:** no new code — the tenant predicate reuses this existing check. Cross-tenant rows produce 0 affected rows and fall through to the same `Err(WriteError::RecordNotFound)` path. Do not add a new check; just extend the WHERE clause.

### Gate::authorize_for with full User
**Source:** `app/src/controllers/mcp.rs` lines 282–314
**Apply to:** write-ability Gate evaluation (also in `app/src/controllers/mcp.rs`)
```rust
let user = crate::models::users::User::find_by_id(user_id)
    .await
    .map_err(|e| HttpResponse::json(json!({ "jsonrpc": "2.0", "id": id.clone(), "error": { "code": -32603, "message": e.to_string() } })))?
    .ok_or_else(|| HttpResponse::new().status(401))?;
match ferro::authorization::Gate::authorize_for(&user, ability, None) {
    Ok(()) => {}
    Err(_) => { ... }
}
```
Gate requires a full `User`, not just `tenant_id`. Reuse the same user-resolution call already present for read tools.

---

## No Analog Found

None. All six modified files have exact analogs within the same files or immediately adjacent files in the same crate.

---

## Metadata

**Analog search scope:** `ferro-projections/src/`, `framework/src/write/`, `ferro-mcp-server/src/`, `app/src/controllers/`
**Files scanned:** 6 source files read in full
**Pattern extraction date:** 2026-06-24
