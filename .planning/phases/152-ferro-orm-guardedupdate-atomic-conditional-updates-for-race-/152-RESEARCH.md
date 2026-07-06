# Phase 152: ferro-orm GuardedUpdate — Research

**Researched:** 2026-05-13
**Domain:** SeaORM-backed atomic conditional UPDATE primitive (new workspace leaf crate)
**Confidence:** HIGH

## Summary

Phase 152 creates a new top-level workspace crate `ferro-orm` shipping one primitive: `GuardedUpdate<E: EntityTrait>`, a chainable builder that compiles to exactly one SeaORM `UpdateMany` statement (one round-trip, one `UPDATE … WHERE …` SQL). The DB is the authority on contention; the crate's job is to make the call site race-free by construction by removing the read-then-write round trip.

The SeaORM 1.1.19 API surface this builder wraps is verified, narrow, and stable: `Update::many(entity) → UpdateMany<E>` (set with `.col_expr(col, SimpleExpr)`, filter with `.filter(impl IntoCondition)` via `QueryFilter`, execute with `.exec(&conn) -> Result<UpdateResult, DbErr>` where `UpdateResult.rows_affected: u64`). The two execution variants (`exec_one` / `exec_at_most_one`) are pure post-processing wrappers over `rows_affected`. There is no third-party library to evaluate — sea-orm IS the standard stack and it is already in the workspace.

The non-obvious load-bearing finding is that SeaORM's `Updater::exec` short-circuits when there are no SET clauses (`is_noop()` returns `rows_affected: 0` without executing any SQL). Without the explicit `EmptyUpdate` check at `exec_*` time, a builder with no `set_*` calls would silently look like a predicate failure — masking the programmer bug as `NoRowsAffected`. D-12 is therefore not a stylistic preference; it is necessary correctness.

**Primary recommendation:** Implement as a thin SeaORM wrapper. Store sets internally as `Vec<(E::Column, SimpleExpr)>` (both `set_expr` and `set_value` collapse into this via `T: Into<Value> ⇒ T: Into<SimpleExpr>`). Build the `UpdateMany` statement only inside `exec_*`, never eagerly. Tests use direct `sea-orm::Database::connect("sqlite::memory:")` plus a raw `CREATE TABLE` via `ConnectionTrait::execute` — `sea-orm-migration` is intentionally NOT added as a dev-dep. The concurrent-decrement test (D-17) must use `sqlite:file::memory:?cache=shared` or a temp-file DB with `max_connections > 1` to expose the SQL-level race; the default `sqlite::memory:` with `max_connections=1` serializes tasks at the pool layer and proves nothing.

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Ship as a new top-level workspace crate at `ferro-orm/` — NOT inside `framework/src/database/`. The roadmap explicitly names `ferro-orm::GuardedUpdate`; phases 154 and external consumer apps will import it as `use ferro_orm::GuardedUpdate;`. Putting it inside `framework` would force every consumer to depend on the full framework crate.
- **D-02:** Crate is thin and additive at v0. It does NOT take over `framework/src/database/` ownership. Migration of `query_builder.rs`, `model.rs`, `connection.rs`, etc. into `ferro-orm` is explicitly deferred — `ferro-orm v0.x` is the GuardedUpdate kernel only. Naming `ferro-orm` claims the future namespace without paying the extraction cost in this phase.
- **D-03:** Re-export only the SeaORM symbols the public API references: `EntityTrait`, `ColumnTrait`, `ConnectionTrait`, `IntoCondition`, `SimpleExpr`, `Value`, `DbErr`. Do NOT `pub use sea_orm::*`. Consumers that need the full SeaORM API can depend on `sea-orm` directly — keeps the `ferro-orm` surface inspectable in MCP and stable across SeaORM upgrades.
- **D-04:** Wave 1a publish (zero internal ferro-* deps). External deps: `sea-orm` (1.0, workspace version), `thiserror` (2). Add to `.github/workflows/publish.yml` Wave 1a alongside `ferro-wallet`. New-crate-first-publish bootstrap from local terminal (CI token has publish-update only).
- **D-05:** Constructor `GuardedUpdate::new(entity: E)`. `E` is the SeaORM entity.
- **D-06:** Filter API `filter(self, f: impl IntoCondition) -> Self`. Multiple `.filter(...)` calls AND-combine.
- **D-07:** Set API — both `set_expr(self, col: E::Column, expr: SimpleExpr) -> Self` and `set_value(self, col: E::Column, value: Value) -> Self`, chainable, multiple sets allowed. Order preserved; later sets to the same column override earlier ones.
- **D-08:** Execution methods — `exec_one<C: ConnectionTrait>(self, conn: &C) -> Result<(), GuardedError>` (errors on 0 rows) AND `exec_at_most_one<C: ConnectionTrait>(self, conn: &C) -> Result<bool, GuardedError>` (Ok(false) on 0 rows).
- **D-09:** Generic over `<C: ConnectionTrait>`. No global `DB::connection()` shortcut. Caller passes connection explicitly.
- **D-10:** No `UPDATE … RETURNING` in v0. Cross-dialect portability concern.
- **D-11/D-12/D-13:** `GuardedError` = `NoRowsAffected | TooManyRows{affected: u64} | EmptyUpdate | Db(#[from] DbErr)`. Display prefix "guarded: ".
- **D-14/D-15:** Atomicity is per-statement, not per-builder. Transaction bracketing is caller's responsibility, documented in rustdoc.
- **D-16:** Unit tests in `ferro-orm/src/guarded.rs` (or `#[cfg(test)] mod tests`) using in-memory SQLite. Cover 7 cases listed in CONTEXT.
- **D-17:** ONE integration test `tests/concurrent_decrement.rs` — 10 tokio tasks, counter starts at K=3, exactly 3 succeed with Ok(()), 7 fail with NoRowsAffected.
- **D-18:** Property tests NOT in scope (Phase 154 budget).
- **D-19:** Postgres CI tests deferred.
- **D-20:** Module-level rustdoc with canonical example + per-statement misuse footgun.
- **D-21:** New `docs/src/database/atomic-updates.md` page.
- **D-22:** ferro-mcp introspection — audit `code_templates` / `generation_context` for ORM/UPDATE references; update if found.
- **D-23:** Workspace `[workspace.package] version` bumps one patch (CONTEXT says `0.2.24 → 0.2.25` — see Open Question 1 below; reality is workspace is at `0.2.30` and last v* tag is `v0.2.24`).
- **D-24:** Add `ferro-orm` to Wave 1a of `.github/workflows/publish.yml`.
- **D-25:** CHANGELOG entry under new `ferro-orm` section.

### Claude's Discretion

- Internal module layout of `ferro-orm/src/` (single `lib.rs` vs `lib.rs + guarded.rs`)
- Internal storage type for column→update mapping (the public surface is the chainable `set_expr` / `set_value` methods; recommended internal is `Vec<(E::Column, SimpleExpr)>` — see Code Examples)
- Exact rustdoc prose & code-block formatting
- Test file names within `ferro-orm/tests/`
- Whether to expose `into_query()` for diagnostics (recommend NO — keeps surface tight)

