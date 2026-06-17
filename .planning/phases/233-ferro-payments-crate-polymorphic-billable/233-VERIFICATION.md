---
phase: 233-ferro-payments-crate-polymorphic-billable
verified: 2026-06-17T00:00:00Z
status: passed
score: 10/10
overrides_applied: 0
re_verification: false
---

# Phase 233: ferro-payments Polymorphic Billable — Verification Report

**Phase Goal:** Create new workspace member `ferro-payments` parallel to `ferro-stripe`. Implement `BillableKind`, `PaymentIntentStatus` enums, the SeaORM `Entity` for `payment_intents`, and lifecycle methods (`create_reserved`, `mark_paid`, `mark_released`, `mark_refunded`, `find_active_for`, `find_by_stripe_session`). Ship migration `m20260617_create_payment_intents` portable across Postgres + SQLite + MySQL with partial unique index `(billable_kind, billable_id) WHERE status IN ('reserved','paid')` and supporting indexes. Unit tests cover state transitions and partial-unique enforcement against in-memory SQLite. No service layer yet.
**Verified:** 2026-06-17T00:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| #  | Truth                                                                                                      | Status     | Evidence                                                                                                                      |
|----|------------------------------------------------------------------------------------------------------------|------------|-------------------------------------------------------------------------------------------------------------------------------|
| 1  | `cargo build -p ferro-payments` succeeds (crate is a real workspace member)                               | VERIFIED   | Workspace `Cargo.toml` line 37: `"ferro-payments"`. Commits 44e6d232+caaee82c confirmed. SUMMARY-01 records build exit 0.    |
| 2  | `PaymentIntentStatus` round-trips reserved/paid/released/failed/refunded through a DB TEXT column          | VERIFIED   | `status.rs`: `DeriveActiveEnum`, `rs_type="String"`, `db_type="Text"`, 5 variants with `string_value`. SUMMARY-01: test `status_string_values_round_trip` green. |
| 3  | `ferro-payments` registered for crates.io publication in Wave 1b                                          | VERIFIED   | `publish.yml` line 247: `WAVE1B_CRATES="… ferro-reservation ferro-payments …"` — after `ferro-orm` (Wave 1a). Correct dependency ordering. |
| 4  | Migration creates `payment_intents` table with all 19 columns and correct nullability                      | VERIFIED   | `m20260617_create_payment_intents.rs`: all 19 `.col()` calls present with correct `.not_null()`/`.null()`. `entity.rs` matches exactly. SUMMARY-02: `migration_creates_table_and_indexes` green. |
| 5  | Migration creates 3 supporting indexes plus the partial unique index (SQLite/PG path)                     | VERIFIED   | Migration source: `idx_payment_intents_tenant_status`, `idx_payment_intents_stripe_session_id` (unique), `idx_payment_intents_payment_intent_id`, plus `uq_payment_intents_active` via `execute_unprepared` WHERE clause. All 4 asserted in `migration_creates_table_and_indexes` test. |
| 6  | A second active row (status reserved/paid) for the same (billable_kind, billable_id) is rejected by the DB | VERIFIED   | `partial_unique_rejects_second_active_row` test: second `INSERT` returns `is_err()`. SUMMARY-02 reports 5/5 tests green.     |
| 7  | After active row transitions to released, a new active row for same billable is accepted                  | VERIFIED   | `partial_unique_allows_new_active_after_release` test: second INSERT after `'released'` succeeds. SUMMARY-02 confirms.       |
| 8  | All six lifecycle methods implemented and re-exported from crate root                                      | VERIFIED   | `lifecycle.rs`: `create_reserved`, `mark_paid`, `mark_released`, `mark_refunded`, `find_active_for`, `find_by_stripe_session` all present. `lib.rs` re-exports all six. SUMMARY-03: 11/11 tests green. |
| 9  | Transitions are atomic guarded updates; stale precondition is Ok(false) no-op                             | VERIFIED   | `lifecycle.rs`: `GuardedUpdate::new` at lines 69, 91, 113; `exec_at_most_one` at lines 80, 102, 124. Tests: `mark_paid_noop_on_wrong_status` asserts `Ok(false)`. |
| 10 | `find_active_for` filters status in (reserved, paid); `find_by_stripe_session` filters by session id      | VERIFIED   | `lifecycle.rs` line 143: `.filter(Column::Status.is_in([Reserved, Paid]))`. Line 156: `.filter(Column::StripeSessionId.eq(session_id))`. Both query tests green. |

**Score:** 10/10 truths verified

---

### Required Artifacts

