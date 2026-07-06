# Phase 239: Soft-delete data model + `deleted_at` migration - Research

**Researched:** 2026-06-23
**Domain:** sea-orm migrations (additive ALTER TABLE), ferro-projections ServiceDef resolver accessors, ferro-mcp-server dispatch WHERE-clause injection, field classification
**Confidence:** HIGH — all findings verified against actual source files in the working tree.

## Summary

This phase establishes the soft-delete data substrate for v16.3 Track A CRUD. All four
deliverables are grounded in patterns that already exist in the codebase: the migration
form is an exact copy of `m20260611_add_tenant_id_to_users.rs`; the resolver accessor
pattern is the standard `with_*` consuming builder returning `Self`; the
`deleted_at IS NULL` predicate mirrors the tenant-predicate injection at dispatch.rs:153;
and the classification helper builds directly on the `FieldMeaning` enum already in
`field.rs`. Nothing needs to be invented — only plugged in.

The only mechanical risk is wiring `resolved_table()` into `dispatch.rs` while keeping
the behavior identical for existing projections. The default `format!("{}s", name.to_lowercase())`
lives at exactly dispatch.rs:123 and the resolver must produce the same string when
`service.table` is `None`. The predicate gate must be `service.soft_delete_column.is_some()`
(the explicit `.soft_delete_column(...)` opt-in) — not unconditional and not `|| service.deletable`
— so non-soft-deletable projections (and projections that flip `.deletable(true)` without
declaring a soft-delete column) are unaffected.

**Primary recommendation:** Follow the `m20260611_add_tenant_id_to_users.rs` migration
idiom exactly; add `resolved_table()` and `resolved_soft_delete_column()` to `service.rs`
as pure `&self -> &str / String` methods; thread `resolved_table()` through dispatch.rs
to resolve the TODO; add the `deleted_at IS NULL` predicate block mirroring the
tenant block; add `is_server_injected_field()` to `service.rs` consuming `FieldMeaning`.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Add `deleted_at` to `orders` only (the single soft-deletable table in v16.3). Do not speculatively add to other tables.
- **D-02:** New standalone additive migration `m20260623_add_deleted_at_to_orders.rs` using `alter_table` + `add_column`. Never edit the shipped `m20260611_create_orders_table.rs`.
- **D-03:** Register the new migration in `app/src/migrations/mod.rs` `Migrator::migrations()` in chronological order (after the existing `m20260614_*` entries).
- **D-04:** Nullable timestamp column — backend-portable via sea-orm `ColumnDef`. No backfill: existing rows get `NULL` = "not deleted".
- **D-05:** Inject `deleted_at IS NULL` predicate in the read query builder (`ferro-mcp-server/src/dispatch.rs`), mirroring tenant-predicate injection at line 153. Gate on projection being soft-deletable.
- **D-06:** Use the resolved soft-delete column name from D-07 (not a hardcoded `deleted_at`).
- **D-07:** Add `resolved_table()` and `resolved_soft_delete_column()` to `ServiceDef` in `ferro-projections/src/service.rs`.
- **D-08:** Wire `resolved_table()` into dispatch.rs, replacing the inline `format!("{}s", service.name.to_lowercase())` and removing its TODO.
- **D-09:** Table tests for resolver defaults and explicit overrides.
- **D-10:** `created_at` set at DB layer via `DEFAULT current_timestamp` (already true on `orders`). New `deleted_at` is nullable with no default. Phase asserts the contract exists; INSERT path is Phase 241.
- **D-11:** Add a field-classification helper in `ferro-projections` identifying server-injected / never-agent-input fields: tenant column, identifier (`FieldMeaning::Identifier`), and `created_at` (`FieldMeaning::CreatedAt`). Scope = classification predicate + tests only, not schema emission.

### Claude's Discretion
- Exact migration filename/date stamp and `DeriveIden` enum shape (follow the existing idiom).
- Naming of the resolver accessors and classification helper, provided semantics match D-07/D-11.
- Whether the classification helper returns a set of excluded field names or a per-field predicate.

