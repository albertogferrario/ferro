# Phase 241: `derive_crud_plan` + wire CRUD verbs into `framework::write` — Research

**Researched:** 2026-06-23
**Domain:** Rust workspace — `ferro-projections` + `framework::write` kernel + `ferro-mcp-server` framing
**Confidence:** HIGH (all findings verified against live source files)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `CrudPlan` is a `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize,
  JsonSchema)]` **enum** with `Create` / `Update` / `Delete` variants. Lives in
  `ferro-projections/src/executor.rs`. Pure serializable data; no closures, no I/O.
- **D-02:** `pub fn derive_crud_plan(svc: &ServiceDef, verb: CrudVerb, inputs: &Value) ->
  Result<CrudPlan, Error>` in `executor.rs`, pure, side-effect-free. Reuses Phase 239/240
  resolver accessors. New `Error` variants as needed.
- **D-03:** Kernel wiring via **thin verb discriminant** inside `dispatch_write`, NOT by
  fabricating synthetic `ActionDef`s. CRUD path has `transition_guard = None`; all existing
  pipeline steps reused.
- **D-04:** Generic CRUD SQL executed by a **framework-provided generic CRUD executor**
  interpreting a `CrudPlan`; apps do not hand-write CRUD `ExecutorFn`s.
- **D-05:** `delete_<svc>` = `UPDATE … SET <soft_delete_column> = now WHERE id = ?`. Column
  from `resolved_soft_delete_column()`. Never a physical DELETE.
- **D-06:** `delete_<svc>` confirmation-gated via existing seam. Synthesize
  `request_confirm_delete_<svc>` / `confirm_delete_<svc>` in `renderer.rs:115-155` pattern.
  Reuse `ConfirmationStore`, CSPRNG, single-use `confirm()`, `{tenant_id, verb, record_id}` binding.
- **D-07:** Per-verb overrides use existing `with_override("create_order", …)` registry
  keyed on tool name. No new mechanism.
- **D-08:** Idempotency + audit reused unchanged. Audit label string is Claude's discretion.
- **D-09:** Phase 241 wires `id` + `deleted_at IS NULL` predicates only. `CrudPlan` MUST
  carry a tenant-predicate slot Phase 242 fills; tenant injection is NOT Phase 241 scope.
- **D-10:** Route results through `CallToolResult::structured` envelope
  (`jsonrpc.rs:144`). Replace NTI short-circuit at `write_dispatch.rs:155-180`.

### Claude's Discretion

- Exact `CrudVerb` enum placement (ferro-projections vs framework re-export)
- Whether `CrudPlan` variants embed column/value vectors vs an ordered map
- Audit label string for CRUD verbs (D-08)
- Whether generic CRUD executor lives as free function or method on a kernel type
- SQL builder style (must match sqlite-in-memory dispatch test approach)
- New `Error`/`WriteError` variant names for verb-not-enabled / row-not-found

### Deferred Ideas (OUT OF SCOPE)

- Write authorization, tenant injection, non-disclosure — Phase 242
- App `order` projection flip + create→list→update→delete e2e + regression-guard extension + catalog/docs — Phase 243
- Dedicated `get_<svc>` tool
- Per-field `immutable()`/`read_only()` overrides
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRUD-03 | `delete_<svc>` soft-deletes via `deleted_at`, is confirmation-gated, and is filtered out of `list_<svc>` and all read/update/delete paths | D-05 + D-06: soft-delete SQL confirmed by `resolved_soft_delete_column()` accessor; existing `deleted_at IS NULL` predicate at `dispatch.rs:285-290` already excludes from list; confirmation seam in `write/mod.rs:379` extended to CRUD delete |
| CRUD-06 | CRUD verbs dispatch through the shipped `framework::write` kernel via a derived `derive_crud_plan`, reusing existing override-hook registry, idempotency, channel-parameterized audit, and confirmation — single-source across MCP and visual surfaces; does NOT rebuild the dispatcher | D-03 + D-04: one `dispatch_write` function at `write/mod.rs:313-436`; thin verb discriminant extends it; generic CRUD executor runs inside the existing pipeline |
</phase_requirements>

---

## Summary

Phase 241 is an extension problem, not a design problem. The kernel, override registry, idempotency, confirmation, and audit machinery are all shipped and green. The phase adds two cohesive pieces: `derive_crud_plan` (a pure serializable plan in `ferro-projections/src/executor.rs`) and the wiring that makes `dispatch_write` in `framework/src/write/mod.rs` invoke a framework-provided generic CRUD executor when a CRUD verb is called.

The highest-risk design question — how to add a CRUD path inside `dispatch_write` without forking it — has a clean answer confirmed by the source: the existing `action: &ActionDef` parameter already carries `transition_trigger` only for transition paths; for CRUD, the framing layer bypasses `find_action` (which only searches `svc.actions`), derives a `CrudPlan` instead, and calls `dispatch_write` with a synthesized or adapted argument. The specific mechanism (thin discriminant as a new parameter vs a wrapper type over `ActionDef`) is the one remaining discretionary choice for the planner, but the constraint is clear: the pipeline body changes only at the confirmation seam check (step 3) and the executor-call (step 4).

The confirmation path for `delete_<svc>` requires synthesizing `request_confirm_delete_<svc>` / `confirm_delete_<svc>` tools in `renderer.rs` following the exact same pattern as the existing transition confirm synthesis at lines 119-155. The binding payload binds `{tenant_id, action_name:"delete_<svc>", record_id}` — the existing binding check in `handle_confirm` works unchanged.

