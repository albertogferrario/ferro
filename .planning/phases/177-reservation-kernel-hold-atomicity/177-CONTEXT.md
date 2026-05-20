# Phase 177: ferro-reservation Kernel Atomicity Hardening — `hold` race fix — Context

**Gathered:** 2026-05-20
**Status:** Ready for planning
**Severity:** URGENT — this is a load-bearing invariant of the entire reservation primitive. Every consumer of `ReservationKernel::hold` relies on `held ≤ capacity`. The current implementation cannot guarantee that under concurrent invocations.
**Source:** Consumer field test 2026-05-20 — gestiscilo-it v6.9 β killer-feature acceptance test `concurrent_double_book_same_staff` fails 5/5 deterministically (~0.07s each). Documented at `.planning/phases/152-booking-staff-binding/152-UI-FINDINGS.md` (Bug R5) in the gestiscilo-it repo.

## Phase Boundary

Phase 177 fixes exactly one invariant: `ReservationKernel::hold` is atomic under concurrent invocation. Two responsibilities:

1. **Land the fix** — wrap the check-then-act sequence so two concurrent INSERTs cannot both succeed.
2. **Add a load-bearing concurrency test** at the ferro-reservation layer (independent of consumer projects) so future regressions are caught in CI.

Out of scope:
- Changing the `Resource` trait surface.
- Changing `commit/release/sweeper` (already atomic via GuardedUpdate).
- Re-architecting reservations as event-sourced.
- Performance optimization beyond what the atomicity fix requires.

## The Race (Verified)

`ReservationKernel::hold` at `ferro-reservation/src/kernel.rs:54-122`:

```rust
pub async fn hold<C: ConnectionTrait>(
    &self,
    conn: &C,
    key: R::Key,
    window: R::Window,
    quantity: u32,
    ttl: Duration,
    ctx: &ReservationContext,
) -> Result<ReservationHandle, ReservationError> {
    let id = Uuid::new_v4();

    // Steps 2–3: capacity check (consumer-defined)
    let capacity = self.resource.capacity(conn, &key, &window).await?;
    let held = self.resource.held(conn, &key, &window).await?;
    let available = capacity.saturating_sub(held);

    // Step 4: enforce invariant
    if quantity > available {
        return Err(ReservationError::Insufficient { requested: quantity, available, capacity });
    }

    // Step 5: INSERT reservations row
    let am = reservations::ActiveModel { ... };
    am.insert(conn).await.map_err(ReservationError::Db)?;
    ...
}
```

Two concurrent `tokio::spawn` tasks racing this method on identical `(key, window)`:
- T_A: read held=0, compute available=1, pass check
- T_B: read held=0, compute available=1, pass check (still no row exists)
- T_A: INSERT row with id=A (held becomes 1)
- T_B: INSERT row with id=B (held becomes 2 — INVARIANT VIOLATED)

There is no transaction wrapping these steps. There is no unique index that would reject the second INSERT. `GuardedUpdate` is used in `commit/release/sweeper` (which all UPDATE existing rows) but never in `hold` (the only method that INSERTs new rows). PITFALLS T-69-1.2 documentation in the consumer field test was wrong against current implementation.

## Implementation Decisions (Locked)

- **D-01 — Fix is kernel-internal.** The race is in `kernel.rs::hold`. Consumers don't change. `Resource` trait surface unchanged. `commit/release/sweeper` unchanged.
- **D-02 — Existing GuardedUpdate discipline preserved.** This phase does NOT touch the existing atomic semantics of `commit/release/sweeper`. It only adds atomicity to `hold`.
- **D-03 — Backend portability.** Fix MUST work on SQLite (consumer dev) AND Postgres (consumer prod). Neither backend can be deprioritized.
- **D-04 — Audit log semantics unchanged.** `reservation.held` audit row still written exactly once per successful hold. Conflict-losing task does NOT write an audit row. (Implementation note: the audit write happens INSIDE the atomicity scope if Path (a) is chosen, ensuring rollback consistency.)
- **D-05 — No new external crates.** `sea_orm::TransactionTrait` is already in the workspace; `ferro_orm::GuardedUpdate` is already in scope. Nothing new to vendor.

## Open Plan-Time Decisions

### Fix path (planner picks one)

**(a) Wrap `hold` body in `conn.begin()` transaction with serializable isolation.** Minimum-blast-radius. Mirrors existing `commit/release` GuardedUpdate discipline by giving `hold` the same atomic-block semantics.
- Pros: Backend-portable via `sea_orm::TransactionTrait`. SQLite serializes write transactions natively (one writer at a time). Postgres needs `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE` or rely on row-level locking via `SELECT ... FOR UPDATE` inside the held()/capacity() calls.
- Cons: Postgres SERIALIZABLE can produce `40001` serialization failures on contention — caller must retry. Acceptable for the conflict-losing task (it gets a retry-then-Err shape); needs documentation.
- Subtle: `Resource::held` and `Resource::capacity` are consumer-defined — they take `&C: ConnectionTrait`, and a `&DatabaseTransaction` satisfies that bound. So passing the txn through is transparent to existing consumers.

