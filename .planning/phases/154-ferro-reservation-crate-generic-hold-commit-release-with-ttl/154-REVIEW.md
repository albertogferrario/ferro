---
phase: 154-ferro-reservation-crate-generic-hold-commit-release-with-ttl
reviewed: 2026-05-14T00:00:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - ferro-reservation/Cargo.toml
  - ferro-reservation/src/lib.rs
  - ferro-reservation/src/error.rs
  - ferro-reservation/src/resource.rs
  - ferro-reservation/src/context.rs
  - ferro-reservation/src/handle.rs
  - ferro-reservation/src/event.rs
  - ferro-reservation/src/kernel.rs
  - ferro-reservation/src/sweeper.rs
  - ferro-reservation/src/entity.rs
  - ferro-reservation/src/migration.rs
  - ferro-reservation/tests/concurrent_hold.rs
  - ferro-reservation/tests/property_invariants.rs
  - ferro-reservation/tests/integration_with_audit_and_events.rs
  - ferro-reservation/README.md
  - docs/src/database/reservations.md
  - Cargo.toml
  - .github/workflows/publish.yml
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 154: Code Review Report

**Reviewed:** 2026-05-14
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

ferro-reservation is well-structured. The critical-path focus areas all pass:
the `NoRowsAffected → ConflictingState` mapping fires via `.map_err(|e| match e { ... })?` before the `?` operator in all three transition methods (commit, release, extend); the three-phase ordering (GuardedUpdate → AuditEntry::write → ferro_events::dispatch) is consistent across all four state-transition methods; the migration has an explicit `impl MigrationName` returning a unique slug; no `unwrap()` on Result types in production paths; no hardcoded application identity; no co-author lines in commits.

Three issues warrant attention before shipping: a silent integer truncation on `quantity as i32`, a documentation step-order inversion in the user-facing `hold()` sequence, and an unguarded `quantity = 0` path that admits vacuous holds.

---

## Warnings

### WR-01: `quantity as i32` silently truncates for values > i32::MAX

**File:** `ferro-reservation/src/kernel.rs:102`

**Issue:** `hold()` accepts `quantity: u32` and stores it as `ActiveValue::Set(quantity as i32)`. For any `quantity` value above `2_147_483_647` the cast wraps to a negative i32 (Rust's `as` cast semantics for out-of-range truncation). The stored row would have a negative quantity column, corrupting the `Resource::held` aggregation (which sums `i32` values) and breaking the capacity invariant without any error returned to the caller.

**Fix:**
```rust
// In hold(), before the INSERT block:
let quantity_i32 = i32::try_from(quantity).map_err(|_| {
    ReservationError::Db(sea_orm::DbErr::Custom(
        format!("reservation: quantity {quantity} overflows i32 column"),
    ))
})?;

// Then use quantity_i32 in the ActiveModel:
quantity: ActiveValue::Set(quantity_i32),
```

---

### WR-02: `hold()` accepts `quantity = 0`, silently inserts a vacuous row

**File:** `ferro-reservation/src/kernel.rs:72`

**Issue:** The capacity check is `if quantity > available { ... }`. When `quantity = 0`, the check always passes regardless of `available`, and a zero-quantity row is inserted into the reservations table. A `ReservationHandle` with `quantity = 0` is returned. The handle can then be committed or released, generating audit entries for a no-op operation. No capacity is actually reserved, and `Resource::held` is not affected, but the row and two audit entries are written unnecessarily. This is a latent correctness gap for any consumer that calls `kernel.hold(&conn, key, window, 0, ttl, &ctx)` by mistake.

**Fix:**
```rust
// At the top of hold(), after the capacity check:
if quantity == 0 {
    return Err(ReservationError::Db(sea_orm::DbErr::Custom(
        "reservation: quantity must be >= 1".to_string(),
    )));
}
```

Alternatively, add `Insufficient` documentation that `quantity = 0` is always successful and document it as intentional if zero-quantity holds are a desired primitive. Either way, the current behavior should be explicit.

---

### WR-03: `docs/src/database/reservations.md` hold() step order is inverted vs code

**File:** `docs/src/database/reservations.md:141-142`

**Issue:** The user-facing documentation lists the `hold()` sequence as:

> 5. Emit `ReservationEvent::Held` via `ferro-events`.  
> 6. Write one `AuditEntry` with `action = "reservation.held"` via `ferro-audit`.

The actual code in `kernel.rs` does the opposite: audit is written first (step 6), then the event is dispatched (step 7). The correct operational order is INSERT → audit → event, not INSERT → event → audit. This matters because the documented semantics say "audit is unconditional; event is best-effort" — a reader of the docs might believe the audit is written after the event and is therefore dependent on the event dispatch succeeding, which is wrong.

**Fix:** Swap steps 5 and 6 in `docs/src/database/reservations.md`:

```markdown
`hold` sequence:
1. Call `R::capacity(&conn, &key, &window)`.
2. Call `R::held(&conn, &key, &window)`.
3. If `held + quantity > capacity` → `Err(Insufficient { requested, available, capacity })`.
4. INSERT one `reservations` row with `status = 'held'`, `expires_at = now() + ttl`.
5. Write one `AuditEntry` with `action = "reservation.held"` via `ferro-audit`.
6. Emit `ReservationEvent::Held` via `ferro-events`.
7. Return `ReservationHandle`.
```

---

## Info

### IN-01: `extend()` event dispatch absence is documented in code but not in user docs

**File:** `ferro-reservation/src/kernel.rs:382-386`, `docs/src/database/reservations.md`

**Issue:** `extend()` writes an audit entry (`"reservation.extended"`) but emits no `ReservationEvent` (no `Extended` variant in v0, per D-25). The kernel.rs comment explains this clearly. The user-facing doc at `docs/src/database/reservations.md` does not mention this gap — the lifecycle methods table lists `extend` but the event subscription section only documents four variants. A consumer wiring up event-driven live read-models would not learn from the docs that TTL extensions are audit-only and not observable via events.

**Suggestion:** Add a note to the lifecycle table row for `extend` and/or to the ReservationEvent Subscription section:

```markdown
> **Note:** `extend` does not emit a `ReservationEvent`. Extensions are recorded
> only in the audit log (`action = "reservation.extended"`). Consumers needing
> extension observability should subscribe to the audit log directly.
```

---

### IN-02: Concurrent-hold test is effectively sequential — test name may mislead

**File:** `ferro-reservation/tests/concurrent_hold.rs:101`

**Issue:** The test is annotated `#[tokio::test(flavor = "current_thread")]` and uses a `tokio::Mutex` that serializes every `hold()` call. The test exercises the capacity bookkeeping correctly but does not exercise any real concurrent execution path — it is a sequential stress test with 20 ordered calls. The file-level doc explains this well (lines 9-15), but the test function name `concurrent_hold_against_capacity_5_admits_exactly_5` and the module path `concurrent_hold.rs` could mislead a reader scanning test names into believing a real concurrency scenario was exercised.

**Suggestion:** Low priority — the doc comment is accurate. Consider renaming the test function to `serialized_hold_against_capacity_5_admits_exactly_5` or adding `// Execution is serialized by tokio::Mutex; see module doc` inline. Not blocking.

---

_Reviewed: 2026-05-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