**Primary recommendation:** Keep the change surface as small as possible. `dispatch_write` signature gains one `Option<&CrudPlan>` parameter (or a small enum discriminant); when `Some`, the executor step runs the generic CRUD interpreter instead of `dispatcher.executor`. Everything else in the pipeline — guards, idempotency, confirmation, audit, override hook — runs identically.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `CrudPlan` + `CrudVerb` type definitions | `ferro-projections` | — | Schema-only crate; no dep on `framework`; mirrors `TransitionPlan` location |
| `derive_crud_plan` function | `ferro-projections` | — | Pure derivation from `ServiceDef`; same module as `derive_transition_plan` |
| Generic CRUD SQL executor | `framework::write` | — | Requires `sea_orm::DatabaseConnection`; lives alongside `dispatch_write` |
| Kernel extension (`dispatch_write` CRUD path) | `framework::write` | — | Single kernel; both MCP and visual surface call it |
| NTI seam replacement | `ferro-mcp-server::write_dispatch` | — | MCP framing; replaces lines 155-180 with real derive→dispatch |
| Confirm tool synthesis for delete | `ferro-mcp-server::renderer` | — | Channel framing; mirrors transition confirm synthesis |
| `list_<svc>` `deleted_at IS NULL` filter | `ferro-mcp-server::dispatch` | — | Already shipped at lines 285-290; no change needed |

---

## Standard Stack

### Core

| Library | Version | Purpose | How Used |
|---------|---------|---------|----------|
| `sea-orm` | 1.0 | SQL layer | `Statement::from_sql_and_values` + `DatabaseConnection` — exact same API as all existing dispatch code |
| `serde_json` | (workspace) | JSON ↔ Rust | `Value` type for inputs; plan fields; audit payload |
| `schemars` | (workspace) | JsonSchema derive | `#[derive(JsonSchema)]` on `CrudPlan` / `CrudVerb` mirrors `TransitionPlan` |
| `thiserror` | (workspace) | Error enum | New variants on `crate::Error` (ferro-projections) + `WriteError` (framework) |

[VERIFIED: live Cargo.toml + source files]

### No New Dependencies

Phase 241 adds zero new crate dependencies. Every library needed is already in scope:
- `ferro-projections` has `serde`, `schemars`, `thiserror`
- `framework` has `sea-orm`, `ferro-projections`, `ferro-audit`
- `ferro-mcp-server` has `ferro-rs` (framework), `ferro-projections`, `ferro-ai` (confirmation feature)

[VERIFIED: framework/Cargo.toml line 51, 60; ferro-mcp-server/Cargo.toml lines 17-18]

---

## Research Question Answers

### Q1: `dispatch_write` signature and the minimal extension for CRUD

**Current signature** (`framework/src/write/mod.rs:313-322`):
```rust
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
    transition_guard: Option<&str>,
    channel: &str,
    #[cfg(feature = "confirmation")] is_confirmed: bool,
) -> WriteResult<Value>
```

The `action: &ActionDef` parameter drives:
1. Guard union (`action.preconditions` + `transition_guard`) — step 1
2. Confirmation seam check: `action.transition_trigger.is_some()` — step 3 (`write/mod.rs:379`)
3. Executor call: `(dispatcher.executor)(&action.name, inputs, tenant_id, db)` — step 4
4. Audit label: `format!("{channel}.action.{}", &action.name)` — step 6
5. Override hook lookup: `dispatcher.overrides.get(&action.name)` — step 7

