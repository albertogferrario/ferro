# Architecture Research

**Domain:** ferro v12.4 — async validation rules + DB constraint→field-level error mapping
**Researched:** 2026-06-09
**Confidence:** HIGH (all findings grounded in source code)

## Existing System Overview

```
POST handler (#[action])
    │
    ├─ Validator::new(&data)         ← sync; Rule trait: validate(&str,&Value,&Value)->Result<(),String>
    │   .rules("field", rules![...])
    │   .validate()                  ← returns Result<(), ValidationError>
    │
    ├─ ValidationError::with_old_input(&data)
    │   .into_action_error(&back_url)  ← flashes _validation_errors + _old_input.*, returns ActionError
    │
    └─ #[action] macro → handle_action_result()
            │
            ├─ Ok(()) → 303 + ?success=1
            └─ Err(ActionError) → 303 + optional ?error=kind&msg= envelope
                    (suppress_url_envelope=true when ActionError::validation_failed)

DB Errors today:
    sea_orm::DbErr  →  From<DbErr> for ActionError  →  ActionError::msg(err.to_string())
                       raw SQL string in flash, no field attribution
```

## v12.4 Integration Design

### Decision 1: Async Rule Architecture

**Chosen: `AsyncRule` trait + `validate_async` method on `Validator`.**

Do not add async capability to the existing `Rule` trait. Rust `async fn` in traits requires either `async_trait` (dyn-safe boxed futures) or returned `impl Future` (not dyn-compatible without boxing). The sync `Rule` trait is already used as `dyn Rule` in `Vec<Box<dyn Rule>>` — introducing async changes the vtable shape and forces every existing rule to grow an async impl or adapter.

The clean approach:

```rust
// New trait — framework/src/validation/async_rule.rs
#[async_trait::async_trait]
pub trait AsyncRule: Send + Sync {
    async fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String>;
    fn name(&self) -> &'static str;
}
```

`Validator` gains two new methods. Existing sync `validate()` is unchanged:

```rust
impl<'a> Validator<'a> {
    // existing — unchanged
    pub fn validate(self) -> Result<(), ValidationError> { ... }

    // new builder — mirrors .rules() signature
    pub fn async_rule<R: AsyncRule + 'static>(mut self, field: impl Into<String>, rule: R) -> Self { ... }

    // new — runs sync rules first, then async rules on remaining fields
    pub async fn validate_async(self) -> Result<(), ValidationError> { ... }
}
```

`validate_async` semantics:
1. Run all sync rules exactly as `validate()` does now.
2. If `stop_on_first_failure` is set and sync errors exist, return early — skip async (pointless to DB-check uniqueness when required fields are missing).
3. Skip async rules for fields that already have sync errors (avoids a `unique()` SELECT on an empty string, which would pass and mask the required error).
4. Run async rules for each field sequentially — not concurrent. All async rules are fast DB point-reads; sequential execution avoids thundering-herd on validation under load.
5. Merge errors from both passes into one `ValidationError`.

**DB connection in async rules:** `DB::connection()` is a global singleton facade (`App::resolve::<DbConnection>()`). Async rules call it directly — no injection argument on the trait. The connection is always initialized before any request handler runs. This matches the established pattern used throughout `framework/src/database/model.rs`, `query_builder.rs`, and `transaction.rs`. No connection argument on `AsyncRule`; no coupling to `Request`.

### Decision 2: The `unique` Rule

```rust
// framework/src/validation/rules.rs (or rules/unique.rs if split)
pub struct Unique {
    table: &'static str,
    column: &'static str,
    except_id: Option<i64>,
    except_column: &'static str,  // default "id"
}

pub fn unique(table: &'static str, column: &'static str) -> Unique { ... }

impl Unique {
    /// Edit forms: exclude the current row from the uniqueness check.
    pub fn ignore(mut self, id: i64) -> Self { ... }
    /// Non-id primary key or composite key.
    pub fn ignore_where(mut self, col: &'static str, id: i64) -> Self { ... }
}
```

