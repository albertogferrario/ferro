# Phase 191: ConstraintMap + Portable UNIQUE-Violation Detection — Research

**Researched:** 2026-06-09
**Domain:** Rust / sea-orm / sqlx error handling; ferro validation module extension
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Builder shape: `ConstraintMap::new().on("pages_slug_unique", "slug", "has already been taken")`. Primary registration key is the Postgres constraint name. Consuming builder (`mut self -> Self`).
- **D-02:** `try_map(err: DbErr) -> Result<ValidationError, DbErr>`. Returns `Ok(ValidationError)` on match; returns `Err(err)` unchanged on non-match or non-UNIQUE error. Never swallows.
- **D-03:** Returned `ValidationError` is built via the existing `framework/src/validation/error.rs` constructor — zero new error-surfacing code.
- **D-04:** New file `framework/src/validation/constraint_map.rs`, re-exported through `mod.rs` + `lib.rs` (crate-root `ferro_rs::ConstraintMap`).
- **D-05:** Violation-type detection: `DbErr::sql_err() -> Option<SqlErr>`, matching `SqlErr::UniqueConstraintViolation(_)`.
- **D-06:** Identity detection is backend-bifurcated: Postgres — structured constraint name via downcast to `PgDatabaseError::constraint()`; SQLite — parse `table.column` from the error message string (`"UNIQUE constraint failed: pages.slug"`).
- **D-07:** Each entry stores BOTH identifiers. `.on(constraint, field, message)` keys by the Postgres name; optional chained `.sqlite("table.column")` adds the SQLite discriminator. Planner may refine exact spelling.
- **D-08:** Reuse the Phase 190 surfacing chain: `ValidationError` → `with_old_input()` → `into_action_error()` → 303 redirect-back. No new redirect path.
- **D-09:** `ConstraintMap` and all `.on(...)` strings are consumer-owned. `framework` crate carries zero constraint/field literals.
- **D-10:** SQLite path is fully `cargo test`-able with in-memory SQLite, reusing the Phase 190 `widgets` fixture pattern.
- **D-11:** Concurrent-insert simulation: seed a row, attempt a duplicate INSERT, feed the resulting `DbErr` to `try_map`, assert the field-level error.
- **D-12:** Postgres constraint-name extraction cannot run under the SQLite-only `cargo test` default. Closure includes a documented manual verification gate in 191-VERIFICATION.md.

### Claude's Discretion

- Exact spelling of the SQLite discriminator API (D-07): `.sqlite("table.column")` vs `.on_sqlite(...)` vs `ConstraintId` value object.
- Concrete `try_map` internals: order of type-check then identity-match, and how the inner sqlx error is downcast for Postgres.
- Whether `ConstraintMap` is `Clone` / reusable across requests or constructed per-handler (recommended: cheap to construct per call site; no global state).
- File-internal helper split within `constraint_map.rs`.
- Ergonomic call-site helper shape (extension trait vs plain `map_err` closure).

### Deferred Ideas (OUT OF SCOPE)

- Foreign-key / check / not-null constraint mapping.
- ferro-mcp template + validation docs (Phase 192).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| VALID-04 | Developer can opt-in to mapping a DB UNIQUE-constraint violation to a field-level `ValidationError` at the handler call site via `ConstraintMap`, so a concurrent-insert violation surfaces inline under the field with input preserved. | `ConstraintMap` builder pattern; `try_map` returning `Ok(ValidationError)` on match. |
| VALID-05 | Constraint-violation detection is backend-portable (SQLite + Postgres) via `DbErr::sql_err()` + bifurcated identification. A non-matching `DbErr` falls through unchanged to the existing `From<sea_orm::DbErr> for ActionError` passthrough. Framework holds no consumer-specific strings. | Verified sea-orm `sql_err()` + `SqlErr::UniqueConstraintViolation(String)`; confirmed `PgDatabaseError::constraint()` and SQLite message format; `DbErr` is movable and can be returned unchanged. |
</phase_requirements>

---

## Summary

Phase 191 adds the defensive half of v12.4's two-layer uniqueness story. The proactive layer (Phase 190's async `unique` rule) catches the common case before the write; this phase closes the TOCTOU race by intercepting a DB UNIQUE-constraint error at the INSERT/UPDATE site and mapping it to the same `ValidationError` field shape the proactive rule produces — so the user sees the same inline message regardless of whether they hit the pre-write check or the concurrent-insert edge case.

The implementation is a small, standalone `ConstraintMap` struct in `framework/src/validation/constraint_map.rs`. It carries a `Vec` of registration entries (Postgres constraint name, optional SQLite `table.column` discriminator, field name, message), and exposes a consuming `on()` builder and a borrowing `try_map(&self, DbErr) -> Result<ValidationError, DbErr>`. `try_map` first calls `err.sql_err()` for portable violation-type detection (sea-orm 1.1.x API, verified from source); on `SqlErr::UniqueConstraintViolation` it attempts identity-matching via backend-bifurcated logic; on any non-match it returns `Err(err)` unchanged so the caller's `?` reaches the existing `From<DbErr> for ActionError` passthrough at `action.rs:196`.

The killer correctness property: Postgres provides the structured constraint name via `PgDatabaseError::constraint() -> Option<&str>` (confirmed from `sqlx-postgres-0.8.6/src/error.rs`). SQLite does not expose constraint names at all; its error message (`"UNIQUE constraint failed: pages.slug"`) carries the `table.column` token, which is the reliable SQLite discriminator. The two backends are bifurcated at the identity-match step — not at the type-detection step, which is portable via `sql_err()`.