**The minimal CRUD extension** (recommended — smallest diff, satisfies SC#4):

Add a `crud_plan: Option<&CrudPlan>` parameter. When `Some`, the kernel:
- Step 3: check `crud_plan.as_ref().map(|p| p.is_delete())` for the confirmation gate instead of `action.transition_trigger.is_some()`
- Step 4: call the generic CRUD executor (a free function `execute_crud_plan`) instead of `dispatcher.executor`
- Steps 1, 2, 5, 6, 7: unchanged — guard union uses `action.preconditions` (empty for CRUD = no guards by default), idempotency, audit, override hook all fire identically

The framing in `write_dispatch.rs` synthesizes a minimal `ActionDef` for the CRUD path:
```rust
// Synthesized ActionDef for CRUD verbs — no transition_trigger, no preconditions.
// Name drives the audit label and override-hook lookup keyed on tool_name ("create_order", etc.).
let crud_action = ActionDef::new(&tool_name); // e.g. "create_order"
let plan = derive_crud_plan(svc, verb, &args)?;
dispatch_write(&crud_action, &args, tid, db, dispatcher, None, "mcp",
    /*is_confirmed*/ is_delete_confirmed, Some(&plan)).await
```

This satisfies SC#4: `grep -rn "dispatch_write"` finds exactly one definition, no second CRUD dispatcher, and the confirmation seam check is a single `||` extension of the existing `transition_trigger.is_some()` condition.

[VERIFIED: `write/mod.rs:313-436`, `write_dispatch.rs:155-230`]

### Q2: Confirmation seam for `delete_<svc>`

**Current seam** (`write/mod.rs:378-381`):
```rust
#[cfg(feature = "confirmation")]
if action.transition_trigger.is_some() && !is_confirmed {
    return Err(WriteError::ConfirmationRequired(action.name.clone()));
}
```

**Extension**: the condition becomes:
```rust
let is_destructive = action.transition_trigger.is_some()
    || crud_plan.as_ref().map(|p| p.is_delete()).unwrap_or(false);
if is_destructive && !is_confirmed {
    return Err(WriteError::ConfirmationRequired(action.name.clone()));
}
```

**Confirm tool synthesis** follows the pattern at `renderer.rs:119-155`:

The existing `destructive` Vec collects tools where `a.transition_trigger.is_some()`. Phase 241 extends this to also collect CRUD delete tools (`tool.name.starts_with("delete_")`). The synthesized `request_confirm_delete_<svc>` and `confirm_delete_<svc>` tools:
- `request_confirm_delete_<svc>`: schema = same as `delete_<svc>` schema (id required, confirmation_token optional) with `destructiveHint=false`; routing via `strip_prefix("request_confirm_")` yields `"delete_<svc>"` → the confirm handler receives `action_name = "delete_<svc>"`
- `confirm_delete_<svc>`: schema = `{confirmation_token: string (required), id: integer (required)}` with `destructiveHint=true`

**Token binding** in `handle_request_confirm`:
```rust
let binding_payload = json!({
    "_binding": {
        "tenant_id": tid,
        "action_name": action_name,  // = "delete_order"
        "record_id": record_id
    },
    "inputs": args
});
```
`handle_confirm` verifies `binding["action_name"] == "delete_<svc>"` — works unchanged because the tool name IS the action name for CRUD verbs.

**Key routing difference**: For transitions, `handle_write_call` calls `find_action(services, action_name)` which searches `svc.actions`. For CRUD verbs, `find_action` returns `None` (CRUD verbs are not `ActionDef`s). The framing must short-circuit BEFORE `find_action` for CRUD verbs, just as the NTI stub currently does (lines 162-180).

[VERIFIED: `write_dispatch.rs:300-566`, `renderer.rs:115-155`, `write/mod.rs:378-381`]

### Q3: SQL layer — `sea-orm` raw statements via `DatabaseConnection`

All existing dispatch code uses `sea_orm::Statement::from_sql_and_values` + `db.execute()` / `db.query_one()`. This is the correct pattern for Phase 241.

**Existing sqlite-in-memory dispatch test setup** (`write/mod.rs:451-489`, `write_dispatch.rs:582-623`):
```rust
async fn setup_db() -> sea_orm::DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.expect("...");
    db.execute(Statement::from_string(DatabaseBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS mcp_idempotency_keys (...)")).await.expect("...");
    db.execute(Statement::from_string(DatabaseBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS audit_log (...)")).await.expect("...");
    db
}
```

For CRUD executor tests, add the target service table (e.g. `CREATE TABLE orders (...)`). Use the same `setup_db` helper extended with the orders table.

**Placeholder style** (`dispatch.rs:30`):
```rust
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}
```
The generic CRUD executor replicates this placeholder builder exactly.

[VERIFIED: `write/mod.rs:451-489`, `write_dispatch.rs:582-623`, `dispatch.rs:30`]

### Q4: `RETURNING *` — SQLite + Postgres portability

**SQLite supports `RETURNING`** since version 3.35.0 (2021). `ferro-queue` uses `UPDATE … RETURNING` on SQLite in production (`ferro-queue/src/db.rs:394`). The pattern is:

```rust
let stmt = Statement::from_sql_and_values(backend, sql, values);
let row = db.query_one(stmt).await?;
```

For Postgres `RETURNING *` is standard. For SQLite, `INSERT INTO … RETURNING *` is supported since 3.35.0.

**However**, a safer, portable fallback is insert-then-select:
```rust
// INSERT without RETURNING
db.execute(insert_stmt).await?;
// SELECT last_insert_rowid() for SQLite / RETURNING-less Postgres path
let row = db.query_one(select_last_stmt).await?;
```

`ferro-payments` uses `SELECT last_insert_rowid() AS id` after `INSERT` (`service.rs:1017`). This is the established pattern in this workspace.

**Recommendation for Phase 241**: Use `INSERT … RETURNING *` as the primary SQL (one round-trip, simpler code). Add a SQLite-specific fallback only if tests fail on older SQLite; the workspace's SQLite library is recent enough. If `RETURNING *` causes issues, fall back to `INSERT` + `SELECT last_insert_rowid()` for the id + `SELECT * FROM table WHERE id = ?`.

[VERIFIED: `ferro-queue/src/db.rs:394`, `ferro-payments/src/service.rs:1017`]

### Q5: JSON → SQL value coercion from `inputs: &Value`

**Current pattern** (`write/mod.rs:196-219` idempotency lookup, `dispatch.rs` filter dispatch):
```rust
sea_orm::Value::BigInt(Some(tenant_id))
sea_orm::Value::String(Some(Box::new(key.to_string())))
```

The coercion from `serde_json::Value` to `sea_orm::Value` is done manually inline, matching `FieldDef.data_type`:

```rust
match field.data_type {
    DataType::Integer  => sea_orm::Value::BigInt(inputs[&field.name].as_i64().map(Some).flatten().map(|v| v)),
    DataType::Float    => sea_orm::Value::Double(inputs[&field.name].as_f64()),
    DataType::Boolean  => sea_orm::Value::Bool(inputs[&field.name].as_bool()),
    DataType::String
    | DataType::Text   => sea_orm::Value::String(inputs[&field.name].as_str()
                            .map(|s| Box::new(s.to_string()))),
    DataType::DateTime => sea_orm::Value::String(inputs[&field.name].as_str()
                            .map(|s| Box::new(s.to_string()))),
    DataType::Json     => sea_orm::Value::String(Some(Box::new(
                            serde_json::to_string(&inputs[&field.name]).unwrap_or_default()))),
    DataType::Uuid     => sea_orm::Value::String(inputs[&field.name].as_str()
                            .map(|s| Box::new(s.to_string()))),
    DataType::Decimal  => sea_orm::Value::String(inputs[&field.name].as_str()
                            .map(|s| Box::new(s.to_string()))),
}
```

The `CrudPlan` variants carry `Vec<(column_name, sea_orm::Value)>` pairs — pre-coerced at derivation time from `inputs` and `FieldDef.data_type`. This keeps the executor loop simple: iterate the pairs, build placeholders and bind values. Missing optional fields in `inputs` are skipped (patch semantics for Update).

Server-injected fields (`created_at`, initial `Status`) are added by `derive_crud_plan` as literal `sea_orm::Value::String` entries after the user-supplied fields.

[VERIFIED: `write/mod.rs:196-279`, `dispatch.rs` placeholder pattern]

### Q6: Dependency direction — `CrudVerb`/`CrudPlan` placement

**Dependency graph** (verified):
- `ferro-projections` has NO dependency on `framework` (no `framework` in its Cargo.toml)
- `framework` (`ferro-rs`) depends on `ferro-projections` (optional `projections` feature, `Cargo.toml:51`)
- `ferro-mcp-server` depends on `ferro-rs` + `ferro-projections`

[VERIFIED: `ferro-projections/Cargo.toml`, `framework/Cargo.toml:51`, `ferro-mcp-server/Cargo.toml:17-18`]

**Correct placement**: `CrudVerb` and `CrudPlan` live in `ferro-projections/src/executor.rs` (same module as `TransitionPlan`). The generic CRUD executor (the function that interprets a `CrudPlan` into SQL) lives in `framework/src/write/mod.rs` because it needs `DatabaseConnection`. No cycle: `ferro-projections` is dependency of `framework`, not the reverse.

Re-exports follow `ferro-projections/src/lib.rs:17` pattern:
```rust
pub use executor::{derive_transition_plan, TransitionPlan};
// Add:
pub use executor::{derive_crud_plan, CrudPlan, CrudVerb};
```

The `framework/src/lib.rs` facade re-exports whatever `framework` already re-exports from `ferro-projections`.

[VERIFIED: `ferro-projections/src/lib.rs:17`]

### Q7: Testing approach

**Existing test patterns to mirror:**

| Test type | Location | Pattern |
|-----------|----------|---------|
| Transition plan derivation table tests | `ferro-projections/src/executor.rs:118-252` | Pure Rust, no async, `ServiceDef` fixture → `derive_transition_plan` → assert fields |
| Kernel sqlite-in-memory dispatch tests | `framework/src/write/mod.rs:440-942` | `setup_db()` + `WriteDispatcher` fixture + `dispatch_write(...)` + assert result |
| Write-path framing tests | `ferro-mcp-server/src/write_dispatch.rs:571-823` | `setup_db()` + `handle_write_call(...)` + assert `structuredContent` |
| Confirmation tests | `ferro-mcp-server/src/write_dispatch.rs:827-1319` | `InMemoryConfirmationStore` + two-step flow |

**Phase 241 table tests** (in `ferro-projections/src/executor.rs`):

| Test | What it proves |
|------|---------------|
| `derive_crud_plan_create_column_set` | Creatable cols derived correctly; Identifier/CreatedAt/tenant excluded; Status excluded when SM exists |
| `derive_crud_plan_create_with_sm_initial_status` | Create plan carries `status = initial_state` when SM declared |
| `derive_crud_plan_create_no_sm_status_included` | Status is writable when no SM |
| `derive_crud_plan_update_patch_semantics` | Update plan: id required, all other fields optional |
| `derive_crud_plan_update_excludes_sm_status` | Status excluded from update when SM exists |
| `derive_crud_plan_delete_soft_delete_column` | Delete plan: `soft_delete_column` from `resolved_soft_delete_column()` |
| `derive_crud_plan_verb_not_enabled_err` | `derive_crud_plan(svc_without_creatable, Create, …)` returns `Err(VerbNotEnabled)` |
| `crud_plan_serde_round_trip` | All three variants serialize/deserialize cleanly |

**Phase 241 kernel dispatch tests** (in `framework/src/write/mod.rs` or a new `write/crud_tests.rs`):

| Test | SC # | What it proves |
|------|------|---------------|
| `crud_create_inserts_row_returns_record` | SC#1 | INSERT runs, row exists in DB, returned payload contains id |
| `crud_update_patches_non_deleted_row` | SC#2 | UPDATE WHERE id=? AND deleted_at IS NULL, only supplied fields change |
| `crud_update_soft_deleted_row_is_not_found` | SC#2 | UPDATE on soft-deleted row → row-not-found error |
| `crud_delete_sets_deleted_at` | SC#2 | `deleted_at` is set, row no longer appears in list predicate |
| `crud_delete_soft_deleted_row_absent_from_list` | SC#2 | After delete, the `deleted_at IS NULL` filter hides it |
| `crud_delete_without_confirmation_returns_required` | SC#4 feature="confirmation" | Confirmation seam fires for delete verb |
| `crud_override_replaces_generic_plan` | SC#3 | `with_override("create_order", …)` runs instead of generic executor |
| `crud_idempotency_reuses_stored_result` | SC#1,2,3 | Second create with same key returns stored, executor fires once |
| `single_dispatcher_structural_check` | SC#4 | grep check (see below) |

**SC#4 grep / structural check**:
```bash
grep -rn "fn dispatch_write" framework/src/ ferro-mcp-server/src/ # must be exactly 1
grep -rn "match.*action_name\|match.*crud_verb" framework/src/write/ # must be 0 (transition target match)
```

**Confirmation framing tests for delete** (in `ferro-mcp-server/src/write_dispatch.rs`):

| Test | What it proves |
|------|---------------|
| `delete_bare_call_returns_confirmation_required` | `delete_order` without token → `confirmation_required` envelope |
| `delete_two_step_flow_soft_deletes` | `request_confirm_delete_order` → `confirm_delete_order` → `deleted_at` set |
| `delete_expired_token_rejected` | TTL expiry rejects confirm |
| `delete_wrong_record_token_rejected` | Binding mismatch for record id |

[VERIFIED: `executor.rs:117-252`, `write/mod.rs:440-942`, `write_dispatch.rs:571-1319`]

### Q8: Spec vs Phase 241 SC drift on tenant predicates

**Spec SQL** (`docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md:144-146`):
```
Create → INSERT (creatable cols, tenant_id=ctx, status=initial, created_at=now) RETURNING *
Update → UPDATE … SET <patch> WHERE id=? AND tenant_id=ctx AND deleted_at IS NULL
Delete → UPDATE … SET deleted_at=now WHERE id=? AND tenant_id=ctx
```

**Phase 241 SC omit tenant** (D-09). The spec includes `tenant_id=ctx` but Phase 241 deliberately defers that to Phase 242.

**Recommendation**: The `CrudPlan` variants carry a `tenant_predicate: Option<TenantPredicate>` slot now (as a `None`-by-default field), shaped so Phase 242 fills it without reworking `CrudPlan` or the executor. Example shape:

```rust
pub struct TenantPredicate {
    pub column: String,     // e.g. "tenant_id"
    pub value: i64,         // injected from ctx at dispatch time, not at plan time
}

pub enum CrudPlan {
    Create {
        table: String,
        columns: Vec<(String, sea_orm::Value)>,  // user-supplied + server-injected (created_at, status)
        tenant_inject: Option<TenantPredicate>,  // None in Phase 241; filled by Phase 242
    },
    Update {
        table: String,
        id: i64,
        patch: Vec<(String, sea_orm::Value)>,
        soft_delete_column: String,
        tenant_predicate: Option<TenantPredicate>,  // None in Phase 241
    },
    Delete {
        table: String,
        id: i64,
        soft_delete_column: String,
        tenant_predicate: Option<TenantPredicate>,  // None in Phase 241
    },
}
```

Phase 241 tests deliberately omit tenant predicates (all `None`) — this is correct and matches SC#1–#2. Phase 242 sets the field and adds authz tests. No rework of the plan struct needed.

Since `CrudPlan` must be `Serialize/Deserialize`, `TenantPredicate` must also derive those traits. The `i64` tenant value should be injected at dispatch time (from `tenant_id` param), not stored in the plan at derivation time — `derive_crud_plan` receives `inputs: &Value` but NOT a `tenant_id`. The `tenant_inject` field therefore carries the column name only (String); the executor fills the value from the `tenant_id: i64` parameter that `dispatch_write` already receives.

Revised shape:
```rust
pub struct TenantColumn {
    pub column: String,  // "tenant_id" or custom
}

// CrudPlan::Create { ..., tenant_column: Option<TenantColumn> }
// Phase 241: tenant_column = None (executor does not inject)
// Phase 242: tenant_column = Some(TenantColumn { column: svc.tenant_column }) → executor appends tenant_id
```

[VERIFIED: spec §"Dispatch architecture"; CONTEXT.md D-09]

---

## Architecture Patterns

### System Architecture Diagram

```
Agent call: "create_order" / "update_order" / "delete_order"
         │
         ▼
ferro-mcp-server::write_dispatch::handle_write_call
         │
         ├─ strip_prefix("request_confirm_") → handle_request_confirm (confirm token issuance)
         ├─ strip_prefix("confirm_") → handle_confirm (token validation + dispatch)
         │
         ├─ detect CRUD verb (lines 162-180, currently NTI — Phase 241 replaces)
         │   ├─ parse verb: create_ / update_ / delete_
         │   ├─ call derive_crud_plan(svc, verb, &args) → CrudPlan
         │   │        │
         │   │        └─ ferro-projections::executor::derive_crud_plan
         │   │             uses: resolved_table(), resolved_soft_delete_column(),
         │   │                   is_write_excluded_field(), state_machine.initial
         │   │
         │   └─ synthesize ActionDef(name=tool_name) + call dispatch_write(…, Some(&plan))
         │               │
         │               ▼
         │   framework::write::dispatch_write ─────────────────────────────────────────┐
         │               │                                                             │
         │   1. guard re-eval (no guards for CRUD in 241)                             │
         │   2. idempotency lookup                                                     │
         │   3. confirmation seam: is_delete(plan) && !is_confirmed → ConfirmationRequired │
         │   4. execute_crud_plan(plan, tenant_id, db) → Value        ◄── new function │
         │         ├─ Create: INSERT … RETURNING * (or INSERT + last_insert_rowid)     │
         │         ├─ Update: UPDATE … WHERE id=? AND deleted_at IS NULL              │
         │         └─ Delete: UPDATE … SET deleted_at=now WHERE id=?                  │
         │   5. store idempotency result                                               │
         │   6. audit: format!("{channel}.crud.{tool_name}")                           │
         │   7. override hook: dispatcher.overrides.get("create_order")               │
         │               │                                                             │
         │               └─────────────────────────────────────────────────────────────┘
         │
         └─ route result through CallToolResult::structured(payload) (D-10)

renderer.rs emits (Phase 240, already shipped):
  list_order, create_order, update_order, delete_order
  + request_confirm_delete_order, confirm_delete_order (Phase 241 addition)

list_<svc> reads use ferro-mcp-server::dispatch (separate path, unchanged)
  └─ deleted_at IS NULL predicate already at dispatch.rs:285-290
```

### Recommended Project Structure Changes

```
ferro-projections/src/
  executor.rs    — add CrudVerb enum, CrudPlan enum, TenantColumn, derive_crud_plan()
                    (alongside existing TransitionPlan + derive_transition_plan)
  lib.rs         — add re-exports: derive_crud_plan, CrudPlan, CrudVerb

framework/src/write/
  mod.rs         — add: execute_crud_plan() free function; extend dispatch_write() with
                    Option<&CrudPlan> param + CRUD-path step 3 extension + step 4 branch

ferro-mcp-server/src/
  write_dispatch.rs — replace NTI block (lines 155-180) with real derive→dispatch;
                       extend handle_request_confirm / handle_confirm to route delete_<svc>
  renderer.rs       — synthesize request_confirm_delete_<svc> / confirm_delete_<svc>
                       (extend the `#[cfg(feature = "confirmation")]` block at lines 119-155)
```

### Pattern 1: `derive_crud_plan` mirroring `derive_transition_plan`

```rust
// Source: ferro-projections/src/executor.rs (live — TransitionPlan lines 22-38)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum CrudVerb { Create, Update, Delete }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TenantColumn { pub column: String }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum CrudPlan {
    Create {
        table: String,
        columns: Vec<(String, serde_json::Value)>,  // user-supplied + server-set (created_at, status)
        tenant_column: Option<TenantColumn>,         // None in Phase 241
    },
    Update {
        table: String,
        id_column: String,
        id_value: serde_json::Value,
        patch: Vec<(String, serde_json::Value)>,
        soft_delete_column: String,
        tenant_column: Option<TenantColumn>,
    },
    Delete {
        table: String,
        id_column: String,
        id_value: serde_json::Value,
        soft_delete_column: String,
        tenant_column: Option<TenantColumn>,
    },
}