| Artifact                                                                    | Expected                                                              | Status      | Details                                                                                 |
|-----------------------------------------------------------------------------|-----------------------------------------------------------------------|-------------|-----------------------------------------------------------------------------------------|
| `ferro-payments/Cargo.toml`                                                 | Crate manifest, v0.1.0, ferro-orm path dep, sea-orm + sea-orm-migration | VERIFIED  | `name="ferro-payments"`, `version="0.1.0"` explicit, `sea-orm-migration="1.0"`, `ferro-orm={path="../ferro-orm"}`. No `ferro-stripe`. |
| `ferro-payments/src/lib.rs`                                                 | Module declarations, BillableKind newtype, public re-exports          | VERIFIED    | All 6 lifecycle fns, entity types, status, error, migration re-exported. `BillableKind` struct with `const fn new` and `as_str`. |
| `ferro-payments/src/error.rs`                                               | Minimal PaymentError enum (Db, StatusPrecondition, NotFound)          | VERIFIED    | Exactly 3 variants present. No `Stripe`, `Loader`, `AutoRefundTriggered`.               |
| `ferro-payments/src/intent/status.rs`                                       | PaymentIntentStatus DeriveActiveEnum, 5 variants, TEXT-backed         | VERIFIED    | `DeriveActiveEnum`, `db_type="Text"`, `rs_type="String"`, variants: Reserved/Paid/Released/Failed/Refunded. |
| `ferro-payments/src/intent/entity.rs`                                       | DeriveEntityModel for payment_intents (all 19 columns, correct types) | VERIFIED    | `table_name="payment_intents"`, all 19 fields at correct types and nullability, no FK relations. |
| `ferro-payments/src/migration/m20260617_create_payment_intents.rs`          | Cross-backend migration with partial unique + supporting indexes + tests | VERIFIED  | `execute_unprepared`, `get_database_backend`, `WHERE status IN ('reserved','paid')`, MySQL `CAST(billable_id AS CHAR) STORED`, manual `MigrationName`, 4 inline tests. |
| `ferro-payments/src/migration/mod.rs`                                       | Migration module + public re-export                                   | VERIFIED    | `pub use … Migration as CreatePaymentIntentsTable`, `migration_create_payment_intents()` constructor. |
| `ferro-payments/src/intent/lifecycle.rs`                                    | All 6 lifecycle fns + inline tests for state transitions              | VERIFIED    | 6 fns implemented. 6 test fns covering all `<behavior>` bullets. `exec_at_most_one` used throughout. |

---

### Key Link Verification

| From                                        | To                                          | Via                                              | Status   | Details                                                                                 |
|---------------------------------------------|---------------------------------------------|--------------------------------------------------|----------|-----------------------------------------------------------------------------------------|
| Workspace `Cargo.toml`                      | `ferro-payments`                            | `members` array entry `"ferro-payments"`         | WIRED    | Line 37 of workspace Cargo.toml confirmed.                                              |
| `.github/workflows/publish.yml`             | `ferro-payments`                            | `WAVE1B_CRATES` list, after `ferro-reservation`  | WIRED    | Line 247: `ferro-reservation ferro-payments` confirmed.                                 |
| Migration                                   | Partial unique index                        | `execute_unprepared` + `get_database_backend`    | WIRED    | Branch at line 161–185; `WHERE status IN ('reserved','paid')` on PG/SQLite path, stored generated column on MySQL path. |
| `entity.rs`                                 | `PaymentIntentStatus`                       | `status: PaymentIntentStatus` field              | WIRED    | `entity.rs` line 30: `pub status: PaymentIntentStatus`. Imported from `crate::intent::status`. |
| `lifecycle.rs`                              | `ferro_orm::GuardedUpdate`                  | `GuardedUpdate::new(Entity).exec_at_most_one`    | WIRED    | Lines 69, 91, 113 in lifecycle.rs. Imported at line 9: `use ferro_orm::{GuardedUpdate, Value}`. |
| `lifecycle.rs`                              | `PaymentIntentStatus` (is_in filter)        | `.filter(Column::Status.is_in([Reserved, Paid]))` | WIRED  | Line 143 in lifecycle.rs.                                                               |

---

### Data-Flow Trace (Level 4)

Not applicable — this phase contains no rendering/UI artifacts. All artifacts are data-layer (entity, migration, lifecycle functions). Tests exercise data flow against in-memory SQLite; 11/11 pass per SUMMARY-03.

---

### Behavioral Spot-Checks

Executor recorded evidence:

| Behavior                                      | Evidence                                                    | Status   |
|-----------------------------------------------|-------------------------------------------------------------|----------|
| `cargo build -p ferro-payments` exits 0       | SUMMARY-01, SUMMARY-02, SUMMARY-03 all record exit 0       | PASS     |
| `cargo test -p ferro-payments` 11 tests green | SUMMARY-03: `cargo test -p ferro-payments` 11 passed, 0 failed | PASS  |
| `cargo clippy --all --all-targets -D warnings` exit 0 | SUMMARY-03 records exit 0 (no warnings)            | PASS     |
| `cargo fmt --all -- --check` exit 0           | SUMMARY-03 records exit 0                                   | PASS     |
| Commits 44e6d232, caaee82c, 00bf2891, 22d76410, e54e2522 exist | `git log --oneline` confirms all 5 hashes  | PASS     |

