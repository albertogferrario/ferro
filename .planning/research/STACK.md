# Stack Research — v12.4 Form Validation DX

**Domain:** Async DB-backed validation + DB-constraint-to-field-error mapping for ferro
**Researched:** 2026-06-09
**Confidence:** HIGH (versions verified from Cargo.toml + cargo tree; SeaORM API verified via Context7 + docs.rs)

---

## Existing Versions (verified — do not change)

| Package | Version in use | Source |
|---------|---------------|--------|
| `sea-orm` | 1.1.19 | `framework/Cargo.toml:51` + `cargo tree` |
| `sqlx` (transitive via sea-orm) | 0.8.6 | `cargo tree` |
| sea-orm features enabled | `sqlx-postgres`, `sqlx-sqlite`, `runtime-tokio-native-tls`, `macros` | `framework/Cargo.toml:51` |

No version bumps required. Both target features are fully implementable against the currently pinned stack.

---

## Feature A: Async DB-backed `unique` validation rule

### The constraint: current `Rule` trait is synchronous

`framework/src/validation/rule.rs`:

```rust
pub trait Rule: Send + Sync {
    fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String>;
    fn name(&self) -> &'static str;
}
```

`Validator::validate()` drives rules with a plain `for` loop — no `.await`, no async executor context. This is intentional and correct for the 22 existing sync rules.

### Decision: separate `AsyncRule` trait and `AsyncValidator`, not async `Rule`

Making `Rule` async is the wrong path:
- `async fn validate(...)` is not object-safe on stable Rust 2021 without nightly `dyn-async-traits`
- Using `async-trait` (box-allocates every rule future) forces all 22 existing sync rules to be re-wrapped and adds a proc-macro dependency for no benefit to sync rules
- RPITIT (`-> impl Future`) breaks `Box<dyn Rule>` and the `rules![...]` macro

The correct design is a parallel `AsyncRule` trait (stable, no new deps) and an `AsyncValidator` that runs async rules in sequence inside an async context. The two paths share `ValidationError` as output type.

### `AsyncRule` trait — new, zero new dependencies

```rust
// framework/src/validation/async_rule.rs  (new file)

use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub trait AsyncRule: Send + Sync {
    fn validate<'a>(
        &'a self,
        field: &'a str,
        value: &'a Value,
        data: &'a Value,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

    fn name(&self) -> &'static str;
}
```

`Pin<Box<dyn Future>>` is the only object-safe async return on stable Rust 2021 edition. `std::future::Future` and `std::pin::Pin` are in `std` — no new crate.

### `unique` rule — integration with SeaORM

