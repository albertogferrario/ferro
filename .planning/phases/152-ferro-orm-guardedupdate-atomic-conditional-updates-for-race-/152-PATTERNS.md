# Phase 152: ferro-orm GuardedUpdate — Pattern Map

**Mapped:** 2026-05-13
**Files analyzed:** 7 new files (5 source + 1 integration test + 1 README + 1 doc page) + 5 workspace/doc edits
**Analogs found:** 6 strong analogs in `ferro-wallet/`, `ferro-events/`, `ferro-stripe/`, `framework/src/database/`, root config files
**No-analog files (NEW PATTERN):** 2 — `src/guarded.rs` body (SeaORM extension; nearest precedent is `framework/src/database/query_builder.rs`, SELECT-side rather than UPDATE-side) and `tests/concurrent_decrement.rs` (no prior tokio-multi-task race test in the workspace; RESEARCH.md §"Code Examples" / Example 5 is the authoritative source)

---

## File Classification

### New files

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `ferro-orm/Cargo.toml` | crate-bootstrap (Wave 1a leaf manifest) | build | `ferro-wallet/Cargo.toml` (newer pattern, `version.workspace = true`, Wave 1a metadata) | exact |
| `ferro-orm/src/lib.rs` | crate-bootstrap (module rustdoc + targeted re-exports) | n/a | `ferro-events/src/lib.rs` (single-primitive crate rustdoc shape) + `ferro-wallet/src/lib.rs` (re-export block shape) | exact (combines both) |
| `ferro-orm/src/error.rs` | error-type (`thiserror` enum, one per crate) | n/a | `ferro-wallet/src/error.rs` (exact pattern: `"prefix: {0}"` Display + per-variant `#[cfg(test)]`) | exact |
| `ferro-orm/src/guarded.rs` | builder-core (consuming-builder + `ConnectionTrait` generic) | request-response (single `UPDATE … WHERE …` round trip) | `framework/src/database/query_builder.rs` lines 58-92 (filter chaining + `IntoCondition`) — SELECT side, partial role-match | role-partial (NEW PATTERN — UPDATE side is new) |
| `ferro-orm/tests/concurrent_decrement.rs` | integration-test (multi-task tokio race) | event-driven (concurrent tasks) | `ferro-stripe/tests/parser_contract.rs` (integration-test layout + `use ferro_…::…` imports) | role-partial (layout only — race-test shape is NEW PATTERN, see RESEARCH.md Example 5) |
| `ferro-orm/README.md` | readme | n/a | `ferro-wallet/README.md` (one-paragraph crate purpose + `docs.rs` link, ~10 lines) | exact |
| `docs/src/database/atomic-updates.md` | doc-page (user-facing mdBook page) | n/a | `docs/src/features/database.md` lines 1-80 (sibling DB doc; same code-block tone) + `docs/src/features/events.md` lines 1-60 (concept→example→requirements flow) | role-match |

### Modified files (workspace/config edits)

| Modified File | Role | Analog (precedent commit/pattern) | Match Quality |
|---------------|------|------------------------------------|---------------|
| `Cargo.toml` (workspace root) | workspace-member-edit (append `"ferro-orm",` to `[workspace.members]`) | line 24 was the Phase 151 append of `"ferro-wallet",` | exact |
| `.github/workflows/publish.yml` | publish-config-edit (append `ferro-orm` to `WAVE1A_CRATES` env string, line 201) | line 201 was extended by Phase 151 to append `ferro-wallet` | exact |
| `CHANGELOG.md` | changelog-edit (add new `## ferro-orm` top-level section with `### [version] — YYYY-MM-DD` entry) | lines 6-35 (the existing `## ferro-wallet` section from Phase 151) | exact |
| `CLAUDE.md` | claude-md-edit (add `ferro-orm` row to Workspace Structure table, line ~58 after `ferro-whatsapp`) | each existing row in the table is the analog | exact |
| `docs/src/SUMMARY.md` | mdbook-nav-edit (add link to `database/atomic-updates.md`) | each existing entry under `# Features` (lines 19-46) | role-match (see Pitfall callout below — sub-page nesting decision required) |

---

## Pattern Assignments

---

### `ferro-orm/Cargo.toml` (crate-bootstrap)

**Analog:** `ferro-wallet/Cargo.toml` lines 1-25 — Wave 1a leaf-crate manifest with workspace-inherited package fields, `homepage = "https://ferro-rs.dev"`, explicit (non-`workspace = true`) dep versions.

**Why this analog over `ferro-events/Cargo.toml`:** ferro-wallet is the *most recent* Wave 1a addition (Phase 151) and explicitly includes the `homepage` line. ferro-events is structurally similar but lacks `homepage` — using the newer pattern keeps the metadata block consistent across recently-added crates.

**`[package]` header pattern** (`ferro-wallet/Cargo.toml` lines 1-11):

```toml
[package]
name = "ferro-wallet"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Digital wallet pass issuance (Apple .pkpass + Google Wallet) for the Ferro framework"
repository = "https://github.com/albertogferrario/repositories/albertogferrario/ferro"
keywords = ["wallet", "pkpass", "google-wallet", "apple-wallet", "ferro"]
categories = ["web-programming"]
readme = "README.md"
homepage = "https://ferro-rs.dev"
```

**Adapt for ferro-orm** (per RESEARCH.md §"Installation" + D-04):

```toml
[package]
name = "ferro-orm"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Atomic conditional updates and ORM primitives for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["orm", "sea-orm", "atomic", "concurrency", "ferro"]
categories = ["database"]
readme = "README.md"
homepage = "https://ferro-rs.dev"

[dependencies]
sea-orm = "1.0"
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```

**What changes vs `ferro-wallet/Cargo.toml`:**

- `name` → `ferro-orm`; `description` → SeaORM-focused.
- `keywords` and `categories` → `["orm", …]` / `["database"]` (not `["web-programming"]`).
- `[dependencies]` slimmed to two crates (sea-orm + thiserror) — ferro-wallet's 10-line dep block does not apply.
- New `[dev-dependencies]` block (ferro-wallet has none) — required for the in-memory SQLite tests (D-16/D-17). Match `ferro-events/Cargo.toml` line 21 for the `tokio = { version = "1", features = ["full", "test-util"] }` line; add a sea-orm dev-dep with sqlite + runtime-tokio-native-tls + macros features. **RESEARCH.md Pitfall 3** is load-bearing here — `framework` uses `runtime-tokio-native-tls`; matching it avoids sqlx feature-collision under `cargo test --all-features`.

