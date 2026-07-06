---
phase: 154-ferro-reservation-crate-generic-hold-commit-release-with-ttl
fixed_at: 2026-05-14T00:00:00Z
review_path: .planning/phases/154-ferro-reservation-crate-generic-hold-commit-release-with-ttl/154-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 154: Code Review Fix Report

**Fixed at:** 2026-05-14
**Source review:** `.planning/phases/154-ferro-reservation-crate-generic-hold-commit-release-with-ttl/154-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (WR-01, WR-02, WR-03)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: `quantity as i32` silently truncates for values > i32::MAX

**File modified:** `ferro-reservation/src/kernel.rs`
**Commit:** `15286c48`
**Applied fix:** Replaced `ActiveValue::Set(quantity as i32)` with a checked conversion using `i32::try_from(quantity)`, returning `ReservationError::Db(DbErr::Custom(...))` when the value exceeds `i32::MAX`. The `quantity_i32` binding is computed just before the `ActiveModel` construction so it can be used in the insert block.

---

### WR-02: `hold()` accepts `quantity = 0`, silently inserts a vacuous row

**File modified:** `ferro-reservation/src/kernel.rs`
**Commit:** `15286c48`
**Applied fix:** Added an explicit early-return guard immediately after the capacity check block. When `quantity == 0` the method returns `Err(ReservationError::Db(DbErr::Custom("reservation: quantity must be >= 1")))` before any INSERT or audit emission occurs.

> WR-01 and WR-02 both modify `ferro-reservation/src/kernel.rs` and were committed together in a single atomic commit.

---

### WR-03: `docs/src/database/reservations.md` hold() step order is inverted vs code

**File modified:** `docs/src/database/reservations.md`
**Commit:** `ae1edb8c`
**Applied fix:** Swapped steps 5 and 6 in the `hold` sequence list. The corrected order is now:
- Step 5: Write one `AuditEntry` with `action = "reservation.held"` via `ferro-audit`.
- Step 6: Emit `ReservationEvent::Held` via `ferro-events`.

This matches the actual three-phase invariant in the code (GuardedUpdate → AuditEntry → event dispatch) and correctly conveys that audit is unconditional and precedes best-effort event emission.

## Skipped Issues

None.

## Test Results

`cargo test -p ferro-reservation --all-features` after both commits:

```
running 27 tests (unit)  — all passed
running 1 test  (concurrent_hold)  — passed
running 3 tests (integration_with_audit_and_events)  — all passed
running 2 tests (property_invariants)  — all passed
4 doc-tests ignored
test result: ok. 33 passed; 0 failed
```

`cargo clippy -p ferro-reservation --all-targets -- -D warnings` — clean.
`cargo fmt --all -- --check` — clean.

---

_Fixed: 2026-05-14_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