**Primary recommendation:** Implement `ConstraintMap` as a lightweight struct with a plain `Vec<ConstraintEntry>`, a consuming `.on()` builder, and a borrowing `try_map`. Add an extension trait `MapConstraintExt` on `Result<T, DbErr>` with `map_constraint(&self, map: &ConstraintMap, data: &Value, url: &str) -> Result<T, ActionError>` so call sites read `record.insert(db).await.map_constraint(&map, &data, url)?` rather than a closure ladder.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| UNIQUE constraint violation detection | API / Backend (`framework` crate) | — | Runs at the DB write site in the handler; no client involvement |
| Constraint-name → field mapping | API / Backend (`framework/src/validation/`) | — | Consumer-provided mapping consumed by `ConstraintMap` at handler time |
| Field-level error surfacing | API / Backend (`framework/src/validation/error.rs`) | — | Reuses existing `ValidationError` → 303 redirect-back path unchanged |
| SQLite test coverage | API / Backend (in-memory SQLite test) | — | `cargo test`-able via existing Phase 190 fixture pattern |
| Postgres verification gate | API / Backend (manual gate, Postgres instance) | — | Cannot run under `cargo test` default (SQLite-only CI) |

---

## Standard Stack

### Core (no new dependencies needed)

All required building blocks are already present in `framework/Cargo.toml`.

| Library | Version (locked) | Purpose | Why Standard |
|---------|-----------------|---------|--------------|
| `sea-orm` | 1.1.19 (lock) / `"1.0"` (manifest) | `DbErr::sql_err()` + `SqlErr` for portable violation-type detection | Already the project's ORM; `sql_err()` is the only portable entry point |
| `sqlx-postgres` | 0.8.6 (lock, via sea-orm feature) | `PgDatabaseError::constraint()` for structured Postgres constraint name | Already enabled via `sea-orm`'s `sqlx-postgres` feature |
| `sqlx-sqlite` | 0.8.6 (lock, via sea-orm feature) | `SqliteError` (type-detection only); identity from message string | Already enabled via sea-orm's `sqlx-sqlite` feature |

**No new `Cargo.toml` entries required.** All dependencies are transitively available.

### Existing Framework Assets (reuse verbatim)

| Asset | Location | How Phase 191 Uses It |
|-------|----------|-----------------------|
| `ValidationError` | `framework/src/validation/error.rs` | `try_map` builds and returns this type |
| `with_old_input()` / `into_action_error()` | `framework/src/validation/error.rs` | Call-site surfacing chain; zero new redirect code |
| `From<sea_orm::DbErr> for ActionError` | `framework/src/http/action.rs:196` | Fall-through target when `try_map` returns `Err(err)` |
| Phase 190 `widgets` fixture (`async_rule_fixture.rs`) | `framework/tests/async_rule_fixture.rs` | Reuse `init_test_db` + add a UNIQUE index for SQLite path tests |
| `#[serial]` from `serial_test` | `framework/Cargo.toml` dev-deps | DB singleton serialization; same pattern as Phase 190 |

---

## Architecture Patterns

### System Architecture Diagram

```
Handler (POST /pages)
        │
        ▼
AsyncValidator::validate_async()    ← Phase 190: proactive TOCTOU guard
        │
        ├─ Err(Validation) → ValidationError::with_old_input → into_action_error → 303
        ├─ Err(Infra)      → ActionError → 500
        │
        └─ Ok(())
                │
                ▼
        Model::insert(db).await
                │
                ├─ Ok(model)  ──────────────────────────────────────────────────► success
                │
                └─ Err(DbErr)
                        │
                        ▼
              map.try_map(err)          ← Phase 191: defensive TOCTOU closure
                        │
                        ├─ Ok(ValidationError)
                        │       │
                        │       └─ .with_old_input(&data).into_action_error(url) → 303
                        │
                        └─ Err(DbErr)  ─ unchanged ─► From<DbErr> for ActionError → 500
```

### Recommended Project Structure

```
framework/src/validation/
├── async_rule.rs          # Phase 190: AsyncRule trait
├── async_validator.rs     # Phase 190: AsyncValidator + AsyncValidationError
├── bridge.rs              # translate_validation()
├── constraint_map.rs      # NEW: ConstraintMap + MapConstraintExt
├── error.rs               # ValidationError (reused unchanged)
├── mod.rs                 # add pub use constraint_map::{ConstraintMap, MapConstraintExt}
├── rule.rs                # sync Rule trait
├── rules.rs               # sync rules
├── rules_async.rs         # Phase 190: Unique rule
├── validatable.rs
└── validator.rs

framework/src/lib.rs       # add ConstraintMap + MapConstraintExt to re-export block
framework/tests/
├── async_rule_fixture.rs  # reuse (add UNIQUE index helper)
└── constraint_map_integration.rs  # new: SQLite-backed integration tests
```

### Pattern 1: ConstraintMap Data Structures

```rust
// framework/src/validation/constraint_map.rs
// Source: inferred from CONTEXT.md D-01/D-07 + sea-orm error.rs (verified)

struct ConstraintEntry {
    /// Postgres structured constraint name (primary key in `.on()`).
    pg_name: String,
    /// SQLite `table.column` discriminator, set via `.sqlite()`.
    sqlite_key: Option<String>,
    /// Logical field name to attach the error to.
    field: String,
    /// User-visible error message.
    message: String,
}

pub struct ConstraintMap {
    entries: Vec<ConstraintEntry>,
}
```

### Pattern 2: Builder API

```rust
impl ConstraintMap {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Register a Postgres constraint name → field/message mapping.
    /// Primary key is the Postgres constraint name.
    pub fn on(
        mut self,
        pg_constraint: impl Into<String>,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        self.entries.push(ConstraintEntry {
            pg_name: pg_constraint.into(),
            sqlite_key: None,
            field: field.into(),
            message: message.into(),
        });
        self
    }

    /// Add a SQLite `table.column` discriminator to the LAST registered entry.
    /// Required for SQLite/dev environments to match the violation.
    pub fn sqlite(mut self, table_col: impl Into<String>) -> Self {
        if let Some(last) = self.entries.last_mut() {
            last.sqlite_key = Some(table_col.into());
        }
        self
    }
}
```