### Deferred Ideas (OUT OF SCOPE)

- `GuardedDelete` / `GuardedInsert`
- Extraction of `framework/src/database/{query_builder,model,connection,...}` into `ferro-orm`
- `UPDATE … RETURNING` for returning the updated row
- Audit-log emission on success (Phase 153)
- Event emission on success
- Postgres CI integration tests
- Property-based tests (Phase 154 budget)
- `ferro::prelude` / framework re-export of `GuardedUpdate`

## Project Constraints (from CLAUDE.md)

The planner must verify these are honored in PLAN.md:

- **Architecture Principle #6 — Project-agnostic crates:** `ferro-orm` must not reference any application identity. This crate has no env-config needs (the connection comes from the caller via `ConnectionTrait`), so `APP_NAME` / `APP_URL` are not relevant here. Confirm: no `WalletConfig`-style `from_env()`, no hardcoded strings like `"gestiscilo"`, `"Ferro Application"`, or `"https://example.com"` anywhere in the crate (including docs examples). Documentation examples must use generic entity names (e.g., `inventory_units`, `counters`, `widgets`).
- **Pre-commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` must pass. `--all-targets` catches test-code issues that `--all` alone misses; CI enforces `-D warnings`.
- **One Error enum per crate, thiserror-derived, name-prefixed Display:** `GuardedError` follows the pattern. Display prefix is `"guarded: …"` (matches `"config: …"`, `"apple sign: …"`, `"wallet: …"`).
- **Builder pattern:** `with_*` / chainable methods take `mut self → Self` (consuming-builder shape). `GuardedUpdate::filter`, `set_expr`, `set_value` all follow this.
- **No co-author lines in commits.**
- **`#[serde(rename_all = "snake_case")]` on enums:** N/A — `GuardedError` is not serialized.
- **Update docs in `docs/src/`** when framework features change. D-21 requires `docs/src/database/atomic-updates.md`.
- **Update ferro-mcp** if introspection surface changes. D-22 requires an audit; current finding (this research) is that **no MCP code changes are required** — `application_info::get_installed_crates` is fully dynamic and will pick up `ferro-orm` automatically.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Build conditional `UPDATE` statement | `ferro-orm` (new leaf crate) | — | New primitive. Public API. |
| SQL compilation and `UPDATE … WHERE …` execution | sea-orm / sea-query | — | Existing standard. Don't hand-roll SQL. |
| Atomicity guarantee | Database engine (SQLite serial-writer, Postgres `READ COMMITTED`) | — | Per-statement atomicity is a DB property, not an app property. |
| Connection pooling and transaction management | Caller (passes `&impl ConnectionTrait`) | sea-orm `DatabaseConnection` / `DatabaseTransaction` | D-09: explicit, not a global. |
| Error mapping (rows_affected → variant) | `ferro-orm::GuardedError` | — | Owned by this crate. |
| Audit logging on success | Phase 153 (`ferro-audit`) at consumer call site | — | Deferred; out of scope here. |
| Event emission on success | Phase 154 (`ferro-reservation`) at its level | — | Out of scope here. |
| Test harness (in-memory SQLite + raw schema) | `ferro-orm/tests/` and `#[cfg(test)]` modules | sea-orm `Database::connect` + `ConnectionTrait::execute` | Avoid sea-orm-migration dev-dep. |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sea-orm` | 1.0 (resolves to 1.1.19) | Provides `Update::many`, `UpdateMany`, `IntoCondition`, `SimpleExpr`, `Value`, `DbErr`, `ConnectionTrait` | [VERIFIED: framework/Cargo.toml line 51] already the workspace ORM; matches existing patterns in `framework/src/database/query_builder.rs` and `model.rs` |
| `thiserror` | 2 | Error derive for `GuardedError` | [VERIFIED: ferro-wallet/Cargo.toml line 23, ferro-events/Cargo.toml line 18] workspace convention since Phase 149+ for leaf crates |

### Supporting (dev-dependencies only)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tokio` | 1 | Async runtime for `#[tokio::test]` and the concurrent-decrement test (D-17) | All async tests |
| `sea-orm` | 1.0 with `features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"]` | In-memory SQLite testing; `DeriveEntityModel` macro for the throwaway test entity | Unit + integration tests |
| `tempfile` | 3 | Optional — temp-file SQLite if shared-cache memory variant proves flaky | Concurrent-decrement test fallback |

**Note on `runtime-tokio-rustls` vs `runtime-tokio-native-tls`:** framework uses `native-tls`. For ferro-orm's dev-dependency only, either works (no TLS connection is opened against in-memory SQLite). Use `runtime-tokio-rustls` to avoid pulling OpenSSL into the dev-dep tree on systems where it's not already loaded; switch to `native-tls` if it conflicts with the resolver. Document the choice in `ferro-orm/Cargo.toml` `[dev-dependencies]`. [VERIFIED: sea-orm/Cargo.toml `[features]` section, sea-orm-1.1.19]

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `sea_orm::Update::many` + `UpdateMany::col_expr` + `.exec` | Raw SQL via `Statement::from_string` + `ConnectionTrait::execute` | Loses entity-type safety (`E::Column` validation), loses cross-dialect quoting/escaping, loses parameterization safety. Rejected. |
| `Vec<(E::Column, SimpleExpr)>` internal storage | Internal `SetTarget` enum wrapping `SimpleExpr`/`Value` | Pointless distinction: `Value: Into<SimpleExpr>` is implemented in sea-query via the `T: Into<Value> ⇒ SimpleExpr::Value(v.into())` blanket impl. [VERIFIED: sea-query-0.32.7/src/expr.rs line 3546]. Rejected as overkill — CONTEXT D-07 mentions the enum but the discretion in CONTEXT explicitly leaves internal shape to the planner. |
| `ConnectionTrait` generic on `exec_*` | `DatabaseConnection` direct | Loses `DatabaseTransaction` support (a transaction is a `ConnectionTrait` but not a `DatabaseConnection`). D-09 locks the generic. Confirmed correct. |
| Type-state to forbid empty SET at compile time | Runtime `EmptyUpdate` error | Type-state would require a marker generic. Adds API noise for a developer-error case. D-12 chose runtime. Aligned. |
| Sea-orm `mock` feature for unit tests | Real in-memory SQLite | Mock can't observe SQL-level atomicity (the whole point). Use real SQLite. |

**Installation:**