**Pitfall callouts:**

- RESEARCH.md Pitfall 3 — sea-orm runtime feature **must** match `framework`'s `runtime-tokio-native-tls`, not the `runtime-tokio-rustls` variant mentioned earlier in the research (RESEARCH.md §"Standard Stack" recommends Option A explicitly).
- RESEARCH.md A6 / Code Example 4 — `Schema::create_table_from_entity` is the test-schema pattern; **do not** add `sea-orm-migration` to `[dev-dependencies]`. If the Schema API surface turns out to be insufficient at implementation time, fall back to a raw `Statement::from_string` + `ConnectionTrait::execute` (still no migration dep).
- Library `[dependencies] sea-orm = "1.0"` declares **no features** — the consumer provides driver + runtime at link time. This is intentional and verified in RESEARCH.md §"Installation".

---

### `ferro-orm/src/lib.rs` (crate-bootstrap)

**Analog (module rustdoc shape):** `ferro-events/src/lib.rs` lines 1-44 — single-primitive crate with a `# Ferro …` header, a one-paragraph purpose, an `## Example` `rust,ignore` block, then `mod` declarations + `pub use` re-exports.

**Analog (re-export block shape):** `ferro-wallet/src/lib.rs` lines 7-22 — `pub mod …` (per file split) followed by a `pub use …::…` block.

**Rustdoc pattern** (`ferro-events/src/lib.rs` lines 1-44):

```rust
//! # Ferro Events
//!
//! Event dispatcher and listener system for the Ferro framework.
//!
//! Provides a Laravel-inspired event system with support for:
//! - …
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_events::{Event, Listener, Error};
//! …
//! ```
```

**Module declaration + re-export pattern** (`ferro-wallet/src/lib.rs` lines 7-17):

```rust
pub mod apple;
pub mod config;
pub mod error;
// …

pub use apple::ApplePassBuilder;
pub use config::{AppleConfig, GoogleConfig, WalletConfig};
pub use error::WalletError;
```

**Adapt for ferro-orm** (per RESEARCH.md §"Pattern 2: Targeted re-exports (D-03)" + Open Question 2/3 — re-export `Expr` and `IntoCondition` at the crate root):

```rust
//! # ferro-orm
//!
//! Atomic conditional updates and ORM primitives for the Ferro framework.
//!
//! `GuardedUpdate<E>` compiles to a single `UPDATE … WHERE …` SQL statement,
//! replacing the hand-rolled `read → check → write` pattern wherever a column's
//! value is conditionally mutated. The database is the authority on contention;
//! the call site is race-free by construction.
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_orm::{GuardedUpdate, ColumnTrait};
//! use sea_orm::sea_query::Expr;
//!
//! GuardedUpdate::new(inventory_units::Entity)
//!     .filter(inventory_units::Column::Id.eq(unit_id))
//!     .filter(inventory_units::Column::Quantity.gte(needed))
//!     .set_expr(
//!         inventory_units::Column::Quantity,
//!         Expr::col(inventory_units::Column::Quantity).sub(needed),
//!     )
//!     .exec_one(&txn)
//!     .await?;
//! // — exactly one row matched and was decremented atomically,
//! //   OR Err(NoRowsAffected) signalling capacity exhausted.
//! ```
//!
//! ## Atomicity guarantee
//!
//! `GuardedUpdate` guarantees atomicity *per statement*, not per builder.
//! A caller building `.set_expr(qty - 1)` and reading the resulting `qty`
//! in a separate query without a transaction re-introduces a race. The crate's
//! job is to make the conditional UPDATE race-free; bracketing it in a
//! transaction is the caller's job.

mod error;
mod guarded;

pub use error::GuardedError;
pub use guarded::GuardedUpdate;

// Targeted re-exports — consumers calling the builder need these.
// Do NOT add `pub use sea_orm::*` per D-03.
pub use sea_orm::sea_query::{Expr, IntoCondition, SimpleExpr, Value};
pub use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait};
```

**What changes vs the analogs:**

- ferro-events uses `mod dispatcher; mod error; mod traits;` (private modules + `pub use`) — match this shape, **not** ferro-wallet's `pub mod …` (the ferro-orm internals are an implementation detail).
- Add the **atomicity-per-statement footgun callout** to the rustdoc directly (D-15 / D-20) — this is the load-bearing misuse that prose alone prevents.
- Re-export `Expr` at the crate root per RESEARCH.md Open Question 3 (the canonical example needs it; D-03's list was illustrative, not exhaustive).

**Pitfall callouts:**

- D-03 forbids `pub use sea_orm::*`. The re-export list is whitelist-only; if a planner adds a new symbol they must justify it.
- The example block uses `rust,ignore` not `rust` — ferro-orm has no test entity in its public API, so a runnable doctest would require a synthetic entity. Match ferro-events' `rust,ignore` precedent.

---

### `ferro-orm/src/error.rs` (error-type)

**Analog:** `ferro-wallet/src/error.rs` lines 1-100 — exact pattern: file-level `//!` doc, `#[derive(Debug, thiserror::Error)]`, per-variant `///` doc + `#[error("prefix: …")]`, `#[from] std::io::Error` for plumbing, exhaustive `#[cfg(test)] mod tests` block with one `#[test] fn …_displays_message()` per variant.

**File-level doc + enum pattern** (`ferro-wallet/src/error.rs` lines 1-40):

```rust
//! `WalletError` — the single error type for the ferro-wallet crate.
//!
//! Each variant's `Display` impl prefixes its name (`"config: …"`, `"apple sign: …"`)
//! so production log greps stay surgical.

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    /// Configuration error (missing env var or invalid value).
    #[error("config: {0}")]
    Config(String),

    /// Apple PKCS#7 signing failed (cert/key parse, sign call, DER serialization).
    #[error("apple sign: {0}")]
    AppleSign(String),
    // …
    /// I/O error — zip writer, file handles, etc.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
```

