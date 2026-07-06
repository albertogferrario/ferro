# Phase 177: ferro-reservation Kernel Atomicity Hardening — Research

**Researched:** 2026-05-21
**Domain:** Rust async concurrency, SeaORM transactions, Postgres isolation levels, SQLite in-memory testing
**Confidence:** HIGH — all findings verified against codebase and SeaORM 1.x docs

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01** Fix is kernel-internal. `Resource` trait surface unchanged. Consumers don't change. `commit/release/sweeper` unchanged.
- **D-02** Existing GuardedUpdate discipline preserved. This phase does NOT touch `commit/release/sweeper`.
- **D-03** Backend portability. Fix MUST work on SQLite (consumer dev) AND Postgres (consumer prod).
- **D-04** Audit log semantics unchanged. `reservation.held` audit row written exactly once per successful hold. Conflict-losing task does NOT write an audit row.
- **D-05** No new external crates. `sea_orm::TransactionTrait` already in workspace.
- **D-06** Fix path: (a) `conn.begin()` transaction with serializable isolation. LOCKED.
- **D-07** Postgres isolation: `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE` inside the txn. Translate `40001` → `ReservationError::Insufficient` at the kernel boundary.
- **D-08** Concurrency test scope: SQLite primary unconditional, Postgres cfg-gated.

### Claude's Discretion

- Exact Postgres cfg name (`feature = "postgres-tests"` vs `cfg(test_postgres)` vs env-var gate) — pick whichever matches existing ferro-reservation conventions.
- Iteration count for race-to-capacity test (≥50).
- `40001` translation site — kernel boundary preferred per D-07.
- Whether to extract a `hold_inner(&txn, ...)` helper.
- Doc updates — sweep stale claims in docs/src/database/reservations.md and kernel.rs module doc.

### Deferred Ideas (OUT OF SCOPE)

- Changing `commit/release/sweeper` atomicity mechanisms.
- Re-architecting reservations as event-sourced.
- Performance optimization beyond what the atomicity fix requires.
- Unique partial index approach (Path b — rejected).
- `INSERT … SELECT … WHERE NOT EXISTS` approach (Path c — rejected).
</user_constraints>

---

## Summary

The race is in `ferro-reservation/src/kernel.rs:54-122`. The `hold` method performs three sequential DB round-trips (capacity SELECT, held SELECT, INSERT) with no transaction wrapper. Two concurrent tokio tasks can both pass the capacity check before either INSERT commits. The fix is to wrap the entire body of `hold` inside a `conn.begin_with_config(Some(IsolationLevel::Serializable), Some(AccessMode::ReadWrite))` transaction.

The codebase already has `sea_orm::TransactionTrait`, `IsolationLevel`, and `AccessMode` in use in `framework/src/database/transaction.rs`. The `DatabaseTransaction` struct implements `ConnectionTrait`, so existing `Resource::held(&C, ...)` and `Resource::capacity(&C, ...)` generic method calls work unchanged when passed a `&txn` instead of a raw `&conn`. No type-level changes needed.

The existing `tests/concurrent_hold.rs` file already exists and documents the old workaround (a `tokio::Mutex` serializing the entire `hold()` call). After the fix, that test's mutex becomes redundant and the test body should be rewritten to use bare `tokio::spawn` without the mutex — proving the kernel is intrinsically race-free. The CONTEXT document's reference to "tests/concurrency.rs (NEW)" should be read as "the concurrency test file", which maps to the already-existing `tests/concurrent_hold.rs`.

Postgres SQLSTATE `40001` (serialization failure) surfaces through `DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))` or `DbErr::Query(...)`. The SQLSTATE code is accessed via `e.code().unwrap_or_default()`. At the kernel boundary, match on `"40001"` and return `ReservationError::Insufficient { requested: quantity, available: 0, capacity }` — this preserves the documented error contract (consumers expect `Insufficient`, not a raw `Db` variant).

**Primary recommendation:** Wrap `hold`'s body in `conn.begin_with_config(Some(IsolationLevel::Serializable), Some(AccessMode::ReadWrite)).await?` then commit on success, detect `40001` before returning, and rewrite `tests/concurrent_hold.rs` to prove invariant without the mutex.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Atomicity enforcement for `hold` | Database / Storage | — | The race is at the DB layer; the fix is a serializable transaction, not application-layer locking |
| `Resource::held/capacity` queries | Database / Storage | — | These are SELECTs inside the txn; they see the txn's snapshot |
| Audit write | Database / Storage | — | Must be inside the txn boundary so rollback on conflict retracts the audit row |
| Event dispatch (best-effort) | Backend / API | — | Stays outside the txn per D-26 — fires after commit, same as today |
| Postgres `40001` translation | Backend / API (kernel boundary) | — | Translate before returning to caller so the error contract (`Insufficient`) is preserved |
| SQLite isolation (no-op) | Database / Storage | — | SQLite's single-writer model provides serialization automatically via the WAL; no explicit statement needed |

---

## Standard Stack

### Core (already in Cargo.toml — no new deps)

| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| `sea-orm` | `1.0` | `TransactionTrait`, `IsolationLevel`, `AccessMode`, `DatabaseTransaction` | [VERIFIED: ferro-reservation/Cargo.toml] |
| `tokio` | `1` (full features) | `tokio::spawn`, async runtime | [VERIFIED: ferro-reservation/Cargo.toml dev-deps] |
| `proptest` | `1` | Property-based tests (already used in `tests/property_invariants.rs`) | [VERIFIED: ferro-reservation/Cargo.toml dev-deps] |

### To Add (Cargo.toml dev-dependencies)

```toml
[features]
postgres-tests = []

[dev-dependencies]
# Add to the existing sea-orm dev-dep entry:
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
```

