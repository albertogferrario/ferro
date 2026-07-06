# Phase 177: reservation-kernel-hold-atomicity — Pattern Map

**Mapped:** 2026-05-21
**Files analyzed:** 5 (2 modify, 1 rewrite, 1 new, 1 doc update)
**Analogs found:** 5 / 5

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-reservation/src/kernel.rs` | service/kernel | CRUD + transaction | `framework/src/database/transaction.rs` (txn pattern) + `ferro-reservation/src/kernel.rs:182-245` (commit method) | exact |
| `ferro-reservation/tests/concurrent_hold.rs` | test | event-driven (concurrency) | same file (current state — rewrite from within) | self-analog |
| `ferro-reservation/tests/concurrent_hold_postgres.rs` | test (new) | event-driven (concurrency) | `ferro-reservation/tests/concurrent_hold.rs` (identical structure, cfg-gated) | role-match |
| `ferro-reservation/Cargo.toml` | config | — | no sibling crate has `[features]` section — no codebase analog exists | no analog |
| `docs/src/database/reservations.md` | doc | — | same file (surrounding accurate sections match tone) | self-analog |

---

## Pattern Assignments

### `ferro-reservation/src/kernel.rs` — MODIFY `hold` body (lines 54-176)

**Primary analog:** `framework/src/database/transaction.rs`
**Secondary analog:** `ferro-reservation/src/kernel.rs:182-245` (the `commit` method — structural reference for the txn wrapper shape)

#### Imports pattern

Add to the existing `use sea_orm::{...}` import block at line 15 of `kernel.rs`:

```rust
// Current line 15 (kernel.rs):
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection};

// After fix — add TransactionTrait:
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, TransactionTrait};
// Also add at top of file:
use sea_orm::{AccessMode, IsolationLevel};
```

Source: `framework/src/database/transaction.rs` lines 25-27:
```rust
use sea_orm::{
    AccessMode, DatabaseConnection, DatabaseTransaction, IsolationLevel, TransactionTrait,
};
```

#### Method bound pattern

`hold` gains `+ TransactionTrait` on the generic bound. The existing bound lives at `kernel.rs:54`:

```rust
// Before (kernel.rs line 54):
pub async fn hold<C: ConnectionTrait>(

// After:
pub async fn hold<C: ConnectionTrait + TransactionTrait>(
```

`DatabaseConnection` implements both. This is the only call-site-visible change to the public API — the method signature is otherwise byte-identical.

#### Transaction begin pattern (lines 99-112 of transaction.rs)

Exact pattern to replicate from `framework/src/database/transaction.rs:transaction_with` (lines 99-123):

```rust
// framework/src/database/transaction.rs lines 110-112 — the begin_with_config call:
let txn = db
    .inner()
    .begin_with_config(Some(isolation_level), Some(AccessMode::ReadWrite))
    .await
    .map_err(|e| FrameworkError::database(format!("Failed to begin transaction: {e}")))?;
```

In `kernel.rs::hold`, adapt as (the `conn` is already available, no `.inner()` needed):

```rust
let txn = conn
    .begin_with_config(
        Some(IsolationLevel::Serializable),
        Some(AccessMode::ReadWrite),
    )
    .await
    .map_err(ReservationError::Db)?;
```

#### Commit + 40001 translation pattern

No direct analog in codebase — pattern sourced from SeaORM internals per RESEARCH.md. The commit step replaces the simple `txn.commit().await?` with error mapping:

```rust
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
```

#### SQLSTATE helper — placement and cfg-gating

Place as a private free function in `kernel.rs` (not in `error.rs` — `error.rs` has no backend-specific logic). Dual `#[cfg]` stubs to prevent compile failure on SQLite-only builds:

```rust
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

Source: mirrors SeaORM `src/error.rs sql_err()` pattern (RESEARCH.md Q3).

#### Event dispatch position (must stay OUTSIDE the transaction)

The event dispatch block at `kernel.rs:157-173` must remain after `txn.commit()`. This is confirmed by the `commit` method's identical structure at lines 229-244 — event dispatch is always the last step, with the explicit comment `"state is committed"`. Do not move this block inside the txn boundary.

```rust
// kernel.rs lines 229-244 — the structural reference for event-after-commit:
if let Err(e) = ferro_events::dispatch(ReservationEvent::Committed { ... }).await {
    tracing::warn!(
        reservation_id = %handle.id,
        error = %e,
        "event dispatch failed after reservation.committed — state is committed"
    );
}
```

#### Rollback-on-drop semantics (no explicit rollback needed)

Source: `ferro-orm/src/guarded.rs` test T-16-6 (lines 315-347):
```rust
// guarded.rs lines 320-339 — confirms drop-based rollback is the pattern:
let txn = conn.begin().await.expect("begin transaction");
// ... work against &txn ...
txn.rollback().await.expect("rollback");
```

When `hold` returns early (`Err(Insufficient)`) before `txn.commit()`, the `DatabaseTransaction` is dropped and auto-rolls back. No explicit `txn.rollback()` call needed for the early-exit paths. This matches the `framework/src/database/transaction.rs` `transaction_with` function at lines 114-122, where the `Err` arm returns without calling rollback.

---

### `ferro-reservation/tests/concurrent_hold.rs` — REWRITE

**Analog:** same file, current state (the harness infrastructure is reused; the test body is rewritten)

#### What to keep verbatim

Module-level imports (lines 21-31 of current file):
```rust
use async_trait::async_trait;
use ferro_reservation::{ReservationContext, ReservationError, ReservationKernel, Resource};
use sea_orm::{
    ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter,
};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;  // REMOVE after rewrite — keep only if Postgres section needs it
```

`TestMigrator` struct (lines 33-43 of current file) — reuse verbatim:
```rust
struct TestMigrator;

#[async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(ferro_audit::CreateAuditLogTable),
            Box::new(ferro_reservation::CreateReservationsTable),
        ]
    }
}
```

`fresh_db()` (lines 44-48 of current file) — reuse verbatim:
```rust
async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}
```

`TestResource` impl (lines 51-92 of current file) — reuse verbatim. The `held()` query already uses `&C: ConnectionTrait`, so it works with `&txn` inside the kernel's new transaction.

#### What to replace

Remove: `hold_lock: Arc<Mutex<()>>` and all mutex acquire/release code (lines 110, 119, 123-128 of current file).

Remove: current test function `concurrent_hold_against_capacity_5_admits_exactly_5`.

Replace with four new test functions:

**SC-1: Race-to-capacity, capacity=1 (primary regression test)**

Pattern source: current `concurrent_hold.rs` lines 101-169 (loop + spawn + join_all structure), minus the mutex:

```rust
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
                // No mutex — the kernel's serializable transaction is the mechanism
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

