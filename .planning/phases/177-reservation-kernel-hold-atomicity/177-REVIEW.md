---
phase: 177-reservation-kernel-hold-atomicity
reviewed: 2026-05-21T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - ferro-reservation/src/kernel.rs
  - ferro-reservation/tests/concurrent_hold.rs
  - ferro-reservation/tests/concurrent_hold_postgres.rs
  - ferro-reservation/Cargo.toml
  - docs/src/database/reservations.md
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
status: issues_found
---

# Phase 177: Code Review Report

**Reviewed:** 2026-05-21
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

The fix is structurally correct and well-scoped. `hold` now opens a
`begin_with_config(Some(IsolationLevel::Serializable), Some(AccessMode::ReadWrite))`
transaction at entry, performs the capacity check, the INSERT, and the audit
write against the `&txn` handle, then commits with a closure that translates
Postgres SQLSTATE 40001 to `ReservationError::Insufficient`. `commit`,
`release`, `extend`, and `run_sweep_once` are byte-identical to pre-fix (D-02
satisfied). The public signature of `hold` changed only by adding
`+ TransactionTrait` to the generic bound (D-01 satisfied). Audit write is
inside the txn (D-04 satisfied). `is_serialization_failure` is correctly
cfg-gated and matches SeaORM's `SqlxError(sqlx::Error::Database(e))` pattern
across both `DbErr::Exec` and `DbErr::Query` arms. Event dispatch is correctly
outside the txn and after `txn.commit()`.

The test rewrite in `concurrent_hold.rs` removes the previous
application-layer `tokio::Mutex` and proves the kernel arbitrates concurrent
holds intrinsically. SC-1 (2 tasks @ cap=1), SC-1 extended (6 tasks @ cap=5),
SC-2 (non-overlapping keys), and SC-5 (audit-row count == capacity) are all
present and assert the correct invariants. The Postgres-gated mirror compiles
to empty without `--features postgres-tests` (correct `#![cfg(...)]` inner
attribute at file top). `Cargo.toml` adds `sqlx` as an optional direct dep and
feature-gates it behind `sqlx-postgres`, with `postgres-tests` chaining to it.

Findings below are observations that do not block the fix; they are
clarifications or minor polish items.

## Warnings

### WR-01: `current_thread` runtime may under-stress the race condition

**File:** `ferro-reservation/tests/concurrent_hold.rs:90,133,187,220`
**File:** `ferro-reservation/tests/concurrent_hold_postgres.rs:98,148`

**Issue:** All race tests are annotated `#[tokio::test(flavor = "current_thread")]`.
On a current-thread runtime, `tokio::spawn`-ed tasks are cooperatively
multiplexed on a single OS thread — only one task executes between `.await`
points. This is sufficient to expose the pre-fix race (the original bug was
visible in single-threaded `tokio::Mutex`-less calls — the race window is
between `.await`s, not across OS threads), but it under-stresses the
serializability claim on Postgres. The Postgres SQLSTATE 40001 path in
particular requires *true* parallelism on the database side; a single-thread
tokio runtime may not generate the kernel-level interleaving needed to drive
two BEGIN-ISOLATION-LEVEL-SERIALIZABLE transactions into a true conflict.

The SQLite-side test still works because SeaORM's `SqliteConnection` pool
serializes operations and the `.await` between `capacity()` and the INSERT
opens the race window on its own. But for the Postgres test, consider
`flavor = "multi_thread", worker_threads = 4` (or omit `flavor` entirely to
get the default multi-threaded runtime) so multiple sqlx pool connections can
race in true parallelism — that is the configuration that actually exercises
the SSI path the fix relies on.

**Fix:**
```rust
// In concurrent_hold_postgres.rs:
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hold_race_capacity_1_exactly_one_succeeds_postgres() { ... }
```
The SQLite-side tests can remain `current_thread` (the race surfaces there
without thread-level parallelism), but consider validating empirically that
flipping to `multi_thread` does not change the 50-iteration pass rate.

---

### WR-02: `fresh_pg_db()` panic message on DATABASE_URL collision risk

**File:** `ferro-reservation/tests/concurrent_hold_postgres.rs:41-50`

**Issue:** `fresh_pg_db()` runs `TestMigrator::down` then `up` on whatever
database `DATABASE_URL` points at. The file-level module doc (lines 1-13) warns
"the database is empty", but the `.expect("DATABASE_URL must be set for the
postgres-tests feature")` message does not warn that running this test against
a production-like DB will *drop* the `reservations` and `audit_entries` tables.
A developer who happens to have `DATABASE_URL` exported in their shell from
another project is one `cargo test --features postgres-tests` away from data
loss.

**Fix:** Make the panic message louder about the destructive nature, and
ideally refuse to run if the URL host is not localhost / 127.0.0.1 / a known
test marker. Example:

```rust
let url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set for the postgres-tests feature \
             — WARNING: the test drops and recreates `reservations` and \
             `audit_entries` tables. Point only at a disposable test DB.");

// Optionally:
let parsed = url::Url::parse(&url).expect("DATABASE_URL must be a valid URL");
let host = parsed.host_str().unwrap_or("");
assert!(
    matches!(host, "localhost" | "127.0.0.1" | "::1") || url.contains("test"),
    "refusing to run destructive Postgres test against non-local host {host:?}; \
     export DATABASE_URL pointing at a disposable test instance"
);
```

The `url` crate is already a transitive dep of sqlx, so this adds no new
external surface. Even without the host check, strengthening the panic
message is a cheap robustness win.

## Info

### IN-01: Doc step list still says `R::capacity(&conn, ...)` but the kernel now calls it on the txn