Example call site:
```rust
let map = ConstraintMap::new()
    .on("pages_slug_unique", "slug", "has already been taken")
    .sqlite("pages.slug");
```

### Pattern 3: try_map Internals (the load-bearing method)

```rust
// Source: verified from sea-orm-1.1.19/src/error.rs + sqlx-postgres-0.8.6/src/error.rs

impl ConstraintMap {
    pub fn try_map(&self, err: DbErr) -> Result<ValidationError, DbErr> {
        // Step 1: portable violation-type gate via sql_err().
        // Only SqlErr::UniqueConstraintViolation(_) proceeds to identity matching.
        // ALL other DbErr variants (connection errors, FK violations, etc.) fall
        // through immediately as Err(err) — no interception.
        match err.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {}
            _ => return Err(err),    // fall through unchanged
        }

        // Step 2: identity matching — backend-bifurcated.
        // Try Postgres path first: downcast the inner sqlx error.
        let pg_constraint = extract_pg_constraint(&err);

        // Try SQLite path: parse "table.column" from the message string.
        let sqlite_key = extract_sqlite_key(&err);

        // Step 3: find the first matching entry.
        for entry in &self.entries {
            let pg_match = pg_constraint.as_deref()
                .map(|c| c == entry.pg_name)
                .unwrap_or(false);
            let sqlite_match = sqlite_key.as_deref()
                .zip(entry.sqlite_key.as_deref())
                .map(|(key, registered)| key == registered)
                .unwrap_or(false);

            if pg_match || sqlite_match {
                let mut ve = ValidationError::new();
                ve.add(&entry.field, &entry.message);
                return Ok(ve);
            }
        }

        // No entry matched: fall through unchanged.
        Err(err)
    }
}
```

### Pattern 4: Postgres Constraint Name Extraction (VERIFIED path)

```rust
// Source: verified from sea-orm-1.1.19/src/error.rs and sqlx-postgres-0.8.6/src/error.rs.
//
// DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
// or DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
// where `e: Box<dyn sqlx::error::DatabaseError>`.
//
// `e.try_downcast_ref::<sqlx::postgres::PgDatabaseError>()` (from
// `dyn DatabaseError` inherent impl in sqlx-core-0.8.6/src/error.rs)
// returns Option<&PgDatabaseError>.
//
// PgDatabaseError::constraint() -> Option<&str> reads protocol field 'n'
// (the constraint name, per Postgres error protocol spec).
//
// NOTE: `constraint()` is also on the `DatabaseError` trait itself — no
// downcast is strictly necessary if we only need the constraint name.
// The trait default returns `None`; PgDatabaseError overrides it.
// Using the trait method avoids importing `sqlx::postgres` in non-Postgres builds.

#[cfg(feature = "sqlx-postgres")]
fn extract_pg_constraint(err: &DbErr) -> Option<String> {
    use sea_orm::DbErr;
    use sea_orm::RuntimeErr;
    match err {
        DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
        | DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e))) => {
            e.constraint().map(ToOwned::to_owned)
        }
        _ => None,
    }
}
```

**Key insight:** `DatabaseError::constraint()` is declared on the `dyn DatabaseError` trait (default `None`), overridden by `PgDatabaseError` to return the protocol field `'n'`. No downcast is needed if only the constraint name is required — calling `e.constraint()` on the trait object is sufficient and does not require importing `sqlx::postgres` in the final code.

### Pattern 5: SQLite Identity Extraction (VERIFIED message format)

```rust
// Source: SQLite error message format verified from sea-orm-1.1.19/src/error.rs
// (the UniqueConstraintViolation carries e.message() verbatim from sqlx-sqlite).
//
// SQLite message: "UNIQUE constraint failed: pages.slug"
// The `table.column` token is the substring after ": ".
//
// sea-orm sql_err() passes e.message() as the String in
// SqlErr::UniqueConstraintViolation(String). No additional downcast needed.

fn extract_sqlite_key(err: &DbErr) -> Option<String> {
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(msg)) => {
            // "UNIQUE constraint failed: pages.slug"
            msg.split(": ").nth(1).map(|s| s.trim().to_owned())
        }
        _ => None,
    }
}
```

**Critical detail:** The `String` inside `SqlErr::UniqueConstraintViolation(String)` is `e.message()` from the sqlx driver (confirmed from sea-orm source). For SQLite this is the human-readable message; for Postgres this is also the message, NOT the constraint name (the constraint name is in a separate protocol field). This asymmetry is why the SQLite path parses the message while the Postgres path calls `e.constraint()` separately.

### Pattern 6: Extension Trait for Ergonomic Call Sites

```rust
// MapConstraintExt on Result<T, DbErr> eliminates closure ladders at call sites.
// Source: inferred from CONTEXT.md code_context + D-02 + D-08.

pub trait MapConstraintExt<T> {
    /// Map a UNIQUE constraint violation to a field-level error and redirect.
    ///
    /// On match: flashes `ValidationError` + old input, returns `Err(ActionError)`
    ///   configured to redirect to `url` (303, no envelope toast).
    /// On no match: falls through to `From<DbErr> for ActionError` via `?`.
    fn map_constraint(
        self,
        map: &ConstraintMap,
        data: &serde_json::Value,
        url: impl Into<String>,
    ) -> Result<T, crate::http::action::ActionError>;
}

impl<T> MapConstraintExt<T> for Result<T, sea_orm::DbErr> {
    fn map_constraint(
        self,
        map: &ConstraintMap,
        data: &serde_json::Value,
        url: impl Into<String>,
    ) -> Result<T, crate::http::action::ActionError> {
        self.map_err(|err| match map.try_map(err) {
            Ok(ve) => ve.with_old_input(data).into_action_error(url),
            Err(original) => crate::http::action::ActionError::from(original),
        })
    }
}
```

