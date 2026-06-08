# Phase 190: Async Rule Infrastructure + `unique` Rule — Pattern Map

**Mapped:** 2026-06-09
**Files analyzed:** 5 (3 new files + 2 modified files)
**Analogs found:** 5 / 5

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `framework/src/validation/async_rule.rs` | trait definition | request-response | `framework/src/validation/rule.rs` | exact (sync counterpart) |
| `framework/src/validation/rules_async.rs` | rule struct + DB query | CRUD / request-response | `framework/src/validation/rules.rs` (`Required`, `RequiredIf`) + `framework/src/database/connection.rs` | role-match + DB pattern |
| `framework/src/validation/async_validator.rs` | validator builder + run loop | request-response | `framework/src/validation/validator.rs` | exact (sync counterpart) |
| `framework/src/validation/mod.rs` | module exports | — | existing `mod.rs` `pub use` lines | exact |
| `framework/src/lib.rs` | public re-exports | — | lines 286-320 validation re-export block | exact |

---

## Pattern Assignments

### `framework/src/validation/async_rule.rs` (trait definition, request-response)

**Analog:** `framework/src/validation/rule.rs`

**Imports pattern** (`rule.rs` lines 1-3):
```rust
use serde_json::Value;
```

**Core trait pattern** (`rule.rs` lines 6-14) — the sync version to mirror exactly:
```rust
pub trait Rule: Send + Sync {
    fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String>;
    fn name(&self) -> &'static str;
}
```

**Async trait adaptation** — apply `#[async_trait]` exactly as done in `framework/src/tenant/lookup.rs` lines 19-26:
```rust
use async_trait::async_trait;

#[async_trait]
pub trait TenantLookup: Send + Sync {
    async fn find_by_slug(&self, slug: &str) -> Option<TenantContext>;
    async fn find_by_id(&self, id: i64) -> Option<TenantContext>;
}
```

**Pattern to replicate in `async_rule.rs`:**
- Add `use async_trait::async_trait;` and `use serde_json::Value;` as imports.
- Annotate the trait with `#[async_trait]`.
- Bounds: `AsyncRule: Send + Sync` — mirror `Rule: Send + Sync` from `rule.rs:6` (confirmed reason: `framework/src/validation/rules.rs` pitfall note — "Missing `Send + Sync` on `AsyncRule`" causes compiler error for `Box<dyn AsyncRule>` across `.await` points).
- Method signature: `async fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String>` — identical parameter shape to `Rule::validate`, just `async`.
- Also keep `fn name(&self) -> &'static str;` — no `async`.

**`#[async_trait]` on impl blocks** — copy the impl pattern from `lookup.rs` lines 97-98 and 158-160:
```rust
#[async_trait]
impl TenantLookup for DbTenantLookup {
    async fn find_by_slug(&self, slug: &str) -> Option<TenantContext> { ... }
}

// In tests, impl blocks also need the attribute:
#[async_trait]
impl TenantLookup for MockLookup { ... }
```

---

### `framework/src/validation/rules_async.rs` (rule struct + DB query, CRUD)

**Analog 1 (rule struct shape):** `framework/src/validation/rules.rs`

**Rule struct + constructor pattern** (`rules.rs` lines 47-73 — `RequiredIf` as the closest struct-with-fields rule):
```rust
pub struct RequiredIf {
    other: String,
    value: Value,
}

pub fn required_if(other: impl Into<String>, value: impl Into<Value>) -> RequiredIf {
    RequiredIf {
        other: other.into(),
        value: value.into(),
    }
}
```

**`impl Rule` core pattern** (`rules.rs` lines 21-44 — `Required`):
```rust
impl Rule for Required {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        // ... check logic ...
        if is_empty {
            Err(
                translate_validation("validation.required", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {field} field is required.")),
            )
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required"
    }
}
```

**Translation pattern to replicate for `Unique`** (`rules.rs` lines 32-35):
```rust
Err(
    translate_validation("validation.unique", &[("attribute", field)])
        .unwrap_or_else(|| format!("The {field} has already been taken.")),
)
```