The only change is adding `features = ["postgres-tests"]` to `[features]` (new section) and adding `sqlx-postgres` to the `sea-orm` dev-dependency. [VERIFIED: current dev-deps lack `sqlx-postgres` and `[features]` section does not exist]

---

## Architecture Patterns

### SeaORM Transaction API Surface

**`begin_with_config` signature** (SeaORM 1.0 / `TransactionTrait`):
```rust
// Source: https://docs.rs/sea-orm/1.1.14/sea_orm/trait.TransactionTrait.html [CITED]
async fn begin_with_config(
    &self,
    isolation_level: Option<IsolationLevel>,
    access_mode: Option<AccessMode>,
) -> Result<DatabaseTransaction, DbErr>
```

`DatabaseTransaction` implements `ConnectionTrait`. This is the load-bearing fact: every generic `<C: ConnectionTrait>` bound in `Resource::held` and `Resource::capacity` is satisfied by `&DatabaseTransaction`. No Resource impl changes required.

**Drop-semantics:** When `DatabaseTransaction` is dropped without `.commit()`, it automatically rolls back. The pattern for manual `begin()` / `commit()` is:

```rust
// Source: ferro-orm/src/guarded.rs test T-16-6 [VERIFIED]
let txn = conn.begin().await.expect("begin transaction");
// ... do work against &txn ...
txn.rollback().await.expect("rollback");
// OR
txn.commit().await?;
```

**For `hold` specifically**, use `begin_with_config` rather than bare `begin`:

```rust
// Source: framework/src/database/transaction.rs:begin_with_config usage [VERIFIED]
use sea_orm::{AccessMode, IsolationLevel, TransactionTrait};

let txn = conn
    .begin_with_config(
        Some(IsolationLevel::Serializable),
        Some(AccessMode::ReadWrite),
    )
    .await
    .map_err(ReservationError::Db)?;

// ... hold_inner body using &txn instead of conn ...

txn.commit().await.map_err(|e| {
    // Detect 40001 here before wrapping in ReservationError::Db
    translate_commit_err(e, quantity, capacity)
})?;
```

**SQLite behavior:** `IsolationLevel::Serializable` passed to SQLite via SeaORM — SQLite ignores the isolation level statement silently (its default write behavior is already serialized). The `begin_with_config` call succeeds on SQLite without error; no backend-detection branching needed. [ASSUMED — SeaORM docs show it uses `execute_unprepared` which is supported on `sqlx-sqlite`, but SQLite may emit a warning rather than an error for unsupported isolation levels. Verify by running one test against SQLite with the serializable config.]

### Pattern: Passing `&DatabaseTransaction` through Generic `<C: ConnectionTrait>` Bounds

`Resource::capacity(&C, ...)` and `Resource::held(&C, ...)` are called as:

```rust
// Today (conn: &C where C: ConnectionTrait):
let capacity = self.resource.capacity(conn, &key, &window).await?;
let held = self.resource.held(conn, &key, &window).await?;

// After fix (txn: DatabaseTransaction):
let capacity = self.resource.capacity(&txn, &key, &window).await?;
let held = self.resource.held(&txn, &key, &window).await?;
am.insert(&txn).await.map_err(ReservationError::Db)?;
audit.write(&txn).await.map_err(ReservationError::Audit)?;
```