Call site becomes:
```rust
let page = new_page.insert(db).await
    .map_constraint(&map, &data, "/pages/new")?;
```

This is the DX the phase exists to deliver: no closure ladder, no raw `map_err`, one method chain.

### Anti-Patterns to Avoid

- **Parsing the Postgres message string for the constraint name:** The Postgres message (`"duplicate key value violates unique constraint \"pages_slug_unique\""`) is not machine-stable across Postgres versions. Always use `PgDatabaseError::constraint()` (protocol field `'n'`), which is the structured wire value. [VERIFIED: sqlx-postgres-0.8.6 source]
- **Downcast to `PgDatabaseError` when `DatabaseError::constraint()` suffices:** The trait method returns the same value and avoids an `#[cfg(feature = "sqlx-postgres")]` guard for the common case.
- **Consuming the `DbErr` inside `try_map` before returning `Err`:** `DbErr` is not `Copy` but IS movable. The value is received by value in `try_map(err: DbErr)`, so `Err(err)` on no-match returns it unchanged without cloning.
- **Storing constraint/field/message literals in `framework`:** The project-agnostic-crates rule (CLAUDE.md + VALID-05) forbids it. Every string is consumer-owned. Reviewer check: no `"pages"`, `"slug"`, `"_unique"` literals in `constraint_map.rs` outside doc examples.
- **Making `try_map` take `&DbErr`:** Taking `DbErr` by value allows returning `Err(err)` unchanged without a clone. Borrow would force either a clone or a lifetime complication.
- **Global ConstraintMap registry:** Construct per call site (cheap) — avoids global mutable state, consistent with the project's established pattern.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Portable UNIQUE-violation type detection | Custom error-code parsing per backend | `DbErr::sql_err()` → `SqlErr::UniqueConstraintViolation` | Handles MySQL/Postgres/SQLite error codes correctly across backends [VERIFIED: sea-orm-1.1.19 source] |
| Postgres constraint name extraction | Regex on `DbErr::to_string()` message | `DatabaseError::constraint()` trait method (or `PgDatabaseError::constraint()`) | Protocol field `'n'` is the stable structured value; message format is not stable [VERIFIED: sqlx-postgres-0.8.6] |
| Field-level error flash round-trip | Custom session key scheme | `ValidationError::with_old_input().into_action_error()` | Phase 190-established chain; handles session namespacing, same-origin referer check [VERIFIED: codebase] |
| DB error → HTTP 500 fallback | New `ActionError` conversion | `From<sea_orm::DbErr> for ActionError` at `action.rs:196` | Already exists; `try_map` returning `Err(err)` lets `?` reach it automatically |

---

## Research Findings: Specific Questions from the Brief

### Q1: Postgres constraint-name extraction — exact match/downcast path

**Finding:** [VERIFIED: sqlx-postgres-0.8.6/src/error.rs, sqlx-core-0.8.6/src/error.rs, sea-orm-1.1.19/src/error.rs]

The downcast chain is:
```
DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
```
where `e: Box<dyn sqlx::error::DatabaseError>`.

`DatabaseError::constraint()` is declared on the trait itself (default `None`), overridden by `PgDatabaseError::constraint()` to return protocol field `'n'`. **No downcast is required.** Calling `e.constraint()` on the `Box<dyn DatabaseError>` dispatches to `PgDatabaseError::constraint()` at runtime when the backend is Postgres.

sea-orm `sql_err()` also confirms this path: it checks `e.try_downcast_ref::<sqlx::postgres::PgDatabaseError>().is_some()` before reading `_error_code_expanded` (SQLSTATE `"23505"`). The `try_downcast_ref` method is defined on `dyn DatabaseError` in sqlx-core.

**Implementation recommendation:** Use `e.constraint()` directly on `Box<dyn DatabaseError>` (the trait object) — no `try_downcast_ref` needed for the constraint-name-only case. This avoids an `#[cfg(feature = "sqlx-postgres")]` gate in the constraint-map helper.

The feature gate IS needed in sea-orm's `sql_err()` to detect Postgres violations; however, `framework/Cargo.toml` already enables both `sqlx-postgres` and `sqlx-sqlite` unconditionally, so `sql_err()` is always available for both backends.

### Q2: SQLite identity extraction — exact message format

**Finding:** [VERIFIED: sea-orm-1.1.19/src/error.rs, sqlx-sqlite-0.8.6/src/error.rs]

SQLite UNIQUE violation message from `e.message()` (via `sqlite3_errmsg`):
```
"UNIQUE constraint failed: table_name.column_name"
```

sea-orm stores this verbatim as the `String` payload of `SqlErr::UniqueConstraintViolation(String)`.

Parse: `msg.split(": ").nth(1)` yields `"table_name.column_name"`. Multiple-column UNIQUE constraints (rare) would yield `"table.col1, table.col2"` but this phase covers single-column UNIQUE indexes only (the slug case).

The consumer registers `.sqlite("pages.slug")` to match `"pages.slug"` extracted from the message.

**Note:** SQLite does NOT expose constraint names. The `table.column` token from the message is the only stable identifier. This is why D-06 bifurcates: Postgres by name, SQLite by message token.

### Q3: Match-key portability (D-07) — ergonomic recommendation

**Finding:** [ASSUMED — inferred from D-07 contract and builder convention]

Recommendation for the planner: use a chained `.sqlite("table.column")` modifier on the preceding `.on()` entry. This is consistent with the consuming-builder convention the project already uses (`with_*` methods). A `ConstraintId` value object is heavier than needed for two string fields.