Note: Full-workspace `cargo test --all-features` not run due to documented disk constraint (ENOSPC risk). Crate-scoped evidence is sufficient for a leaf crate with no workspace cross-dependencies in its test scope.

---

### Requirements Coverage

Requirements are defined in the design spec (`docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md`) and ROADMAP, not in REQUIREMENTS.md.

| Requirement    | Source Plan | Description                                                              | Status      | Evidence                                              |
|----------------|-------------|--------------------------------------------------------------------------|-------------|-------------------------------------------------------|
| PAY-POLY-DM-01 | 233-02      | `payment_intents` entity + cross-backend migration with partial index    | SATISFIED   | `entity.rs` (19 cols) + migration (partial index + 3 supporting) confirmed. |
| PAY-POLY-DM-02 | 233-01      | `PaymentIntentStatus` DeriveActiveEnum, TEXT, 5 variants, round-trip     | SATISFIED   | `status.rs` confirmed; round-trip test green.         |
| PAY-POLY-DM-03 | 233-03      | Six lifecycle methods via GuardedUpdate, no-op semantics, tested         | SATISFIED   | `lifecycle.rs` with 6 fns + 6 test functions; all 11 tests green. |
| PAY-POLY-DM-04 | 233-02      | Migration portable across PG/SQLite/MySQL; partial unique enforcement proven | SATISFIED | Cross-backend branch confirmed; 2 partial-unique tests green. |

---

### Anti-Patterns Found

Static scan of all ferro-payments source files:

| File                         | Finding                                                                              | Severity | Impact                                                                                 |
|------------------------------|--------------------------------------------------------------------------------------|----------|----------------------------------------------------------------------------------------|
| `lifecycle.rs` line 203-209  | `seed_with_status` uses string interpolation into `execute_unprepared` (test-only)   | Info     | Test code only (`#[cfg(test)]`); caller supplies only 5 known status literals — no real injection surface. WR-03 from code review. Pattern inconsistency, not a production risk. |
| `lifecycle.rs` lines 74,96,118 | Raw string literals `"paid"`, `"released"`, `"refunded"` in `set_value` calls instead of `enum.to_value()` | Info | Future sync risk if enum string values change. Not a correctness bug today. IN-01 from code review. |
| `lifecycle.rs` line 111-127  | `mark_refunded` does not set `refund_amount_cents`                                   | Info     | Intentional deferral — `request_refund` (Phase 234) fills this from the Stripe API per design spec. See WR-02 assessment below. |

No blocker or warning anti-patterns found. All three findings are Info-level items.

---

### Review Findings Assessment (WR-02 and WR-04)

**WR-02 — `mark_refunded` never sets `refund_amount_cents`:**
Defensible deferral. The design spec explicitly assigns refund amount capture to `request_refund` in Phase 234: "calls the Stripe refund API and writes the refund_amount snapshot. Webhook `charge.refunded` arrives later and flips status to `refunded`." The `mark_refunded` function's role in Phase 233 is the atomic status transition only; the amount is a Stripe-side concern resolved in Phase 234. Not a gap against Phase 233's stated goal (data layer only, no service layer yet).

**WR-04 — `find_active_for` does not filter by `tenant_id`:**
Consistent with explicit phase design decisions. CONTEXT.md D-11 specifies: "`find_active_for(kind, id)` filters `status IN ('reserved','paid')`" — tenant_id is not in the D-11 specification. T-233-08 explicitly accepts: "Pure data-layer functions, no auth/tenant enforcement in this crate (tenant_id is a plain column; scoping is the consumer's concern per design)." The composite index `idx_payment_intents_tenant_status` exists for consumer-side use. The review correctly flags this as a risk for multi-tenant consumers; it is noted for Phase 234's `PaymentService<L>` to address at the service layer. Not a gap against Phase 233's stated goal.

---

### Human Verification Required

None. All must-haves are verifiable programmatically against the codebase. The crate is a pure data layer (no UI, no external service calls in phase 233 scope).

---

### Gaps Summary

No gaps. All 10 observable truths verified against the actual codebase. All artifacts exist at the required substantive level. All key links are wired. The 3 anti-pattern findings are Info-level items (two style/consistency notes from the code review, one intentional deferral to Phase 234). The phase successfully delivers its stated data-layer goal.

---

_Verified: 2026-06-17T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