Note: `translate_validation` is `pub(crate)` in `bridge.rs` line 37. Import it as:
```rust
use crate::validation::translate_validation;
```
(Mirror `rules.rs` line 3: `use crate::validation::translate_validation;`)

**Analog 2 (DB access pattern):** `framework/src/database/connection.rs` + `framework/src/database/mod.rs`

**DB singleton access** (`database/mod.rs` lines 171-173):
```rust
pub fn connection() -> Result<DbConnection, FrameworkError> {
    App::resolve::<DbConnection>()
}
```

**`DbConnection` Deref to `DatabaseConnection`** (`connection.rs` lines 121-127):
```rust
impl std::ops::Deref for DbConnection {
    type Target = DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
```

This means `DB::connection()?` returns a `DbConnection` and `.get_database_backend()` is callable directly via `Deref` (`ConnectionTrait` on `DatabaseConnection`).

**Imports for `rules_async.rs`:**
```rust
use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use serde_json::Value;
use crate::database::DB;
use crate::error::FrameworkError;
use crate::validation::translate_validation;
use super::async_rule::AsyncRule;
```

**`Unique` struct shape:**
```rust
pub struct Unique {
    table: String,
    col: String,
    ignore: Option<(String, sea_orm::Value)>,  // (pk_col, pk_value)
}

pub fn unique(table: impl Into<String>, col: impl Into<String>) -> Unique {
    Unique { table: table.into(), col: col.into(), ignore: None }
}
```

**`.ignore()` / `.ignore_on()` builder methods** — consuming `mut self -> Self` (same builder style as `Validator` in `validator.rs` lines 51-58):
```rust
pub fn ignore(mut self, id: impl Into<sea_orm::Value>) -> Self {
    self.ignore = Some(("id".to_string(), id.into()));
    self
}

pub fn ignore_on(mut self, pk_col: impl Into<String>, id: impl Into<sea_orm::Value>) -> Self {
    self.ignore = Some((pk_col.into(), id.into()));
    self
}
```

**Identifier validation guard** — new private helpers (no analog, must write fresh):
```rust
fn validate_identifier(ident: &str) -> Result<(), String> {
    if !ident.is_empty() && ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(format!("Invalid SQL identifier: {ident:?}"))
    }
}

fn quote_ident(ident: &str) -> String {
    format!("\"{ident}\"")
}
```

**SeaORM COUNT query pattern** — per-backend branch required (no in-tree analog; pattern from RESEARCH.md verified against sea-orm 1.1.19 source):
```rust
let backend = db.get_database_backend();
let sql = match backend {
    DatabaseBackend::Postgres =>
        format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = $1"),
    _ =>
        format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = ?"),
};
let stmt = Statement::from_sql_and_values(backend, sql, [value]);
let row = db.query_one(stmt).await.map_err(|e| FrameworkError::database(e.to_string()))?;
let count: i64 = row
    .and_then(|r| r.try_get::<i64>("", "count").ok())
    .unwrap_or(0);
```

**`AsyncRule` impl for `Unique`** — annotate with `#[async_trait]` exactly as `TenantLookup` impls in `lookup.rs` lines 97-98:
```rust
#[async_trait]
impl AsyncRule for Unique {
    async fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        // identifier guards first, then DB query, then translate_validation pattern
    }

    fn name(&self) -> &'static str { "unique" }
}
```

**`json_value_to_sea_value` helper** — private `pub(crate)` helper function in this file (no codebase analog; write fresh per RESEARCH.md Pattern 6):
```rust
pub(crate) fn json_value_to_sea_value(v: &serde_json::Value) -> sea_orm::Value {
    match v {
        serde_json::Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { sea_orm::Value::BigInt(Some(i)) }
            else if let Some(f) = n.as_f64() { sea_orm::Value::Double(Some(f)) }
            else { sea_orm::Value::String(Some(Box::new(n.to_string()))) }
        }
        serde_json::Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        _ => sea_orm::Value::String(Some(Box::new(v.to_string()))),
    }
}
```

**Test structure** — mirror `rules.rs` `#[cfg(test)] mod tests` block (lines 887-893). Use `#[tokio::test]` for async tests (pattern from `lookup.rs` line 155 `#[tokio::test]`). Add `#[serial]` from `serial_test` crate for tests that touch the `DB` singleton.