The `.sqlite()` method mutates the last entry (or panics/no-ops if called without a prior `.on()`). Alternatively, `.sqlite()` could be a method on a builder-sub-type returned by `.on()`, but that adds type complexity. The mutable-last-entry approach is simpler and consistent with the established ferro builder style.

`try_map` decides which identifier to match against by trying both: for each registered entry, check if the Postgres constraint name matches (via `e.constraint()`) OR if the SQLite key matches (via message parsing). This avoids backend detection — the map simply tries all registered identifiers it has and matches the first hit.

### Q4: `try_map` internals and ordering — `DbErr` moveability confirmed

**Finding:** [VERIFIED: sea-orm-1.1.19/src/error.rs]

`DbErr` derives nothing that prevents move. `sql_err(&self)` borrows `self`. So `try_map(err: DbErr)` can:
1. Call `err.sql_err()` (borrows `err`) to get `Option<SqlErr>` — if not `UniqueConstraintViolation`, call `return Err(err)` (moves `err` out).
2. Extract the message string from `SqlErr::UniqueConstraintViolation(msg)` — BUT `sql_err()` creates a new `SqlErr` value from the data inside `err`, NOT from `err` itself; the original `err` is not consumed by `sql_err()`. After `sql_err()` returns, `err` is still live and movable.
3. Also borrow `err` to extract `e.constraint()` from the inner sqlx error (requires matching `err` by reference again).

**Recommended internal ordering:**

```rust
pub fn try_map(&self, err: DbErr) -> Result<ValidationError, DbErr> {
    // 1. Portable type gate — borrows err, does NOT consume it.
    if !matches!(err.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) {
        return Err(err);  // NOT a UNIQUE violation — fall through unchanged
    }

    // 2a. Extract Postgres constraint name by borrowing err's inner error.
    let pg_name: Option<String> = match &err {
        DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
        | DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e))) => {
            e.constraint().map(ToOwned::to_owned)
        }
        _ => None,
    };

    // 2b. Extract SQLite key from the SqlErr message (sql_err() borrows err again).
    let sqlite_key: Option<String> = match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(msg)) => {
            msg.split(": ").nth(1).map(|s| s.trim().to_owned())
        }
        _ => None,
    };

    // 3. Identity match against registered entries.
    for entry in &self.entries {
        let pg_hit = pg_name.as_deref().map(|c| c == entry.pg_name).unwrap_or(false);
        let sqlite_hit = sqlite_key.as_deref()
            .zip(entry.sqlite_key.as_deref())
            .map(|(key, reg)| key == reg)
            .unwrap_or(false);
        if pg_hit || sqlite_hit {
            let mut ve = ValidationError::new();
            ve.add(&entry.field, &entry.message);
            return Ok(ve);
        }
    }

    // 4. No entry matched — fall through unchanged.
    Err(err)
}
```

`sql_err()` is called twice (type gate + SQLite extraction). Both calls borrow `err` and create a new `SqlErr` from its data — this is lightweight and avoids storing intermediate values.

