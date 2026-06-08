# Phase 190: Async Rule Infrastructure + `unique` Rule — Research

**Researched:** 2026-06-09
**Domain:** Rust async trait objects, SeaORM raw SQL, ferro validation extension
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use `async-trait = "0.1"` (already in `framework/Cargo.toml:30`) for the `AsyncRule` trait. Trait shape mirrors `Rule`: `async fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String>` plus `fn name(&self) -> &'static str`. Stored as `Box<dyn AsyncRule>`.
- **D-02:** `AsyncValidator` holds both sync rules (`Box<dyn Rule>`) and async rules (`Box<dyn AsyncRule>`). Single validator, not two.
- **D-03:** `validate_async().await` runs all sync rules first, then async rules only on fields with no sync error (fail-fast). Locked by success criterion 3.
- **D-04:** `validate_async` returns the existing `ValidationError` on validation failure. No new error-surfacing path.
- **D-05 (Claude's discretion):** Exact constructor/`validate_async` signature — see Section "AsyncValidator Signature Resolution" below for recommendation.
- **D-06:** `unique(table, column)` uses parameterized `SELECT COUNT(*) FROM <table> WHERE <column> = ?`. Backend detected via `get_database_backend()`. Per-backend quoting.
- **D-07:** DB access inside `Unique` via `DB::connection()` singleton. No connection threaded through the `AsyncRule` signature.
- **D-08:** Table/column are developer-controlled identifiers. Interpolated (cannot be SQL-bound). Guarded by `[A-Za-z0-9_]` regex + double-quote wrapping.
- **D-09:** `.ignore(id)` accepts `impl Into<sea_orm::Value>`. Adds `AND <pk> <> ?` with id bound as parameter. Default PK column `"id"`.
- **D-10 (Claude's discretion):** Non-default PK column via an explicit form (e.g. `.ignore_on(pk_col, id)`). Happy-path default-`"id"` is mandatory.
- **D-11:** Default message via `validation.unique` translation key, `("attribute", field)` param, English fallback `"The {field} has already been taken."`.
- **D-12 (Claude's discretion):** DB/infra failure during an async rule propagates as a framework error (handler → 500), never as a validation result. Planner picks the concrete `Result` type.

### Claude's Discretion
- Precise `AsyncValidator` constructor / `validate_async` signature (D-05).
- Exact spelling of the non-default-PK exclude-self API (D-10).
- Concrete `Result` type encoding the validation-vs-infra distinction (D-12).
- File split within `framework/src/validation/` (new files: `async_rule.rs`, `async_validator.rs`, `rules_async.rs`).

### Deferred Ideas (OUT OF SCOPE)
- Scoped/conditional uniqueness (`.where_eq(col, val)` for per-tenant uniqueness) — see "Gestiscilo Tenancy" note below.
- Additional async rules (`exists`, `custom_async`).
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| VALID-01 | Developer can validate field uniqueness via `unique(table, column)` before insert/update, with a field-level error message | D-06 query pattern + SeaORM `Statement::from_sql_and_values` + `query_one` verified against sea-orm 1.1.19 source |
| VALID-02 | Developer can exclude current record from uniqueness check on edit forms via `.ignore(id)` | D-09 `impl Into<sea_orm::Value>` pattern; `sea_orm::Value` confirmed to accept `i64`, `String`, `&str`, `Uuid` |
| VALID-03 | Async rules run through `AsyncValidator`/`validate_async`, leave sync `Validator` untouched, use `DB::connection()` singleton, surface failures through existing `ValidationError` → `with_old_input` → 303 flow | D-01 through D-07; `DB::connection()` verified at `database/mod.rs:171`; `ValidationError` flash flow verified in `error.rs` |
</phase_requirements>

---

## Summary

Phase 190 introduces a DB-backed async validation path to `framework/src/validation/` as a parallel sibling of the existing synchronous `Validator`. The design is purely additive: the sync API is untouched, and all new components live in new files within the same module.

The three new components are:
1. **`AsyncRule` trait** — the async counterpart to `Rule`, using `#[async_trait]` for dyn-compatibility.
2. **`Unique` rule** — a parameterized `SELECT COUNT(*)` over SeaORM's raw statement API, with `.ignore()` exclude-self.
3. **`AsyncValidator`** — a builder that holds both sync and async rules, runs them in the correct order, and returns the existing `ValidationError` on failure or a `FrameworkError` on infra failure.

The single technically uncertain area is the SeaORM raw statement API: `Statement::from_sql_and_values` does NOT translate placeholder syntax between backends. The caller must branch on `DbBackend::Postgres` (uses `$1`, `$2`, ...) vs. everything else (uses `?`). This is a locked implementation requirement confirmed from sea-orm 1.1.19 source code and gestiscilo-it field usage.

**Primary recommendation:** Implement in three new files (`async_rule.rs`, `async_validator.rs`, `rules_async.rs`), all within `framework/src/validation/`. `validate_async` takes `&data` (not `&req`) to avoid any concern about double-consuming the request body.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `AsyncRule` trait definition | validation module (framework) | — | Mirrors existing `Rule` trait; lives at the same level |
| `Unique` rule (DB query) | validation module (framework) | Database layer (singleton) | Rule owns its query; gets connection from `DB::connection()` — no threading |
| `AsyncValidator` orchestration | validation module (framework) | — | Parallel to `Validator`; same builder pattern, different run loop |
| DB access | Database facade (`DB::connection()`) | — | Singleton already wired; no new DI |
| Error surfacing (infra) | `FrameworkError` | `ActionError` (via `From`) | DB error → `FrameworkError::Database` → handler returns 500 |
| Error surfacing (validation) | `ValidationError` | flash session | Same path as sync validation; `with_old_input` → `redirect_back` |
| Translation key registration | `ferro-lang` consumer files | English fallback in `bridge.rs` pattern | Framework ships fallback only; consumer files provide locale |

---

## Standard Stack

### Core (already present — no new deps)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `async-trait` | `0.1` | `AsyncRule` trait dyn-compatibility | Already in `framework/Cargo.toml:30`; Rust stable async fn in traits is not dyn-compatible [VERIFIED: Cargo.toml] |
| `sea-orm` | `1.1.19` | Raw `SELECT COUNT(*)` query | Already present; `Statement::from_sql_and_values` + `query_one` confirmed [VERIFIED: cargo tree] |
| `serde_json::Value` | `1` | Field value type in `AsyncRule::validate` | Mirrors existing `Rule` trait signature [VERIFIED: rule.rs] |

No new dependencies for Phase 190.

**Version verification:** [VERIFIED: `cargo tree -p ferro-rs --depth 1`]
- `sea-orm v1.1.19` (workspace uses this)
- `async-trait = "0.1"` (in Cargo.toml at line 30)

---

## Architecture Patterns

### System Architecture Diagram

```
Handler (POST)
    │
    ├─ req.input::<Value>().await     ← consumes request body once
    │      │
    │      └─ data: serde_json::Value
    │
    └─ AsyncValidator::new(&data)
           .rule("slug", required())       ← sync rules
           .async_rule("slug", unique("pages", "slug").ignore(record_id))
           .validate_async()
           .await
              │
              ├─ [sync rules run first, all fields]
              │      │
              │      └─ ValidationError (if any sync failures)
              │             │
              │             └─ .with_old_input(&data).redirect_back(referer)  ← 303
              │
              ├─ [async rules run only on fields with no sync error]
              │      │
              │      ├─ DB::connection()? ────────────► DB singleton
              │      │      │
              │      │      └─ query_one(Statement)    ← count query
              │      │             │
              │      │             ├─ DbErr ──────────► Err(FrameworkError::Database)
              │      │             │                        │
              │      │             │                        └─ handler → 500
              │      │             │
              │      │             └─ count > 0 ──────► ValidationError (field error)
              │      │
              │      └─ ValidationError (if any async failures)
              │             │
              │             └─ .with_old_input(&data).redirect_back(referer)  ← 303
              │
              └─ Ok(()) ──────────────────────────────► handler proceeds to DB write
```

### Recommended Project Structure

New files only — no existing files modified except `mod.rs` (exports):

```
framework/src/validation/
├── mod.rs              # Add: pub use async_rule::AsyncRule;
│                       #      pub use async_validator::AsyncValidator;
│                       #      pub use rules_async::unique;
├── async_rule.rs       # NEW: AsyncRule trait
├── async_validator.rs  # NEW: AsyncValidator struct + validate_async()
├── rules_async.rs      # NEW: Unique rule struct + unique() constructor
├── bridge.rs           # UNCHANGED
├── error.rs            # UNCHANGED
├── rule.rs             # UNCHANGED
├── rules.rs            # UNCHANGED
├── validatable.rs      # UNCHANGED
└── validator.rs        # UNCHANGED
```

framework/src/lib.rs: add `AsyncValidator` and `unique` to the public re-exports.

### Pattern 1: AsyncRule Trait Definition

```rust
// Source: async-trait 0.1 docs + existing TenantLookup pattern in tenant/lookup.rs
use async_trait::async_trait;
use serde_json::Value;

/// An async validation rule for DB-backed or I/O-bound checks.
///
/// Implement this trait for rules that require async I/O (e.g. DB lookups).
/// Use `#[async_trait]` because stable Rust async fn in traits is not
/// dyn-compatible — `Box<dyn AsyncRule>` requires the `async_trait` transform.
///
/// # Safety
/// `Send + Sync` bounds are required: boxed trait objects are shared across
/// async request tasks.
#[async_trait]
pub trait AsyncRule: Send + Sync {
    /// Validate the field value. Returns `Ok(())` on pass, `Err(message)` on
    /// validation failure. Returns `Err` from this method ONLY for validation
    /// failures, never for I/O errors — I/O errors must surface as
    /// `FrameworkError` via the validator's outer `Result`.
    async fn validate(
        &self,
        field: &str,
        value: &Value,
        data: &Value,
    ) -> Result<(), String>;

    /// Rule name (used for custom-message lookup, e.g. `"unique"`).
    fn name(&self) -> &'static str;
}
```

Note: `async fn validate` inside `#[async_trait]` compiles to a boxed future — this is the established pattern in the codebase (see `tenant/lookup.rs:19-28`). [VERIFIED: framework/src/tenant/lookup.rs]

### Pattern 2: Unique Rule — Parameterized COUNT Query

**Critical implementation detail** [VERIFIED: sea-orm 1.1.19 source `src/database/statement.rs` + gestiscilo-it comment at `m20260529_108_add_slug_to_products.rs:112`]:

`Statement::from_sql_and_values` does NOT translate placeholder syntax. It stores the SQL as-is. The sqlx SQLite driver sends `stmt.sql` verbatim to sqlx (`sqlx::query_with(&stmt.sql, ...)`). Therefore:
- **SQLite/MySQL:** must use `?` placeholders
- **Postgres:** must use `$1`, `$2`, ... positional placeholders

The pattern is a per-backend `match` on `get_database_backend()`:

```rust
// Source: gestiscilo-it/app/src/models/bookings.rs:1999-2009 (verified pattern)
// Source: sea-orm 1.1.19 src/database/db_connection.rs:173-187
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use crate::database::DB;
use crate::error::FrameworkError;
use crate::validation::translate_validation;

/// Validates that a field value is unique in the given table column.
///
/// # Trust boundary
/// `table` and `col` are developer-controlled identifiers from handler code,
/// never end-user input. They are validated against `[A-Za-z0-9_]` and
/// double-quoted but still assumed to come from trusted source code.
pub struct Unique {
    table: String,
    col: String,
    ignore: Option<(String, sea_orm::Value)>, // (pk_col, pk_value)
}

pub fn unique(table: impl Into<String>, col: impl Into<String>) -> Unique {
    Unique {
        table: table.into(),
        col: col.into(),
        ignore: None,
    }
}

impl Unique {
    /// Exclude the record with this PK value from the uniqueness check.
    /// Uses default PK column "id".
    pub fn ignore(mut self, id: impl Into<sea_orm::Value>) -> Self {
        self.ignore = Some(("id".to_string(), id.into()));
        self
    }

    /// Exclude the record using an explicit PK column name.
    pub fn ignore_on(
        mut self,
        pk_col: impl Into<String>,
        id: impl Into<sea_orm::Value>,
    ) -> Self {
        self.ignore = Some((pk_col.into(), id.into()));
        self
    }

    fn validate_identifier(ident: &str) -> Result<(), String> {
        let valid = ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if valid && !ident.is_empty() {
            Ok(())
        } else {
            Err(format!("Invalid SQL identifier: {ident:?}"))
        }
    }

    fn quote_ident(ident: &str) -> String {
        format!("\"{ident}\"")
    }
}

#[async_trait]
impl AsyncRule for Unique {
    async fn validate(
        &self,
        field: &str,
        value: &Value,
        _data: &Value,
    ) -> Result<(), String> {
        // Developer trust-boundary guard
        Self::validate_identifier(&self.table)
            .map_err(|e| format!("Unique rule misconfigured: {e}"))?;
        Self::validate_identifier(&self.col)
            .map_err(|e| format!("Unique rule misconfigured: {e}"))?;
        if let Some((ref pk_col, _)) = self.ignore {
            Self::validate_identifier(pk_col)
                .map_err(|e| format!("Unique rule misconfigured: {e}"))?;
        }

        let table = Self::quote_ident(&self.table);
        let col = Self::quote_ident(&self.col);

        // DB access — any I/O error propagates up as FrameworkError, not as
        // a validation error. See AsyncValidator::validate_async for how this
        // is carried in the outer Result.
        let db = DB::connection()
            .map_err(|e| /* NOT a validation message */
                // This Err(String) is a sentinel that AsyncValidator uses to
                // re-raise as FrameworkError. See D-12 pattern in AsyncValidator.
                format!("__infra_error__: {e}")
            )?;

        let backend = db.get_database_backend();
        let (sql, values): (String, Vec<sea_orm::Value>) = match &self.ignore {
            None => {
                let sql = match backend {
                    DatabaseBackend::Postgres =>
                        format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = $1"),
                    _ =>
                        format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = ?"),
                };
                let val = json_value_to_sea_value(value);
                (sql, vec![val])
            }
            Some((pk_col, pk_val)) => {
                let pk = Self::quote_ident(pk_col);
                let sql = match backend {
                    DatabaseBackend::Postgres =>
                        format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = $1 AND {pk} <> $2"),
                    _ =>
                        format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = ? AND {pk} <> ?"),
                };
                let val = json_value_to_sea_value(value);
                (sql, vec![val, pk_val.clone()])
            }
        };

        let stmt = Statement::from_sql_and_values(backend, sql, values);
        let row = db
            .query_one(stmt)
            .await
            .map_err(|e| format!("__infra_error__: {e}"))?;

        let count: i64 = row
            .and_then(|r| r.try_get::<i64>("", "count").ok())
            .unwrap_or(0);

        if count > 0 {
            Err(
                translate_validation("validation.unique", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {field} has already been taken.")),
            )
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "unique"
    }
}
```

**Note on the `__infra_error__:` sentinel approach:** This is one implementation option for D-12, but see "Result Shape for D-12" below for the recommended cleaner approach using the outer `Result`.

### Pattern 3: AsyncValidator

**Recommendation for D-05 (signature):** `validate_async` takes `&self` with data already in the struct — mirrors `Validator::new(&data)` exactly. Handlers call `req.input().await?` once, then pass `&data` to `AsyncValidator::new(&data)`. This avoids double-consuming the request.

```rust
// Source: mirrors framework/src/validation/validator.rs Validator pattern
use crate::validation::{Rule, ValidationError};
use crate::error::FrameworkError;
use serde_json::Value;
use std::collections::HashMap;

pub struct AsyncValidator<'a> {
    data: &'a Value,
    sync_rules: HashMap<String, Vec<Box<dyn Rule>>>,
    async_rules: HashMap<String, Vec<Box<dyn AsyncRule>>>,
    custom_messages: HashMap<String, String>,
    custom_attributes: HashMap<String, String>,
}

impl<'a> AsyncValidator<'a> {
    pub fn new(data: &'a Value) -> Self {
        Self {
            data,
            sync_rules: HashMap::new(),
            async_rules: HashMap::new(),
            custom_messages: HashMap::new(),
            custom_attributes: HashMap::new(),
        }
    }

    /// Add sync rules (same ergonomics as Validator)
    pub fn rule<R: Rule + 'static>(mut self, field: impl Into<String>, rule: R) -> Self { ... }
    pub fn rules(mut self, field: impl Into<String>, rules: Vec<Box<dyn Rule>>) -> Self { ... }

    /// Add async rules
    pub fn async_rule<R: AsyncRule + 'static>(mut self, field: impl Into<String>, rule: R) -> Self { ... }

    /// Run validation. Returns:
    /// - Ok(()) — all rules pass
    /// - Err(AsyncValidationError::Validation(e)) — field-level failures
    /// - Err(AsyncValidationError::Infra(e)) — DB/infra failure (handler → 500)
    pub async fn validate_async(self) -> Result<(), AsyncValidationError> {
        let mut errors = ValidationError::new();

        // 1. Run sync rules first (same loop as Validator::validate)
        for (field, rules) in &self.sync_rules {
            let value = get_nested_value(self.data, field)
                .cloned()
                .unwrap_or(Value::Null);
            let display_field = self.get_display_field(field);

            let has_nullable = rules.iter().any(|r| r.name() == "nullable");
            if has_nullable && value.is_null() { continue; }

            for rule in rules {
                if rule.name() == "nullable" { continue; }
                if let Err(msg) = rule.validate(&display_field, &value, self.data) {
                    let key = format!("{field}.{}", rule.name());
                    let msg = self.custom_messages.get(&key).cloned().unwrap_or(msg);
                    errors.add(field, msg);
                }
            }
        }

        // 2. Run async rules only on fields without sync errors (D-03)
        for (field, rules) in &self.async_rules {
            if errors.has(field) { continue; } // fail-fast per D-03

            let value = get_nested_value(self.data, field)
                .cloned()
                .unwrap_or(Value::Null);
            let display_field = self.get_display_field(field);

            for rule in rules {
                match rule.validate(&display_field, &value, self.data).await {
                    Ok(()) => {}
                    Err(msg) => {
                        let key = format!("{field}.{}", rule.name());
                        let msg = self.custom_messages.get(&key).cloned().unwrap_or(msg);
                        errors.add(field, msg);
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AsyncValidationError::Validation(errors))
        }
    }
}
```

### Pattern 4: Result Shape for D-12 — Validation vs. Infra Distinction

**Recommendation:** A dedicated error enum wrapping the two distinct failure modes.

```rust
/// Errors from `AsyncValidator::validate_async`.
///
/// Separates field-level validation failures (which produce a redirect-back)
/// from infrastructure failures (which produce a 500).
#[derive(Debug)]
pub enum AsyncValidationError {
    /// One or more field validation rules failed. Use `with_old_input` +
    /// `redirect_back` / `redirect_to` / `into_action_error` as usual.
    Validation(ValidationError),
    /// A DB or infrastructure error occurred during an async rule. The
    /// handler should propagate this as a framework error (→ 500).
    Infra(FrameworkError),
}

impl From<AsyncValidationError> for crate::http::action::ActionError {
    fn from(e: AsyncValidationError) -> Self {
        match e {
            AsyncValidationError::Validation(_) =>
                // Caller has already flashed errors; use validation_failed
                // to suppress the redundant URL envelope
                crate::http::action::ActionError::validation_failed("/"),
            AsyncValidationError::Infra(fe) =>
                crate::http::action::ActionError::from(fe),
        }
    }
}
```

**Usage in an `#[action]` handler:**

```rust
#[action(redirect_to = "/pages")]
async fn store(req: Request) -> ActionResult {
    let data = req.input::<serde_json::Value>().await?;
    match AsyncValidator::new(&data)
        .rules("slug", rules![required(), string()])
        .async_rule("slug", unique("pages", "slug"))
        .validate_async()
        .await
    {
        Ok(()) => {}
        Err(AsyncValidationError::Validation(e)) => {
            return Err(e.with_old_input(&data).into_action_error("/pages/create"));
        }
        Err(AsyncValidationError::Infra(fe)) => {
            return Err(ActionError::from(fe));
        }
    }
    // proceed with insert
    Ok(())
}
```

**Why not `Result<(), ValidationError>`:** That shape cannot distinguish a failed validation from a DB error at the call site — both would look like validation failures.

**Why not return `FrameworkError` directly:** The validation path needs to call `with_old_input` (which is on `ValidationError`, not `FrameworkError`). The `AsyncValidationError` enum keeps both paths accessible without downcasting.

The enum itself has zero new framework dependencies — `FrameworkError` and `ValidationError` are already in the same crate. [VERIFIED: framework/src/error.rs, framework/src/validation/error.rs]

### Pattern 5: Handler Signature (for `validate_async` argument — D-05)

The success-criterion-1 snippet shows `validate_async(&req)`. Based on reading `framework/src/http/request.rs`, `req.input()` consumes the request body. If `validate_async` took `&req`, it would be unable to call `req.input()` again because `&req` is an immutable reference but `input()` requires `&mut` (or moves). 

**Recommendation:** `AsyncValidator::new(&data)` with `validate_async` consuming `self` (no argument). The handler calls `req.input().await?` first, then builds the validator from `&data`. This is identical to how sync `Validator::new(&data)` works.

```rust
// CORRECT pattern
let data = req.input::<serde_json::Value>().await?;
let result = AsyncValidator::new(&data)
    .async_rule("slug", unique("pages", "slug"))
    .validate_async()
    .await;
```

The illustrative snippet in the ROADMAP (`validate_async(&req)`) was not a binding API contract — it was illustrative prose. The `validate_async()` signature with no arg is cleaner and avoids any re-parse concern.

### Pattern 6: Helper — `json_value_to_sea_value`

The `AsyncRule::validate` receives a `serde_json::Value`. SeaORM's `Statement::from_sql_and_values` requires `sea_orm::Value`. A small helper is needed:

```rust
fn json_value_to_sea_value(v: &serde_json::Value) -> sea_orm::Value {
    match v {
        serde_json::Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                sea_orm::Value::BigInt(Some(i))
            } else if let Some(f) = n.as_f64() {
                sea_orm::Value::Double(Some(f))
            } else {
                sea_orm::Value::String(Some(Box::new(n.to_string())))
            }
        }
        serde_json::Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        _ => sea_orm::Value::String(Some(Box::new(v.to_string()))),
    }
}
```

This helper lives in `rules_async.rs` as `pub(crate)`.

### Anti-Patterns to Avoid

- **Threading the DB connection through `AsyncRule::validate`:** Locked out by D-07. The singleton `DB::connection()` is the only access point.
- **Using `Statement::from_sql_and_values` with `?` placeholders for Postgres:** The SQL is passed verbatim to the sqlx driver. Postgres will interpret `?` as the JSON key-exists operator. Always branch on `get_database_backend()`.
- **Using `from_sql_and_values` to translate placeholders automatically:** It does NOT. Confirmed in `sea-orm-1.1.19/src/database/statement.rs:39-45` — the SQL is stored as-is.
- **Returning an infra error as a validation failure:** Would silently pass or confuse the redirect-back flow. `AsyncValidationError::Infra` keeps them separate.
- **Hardcoding table/column names without identifier validation:** Guarding with `[A-Za-z0-9_]` + double-quoting is required (D-08).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Async trait objects | Manual HRTB + Pin<Box<dyn Future>> wrappers | `async-trait = "0.1"` | Already in Cargo.toml; handles Send bounds and lifetime elision correctly |
| Parameterized raw SQL | String formatting with user values | `Statement::from_sql_and_values` + `Values` binding | SQL injection prevention; values are bound by sqlx, not interpolated |
| Backend detection | Hardcoding SQLite or Postgres | `conn.get_database_backend()` | Supports both backends; consumer switches DB without code changes |
| Translation | Custom message map | `translate_validation("validation.unique", ...)` + English fallback | Consistent with all existing rules; hooks into consumer's ferro-lang files |

---

## SeaORM Raw Query Reference (Verified)

### Exact API for the COUNT query

```rust
// Source: sea-orm 1.1.19 — verified from registry source
// db: &sea_orm::DatabaseConnection  (via DB::connection()? which Derefs to this)

use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

let backend: DatabaseBackend = db.get_database_backend();

// Placeholder syntax MUST match the backend:
let sql = match backend {
    DatabaseBackend::Postgres =>
        r#"SELECT COUNT(*) AS count FROM "pages" WHERE "slug" = $1"#,
    _ =>
        r#"SELECT COUNT(*) AS count FROM "pages" WHERE "slug" = ?"#,
};

let stmt = Statement::from_sql_and_values(
    backend,
    sql,
    [sea_orm::Value::String(Some(Box::new("my-slug".to_string())))],
);

let row: Option<sea_orm::QueryResult> = db.query_one(stmt).await?;
// query_one signature: async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr>

let count: i64 = row
    .and_then(|r| r.try_get::<i64>("", "count").ok())
    .unwrap_or(0);
// try_get signature: pub fn try_get<T: TryGetable>(&self, pre: &str, col: &str) -> Result<T, DbErr>
// pre = "" (no table prefix needed for raw SELECT), col = "count" (the alias)
```

[VERIFIED: sea-orm-1.1.19 source at `src/database/statement.rs`, `src/database/db_connection.rs:173`, gestiscilo-it `bookings.rs:2024-2049`]

### `sea_orm::Value` From implementations (for `.ignore(id)`)

The `impl Into<sea_orm::Value>` bound on `.ignore()` accepts all of:
- `i64`, `i32`, `i8`, `i16`, `u32`, `u64` (signed/unsigned integers)
- `String`, `&str`
- `uuid::Uuid` (with uuid feature)
- `bool`, `f32`, `f64`

[VERIFIED: docs.rs/sea-orm/1.1.19/sea_orm/enum.Value.html]

### `get_database_backend()` source

```rust
// Source: framework/src/database/connection.rs — DbConnection Derefs to DatabaseConnection
// DB::connection() -> Result<DbConnection, FrameworkError>
// DbConnection: Deref<Target=DatabaseConnection>
// DatabaseConnection::get_database_backend() -> DbBackend  [ConnectionTrait required method]
let db = DB::connection()?;
let backend = db.get_database_backend(); // works via Deref
```

[VERIFIED: framework/src/database/connection.rs:121-127 (Deref impl), docs.rs ConnectionTrait]

---

## Gestiscilo Tenancy Note (Deferred Question from CONTEXT.md)

The deferred question was: does gestiscilo use separate databases per tenant or shared tables with `tenant_id`?

**Answer:** [VERIFIED: gestiscilo-it/app/src/models/products.rs:58, :105] Gestiscilo uses **shared tables with a `tenant_id` column**. Every row-scoped query filters on `tenant_id`. The slug uniqueness constraint is per-tenant (a composite unique index on `(tenant_id, slug)` at `m20260529_108_add_slug_to_products.rs:124`).

**Implication for Phase 190:** Global `unique("products", "slug")` would check uniqueness across all tenants — which is incorrect for gestiscilo. The first consumer that uses `unique` for a tenant-scoped column will need scoped uniqueness (`.where_eq("tenant_id", tenant_id)`).

**Scope decision:** Phase 190 ships only the base `unique(table, column)` + `.ignore(id)` per the locked success criteria. The `.where_eq` scoping extension is a follow-up phase. This does NOT block Phase 190 — the infrastructure is correct, and `unique` is still useful for system-wide unique columns (email, username). Document the limitation explicitly in the `Unique` struct's rustdoc. [ASSUMED: the follow-up scoping extension will be scheduled before gestiscilo uses `unique` on a `tenant_id`-scoped column]

---

## Common Pitfalls

### Pitfall 1: Wrong Placeholder Syntax per Backend
**What goes wrong:** Using `?` placeholders in the SQL string when the backend is Postgres. Postgres interprets `?` as the JSON key-exists operator, causing a parse error at the clause following the `?`.
**Why it happens:** `Statement::from_sql_and_values` stores the SQL as-is. It does NOT translate `?` to `$1`. The sqlx driver receives the literal SQL string.
**How to avoid:** Always branch on `get_database_backend()`. Use `$1`, `$2` for `DatabaseBackend::Postgres`, `?` for `_` (SQLite/MySQL).
**Warning signs:** `DbErr` containing "syntax error at or near" on Postgres but not SQLite.
[VERIFIED: sea-orm-1.1.19 `src/database/statement.rs:39-45`, gestiscilo-it comment `m20260529_108...:112`]

### Pitfall 2: Infra Error Silently Becoming a Validation Pass
**What goes wrong:** A `DB::connection()` error or `query_one` error is swallowed (e.g. `.unwrap_or(0)` applied before distinguishing I/O from result), causing the uniqueness check to silently pass — inserting a duplicate.
**Why it happens:** `query_one` returns `Result<Option<QueryResult>, DbErr>`. An eager `.ok().flatten()` discards the `DbErr`.
**How to avoid:** Propagate `DbErr` as `AsyncValidationError::Infra`. Only call `.ok()` on the inner `Option<QueryResult>` after the `Result` has been checked with `?` or `map_err`.
**Warning signs:** Duplicate rows appearing in the DB despite `unique` validation being in place.

### Pitfall 3: Double-Consuming the Request Body
**What goes wrong:** `validate_async` takes `&req` and calls `req.input()` internally, but the handler also called `req.input()` earlier — compile error because `input()` moves the body.
**Why it happens:** The ROADMAP snippet `validate_async(&req)` was illustrative. The actual `input()` API consumes the body.
**How to avoid:** `AsyncValidator::new(&data)` with `validate_async()` (no arg). Call `req.input().await?` once in the handler, then pass `&data` to the validator.

### Pitfall 4: Hardcoded Identifier Injection
**What goes wrong:** A developer passes an identifier like `"users; DROP TABLE users; --"` (intentionally or through a bug). Without identifier validation, this would be interpolated into the SQL string.
**Why it happens:** Table/column identifiers cannot be SQL-bound; they must be interpolated into the string.
**How to avoid:** Validate identifiers against `[A-Za-z0-9_]` before constructing the SQL. Document the trust boundary in rustdoc. Identifiers come from handler source code, not user input — but defense in depth applies.

### Pitfall 5: Missing `Send + Sync` on `AsyncRule`
**What goes wrong:** Clippy/rustc error: "`dyn AsyncRule` cannot be sent between threads safely".
**Why it happens:** `Box<dyn AsyncRule>` stored in `AsyncValidator` is held across `.await` points, requiring `Send`. Stored in a struct, requiring `Sync`.
**How to avoid:** `AsyncRule: Send + Sync` in the trait definition. Mirror the existing `Rule: Send + Sync` (confirmed in `rule.rs:6`).

---

## ferro-lang Translation Key Wiring

**Pattern:** [VERIFIED: framework/src/validation/bridge.rs + rules.rs]

The validation module never depends on `ferro-lang` directly. Instead:
1. `bridge.rs` exposes `register_validation_translator(f: TranslatorFn)` — called once at app boot by the consumer's bootstrap.
2. Rules call `translate_validation("validation.unique", &[("attribute", field)]).unwrap_or_else(|| English fallback)`.
3. The translation key `validation.unique` is resolved by whatever translator the consumer registers.
4. If no translator is registered (e.g. tests, bare `DB::init()`), the English fallback fires.

**What Phase 190 must ship:**
- The `validation.unique` key called in `Unique::validate` with the English fallback `"The {field} has already been taken."`.
- No ferro-lang JSON file changes required in Phase 190. The consumer adds `"unique": "Il campo {attribute} esiste già."` (or similar) to their own `lang/it/validation.json`. Documenting this is Phase 192 scope.

**No new bridge infrastructure needed** — the existing `OnceLock<TranslatorFn>` pattern handles the new key automatically.

---

## Module Export Checklist

After creating the three new files, update:

1. `framework/src/validation/mod.rs` — add:
   ```rust
   mod async_rule;
   mod async_validator;
   mod rules_async;

   pub use async_rule::AsyncRule;
   pub use async_validator::{AsyncValidator, AsyncValidationError};
   pub use rules_async::unique;
   ```

2. `framework/src/lib.rs` — add to the validation re-exports section:
   ```rust
   pub use validation::{AsyncRule, AsyncValidator, AsyncValidationError, unique};
   ```

The `rules!` macro in `mod.rs` boxes `Box<dyn Rule>` items. A parallel `async_rules!` macro may help ergonomics for `Box<dyn AsyncRule>` vectors, but is optional — the planner should decide. If added, it goes in `mod.rs` next to `rules!`.

---

## Validation Architecture (Nyquist)

`workflow.nyquist_validation` key is absent from `.planning/config.json` — treated as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `tokio::test` for async |
| Config file | `framework/Cargo.toml` — `[dev-dependencies]` already has `serial_test = "3"` |
| Quick run command | `cargo test -p ferro-rs --lib validation` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VALID-01 | `unique(table, col)` returns field error when count > 0 in SQLite | unit + async | `cargo test -p ferro-rs --lib unique_detects_existing_value -x` | ❌ Wave 0 |
| VALID-01 | `unique(table, col)` passes when count = 0 | unit + async | `cargo test -p ferro-rs --lib unique_passes_on_no_match -x` | ❌ Wave 0 |
| VALID-02 | `.ignore(id)` skips the current record | unit + async | `cargo test -p ferro-rs --lib unique_ignore_excludes_self -x` | ❌ Wave 0 |
| VALID-02 | `.ignore_on(pk_col, id)` with non-default PK | unit + async | `cargo test -p ferro-rs --lib unique_ignore_on_custom_pk -x` | ❌ Wave 0 |
| VALID-03 | Sync rules run before async rules | unit + async | `cargo test -p ferro-rs --lib async_validator_sync_first -x` | ❌ Wave 0 |
| VALID-03 | Async rules skipped on fields with sync errors | unit + async | `cargo test -p ferro-rs --lib async_validator_skips_async_on_sync_error -x` | ❌ Wave 0 |
| VALID-03 | Existing sync `Validator` API unchanged (compile test) | compile | `cargo test -p ferro-rs --lib validator -x` | ✅ existing |
| D-12 | DB infra error → `AsyncValidationError::Infra`, not `Validation` | unit + async | `cargo test -p ferro-rs --lib async_validator_infra_error_shape -x` | ❌ Wave 0 |

### SQLite-Only CI Strategy

All tests run against an in-memory SQLite database (`sqlite::memory:`). The uniqueness and exclude-self behaviors are fully testable with SQLite. Tests create a scratch table via `CREATE TABLE IF NOT EXISTS`, insert a row, then run `AsyncValidator`.

Postgres-specific behavior (the `$1` placeholder path) can be smoke-tested via a mock: if a test needs to verify the Postgres branch, use a `MockDatabase` (sea-orm mock feature) rather than a live Postgres instance.

**Test infrastructure needed (Wave 0):**
- A test helper that sets up an in-memory SQLite DB via `DB::init_with(DatabaseConfig::builder().url("sqlite::memory:").build())` and creates a minimal scratch table.
- Note: `DB::connection()` uses `App::resolve::<DbConnection>()` — the `App` singleton. Tests must call `DB::init_with(...)` before using the `Unique` rule. Tests need `#[serial]` (from `serial_test`) to isolate singleton state.

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-rs --lib validation`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `framework/src/validation/async_rule.rs` — `AsyncRule` trait
- [ ] `framework/src/validation/async_validator.rs` — `AsyncValidator` + `AsyncValidationError`
- [ ] `framework/src/validation/rules_async.rs` — `Unique` rule + tests
- [ ] Test helper: in-memory SQLite fixture setup (`DB::init_with` + `CREATE TABLE`)
- [ ] `serial_test` already in `[dev-dependencies]` — no new deps needed [VERIFIED: Cargo.toml:79]

---

## Environment Availability

Step 2.6: No new external dependencies. All required tools (`cargo`, `sea-orm`, `async-trait`) are already present and verified.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `async-trait` crate | `AsyncRule` trait | ✓ | `"0.1"` | — |
| `sea-orm` | Raw SQL count query | ✓ | `1.1.19` | — |
| SQLite (in-memory) | Unit tests | ✓ | via sqlx-sqlite feature already enabled | — |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `try_get::<i64>("", "count")` reads the `COUNT(*) AS count` column from a raw SQLite/Postgres result with empty table prefix | Code Examples — COUNT query | If the column alias `count` is not accessible via `("", "count")`, use `("", "count")` or scan column 0. Gestiscilo uses `try_get::<String>("", "vessel_name")` (bookings.rs:2049) with `""` prefix — same pattern. [VERIFIED from codebase] |
| A2 | The follow-up scoped uniqueness (`.where_eq`) is not needed before Phase 190 ships | Gestiscilo Tenancy Note | If gestiscilo immediately uses `unique` on a `tenant_id`-scoped field, global `unique` would reject valid values from other tenants. Risk: LOW — the slug uniqueness field test that motivated this phase uses a per-tenant slug; Phase 190 must document the limitation. |
| A3 | `App::resolve::<DbConnection>()` in tests requires `#[serial]` test isolation | Validation Architecture | If tests run in parallel and race on the `App` singleton, the DB fixture may be overwritten. `serial_test` is already a dev-dependency. |

---

## Open Questions

1. **`async_rules!` macro — needed?**
   - What we know: `rules!` exists for `Box<dyn Rule>`. Async rules can be added via `.async_rule(field, rule)` one at a time.
   - What's unclear: Whether handlers will commonly add multiple async rules per field (unlikely for v1).
   - Recommendation: Omit `async_rules!` in Phase 190. Add if a consumer friction phase requests it.

2. **`nullable` handling in `AsyncValidator`**
   - What we know: The sync `Validator` skips all rules for a field if `nullable()` is in the rule list and the value is null.
   - What's unclear: Should `AsyncValidator` implement the same `nullable` skip for async rules?
   - Recommendation: Yes — mirror the sync behavior exactly. If the field has a `nullable()` sync rule and the value is null, skip all async rules for that field too (even before the sync-rule pass). This prevents a DB query for a null value.

---

## Sources

### Primary (HIGH confidence)
- `framework/src/validation/rule.rs` — sync `Rule` trait shape (AsyncRule mirrors it)
- `framework/src/validation/validator.rs` — `Validator` builder/loop pattern
- `framework/src/validation/error.rs` — `ValidationError`, `with_old_input`, `into_action_error`
- `framework/src/validation/bridge.rs` — `translate_validation` / `TranslatorFn` pattern
- `framework/src/validation/rules.rs` — `translate_validation(...).unwrap_or_else(|| English)` per-rule pattern
- `framework/src/database/mod.rs` — `DB::connection()` signature
- `framework/src/database/connection.rs` — `DbConnection` Deref to `DatabaseConnection`
- `framework/src/error.rs` — `FrameworkError::Database` variant
- `framework/src/http/action.rs` — `ActionError::validation_failed`, `From<FrameworkError>`
- `~/.cargo/registry/src/.../sea-orm-1.1.19/src/database/statement.rs` — `from_sql_and_values` source (placeholder NOT translated)
- `~/.cargo/registry/src/.../sea-orm-1.1.19/src/database/db_connection.rs` — `query_one` signature
- `~/.cargo/registry/src/.../sea-orm-1.1.19/src/driver/sqlx_sqlite.rs` — `sqlx_query` shows SQL passed verbatim to sqlx

### Secondary (MEDIUM confidence)
- `gestiscilo-it/app/src/models/bookings.rs:1999-2049` — `Statement::from_sql_and_values` + `get_database_backend()` + `query_one` + `try_get` field pattern (in production, same sea-orm version)
- `gestiscilo-it/app/src/migrations/m20260529_108...:112` — explicit comment: `Statement::from_sql_and_values` does not translate placeholders per backend
- `framework/src/tenant/lookup.rs:19-28` — `#[async_trait]` + `Send + Sync` trait pattern in the framework
- docs.rs/sea-orm/1.1.19/sea_orm/struct.QueryResult.html — `try_get::<T>` signature

### Tertiary (LOW confidence)
- None — all key claims were verified from source.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions confirmed from Cargo.toml and `cargo tree`
- Architecture: HIGH — verified from existing codebase patterns and sea-orm source
- SeaORM raw query API: HIGH — verified from registry source + gestiscilo production usage
- Pitfalls: HIGH — placeholder pitfall confirmed from sea-orm source + gestiscilo comment
- Test strategy: HIGH — existing `serial_test` dep + `DB::init_with` pattern available

**Research date:** 2026-06-09
**Valid until:** 2026-09-09 (sea-orm 1.x is stable; async-trait unlikely to change)