---

### `framework/src/validation/async_validator.rs` (validator builder + run loop, request-response)

**Analog:** `framework/src/validation/validator.rs`

**Imports pattern** (`validator.rs` lines 1-5):
```rust
use crate::validation::{Rule, ValidationError};
use serde_json::Value;
use std::collections::HashMap;
```

For `AsyncValidator`, extend with:
```rust
use crate::error::FrameworkError;
use crate::validation::async_rule::AsyncRule;
```

**Struct definition pattern** (`validator.rs` lines 30-36):
```rust
pub struct Validator<'a> {
    data: &'a Value,
    rules: HashMap<String, Vec<Box<dyn Rule>>>,
    custom_messages: HashMap<String, String>,
    custom_attributes: HashMap<String, String>,
    stop_on_first_failure: bool,
}
```

`AsyncValidator` mirrors this, adding `async_rules`:
```rust
pub struct AsyncValidator<'a> {
    data: &'a Value,
    sync_rules: HashMap<String, Vec<Box<dyn Rule>>>,
    async_rules: HashMap<String, Vec<Box<dyn AsyncRule>>>,
    custom_messages: HashMap<String, String>,
    custom_attributes: HashMap<String, String>,
}
```

**Constructor pattern** (`validator.rs` lines 38-48):
```rust
pub fn new(data: &'a Value) -> Self {
    Self {
        data,
        rules: HashMap::new(),
        custom_messages: HashMap::new(),
        custom_attributes: HashMap::new(),
        stop_on_first_failure: false,
    }
}
```

**Builder methods pattern** (`validator.rs` lines 51-75 — consuming `mut self -> Self`):
```rust
pub fn rule<R: Rule + 'static>(mut self, field: impl Into<String>, rule: R) -> Self {
    let field = field.into();
    self.rules
        .entry(field)
        .or_default()
        .push(Box::new(rule) as Box<dyn Rule>);
    self
}

pub fn rules(mut self, field: impl Into<String>, rules: Vec<Box<dyn Rule>>) -> Self {
    self.rules.insert(field.into(), rules);
    self
}
```

Add `.async_rule()` following the same consuming builder pattern:
```rust
pub fn async_rule<R: AsyncRule + 'static>(mut self, field: impl Into<String>, rule: R) -> Self {
    self.async_rules
        .entry(field.into())
        .or_default()
        .push(Box::new(rule) as Box<dyn AsyncRule>);
    self
}
```

**Custom message / attribute methods** (`validator.rs` lines 83-123) — copy verbatim (same ergonomics required).

**Sync rule loop pattern** (`validator.rs` lines 132-174):
```rust
pub fn validate(self) -> Result<(), ValidationError> {
    let mut errors = ValidationError::new();

    for (field, rules) in &self.rules {
        let value = self.get_value(field);
        let display_field = self.get_display_field(field);

        let has_nullable = rules.iter().any(|r| r.name() == "nullable");
        if has_nullable && value.is_null() {
            continue;
        }

        for rule in rules {
            if rule.name() == "nullable" { continue; }

            if let Err(default_message) = rule.validate(&display_field, &value, self.data) {
                let message_key = format!("{}.{}", field, rule.name());
                let message = self.custom_messages.get(&message_key).cloned()
                    .unwrap_or(default_message);
                errors.add(field, message);
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

`validate_async` runs this sync loop first (copy it exactly), then adds the async rule loop that skips fields already in `errors` (D-03 fail-fast).

**`get_value` / `get_display_field` helpers** (`validator.rs` lines 208-224):
```rust
fn get_value(&self, field: &str) -> Value {
    get_nested_value(self.data, field).cloned().unwrap_or(Value::Null)
}