**Important:** `RuntimeErr` must be imported from `sea_orm` (it is `pub` in sea-orm's error module). The match arm `DbErr::Exec(RuntimeErr::SqlxError(...))` requires `RuntimeErr` to be in scope. sea-orm re-exports it as `sea_orm::RuntimeErr`.

### Q5: Ergonomic call site — extension trait recommendation

**Finding:** [ASSUMED — inferred from CONTEXT.md code_context + established ferro handler patterns]

The CONTEXT.md code_context section shows the "closure ladder" problem:
```rust
record.insert(db).await.map_err(|e| map.try_map(e)
    .map(|ve| ve.with_old_input(&data).into_action_error(url))
    .unwrap_or_else(ActionError::from))?
```

This is ergonomically poor. The recommended solution: a `MapConstraintExt` extension trait on `Result<T, DbErr>` with a `map_constraint` method. This is the same pattern ferro uses for `ActionResultExt::action_err`.

The extension trait should live in `constraint_map.rs` alongside `ConstraintMap` and be re-exported at the same path.

Call site becomes:
```rust
let page = new_page.insert(db).await
    .map_constraint(&map, &data, "/pages/new")?;
```

Clean, single chain, idiomatic ferro.

### Q6: Concurrent-insert simulation (SC4)

**Finding:** [VERIFIED: sea-orm-1.1.19 source, existing test fixture in `async_rule_fixture.rs`]

The Phase 190 `widgets` table does NOT have a UNIQUE index on `slug` by default (only `id` is PK). For Phase 191 tests, we need to add one:

```sql
CREATE TABLE IF NOT EXISTS constraint_widgets (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL,
    UNIQUE(slug)
)
```

Or: add `CREATE UNIQUE INDEX IF NOT EXISTS constraint_widgets_slug_unique ON constraint_widgets (slug)` after creating the table.

Test shape (deterministic concurrent-insert simulation per D-11):
```rust
#[tokio::test]
#[serial]
async fn concurrent_insert_toctou_simulation() {
    init_constraint_db().await;  // creates table with UNIQUE index
    // Seed the "winning" insert
    db.execute("INSERT INTO constraint_widgets (id, slug) VALUES (1, 'taken')").await;
    // Simulate the "losing" insert — attempt a duplicate
    let result = db.execute("INSERT INTO constraint_widgets (id, slug) VALUES (2, 'taken')").await;
    let err = result.unwrap_err();  // DbErr from UNIQUE violation
    let map = ConstraintMap::new()
        .on("constraint_widgets_slug_unique", "slug", "has already been taken")
        .sqlite("constraint_widgets.slug");
    let ve = map.try_map(err).expect("should match");
    assert!(ve.has("slug"));
}
```

The SQLite message for a UNIQUE INDEX (not PK) violation uses code 2067 (`SQLITE_CONSTRAINT_UNIQUE`). sea-orm's `sql_err()` handles this (matches `"2067"`) — confirmed from sea-orm source. The message format is `"UNIQUE constraint failed: constraint_widgets.slug"`.

### Q7: Project-agnostic rule (VALID-05 / SC5)

**Finding:** [VERIFIED: CLAUDE.md project-agnostic-crates rule + CONTEXT.md D-09]

`ConstraintMap` holds all strings as consumer-provided `String` values. The `framework` crate contributes only:
- The `ConstraintMap` struct and builder methods (generic, no embedded strings)
- The `try_map` logic (generic)
- The `MapConstraintExt` trait (generic)

No `"pages"`, `"slug"`, `"_unique"` literals appear in the framework code except in documentation examples explicitly framed as samples (which is the sole exception per CLAUDE.md).

---

## Common Pitfalls

### Pitfall 1: Calling `sql_err()` After Moving `err`

**What goes wrong:** If `err` is consumed by `Err(err)` return before extracting the SQLite key, the code won't compile. Alternatively, if `sql_err()` is called on a moved value, it won't compile.

**Why it happens:** `sql_err()` borrows; `Err(err)` moves. The ordering matters.

**How to avoid:** Call `sql_err()` (borrow) before any move. The "type gate first, identity extraction second, return Err(err) at end" ordering in Pattern 3 above is correct. Alternatively, extract `sql_err()` result at the top, handle all non-match returns, then proceed to identity extraction.

**Warning signs:** Borrow-after-move compiler errors in `try_map`.

### Pitfall 2: SQLite UNIQUE Index vs Primary Key Constraint Names

**What goes wrong:** SQLite `PRIMARY KEY` violations use code 1555 (`SQLITE_CONSTRAINT_PRIMARYKEY`), UNIQUE index violations use 2067 (`SQLITE_CONSTRAINT_UNIQUE`). sea-orm handles both in `sql_err()`, but the message formats differ slightly.

**Why it happens:** The message for a PK violation may say `"UNIQUE constraint failed: table.id"` (using the column name) rather than a named index.

**How to avoid:** Phase 191 targets UNIQUE index violations on non-PK columns (the slug case). Test with a named UNIQUE INDEX (`CREATE UNIQUE INDEX ... ON table(col)`), not a PK. Document this in test comments.

**Warning signs:** `try_map` not matching when a UNIQUE index is on the PK column.

### Pitfall 3: Postgres Constraint Name Not Matching Registration

**What goes wrong:** Consumer registers `"pages_slug_unique"` but the Postgres index name is `"pages_slug_key"` (the Postgres default for `UNIQUE` column constraints created without an explicit name).

**Why it happens:** Postgres auto-generates constraint names as `{table}_{col}_key` for inline UNIQUE constraints, but most migrations create explicit names with `CREATE UNIQUE INDEX {name} ON ...`.

**How to avoid:** Document in the rustdoc that the registered name MUST match the exact Postgres constraint/index name from the schema. The consumer should verify with `\d+ table_name` in psql or inspect `pg_constraint.conname`. This is a user-education issue, not a framework bug.

**Warning signs:** `try_map` returns `Err(DbErr)` unchanged on Postgres when a UNIQUE violation occurs.

### Pitfall 4: Missing `RuntimeErr` Import in `try_map`

**What goes wrong:** The match arm `DbErr::Exec(RuntimeErr::SqlxError(...))` requires `RuntimeErr` in scope. If only `DbErr` is imported, the pattern is incomplete.

**Why it happens:** `RuntimeErr` is a separate type in `sea_orm::error`, not part of `DbErr`'s namespace.

**How to avoid:** `use sea_orm::{DbErr, RuntimeErr};` at the top of `constraint_map.rs`. Or use `sea_orm::error::RuntimeErr`.

### Pitfall 5: `#[cfg(feature = "sqlx-postgres")]` Guards on the Downcast

**What goes wrong:** If the Postgres constraint-name extraction is gated behind `#[cfg(feature = "sqlx-postgres")]`, the SQLite build will fail to compile the `pg_name` extraction block.

**Why it happens:** `PgDatabaseError` type only exists when the postgres feature is enabled.

**How to avoid:** Use `DatabaseError::constraint()` on the trait object (`e.constraint()` on `Box<dyn DatabaseError>`) — no feature gate needed. This is the recommended approach (Q1 finding above). If a downcast to `PgDatabaseError` is preferred, add the `#[cfg]` guard correctly.

---

## Runtime State Inventory

Step 2.6: SKIPPED — Phase 191 is a new-code addition to `framework/src/validation/`. No rename, refactor, or migration. No runtime state to audit.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| SQLite (in-memory) | SQLite test path (SC1–SC3, SC4) | ✓ | Bundled via `libsqlite3-sys` (sqlx-sqlite) | — |
| Postgres instance | Postgres identity-match path (SC3 Postgres side) | ✗ (no local Postgres in default CI) | — | Documented manual gate (D-12) |
| `cargo test` | Phase verification | ✓ | Rust toolchain | — |

**Missing dependencies with no fallback:** None that block the SQLite-testable path (SC1, SC2, SC3 SQLite side, SC4, SC5 are all fully `cargo test`-able).

**Missing dependencies with manual gate:** Postgres instance for SC3 Postgres side. Documented in the ROADMAP: "Phase closure criteria include either a Postgres CI step or a documented manual test step signed off in the phase VERIFICATION.md."

---

## Code Examples

### Complete `try_map` (SQLite-testable, Postgres-prepared)

```rust
// Source: verified from sea-orm-1.1.19/src/error.rs + sqlx-core-0.8.6/src/error.rs
use sea_orm::{DbErr, RuntimeErr};
use sea_orm::error::{SqlErr};

pub fn try_map(&self, err: DbErr) -> Result<ValidationError, DbErr> {
    // Gate 1: portable UNIQUE violation type check (borrows err).
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => {} // proceed
        _ => return Err(err),                            // not a UNIQUE violation
    }

    // Gate 2a: Postgres constraint name (from inner DatabaseError trait object).
    let pg_name: Option<String> = match &err {
        DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
        | DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e))) => {
            e.constraint().map(ToOwned::to_owned)
        }
        _ => None,
    };

    // Gate 2b: SQLite table.column from message string.
    let sqlite_key: Option<String> = match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(msg)) => {
            // "UNIQUE constraint failed: table.column" → "table.column"
            msg.split(": ").nth(1).map(|s| s.trim().to_owned())
        }
        _ => None,
    };

    // Gate 3: entry lookup.
    for entry in &self.entries {
        let pg_hit = pg_name.as_deref()
            .map(|c| c == entry.pg_name).unwrap_or(false);
        let sqlite_hit = sqlite_key.as_deref()
            .zip(entry.sqlite_key.as_deref())
            .map(|(k, r)| k == r).unwrap_or(false);
        if pg_hit || sqlite_hit {
            let mut ve = ValidationError::new();
            ve.add(&entry.field, &entry.message);
            return Ok(ve);
        }
    }

    Err(err) // no entry matched — fall through unchanged
}
```

### SQLite Integration Test (SC4 — concurrent-insert simulation)

```rust
// framework/tests/constraint_map_integration.rs
use ferro_rs::validation::{ConstraintMap, ValidationError};
use sea_orm::{ConnectionTrait, Statement};
use serial_test::serial;

async fn init_constraint_db() {
    use ferro_rs::database::{DatabaseConfig, DB};
    let config = DatabaseConfig::builder().url("sqlite::memory:").build();
    DB::init_with(config).await.expect("init");
    let db = DB::connection().expect("connection");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS cw (id INTEGER PRIMARY KEY, slug TEXT NOT NULL)".into(),
    )).await.expect("create table");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE UNIQUE INDEX IF NOT EXISTS cw_slug_unique ON cw (slug)".into(),
    )).await.expect("create unique index");
}

#[tokio::test]
#[serial]
async fn toctou_simulation_maps_to_field_error() {
    init_constraint_db().await;
    let db = ferro_rs::database::DB::connection().expect("connection");
    db.execute(Statement::from_string(db.get_database_backend(),
        "INSERT INTO cw (id, slug) VALUES (1, 'taken')".into(),
    )).await.expect("seed");

    // Simulate the "losing" concurrent insert.
    let result = db.execute(Statement::from_string(db.get_database_backend(),
        "INSERT INTO cw (id, slug) VALUES (2, 'taken')".into(),
    )).await;
    let err = result.expect_err("expected UNIQUE violation");

    let map = ConstraintMap::new()
        .on("cw_slug_unique", "slug", "has already been taken")
        .sqlite("cw.slug");

    match map.try_map(err) {
        Ok(ve) => {
            assert!(ve.has("slug"), "expected 'slug' field error");
        }
        Err(e) => panic!("expected Ok(ValidationError), got Err({e})"),
    }
}

#[tokio::test]
#[serial]
async fn non_unique_error_passes_through_unchanged() {
    init_constraint_db().await;
    // A FK violation or connection error must pass through.
    // Construct a non-UNIQUE DbErr manually (custom error for test isolation).
    let err = sea_orm::DbErr::Custom("some other error".to_string());
    let map = ConstraintMap::new()
        .on("cw_slug_unique", "slug", "has already been taken")
        .sqlite("cw.slug");
    match map.try_map(err) {
        Err(sea_orm::DbErr::Custom(msg)) => {
            assert_eq!(msg, "some other error");
        }
        other => panic!("expected Err(DbErr::Custom), got {other:?}"),
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|-----------------|--------------|--------|
| Raw `From<DbErr> for ActionError` passthrough (raw SQL error to user) | `ConstraintMap::try_map` intercepts before the passthrough | Phase 191 | User sees inline field error instead of raw SQL |
| No UNIQUE constraint name API in sqlx | `DatabaseError::constraint()` trait method returning Postgres field `'n'` | sqlx 0.8.x | Structured, message-format-independent constraint name extraction |
| `sql_err()` not available in sea-orm < 1.1 | `DbErr::sql_err() -> Option<SqlErr>` available since sea-orm 1.1 | sea-orm 1.1 | Portable violation-type detection without per-backend code |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `.sqlite("table.column")` chained modifier is the best ergonomic spelling for D-07 | Match-key portability finding (Q3) | Planner may choose `.on_sqlite()` sibling or `ConstraintId`; behavioral contract is identical; API surface changes but implementation is equivalent |
| A2 | `MapConstraintExt` extension trait is the right ergonomic wrapper (vs returning `Result<T, Either<ValidationError, DbErr>>`) | Q5 finding | If planner prefers a different call-site shape, the DX argument holds for any shape that avoids the closure ladder |
| A3 | SQLite UNIQUE INDEX message format is `"UNIQUE constraint failed: table.column"` (verified from sea-orm source, not from a live test) | Q2 finding | If SQLite message format varies by version or configuration, the parse logic may need adjustment; risk is LOW given verification from sqlx source |

**All other claims are VERIFIED from codebase sources (sea-orm-1.1.19, sqlx-core-0.8.6, sqlx-postgres-0.8.6, sqlx-sqlite-0.8.6, framework source files).**

---

## Open Questions (RESOLVED)

1. **Multiple UNIQUE columns in one table entry** — RESOLVED: first registered match wins (order of `.on()` calls = priority); document this in the `ConstraintMap` rustdoc. Plan 01 implements the sequential entry-list match and the rustdoc note.
   - What we know: The `.sqlite()` modifier stores a single `table.column` string; `try_map` matches on exact string equality.

2. **`MapConstraintExt` import at crate root vs validation module** — RESOLVED: `Result<T, sea_orm::DbErr>` is external but `MapConstraintExt` is a local trait, so the impl is sound under Rust's orphan rules. Plan 01 re-exports `MapConstraintExt` alongside `ConstraintMap` at both `validation::` (mod.rs) and crate root (lib.rs).

---

## Validation Architecture

> `workflow.nyquist_validation` key is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `tokio::test` + `serial_test` (already in dev-deps) |
| Config file | None (Rust test harness) |
| Quick run command | `cargo test -p ferro-rs constraint_map` |
| Full suite command | `cargo test --all-features -p ferro-rs` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VALID-04 (SC1) | `try_map` returns `Ok(ValidationError)` on matching UNIQUE violation | unit / integration | `cargo test -p ferro-rs try_map` | ❌ Wave 0 |
| VALID-04 (SC2) | Non-UNIQUE `DbErr` passes through `try_map` unchanged | unit | `cargo test -p ferro-rs non_unique_passes_through` | ❌ Wave 0 |
| VALID-05 (SC3 SQLite) | SQLite `table.column` identity match via message parsing | integration | `cargo test -p ferro-rs sqlite_identity_match` | ❌ Wave 0 |
| VALID-05 (SC3 Postgres) | Postgres constraint name via `PgDatabaseError::constraint()` | manual gate | `191-VERIFICATION.md` sign-off | N/A — manual |
| VALID-04 (SC4) | Concurrent-insert simulation (duplicate INSERT → `try_map` → field error) | integration | `cargo test -p ferro-rs toctou_simulation` | ❌ Wave 0 |
| VALID-05 (SC5) | No consumer strings in `framework/src/validation/constraint_map.rs` | audit | `grep -n '"pages"\|"slug"\|"_unique"' framework/src/validation/constraint_map.rs` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-rs constraint_map -p ferro-rs -- --test-threads=1`
- **Per wave merge:** `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features -p ferro-rs`
- **Phase gate:** Full suite green + Postgres manual gate signed in `191-VERIFICATION.md`

### Wave 0 Gaps

- [ ] `framework/src/validation/constraint_map.rs` — new file; covers VALID-04, VALID-05
- [ ] `framework/tests/constraint_map_integration.rs` — SQLite integration + simulation tests; covers SC1–SC4
- [ ] Shared fixture extension: add `init_constraint_db` helper (table with UNIQUE INDEX) to `async_rule_fixture.rs` or a new `constraint_map_fixture.rs`

---

## Security Domain

> `security_enforcement` absent from config — treated as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | No | — |
| V5 Input Validation | Yes | `ConstraintMap` strings are developer-owned (not user-supplied); no injection surface |
| V6 Cryptography | No | — |

### Known Threat Patterns for This Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| `ConstraintMap` strings used as SQL identifiers | Tampering | N/A — `ConstraintMap` strings are NEVER used as SQL identifiers; they are matched against error metadata only. No SQL construction. |
| Swallowing DB errors that indicate data integrity failures | Tampering / Information Disclosure | `try_map` returns `Err(err)` unchanged on any non-match; the existing `From<DbErr> for ActionError` passthrough is reached. No error is ever discarded. |
| Exposing raw DB error messages to end users | Information Disclosure | `try_map` replaces the raw `DbErr` with a consumer-provided human message on match; on no-match the existing `ActionError::msg(err.to_string())` applies (current behavior, unchanged by this phase). |

---

## Sources

### Primary (HIGH confidence)
- `sea-orm-1.1.19/src/error.rs` — `DbErr`, `SqlErr`, `RuntimeErr`, `sql_err()` implementation (verified from local cargo registry)
- `sqlx-postgres-0.8.6/src/error.rs` — `PgDatabaseError::constraint()` returning protocol field `'n'` (verified from local cargo registry)
- `sqlx-core-0.8.6/src/error.rs` — `DatabaseError::constraint()` trait method (default `None`), `try_downcast_ref` on `dyn DatabaseError` (verified from local cargo registry)
- `sqlx-sqlite-0.8.6/src/error.rs` — `SqliteError` message format (indirectly, via `sqlite3_errmsg`) (verified)
- `framework/src/validation/error.rs` — `ValidationError` API, `with_old_input`, `into_action_error` (verified from codebase)
- `framework/src/http/action.rs:196` — `From<sea_orm::DbErr> for ActionError` (verified from codebase)
- `framework/src/validation/mod.rs` + `framework/src/lib.rs` — current re-export chain for Phase 190 symbols (verified from codebase)
- `framework/tests/async_rule_fixture.rs` — `init_test_db`, `seed_widget` fixture helpers (verified from codebase)
- `framework/src/validation/rules_async.rs` — Phase 190 `Unique` rule patterns to stay consistent with (verified from codebase)
- `.planning/ROADMAP.md` § Phase 191 — 5 success criteria and Postgres manual gate (verified from project planning files)

### Secondary (MEDIUM confidence)
- `.planning/phases/191-constraintmap-portable-unique-violation-detection/191-CONTEXT.md` — locked decisions D-01..D-12

### Tertiary (LOW confidence — ASSUMED)
- Ergonomic recommendation for `.sqlite()` modifier spelling (Q3) — inferred from project builder conventions
- `MapConstraintExt` trait as the call-site ergonomic wrapper (Q5) — inferred from `ActionResultExt` precedent in `action.rs`

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all required types verified from local cargo registry source
- Architecture: HIGH — `ConstraintMap` shape locked by CONTEXT.md; implementation path verified from sea-orm + sqlx sources
- Pitfalls: HIGH — derived from actual source inspection of error chains and builder patterns
- Postgres path: MEDIUM — source-verified but cannot be exercised without a live Postgres instance; manual gate required

**Research date:** 2026-06-09
**Valid until:** 2026-09-09 (stable — sea-orm 1.x / sqlx 0.8.x; no fast-moving surface)