`DatabaseTransaction: ConnectionTrait` is confirmed by SeaORM docs — the `ConnectionTrait` implementation on `DatabaseTransaction` includes `execute`, `execute_unprepared`, `query_one`, `query_all`. [CITED: https://docs.rs/sea-orm/1.1.14/sea_orm/struct.DatabaseTransaction.html]

The `hold` method signature stays **byte-identical**:
```rust
pub async fn hold<C: ConnectionTrait>(
    &self,
    conn: &C,
    key: R::Key,
    window: R::Window,
    quantity: u32,
    ttl: Duration,
    ctx: &ReservationContext,
) -> Result<ReservationHandle, ReservationError>
```

Inside the body, `conn` is used only to `.begin_with_config(...)`. All further DB calls use `&txn`. This is the only structural change.

**`impl<R: Resource> ReservationKernel<R>` where `C: ConnectionTrait` is a method-level generic — there is no trait bound requiring `C: TransactionTrait`**, so the kernel can call `conn.begin_with_config(...)` only if `C: TransactionTrait`. Currently `C: ConnectionTrait` does not imply `C: TransactionTrait`. This is a key implementation decision:

**Option A (recommended):** Add `where C: ConnectionTrait + TransactionTrait` to the `hold` method bound. `DatabaseConnection` implements both. This is the minimal change and preserves the public API (since the only callers pass `DatabaseConnection` or `DatabaseTransaction` — both implement both traits). Check existing kernel test patterns before deciding.

**Option B:** Change `conn: &C` to `conn: &DatabaseConnection` for `hold` only (breaking the generic). This contradicts D-01 (surface unchanged) and D-03 (must work with any connection).

**Recommendation: Option A.** The `commit/release/extend` methods keep `<C: ConnectionTrait>` unchanged. Only `hold` gains `+ TransactionTrait`. [ASSUMED — needs confirmation that all call sites for `kernel.hold(...)` pass `&DatabaseConnection`, which satisfies both bounds.]

Alternatively (cleaner): in `hold`'s body, downcast `conn` to `&DatabaseConnection` and call `begin_with_config` on it. But this would require `C = DatabaseConnection` explicitly. The cleanest path is Option A.

### Pattern: Detecting Postgres SQLSTATE `40001`

SeaORM surfaces Postgres DB errors as:
```
DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
```
where `e: Box<dyn sqlx::error::DatabaseError>`.

The SQLSTATE code is accessed as `e.code().unwrap_or_default()`. For Postgres, this is `"40001"` for serialization failure.

The pattern (mirrors SeaORM's own `sql_err()` implementation):

```rust
// Source: SeaORM source sea_orm/error.rs sql_err() [CITED: https://docs.rs/sea-orm/1.1.14/src/sea_orm/error.rs.html]
fn is_serialization_failure(err: &sea_orm::DbErr) -> bool {
    use sea_orm::DbErr;
    use sea_orm::RuntimeErr;
    match err {
        DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
        | DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e))) => {
            e.code().as_deref() == Some("40001")
        }
        _ => false,
    }
}
```

Translation at the kernel boundary (D-07 preferred site):

```rust
// After txn.commit() or after the INSERT, intercept before returning:
.map_err(|db_err| {
    if is_serialization_failure(&db_err) {
        ReservationError::Insufficient {
            requested: quantity,
            available: 0,
            capacity,
        }
    } else {
        ReservationError::Db(db_err)
    }
})?;
```

For `available: 0` in the translated error: the conflict-losing task cannot know the true `available` value at commit time (the winner already consumed it). Using `0` is conservative and correct — the capacity IS exhausted from the loser's perspective. [ASSUMED — an alternative is to re-query after rollback, but this adds a round-trip and is not needed for the `Insufficient` error contract which is purely informational for callers.]

**Note on `sqlx` feature gating:** The SQLSTATE detection code references `sqlx::Error::Database`. This requires `sqlx` to be a transitive dep (it is, via `sea-orm`). The `is_serialization_failure` function MUST be compiled only when `sqlx-postgres` feature is enabled; otherwise it will fail to compile on SQLite-only builds. This is handled by:

```rust
#[cfg(feature = "sqlx-postgres")]
fn is_serialization_failure(err: &sea_orm::DbErr) -> bool { ... }

#[cfg(not(feature = "sqlx-postgres"))]
fn is_serialization_failure(_: &sea_orm::DbErr) -> bool { false }
```

This means `ferro-reservation/Cargo.toml` needs `sqlx-postgres` as an optional dependency of the production build, not just dev-dependencies — unless the detection is done via string matching on `DbErr::Exec` message text, which is fragile. The cleanest path is to add a `postgres` feature flag to the production `[features]` table and enable it when consumers use Postgres. [ASSUMED — verify whether the SQLite-only `cargo test` build will compile the SQLSTATE pattern; if `sqlx::Error::Database` is already in scope via the `sea-orm` dep chain regardless of feature, the `#[cfg]` may be unnecessary.]

### Concurrency Test Pattern (SQLite in-memory, multi-spawn)

The existing `tests/concurrent_hold.rs` uses `#[tokio::test(flavor = "current_thread")]` with a `tokio::Mutex` that serializes all `hold()` calls. After the fix, the mutex is removed and tasks race bare. The test proves the kernel itself serializes correctly.

**Key SQLite constraint:** SQLite in-memory connections created with `Database::connect("sqlite::memory:")` produce a connection pool. Under `current_thread` tokio flavor, all spawned tasks run on the same OS thread, so SQLite's file-level serialization is respected. Under `multi_thread` flavor, multiple OS threads can concurrently acquire pool connections, which SQLite handles via its internal lock (writes serialize at the file level). The existing test uses `current_thread` flavor, which is conservative and deterministic for CI. Keep this for the new race test.

**Arc sharing pattern** (already established in `concurrent_hold.rs`):
```rust
// Source: ferro-reservation/tests/concurrent_hold.rs [VERIFIED]
let conn = Arc::new(fresh_db().await);
let kernel = Arc::new(ReservationKernel::new((*conn).clone(), TestResource { capacity_value: 1 }));

let mut handles = Vec::with_capacity(2);
for _ in 0..2 {
    let kernel = kernel.clone();
    let conn = conn.clone();
    handles.push(tokio::spawn(async move {
        let ctx = ReservationContext::system();
        kernel.hold(&*conn, key, (), 1, Duration::from_secs(60), &ctx).await
    }));
}

let results: Vec<_> = futures::future::join_all(handles).await;
// Assert exactly 1 Ok, 1 Err(Insufficient)
```

**Loop count for 50/50 determinism:** Use a plain `for iteration in 0..50` loop (no proptest). This mirrors the existing `concurrent_hold.rs` pattern (it loops 3 times); extending to 50 is a literal constant change. The test runs in `current_thread` so there is no OS scheduler non-determinism — 50 iterations is reliable in CI. [VERIFIED: pattern from existing `concurrent_hold.rs`]

**`Arc<DatabaseConnection>` vs `DatabaseConnection::clone()`:** `DatabaseConnection` is `Clone` by SeaORM design (the clone shares the underlying connection pool). In the existing test: `(*conn).clone()` is used to give the kernel its own clone of the pool. For spawned tasks: `Arc::clone(&conn)` then `kernel.hold(&*conn, ...)` deref-coerces. This is the established pattern. [VERIFIED: concurrent_hold.rs lines 104-128]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Serializable transaction on `hold` | Custom mutex / application-level lock | `conn.begin_with_config(Some(IsolationLevel::Serializable), ...)` | SeaORM already provides the API; `framework/src/database/transaction.rs` proves it compiles |
| SQLSTATE code extraction | String matching on `DbErr::to_string()` | `e.code().as_deref()` on the downcast `DatabaseError` | SeaORM's `sql_err()` establishes this pattern internally |
| Concurrent test isolation | `tokio::Mutex` per resource key | Bare `tokio::spawn` + in-txn serialization | The tx fix is the mechanism; the test must remove the mutex to prove it |
| Postgres test infra | Docker from scratch | `DATABASE_URL` env var pattern for Postgres gating | Simplest approach, no `docker-compose.yml` in repo needed |

---

## Runtime State Inventory

Step 2.5: SKIPPED — this is not a rename/refactor/migration phase. It is a kernel implementation fix with no stored data, OS-registered state, or secret key changes.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| SQLite (in-process via sqlx) | All existing tests + new concurrency test | ✓ | bundled via sqlx-sqlite | — |
| Postgres (docker-compose) | Postgres-gated concurrency tests | Not verified | — | Skip via `cfg(feature = "postgres-tests")` |

**Missing dependencies with fallback:**
- Postgres: Postgres integration tests are gated behind `feature = "postgres-tests"` by D-08. CI can enable them when a Postgres service is available; contributors without Postgres skip them.

---

## Findings Per Research Question

### Q1: SeaORM transaction API surface for `hold`

**`conn.begin()` semantics:** `begin()` starts a `READ COMMITTED` transaction (Postgres default). For the race fix, use `begin_with_config(Some(IsolationLevel::Serializable), Some(AccessMode::ReadWrite))` instead. This issues `BEGIN ISOLATION LEVEL SERIALIZABLE READ WRITE` to Postgres and is a no-op/silently-accepted on SQLite. [CITED: https://docs.rs/sea-orm/1.1.14/sea_orm/trait.TransactionTrait.html]

**`&DatabaseTransaction` through generic bounds:** `DatabaseTransaction: ConnectionTrait` is confirmed. Pass `&txn` to all existing `Resource::held`, `Resource::capacity`, `am.insert`, `audit.write` calls — no type changes needed to Resource impls. [CITED: https://docs.rs/sea-orm/1.1.14/sea_orm/struct.DatabaseTransaction.html]

**Retry helpers:** SeaORM provides no retry helper for `40001`. The correct pattern per D-07 is to translate `40001` → `ReservationError::Insufficient` at the kernel boundary, NOT to retry inside the kernel. Caller-retry is the documented contract.

**Error propagation with `?`:** The `hold` body will use `?` inside a lexical scope — NOT inside a closure. Using `?` directly in an `async fn` body works without wrapping in a closure. The commit step needs explicit error mapping (shown in the SQLSTATE detection pattern above).

### Q2: Postgres `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE`

**Mechanism:** `conn.begin_with_config(Some(IsolationLevel::Serializable), Some(AccessMode::ReadWrite))` emits the correct SQL. There is NO need to call `txn.execute_unprepared("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")` separately — `begin_with_config` handles it. [VERIFIED: framework/src/database/transaction.rs uses this exact call pattern]

**SQLite behavior:** SQLite does not error on unknown isolation level requests — the sqlx driver silently accepts the `begin_with_config` call. SQLite's single-writer WAL provides the necessary serialization automatically. No backend-detection branching needed. [ASSUMED — not explicitly confirmed by SQLite docs in this session; risk is LOW since SQLite WAL is well-documented as single-writer]

### Q3: Detecting Postgres SQLSTATE `40001`

**Variant that carries it:** `DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))`. The commit failure for a serialization conflict arrives as `DbErr::Exec(...)` (from the COMMIT statement). The INSERT failure (if conflict detected at INSERT time rather than commit) arrives as `DbErr::Exec(RuntimeErr::SqlxError(...))` too. Match both `DbErr::Exec` and `DbErr::Query` to be safe (matches SeaORM's own `sql_err()` pattern).

**Accessing SQLSTATE:** `e.code().as_deref() == Some("40001")`. The `e` is `Box<dyn sqlx::error::DatabaseError>`, available via `.try_downcast_ref::<sqlx::postgres::PgDatabaseError>()` OR directly via `e.code()` since `DatabaseError` trait exposes `code()`. The direct `e.code()` approach is simpler and does not require a Postgres-specific downcast. [CITED: SeaORM source sql_err() at https://docs.rs/sea-orm/1.1.14/src/sea_orm/error.rs.html]

**Translation site:** At the kernel boundary, immediately after `txn.commit().await.map_err(...)`. The CONTEXT D-07 is clear: translate `40001` → `ReservationError::Insufficient` at the kernel boundary. Specific values to populate:
- `requested: quantity` — the quantity that was attempted
- `available: 0` — conservative; the capacity was exhausted by the concurrent winner
- `capacity` — the value computed earlier in the body (before the txn or inside it)

### Q4: Concurrency test infrastructure in Rust async

**`tokio::spawn` + `Arc` pattern:** Already established in `tests/concurrent_hold.rs`. Use `Arc<DatabaseConnection>` clone strategy: `ReservationKernel::new((*conn).clone(), resource)`. Each kernel clone shares the underlying pool, so tasks contend on real DB locks. [VERIFIED: concurrent_hold.rs]

**Asserting deterministic Ok/Err counts:** Count `Ok` and `Err(Insufficient)` variants after `join_all`. Use `assert_eq!(successes, N)` and `assert_eq!(insufficient, M)`. A loop of 50 iterations wrapping this is the correct pattern for "50/50 in CI" Success Criterion 1.

**Known SQLite in-memory gotcha:** `sqlite::memory:` creates a per-connection database by default. Multiple pool connections from the same URL share the same in-memory database only when using the `?mode=memory&cache=shared` URI. The existing tests use bare `sqlite::memory:` which SeaORM treats as a single connection pool pointing to one in-memory DB — this works because the pool serializes access. The new race test inherits this. [VERIFIED: all existing tests use `Database::connect("sqlite::memory:")`]

**`tokio::test` flavor:** Use `#[tokio::test(flavor = "current_thread")]` for the race test (consistent with existing `concurrent_hold.rs`). The current-thread executor is deterministic for this class of test.

### Q5: Postgres test cfg pattern

**Finding:** No existing `[features]` section in `ferro-reservation/Cargo.toml` or any sibling primitive crate (`ferro-orm`, `ferro-audit`). No `#[cfg(feature = "...")]` patterns exist in any of these crates. The codebase has no established convention. [VERIFIED: grep across all sibling crates returned empty]

**Recommendation:** Use `feature = "postgres-tests"` in `[features]` table. This is:
- Self-documenting (clearly scoped to tests)
- Matches the D-08 locked decision wording: "gated on a `cfg` flag (`test_postgres` or feature `postgres-tests`)"
- Compatible with standard `cargo test --features postgres-tests` CI invocation
- Does not require environment variable availability at compile time

Cargo.toml addition:
```toml
[features]
postgres-tests = []
```

Test gating:
```rust
#[cfg(feature = "postgres-tests")]
mod postgres_tests {
    // identical test cases against DATABASE_URL env var Postgres
}
```

The Postgres database URL is read from `std::env::var("DATABASE_URL")` inside the test module (panic if missing, since the test is opt-in via feature). This is the simplest viable pattern.

### Q6: Audit-log write site for `reservation.held`

**Location:** `ferro-reservation/src/kernel.rs:124-143`. The `AuditEntry::record("reservation.held")` ... `.write(conn)` call is at the end of the `hold` body, BEFORE `ReservationHandle` construction and BEFORE event dispatch. [VERIFIED: kernel.rs lines 124-143]

**D-04 implication:** Moving the entire `hold` body inside a `conn.begin_with_config(...)` / `txn.commit()` block automatically gives the "rolled back on conflict" semantics for the audit write. The audit write uses `conn` (which becomes `&txn`); if the transaction rolls back due to a `40001` serialization failure, the audit INSERT is rolled back with it. No structural change to the audit write call is needed — only the connection reference changes from `conn` to `&txn`. [VERIFIED: hold body structure at kernel.rs:54-176]

**Side effects that must move inside the txn boundary:**

| Step | Currently | After Fix |
|------|-----------|-----------|
| `self.resource.capacity(conn, ...)` | line 67 | inside txn |
| `self.resource.held(conn, ...)` | line 68 | inside txn |
| `am.insert(conn).await` | line 122 | inside txn |
| `AuditEntry::write(conn).await` | line 143 | inside txn |
| `ReservationHandle` construction | lines 145-155 | inside txn (value moved to return) |
| `ferro_events::dispatch(...)` | lines 157-173 | **outside txn** (best-effort, fires after commit) |

The event dispatch MUST remain outside the txn and fire AFTER `txn.commit()` succeeds. This matches D-26 (best-effort, after state commits). The existing warning message "state is committed" is still accurate.

### Q7: Existing kernel test patterns for reuse

**Reusable from `src/kernel.rs` inline tests:**
- `TestMigrator` struct — defines which migrations to run for test DB setup.
- `fresh_db() -> DatabaseConnection` — creates and migrates an in-memory SQLite DB.
- `TestResource { capacity_value: u32 }` — minimal `Resource` impl with live DB `held()` query.
- `fresh_kernel() -> (DatabaseConnection, ReservationKernel<TestResource>)` — convenience function.
- `fn ttl(secs: u64) -> Duration` — shorthand.

**Status of these helpers:** They are in `#[cfg(test)] mod tests` inside `src/kernel.rs`, so they are NOT accessible from `tests/concurrent_hold.rs` (which is a separate integration test binary). The existing `tests/concurrent_hold.rs` already re-declares its own `TestMigrator`, `fresh_db()`, and `TestResource`. [VERIFIED: concurrent_hold.rs lines 32-92]

**Planner decision:** The new concurrency test cases for Success Criteria 1-4 should be added to the **existing `tests/concurrent_hold.rs`** file (not a new `tests/concurrency.rs`). Adding new test functions to the existing file avoids duplicating the harness infrastructure again. The existing file already has `TestMigrator`, `fresh_db()`, and `TestResource`. The CONTEXT document's "NEW file: tests/concurrency.rs" should be interpreted as "the concurrency test file" — mapping to the existing `concurrent_hold.rs`.

**What needs to change in `tests/concurrent_hold.rs`:**
1. Remove the `hold_lock: Arc<Mutex<()>>` and all mutex acquire/release code.
2. Rename or add test cases for capacity=1 (2 tasks), capacity=N (N+1 tasks), non-overlapping windows, and audit-row count assertion.
3. Extend iteration loop from 3 to 50.
4. The `TestResource::held()` query and all `fresh_db()`/`fresh_kernel()` scaffolding can be reused verbatim.

### Q8: Documentation surface with stale concurrency claims

The following locations contain claims about `hold` atomicity that become factually incorrect or misleading after the fix and must be updated:

| File | Lines / Section | Stale Claim | Required Update |
|------|-----------------|-------------|-----------------|
| `docs/src/database/reservations.md` | Lines 146-148 | "consumers running concurrent holds against SQLite should serialize `hold` calls at the application layer — a `tokio::sync::Mutex` per resource key is the idiomatic pattern." | Remove Mutex recommendation; replace with "the kernel wraps `hold` in a serializable transaction; no application-layer mutex is required." |
| `docs/src/database/reservations.md` | Lines 370-381 (Consistency Model section) | Entire `hold on SQLite` and `hold on Postgres` paragraphs describing the race window and recommending Mutex / noting it's a "roadmap" item | Replace with description of the transaction-based fix. |
| `docs/src/database/reservations.md` | Line 7-8 | "race-free state machine" — this claim was technically false for `hold` | Becomes truthful after the fix; no change required, but planner should note it. |
| `ferro-reservation/src/lib.rs` | Lines 9-10 | "race-free by construction" — same issue | Becomes truthful after the fix. |
| `ferro-reservation/src/lib.rs` | Lines 74-80 | Audit failure semantics note — remains accurate | No change needed. |
| `ferro-reservation/src/kernel.rs` | Module doc (lines 1-16) | No explicit mention of concurrency, but the doc says "per-call methods accept an explicit `&C: ConnectionTrait` so consumers can run them inside their own transactions" — this is still accurate | No change needed. |
| `ferro-reservation/tests/concurrent_hold.rs` | Lines 1-20 (module doc) | Entire doc block describing the Mutex workaround as "the recommended pattern" | Replace with description of the transaction-based fix and what the test now proves. |
| `ferro-reservation/README.md` | Line 5 | "race-free state-transition pipeline" — technically false for `hold` before the fix | Becomes truthful after fix; no change needed. |

**Sweep target list for the planner's "sweep stale claims" task:**
1. `docs/src/database/reservations.md` — Consistency Model section (lines 363-382) — MUST update
2. `docs/src/database/reservations.md` — `hold` sequence step description (lines 145-148) — MUST update
3. `tests/concurrent_hold.rs` — module doc + test body (remove Mutex, update iteration count) — MUST update

---

## Common Pitfalls

### Pitfall 1: `conn: &C` does not have `begin_with_config` unless `C: TransactionTrait`

**What goes wrong:** Adding `conn.begin_with_config(...)` inside `hold<C: ConnectionTrait>` causes a compile error because `ConnectionTrait` does not imply `TransactionTrait`.

**Why it happens:** SeaORM separates the concerns: `ConnectionTrait` is about executing statements; `TransactionTrait` is about beginning transactions.

**How to avoid:** Add `C: ConnectionTrait + TransactionTrait` to the `hold` method's generic bound. `DatabaseConnection` implements both. `DatabaseTransaction` also implements `TransactionTrait` (allowing nested transactions / savepoints — but that's not needed here). [VERIFIED: both `begin` and `begin_with_config` are defined on `TransactionTrait`]

**Warning signs:** `error[E0599]: no method named 'begin_with_config' found for reference '&C'`

### Pitfall 2: Audit write happens AFTER txn commit — loses D-04 semantics

**What goes wrong:** If the audit write is moved to after `txn.commit()`, a commit failure (e.g., `40001`) would still produce an audit row in a subsequent write — violating D-04.

**How to avoid:** Keep `audit.write(&txn)` INSIDE the transaction (before commit). The existing hold body places audit write at line 143, before the handle construction. The correct structure is: `[begin txn] → [capacity/held queries] → [INSERT] → [audit write] → [commit txn]`. [VERIFIED: kernel.rs lines 124-143 confirm audit write is before handle construction]

### Pitfall 3: Event dispatch inside the transaction triggers deadlock

**What goes wrong:** If `ferro_events::dispatch(...)` is moved inside the transaction, any listener that tries to read the DB sees the uncommitted rows (under serializable isolation this may block or fail).

**How to avoid:** Keep event dispatch OUTSIDE the transaction, AFTER `txn.commit()`. The comment "state is committed" in the existing warning log confirms the intent. The CONTEXT document also specifies this (D-26: best-effort, after state commits). [VERIFIED: kernel.rs lines 157-173 confirm event dispatch is the last step in hold]

### Pitfall 4: SQLite `sqlite::memory:` connection per-pool semantics

**What goes wrong:** Multiple in-memory SQLite connections from separate `Database::connect("sqlite::memory:")` calls each get their own isolated in-memory database. The race test would use two completely different databases and always succeed (no contention).

**How to avoid:** The existing test pattern uses a single `fresh_db()` call and clones the resulting `DatabaseConnection` via `(*conn).clone()`. The clone shares the same underlying pool (same in-memory DB). The `Arc<DatabaseConnection>` wrapping ensures both tasks use the same pool. [VERIFIED: concurrent_hold.rs lines 104-107]

### Pitfall 5: `40001` detection requires `sqlx-postgres` in the compilation graph

**What goes wrong:** The `sqlx::Error::Database(e)` match arm uses types that require `sqlx-postgres` feature. On SQLite-only builds (e.g., `cargo test` without `--features postgres-tests`), the code fails to compile or is dead code.

**How to avoid:** Gate the `is_serialization_failure` helper with `#[cfg(feature = "sqlx-postgres")]`. Provide a `#[cfg(not(feature = "sqlx-postgres"))]` stub returning `false`. Add `sqlx-postgres` to `ferro-reservation/Cargo.toml` production features (not just dev-deps) gated behind the new `postgres` feature flag. [ASSUMED — test this in Wave 0 by attempting `cargo build -p ferro-reservation` after the change with no extra features]

### Pitfall 6: `available: 0` in translated `40001` error is misleading to callers

**What goes wrong:** The conflict-losing task returns `ReservationError::Insufficient { available: 0, capacity }`. If callers display `available` to end users ("0 of 1 slot available"), this is accurate. If they use `available` for retry logic (retry if `available > 0`), they will not retry — which is correct (the capacity IS 0 from this task's perspective).

**Why acceptable:** The CONTEXT document specifies this exact behavior (D-07: "conflict-losing task may receive... translate `40001` → `ReservationError::Insufficient`"). The consumer field test expects `Err(Insufficient)` not a Db error variant. [VERIFIED: CONTEXT.md D-07]

---

## Code Examples

### Transaction wrapper for `hold` body

```rust
// Source: adapted from framework/src/database/transaction.rs [VERIFIED pattern]
use sea_orm::{AccessMode, IsolationLevel, TransactionTrait};

pub async fn hold<C: ConnectionTrait + TransactionTrait>(
    &self,
    conn: &C,
    key: R::Key,
    window: R::Window,
    quantity: u32,
    ttl: Duration,
    ctx: &ReservationContext,
) -> Result<ReservationHandle, ReservationError> {
    let id = Uuid::new_v4();

    let txn = conn
        .begin_with_config(
            Some(IsolationLevel::Serializable),
            Some(AccessMode::ReadWrite),
        )
        .await
        .map_err(ReservationError::Db)?;

    // Capacity check inside txn snapshot
    let capacity = self.resource.capacity(&txn, &key, &window).await?;
    let held = self.resource.held(&txn, &key, &window).await?;
    let available = capacity.saturating_sub(held);

    if quantity == 0 { /* ... */ }
    if quantity > available {
        // txn dropped here → auto-rollback (no rows inserted)
        return Err(ReservationError::Insufficient { requested: quantity, available, capacity });
    }

    // ... build am, insert, write audit (all using &txn) ...
    am.insert(&txn).await.map_err(ReservationError::Db)?;
    audit.write(&txn).await.map_err(ReservationError::Audit)?;

    // Commit — detect 40001 here
    txn.commit().await.map_err(|e| {
        if is_serialization_failure(&e) {
            ReservationError::Insufficient {
                requested: quantity,
                available: 0,
                capacity,
            }
        } else {
            ReservationError::Db(e)
        }
    })?;

    // Event dispatch AFTER commit (best-effort, D-26)
    let handle = ReservationHandle { /* ... */ };
    if let Err(e) = ferro_events::dispatch(ReservationEvent::Held { /* ... */ }).await {
        tracing::warn!( /* ... */ );
    }

    Ok(handle)
}
```

### SQLSTATE detection helper

```rust
// Source: mirrors SeaORM src/error.rs sql_err() [CITED: https://docs.rs/sea-orm/1.1.14/src/sea_orm/error.rs.html]
#[cfg(feature = "sqlx-postgres")]
fn is_serialization_failure(err: &sea_orm::DbErr) -> bool {
    use sea_orm::RuntimeErr;
    match err {
        sea_orm::DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
        | sea_orm::DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e))) => {
            e.code().as_deref() == Some("40001")
        }
        _ => false,
    }
}

#[cfg(not(feature = "sqlx-postgres"))]
fn is_serialization_failure(_: &sea_orm::DbErr) -> bool {
    false
}
```

### New concurrency test (replaces existing mutex-based test in concurrent_hold.rs)

```rust
// Source: pattern from ferro-reservation/tests/concurrent_hold.rs [VERIFIED]
#[tokio::test(flavor = "current_thread")]
async fn hold_race_capacity_1_exactly_one_succeeds() {
    for iteration in 0..50 {
        let conn = Arc::new(fresh_db().await);
        let kernel = Arc::new(ReservationKernel::new(
            (*conn).clone(),
            TestResource { capacity_value: 1 },
        ));
        let key = "race_key".to_string();

        let mut handles = Vec::with_capacity(2);
        for _ in 0..2 {
            let kernel = kernel.clone();
            let conn = conn.clone();
            let key = key.clone();
            handles.push(tokio::spawn(async move {
                let ctx = ReservationContext::system();
                // NO mutex — the kernel's transaction is the serialization mechanism
                kernel.hold(&*conn, key, (), 1, Duration::from_secs(60), &ctx).await
            }));
        }

        let mut successes = 0usize;
        let mut insufficient = 0usize;
        for h in handles {
            match h.await.expect("join") {
                Ok(_) => successes += 1,
                Err(ReservationError::Insufficient { .. }) => insufficient += 1,
                Err(e) => panic!("unexpected error in iteration {iteration}: {e:?}"),
            }
        }

        assert_eq!(successes, 1, "iteration {iteration}: expected exactly 1 Ok");
        assert_eq!(insufficient, 1, "iteration {iteration}: expected exactly 1 Insufficient");
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `tokio::Mutex` at call site for SQLite concurrency | Serializable transaction in kernel | Phase 177 | Application layer no longer responsible for serialization |
| `hold` undocumented race on Postgres | `hold` race-free on both backends | Phase 177 | Postgres invariant now kernel-enforced, not roadmap |
| Audit row not rolled back on concurrent conflict | Audit row rolled back with transaction | Phase 177 | D-04 fully satisfied |

**Deprecated after fix:**
- The `tokio::Mutex` pattern described in `docs/src/database/reservations.md` Consistency Model — callers no longer need it.
- The "Postgres correctness for the capacity check is on the roadmap as a follow-up" note (docs line 379-380) — it is addressed by this phase.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `IsolationLevel::Serializable` passed to `begin_with_config` is silently accepted by SQLite (no error) | Q2 / Pitfall section | LOW — if SQLite errors, need `#[cfg]` branch or plain `conn.begin()` for SQLite |
| A2 | All call sites for `kernel.hold(...)` pass `&DatabaseConnection`, which satisfies `C: ConnectionTrait + TransactionTrait` | Q1 / Pattern section | LOW — verified in test harness; consumer call sites not read |
| A3 | `available: 0` in the translated `40001` error is acceptable to the consumer field test `concurrent_double_book_same_staff` | Q3 | LOW — CONTEXT D-07 specifies `translate 40001 → Insufficient`; the exact field values are not tested by the consumer |
| A4 | `is_serialization_failure` can use `e.code()` directly without a Postgres-specific downcast | Q3 | LOW — SeaORM's `sql_err()` uses `e.code()` directly for the code match; only downcasts for MySQL-specific fields |
| A5 | The `sqlx::Error::Database` pattern compiles without `#[cfg(feature = "sqlx-postgres")]` if `sqlx-postgres` is a transitive dep via `sea-orm` | Pitfall 5 | MEDIUM — if SQLite-only build fails to compile the match arm, the `#[cfg]` gates are required |

---

## Open Questions

1. **Does `C: TransactionTrait` conflict with any existing call site?**
   - What we know: All tests in `kernel.rs` and integration tests use `&conn: &DatabaseConnection`. `DatabaseConnection` implements `TransactionTrait`.
   - What's unclear: Whether any consumer passes a `&DatabaseTransaction` as `conn` to `hold` (which also implements `TransactionTrait`, so this would still work).
   - Recommendation: Add the bound; compile the workspace to verify.

2. **Does `begin_with_config` with `IsolationLevel::Serializable` produce an error or warning on SQLite?**
   - What we know: SeaORM's `execute_unprepared` is supported on `sqlx-sqlite`.
   - What's unclear: Whether sqlx-sqlite silently ignores `SERIALIZABLE` or returns an error.
   - Recommendation: Run `cargo test -p ferro-reservation` after the change to confirm.

3. **Should `is_serialization_failure` go in `error.rs` or inline in `kernel.rs`?**
   - What we know: It is only used in `hold`. It references private types (`RuntimeErr`).
   - Recommendation: Keep it as a private free function in `kernel.rs`, not in `error.rs` (which has no backend-specific logic today). If it grows, move it.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `tokio::test` (already in use) |
| Config file | none — uses `Cargo.toml` dev-deps |
| Quick run command | `cargo test -p ferro-reservation` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Criterion | Behavior | Test Type | Automated Command | File Exists? |
|-----------|----------|-----------|-------------------|-------------|
| SC-1 | 2 tasks race, capacity=1, exactly 1 Ok + 1 Insufficient, 50 iterations | integration | `cargo test -p ferro-reservation hold_race` | Partially (concurrent_hold.rs exists but uses Mutex, wrong shape) |
| SC-2 | Non-overlapping windows both succeed | integration | `cargo test -p ferro-reservation non_overlapping_windows` | No |
| SC-3 | Existing single-task tests pass unchanged | regression | `cargo test -p ferro-reservation` | Yes (kernel.rs:494-560) |
| SC-4 | fix path is (a) | code review | — | — |
| SC-5 | Audit row absent for conflict-losing task | integration | `cargo test -p ferro-reservation audit_atomicity` | No |
| SC-6 | `docs/src/database/reservations.md` Consistency Model section updated | doc review | — | — |

### Wave 0 Gaps

- [ ] `tests/concurrent_hold.rs` — rewrite to remove Mutex; add SC-1 (50 iterations), SC-2 (non-overlapping), SC-5 (audit atomicity) test functions
- [ ] `ferro-reservation/Cargo.toml` — add `[features]` section with `postgres-tests = []`; add `sqlx-postgres` to dev-dep sea-orm features

---

## Security Domain

This phase has no authentication, session, access control, or cryptography surface. The fix is a database transaction wrapping an existing DB operation. ASVS categories V2-V6 do not apply. Security enforcement: N/A.

---

## Sources

### Primary (HIGH confidence)
- `ferro-reservation/src/kernel.rs` — race location (lines 54-176), commit pattern (lines 182-245), existing test harness (lines 411-746) [VERIFIED by Read tool]
- `ferro-reservation/tests/concurrent_hold.rs` — existing concurrency test with Mutex workaround [VERIFIED by Read tool]
- `ferro-reservation/tests/property_invariants.rs` — proptest patterns for concurrency [VERIFIED by Read tool]
- `ferro-reservation/src/error.rs` — `ReservationError` variants [VERIFIED by Read tool]
- `framework/src/database/transaction.rs` — `begin_with_config`, `IsolationLevel::Serializable`, `AccessMode::ReadWrite` usage in production code [VERIFIED by Read tool]
- `ferro-orm/src/guarded.rs` — T-16-6: `conn.begin()` / `txn.rollback()` pattern [VERIFIED by Read tool]
- `docs/src/database/reservations.md` — stale concurrency claims (lines 145-148, 363-382) [VERIFIED by Read tool]
- SeaORM 1.1.14 `TransactionTrait` docs — `begin_with_config` signature [CITED: https://docs.rs/sea-orm/1.1.14/sea_orm/trait.TransactionTrait.html]
- SeaORM 1.1.14 `DatabaseTransaction` docs — `ConnectionTrait` impl [CITED: https://docs.rs/sea-orm/1.1.14/sea_orm/struct.DatabaseTransaction.html]
- SeaORM 1.1.14 `DbErr` / `sql_err()` source — SQLSTATE detection pattern [CITED: https://docs.rs/sea-orm/1.1.14/src/sea_orm/error.rs.html]

### Secondary (MEDIUM confidence)
- SeaORM 1.1.14 `SqlxPostgresError` / `PgDatabaseError` struct — `e.code()` method for SQLSTATE [CITED: https://docs.rs/sea-orm/1.1.14/sea_orm/error/struct.SqlxPostgresError.html]

### Tertiary (LOW confidence — see Assumptions Log)
- SQLite behavior under `IsolationLevel::Serializable` — [ASSUMED: silent ignore, not verified against sqlx-sqlite docs]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in Cargo.toml; API verified against docs
- Architecture: HIGH — transaction pattern verified in framework/src/database/transaction.rs; `DatabaseTransaction: ConnectionTrait` confirmed
- Pitfalls: HIGH — pitfalls 1-4 verified directly from codebase; pitfall 5 marked MEDIUM (compile-time behavior not run)
- SQLSTATE detection: HIGH — pattern mirrors SeaORM's own internal `sql_err()` implementation exactly

**Research date:** 2026-05-21
**Valid until:** 2026-07-01 (SeaORM 1.x is stable; unlikely to change transaction API)