**Test pattern** (`ferro-wallet/src/error.rs` lines 42-99):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_config_displays_message() {
        let e = WalletError::Config("APP_NAME not set".into());
        assert_eq!(e.to_string(), "config: APP_NAME not set");
    }
    // … one #[test] per variant

    #[test]
    fn io_from_std_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let wallet_err: WalletError = WalletError::from(io_err);
        assert!(matches!(wallet_err, WalletError::Io(_)));
    }
}
```

**Adapt for ferro-orm** (per D-11/D-12/D-13 and RESEARCH.md §"Pattern 3: GuardedError shape"):

```rust
//! `GuardedError` — the single error type for the ferro-orm crate.
//!
//! Every variant's `Display` impl prefixes `"guarded: …"` so production log greps
//! stay surgical (matches the workspace convention used by `WalletError`,
//! `ConfigError`, etc.).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuardedError {
    /// The conditional UPDATE matched zero rows — the predicate was not satisfied.
    /// For counter mutations this is the load-bearing "capacity exhausted" signal.
    #[error("guarded: predicate matched no rows")]
    NoRowsAffected,

    /// The conditional UPDATE matched more than one row — every guarded update is
    /// morally a unique-key-equivalent operation; >1 indicates an index/uniqueness bug.
    #[error("guarded: predicate matched {affected} rows (expected 1) — likely an index/uniqueness bug")]
    TooManyRows { affected: u64 },

    /// The builder was executed with no `set_*` calls — a programming error.
    /// Without this guard, sea-orm's `Updater::exec` short-circuits with
    /// `rows_affected: 0`, which would silently look like a predicate miss.
    #[error("guarded: no columns to set — builder is empty")]
    EmptyUpdate,

    /// Underlying SeaORM database error.
    #[error("guarded: db error: {0}")]
    Db(#[from] sea_orm::DbErr),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_no_rows_affected_displays_message() {
        assert_eq!(
            GuardedError::NoRowsAffected.to_string(),
            "guarded: predicate matched no rows"
        );
    }

    #[test]
    fn error_too_many_rows_displays_message() {
        assert_eq!(
            GuardedError::TooManyRows { affected: 3 }.to_string(),
            "guarded: predicate matched 3 rows (expected 1) — likely an index/uniqueness bug"
        );
    }

    #[test]
    fn error_empty_update_displays_message() {
        assert_eq!(
            GuardedError::EmptyUpdate.to_string(),
            "guarded: no columns to set — builder is empty"
        );
    }

    #[test]
    fn db_from_sea_orm_dberr() {
        let db_err = sea_orm::DbErr::Custom("test".into());
        let guarded_err: GuardedError = GuardedError::from(db_err);
        assert!(matches!(guarded_err, GuardedError::Db(_)));
    }
}
```

**What changes vs `ferro-wallet/src/error.rs`:**

- Type name: `WalletError` → `GuardedError`.
- `Display` prefix: `"config: …"`, `"apple sign: …"` → all variants use `"guarded: …"` (RESEARCH.md verified: matches workspace pattern).
- `Io(#[from] std::io::Error)` variant **dropped** — ferro-orm never touches the filesystem; the I/O equivalent is `Db(#[from] sea_orm::DbErr)`.
- Four variants total (vs WalletError's eight). Keep them in the order: `NoRowsAffected`, `TooManyRows`, `EmptyUpdate`, `Db` — matches the order the error is most likely to surface in (predicate failure first, programming-error guards next, DB-layer last).
- Test count: 4 variants → 4 tests (3 Display assertions + 1 `From<DbErr>` round-trip via `from()`).

**Pitfall callouts:**

- RESEARCH.md Pitfall 1 — `EmptyUpdate` is **not optional**. The variant must exist *and* be checked at the top of `exec_raw` in `guarded.rs`. A future contributor "simplifying away dead code" would re-introduce a class of bug; the rustdoc on the variant explicitly names why it exists.
- `TooManyRows` is preserved (D-13). Do not let a "tidy-up" remove it on the grounds that it's "rarely hit" — it is the safety net for filter bugs.

---

### `ferro-orm/src/guarded.rs` (builder-core) — partial analog / NEW PATTERN

**Analog (filter chaining + `IntoCondition`):** `framework/src/database/query_builder.rs` lines 36-92 — SELECT-side `QueryBuilder<E>` with `.filter(impl IntoCondition)`. ferro-orm's `GuardedUpdate` is the UPDATE-side analog.

**Authoritative source:** RESEARCH.md §"Pattern 1: Building the `UpdateMany` lazily inside `exec_*`" (which has the full reference body).

**Filter-chaining pattern** (`framework/src/database/query_builder.rs` lines 36-92):

```rust
use sea_orm::{
    ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Select,
};

pub struct QueryBuilder<E>
where
    E: EntityTrait,
{
    select: Select<E>,
}

impl<E> QueryBuilder<E>
where
    E: EntityTrait,
    E::Model: Send + Sync,
{
    pub fn new() -> Self {
        Self { select: E::find() }
    }

    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: sea_orm::sea_query::IntoCondition,
    {
        self.select = self.select.filter(filter);
        self
    }
    // …
}
```

**Adapt for `GuardedUpdate`** (per RESEARCH.md §"Pattern 1" — store filters/sets, build the `UpdateMany` lazily inside `exec_*`):

```rust
//! `GuardedUpdate<E>` — chainable builder for atomic conditional `UPDATE` statements.
//!
//! Compiles to exactly one `UPDATE … WHERE …` SQL statement. The database
//! engine's per-statement atomicity (SQLite serial writer, Postgres
//! `READ COMMITTED`) is the entire correctness mechanism — this builder
//! adds the chainable surface and the rows-affected → `GuardedError` mapping
//! on top.

use sea_orm::sea_query::{Condition, IntoCondition, SimpleExpr};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Update, Value};

use crate::GuardedError;

pub struct GuardedUpdate<E: EntityTrait> {
    entity: E,
    filters: Condition,
    sets: Vec<(E::Column, SimpleExpr)>,
}

impl<E: EntityTrait> GuardedUpdate<E> {
    pub fn new(entity: E) -> Self {
        Self {
            entity,
            filters: Condition::all(), // AND-combiner per D-06
            sets: Vec::new(),
        }
    }

    pub fn filter<F: IntoCondition>(mut self, f: F) -> Self {
        self.filters = self.filters.add(f);
        self
    }

    pub fn set_expr(mut self, col: E::Column, expr: SimpleExpr) -> Self {
        self.sets.push((col, expr));
        self
    }

    pub fn set_value(mut self, col: E::Column, value: Value) -> Self {
        // `T: Into<Value> ⇒ T: Into<SimpleExpr>` blanket impl in sea-query.
        self.sets.push((col, SimpleExpr::Value(value)));
        self
    }

    pub async fn exec_one<C: ConnectionTrait>(self, conn: &C) -> Result<(), GuardedError> {
        match self.exec_raw(conn).await? {
            0 => Err(GuardedError::NoRowsAffected),
            1 => Ok(()),
            n => Err(GuardedError::TooManyRows { affected: n }),
        }
    }

    pub async fn exec_at_most_one<C: ConnectionTrait>(
        self,
        conn: &C,
    ) -> Result<bool, GuardedError> {
        match self.exec_raw(conn).await? {
            0 => Ok(false),
            1 => Ok(true),
            n => Err(GuardedError::TooManyRows { affected: n }),
        }
    }

    async fn exec_raw<C: ConnectionTrait>(self, conn: &C) -> Result<u64, GuardedError> {
        // Load-bearing — sea-orm's `Updater::is_noop()` short-circuits with
        // `rows_affected: 0` when SET is empty, which would otherwise look
        // like a predicate miss.
        if self.sets.is_empty() {
            return Err(GuardedError::EmptyUpdate);
        }

        let mut stmt = Update::many(self.entity).filter(self.filters);
        for (col, expr) in self.sets {
            stmt = stmt.col_expr(col, expr);
        }
        let result = stmt.exec(conn).await?; // From<DbErr> via #[from] on Db variant.
        Ok(result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::sea_query::Expr;
    use sea_orm::{
        ConnectionTrait, Database, DatabaseBackend, EntityTrait, Schema, Set, TransactionTrait,
    };

    mod counters {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "counters")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub quantity: i32,
            pub status: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(counters::Entity);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .expect("create table");
        conn
    }

    // T-16-1, T-16-2, T-16-3, T-16-4, T-16-5, T-16-6, T-16-7 — one #[tokio::test] each.
    // See RESEARCH.md §"Validation Architecture" → "Phase Requirements → Test Map".
}
```

**What changes vs `query_builder.rs`:**

- SELECT-side `Select<E>` → UPDATE-side lazy state (`Condition` + `Vec<(E::Column, SimpleExpr)>`); the `Update::many` statement is built only inside `exec_*`, not stored eagerly (RESEARCH.md §"Pattern 1" — sea-orm's `UpdateMany` is awkward to hold eagerly).
- Consuming builder (`self` not `mut self → Self` taking `&mut`) — but `query_builder.rs::filter` already uses `mut self → Self`, so the shape matches. This is the workspace convention per CLAUDE.md "Builder pattern: `with_*` methods taking `mut self` → `Self`".
- Adds **two execution methods** with different semantics (`exec_one` errors on 0 rows; `exec_at_most_one` returns `Ok(false)` on 0 rows). Both are `<C: ConnectionTrait>` generic per D-09.
- No global `DB::connection()` — the caller passes `&conn` explicitly. This is a deliberate departure from `query_builder.rs::all()`/`first()` which read from the global `DB` (D-09 forbids the shortcut here).
- The `exec_raw` private method exists to share the `EmptyUpdate` guard + `Update::many` build between `exec_one` and `exec_at_most_one`. Do **not** inline it twice.

**Test cases inside `#[cfg(test)] mod tests`** (per D-16 / RESEARCH.md Validation Architecture):

| Test | Behaviour |
|------|-----------|
| `predicate_matches_one_row_succeeds` (T-16-1) | Insert row, build matching `GuardedUpdate`, assert `exec_one` returns `Ok(())`. |
| `predicate_fails_zero_rows` (T-16-2) | Build `GuardedUpdate` whose filter does not match, assert `exec_one` returns `Err(NoRowsAffected)` AND `exec_at_most_one` returns `Ok(false)`. |
| `predicate_matches_multiple_rows` (T-16-3) | Insert 2 rows that share a filter column value, assert both methods return `Err(TooManyRows { affected: 2 })`. |
| `empty_update_no_sets` (T-16-4) | Build `GuardedUpdate` without any `set_*` calls, assert `Err(EmptyUpdate)` is returned **before** any SQL fires. |
| `multi_column_set_atomic` (T-16-5) | Two `.set_expr` / `.set_value` calls → assert both columns mutated in the resulting row. |
| `transaction_rollback` (T-16-6) | Run `GuardedUpdate.exec_one(&txn)` inside a `DatabaseTransaction`, drop the transaction without commit, assert the column reverts. Uses `TransactionTrait::begin`. |
| `filter_and_combine` (T-16-7) | Two `.filter` calls — only the row matching BOTH conditions is updated. |

**Pitfall callouts:**

- RESEARCH.md Pitfall 1 — the `if self.sets.is_empty()` guard must run **before** building the `UpdateMany`. The test (T-16-4) is the regression lock.
- RESEARCH.md Pitfall 4 — `TooManyRows` is the safety net for filter-shape bugs. Do not weaken `exec_one` to "succeed on 1 or more rows".
- RESEARCH.md Pitfall 3 — when adding the dev-dep `tokio` features, match `ferro-events/Cargo.toml` line 21 (`["full", "test-util"]`) per RESEARCH.md Open Question 4 recommendation.
- `Vec<(E::Column, SimpleExpr)>` is intentional (RESEARCH.md A2 / "Alternatives Considered") — **do not** introduce an internal `SetTarget` enum. CONTEXT D-07 mentions it but the discretion in CONTEXT explicitly leaves internal shape to the planner; the research confirms `Value: Into<SimpleExpr>` makes the enum redundant.
- Order semantics: insertion order preserved; later sets to the same column override earlier ones. Sea-query's `UpdateStatement::value` already provides this. Do **not** deduplicate in the builder.

---

### `ferro-orm/tests/concurrent_decrement.rs` (integration-test) — partial analog / NEW PATTERN

**Analog (test layout / import shape):** `ferro-stripe/tests/parser_contract.rs` lines 1-18 — integration-test file in `crate/tests/`, file-level `//!` doc, `use ferro_…::…` imports, helper function.

**Authoritative source for the race-test shape:** RESEARCH.md §"Code Examples" → Example 5 (the only place in the workspace where a concurrent SQL race test is fully specified).

**Layout pattern** (`ferro-stripe/tests/parser_contract.rs` lines 1-18):

```rust
//! Parser-contract integration tests — asserts every `StripeEvent::from_raw`
//! implementation extracts fields correctly from its golden-JSON fixture, …
//!
//! Fixtures live in `tests/fixtures/stripe_events/`.

use std::collections::HashMap;

use ferro_stripe::{
    StripeChargeDisputeCreated, StripeChargeRefunded, StripeCheckoutCompleted,
    // …
};

fn parse_event(raw: &str) -> stripe::Event {
    serde_json::from_str::<stripe::Event>(raw).expect("…")
}
```

**Adapt for ferro-orm** (per D-17 + RESEARCH.md Example 5):

```rust
//! Concurrent-decrement integration test — 10 tokio tasks attempt
//! `GuardedUpdate` on a counter starting at K=3. The SQLite serial writer
//! enforces atomicity at the SQL layer; the test asserts that **exactly 3
//! tasks see `Ok(())`** and the remaining 7 see `Err(NoRowsAffected)`.
//!
//! Uses `sqlite:file::memory:?cache=shared` + `max_connections = 4` so
//! multiple connections see the same in-memory DB (see RESEARCH.md Pitfall 2).

use ferro_orm::GuardedUpdate;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseBackend, EntityTrait, Schema,
    Set,
};
use std::sync::Arc;

mod counters {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "counters")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub quantity: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ten_tasks_against_capacity_three_exactly_three_succeed() {
    let mut opts = ConnectOptions::new("sqlite:file::memory:?cache=shared");
    opts.max_connections(4).min_connections(1);
    let conn = Arc::new(Database::connect(opts).await.expect("connect"));

    let schema = Schema::new(DatabaseBackend::Sqlite);
    let stmt = schema.create_table_from_entity(counters::Entity);
    conn.execute(conn.get_database_backend().build(&stmt))
        .await
        .unwrap();

    counters::Entity::insert(counters::ActiveModel {
        id: Set(1),
        quantity: Set(3),
    })
    .exec(&*conn)
    .await
    .unwrap();

    let mut tasks = Vec::with_capacity(10);
    for _ in 0..10 {
        let conn = Arc::clone(&conn);
        tasks.push(tokio::spawn(async move {
            GuardedUpdate::new(counters::Entity)
                .filter(counters::Column::Id.eq(1))
                .filter(counters::Column::Quantity.gte(1))
                .set_expr(
                    counters::Column::Quantity,
                    Expr::col(counters::Column::Quantity).sub(1),
                )
                .exec_one(&*conn)
                .await
        }));
    }

    let mut successes = 0usize;
    let mut no_rows = 0usize;
    for handle in tasks {
        match handle.await.unwrap() {
            Ok(()) => successes += 1,
            Err(ferro_orm::GuardedError::NoRowsAffected) => no_rows += 1,
            Err(other) => panic!("unexpected error: {other}"),
        }
    }

    assert_eq!(successes, 3, "exactly 3 of 10 tasks should succeed");
    assert_eq!(no_rows, 7, "the other 7 should fail with NoRowsAffected");

    let final_row = counters::Entity::find_by_id(1)
        .one(&*conn)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(final_row.quantity, 0);
}
```

**What changes vs `parser_contract.rs`:**

- Static fixture parsing → live in-process DB.
- `#[test]` synchronous → `#[tokio::test(flavor = "multi_thread", worker_threads = 4)]` (the multi-thread flavor is **load-bearing** — single-thread tokio would funnel all 10 tasks through one OS thread and reduce contention).
- `futures::future::join_all` is **avoided** to keep the dev-dep set to just tokio + sea-orm. Hand-roll the await loop instead (RESEARCH.md §"Code Examples" Example 5 final note).
- Avoid `Result::unwrap` on the spawn JoinHandle if Clippy complains — explicit `match` is already clearer.

**Pitfall callouts:**

- **RESEARCH.md Pitfall 2 is the load-bearing pitfall for this file.** Using `sqlite::memory:` (no `file::` prefix, no `?cache=shared`) with `max_connections = 1` would cause the pool to serialize tasks and the test would pass for the wrong reason. The connect string and the `max_connections(4)` are not optional.
- Multi-thread tokio runtime (`flavor = "multi_thread", worker_threads = 4`) is needed to actually generate contention.
- `seed quantity = 3` and `decrement by 1, predicate `quantity >= 1`` are the canonical D-17 numbers. Do not "round up" to easier numbers — the test name asserts the specific 3/7 split.

---

### `ferro-orm/README.md` (readme)

**Analog:** `ferro-wallet/README.md` lines 1-12 — one-paragraph crate purpose, `Status:` line linking the framework repo, `Documentation:` line linking docs.rs, `License: MIT`. ~10 lines.

**Pattern** (`ferro-wallet/README.md` lines 1-11):

```markdown
# ferro-wallet

Digital wallet pass issuance for the Ferro framework — Apple `.pkpass` files and Google Wallet save-links.

The crate exposes a `WalletSubject` trait …. Reads `APP_NAME` / `APP_URL` from environment, matching the project-agnostic convention shared by every `ferro-*` crate.

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-wallet

License: MIT
```

**Adapt for ferro-orm:**

```markdown
# ferro-orm

Atomic conditional updates and ORM primitives for the Ferro framework.

The crate exposes `GuardedUpdate<E>` — a chainable builder that compiles to a single `UPDATE … WHERE …` SQL statement, replacing the hand-rolled `read → check → write` pattern wherever a column's value is conditionally mutated. The database engine's per-statement atomicity (SQLite serial writer, Postgres `READ COMMITTED`) is the entire correctness mechanism; `GuardedUpdate` adds the chainable surface and the rows-affected → `GuardedError` mapping on top.

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-orm

License: MIT
```

**What changes vs `ferro-wallet/README.md`:**

- Domain wording.
- **Drop the `APP_NAME` / `APP_URL` env mention** — ferro-orm has **no** env-driven config (D-09 / RESEARCH.md §"Project Constraints"). The line in ferro-wallet's README about env vars does not apply.

**Pitfall callouts:**

- CLAUDE.md "Architecture Principle #6 — Project-agnostic crates" applies: the README must contain zero tenant identifiers ("gestiscilo", "Ferro Application", "https://example.com"). The example values in any later expansion should be generic (`inventory_units`, `counters`, `widgets`).
- Keep length to ~10 lines. The expanded explanation belongs in `docs/src/database/atomic-updates.md`, not in README.md.

---

### `docs/src/database/atomic-updates.md` (doc-page)

**Analog (code-block tone + section structure):** `docs/src/features/database.md` lines 1-80 — sibling DB doc, uses `## Configuration` / `## Basic Usage` / `## Getting a Connection` H2 structure, fenced `rust` blocks (not `rust,ignore`).

**Analog (concept→example→requirements flow):** `docs/src/features/events.md` lines 1-60 — opens with a one-line purpose, jumps straight into a generated-code example, then enumerates trait requirements. Apply the same arc.

**Authoritative source for content:** D-21 specifies the required sections explicitly: "why race-free updates matter, the `read → check → write` anti-pattern this replaces, the GuardedUpdate API, common patterns (counter decrement, status transition, optimistic concurrency), the `exec_one` vs `exec_at_most_one` decision tree."

**Document structure (suggested H2 hierarchy):**

```markdown
# Atomic Updates

[One-paragraph framing — why race-free updates are a first-class concern; the `read → check → write` bug that prompts this primitive.]

## The Anti-Pattern: `read → check → write`

[Show the buggy pattern in pseudo-Rust — a SELECT followed by a check, followed by an UPDATE. Two concurrent callers both pass the check, both write, both "succeed" — capacity is exceeded.]

## The Replacement: `GuardedUpdate`

[Show the same operation as a single `GuardedUpdate` call. One round trip, one statement, atomic at the DB.]

## API

### `GuardedUpdate::new(entity)`
### `.filter(condition)` — AND-combined
### `.set_expr(col, expr)` and `.set_value(col, value)`
### `.exec_one(&conn)` vs `.exec_at_most_one(&conn)`

## Common Patterns

### Counter decrement (canonical inventory example)
### Status transition (held → committed)
### Optimistic update (session refresh)

## `exec_one` vs `exec_at_most_one` decision tree

[Two-bullet rule: if predicate failure is a programming/contention error, use `exec_one` (it surfaces `NoRowsAffected` as an error). If predicate failure is a normal outcome, use `exec_at_most_one`.]

## Atomicity guarantee (and its limit)

[Per-statement, not per-builder. Reading the post-update row in a separate query reintroduces a race. Bracket in a transaction if needed.]

## Errors

[Brief table of `GuardedError` variants.]
```

**Tone / formatting rules (from `docs/src/features/database.md`):**

- Use plain `rust` fenced blocks (not `rust,ignore`) when the snippet is self-contained and would compile against the published crate.
- Use `bash` for shell commands.
- Headings are `##` for top-level sections, `###` for subsections.
- No emojis (CLAUDE.md "Comments and documentation: Scientific and minimalistic, no marketing language").

**Pitfall callouts:**

- CLAUDE.md "Repository documents must read as neutral" — write in product-communication voice. **Forbidden trigger phrases:** "killer feature", "the bet", "load-bearing weakness", "no stop-loss", "forcing function", any "we accept that" / "the risk we're taking" framing. The internal framing this doc carries — "this is the foundational kernel for the v11.11 reservation milestone" — does not belong here. The doc reads as if the GuardedUpdate primitive simply *exists*.
- Project-agnostic examples — generic entity names (`inventory_units`, `counters`, `sessions`, `widgets`). **No** tenant identifiers.
- The "Atomicity guarantee (and its limit)" section is **not optional** — D-15 mandates it and RESEARCH.md flags it as the load-bearing footgun.

---

## Workspace Edits — Exact Insertion Points

### Edit 1: `Cargo.toml` workspace `[workspace] members` append

**File:** `/Users/alberto/repositories/albertogferrario/ferro/Cargo.toml`

**Current state lines 1-25** (members array; non-alphabetical, phase-introduction order):

```toml
[workspace]
resolver = "2"
members = [
    "framework",
    "app",
    "ferro-cli",
    "ferro-macros",
    "ferro-events",
    "ferro-queue",
    "ferro-notifications",
    "ferro-broadcast",
    "ferro-storage",
    "ferro-cache",
    "ferro-mcp",
    "ferro-inertia",
    "ferro-json-ui",
    "ferro-lang",
    "ferro-api-mcp",
    "ferro-projections",
    "ferro-stripe",
    "ferro-theme",
    "ferro-ai",
    "ferro-whatsapp",
    "ferro-wallet",     ← current last member, line 24
]
```

**Required edit:** append `"ferro-orm",` after line 24. Order is not alphabetical; preserve phase-introduction-order.

**Resulting array (lines 24-25 → 24-26):**

```toml
    "ferro-wallet",
    "ferro-orm",
]
```

**Precedent commit/diff to mimic:** Phase 151 appended `"ferro-wallet",` after `"ferro-whatsapp",`. This edit is the same mechanical pattern.

### Edit 2: `.github/workflows/publish.yml` Wave 1a append

**File:** `/Users/alberto/repositories/albertogferrario/ferro/.github/workflows/publish.yml`

**Current state line 201:**

```yaml
          WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet"
```

**Rationale:** `ferro-orm` has zero internal `ferro-*` dependencies (CONTEXT D-04 / RESEARCH.md §"Standard Stack"). It belongs in Wave 1a, not 1b. Wave 1b (line 236) lists crates with internal deps; `ferro-orm` has none.

**Resulting line 201:**

```yaml
          WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm"
```

**Precedent commit/diff to mimic:** Phase 151 appended `ferro-wallet` at the end of the same string. Same mechanical pattern.

**Pitfall callout:** RESEARCH.md Pitfall 5 — CI's `CARGO_REGISTRY_TOKEN` is `publish-update`-scoped. The first `ferro-orm` publish from CI will fail with "not found" / "no upload permission". This is **expected exactly once**. The plan-checker should NOT treat the CI failure as a blocker; the operator runs `cargo publish -p ferro-orm` locally with a personal `publish-new`-scoped token, then subsequent versions auto-publish. Mirror Phase 151 PLAN-09 for the bootstrap step (see `.planning/phases/151-ferro-wallet-crate/151-09-SUMMARY.md`).

### Edit 3: `CHANGELOG.md` new `## ferro-orm` section

**File:** `/Users/alberto/repositories/albertogferrario/ferro/CHANGELOG.md`

**Current state lines 1-36:**

```markdown
# Changelog

All notable changes to Ferro crates are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## ferro-wallet

### [0.2.24] — 2026-05-11

Initial release. Phase 151 — `ferro-wallet` crate (Apple `.pkpass` +
Google Wallet save-link issuance). Milestone v11.10.

#### Added

- New crate `ferro-wallet` exposing the `WalletSubject` trait, …
```

**Required edit:** insert a new `## ferro-orm` section **above** `## ferro-wallet` (newest-on-top within the per-crate sections). Date is the merge date. Version is whatever `[workspace.package] version` shows when the phase merges (see RESEARCH.md Open Question 1 — workspace is currently at `0.2.30` despite CONTEXT D-23 saying `0.2.25`; the CI `check-version` job auto-bumps if needed, so the planner should treat the version as "whatever CI publishes" and not hand-bump).

**Resulting section shape** (mirror the Phase 151 ferro-wallet entry):

```markdown
## ferro-orm

### [0.2.x] — YYYY-MM-DD

Initial release. Phase 152 — `ferro-orm` crate (atomic conditional UPDATE
primitive for race-free counter mutations and state transitions).
Milestone v11.11.

#### Added

- New crate `ferro-orm` exposing the `GuardedUpdate<E>` builder — compiles
  to a single `UPDATE … WHERE …` SQL statement, replacing the hand-rolled
  `read → check → write` pattern wherever a column's value is conditionally
  mutated. The database engine's per-statement atomicity (SQLite serial
  writer, Postgres `READ COMMITTED`) is the correctness mechanism;
  `GuardedUpdate` adds the chainable surface and the rows-affected →
  `GuardedError` mapping on top.
- `GuardedUpdate::filter(impl IntoCondition)` — AND-combines multiple
  filter calls onto an internal `Condition`. Matches `sea_orm::QueryFilter`
  ergonomics.
- `GuardedUpdate::set_expr(col, SimpleExpr)` and `set_value(col, Value)` —
  chainable per-column set, supports value-derived (`Expr::col(…).sub(…)`)
  and literal (`Value::String(…)`) assignments in the same statement.
- `GuardedUpdate::exec_one(&conn)` — succeeds iff exactly one row matched;
  `0 → Err(NoRowsAffected)`, `>1 → Err(TooManyRows { affected })`. Default
  for race-free counter mutations.
- `GuardedUpdate::exec_at_most_one(&conn)` — `Ok(true)` on 1 row,
  `Ok(false)` on 0 rows (predicate failure is a normal outcome),
  `Err(TooManyRows)` on >1 rows. For optimistic updates.
- `GuardedError` — `NoRowsAffected | TooManyRows { affected } |
  EmptyUpdate | Db(#[from] DbErr)`. Display prefix `"guarded: …"`.
- Targeted re-exports of the SeaORM symbols required by the public API
  (`EntityTrait`, `ColumnTrait`, `ConnectionTrait`, `IntoCondition`,
  `SimpleExpr`, `Value`, `DbErr`, `Expr`); no blanket `pub use sea_orm::*`.
- Workspace member registered in `Cargo.toml`; auto-publish Wave 1a slot
  reserved in `.github/workflows/publish.yml`. First publish is
  bootstrapped from a local terminal (CI publish token has
  `publish-update` scope only); subsequent versions auto-publish.
- New documentation page `docs/src/database/atomic-updates.md` covering
  the anti-pattern, the API, common patterns (counter decrement, status
  transition, optimistic concurrency), and the per-statement atomicity
  contract.

## ferro-wallet
…
```

**What changes vs the `## ferro-wallet` precedent:**

- Section header: `## ferro-wallet` → `## ferro-orm`.
- Milestone: v11.10 → v11.11.
- Bullet content is GuardedUpdate-specific. **Keep the same shape** (lead with what the crate provides, then list the methods, then the release plumbing, then the docs).
- **Drop** any "Reads `APP_NAME` / `APP_URL` from environment" line — ferro-orm has no env config.

**Precedent commit/diff to mimic:** the Phase 151 commit that introduced lines 6-35.

### Edit 4: `CLAUDE.md` Workspace Structure table row

**File:** `/Users/alberto/repositories/albertogferrario/ferro/CLAUDE.md`

**Current state lines 35-58** (Workspace Structure table). The last `ferro-*` row is `ferro-whatsapp` at line 57; the `app` row at line 58 marks the end of the framework crate listing.

**Required edit:** insert a new row for `ferro-orm` **after** the `ferro-whatsapp` row and **before** the `app` row. Preserve the existing column shape (`Crate | Purpose | Key Files`).

**Resulting insert** (between lines 57 and 58):

```markdown
| `ferro-whatsapp` | WhatsApp Business Cloud API integration | `src/lib.rs` |
| `ferro-orm` | Atomic conditional updates and ORM primitives (`GuardedUpdate`) | `src/lib.rs` |
| `app` | Sample application | Reference implementation |
```

**What changes vs the other rows:**

- Purpose phrasing matches the rustdoc / Cargo.toml `description` field, kept short.
- `Key Files` points at `src/lib.rs` (single re-export surface), matching every other leaf-crate row.

**Pitfall callout:**

- The table is **not alphabetical** — preserve phase-introduction order. ferro-wallet was added in Phase 151 but is **not** in this table yet (an oversight the planner should flag). The new row goes at the end of the `ferro-*` block; if the planner also wants to add the missing `ferro-wallet` row in the same edit, that is fine but out of strict Phase 152 scope.

### Edit 5: `docs/src/SUMMARY.md` mdBook nav entry

**File:** `/Users/alberto/repositories/albertogferrario/ferro/docs/src/SUMMARY.md`

**Current state lines 19-46** (`# Features` section, listed flat — no sub-page nesting yet):

```markdown
# Features

- [Events & Listeners](features/events.md)
- …
- [Database](features/database.md)
- [Derive Macros](features/derive-macros.md)
- …
```

**Sub-page nesting decision required** (RESEARCH.md Wave 0 Gaps flagged this):

The new doc page lives at `docs/src/database/atomic-updates.md` (note: a sibling `database/` directory at the docs root, not `features/database/`). Two equally-reasonable navigation patterns exist:

1. **Flat entry** (matches every other doc page in `# Features`):

    ```markdown
    - [Database](features/database.md)
    - [Atomic Updates](database/atomic-updates.md)
    - [Derive Macros](features/derive-macros.md)
    ```

2. **Nested under Database** (preferred if mdBook nesting is desired; requires `[Database]` to become a parent):

    ```markdown
    - [Database](features/database.md)
      - [Atomic Updates](database/atomic-updates.md)
    - [Derive Macros](features/derive-macros.md)
    ```

**Recommendation:** **Pattern 1 (flat)**. It matches every existing entry in `# Features`; no sibling page in this SUMMARY currently uses nested children. The path `database/atomic-updates.md` (without the `features/` prefix) is what the wave-0 file plan specifies — this aligns with the link target the planner already committed to.

**Pitfall callout:**

- The doc page lives at `docs/src/database/atomic-updates.md`, **not** `docs/src/features/database/atomic-updates.md`. The path on disk and the link in SUMMARY must agree. If the planner prefers the nested-under-features path, both the file location and the SUMMARY entry need to be adjusted together.
- mdBook silently ignores unreferenced files. If SUMMARY.md is not updated, the doc page exists on disk but won't render in the rendered book. The test (`mdbook build`) is the lock — wave-0 commands should include `mdbook build docs/` or whatever pre-commit hook the repo already runs.

---

## Shared Patterns

### Error variant prefix convention

**Source:** `ferro-wallet/src/error.rs` lines 10, 14, 18, 22, … — every variant's `#[error("…")]` string begins with a lowercase, name-prefixed token.

**Apply to:** every `GuardedError` variant per D-11. Prefix is `"guarded: …"`.

```rust
#[error("guarded: predicate matched no rows")]
NoRowsAffected,
#[error("guarded: db error: {0}")]
Db(#[from] sea_orm::DbErr),
```

### `thiserror = "2"` derive

**Source:** `ferro-wallet/Cargo.toml` line 23, `ferro-events/Cargo.toml` line 18 — both on `"2"`.

**Apply to:** `ferro-orm/Cargo.toml` — match `thiserror = "2"`. RESEARCH.md verified `2.0.17` resolves in `Cargo.lock`.

### `version.workspace = true`

**Source:** `ferro-wallet/Cargo.toml` line 3, `ferro-events/Cargo.toml` line 3 — the current convention.

**Apply to:** `ferro-orm/Cargo.toml` line 3 — tracks workspace `0.2.x` bumps.

### Consuming builder (`mut self → Self`)

**Source:** `framework/src/database/query_builder.rs` lines 86-92 — `filter(mut self, …) -> Self` shape.

**Apply to:** every method on `GuardedUpdate` (`filter`, `set_expr`, `set_value`, and the two `exec_*` terminators). CLAUDE.md "Builder pattern: `with_*` methods taking `mut self` → `Self` (consuming)" is the workspace rule.

### `ConnectionTrait` generic (no global `DB::get()`)

**Source:** `framework/src/database/query_builder.rs` accepts `impl ConnectionTrait` via the SeaORM trait surface; sea-orm's `Update::many(…).exec(conn)` requires `&impl ConnectionTrait`.

**Apply to:** `exec_one<C: ConnectionTrait>(self, conn: &C)` and `exec_at_most_one<C: ConnectionTrait>(self, conn: &C)` per D-09. **Do not** add a `DB::connection()` shortcut — the caller passes the connection explicitly so transaction bracketing is visible at the call site.

### Single in-memory SQLite test scaffold

**Source:** RESEARCH.md §"Code Examples" Example 4 — `Database::connect("sqlite::memory:")` + `Schema::create_table_from_entity(entity)` + `conn.execute(conn.get_database_backend().build(&stmt))`.

**Apply to:** every test inside `#[cfg(test)] mod tests` in `guarded.rs`. Use `max_connections = 1` (the default) for the **unit** tests — only `tests/concurrent_decrement.rs` needs the `?cache=shared` + `max_connections = 4` variant to expose the race (RESEARCH.md Pitfall 2).

### Per-variant Display assertion test

**Source:** `ferro-wallet/src/error.rs` lines 42-99 — exhaustive `#[cfg(test)] mod tests` with one `#[test] fn …_displays_message()` per variant + a `#[test]` for the `From` conversion.

**Apply to:** `ferro-orm/src/error.rs` — three Display assertions (`NoRowsAffected`, `TooManyRows`, `EmptyUpdate`) + one `From<DbErr>` round-trip assertion.

---

## No Analog Found — Authoritative Source for Each

| File | Reason no analog | Authoritative source |
|------|------------------|----------------------|
| `ferro-orm/src/guarded.rs` (builder body) | No UPDATE-side SeaORM extension exists; `framework/src/database/query_builder.rs` is SELECT-side only | RESEARCH.md §"Pattern 1: Building the `UpdateMany` lazily inside `exec_*`" (full reference body) + sea-orm-1.1.19 source citations |
| `ferro-orm/tests/concurrent_decrement.rs` (race-test body) | No prior multi-task tokio race test in the workspace | RESEARCH.md §"Code Examples" → Example 5 (full reference body) + RESEARCH.md Pitfall 2 (shared-cache requirement) |
| `docs/src/database/atomic-updates.md` (full prose) | No prior tutorial-style doc page covers an atomic-write primitive | D-21 (required section list) + the rustdoc on `lib.rs` (canonical example) — the doc page expands on both |

---

## Cross-Cutting Constraints (apply to every file)

### Project-agnostic crate rule (CLAUDE.md Architecture Principle #6)

`ferro-orm` is a leaf library; it has **no env-driven config** (D-09 forbids the global `DB::get()` shortcut, which removes the only realistic env-var surface). The planner must verify:

- No `WalletConfig`-style `from_env()` is introduced.
- No hardcoded tenant strings (`"gestiscilo"`, `"Ferro Application"`, `"https://example.com"`) anywhere — including in docs examples.
- Documentation examples use generic entity names: `inventory_units`, `counters`, `sessions`, `reservations`, `widgets`.

### CI gate (CLAUDE.md "Testing & Linting")

Before commit:

```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets -- -D warnings
cargo test --all-features
```

`--all-targets` catches test-code issues that `--all` alone misses. CI enforces `-D warnings` — any warning is a build failure.

### No co-author lines in commit messages

CLAUDE.md "Git Commit Rules": no co-author attribution, no "Generated with Claude" lines.

---

## Metadata

**Analog search scope:** `ferro-wallet/`, `ferro-events/`, `ferro-stripe/`, `framework/src/database/`, root `Cargo.toml`, `.github/workflows/publish.yml`, `CHANGELOG.md`, `CLAUDE.md`, `docs/src/`
**Files scanned (Read):** 13
**Prior PATTERNS.md mirrored:** `.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md` (style + structure)
**Pattern extraction date:** 2026-05-13
**Linked artifacts:**
- `.planning/phases/152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-/152-CONTEXT.md` (decisions D-01..D-25)
- `.planning/phases/152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-/152-RESEARCH.md` (§"Pattern 1" full reference body + §"Code Examples" 5 worked examples + §"Common Pitfalls" 5 verified pitfalls + §"Validation Architecture" test map)
- `.planning/research/INVENTORY-PRIMITIVES.md` §`ferro-orm::guarded` (design spec)

## PATTERN MAPPING COMPLETE