**File:** `docs/src/database/reservations.md:137-138`

**Issue:** The "hold sequence" step list at lines 137-138 says
"Call `R::capacity(&conn, &key, &window)`" and
"Call `R::held(&conn, &key, &window)`". After the fix, the kernel calls these
against the *internal transaction handle*, not the consumer-passed
connection. The consumer-facing API is unchanged (the user still calls
`kernel.hold(&conn, ...)`), so this is technically still accurate at the
consumer boundary, but the seven-step description is now an
*implementation-level* enumeration that no longer matches the implementation
in one detail.

**Fix:** Either (a) reword 137-138 to be connection-agnostic
("Call `R::capacity` and `R::held` to determine available units"), or
(b) explicitly note the SERIALIZABLE txn wraps steps 1-5:

```markdown
`hold` sequence (all of steps 1–5 execute inside one SERIALIZABLE transaction):
1. Generate a UUIDv4 reservation id (pure).
2. Call `R::capacity` and `R::held` inside the transaction to determine
   `available = capacity − held`.
3. If `held + quantity > capacity` → `Err(Insufficient { ... })` and the
   transaction auto-rolls back.
4. INSERT one `reservations` row with `status = 'held'`,
   `expires_at = now() + ttl`.
5. Write one `AuditEntry` with `action = "reservation.held"` via `ferro-audit`.
6. Commit the transaction. (Postgres SQLSTATE 40001 is translated to
   `Insufficient` here.)
7. Emit `ReservationEvent::Held` via `ferro-events` (best-effort, post-commit).
8. Return `ReservationHandle`.
```

The current text is not actively misleading, but the eight-step phrasing
makes the txn boundary explicit and matches the in-source comments.

---

### IN-02: `dev-dependencies` enables `sqlx-postgres` unconditionally — postgres test binaries always link sqlx-postgres

**File:** `ferro-reservation/Cargo.toml:39`

**Issue:** Line 39 sets:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
```
This means every `cargo test -p ferro-reservation` (including the default
non-postgres-tests run) pulls in `sqlx-postgres`. That is fine for
correctness — the `is_serialization_failure` cfg arm in `kernel.rs` is gated
on the *crate*'s `sqlx-postgres` feature (line 31), not on sea-orm's. The
crate-feature is opt-in via `--features sqlx-postgres` or `--features
postgres-tests`. So the kernel's live arm is correctly stubbed for default
test builds.

However, this couples test compile time to having sqlx-postgres' native
compile cost (and TLS dep chain) on every developer machine. Consider
moving `"sqlx-postgres"` behind a dev-feature gate, or accepting the cost as
a tradeoff for keeping the dev-dep table flat. This is a stylistic call; flag
it for awareness.

**Fix:** Status quo is acceptable. If compile-time is a concern, the
alternative is:
```toml
[dev-dependencies]
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }

[target.'cfg(any())'.dev-dependencies]
# trick to make a feature-gated dev-dep — not idiomatic, generally not worth it
```
No change required for SC compliance.

---

### IN-03: Audit-atomicity SC-5 test does not assert audit-row count when capacity > 1

**File:** `ferro-reservation/tests/concurrent_hold.rs:220-290`

**Issue:** `hold_race_audit_atomicity_exactly_n_audit_rows` asserts the audit
invariant at `CAPACITY = 1`. The general invariant — "audit-row count ==
successful hold count" — is the SC-5 success criterion. At capacity=1 with
2 tasks, the invariant degenerates to "1 audit row total". Adding a
parametric variant at `CAPACITY = 5, TASKS = 6` would test the invariant in
its general form (5 audit rows, not 6).

The 50-iteration race test at `hold_race_capacity_n_admits_exactly_n` (line
133) verifies the *hold-result* count is N, but does not query the audit
table. A third assertion in that test, or a dedicated capacity-N audit-count
test, would close the gap.

**Fix:** Either extend the existing `_exactly_n` test to also assert the
audit row count, or add a parametric capacity=5 variant:

```rust
#[tokio::test(flavor = "current_thread")]
async fn hold_race_audit_atomicity_capacity_n() {
    const CAPACITY: u32 = 5;
    const TASKS: usize = 6;
    // ... race TASKS holds, then:
    use ferro_reservation::ReservationEntity;
    let reservation_rows = ReservationEntity::find().all(&*conn).await.unwrap();
    assert_eq!(reservation_rows.len(), CAPACITY as usize);
    // total audit rows across all reservation ids should also equal CAPACITY
    let mut total_audit_rows = 0;
    for row in &reservation_rows {
        let history = ferro_audit::history_for_target(
            &ferro_audit::AuditTarget::new("reservation", row.id.to_string()),
            &*conn,
        ).await.unwrap();
        total_audit_rows += history.len();
    }
    assert_eq!(total_audit_rows, CAPACITY as usize);
}
```

Not required for v1 — the capacity=1 test is the load-bearing SC-5 check.

---

### IN-04: `is_serialization_failure` lives in module scope rather than as an `impl` associated fn — minor

**File:** `ferro-reservation/src/kernel.rs:445-460`

**Issue:** `is_serialization_failure` is a free function at module scope.
This is fine; it has no state and the cfg pair makes the two arms locally
visible. A pure-style nit is that since it is only called from
`hold`'s commit closure, scoping it as a `pub(crate)` const in the same impl
block or moving it next to `hold` would localize the cfg surface. Not a bug;
flag for awareness only.

**Fix:** No change required. The current placement (immediately after the
`Clone` impl, before the `#[cfg(test)]` module) is clean and discoverable.

---

_Reviewed: 2026-05-21_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