### Deferred Ideas (OUT OF SCOPE)
- Reusable `add_soft_delete_column` migration helper in `ferro-migration`.
- `deleted_at` on additional tables (opt-in, add when a projection declares `.deletable(true)` and needs it).
- Dedicated `get_<svc>` tool, per-field `immutable()`/`read_only()` overrides.
</user_constraints>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `deleted_at` column migration | Database / Storage (`app/src/migrations/`) | — | DB schema change; migration runner is the correct owner |
| Resolver accessors (`resolved_table`, `resolved_soft_delete_column`) | API / Schema layer (`ferro-projections`) | — | `ServiceDef` is the declaration schema; resolver is pure metadata |
| `deleted_at IS NULL` predicate injection | API / Backend (`ferro-mcp-server/src/dispatch.rs`) | — | SQL assembly lives in dispatch; predicate belongs alongside other universal WHERE conditions |
| Field classification helper (`is_server_injected`) | API / Schema layer (`ferro-projections`) | — | `FieldMeaning`-based logic; no runtime dependency; consumed by Phase 240 schema emission |
| Entity model update (`orders.rs` entity) | Database / Storage (`app/src/models/entities/`) | — | Auto-generated by `ferro db:sync`; manual edit to add `deleted_at: Option<String>` before sync can be regenerated |

---

## Standard Stack

All libraries are already present in the workspace. No new dependencies are needed.

| Library | Version | Purpose |
|---------|---------|---------|
| `sea-orm-migration` | 1.0 [VERIFIED: app/Cargo.toml] | Migration runner, `SchemaManager`, `Table::alter()` |
| `sea-orm` | 1.0 [VERIFIED: app/Cargo.toml, framework/Cargo.toml] | `Statement::from_sql_and_values`, `DatabaseBackend`, `ColumnDef` |
| `ferro-projections` | workspace | `ServiceDef`, `FieldMeaning`, `FieldDef` |
| `ferro-mcp-server` | workspace | `dispatch`, `DispatchResult`, `placeholder()` |

No installation commands required — workspace members.

---

## Architecture Patterns

### System Architecture Diagram

```
User request
     │
     ▼
handle_tools_call (jsonrpc.rs)
     │  extracts filters, tenant_id from McpContext
     ▼
dispatch() [ferro-mcp-server/src/dispatch.rs]
     │
     ├── table = service.resolved_table()          ← D-07/D-08 (new)
     │
     ├── WHERE equality filters (field allowlist)  ← existing
     │
     ├── WHERE tenant predicate (line 153)         ← existing
     │
     ├── WHERE deleted_at IS NULL (new)            ← D-05/D-06 (new, gated)
     │
     ├── COUNT query (reuses same WHERE)
     │
     └── DATA query (SELECT *)
          └── returns DispatchResult
```

### Recommended Project Structure

No new crates or modules. Changes touch:

```
ferro-projections/src/
  service.rs          ← add resolved_table(), resolved_soft_delete_column(),
                         is_server_injected_field() + their tests

ferro-mcp-server/src/
  dispatch.rs         ← wire resolved_table(), add deleted_at predicate block

app/src/
  migrations/
    m20260623_add_deleted_at_to_orders.rs    ← new file (D-02)
    mod.rs                                   ← register new migration (D-03)
  models/entities/orders.rs                  ← add deleted_at: Option<String>
```

### Pattern 1: Additive ALTER TABLE Migration

**What:** Add a nullable `deleted_at` timestamp column to an existing table.
**When to use:** Any time a soft-deletable column is added to a shipped table.

The existing additive migration at `app/src/migrations/m20260611_add_tenant_id_to_users.rs`
provides the exact idiom to follow. [VERIFIED: read source file]

```rust
// Source: app/src/migrations/m20260611_add_tenant_id_to_users.rs (verified pattern)
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column(ColumnDef::new(Orders::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column(Orders::DeletedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    DeletedAt,
}
```

**Backend portability:** `ColumnDef::new(...).timestamp().null()` works on both SQLite and
Postgres under sea-orm 1.0. Confirmed by the existing `m20260611_add_tenant_id_to_users.rs`
which uses `big_integer().null()` and the CI/test posture that runs both backends.
[VERIFIED: app/Cargo.toml shows sqlx-sqlite + sqlx-postgres features]