Implementation executes `SELECT COUNT(*) FROM <table> WHERE <column> = ? [AND <except_col> != ?]` via `sea_orm::Statement::from_sql_and_values`. Using raw SQL keeps the rule free of `EntityTrait` generics and avoids any dependency on consumer model types. Table and column names are `'static str` constants — no user-supplied strings at runtime. Project-agnostic-crates rule respected; no consumer strings embedded in the framework crate.

### Decision 3: Constraint-Error Mapping

**Chosen: explicit `ConstraintMap` builder, called at the handler write site.**

Three options evaluated:

| Option | Verdict |
|--------|---------|
| Mutate `From<DbErr> for ActionError` to detect UNIQUE and attribute fields | Rejected. `From` impls have no knowledge of field names. A UNIQUE violation on `pages.slug` cannot be attributed to a field without consumer-specific configuration embedded in the framework crate — violates project-agnostic-crates rule. |
| Catch in `#[action]` macro wrapper with a constraint map attribute | Rejected. The macro wrapper runs after the handler body returns; it has `ActionError` but not the original `DbErr` unless re-surfaced. Requires non-trivial macro expansion changes and a new attribute syntax. |
| Explicit `ConstraintMap` builder at the call site in the handler | Chosen. Handler code holds the `DbErr` immediately after the insert/update call. One method converts it to `ValidationError` shape, flowing through the existing `into_action_error` path identically to a proactive rule failure. |

```rust
// framework/src/validation/constraint.rs  (new file)

/// Maps a DB constraint violation to a field-level ValidationError.
///
/// Declared at the call site — no consumer strings live inside the framework
/// crate. The constraint name is the DB-level index name; the field name is
/// the form field the error surfaces under.
pub struct ConstraintMap {
    mappings: Vec<ConstraintMapping>,
}

struct ConstraintMapping {
    constraint: String,
    field: String,
    message: String,
}

impl ConstraintMap {
    pub fn new() -> Self { ... }

    /// Register a mapping: constraint_name → field + user-facing message.
    pub fn on(mut self, constraint: impl Into<String>, field: impl Into<String>, message: impl Into<String>) -> Self { ... }

    /// Try to convert a `sea_orm::DbErr` into a `ValidationError`.
    ///
    /// Returns `Ok(ValidationError)` when a constraint match is found.
    /// Returns `Err(DbErr)` when no mapping matched — caller handles as generic error.
    pub fn try_map(&self, err: sea_orm::DbErr) -> Result<ValidationError, sea_orm::DbErr> { ... }
}
```

Consumer call-site:

```rust
match page.insert(db.conn()).await {
    Ok(_) => { /* success */ }
    Err(e) => {
        return Err(ConstraintMap::new()
            .on("uniq_pages_slug", "slug", "This slug is already taken.")
            .try_map(e)
            .map(|ve| ve.with_old_input(&data).into_action_error(&back_url))
            .unwrap_or_else(|e| ActionError::msg(e.to_string()).redirect_to(&back_url)));
    }
}
```

The consumer supplies constraint name and field name. Framework supplies the plumbing. No consumer string in `framework/`. Project-agnostic-crates rule holds.

### Decision 4: Constraint Name Extraction

`sea_orm::DbErr` surfaces constraint violations as:
`DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))` or the `Query` variant.

The underlying `sqlx::DatabaseError` provides `.code()` and `.message()`. Format differs by backend:

- **SQLite:** `"UNIQUE constraint failed: pages.slug"` — constraint name parseable from message text
- **Postgres:** code `"23505"`, message `"duplicate key value violates unique constraint \"uniq_pages_slug\""` — constraint name appears in double-quotes in the message

`try_map` strategy (cfg-agnostic — no feature flag required):
1. Detect violation: message contains `"UNIQUE constraint failed"` (SQLite) OR code is `"23505"` (Postgres) OR message contains `"duplicate key value violates unique constraint"`.
2. If violation detected, scan registered `constraint` strings. Each registered name is checked against the full `DbErr` message with `.contains(constraint_name)`. The names are short and known at compile time — no regex needed.
3. First match wins → construct `ValidationError`, `add(field, message)`, return `Ok(ValidationError)`.
4. No match → return `Err(original_err)`.