**(b) Add a unique partial index on `reservations (resource_kind, resource_key, window_hash) WHERE status='held'`.** Schema-level enforcement; second INSERT fails with a unique-constraint violation.
- Pros: No transaction needed; the DB enforces atomicity at the storage layer.
- Cons: Requires deterministic JSON canonicalization for `window_hash` (JSON object key ordering, number precision). Cross-backend canonicalization is non-trivial. Also: `capacity > 1` resources legitimately have multiple `held` rows for the same `(key, window)` — the unique constraint would BREAK capacity > 1 use cases. So this path needs a per-resource composite that includes a discriminator (`held_position`?) and gets ugly fast.

**(c) `INSERT … SELECT … WHERE NOT EXISTS` atomic check-and-insert.** One SQL statement; atomic at the DB layer.
- Pros: Backend-portable; no transaction needed; works with `capacity > 1`.
- Cons: The `WHERE NOT EXISTS` predicate would need to encode the full capacity check inline (or a subquery counting `held` rows + comparing to capacity). Sea-ORM doesn't have a first-class API for this — likely requires raw SQL with backend-branched escaping. Loses the `Resource::capacity()` extension point (or duplicates its logic in SQL).

**Recommendation: Path (a).** Smallest delta, matches existing GuardedUpdate discipline, no schema migration, works with `capacity > 1`, the SERIALIZABLE retry concern is documented and acceptable. Plans 01+02 in this phase ship Path (a) unless plan-time research surfaces a blocker.

### Concurrency test infrastructure

Test lives in `ferro-reservation/tests/concurrency.rs` (NEW). Pattern: spawn two tokio tasks racing `kernel.hold` against an in-memory SQLite (and ideally one against a docker-compose Postgres too, gated on `cfg(feature = "postgres")` or `cfg(test_postgres)`). Assert `Ok` count + `Err` count match `capacity` + `(N - capacity)` where N is the number of racing tasks.

A second test asserts boundary semantics are preserved — two non-overlapping windows on the same key both succeed (no false positives from the atomicity fix).

## Specifics

- **Single-writer SQLite:** SQLite's default journaling mode is `delete` (one writer at a time across the whole DB file). Path (a) leverages this naturally — the second tokio task's `conn.begin()` waits for the first to commit, then re-reads held() inside the new txn and sees the just-inserted row.
- **Postgres SERIALIZABLE:** requires either (i) `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE` inside the txn, OR (ii) `SELECT ... FOR UPDATE` on a key sentinel row. Option (i) is simpler. Caller-retry on `40001` is the consumer's job per the documented contract.
- **Existing kernel tests** (`kernel.rs:494-560` happy-path + insufficient) continue to pass byte-identical — they're single-task tests that don't exercise the race.

## Code Insights (Reusable Assets)

- `ferro-reservation/src/kernel.rs:175-220` — `commit` method's existing GuardedUpdate pattern is the structural reference for the `hold` txn wrapper.
- `ferro-orm::GuardedUpdate` (workspace) — already used in commit/release.
- `sea_orm::TransactionTrait` (workspace) — `conn.begin()` returns `&DatabaseTransaction` which satisfies `&C: ConnectionTrait`. Existing Resource impls (`NoleggioUnitResource::held`, `StaffSlotResource::held`) work unchanged when passed a txn instead of a raw connection.

## Established Patterns

- Reservation kernel public API: `hold/commit/release/run_sweep_once/extend` — surface STAYS THE SAME after this fix. Only `hold`'s internal implementation changes.
- Audit log discipline: every state transition (held, committed, released, expired) writes one `audit_entries` row. After this fix: `hold`'s audit write is inside the atomic block, so the row is rolled back on conflict.
- Error categorization: `ReservationError::Insufficient { requested, available, capacity }` is the user-visible signal for capacity violation. Conflict-losing tasks under Path (a) return this same variant (NOT a new "concurrent_conflict" variant).

## Canonical References

- Consumer field test: `gestiscilo-it/.planning/phases/152-booking-staff-binding/152-UI-FINDINGS.md` (Bug R5)
- Killer-feature failing test: `gestiscilo-it/tests/integration/staff_booking_concurrency_tests.rs::concurrent_double_book_same_staff` (fails 5/5)
- Kernel implementation: `ferro-reservation/src/kernel.rs:54-122` (the race) + `ferro-reservation/src/kernel.rs:175-220` (the commit pattern to mirror)
- Phase 152 (consumer): `gestiscilo-it/.planning/phases/152-booking-staff-binding/` — STBOOK-15 is the load-bearing acceptance that depends on this fix.
- Phase 130/131/132 (consumer inventory): existing single-writer tests that must continue to pass.

## Folded Todos

None.