**SC-1 extended: Race-to-capacity, capacity=N (confirms capacity > 1 is not falsely rejected)**

Same loop/spawn shape, N=5, 6 tasks (mirrors the current test's 20-task shape):

```rust
#[tokio::test(flavor = "current_thread")]
async fn hold_race_capacity_n_admits_exactly_n() {
    const CAPACITY: u32 = 5;
    const TASKS: usize = 6;

    for iteration in 0..50 {
        let conn = Arc::new(fresh_db().await);
        let kernel = Arc::new(ReservationKernel::new(
            (*conn).clone(),
            TestResource { capacity_value: CAPACITY },
        ));
        let key = "race_key_n".to_string();

        let mut handles = Vec::with_capacity(TASKS);
        for _ in 0..TASKS {
            let kernel = kernel.clone();
            let conn = conn.clone();
            let key = key.clone();
            handles.push(tokio::spawn(async move {
                let ctx = ReservationContext::system();
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

        assert_eq!(successes, CAPACITY as usize,
            "iteration {iteration}: expected exactly {CAPACITY} Ok");
        assert_eq!(insufficient, TASKS - CAPACITY as usize,
            "iteration {iteration}: expected exactly {} Insufficient", TASKS - CAPACITY as usize);
    }
}
```

**SC-2: Non-overlapping windows (sequential, boundary check)**

Single-task test — no race. Uses `String` window type to distinguish windows. Note: `TestResource` uses `type Window = ()` — planner may need to add a second `TestResource` variant with `type Window = String` for this test case, or use a new `&str`-keyed key instead of window. The easiest approach: use different `key` values to simulate non-overlapping resources (avoids changing `TestResource`):

```rust
#[tokio::test(flavor = "current_thread")]
async fn hold_non_overlapping_keys_both_succeed() {
    let conn = Arc::new(fresh_db().await);
    let kernel = Arc::new(ReservationKernel::new(
        (*conn).clone(),
        TestResource { capacity_value: 1 },
    ));
    let ctx = ReservationContext::system();

    // Two holds on DIFFERENT keys — should both succeed
    kernel.hold(&*conn, "key_a".to_string(), (), 1, Duration::from_secs(60), &ctx)
        .await
        .expect("key_a hold must succeed");
    kernel.hold(&*conn, "key_b".to_string(), (), 1, Duration::from_secs(60), &ctx)
        .await
        .expect("key_b hold must succeed");
}
```

**SC-5: Audit-log atomicity (conflict-losing task's audit row rolled back)**

Pattern source: `kernel.rs` test `hold_emits_audit_entry` (lines 724-745) — the `ferro_audit::history_for_target` query pattern:

```rust
#[tokio::test(flavor = "current_thread")]
async fn hold_race_audit_atomicity_exactly_n_audit_rows() {
    const CAPACITY: u32 = 1;

    let conn = Arc::new(fresh_db().await);
    let kernel = Arc::new(ReservationKernel::new(
        (*conn).clone(),
        TestResource { capacity_value: CAPACITY },
    ));
    let key = "audit_race_key".to_string();

    let mut handles = Vec::with_capacity(2);
    for _ in 0..2 {
        let kernel = kernel.clone();
        let conn = conn.clone();
        let key = key.clone();
        handles.push(tokio::spawn(async move {
            let ctx = ReservationContext::system();
            kernel.hold(&*conn, key, (), 1, Duration::from_secs(60), &ctx).await
        }));
    }

    let mut successful_ids = Vec::new();
    for h in handles {
        match h.await.expect("join") {
            Ok(handle) => successful_ids.push(handle.id),
            Err(ReservationError::Insufficient { .. }) => {}
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }

    // Exactly 1 successful hold → exactly 1 audit row for "reservation.held"
    assert_eq!(successful_ids.len(), CAPACITY as usize);
    for id in &successful_ids {
        let history = ferro_audit::history_for_target(
            &ferro_audit::AuditTarget::new("reservation", id.to_string()),
            &*conn,
        )
        .await
        .expect("audit query");
        assert_eq!(history.len(), 1, "expected exactly 1 audit entry for reservation {id}");
        assert_eq!(history[0].action, "reservation.held");
    }

    // Verify NO audit rows exist for any other reservation id (conflict-loser row rolled back)
    // The total audit count for "reservation.held" action must equal the number of successes
    // This is verified indirectly: only 1 hold succeeded, so only 1 reservation id exists
    use ferro_reservation::ReservationEntity;
    use sea_orm::EntityTrait;
    let all_reservations = ReservationEntity::find()
        .all(&*conn)
        .await
        .expect("count all reservations");
    assert_eq!(
        all_reservations.len(),
        CAPACITY as usize,
        "DB must contain exactly {CAPACITY} reservation rows — conflict-loser row rolled back"
    );
}
```

---

### `ferro-reservation/tests/concurrent_hold_postgres.rs` — NEW (cfg-gated)

**Analog:** `ferro-reservation/tests/concurrent_hold.rs` (identical structure, all test bodies mirrored)

**Note:** No existing Postgres-gated integration test exists in any `ferro-*` crate (RESEARCH.md Q5 confirmed: grep returned empty). This file establishes the first instance of the `#[cfg(feature = "postgres-tests")]` convention.

#### File structure pattern

```rust
//! Postgres-gated mirror of concurrent_hold.rs.
//!
//! Run with: `cargo test -p ferro-reservation --features postgres-tests`
//! Requires: DATABASE_URL env var pointing to a Postgres instance.

#[cfg(feature = "postgres-tests")]
mod postgres_tests {
    // Re-declare harness (cannot share across integration test binaries)
    use async_trait::async_trait;
    use ferro_reservation::{ReservationContext, ReservationError, ReservationKernel, Resource};
    use sea_orm::{
        ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, PaginatorTrait,
        QueryFilter,
    };
    use sea_orm_migration::MigratorTrait;
    use std::sync::Arc;
    use std::time::Duration;

    struct TestMigrator;

    #[async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![
                Box::new(ferro_audit::CreateAuditLogTable),
                Box::new(ferro_reservation::CreateReservationsTable),
            ]
        }
    }

    async fn fresh_pg_db() -> DatabaseConnection {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for postgres-tests feature");
        let conn = Database::connect(&url).await.expect("connect to postgres");
        TestMigrator::up(&conn, None).await.expect("migrate");
        conn
    }

    // ... TestResource identical to concurrent_hold.rs ...

    // Mirror all four test functions with identical assertions:
    // - hold_race_capacity_1_exactly_one_succeeds
    // - hold_race_capacity_n_admits_exactly_n
    // - hold_non_overlapping_keys_both_succeed
    // - hold_race_audit_atomicity_exactly_n_audit_rows
}
```

**Key difference from SQLite version:** `fresh_pg_db()` reads `DATABASE_URL` env var instead of using `"sqlite::memory:"`. All test bodies are otherwise identical — the transaction-based fix works on both backends via `begin_with_config(Some(IsolationLevel::Serializable), ...)`.

---

### `ferro-reservation/Cargo.toml` — ADD `[features]` section + `sqlx-postgres` dev-dep

**Analog:** No codebase analog — first `[features]` section in any `ferro-*` primitive crate.

**Pattern to add** (after the existing `[dev-dependencies]` block at line 29):

```toml
[features]
# Gate Postgres integration tests behind this feature.
# Enable with: cargo test -p ferro-reservation --features postgres-tests
postgres-tests = []

# Enable sqlx-postgres for SQLSTATE detection (40001 serialization failure).
# Required when consumers use a Postgres backend.
postgres = ["sea-orm/sqlx-postgres"]
```

**Modify existing dev-dependency** (current line 29):

```toml
# Before:
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }

# After:
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
```

`sqlx-postgres` is added to dev-dependencies unconditionally so the SQLSTATE match arm compiles in the test binary. The production `postgres` feature gates it for library consumers. This mirrors the `[features]` + `[dev-dependencies]` separation convention.

---

### `docs/src/database/reservations.md` — DOC UPDATE

**Analog:** Same file. The surrounding sections are accurate and set the tone. Match:
- The terse, neutral voice of the `## Consistency Model` header and its `commit/release/extend` paragraph (lines 362-368)
- The code-block-then-prose rhythm of the `hold` sequence description (lines 136-148)

#### Replace `hold` sequence note (lines 145-148 of current file)

Current stale text:
```
Note: the capacity check and the INSERT are two separate statements, not a
single atomic SQL operation. Under SQLite's serial-writer semantics concurrent
tasks should serialize `hold` calls at the application layer (e.g., a
`tokio::Mutex` per resource key). See the Consistency Model section.
```

Replace with:
```
The capacity check, INSERT, and audit write are wrapped in a single serializable
transaction. Concurrent callers on the same `(key, window)` are serialized at
the database level — no application-layer mutex is required. The conflict-losing
task receives `ReservationError::Insufficient`.
```

#### Replace Consistency Model section body (lines 370-382 of current file)

Current stale text (lines 370-382):
```
**`hold` on SQLite:** SQLite WAL mode serializes writers at the file level, but
the three-step hold sequence (capacity SELECT + held SELECT + INSERT) is not a
single statement. Under concurrent tokio tasks connecting to the same SQLite
database, the capacity check and the INSERT can interleave. Consumers running
concurrent holds against SQLite should serialize `hold` calls at the application
layer — a `tokio::sync::Mutex` per resource key is the idiomatic pattern.

**`hold` on Postgres:** Under Postgres `READ COMMITTED`, the capacity check has
a theoretical race window between the SELECT and the INSERT. The current crate is
SQLite-validated; Postgres correctness for the capacity check is on the roadmap
as a follow-up addition (`SELECT FOR UPDATE` or a counter column approach).
`commit`, `release`, and `extend` via `GuardedUpdate` are race-free on both
dialects.
```

Replace with:
```
**`hold`:** The capacity check, INSERT, and audit write execute inside a
`SERIALIZABLE` transaction (`sea_orm::IsolationLevel::Serializable`). On SQLite
the transaction aligns with the WAL single-writer model; on Postgres it prevents
phantom reads between the SELECT and INSERT. If two concurrent tasks race on the
same `(key, window)`, the database serializes them — exactly one succeeds and the
other receives `ReservationError::Insufficient`. No application-layer mutex is
needed.

A conflict-losing task on Postgres may receive SQLSTATE `40001` (serialization
failure); the kernel translates this to `ReservationError::Insufficient` before
returning to the caller. The error contract is uniform across backends.

`commit`, `release`, and `extend` via `GuardedUpdate` are race-free on both
dialects (single `UPDATE … WHERE` statement).
```

---

## Shared Patterns

### SeaORM `begin_with_config` — the single load-bearing transaction primitive

**Source:** `framework/src/database/transaction.rs:transaction_with` (lines 99-123)
**Apply to:** `ferro-reservation/src/kernel.rs::hold` body

```rust
// framework/src/database/transaction.rs lines 110-112 — exact call pattern:
let txn = db
    .inner()
    .begin_with_config(Some(isolation_level), Some(AccessMode::ReadWrite))
    .await
    .map_err(|e| FrameworkError::database(format!("Failed to begin transaction: {e}")))?;
```

Note: `framework` uses `.inner()` because it wraps `DatabaseConnection` in its own `DB` type. `kernel.rs` has `conn: &C` directly — call `conn.begin_with_config(...)` without `.inner()`.

### Rollback-on-drop discipline

**Source:** `ferro-orm/src/guarded.rs` test T-16-6 (lines 315-347); `framework/src/database/transaction.rs` lines 114-122

Both confirm: when a `DatabaseTransaction` goes out of scope without `.commit()`, rollback is automatic. Early `return Err(...)` before `txn.commit()` in `hold` is sufficient — no explicit `txn.rollback()` needed.

### Error mapping with explicit match arms

**Source:** `ferro-reservation/src/kernel.rs:182-210` (the `commit` method's `GuardedError` mapping):

```rust
// kernel.rs lines 204-209 — the .map_err with match pattern used in commit/release/extend:
.map_err(|e| match e {
    GuardedError::NoRowsAffected => ReservationError::ConflictingState {
        id: handle.id,
        expected: "held",
    },
    other => ReservationError::Guarded(other),
})?;
```

For the `txn.commit()` step in `hold`, mirror this pattern with SQLSTATE detection instead of `GuardedError` matching (see `is_serialization_failure` helper above).

### Integration test harness (TestMigrator + fresh_db + TestResource)

**Source:** `ferro-reservation/tests/concurrent_hold.rs` lines 33-92; also `ferro-reservation/src/kernel.rs:420-481` (in-module test version)

```rust
// The three-piece harness every integration test in this crate uses:
struct TestMigrator; // + MigratorTrait impl referencing CreateAuditLogTable + CreateReservationsTable
async fn fresh_db() -> DatabaseConnection { Database::connect("sqlite::memory:") + migrate }
struct TestResource { capacity_value: u32 }; // + Resource impl with live DB held() query
```

Every new test function in `concurrent_hold.rs` and `concurrent_hold_postgres.rs` uses this harness. Do not extract it to a shared module — integration test binaries cannot share code across files without a `tests/common/mod.rs` (not needed here; duplication is acceptable per existing convention).

### Audit query in tests

**Source:** `ferro-reservation/src/kernel.rs` test `hold_emits_audit_entry` (lines 724-745):

```rust
// kernel.rs lines 734-745 — the audit history query pattern reused in SC-5 test:
let history = ferro_audit::history_for_target(
    &ferro_audit::AuditTarget::new("reservation", handle.id.to_string()),
    &conn,
)
.await
.expect("query audit");
assert!(!history.is_empty(), "...");
assert_eq!(history.last().unwrap().action, "reservation.held");
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `ferro-reservation/Cargo.toml` (`[features]` section) | config | — | No `ferro-*` primitive crate has a `[features]` section. This establishes the first instance. RESEARCH.md Q5 confirmed: grep across all sibling crates returned empty. The planner should follow the `[features]` + `[dev-dependencies]` split documented above. |

---

## Analog Verification Status

| Analog File | Verified By | Status |
|-------------|-------------|--------|
| `framework/src/database/transaction.rs` | Read tool (full file) | CONFIRMED — `begin_with_config(Some(isolation_level), Some(AccessMode::ReadWrite))` at line 110 |
| `ferro-orm/src/guarded.rs` | Read tool (full file) | CONFIRMED — T-16-6 rollback discipline at lines 315-347 |
| `ferro-reservation/src/kernel.rs` | Read tool (full file) | CONFIRMED — `commit` method at lines 182-245; `hold` race at lines 54-176; test harness at lines 411-745 |
| `ferro-reservation/tests/concurrent_hold.rs` | Read tool (full file) | CONFIRMED — mutex pattern + spawn + join harness at lines 101-170 |
| `ferro-reservation/tests/property_invariants.rs` | Read tool (lines 1-60) | CONFIRMED — proptest + `tokio::Runtime::Builder` pattern; proptest NOT used in concurrent_hold.rs so plain loop preferred |
| `ferro-reservation/src/error.rs` | Read tool (full file) | CONFIRMED — `ReservationError::Insufficient { requested, available, capacity }` at lines 13-18 |
| `docs/src/database/reservations.md` | Read tool (offsets 1-15, 130-210, 355-394) | CONFIRMED — stale claims at lines 145-148 and 370-382 identified |

---

## Metadata

**Analog search scope:** `framework/src/database/`, `ferro-orm/src/`, `ferro-reservation/src/`, `ferro-reservation/tests/`, `docs/src/database/`
**Files read:** 8
**Pattern extraction date:** 2026-05-21