This is a direct parallel to `ferro-reservation/src/kernel.rs:is_serialization_failure` which uses the same `DbErr::Exec(RuntimeErr::SqlxError(...))` pattern. Unlike that function, constraint detection does NOT require the `sqlx-postgres` feature gate because string-based matching works on both backends.

## Component Map: New vs Modified

```
framework/src/validation/
├── mod.rs              MODIFIED  — re-export AsyncRule, ConstraintMap, new validate() docs
├── rule.rs             UNCHANGED — sync Rule trait
├── validator.rs        MODIFIED  — add async_rules field, async_rule(), validate_async()
├── async_rule.rs       NEW       — AsyncRule trait (async_trait)
├── constraint.rs       NEW       — ConstraintMap, try_map()
├── rules.rs            MODIFIED  — add unique() and Unique struct implementing AsyncRule
├── bridge.rs           UNCHANGED — OnceLock<TranslatorFn> translation bridge
├── error.rs            UNCHANGED — ValidationError, with_old_input, into_action_error
└── validatable.rs      UNCHANGED — Validatable trait

framework/src/http/action.rs    UNCHANGED — ActionError, handle_action_result
framework/Cargo.toml            MODIFIED  — add async_trait (already in workspace)
docs/src/the-basics/validation.md  MODIFIED — async rules section + constraint mapping section
ferro-mcp/src/tools/code_templates.rs  MODIFIED — add unique/ConstraintMap templates
```

## Data Flows

### Async Uniqueness Check (proactive — before write)

```
POST /pages
    │
    ├─ let data = req.input::<Value>().await?
    │
    ├─ Validator::new(&data)
    │   .rules("title", rules![required()])
    │   .async_rule("slug", unique("pages", "slug").ignore(page_id))  ← edit form
    │   .validate_async().await
    │        │
    │        ├─ sync pass: required() on title
    │        └─ async pass: SELECT COUNT(*) FROM pages WHERE slug = ? AND id != ?
    │                        via DB::connection() — global singleton, no arg needed
    │
    ├─ Err(ve) → ve.with_old_input(&data).into_action_error(&back_url)
    │             flash _validation_errors + _old_input.* → 303
    │
    └─ Ok(()) → proceed to insert/update
```

### Constraint Violation Catch (defensive — after write)

```
    ├─ entity.insert(db.conn()).await
    │        │
    │        ├─ Ok(row) → success path
    │        └─ Err(DbErr)
    │                │
    │                └─ ConstraintMap::new()
    │                     .on("uniq_pages_slug", "slug", "This slug is already taken.")
    │                     .try_map(err)
    │                      │
    │                      ├─ Ok(ValidationError)
    │                      │    → .with_old_input(&data).into_action_error(&back_url)
    │                      │    → flash _validation_errors + _old_input.* → 303
    │                      │    (identical redirect path to proactive rule failure)
    │                      │
    │                      └─ Err(DbErr) — no constraint match
    │                           → ActionError::msg(err.to_string()).redirect_to(&back_url)
    │                           (non-constraint DB error → generic toast + tracing::error!)
```

## Integration Points

### Existing Surfaces Unchanged

| Surface | Status | Note |
|---------|--------|------|
| `Rule` trait signature | Unchanged | All existing sync rules continue to work |
| `Validator::validate()` | Unchanged | Existing callers compile without modification |
| `ValidationError` (all methods) | Unchanged | `add`, `with_old_input`, `into_action_error`, flash format |
| `ActionError` (all methods) | Unchanged | `validation_failed`, `suppress_url_envelope` |
| `handle_action_result()` | Unchanged | No modification needed |
| `From<DbErr> for ActionError` | Unchanged | Still the fallback for unmapped/non-constraint DB errors |
| `DB::connection()` | Used, not modified | Async rules call the existing singleton |

### New Dependency

`async_trait` crate — required for `Box<dyn AsyncRule>` dyn-compatibility. Already present in the workspace (used by `ferro-events`, `ferro-queue`). Add to `framework/Cargo.toml` only.