The `Unique` struct takes a `sea_orm::DatabaseConnection` at construction time (passed in by the handler, not a global). It runs a parameterised `SELECT COUNT(*)` using `sea_orm::Statement::from_sql_and_values` — this form is backend-agnostic (SQLite and Postgres both accept the same `?`/`$1` placeholder, switched by sea-orm's `DbBackend`).

```rust
// framework/src/validation/rules/unique.rs  (new file)

pub struct Unique {
    db: sea_orm::DatabaseConnection,
    table: String,
    column: String,
    except: Option<(String, i64)>,   // (pk_column, id_to_exclude)
}

pub fn unique(
    db: sea_orm::DatabaseConnection,
    table: impl Into<String>,
    column: impl Into<String>,
) -> Unique { ... }

impl Unique {
    /// Exclude an existing row from the uniqueness check (edit-form pattern).
    pub fn ignore_self(mut self, pk_column: impl Into<String>, id: i64) -> Self {
        self.except = Some((pk_column.into(), id));
        self
    }
}
```

No ORM entity type is needed. Raw-SQL avoids requiring callers to pass generic entity type parameters (which would make the rule non-ergonomic in a `dyn AsyncRule` context).

### Target handler API

```rust
#[action]
pub async fn store(req: Request, db: DatabaseConnection) -> ActionResult {
    let data = req.input::<serde_json::Value>().await?;

    // Sync rules — no change to existing pattern
    Validator::new(&data)
        .rules("name", rules![required(), string(), max(255)])
        .validate()
        .map_err(|e| e.with_old_input(&data).into_action_error("/items/new"))?;

    // Async rules — new AsyncValidator, same ValidationError output
    AsyncValidator::new(&data, &db)
        .rule("slug", unique(db.clone(), "items", "slug"))
        .validate()
        .await
        .map_err(|e| e.with_old_input(&data).into_action_error("/items/new"))?;

    // ... insert
    Ok(())
}

// Edit-form variant
AsyncValidator::new(&data, &db)
    .rule("slug", unique(db.clone(), "items", "slug").ignore_self("id", item.id))
    .validate()
    .await
    ...
```

Two-step call (sync then async) is explicit. Combining sync and async rules in a single `AsyncValidator` (with sync rules adapted via a wrapper) is also valid and produces a single chain — design choice for the implementation phase.

---

## Feature B: DB-constraint violation → field-level error mapping

### SeaORM's `DbErr::sql_err()` — verified current API (sea-orm 1.1.19)

```rust
// sea_orm::error — verified via Context7 (/websites/rs_sea-orm_1_1_14)

impl DbErr {
    /// Portable UNIQUE/FK constraint detection across all supported backends.
    pub fn sql_err(&self) -> Option<SqlErr>;
}

#[non_exhaustive]
pub enum SqlErr {
    UniqueConstraintViolation(String),       // carries e.message()
    ForeignKeyConstraintViolation(String),
}
```

This is the correct and only portable detection point. SeaORM's `sql_err()` performs the backend dispatch internally:

| Backend | Error codes handled | Source |
|---------|-------------------|--------|
| Postgres | SQLSTATE `23505` | sea-orm source, verified via Context7 |
| SQLite | Extended result codes `1555` (PK conflict), `2067` (UNIQUE index conflict) | sea-orm source, verified via Context7 |
| MySQL | Numbers 1022, 1062, 1169, 1586 | sea-orm source (not in scope — ferro does not use MySQL) |

The `String` payload in `UniqueConstraintViolation` is the raw `e.message()` from the underlying sqlx driver — **not** parsed further by SeaORM.

### What the message string contains (per backend — HIGH confidence)

**SQLite:** `"UNIQUE constraint failed: table_name.column_name"` — the table and column are embedded in the message. Substring match on `"table.column"` is reliable.

**Postgres:** Short human-readable message, e.g., `"duplicate key value violates unique constraint \"idx_name\""`. The constraint/index name appears in the message. The constraint name is also available via `PgDatabaseError::constraint() -> Option<&str>` (sqlx 0.8 `DatabaseError` trait), but `sql_err()` discards it — only the message string is carried into `SqlErr::UniqueConstraintViolation(String)`.

**Implication:** `sql_err()` is sufficient for detection but does not provide column identity directly. Field mapping must be caller-supplied, not automatically inferred.

### Why NOT to downcast to `PgDatabaseError` for constraint name

To extract `PgDatabaseError::constraint()`, a caller would need to pattern-match through `DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))`, then downcast with `e.try_downcast_ref::<sqlx::postgres::PgDatabaseError>()`. This:
- Requires adding `sqlx` as a direct dependency in `framework` (currently only transitive)
- Is Postgres-only (breaks SQLite portability)
- Duplicates the backend dispatch already done by `sql_err()`

SeaORM's `sql_err()` + caller-supplied hint strings is the portable design.

### Recommended approach: `map_unique_violation` free function in `framework::validation`

```rust
// framework/src/validation/mod.rs or a new framework/src/validation/constraint.rs

/// Map a `DbErr` UNIQUE constraint violation to a named field's ValidationError
/// and redirect, if the constraint detail matches a caller-supplied hint.
///
/// `mappings`: `&[("hint_substring", "field_name")]`
///
/// Match: substring of `SqlErr::UniqueConstraintViolation(detail)`.
/// - SQLite: hint = `"table_name.column_name"` (embedded verbatim in message)
/// - Postgres: hint = constraint/index name (embedded in message after `\"`)
///
/// Returns `Err(ActionError)` wrapping a per-field ValidationError on match,
/// or falls through to `Err(ActionError::from(err))` (raw message) on no match.
pub fn map_unique_violation(
    err: sea_orm::DbErr,
    mappings: &[(&str, &str)],
    message: impl Into<String>,
    redirect_url: impl Into<String>,
    form_data: &serde_json::Value,
) -> Result<(), crate::http::action::ActionError> {
    use sea_orm::SqlErr;
    if let Some(SqlErr::UniqueConstraintViolation(detail)) = err.sql_err() {
        let msg = message.into();
        let url = redirect_url.into();
        for (hint, field) in mappings {
            if detail.contains(hint) {
                let mut ve = ValidationError::new();
                ve.add(field, msg);
                return Err(ve.with_old_input(form_data).into_action_error(url));
            }
        }
    }
    Err(crate::http::action::ActionError::from(err))
}
```

Consumer call:

```rust
MyEntity::insert(model)
    .exec(&db)
    .await
    .map_err(|e| map_unique_violation(
        e,
        &[("items.slug", "slug"), ("items.name", "name")],
        "This slug is already taken.",
        "/items/new",
        &data,
    ))?;
```

### Why NOT to modify `From<sea_orm::DbErr> for ActionError` (action.rs:196)

The current passthrough (`Self::msg(err.to_string())`) is correct for all `DbErr` variants that are not UNIQUE violations — connection failures, type errors, query errors, etc. Changing the `From` impl would silently alter behavior for all consumers of `?` on `DbErr`. `map_unique_violation` is an explicit opt-in, called only in handlers that know a UNIQUE constraint can fire.

---

## New Files and Change Locations

| File | Change type | What |
|------|-------------|------|
| `framework/src/validation/async_rule.rs` | New | `AsyncRule` trait |
| `framework/src/validation/rules/unique.rs` | New | `Unique` struct + `unique()` constructor |
| `framework/src/validation/async_validator.rs` | New | `AsyncValidator` struct |
| `framework/src/validation/constraint.rs` | New (or inline in `mod.rs`) | `map_unique_violation` free function |
| `framework/src/validation/mod.rs` | Edit | Re-export new public items |
| `framework/src/http/action.rs` | No change | `From<DbErr>` passthrough stays as-is |

No changes to `ferro-orm`, `ferro-macros`, or any crate outside `framework`.

---

## What NOT to Add

| Do not add | Why |
|------------|-----|
| `async-trait` crate | `Pin<Box<dyn Future>>` in `AsyncRule` achieves object-safe async on stable Rust without a proc-macro dep |
| Direct `sqlx` dep in `framework` | `sqlx` is already transitive via `sea-orm`; adding it directly just to call `PgDatabaseError::constraint()` sacrifices backend portability for no gain |
| `validator 0.20` extension points | Already in `Cargo.toml` but unused by ferro's validation module; adopting its derive-macro API would conflict with ferro's builder-style API |
| Automatic constraint→column inference | No portable API exists across SQLite+Postgres without backend-specific downcast; caller-supplied hint strings are the correct design |
| Global DB connection registry for rules | Rules receive the connection at construction time; a global couples the validation layer to framework internals and is not testable in isolation |
| Async `Rule` trait (making existing `Rule` async) | Breaks object safety on stable; forces wrapping all 22 sync rules; correct design is a parallel async trait |
| New crate for async validation | `std::future::Future` + `std::pin::Pin` cover the requirement; no external dependency justified |

---

## Constraint Detection Reference (portable, HIGH confidence)

| Backend | Detection method | Codes | Constraint name? | Column name? |
|---------|-----------------|-------|-----------------|--------------|
| Postgres | `DbErr::sql_err()` → `SqlErr::UniqueConstraintViolation(msg)` | SQLSTATE `23505` | In message string (index/constraint name) | No — not in standard message |
| SQLite | `DbErr::sql_err()` → `SqlErr::UniqueConstraintViolation(msg)` | `1555`, `2067` | No | Yes — `"table.column"` embedded in message |

**Canonical detection call:** `err.sql_err()` — never pattern-match on `DbErr::Exec` / `DbErr::Query` directly. `sql_err()` encapsulates the backend dispatch and is maintained by SeaORM across backend changes.

---

## Sources

- Context7 `/websites/rs_sea-orm_1_1_14` — `DbErr::sql_err()` full source, `SqlErr` enum definition, per-backend error-code dispatch table. HIGH confidence.
- `https://docs.rs/sqlx/0.8.6/sqlx/postgres/struct.PgDatabaseError.html` — `constraint() -> Option<&str>` confirmed; Postgres-only, not on SQLite driver. HIGH confidence.
- `https://docs.rs/sqlx/0.8.6/sqlx/error/trait.DatabaseError.html` — `DatabaseError` trait methods; confirmed no `column()` method exists on the trait. HIGH confidence.
- `framework/Cargo.toml` + `cargo tree` output — sea-orm 1.1.19, sqlx 0.8.6 pinning verified directly in the repo. HIGH confidence.

---
*Stack research for: ferro v12.4 Form Validation DX*
*Researched: 2026-06-09*