```toml
# ferro-orm/Cargo.toml
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
tokio = { version = "1", features = ["full", "test-util", "macros", "rt-multi-thread"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

**Version verification:**
- `sea-orm` workspace declaration `"1.0"` resolves to `1.1.19` per Cargo.lock. [VERIFIED: Cargo.lock]
- `thiserror = "2"` resolves to `2.0.17`. [VERIFIED: Cargo.lock]
- The library's `[dependencies] sea-orm = "1.0"` deliberately declares **no features** — at link time the consumer provides them. This is intentional (the library code never touches a driver). The `[dev-dependencies]` re-declaration with sqlite + runtime is what cargo uses for the crate's own tests.

## Architecture Patterns

### System Architecture Diagram

```
                         CONSUMER (e.g., ferro-reservation, app code)
                                       │
                                       │ &conn: &impl ConnectionTrait
                                       │  (DatabaseConnection or DatabaseTransaction)
                                       ▼
                ┌─────────────────────────────────────────────┐
                │  GuardedUpdate<E: EntityTrait>              │
                │  ┌───────────────────────────────────────┐  │
                │  │ filters: Condition (AND-combined)     │  │
                │  │ sets:    Vec<(E::Column, SimpleExpr)> │  │
                │  └───────────────────────────────────────┘  │
                │       │                                     │
                │       │ .exec_one(&conn) / .exec_at_most_one│
                │       │                                     │
                │       ▼                                     │
                │   Validation                                │
                │   ┌──────────────────────────────────┐      │
                │   │ sets.is_empty() ─Yes─► EmptyUpdate│     │
                │   │ sets.is_empty() ─No ──► continue  │     │
                │   └──────────────────────────────────┘      │
                │       │                                     │
                │       ▼                                     │
                │   Build UpdateMany                          │
                │   ┌──────────────────────────────────┐      │
                │   │ Update::many(E)                  │      │
                │   │   .filter(merged_condition)      │      │
                │   │   .col_expr(col_i, expr_i)*      │      │
                │   └──────────────────────────────────┘      │
                └───────────────────────┼─────────────────────┘
                                        │
                                        ▼
                       ┌──────────────────────────────────┐
                       │  sea_orm::Updater::exec(&conn)   │
                       │  → builds UPDATE … WHERE … SQL   │
                       │  → conn.execute(statement)       │
                       │  → returns UpdateResult          │
                       │      .rows_affected: u64         │
                       └────────────────┬─────────────────┘
                                        │
                                        ▼
                       ┌──────────────────────────────────┐
                       │  Database (SQLite / Postgres)    │
                       │  Atomic UPDATE — per-statement   │
                       │  serializability                 │
                       └────────────────┬─────────────────┘
                                        │
                                        ▼ rows_affected ∈ {0, 1, N>1}
                       ┌──────────────────────────────────┐
                       │  Post-processor (in ferro-orm)   │
                       │                                  │
                       │  exec_one:                       │
                       │   0   → Err(NoRowsAffected)      │
                       │   1   → Ok(())                   │
                       │   N>1 → Err(TooManyRows{N})      │
                       │                                  │
                       │  exec_at_most_one:               │
                       │   0   → Ok(false)                │
                       │   1   → Ok(true)                 │
                       │   N>1 → Err(TooManyRows{N})      │
                       │                                  │
                       │  DbErr (any tier) → Err(Db(e))   │
                       └──────────────────────────────────┘
```

### Recommended Project Structure

```
ferro-orm/
├── Cargo.toml             # leaf crate, Wave 1a metadata
├── README.md              # one-paragraph crate purpose + canonical example
└── src/
    ├── lib.rs             # module rustdoc + targeted re-exports + pub use guarded::*
    ├── guarded.rs         # GuardedUpdate<E> + #[cfg(test)] mod tests (D-16 cases 1-7)
    └── error.rs           # GuardedError enum
tests/
└── concurrent_decrement.rs # D-17: 10 tokio tasks vs counter at K=3
```

**Discretion:** A single `lib.rs` (no `guarded.rs` / `error.rs` split) is equally acceptable per CONTEXT, since the crate is small. The split above mirrors `ferro-wallet`'s convention and keeps `lib.rs` to module-level rustdoc + re-exports.

### Pattern 1: Building the `UpdateMany` lazily inside `exec_*`

**What:** Store `filters: Condition` and `sets: Vec<(E::Column, SimpleExpr)>` in the builder. Construct the `UpdateMany<E>` only when `exec_one` / `exec_at_most_one` is called. Reason: SeaORM's `UpdateMany` is not trivially clonable across multiple calls and the builder methods take `&mut`-style refs to the underlying `UpdateStatement`. Lazy construction keeps the builder a pure value type (no PhantomData magic exposed to consumers).

**When to use:** This is the canonical implementation shape for thin SeaORM extension builders. Used (in a query-builder direction) by `framework/src/database/query_builder.rs::QueryBuilder` — though that holds a `Select<E>` eagerly. For UPDATE, lazy construction is cleaner because there is no equivalent `Update::find` cursor to hold.

**Example:**

```rust
// Source: derived from sea-orm-1.1.19/src/query/update.rs lines 77-86, 187-211
//         and sea-orm-1.1.19/src/query/helper.rs lines 828-834 (QueryFilter::filter)
use sea_orm::sea_query::{Condition, IntoCondition, SimpleExpr};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Update, Value};

pub struct GuardedUpdate<E: EntityTrait> {
    entity: E,
    filters: Condition,
    sets: Vec<(E::Column, SimpleExpr)>,
}