fn get_display_field(&self, field: &str) -> String {
    self.custom_attributes.get(field).cloned().unwrap_or_else(|| {
        field.split('_').collect::<Vec<_>>().join(" ")
    })
}
```

Copy `get_nested_value` from `validator.rs` lines 227-247 (dot-notation traversal) — same function needed in `async_validator.rs`.

**`AsyncValidationError` enum** — no exact codebase analog; use `FrameworkError` as the model for `thiserror`-free manual `Debug` derive (the enum is simple enough):

The closest derive-error analog is `FrameworkError` in `framework/src/error.rs` lines 259-337, which uses:
```rust
#[derive(Debug, Clone, Error)]
pub enum FrameworkError {
    #[error("Database error: {0}")]
    Database(String),
    // ...
}
```

`AsyncValidationError` does NOT use `thiserror` (it wraps existing types, not strings). Use plain `#[derive(Debug)]`:
```rust
#[derive(Debug)]
pub enum AsyncValidationError {
    /// Field validation failures — use `.with_old_input()` + redirect.
    Validation(ValidationError),
    /// DB or infra failure during an async rule — propagate as 500.
    Infra(FrameworkError),
}
```

Add `std::fmt::Display` impl manually (no `thiserror` needed — the enum is only ever matched at the call site, never formatted as a string directly):
```rust
impl std::fmt::Display for AsyncValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(e) => write!(f, "Validation failed: {e}"),
            Self::Infra(e) => write!(f, "Infrastructure error: {e}"),
        }
    }
}
impl std::error::Error for AsyncValidationError {}
```

**Test structure** — `#[tokio::test]` for async tests (same as `lookup.rs` line 155). Tests that call `validate_async` on an `AsyncValidator` with only sync rules can use regular `#[test]` + `tokio::runtime::Runtime::new()` or `#[tokio::test]`. Tests that exercise `Unique` need `#[serial]` (singleton DB).

---

### `framework/src/validation/mod.rs` (module exports)

**Analog:** current `mod.rs` lines 53-88

**Existing module declarations** (lines 53-58):
```rust
mod bridge;
mod error;
mod rule;
mod rules;
mod validatable;
mod validator;
```

**Pattern to follow** — append new `mod` declarations in the same style:
```rust
mod async_rule;
mod async_validator;
mod rules_async;
```

**Existing pub-use block** (lines 60-66):
```rust
pub(crate) use bridge::translate_validation;
pub use bridge::{register_validation_translator, TranslatorFn};
pub use error::ValidationError;
pub use rule::Rule;
pub use rules::*;
pub use validatable::Validatable;
pub use validator::{validate, Validator};
```

**New pub-use lines to add** (mirror the `Rule` / `Validator` / `rules::*` lines above):
```rust
pub use async_rule::AsyncRule;
pub use async_validator::{AsyncValidationError, AsyncValidator};
pub use rules_async::unique;
```

**`rules!` macro** (lines 83-88) — for reference if an `async_rules!` macro is added (optional per RESEARCH.md):
```rust
#[macro_export]
macro_rules! rules {
    ($($rule:expr),* $(,)?) => {
        vec![$(Box::new($rule) as Box<dyn $crate::validation::Rule>),*]
    };
}
```

If adding `async_rules!`, follow the same shape with `$crate::validation::AsyncRule` as the target type.

---

### `framework/src/lib.rs` (public re-exports)

**Analog:** `framework/src/lib.rs` lines 285-320 (the validation re-export block)

**Existing block to extend** (lines 286-320):
```rust
pub use validation::{
    // Rules
    accepted,
    alpha,
    // ...
    required,
    required_if,
    same,
    string,
    url,
    validate,
    Rule,
    TranslatorFn,
    Validatable,
    ValidationError,
    Validator,
};
```

**Lines to add** inside the same `pub use validation::{...}` block:
```rust
    AsyncRule,
    AsyncValidationError,
    AsyncValidator,
    unique,
```

Insert alphabetically: `AsyncRule` / `AsyncValidationError` / `AsyncValidator` before the existing `Rule` line; `unique` after `url` (alphabetical order matches the existing block style).

---

## Shared Patterns

### `#[async_trait]` attribute placement
**Source:** `framework/src/tenant/lookup.rs` lines 19, 97, 158
**Apply to:** trait definition in `async_rule.rs` AND every `impl AsyncRule for X` block
```rust
// On the trait:
#[async_trait]
pub trait AsyncRule: Send + Sync { ... }

// On every impl:
#[async_trait]
impl AsyncRule for Unique { ... }
```