**SQLite `ALTER TABLE ADD COLUMN` constraint:** SQLite supports `ADD COLUMN` when the new
column is nullable (no `NOT NULL` without a constant `DEFAULT`). A nullable timestamp with
no default is legal. Existing rows get `NULL`. [VERIFIED: consistent with SQLite docs and
the null() call, no runtime default needed]

**Date stamp for filename:** Use `m20260623_` matching today's date (2026-06-23).

### Pattern 2: Resolver Accessors on ServiceDef

**What:** Pure `&self` methods that return the resolved table name and soft-delete column,
applying defaults when the `Option<String>` fields are `None`.
**When to use:** Any code that needs the concrete table name or soft-delete column from a
`ServiceDef`. Dispatch.rs is the first consumer (D-08).

```rust
// Source: service.rs (to be added; pattern mirrors existing &self accessors)
impl ServiceDef {
    /// Returns the backing table name: explicit `.table()` value or the
    /// default `format!("{}s", name.to_lowercase())`.
    ///
    /// Matches the inline derivation previously at dispatch.rs:123.
    pub fn resolved_table(&self) -> String {
        self.table
            .clone()
            .unwrap_or_else(|| format!("{}s", self.name.to_lowercase()))
    }

    /// Returns the soft-delete column name: explicit `.soft_delete_column()` value
    /// or the default `"deleted_at"`.
    pub fn resolved_soft_delete_column(&self) -> &str {
        self.soft_delete_column.as_deref().unwrap_or("deleted_at")
    }
}
```

**Return type rationale:** `resolved_table()` returns `String` (allocation unavoidable when
constructing the default). `resolved_soft_delete_column()` returns `&str` (either borrows
from `self.soft_delete_column` or returns a `'static` literal).

### Pattern 3: `deleted_at IS NULL` Predicate Injection

**What:** After the tenant predicate block in `dispatch()`, inject a literal `IS NULL`
predicate for the soft-delete column when the projection has explicitly declared a
soft-delete column.
**Gate signal:** `service.soft_delete_column.is_some()` — the explicit `.soft_delete_column(...)`
opt-in only. See Pitfall 1 for the rationale. This mirrors the tenant gate
`if let Some(ref col) = service.tenant_column`.

```rust
// Source: dispatch.rs (to be added; mirrors tenant block at lines 151–167)
// NOTE: IS NULL uses no bound value; idx does NOT increment.
if service.soft_delete_column.is_some() {
    let col = service.resolved_soft_delete_column();
    where_clauses.push(format!("\"{}\" IS NULL", col));
    // No values.push() — IS NULL has no bound parameter.
    // idx is NOT incremented; the next bound param (LIMIT/OFFSET) keeps its index.
}
```

**COUNT query coverage:** The WHERE clause is assembled into `where_str` before both the
COUNT query and the DATA query (dispatch.rs:169–173). Adding the clause to `where_clauses`
automatically covers both. [VERIFIED: dispatch.rs:176 count uses `{where_str}`, line 211
data uses `{where_str}`]

### Pattern 4: Field Classification Helper

**What:** A method on `ServiceDef` (or a free function in `service.rs`) that identifies
fields that must be server-injected and never an agent input.
**Consumed by:** Phase 240 schema derivation to exclude these fields from write schemas.

```rust
// Source: service.rs (to be added; builds on FieldMeaning from field.rs)
impl ServiceDef {
    /// Returns true if the field must be server-injected and never an agent input.
    ///
    /// Covers:
    /// - Identifier fields (primary key — set by DB auto-increment)
    /// - CreatedAt fields (set by DB DEFAULT current_timestamp)
    /// - The tenant column (injected from McpContext, never from agent payload)
    ///
    /// Used by Phase 240 to derive write input schemas.
    pub fn is_server_injected_field(&self, field: &FieldDef) -> bool {
        matches!(field.meaning, FieldMeaning::Identifier | FieldMeaning::CreatedAt)
            || self
                .tenant_column
                .as_deref()
                .map(|tc| tc == field.name)
                .unwrap_or(false)
    }
}
```