## Build Order

Dependency-ordered sequence:

1. **`AsyncRule` trait** (`async_rule.rs`) — foundation, zero deps beyond `serde_json` and `async_trait`
2. **`Unique` struct** (`rules.rs` or split file) — implements `AsyncRule`; uses `DB::connection()`; compile-tests `ignore()` / `ignore_where()` builders
3. **`Validator` extension** (`validator.rs`) — adds `async_rules` field, `async_rule()` builder, `validate_async()` method; run existing validator tests to confirm no regression
4. **`ConstraintMap`** (`constraint.rs`) — standalone struct; unit-testable with synthetic `DbErr` values; no async dependency
5. **`mod.rs` re-exports** — expose `AsyncRule`, `ConstraintMap`, `unique` at the `ferro_rs::validation` level
6. **Integration tests** — async rule tests via `TestDatabase` (exists in `framework/src/database/testing.rs`); constraint-map tests for both SQLite and Postgres message formats
7. **`ferro-mcp` code templates** — `unique_validation` and `constraint_map` template entries in `code_templates.rs`
8. **Docs** — `docs/src/the-basics/validation.md` async rules section + constraint mapping section

Steps 1–5 are a single logical unit; 6–8 can trail but must complete before the phase closes.

## Anti-Patterns to Avoid

### 1. Async Rule in Sync Trait via Block-On

**What goes wrong:** Implementing `Rule::validate()` to call `tokio::task::block_in_place` or `Handle::current().block_on(...)` to make a DB query synchronously.
**Why it's wrong:** Blocks an async executor thread. Causes starvation under concurrent requests. The sync `Rule` trait must stay sync — that is the architectural invariant.

### 2. Consumer Constraint Names in a Global Registry

**What goes wrong:** Adding a `register_constraint_mapping(constraint, field)` global at framework boot so handlers can skip the `ConstraintMap::new().on(...)` call.
**Why it's wrong:** The registry stores consumer strings ("uniq_pages_slug") inside the framework crate at runtime. Violates project-agnostic-crates rule. The per-call builder keeps the mapping at the correct scope (handler).

### 3. Running Async Rules on Fields Already Failing Sync Rules

**What goes wrong:** Calling `SELECT COUNT(*) FROM pages WHERE slug = ''` after `required()` already failed on `slug` — the SELECT returns 0 (no match), passes, and the required error may be obscured or the behavior seems inconsistent.
**Prevention:** `validate_async` skips async rules for fields that already carry sync errors. This is the correct default; opt-out not provided.

### 4. Returning Generic ValidationError on Constraint Miss

**What goes wrong:** `try_map` returning `Ok(ValidationError)` with a generic "database error" message when no constraint mapping matched.
**Why it's wrong:** Produces a field-less validation error bag, corrupts the flash UX, and hides unexpected DB errors from `tracing::error!` logging. `try_map` must return `Err(DbErr)` on no-match so the caller routes it through `ActionError::msg(...)`.

## Sources

- `framework/src/validation/validator.rs` — existing `Validator`, `Rule` trait bounds, `validate()` body
- `framework/src/validation/rule.rs` — current `Rule` trait signature
- `framework/src/validation/error.rs` — `ValidationError`, `into_action_error`, `flash_into_session`
- `framework/src/validation/bridge.rs` — `OnceLock<TranslatorFn>` precedent for framework-internal registries
- `framework/src/http/action.rs` — `ActionError`, `ActionError::validation_failed`, `handle_action_result`, `suppress_url_envelope`
- `framework/src/database/mod.rs` — `DB::connection()` singleton facade, `App::singleton` / `App::resolve` pattern
- `framework/src/database/connection.rs` — `DbConnection` Arc wrapper, `Deref<Target=DatabaseConnection>`
- `ferro-reservation/src/kernel.rs` — `DbErr::Exec(RuntimeErr::SqlxError(...))` destructuring precedent for constraint detection

---
*Architecture research for: ferro v12.4 Form Validation DX*
*Researched: 2026-06-09*