### `translate_validation` + English fallback
**Source:** `framework/src/validation/rules.rs` lines 32-35 (`Required`) and lines 92-95 (`IsString`)
**Apply to:** `Unique::validate` in `rules_async.rs`
```rust
Err(
    translate_validation("validation.unique", &[("attribute", field)])
        .unwrap_or_else(|| format!("The {field} has already been taken.")),
)
```

### `translate_validation` import path
**Source:** `framework/src/validation/rules.rs` line 3
**Apply to:** `rules_async.rs`
```rust
use crate::validation::translate_validation;
```

### Consuming builder methods (`mut self -> Self`)
**Source:** `framework/src/validation/validator.rs` lines 51-75
**Apply to:** All builder methods on `AsyncValidator` and `Unique`
```rust
pub fn some_builder(mut self, arg: impl Into<String>) -> Self {
    self.field = arg.into();
    self
}
```

### `DB::connection()` error propagation
**Source:** `framework/src/database/mod.rs` line 171
**Apply to:** `Unique::validate` in `rules_async.rs`

DB errors must become `AsyncValidationError::Infra(FrameworkError::Database(...))`, never a validation `Err(String)`. The `AsyncRule::validate` signature returns `Result<(), String>`, so the `Unique` impl must call `DB::connection()` and `query_one()` errors through the `AsyncValidator`'s outer `Result` — not through the `Result<(), String>` return of the rule itself. The validator calls the rule and, if the `AsyncRule` method signals an infra error (via a sentinel or by having the outer loop handle `FrameworkError`), wraps it in `AsyncValidationError::Infra`.

The cleanest approach (given the `AsyncRule` signature is `Result<(), String>`): have `AsyncValidator::validate_async` catch infra errors before they become field messages. One pattern: the `Unique` impl calls `DB::connection().map_err(|e| format!("__infra_error__:{}", e))?` and `AsyncValidator` checks if the `Err(String)` starts with `__infra_error__:` before adding it to field errors. An alternative: change the `AsyncRule` trait return to `Result<(), AsyncRuleError>` with an enum. The planner chooses the concrete shape; the `AsyncValidationError::Infra(FrameworkError)` variant in the outer return type is the fixed requirement.

### `FrameworkError::database(...)` constructor
**Source:** `framework/src/error.rs` lines 363-365
```rust
pub fn database(message: impl Into<String>) -> Self {
    Self::Database(message.into())
}
```
Use `FrameworkError::database(db_err.to_string())` when wrapping `sea_orm::DbErr`.

### `ValidationError::new()` + `.add()` + `.is_empty()`
**Source:** `framework/src/validation/error.rs` lines 22-42
**Apply to:** `AsyncValidator::validate_async` run loop (identical to `Validator::validate` loop)
```rust
let mut errors = ValidationError::new();
// ...
errors.add(field, message);
// ...
if errors.is_empty() { Ok(()) } else { Err(AsyncValidationError::Validation(errors)) }
```

### Test `#[serial]` annotation for singleton DB
**Source:** `framework/Cargo.toml` dev-dependency `serial_test = "3"` (confirmed by RESEARCH.md)
**Apply to:** All tests in `rules_async.rs` that call `DB::init_with(...)` or `DB::connection()`
```rust
#[tokio::test]
#[serial]
async fn unique_detects_existing_value() { ... }
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `Unique::validate_identifier` + `quote_ident` private helpers | utility | — | No existing SQL-identifier-validation helper in the codebase |
| `json_value_to_sea_value` helper | utility / transform | transform | No `serde_json::Value` → `sea_orm::Value` conversion exists in-tree |
| SeaORM per-backend `Statement::from_sql_and_values` count query | DB I/O | CRUD | No raw statement query exists in `framework/src/`; analog is in gestiscilo-it (external) |

For these, use the concrete code from RESEARCH.md Patterns 2 and 6 directly.

---

## Metadata

**Analog search scope:** `framework/src/validation/`, `framework/src/database/`, `framework/src/tenant/`, `framework/src/error.rs`, `framework/src/lib.rs`
**Files scanned:** 10
**Pattern extraction date:** 2026-06-09