impl<E: EntityTrait> GuardedUpdate<E> {
    pub fn new(entity: E) -> Self {
        Self {
            entity,
            filters: Condition::all(),  // AND-combiner
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
        // T: Into<Value> ⇒ T: Into<SimpleExpr> blanket impl
        // (sea-query-0.32.7/src/expr.rs line 3546)
        self.sets.push((col, SimpleExpr::Value(value)));
        self
    }

    pub async fn exec_one<C: ConnectionTrait>(self, conn: &C) -> Result<(), GuardedError> {
        let affected = self.exec_raw(conn).await?;
        match affected {
            0 => Err(GuardedError::NoRowsAffected),
            1 => Ok(()),
            n => Err(GuardedError::TooManyRows { affected: n }),
        }
    }

    pub async fn exec_at_most_one<C: ConnectionTrait>(
        self,
        conn: &C,
    ) -> Result<bool, GuardedError> {
        let affected = self.exec_raw(conn).await?;
        match affected {
            0 => Ok(false),
            1 => Ok(true),
            n => Err(GuardedError::TooManyRows { affected: n }),
        }
    }

    async fn exec_raw<C: ConnectionTrait>(self, conn: &C) -> Result<u64, GuardedError> {
        // Load-bearing — SeaORM's Updater::is_noop() short-circuits with
        // rows_affected: 0 when SET is empty, which would otherwise look like
        // a predicate miss. See sea-orm-1.1.19/src/executor/update.rs line 168.
        if self.sets.is_empty() {
            return Err(GuardedError::EmptyUpdate);
        }

        let mut stmt = Update::many(self.entity).filter(self.filters);
        for (col, expr) in self.sets {
            stmt = stmt.col_expr(col, expr);
        }
        let result = stmt.exec(conn).await?;  // From<DbErr> via #[from] on Db variant
        Ok(result.rows_affected)
    }
}
```

### Pattern 2: Targeted re-exports (D-03)

`src/lib.rs`:

```rust
//! Atomic conditional update primitive for Ferro applications.
//!
//! ... (module rustdoc with canonical inventory-decrement example)

mod error;
mod guarded;

pub use error::GuardedError;
pub use guarded::GuardedUpdate;

// Targeted re-exports — consumers calling the builder need these.
// Do NOT add `pub use sea_orm::*` per D-03.
pub use sea_orm::sea_query::{IntoCondition, SimpleExpr, Value};
pub use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait};
```

[VERIFIED: `IntoCondition`, `SimpleExpr`, `Value` live in `sea_orm::sea_query` re-export; `ConnectionTrait`, `EntityTrait`, `ColumnTrait`, `DbErr` live at the `sea_orm` crate root. Confirmed by `sea-orm-1.1.19/src/lib.rs` re-export structure.]

### Pattern 3: GuardedError shape (D-11/D-12/D-13)

```rust
// src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuardedError {
    #[error("guarded: predicate matched no rows")]
    NoRowsAffected,

    #[error("guarded: predicate matched {affected} rows (expected 1) — likely an index/uniqueness bug")]
    TooManyRows { affected: u64 },

    #[error("guarded: no columns to set — builder is empty")]
    EmptyUpdate,

    #[error("guarded: db error: {0}")]
    Db(#[from] sea_orm::DbErr),
}
```

[VERIFIED: Display prefix `"guarded: "` matches workspace pattern — `WalletError` uses `"wallet: "`, `ConfigError` uses `"config: "`. Grep-friendly.]

### Anti-Patterns to Avoid

- **`pub use sea_orm::*;`** — re-exports the whole SeaORM surface, makes the ferro-orm API non-inspectable in MCP, and creates churn every time SeaORM moves a symbol. D-03 forbids this.
- **Eager `UpdateMany<E>` storage in the builder.** Sea-orm's `UpdateMany` is `Clone` but exposes an internal `UpdateStatement` that gets mutated by `.filter` / `.col_expr`. Storing it eagerly forces awkward `mem::take` patterns or PhantomData juggling for type inference. Use the lazy `Vec` + `Condition` approach above.
- **A separate `set_null(col)` method.** Use `set_value(col, Value::* (None))` per the canonical sea-query example `Expr::value(Value::Int(None))` → `SET cake_id = NULL`. [VERIFIED: sea-query-0.32.7/src/query/update.rs lines 187-203]
- **Global `DB::get()` shortcut** for `exec_*`. D-09 explicitly rejects this; the caller passes `&conn` so transaction bracketing is visible at the call site.
- **`expect`/`unwrap` anywhere in the library code.** This is a library, not an app; every error path is `Result`. ferro-events, ferro-wallet, and ferro-stripe all panic-nowhere — match the convention.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| `UPDATE … WHERE …` SQL string assembly | Manual `format!` of SQL | `sea_orm::Update::many(entity).col_expr(col, expr).filter(cond)` | Parameter binding, cross-dialect quoting, value escaping, NULL handling — all already correct in sea-query. |
| Filter AND-composition | `Vec<SimpleExpr>` with manual concat | `Condition::all()` + `.add(IntoCondition)` | Already does AND-combining; trivially supports nested `Condition::any()` if a future caller needs OR. |
| Rows-affected counting | Custom `Statement` execution + manual unpacking | `UpdateMany::exec(&conn)` returns `UpdateResult { rows_affected: u64 }` | One line. Stable since sea-orm 0.x. [VERIFIED: sea-orm-1.1.19/src/executor/update.rs line 92] |
| Error mapping from `DbErr` | Manual `match` on every variant | `#[from] DbErr` via thiserror | Standard workspace convention. |
| Multi-set ordering | Custom `IndexMap<Column, Expr>` | Plain `Vec<(Column, SimpleExpr)>` + push semantics | Sea-query's `UpdateStatement::value` already preserves insertion order; later values for the same column override earlier ones at SQL build time (it pushes to `self.values: Vec`). Don't deduplicate in the builder. |

**Key insight:** SeaORM 1.x already provides the correct primitive. `GuardedUpdate` exists not to wrap unsafe SQL but to **add three things on top**: (a) the chainable surface a consumer prefers over `Update::many().filter().col_expr()` (which is fine but verbose), (b) the post-execution rows-affected → `GuardedError` mapping, (c) the `EmptyUpdate` guard for the `is_noop()` short-circuit. Without (b) and (c), every consumer hand-rolls the same `if result.rows_affected == 1 { … } else { … }` block — which is precisely the boilerplate that gets wrong in races.

## Runtime State Inventory

This is a NEW-CRATE phase, not a rename/refactor. No existing runtime state references "ferro-orm" or `GuardedUpdate`. The five runtime-state categories don't apply.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | None — no schema changes | — |
| Live service config | None — crate has no env vars (D-09 forbids global DB) | — |
| OS-registered state | None — no daemons / scheduled tasks | — |
| Secrets/env vars | None — crate has no config struct | — |
| Build artifacts | None — first build of the crate; no stale `target/` carryover from a renamed predecessor | — |

**Nothing found in any category. Verified by:** new-crate scaffold inspection (CONTEXT.md D-01, D-02), CLAUDE.md project-agnostic crates rule (no env config), grep for `ferro-orm` across the workspace returns zero results except the planning files for this phase.

## Common Pitfalls

### Pitfall 1: The `is_noop()` short-circuit silently masks empty-builder bugs

**What goes wrong:** A caller forgets to chain `set_expr` / `set_value`. They call `.exec_one(&conn)` expecting an obvious error. Without the `EmptyUpdate` guard, sea-orm's `Updater::exec` sees `query.get_values().is_empty() == true`, returns `UpdateResult { rows_affected: 0 }` WITHOUT executing any SQL, and `exec_one` maps that to `Err(NoRowsAffected)` — looking exactly like a predicate miss.

**Why it happens:** SeaORM 1.1.19 line 84 of `src/executor/update.rs`: `if self.is_noop() { return Ok(UpdateResult::default()); }`. `UpdateResult::default()` is `rows_affected: 0`. This is by design in sea-orm (a no-op should not hit the wire), but it conflicts with `GuardedUpdate`'s contract.

**How to avoid:** Explicit `if self.sets.is_empty() { return Err(EmptyUpdate); }` at the top of `exec_raw`. **D-12 is correct and load-bearing.** Test case 4 in D-16 (`EmptyUpdate returned when no set_* called`) is the regression guard.

**Warning signs:** A future contributor "simplifies" the builder by removing the `EmptyUpdate` check thinking it's dead code. The unit test must lock this behavior.

### Pitfall 2: SQLite test pool serializes everything when `max_connections = 1`

**What goes wrong:** The naive concurrent-decrement test (D-17) uses `sqlite::memory:` with `max_connections = 1` (the existing `framework/src/database/testing.rs` default). All 10 tokio tasks share a single connection slot; the sea-orm pool serializes them at the Rust layer before any SQL fires. The test "passes" — exactly 3 succeed, 7 see `NoRowsAffected` — but it proves only that the pool serializes, not that the SQL is race-free. A buggy non-atomic implementation (e.g., a read-then-write fallback) would also pass.

