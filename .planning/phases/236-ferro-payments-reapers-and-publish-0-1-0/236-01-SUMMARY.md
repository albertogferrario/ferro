---
phase: 236-ferro-payments-reapers-and-publish-0-1-0
plan: 01
subsystem: payments
tags: [sea-orm, sqlite, chrono, ferro-payments, lifecycle, finders]

# Dependency graph
requires:
  - phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto-refund
    provides: lifecycle helpers (mark_released, mark_refunded) and webhook handler pattern this plan's tests mirror
provides:
  - find_expired finder: reserved-only row selection predicate for the release reaper
  - find_refunds_in_flight finder: paid + refund-snapshot + null-refunded_at predicate for the reconcile reaper
affects:
  - 236-02 (release reaper and service methods consume find_expired)
  - 236-03 (reconcile reaper and service methods consume find_refunds_in_flight)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "sea-orm filter chain with .is_not_null() / .is_null() for nullable column predicates"
    - "TDD: tests added to existing #[cfg(test)] module in the production file, before the implementation"

key-files:
  created: []
  modified:
    - ferro-payments/src/intent/lifecycle.rs

key-decisions:
  - "Used paid_at as the age anchor for find_refunds_in_flight (always set for paid rows per lifecycle invariant; simpler than a new refund_requested_at column)"
  - "Status filter pins Reserved-only in find_expired — paid/released rows structurally excluded (T-236-01)"
  - "refunded_at IS NULL filter in find_refunds_in_flight structurally prevents double-processing (T-236-01b)"

patterns-established:
  - "Reaper source queries follow the find_active_for filter-chain pattern: Entity::find() + N filters + .all(conn) + map_err(PaymentError::Db)"

requirements-completed: [PAY-POLY-REAP-01, PAY-POLY-REAP-02]

# Metrics
duration: 5min
completed: 2026-06-20
---

# Phase 236 Plan 01: Lifecycle Finders Summary

**Two sea-orm filter-chain finders added to lifecycle.rs: find_expired (Reserved + ExpiresAt.lt) and find_refunds_in_flight (Paid + RefundAmountCents IS NOT NULL + RefundedAt IS NULL + PaidAt.lt), each with three unit tests against in-memory SQLite.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-06-20T22:36:54Z
- **Completed:** 2026-06-20T22:42:02Z
- **Tasks:** 1 (TDD: RED then GREEN in one task)
- **Files modified:** 1

## Accomplishments
- `find_expired(now, conn)` selects only `Reserved` rows with `expires_at < now`; paid/released rows are structurally excluded by the status filter
- `find_refunds_in_flight(older_than, conn)` selects `Paid` rows with a non-null refund snapshot and null `refunded_at`, age-gated by `paid_at < older_than`
- Six unit tests covering all six behaviors specified in the plan, green against in-memory SQLite
- Clippy clean (`-D warnings`)

## Task Commits

1. **Task 1: Add find_expired and find_refunds_in_flight finders** - `e28fb816` (feat)

**Plan metadata:** committed with docs commit below

## Files Created/Modified
- `ferro-payments/src/intent/lifecycle.rs` - Added two public finders and six `#[cfg(test)]` unit tests

## Decisions Made
- Followed plan exactly: `paid_at` as the age anchor for `find_refunds_in_flight` (D-04 rationale from RESEARCH.md — always set for paid rows, simpler than a new column)
- No new imports were needed; `ColumnTrait`, `EntityTrait`, `QueryFilter`, `ConnectionTrait` were all already present at line 10

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Both finders are in place and unit-tested; Plan 02 (release reaper service methods) and Plan 03 (reconcile reaper service methods) can now compose them directly without inlining SQL
- No blockers

---
*Phase: 236-ferro-payments-reapers-and-publish-0-1-0*
*Completed: 2026-06-20*
