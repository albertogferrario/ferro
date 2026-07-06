# Phase 177: ferro-reservation Kernel Atomicity Hardening — `hold` race fix — Context

**Gathered:** 2026-05-20 (initial scope); 2026-05-21 (auto-mode lock-in of open plan-time decisions)
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
- **D-06 — Fix path: (a) `conn.begin()` transaction with serializable isolation.** Locked 2026-05-21 (auto-mode recommended default). Smallest delta, matches existing GuardedUpdate discipline, no schema migration, works with `capacity > 1`. Plan-time research may still surface a blocker — if so, that surfaces as a checkpoint, not a silent path switch. See "Rejected paths" below for (b)/(c) rationale.
- **D-07 — Postgres isolation strategy: `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE` inside the txn.** Locked 2026-05-21 (simpler option per Specifics note). The conflict-losing task may receive Postgres SQLSTATE `40001` (serialization failure). Plan-time decides whether to (i) translate `40001` → `ReservationError::Insufficient` at the kernel boundary (preferred — preserves the documented error contract) or (ii) surface `40001` raw as `ReservationError::Db` and require caller-retry. Default for plans 01+02: (i) translate, since the consumer field test expects `Err(Insufficient)` not a db-error variant.
- **D-08 — Concurrency test scope: SQLite primary, Postgres cfg-gated.** Locked 2026-05-21. SQLite tests are unconditional (in-memory, fast, deterministic). Postgres tests are gated on a `cfg` flag (`test_postgres` or feature `postgres-tests`) so CI can run them when a docker-compose Postgres is available, without forcing every contributor to run Postgres locally.

### Claude's Discretion

Plan-time judgment calls left to the planner — no upstream user preference applies:

- **Exact Postgres cfg name** — `feature = "postgres-tests"` vs `cfg(test_postgres)` vs gated on `DATABASE_URL` env var presence. Pick whichever matches existing ferro-reservation test conventions (verify in plan-phase research).
- **Iteration count for race-to-capacity test** — Success Criterion 1 calls for "50/50 runs in CI". Planner picks the loop count (≥50) and whether to use `proptest`/`quickcheck` or a plain `for` loop. Plain loop is preferred unless ferro-reservation already uses proptest.
- **`40001` translation site** — D-07 default is translate at the kernel boundary; if the planner finds the translation cleanly fits inside the txn-retry helper instead, that's equivalent.
- **Whether to extract a `hold_inner(&txn, ...)` helper** — pure refactor judgment. If the txn body is short enough to inline, inline it. If extracting improves readability or unlocks the test seam (e.g., for boundary-case unit tests against an injected txn), extract.
- **Doc updates** — kernel.rs module doc, PITFALLS T-69-1.2 doc fix, and any consumer-facing docs that mention "the kernel arbitrates concurrent holds" — planner sweeps for stale claims and corrects them; no need to enumerate every doc file in advance.

## Rejected Paths (kept for plan-time blocker recovery)

Documented so the planner does not re-litigate, but available if research surfaces a blocker against Path (a).

**(b) Unique partial index on `reservations (resource_kind, resource_key, window_hash) WHERE status='held'`.** REJECTED: breaks `capacity > 1` resources legitimately holding multiple `held` rows for the same `(key, window)`. Would require a per-resource discriminator (`held_position`) — gets ugly fast. Also requires deterministic JSON canonicalization for `window_hash` which is cross-backend non-trivial.

**(c) `INSERT … SELECT … WHERE NOT EXISTS` atomic check-and-insert.** REJECTED: Sea-ORM has no first-class API for this — requires raw SQL with backend-branched escaping, and the `WHERE NOT EXISTS` predicate would need to encode the full capacity check inline. Loses the `Resource::capacity()` extension point or duplicates its logic in SQL.

## Concurrency Test Infrastructure (locked shape)

Test file: `ferro-reservation/tests/concurrency.rs` (NEW).

Required cases:
1. **Race-to-capacity (capacity=1):** spawn 2 tokio tasks racing `kernel.hold` on identical `(key, window)` with `quantity=1`. Assert exactly 1 Ok + 1 `Err(Insufficient)`. Run loop ≥50 iterations (Success Criterion 1 calls for 50/50).
2. **Race-to-capacity (capacity=N, N≥2):** spawn N+1 tokio tasks racing `kernel.hold` with `quantity=1` on the same `(key, window)`. Assert exactly N Ok + 1 `Err(Insufficient)`. Confirms the fix correctly handles `capacity > 1` without false rejections.
3. **Non-overlapping windows (boundary preservation):** two `hold(...)` on same `(key)` with non-overlapping windows BOTH succeed (Success Criterion 2). Single-task sequential test — no race — but lives in concurrency.rs as a regression boundary check.
4. **Audit-log atomicity:** after a race resolves, assert exactly N `audit_entries` rows with action `reservation.held` exist for the `(key, window)` — the conflict-losing task's audit row was rolled back with its transaction (D-04 invariant).

Postgres mirror tests gated on `#[cfg(feature = "postgres-tests")]` (or equivalent — planner picks the exact cfg name); identical cases against a docker-compose Postgres.

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