**Why it happens:** `sqlite::memory:` opens a fresh, per-connection memory database. With `max_connections = 1`, you only ever have one DB. Even with `max_connections = N`, separate `:memory:` connections are separate databases — so concurrent writes to "the same row" are literally impossible (it's N separate tables).

**How to avoid:** Use one of these for the concurrent test:
1. **Shared-cache memory:** `sqlite:file::memory:?cache=shared` with `max_connections >= 4` (recommended). All connections see the same in-memory DB; the SQLite serial writer enforces atomicity at the SQL layer.
2. **Temp-file SQLite:** `sqlite://{tempfile_path}` with `max_connections >= 4`. Slightly heavier but isolates well.

The unit tests (D-16 cases 1-7) can stick with `sqlite::memory:` + `max_connections = 1` — they don't need concurrency. Only the integration test (D-17) needs the shared-cache variant.

**Warning signs:** The integration test passes in N seconds with no SQLite contention warnings. Confirm by inserting a sanity check: run the same test with a deliberately-broken builder that does `SELECT` then `UPDATE` without a predicate — it should FAIL (allow >K successes) under shared-cache + `max_connections > 1`, but PASS under the naive `:memory:` setup. If both pass, the test setup is wrong.

### Pitfall 3: SeaORM `runtime-tokio-*` feature collision

**What goes wrong:** `ferro-orm/[dev-dependencies] sea-orm` declares `runtime-tokio-rustls` but the integration test runs inside a workspace context that already pulls `runtime-tokio-native-tls` from `framework`. Cargo's feature unification may pick the wrong runtime, or worse, error with "multiple sqlx runtime features enabled."

**Why it happens:** sqlx (which sea-orm wraps) requires exactly one runtime feature to be active. The `runtime-tokio-*` features in sea-orm are mutually exclusive at the sqlx level.

**How to avoid:**
- Option A: Match `framework`'s `runtime-tokio-native-tls`. This is safest (one runtime across the workspace).
- Option B: Run `ferro-orm` tests via `cargo test -p ferro-orm` (which excludes framework from the build graph), and accept that `cargo test --all-features` may need to pick a single runtime.

Recommend **Option A** for simplicity and to keep `cargo test --all-features` green. Document the choice in the dev-dep line.

**Warning signs:** `cargo test --all-features` from the workspace root produces sqlx feature-collision errors after ferro-orm is added.

### Pitfall 4: `Update::many` succeeds when filter matches >1 rows — `TooManyRows` is real

**What goes wrong:** A caller writes `.filter(Column::Status.eq("pending"))` thinking only one row matches. There are actually two rows. The `UPDATE` succeeds, mutating both. Without `TooManyRows`, this is silent corruption — two reservations both look "committed."

**Why it happens:** `Update::many` is explicitly multi-row. The "guarded" contract in `GuardedUpdate::exec_one` is that the filter is *intended* to be unique-key-equivalent. If it isn't, that's a bug — and the right thing to do is shout, not silently succeed.

**How to avoid:** D-13 keeps `TooManyRows`. Test case 3 in D-16 (predicate matches >1 row → both methods error) is the regression guard. Document in rustdoc: "every guarded update is morally a unique-key-equivalent operation; `TooManyRows` is your index/uniqueness bug."

**Warning signs:** A consumer calls `exec_one` on a filter that doesn't include a unique-key column. Code review should catch this; the runtime error is the safety net.

### Pitfall 5: First-publish bootstrap requires a personal crates.io token

**What goes wrong:** CI tries to publish `ferro-orm` for the first time using `CARGO_REGISTRY_TOKEN`, which (per workspace convention) is scoped to `publish-update` only. Publishing a brand-new crate name requires `publish-new`, which the CI token doesn't have. Publish fails; release is blocked.

**Why it happens:** Locked-down CI token policy. Documented in MEMORY `project_ferro_publish_token_scoping.md` and re-verified in Phase 151 (ferro-wallet's first publish).

**How to avoid:** The phase plan MUST include a manual first-publish step (analogous to Phase 151 PLAN-09):
1. Bump version & merge to master.
2. CI auto-publishes Wave 1a — `ferro-orm` fails because it doesn't exist on crates.io yet.
3. Locally: `cargo publish -p ferro-orm` using a personal `publish-new`-scoped token.
4. Subsequent versions auto-publish via CI's existing `publish-update` token.

**Warning signs:** The PR is merged and the CI publish step errors with "not found" or "no upload permission" on `ferro-orm`. This is expected exactly once. The bootstrap signal is "I just published ferro-orm v0.x.y from local" from the human operator.

## Code Examples

### Example 1: The canonical use case (inventory decrement)

[CITED: `.planning/research/INVENTORY-PRIMITIVES.md` §`ferro-orm::guarded` lines 104-113; CONTEXT.md `<specifics>`]

```rust
use ferro_orm::{GuardedUpdate, SimpleExpr};
use sea_orm::sea_query::Expr;
use sea_orm::ColumnTrait;
// ... inventory_units::{Column, Entity} from caller's entity module

GuardedUpdate::new(inventory_units::Entity)
    .filter(inventory_units::Column::Id.eq(unit_id))
    .filter(inventory_units::Column::Quantity.gte(needed))
    .set_expr(
        inventory_units::Column::Quantity,
        Expr::col(inventory_units::Column::Quantity).sub(needed),
    )
    .exec_one(&txn)
    .await?;
// — exactly one row matched and was decremented atomically,
//   OR Err(NoRowsAffected) signaling capacity exhausted.
```

### Example 2: Multi-column atomic set (D-07 multi-set, D-16 case 5)

```rust
use ferro_orm::{GuardedUpdate, Value};
use sea_orm::sea_query::Expr;
use sea_orm::ColumnTrait;
use chrono::Utc;

GuardedUpdate::new(reservations::Entity)
    .filter(reservations::Column::Id.eq(handle_id))
    .filter(reservations::Column::Status.eq("held"))
    .set_value(reservations::Column::Status, Value::String(Some(Box::new("committed".into()))))
    .set_value(reservations::Column::CommittedAt, Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))))
    .exec_one(&conn)
    .await?;
// One UPDATE statement, two columns set, race-free transition from "held" → "committed".
```

### Example 3: Optimistic update where 0 rows is normal (D-08 exec_at_most_one)

```rust
let updated = GuardedUpdate::new(sessions::Entity)
    .filter(sessions::Column::Token.eq(&token))
    .filter(sessions::Column::ExpiresAt.gt(now))
    .set_value(sessions::Column::LastSeenAt, Value::ChronoDateTimeUtc(Some(Box::new(now))))
    .exec_at_most_one(&conn)
    .await?;

if !updated {
    // session expired or token unknown — not an error, just a normal outcome
    return Err(AuthError::SessionExpired);
}
```

### Example 4: Test scaffolding for D-16 unit tests

```rust
// In ferro-orm/src/guarded.rs, #[cfg(test)] mod tests
use sea_orm::{
    ConnectionTrait, Database, DatabaseBackend, DbErr, EntityTrait, Schema, Set, Statement,
};

// Define a tiny inline entity — the SeaORM canonical pattern.
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

async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.expect("connect");
    // Schema::create_table_from_entity is the lightweight alternative to sea-orm-migration
    let schema = Schema::new(DatabaseBackend::Sqlite);
    let stmt = schema.create_table_from_entity(counters::Entity);
    conn.execute(conn.get_database_backend().build(&stmt))
        .await
        .expect("create table");
    conn
}

#[tokio::test]
async fn predicate_matches_one_row_succeeds() {
    let conn = fresh_db().await;
    // Insert a row via the entity's standard API…
    counters::Entity::insert(counters::ActiveModel {
        id: Set(1),
        quantity: Set(5),
    })
    .exec(&conn)
    .await
    .expect("insert");

    GuardedUpdate::new(counters::Entity)
        .filter(counters::Column::Id.eq(1))
        .filter(counters::Column::Quantity.gte(3))
        .set_expr(
            counters::Column::Quantity,
            sea_orm::sea_query::Expr::col(counters::Column::Quantity).sub(3),
        )
        .exec_one(&conn)
        .await
        .expect("guarded update");
}
```

[VERIFIED: `Schema::create_table_from_entity` is the standard sea-orm pattern for derive-only test schemas — avoids the `sea-orm-migration` dev-dep. Confirmed in sea-orm-1.1.19 module structure.]

### Example 5: The concurrent-decrement integration test (D-17)

```rust
// tests/concurrent_decrement.rs
use ferro_orm::GuardedUpdate;
use sea_orm::sea_query::Expr;
use sea_orm::{ConnectionTrait, ColumnTrait, ConnectOptions, Database, DatabaseBackend, EntityTrait, Schema, Set};
use std::sync::Arc;

mod counters {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "counters")]
    pub struct Model {
        #[sea_orm(primary_key)] pub id: i32,
        pub quantity: i32,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)] pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ten_tasks_against_capacity_three_exactly_three_succeed() {
    // Shared-cache memory variant so multiple connections see the same DB
    let mut opts = ConnectOptions::new("sqlite:file::memory:?cache=shared");
    opts.max_connections(4).min_connections(1);
    let conn = Arc::new(Database::connect(opts).await.expect("connect"));

    // Schema
    let schema = Schema::new(DatabaseBackend::Sqlite);
    let stmt = schema.create_table_from_entity(counters::Entity);
    conn.execute(conn.get_database_backend().build(&stmt)).await.unwrap();

    // Seed: id=1, quantity=3
    counters::Entity::insert(counters::ActiveModel {
        id: Set(1), quantity: Set(3),
    }).exec(&*conn).await.unwrap();

    // Spawn 10 tasks each trying to decrement by 1 with guard `quantity >= 1`
    let mut tasks = Vec::new();
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

    let results: Vec<_> = futures::future::join_all(tasks).await
        .into_iter().map(Result::unwrap).collect();

    let successes = results.iter().filter(|r| r.is_ok()).count();
    let no_rows = results.iter().filter(|r|
        matches!(r, Err(ferro_orm::GuardedError::NoRowsAffected))
    ).count();

    assert_eq!(successes, 3, "exactly 3 of 10 tasks should succeed");
    assert_eq!(no_rows, 7, "the other 7 should fail with NoRowsAffected");

    // Final quantity should be 0
    let final_row = counters::Entity::find_by_id(1).one(&*conn).await.unwrap().unwrap();
    assert_eq!(final_row.quantity, 0);
}
```

Note `futures` will need to be a dev-dep if used; alternative is hand-rolling with `tokio::join!` or a loop. Plan accordingly.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-rolled SQL `format!("UPDATE … WHERE …")` + `Statement::from_string` | `Update::many(entity).col_expr(col, expr).filter(cond).exec(conn)` | SeaORM 0.x (years ago) | Type-safe column references, dialect-portable, parameter-bound. The only reason to hand-roll SQL is dialect-specific features (e.g., `RETURNING`) sea-orm doesn't yet abstract — D-10 defers this. |
| `UpdateOne` (model-based, requires `ActiveModel` with PK set) | `UpdateMany` (entity-based, free-form filter) | sea-orm 0.x | For guarded updates the predicate isn't always just the PK (e.g., `WHERE id = ? AND quantity >= ?`). `UpdateMany` is the right primitive. `UpdateOne` is for "I have an ActiveModel and want to persist it." |
| Per-statement transactions for atomic state transitions | Single-statement `UPDATE … WHERE …` (race-free on SQLite serial-writer + Postgres `READ COMMITTED`) | This phase | The whole point of the kernel. |

**Not deprecated, still relevant:**
- The existing `framework/src/database/query_builder.rs::QueryBuilder` (SELECT-oriented) remains the way to express read queries. `GuardedUpdate` is the analog for guarded writes. They coexist; D-02 explicitly defers any consolidation.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Workspace will use `runtime-tokio-rustls` in dev-deps without conflicting with framework's `runtime-tokio-native-tls` | Standard Stack / Pitfall 3 | If they conflict at unification time, switch to `runtime-tokio-native-tls`. No design impact; cosmetic Cargo.toml change. Locked Option A (match framework) recommended. |
| A2 | `sqlite:file::memory:?cache=shared` will reliably expose the race in the D-17 test under a 4-worker tokio multi-thread runtime | Pitfall 2 / Code Example 5 | If the shared-cache variant fails to actually contend (e.g., due to SQLite's BEGIN IMMEDIATE behavior), fall back to a `tempfile::NamedTempFile`-backed `sqlite://` URL. Both are documented; test setup is the only change needed. |
| A3 | The crate name `ferro-orm` is available on crates.io | Pitfall 5 / D-04 | Confirm with `cargo search ferro-orm` before bootstrap. If taken, fall back to `ferro-orm-rs` matching the `ferro-rs` precedent. Phase has no contingent ferro-orm-something fallback documented. |
| A4 | First-publish bootstrap follows the exact Phase 151 pattern (local `cargo publish -p ferro-orm` with personal `publish-new` token, then CI takes over) | Pitfall 5 | If the token policy has changed since 2026-05-11, the operator step changes. Verify with `~/.claude/projects/.../memory/project_ferro_publish_token_scoping.md` before merging. |
| A5 | CHANGELOG voice and structure should mirror the existing `ferro-wallet` section (single date-prefixed `[0.x.y]` header under a `## ferro-orm` top-level section) | D-25 | Cosmetic only; the planner can adjust the format. CHANGELOG.md format is loose Keep-a-Changelog. |
| A6 | Test entity for unit tests can be defined inline in `#[cfg(test)] mod tests` using `DeriveEntityModel`, and schema can be created via `Schema::create_table_from_entity` + `ConnectionTrait::execute` — no `sea-orm-migration` dev-dep needed | Code Example 4 / Standard Stack | If `Schema::create_table_from_entity` is unavailable or unstable in 1.1.19, fall back to a raw `Statement::from_string(DbBackend::Sqlite, "CREATE TABLE counters (id INTEGER PRIMARY KEY, quantity INTEGER NOT NULL)")`. Verified path exists in sea-orm; minor risk only. |

## Open Questions

1. **Bump target version — CONTEXT.md says `0.2.24 → 0.2.25`, but workspace `Cargo.toml` shows `0.2.30` and the latest git tag is `v0.2.24`.**
   - What we know: CI's `check-version` job auto-bumps the patch only if the current Cargo.toml version is already tagged. With Cargo at `0.2.30` and tag at `v0.2.24`, `0.2.30` is untagged, so CI will publish `0.2.30` as-is (no bump needed in the PLAN).
   - What's unclear: Has the workspace been advancing 0.2.25→0.2.30 across phases 144-151 without tagging? STATE.md is stale (says "workspace version: 0.2.24" while Cargo.toml says `0.2.30`).
   - Recommendation: The planner should treat D-23 as "no manual bump required for this phase" and let CI publish `0.2.30` (or whatever the live Cargo.toml says when the phase merges). The plan-checker should NOT block on the stale `0.2.25` target. STATE.md cleanup is a separate concern — note it in the phase summary.

2. **Should `IntoCondition` be re-exported from `ferro_orm` root, or only via `ferro_orm::sea_query`?**
   - What we know: D-03 says "Targeted re-exports — including `IntoCondition`." Pattern 2 re-exports at the crate root.
   - What's unclear: Whether consumers will primarily write `use ferro_orm::IntoCondition` (root) or `use sea_orm::sea_query::IntoCondition` (pass-through). Both work.
   - Recommendation: Re-export at the crate root (Pattern 2 above). The Phase 154 reservation crate is the immediate consumer and benefits from a flat import surface.

3. **Should `Expr` (the sea-query expression builder) be re-exported too?**
   - The canonical example uses `Expr::col(Column::Quantity).sub(needed)` — that `Expr` is `sea_orm::sea_query::Expr`. D-03 lists `IntoCondition`, `SimpleExpr`, `Value` but not `Expr`. Consumers will need `Expr` for any value-derived update.
   - Recommendation: Add `Expr` to the targeted re-exports. The CONTEXT D-03 list is illustrative ("a consumer needs to call the builder"); omitting `Expr` would force every consumer to also depend on `sea-orm` directly just for one symbol — defeating the cleanliness motivation. Flag this for `discuss-phase` if the planner wants explicit confirmation.

4. **Tokio dev-dep features — `full` or minimal?**
   - The unit tests need `macros` (for `#[tokio::test]`) and `rt-multi-thread` (for `flavor = "multi_thread"` in the D-17 test) at minimum.
   - The `ferro-events` precedent uses `["full", "test-util"]` (which includes `rt-multi-thread` and `macros`).
   - Recommendation: Match `ferro-events`: `tokio = { version = "1", features = ["full", "test-util"] }`. Slightly larger than minimum but matches the workspace convention.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain 1.88.0 | All ferro crates | ✓ | per `rust-toolchain.toml` workspace pin | — |
| `cargo` | Build & test | ✓ | bundled with toolchain | — |
| sea-orm 1.0+ | ferro-orm dependency | ✓ | 1.1.19 in Cargo.lock | — |
| thiserror 2 | Error derive | ✓ | 2.0.17 in Cargo.lock | — |
| SQLite (in-memory via sqlx-sqlite) | Tests only | ✓ | bundled (sqlx-sqlite includes libsqlite3) | — |
| crates.io network access for first publish | Bootstrap step | ✓ | accessible from local terminal | none (publish is required for Phase 154 to depend on it) |
| Personal `publish-new`-scoped crates.io token | First publish only | ⚠ user-provided | — | Bootstrap step is manual; CI's `publish-update` token does subsequent releases |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** First-publish credential is operator-provided (one-time, documented in Pitfall 5).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `tokio` async tests via `#[tokio::test]` (workspace convention; no separate test framework) |
| Config file | `ferro-orm/Cargo.toml` `[dev-dependencies]` — no separate test config |
| Quick run command | `cargo test -p ferro-orm` |
| Full suite command | `cargo test --all-features` (from workspace root) |

### Phase Requirements → Test Map

The phase has no `REQ-XX` requirement IDs (`phase_req_ids` is null). The verification surface is the D-16 unit cases + D-17 integration test:

| Test ID | Behavior (from D-16/D-17) | Test Type | Automated Command | File Exists? |
|---------|---------------------------|-----------|-------------------|-------------|
| T-16-1 | Predicate matches → 1 row affected → `exec_one` returns `Ok(())` | unit | `cargo test -p ferro-orm predicate_matches_one_row_succeeds` | ❌ Wave 0 |
| T-16-2 | Predicate fails → 0 rows → `exec_one` returns `Err(NoRowsAffected)`, `exec_at_most_one` returns `Ok(false)` | unit | `cargo test -p ferro-orm predicate_fails_zero_rows` | ❌ Wave 0 |
| T-16-3 | Predicate matches >1 row → both methods return `Err(TooManyRows { affected: 2 })` | unit | `cargo test -p ferro-orm predicate_matches_multiple_rows` | ❌ Wave 0 |
| T-16-4 | `EmptyUpdate` returned when no `set_*` called | unit | `cargo test -p ferro-orm empty_update_no_sets` | ❌ Wave 0 |
| T-16-5 | Multiple `.set_expr` / `.set_value` calls produce a single UPDATE that mutates all columns atomically | unit | `cargo test -p ferro-orm multi_column_set_atomic` | ❌ Wave 0 |
| T-16-6 | Builder works inside `&DatabaseTransaction` (rollback rolls back the guarded update) | unit | `cargo test -p ferro-orm transaction_rollback` | ❌ Wave 0 |
| T-16-7 | Multiple `.filter` calls AND-combine | unit | `cargo test -p ferro-orm filter_and_combine` | ❌ Wave 0 |
| T-17-1 | 10 tokio tasks vs counter at K=3 → exactly 3 `Ok(())`, 7 `NoRowsAffected`, final counter = 0 | integration | `cargo test -p ferro-orm --test concurrent_decrement` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-orm`
- **Per wave merge:** `cargo test --all-features` from workspace root + `cargo clippy --all --all-targets -- -D warnings` + `cargo fmt --all -- --check`
- **Phase gate:** All three commands green before `/gsd-verify-work`. First-publish bootstrap (Pitfall 5) is post-merge and manual.

### Wave 0 Gaps

- [ ] `ferro-orm/Cargo.toml` — crate metadata + deps + dev-deps (Wave 0)
- [ ] `ferro-orm/src/lib.rs` — module rustdoc + re-exports
- [ ] `ferro-orm/src/error.rs` — `GuardedError` enum
- [ ] `ferro-orm/src/guarded.rs` — `GuardedUpdate<E>` + `#[cfg(test)] mod tests` covering T-16-1 through T-16-7
- [ ] `ferro-orm/tests/concurrent_decrement.rs` — T-17-1
- [ ] `ferro-orm/README.md` — one-paragraph crate purpose + canonical example (mirror `ferro-wallet/README.md` shape)
- [ ] `Cargo.toml` (workspace root) — add `"ferro-orm"` to `[workspace.members]`
- [ ] `.github/workflows/publish.yml` — append `ferro-orm` to `WAVE1A_CRATES` string on line 201
- [ ] `CHANGELOG.md` — new `## ferro-orm` section above (or below) the existing `## ferro-wallet` section, with `### [0.x.y] — YYYY-MM-DD` entry
- [ ] `CLAUDE.md` — add `ferro-orm` row to the Workspace Structure table (line ~58, after `ferro-whatsapp`)
- [ ] `docs/src/SUMMARY.md` — add `[Atomic Updates](features/database/atomic-updates.md)` OR equivalent under Features (see Open Question — current docs has `[Database](features/database.md)` as a single page; sub-page nesting needs a planner decision)
- [ ] `docs/src/database/atomic-updates.md` — new user-facing doc page (D-21)

No framework install needed — workspace already has Rust 1.88.0 + sea-orm + tokio.

## Security Domain

`security_enforcement` is not set in `.planning/config.json` for this project. Default is "enabled." For a database UPDATE primitive, the relevant ASVS surface is narrow:

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | crate doesn't authenticate |
| V3 Session Management | no | crate doesn't manage sessions |
| V4 Access Control | no | caller is responsible for authorization before invoking the builder |
| V5 Input Validation | yes | typed `E::Column` and `SimpleExpr` prevent column-name injection at the API surface; sea-query parameterizes values |
| V6 Cryptography | no | no crypto |
| V8 Data Protection | partial | atomic state transitions prevent the TOCTOU class of bugs (e.g., double-spend, over-reservation) — this IS the security feature this crate provides |

### Known Threat Patterns for `ferro-orm`

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection via dynamic column names | Tampering | `E::Column` is typed; the API doesn't accept strings for column identifiers. sea-query handles quoting. [VERIFIED: typed-builder shape per `UpdateMany::col_expr<T: IntoIden>`] |
| SQL injection via value substitution | Tampering | sea-query parameterizes all `Value`s. The builder never concatenates user input into SQL strings. |
| TOCTOU race in read-then-write capacity checks | Tampering / Repudiation | The whole point of the crate: single `UPDATE … WHERE …` statement replaces the read-check-write pattern. DB-level atomicity is the mitigation. |
| Wrong-row update (filter matches > 1 row) | Tampering | `TooManyRows { affected }` error surfaces the bug loudly instead of silently corrupting state. |
| Empty UPDATE (programmer bug) silently looking like predicate failure | Repudiation (debugging confusion, not malicious) | `EmptyUpdate` runtime check — see Pitfall 1. |

No new attack surface; this crate REDUCES the existing attack surface by eliminating an entire class of TOCTOU bugs at the framework level.

## Sources

### Primary (HIGH confidence)

- `sea-orm-1.1.19/src/query/update.rs` lines 32-86, 182-211 — `Update::many`, `UpdateMany::col_expr`, `UpdateMany::set`. Verified directly from registry source.
- `sea-orm-1.1.19/src/executor/update.rs` lines 14-19, 80-95, 168-170 — `UpdateResult { rows_affected: u64 }`, `Updater::exec`, `is_noop()` short-circuit. Verified directly from registry source.
- `sea-orm-1.1.19/src/query/helper.rs` lines 828-834 — `QueryFilter::filter` calls `cond_where(filter.into_condition())`. AND-combining confirmed.
- `sea-query-0.32.7/src/expr.rs` line 3546-3553 — `impl<T: Into<Value>> From<T> for SimpleExpr`. Confirms `Value: Into<SimpleExpr>` blanket impl.
- `sea-query-0.32.7/src/query/update.rs` lines 205-212 — `UpdateStatement::value<C: IntoIden, T: Into<SimpleExpr>>`. Confirms the column→expr backing store accepts both.
- `sea-query-0.32.7/src/query/condition.rs` lines 597-606 — `ConditionalStatement::cond_where`, `impl IntoCondition for SimpleExpr`. Confirms `Column::Id.eq(value)` works directly as a filter.
- `framework/src/database/query_builder.rs` lines 86-92 — existing `.filter(impl IntoCondition)` pattern in the workspace. Stylistic mirror.
- `framework/src/database/testing.rs` lines 94-122 — existing in-memory SQLite testing pattern (with the `max_connections = 1` pitfall noted).
- `ferro-wallet/Cargo.toml`, `ferro-events/Cargo.toml` — Wave 1a Cargo.toml conventions.
- `.github/workflows/publish.yml` lines 196-219 — Wave 1a publish stage and `WAVE1A_CRATES` string.
- `Cargo.toml` workspace root — workspace.members (line 24 = `ferro-wallet`), `[workspace.package] version = "0.2.30"` (line 28).
- `Cargo.lock` — sea-orm 1.1.19, thiserror 2.0.17 pinned.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`ferro-orm::guarded` (lines 104-135) — design spec.
- `.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md` lines 980-1023 — Phase 151 publish.yml + Cargo.toml edit patterns (analog).
- `.planning/phases/151-ferro-wallet-crate/151-09-SUMMARY.md` — first-publish bootstrap pattern.

### Secondary (MEDIUM confidence)

- `ferro-mcp/src/tools/application_info.rs` lines 206-241 — `get_installed_crates` is fully dynamic; confirms D-22 finding that no MCP code changes are needed.
- `ferro-mcp/src/tools/code_templates.rs`, `generation_context.rs` — no UPDATE/race/guarded references found via grep; confirms zero MCP impact.

### Tertiary (LOW confidence)

- None — every claim above has at least one primary source.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — versions verified against Cargo.lock; API surface verified against registry source.
- Architecture (builder shape, error mapping, exec variants): HIGH — pattern is a direct mechanical wrap over verified sea-orm primitives.
- Pitfalls: HIGH — Pitfall 1 (is_noop short-circuit) and Pitfall 2 (sqlite max_connections=1) are both verified against source; Pitfall 5 is verified against Phase 151's actual experience.
- Concurrency contract (atomicity): HIGH — the SQLite serial-writer claim is widely documented and the `READ COMMITTED` claim on Postgres for `UPDATE … WHERE …` is standard SQL semantics.
- Test scaffolding: MEDIUM — A6 (Schema::create_table_from_entity availability) needs runtime confirmation by the implementer; fallback is documented.
- Version bump target (0.2.24 vs 0.2.30): MEDIUM — Open Question 1 documents the discrepancy.

**Research date:** 2026-05-13
**Valid until:** 2026-06-13 (30 days — sea-orm is on the stable 1.x line; thiserror 2 is also stable; minimal churn risk)