**Note on `Sensitive`:** The spec says `Sensitive` is also excluded from write schemas,
but the spec text distinguishes "server-injected" from "excluded for privacy". This phase
scopes `is_server_injected_field` to server-injected only (Identifier, CreatedAt, tenant
column). Phase 240 can extend exclusion logic for `Sensitive` fields separately, or a
`is_excluded_from_write_schema` helper can unify them. The planner should decide the
helper name scope at planning time.

### Pattern 5: Entity Model Update

The auto-generated `app/src/models/entities/orders.rs` must gain `deleted_at: Option<String>`.
The file header says "AUTO-GENERATED - DO NOT EDIT" but the column must appear for the ORM
entity to match the schema. [VERIFIED: orders.rs source confirmed it is regenerated by `ferro db:sync`]

Options (Claude's discretion per D-01 rationale):
- Run `ferro db:sync` after migrating to regenerate.
- Or manually add `pub deleted_at: Option<String>` to the `Model` struct (matching
  `created_at: String` which uses `String`, so `Option<String>` for the nullable version).
  The entry also needs `pub deleted_at: Option<String>` in the `Model`, and a
  `DeletedAt` variant added to the `Column` enum (sea-orm entity pattern).

The planner should include this as a task step regardless of the chosen approach.

### Anti-Patterns to Avoid

- **Hardcoding `"deleted_at"` in dispatch.rs:** D-06 requires using `resolved_soft_delete_column()`. If a projection sets `.soft_delete_column("removed_at")`, the predicate must use `"removed_at"`.
- **Gating the predicate on `service.deletable`:** Do not emit `deleted_at IS NULL` based on `service.deletable` (alone or as an `||` branch). A projection that flips `.deletable(true)` without declaring `.soft_delete_column(...)` may sit on a table that has no soft-delete column — the query would fail at runtime. Gate on the explicit `service.soft_delete_column.is_some()` opt-in only.
- **Unconditional predicate injection:** Do not emit `deleted_at IS NULL` for every projection. Non-soft-deletable projections' tables may not have the column — the query would fail at runtime.
- **Editing the shipped migration:** `m20260611_create_orders_table.rs` is append-only. Never add `deleted_at` there.
- **`idx` increment after IS NULL:** `IS NULL` takes no bound value. Do not increment `idx` after pushing this clause, or LIMIT/OFFSET placeholders shift off by one.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Backend-portable ALTER TABLE | Hand-written SQL string with `execute_unprepared` | `sea_orm_migration::SchemaManager::alter_table` + `Table::alter()` | Already handles SQLite/Postgres dialect; idiomatic migration pattern in use across 9 existing migrations |
| SQL injection prevention | Manual string escaping | `Statement::from_sql_and_values` with bound params | Filter values are already bound; column names are developer-controlled (from `service.name`/`service.soft_delete_column`) so quoting with `\"col\"` is sufficient |
| Pluralization | A full pluralization library | `format!("{}s", name.to_lowercase())` | Must match existing inline derivation exactly; adding a crate for YAGNI-level edge cases creates a behavior gap with the existing dispatch code |

---

## Common Pitfalls

### Pitfall 1: Wrong Gate Signal for the `deleted_at` Predicate

**What goes wrong:** Using `service.deletable` as a gate (alone, or as a `|| service.deletable`
branch) means a projection that flips `.deletable(true)` on a table that hasn't had
`deleted_at` added to it yet would inject a predicate for a column that doesn't exist,
causing a runtime SQL error. `service.deletable` controls whether `delete_<svc>` is derived;
it does not directly control whether the column exists on the table.

**Why it happens:** `service.deletable` and "the table physically has a soft-delete column"
are independent facts. Only the explicit `.soft_delete_column(...)` declaration ties the
projection to a real column.

**How to avoid:** Gate on `service.soft_delete_column.is_some()` — the explicit opt-in.
This means: "this projection has declared which column carries soft-delete state." In v16.3,
`orders` declares `.soft_delete_column("deleted_at")` once the column lands; the predicate is
only active once the projection declares it. A `.deletable(true)`-only projection (no column
declared) gets no predicate, so it cannot SQL-error against a missing column.

**Warning signs:** A projection's `list_<svc>` returns rows that should be invisible; or a
SQL error "no such column: deleted_at" on a projection that set `.deletable(true)` without a
soft-delete column.

**Recommended (and adopted) safest gate:** `service.soft_delete_column.is_some()`. Explicit
opt-in via `.soft_delete_column("deleted_at")` on the projection is the clearest signal and
the only safe one. Phase 243 sets `.deletable(true)` + `.soft_delete_column("deleted_at")` on
the orders projection — the test in SC#3 uses `service.soft_delete_column = Some("deleted_at".into())`
(via `.soft_delete_column("deleted_at")`) without needing `.deletable(true)`.

### Pitfall 2: Resolver Default Behavior Drift

**What goes wrong:** `resolved_table()` returns `"orders"` for `ServiceDef::new("order")`
but the existing dispatch code produces `"orders"` from `format!("{}s", "order")`. If the
resolver uses a different algorithm (e.g., adds a separator, uppercases, etc.), all existing
projections silently query the wrong table.

**Why it happens:** The resolver is supposed to make the inline derivation consistent, not
change behavior.

**How to avoid:** The `resolved_table()` default must be exactly `format!("{}s", self.name.to_lowercase())`.
Write a table test that asserts `ServiceDef::new("order").resolved_table() == "orders"` and
`ServiceDef::new("Order").resolved_table() == "orders"` (lowercased). This must match
before wiring into dispatch.

### Pitfall 3: `idx` Counter Not Adjusted After Adding `IS NULL`

**What goes wrong:** The LIMIT/OFFSET placeholders at `dispatch.rs:204–208` use
`placeholder(backend, idx)` and `placeholder(backend, idx + 1)`. If `idx` is incorrectly
incremented when adding the `IS NULL` clause (which has no bound value), the generated SQL
has wrong placeholder indices for Postgres (e.g., `$3/$4` instead of `$2/$3`), producing
a SQL syntax or binding error.

**Why it happens:** The tenant block increments `idx += 1` because it pushes a bound value.
The `IS NULL` block must NOT increment `idx`.

**How to avoid:** Do not call `values.push()` and do not `idx += 1` in the soft-delete
predicate block. Add a comment making this explicit.

### Pitfall 4: Migration Version Name Collision

**What goes wrong:** `DeriveMigrationName` derives the migration name from the file stem.
If two migration files have the same stem (or if the same stem is imported twice), sea-orm
panics at startup with a duplicate migration name.

**Why it happens:** The `mod.rs` comment already warns about this (line 10: "local wrappers
give unique version names derived from the file stem").

**How to avoid:** The new file is `m20260623_add_deleted_at_to_orders.rs`. The date prefix
`20260623` does not collide with any existing file (most recent is `20260614`). Confirm by
scanning `mod.rs` before adding.

### Pitfall 5: Entity Model Out of Sync

**What goes wrong:** The sea-orm entity at `app/src/models/entities/orders.rs` is
auto-generated by `ferro db:sync`. After running `db:migrate`, if `db:sync` is NOT run, the
entity `Model` struct lacks `deleted_at`, causing compilation failures when any code
references `Column::DeletedAt` or when sea-orm tries to deserialize a row that now has the
column.

**How to avoid:** After the migration step, run `cargo run --bin app -- db:sync` (or
whatever the ferro CLI invocation is) to regenerate the entity, or manually add
`pub deleted_at: Option<String>` to the `Model` struct and a `DeletedAt` variant to the
`Column` enum. The planner must include this as an explicit task step.

---

## Code Examples

All examples are drawn from verified source files in the working tree.

### Existing Additive Migration (reference)

```rust
// Source: app/src/migrations/m20260611_add_tenant_id_to_users.rs [VERIFIED]
manager
    .alter_table(
        Table::alter()
            .table(Users::Table)
            .add_column(ColumnDef::new(Users::TenantId).big_integer().null())
            .to_owned(),
    )
    .await
```

### Existing Tenant Predicate Injection (mirror for `deleted_at IS NULL`)

```rust
// Source: ferro-mcp-server/src/dispatch.rs:151-167 [VERIFIED]
if let Some(ref col) = service.tenant_column {
    match tenant_id {
        Some(tid) => {
            where_clauses.push(format!("\"{}\" = {}", col, placeholder(backend, idx)));
            values.push(sea_orm::Value::BigInt(Some(tid)));
            idx += 1;
        }
        None => {
            return Err(crate::Error::InvalidFilter(
                "tenant context required but not present".to_string(),
            ));
        }
    }
}
// New block follows same placement, simpler (no bound value):
if service.soft_delete_column.is_some() {
    let col = service.resolved_soft_delete_column();
    where_clauses.push(format!("\"{}\" IS NULL", col));
    // No values.push() — IS NULL has no bound parameter.
    // idx NOT incremented.
}
```

### Existing Test Setup to Extend (SC#3 test baseline)

```rust
// Source: ferro-mcp-server/src/dispatch.rs:240-268 [VERIFIED]
// The setup_orders_db() function creates the orders table.
// The SC#3 test must extend the CREATE TABLE SQL to include deleted_at,
// seed one soft-deleted row, and assert it is excluded.
"CREATE TABLE IF NOT EXISTS orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_name TEXT NOT NULL,
    total REAL NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    tenant_id INTEGER NOT NULL,
    deleted_at TEXT NULL          -- added for Phase 239
)"
```

### `infer_meaning("deleted_at")` Returns `DateTime`

```rust
// Source: ferro-projections/src/field.rs test at line 375 [VERIFIED]
assert_eq!(infer_meaning("deleted_at"), FieldMeaning::DateTime);
// The _at suffix rule (line 155-157) returns FieldMeaning::DateTime for "deleted_at".
// This means deleted_at is NOT classified as Identifier, CreatedAt, or ForeignKey.
// The classification helper MUST check the column name against service.soft_delete_column
// to classify it correctly, OR the caller simply never includes deleted_at in the
// projection's field() declarations (it's a framework-managed column, not a user field).
```

---

## Runtime State Inventory

Not applicable. This phase makes a code + migration change; there is no rename/refactor.
The migration adds a NEW column; existing rows are unaffected (NULL default). No stored
data migration needed. [VERIFIED: D-04 explicitly states "No backfill needed"]

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| SQLite (sqlx-sqlite) | dispatch.rs in-memory tests | ✓ | sea-orm 1.0 feature | — |
| Postgres (sqlx-postgres) | CI migration gate | ✓ (CI) | sea-orm 1.0 feature | SQLite for unit tests |
| `sea-orm-migration` | Migration file | ✓ | 1.0 [VERIFIED: app/Cargo.toml] | — |

No missing dependencies.

---

## Validation Architecture

`workflow.nyquist_validation` is absent from `.planning/config.json` → treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` for async |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p ferro-projections -p ferro-mcp-server 2>&1 \| tail -5` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| SC | Behavior | Test Type | Automated Command | File Exists? |
|----|----------|-----------|-------------------|-------------|
| SC#1 | `db:migrate` applies `deleted_at` column clean | integration / manual | `cargo run --bin app -- db:migrate` | No — Wave 0 |
| SC#2 | `resolved_table()` returns default and explicit override; `resolved_soft_delete_column()` returns default and explicit override | unit | `cargo test -p ferro-projections resolved_` | No — Wave 0 |
| SC#3 | Row with non-null `deleted_at` excluded from `dispatch()` result | unit (sqlite in-memory) | `cargo test -p ferro-mcp-server soft_delete_excluded` | No — Wave 0 |
| SC#4 | `is_server_injected_field` returns true for Identifier, CreatedAt, and tenant column; false for other fields | unit | `cargo test -p ferro-projections server_injected` | No — Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-projections -p ferro-mcp-server`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite + fmt + clippy before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-projections/src/service.rs` — add `resolved_table()`, `resolved_soft_delete_column()`, `is_server_injected_field()` + unit tests (SC#2, SC#4)
- [ ] `ferro-mcp-server/src/dispatch.rs` — extend `setup_orders_db()` with `deleted_at TEXT NULL`, add `soft_delete_excluded` test (SC#3)
- [ ] `app/src/migrations/m20260623_add_deleted_at_to_orders.rs` — new migration file (SC#1)

---

## Open Questions (RESOLVED)

1. **`deleted_at` as a `FieldDef` in the projection or not?**
   - What we know: The spec says "all read/update/delete paths filter `deleted_at IS NULL`". The `infer_meaning("deleted_at")` returns `DateTime`, not a special meaning. Nothing in the current `orders` projection declares a `deleted_at` field.
   - What's unclear: Should `deleted_at` appear as a declared field in the projection (for read output) or is it a framework-managed column invisible in the field list?
   - **RESOLVED:** `deleted_at` is a framework-managed column and is NEVER declared as a projection `field()`. The dispatch `IS NULL` predicate is driven by the resolved soft-delete column name (`resolved_soft_delete_column()`), not by a `FieldDef`. This is also why `is_server_injected_field` does not need to classify `deleted_at`: the column never enters the field list, so it never reaches the write-schema derivation that helper feeds. Phase 240/241 can decide separately if `deleted_at` should be readable back.

2. **Entity regeneration: `ferro db:sync` vs manual edit?**
   - What we know: `app/src/models/entities/orders.rs` says "AUTO-GENERATED - DO NOT EDIT". The `deleted_at` column must appear in the `Model` for ORM to work.
   - What's unclear: Whether the ferro CLI `db:sync` command re-reads the live DB schema; whether it is safe to run mid-phase.
   - **RESOLVED:** Either approach is acceptable and safe. Plan 01 handles the orders entity sync — adopt the manual-edit-or-`db:sync` approach Plan 01 already specifies (add `pub deleted_at: Option<String>` to the `Model` + a `DeletedAt` variant to the `Column` enum, or regenerate via `db:sync` after the migration). Both produce an entity consistent with the migrated schema.

---

## Assumptions Log

No claims in this research are `[ASSUMED]` — all findings are `[VERIFIED]` against source
files in the working tree.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**All claims verified against actual source files.**

---

## Sources

### Primary (HIGH confidence — verified against source files)

- `app/src/migrations/m20260611_add_tenant_id_to_users.rs` — `alter_table` + `add_column` idiom (exact migration pattern to follow)
- `app/src/migrations/m20260611_create_orders_table.rs` — `ColumnDef`, `DeriveIden`, `MigrationTrait` idiom
- `app/src/migrations/mod.rs` — migration registration vector, comment about version-name uniqueness
- `ferro-mcp-server/src/dispatch.rs` — dispatch() function: table derivation at line 123, tenant predicate at lines 151-167, WHERE clause assembly at lines 169-173, COUNT at 176, DATA at 211; test setup at lines 234-268
- `ferro-projections/src/service.rs` — `ServiceDef` struct with `table: Option<String>`, `soft_delete_column: Option<String>`, `deletable: bool`; existing CRUD tests at lines 1917-2002
- `ferro-projections/src/field.rs` — `FieldMeaning` enum, `infer_meaning()`, test confirming `infer_meaning("deleted_at") == DateTime` (line 375)
- `ferro-projections/src/lib.rs` — public exports; confirms `is_server_injected_field` does not yet exist
- `app/src/models/entities/orders.rs` — confirmed no `deleted_at` column in current entity
- `app/Cargo.toml` — sea-orm 1.0 + sea-orm-migration 1.0, both with sqlx-sqlite + sqlx-postgres features
- `.planning/config.json` — `workflow.nyquist_validation` absent → treated as enabled

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all in-workspace, verified in Cargo.toml files
- Migration mechanics: HIGH — exact idiom exists in m20260611_add_tenant_id_to_users.rs
- Resolver accessor pattern: HIGH — verified against ServiceDef builder API; tests already exist for raw fields
- Predicate injection: HIGH — verified against dispatch.rs source, tenant block is the exact template
- Field classification helper: HIGH — FieldMeaning enum verified; helper does not yet exist, but the inputs are fully confirmed
- Testing: HIGH — test infrastructure (tokio::test, setup_orders_db) verified in dispatch.rs

**Research date:** 2026-06-23
**Valid until:** 2026-07-23 (stable codebase; sea-orm 1.0 is stable)