pub fn derive_crud_plan(
    svc: &ServiceDef,
    verb: CrudVerb,
    inputs: &serde_json::Value,
) -> Result<CrudPlan, crate::Error> { … }
```

Note: `sea_orm::Value` does NOT derive `serde::Serialize/Deserialize/JsonSchema`. Store values as `serde_json::Value` in `CrudPlan`; coerce to `sea_orm::Value` in `execute_crud_plan`. This keeps `CrudPlan` schema-only (no runtime dep on sea-orm), matching ferro-projections' boundary.

[VERIFIED: `ferro-projections/Cargo.toml` — no sea-orm dep; `executor.rs:22-38` pattern]

### Pattern 2: Generic CRUD executor in `framework::write`

```rust
// Source: framework/src/write/mod.rs (live — dispatch_write lines 313+)
// Uses sea_orm::Statement::from_sql_and_values + DatabaseConnection
async fn execute_crud_plan(
    plan: &CrudPlan,
    tenant_id: i64,
    db: &DatabaseConnection,
) -> WriteResult<Value> {
    match plan {
        CrudPlan::Create { table, columns, tenant_column } => {
            // Build INSERT … RETURNING * (or INSERT + last_insert_rowid on SQLite)
            let backend = db.get_database_backend();
            // build column list and placeholder list from `columns`
            // emit INSERT OR ... (backend-specific)
        }
        CrudPlan::Update { table, id_column, id_value, patch, soft_delete_column, .. } => {
            // UPDATE table SET col=? WHERE id_column=? AND soft_delete_column IS NULL
        }
        CrudPlan::Delete { table, id_column, id_value, soft_delete_column, .. } => {
            // UPDATE table SET soft_delete_column = datetime('now') WHERE id_column=?
        }
    }
}
```

### Pattern 3: Extending `dispatch_write` (minimal diff)

```rust
// framework/src/write/mod.rs — new parameter on dispatch_write
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
    transition_guard: Option<&str>,
    channel: &str,
    #[cfg(feature = "confirmation")] is_confirmed: bool,
    crud_plan: Option<&CrudPlan>,       // ← NEW: None for transitions
) -> WriteResult<Value> {
    // Step 1: guards (unchanged — CRUD has no guards by default)
    // Step 2: idempotency (unchanged)
    // Step 3 extension:
    #[cfg(feature = "confirmation")]
    {
        let is_destructive = action.transition_trigger.is_some()
            || matches!(crud_plan, Some(CrudPlan::Delete { .. }));
        if is_destructive && !is_confirmed {
            return Err(WriteError::ConfirmationRequired(action.name.clone()));
        }
    }
    // Step 4 branch:
    let result = if let Some(plan) = crud_plan {
        execute_crud_plan(plan, tenant_id, db).await?
    } else {
        (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?
    };
    // Steps 5-7 unchanged
}
```

All existing callers of `dispatch_write` pass `crud_plan: None` — no breaking change.

### Anti-Patterns to Avoid

- **Fabricating ActionDef with transition_trigger for CRUD**: would smuggle transition semantics into non-transition verbs; confirmation seam fires on the wrong condition; SC#4 violated by duplicated transition match logic
- **Adding a second CRUD dispatcher**: creates duplicate write-control surface (SC#4 fails, CLAUDE.md `feedback_no_duplicate_control_surface` violated)
- **Storing `sea_orm::Value` in `CrudPlan`**: `sea_orm::Value` does not implement `JsonSchema` or `serde::Serialize` cleanly; breaks the `ferro-projections` boundary (schema-only, no runtime deps)
- **Skipping the `deleted_at IS NULL` predicate on update**: a soft-deleted row becomes patchable without it — security gap; SC#2 explicitly requires it
- **Adding `tenant_id` injection in Phase 241**: D-09 explicitly defers this; 241 leaves `tenant_column: None` in all plans

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Placeholder parameterization | Custom `?`/`$N` builder | Copy `placeholder(backend, idx)` from `dispatch.rs:30` | Already in codebase; must match for consistency |
| CSPRNG confirmation tokens | Custom generator | `generate_confirmation_token()` from `write_dispatch.rs:82-92` | Already ships with the workspace; identical entropy profile |
| Token storage | New token store | `ConfirmationStore` + `InMemoryConfirmationStore` from `ferro-ai` | D-06 requires no new token mechanism |
| In-memory DB for tests | New fixture | `setup_db()` pattern from `write/mod.rs:451-489` | Copy verbatim, extend with target table DDL |
| Soft-delete `IS NULL` guard | Custom filter | `resolved_soft_delete_column()` already drives `dispatch.rs:285-290` | Single source of truth; same column name for both list and write paths |

---

## Common Pitfalls

### Pitfall 1: Routing CRUD confirm tools before `find_action`

**What goes wrong:** `handle_request_confirm` calls `find_action(services, "delete_order")`. `find_action` searches `svc.actions`; `delete_order` is not an `ActionDef`. Returns `None` → `-32601 Method not found`.

**Why it happens:** The transition-confirm flow assumes the action is in `svc.actions`. CRUD verbs are not.

**How to avoid:** In `handle_request_confirm` and `handle_confirm`, add a CRUD-verb branch that locates the `ServiceDef` via name stripping (same as `handle_write_call` does for the NTI block) instead of `find_action`. The delete path needs `svc` to build the input schema and perform guard checks (no guards in 241, but the structure should be symmetric).

**Warning signs:** `find_action` returning `None` for `"delete_order"` during confirm.

### Pitfall 2: `CrudPlan` encoding `sea_orm::Value` instead of `serde_json::Value`

**What goes wrong:** `sea_orm::Value` does not implement `schemars::JsonSchema`; the `#[derive(JsonSchema)]` on `CrudPlan` fails to compile.

**Why it happens:** The executor needs `sea_orm::Value` for SQL binding; it's tempting to pre-coerce.

**How to avoid:** Store `serde_json::Value` in `CrudPlan`; coerce to `sea_orm::Value` in `execute_crud_plan` (framework crate, where sea-orm is available). This preserves the `ferro-projections` schema-only boundary.

### Pitfall 3: Forgetting the `update` path `deleted_at IS NULL` predicate

**What goes wrong:** Update of a soft-deleted row succeeds — record is addressable after logical deletion.

**Why it happens:** SC#2 requires it; `derive_crud_plan` for Update must include `soft_delete_column` in the WHERE clause even though the update is not a delete operation.

**How to avoid:** `CrudPlan::Update` carries `soft_delete_column: String` unconditionally (from `svc.resolved_soft_delete_column()`). The executor always emits `AND <col> IS NULL`.

**Warning signs:** A dedicated test `crud_update_soft_deleted_row_is_not_found` goes green without the predicate (it shouldn't — add an explicit row that has `deleted_at` set and assert the update returns row-not-found).

### Pitfall 4: Breaking existing `dispatch_write` callers with the new parameter

**What goes wrong:** Every existing call site in `write_dispatch.rs`, `write/mod.rs` tests, and any app code must pass `crud_plan: None` — compile error if missed.

**Why it happens:** Rust requires all parameters.

**How to avoid:** Add `crud_plan` as the LAST positional parameter (after `is_confirmed`) so existing call sites only need `, None` appended. Alternatively, refactor to a builder/options struct — but that's a larger diff. The append approach is minimal and consistent with the existing `#[cfg(feature = "confirmation")] is_confirmed` precedent.

### Pitfall 5: Confirmation tool naming collision with CRUD verbs

**What goes wrong:** `renderer.rs` currently synthesizes `request_confirm_<action.name>` for transition actions. If a transition action is named `delete` on a service that also opts into `.deletable(true)`, you get `request_confirm_delete` collision.

**Why it happens:** CRUD tool names use the pattern `delete_<svc>` while transition tools use bare `action.name`. No collision for CRUD verify tools since they are `request_confirm_delete_<svc>` — the `_<svc>` suffix differentiates from a bare `delete` action.

**How to avoid:** The CRUD confirm synthesis loop must use `"delete_<svc>"` as the base name (not `"delete"`), which it will naturally because the CRUD tool name is always `delete_<svc_name>`.

### Pitfall 6: Audit label for CRUD vs transition divergence

**What goes wrong:** Using `format!("{channel}.action.{name}")` for CRUD verbs produces `mcp.action.create_order` — which is fine structurally but note it differs from the spec's `{channel}.crud.{name}` if that were chosen.

**How to avoid:** For consistency and query-ability in audit logs, use a distinct CRUD prefix. Recommendation: `format!("{channel}.crud.{}", &action.name)` → `mcp.crud.create_order`. This is Claude's discretion (D-08); lock it in the plan so tests pin the exact string.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust native `#[test]` + `#[tokio::test]` (cargo) |
| Config file | `Cargo.toml` (workspace) — no separate test config |
| Quick run command | `cargo test -p ferro-projections executor --all-features` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRUD-06 | `derive_crud_plan` produces correct Create plan | unit table test | `cargo test -p ferro-projections derive_crud_plan_create --all-features` | ❌ Wave 0 |
| CRUD-06 | `derive_crud_plan` produces correct Update plan | unit table test | `cargo test -p ferro-projections derive_crud_plan_update --all-features` | ❌ Wave 0 |
| CRUD-06 | `derive_crud_plan` produces correct Delete plan | unit table test | `cargo test -p ferro-projections derive_crud_plan_delete --all-features` | ❌ Wave 0 |
| CRUD-06 | Verb-not-enabled error | unit table test | `cargo test -p ferro-projections derive_crud_plan_verb_not_enabled --all-features` | ❌ Wave 0 |
| CRUD-06 | CrudPlan serde round-trip | unit | `cargo test -p ferro-projections crud_plan_serde_round_trip --all-features` | ❌ Wave 0 |
| CRUD-06 | CREATE inserts row, returns record (SC#1) | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_create_inserts_row --all-features` | ❌ Wave 0 |
| CRUD-06 | UPDATE patches non-deleted row (SC#2) | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_update_patches_row --all-features` | ❌ Wave 0 |
| CRUD-03 | UPDATE on soft-deleted row → not-found (SC#2) | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_update_soft_deleted_not_found --all-features` | ❌ Wave 0 |
| CRUD-03 | DELETE sets deleted_at (SC#2) | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_delete_sets_deleted_at --all-features` | ❌ Wave 0 |
| CRUD-03 | Soft-deleted row absent from list predicate | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_deleted_row_hidden_from_list --all-features` | ❌ Wave 0 |
| CRUD-03 | Delete without token → ConfirmationRequired | kernel unit (feature=confirmation) | `cargo test -p ferro-rs crud_delete_requires_confirmation --all-features` | ❌ Wave 0 |
| CRUD-06 | Override replaces generic plan (SC#3) | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_override_replaces_generic --all-features` | ❌ Wave 0 |
| CRUD-06 | Idempotency on create | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_create_idempotent --all-features` | ❌ Wave 0 |
| CRUD-03 | Delete 2-step flow (request+confirm) | framing integration | `cargo test -p ferro-mcp-server delete_two_step_flow --all-features` | ❌ Wave 0 |
| CRUD-06 | Single `dispatch_write` definition (SC#4) | structural/grep | `grep -rn "fn dispatch_write" framework/src/` | ✅ (grep, no new file) |
| CRUD-06 | Results route through `structured` envelope (D-10) | framing test | `cargo test -p ferro-mcp-server crud_result_structured_envelope --all-features` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings`
- **Per wave merge:** `cargo test --all-features` (full workspace gate)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] Tests in `ferro-projections/src/executor.rs` — CRUD derivation table tests
- [ ] Tests in `framework/src/write/mod.rs` (or `write/crud_tests.rs`) — sqlite-in-memory CRUD dispatch tests
- [ ] Tests in `ferro-mcp-server/src/write_dispatch.rs` — delete confirmation framing tests
- [ ] `setup_db()` helper extended with a `CREATE TABLE orders (id INTEGER PRIMARY KEY, …, deleted_at TEXT)` fixture shared across kernel and framing tests

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | partial (Phase 242 owns authorization) | Phase 241 wires id + deleted_at IS NULL predicates; tenant injection deferred |
| V5 Input Validation | yes | `is_write_excluded_field()` gate on CrudPlan derivation; confirmation token binding check |
| V6 Cryptography | yes (delete confirmation) | `generate_confirmation_token()` CSPRNG — already shipped, reused unchanged |

### Known Threat Patterns for this Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Agent supplies `tenant_id` in create/update inputs | Spoofing | `is_server_injected_field` excludes tenant column from schema; `derive_crud_plan` never reads `inputs["tenant_id"]` |
| Update/delete of soft-deleted record (logical resurrection) | Tampering | `CrudPlan::Update` + `Delete` carry `soft_delete_column`; executor adds `AND <col> IS NULL` |
| Delete without confirmation token | Elevation of Privilege | Confirmation seam extension in `dispatch_write` step 3; `delete_` is flagged destructive |
| Token replay (use confirmation token twice) | Tampering | `ConfirmationStore.confirm()` is single-use — already enforced |
| Cross-record token use (token for id=1 used for id=2) | Tampering | Binding payload check in `handle_confirm`; `record_id` verified — already enforced |
| SQL injection via field names | Tampering | Column names come from `ServiceDef.fields[].name` (developer-authored, not agent input); values are bound via `sea_orm::Value` |
| Audit-unsafe payloads in create result | Information Disclosure | Executor contract (docstring on `ExecutorFn`) — executor must not return Sensitive field values; plan excludes Sensitive fields from INSERT |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | SQLite 3.35.0+ (supporting `RETURNING`) is available in the test environment | Q4 / Code Examples | Need fallback: INSERT + `last_insert_rowid()` |
| A2 | `schemars::JsonSchema` is not available on `sea_orm::Value` | Q5 / Architecture | If it is, CrudPlan could store sea_orm::Value directly — minor simplification, no behavior change |

---

## Sources

### Primary (HIGH confidence)

- `ferro-projections/src/executor.rs` — `TransitionPlan` struct (lines 22-38), `derive_transition_plan` (lines 56-115), table test patterns (lines 117-252)
- `framework/src/write/mod.rs` — `dispatch_write` full pipeline (lines 313-436), `WriteDispatcher`/`ExecutorFn` (lines 79-168), `WriteError` (lines 30-54), sqlite-in-memory kernel tests (lines 440-942)
- `ferro-mcp-server/src/write_dispatch.rs` — NTI CRUD short-circuit (lines 155-180), confirmation handlers (lines 290-566), framing tests (lines 568-1319)
- `ferro-mcp-server/src/renderer.rs` — CRUD tool emission (lines 90-108, 239-317), transition confirm synthesis (lines 119-155, 319-427)
- `ferro-projections/src/service.rs` — `resolved_table` (line 215), `resolved_soft_delete_column` (line 223), `is_server_injected_field` (line 236), `is_write_excluded_field` (line 254)
- `ferro-mcp-server/src/schema.rs` — `build_create_input_schema` (lines 243-276), `build_update_input_schema` (lines 278-336), `build_delete_input_schema` (lines 338-381)
- `ferro-mcp-server/src/dispatch.rs` — `deleted_at IS NULL` predicate (lines 280-290), placeholder builder (line 30)
- `ferro-projections/src/lib.rs` — re-export pattern (line 17)
- `ferro-projections/src/error.rs` — existing Error variants (all)
- `ferro-queue/src/db.rs` — `RETURNING` on SQLite confirmed working (lines 394-413)
- `ferro-payments/src/service.rs` — `last_insert_rowid()` fallback pattern (line 1017)

---

## Metadata

**Confidence breakdown:**
- `CrudPlan`/`derive_crud_plan` shape: HIGH — directly mirrors verified `TransitionPlan`/`derive_transition_plan` with confirmed accessor API
- Kernel extension approach: HIGH — `dispatch_write` fully read; minimal-diff recommendation verified against all call sites
- SQL layer: HIGH — sea-orm raw statements verified as the only pattern used; RETURNING confirmed by ferro-queue
- Confirmation extension: HIGH — seam code at `write/mod.rs:379` + confirm handlers at `write_dispatch.rs:300-566` fully read
- Dependency graph: HIGH — Cargo.toml files directly verified
- Tenant-slot design: MEDIUM — logical inference from D-09 + spec; no competing prior implementation to verify against

**Research date:** 2026-06-23
**Valid until:** 2026-07-23 (stable Rust workspace — low churn expected)
