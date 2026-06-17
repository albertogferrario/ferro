---
phase: 233-ferro-payments-crate-polymorphic-billable
reviewed: 2026-06-17T00:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - ferro-payments/src/lib.rs
  - ferro-payments/src/error.rs
  - ferro-payments/src/intent/mod.rs
  - ferro-payments/src/intent/status.rs
  - ferro-payments/src/intent/entity.rs
  - ferro-payments/src/intent/lifecycle.rs
  - ferro-payments/src/migration/mod.rs
  - ferro-payments/src/migration/m20260617_create_payment_intents.rs
  - ferro-payments/Cargo.toml
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 233: Code Review Report

**Reviewed:** 2026-06-17
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

`ferro-payments` is a new pure data-layer crate implementing a polymorphic
`payment_intents` table with a SeaORM entity, a five-variant TEXT-backed
status enum, atomic GuardedUpdate lifecycle transitions, a cross-backend
migration (Postgres/SQLite partial unique index + MySQL generated-column
emulation), and a suite of integration tests against an in-memory SQLite
database.

The SQL injection surface is clean — all `execute_unprepared` strings are
static literals. The GuardedUpdate usage is correct and the no-op semantics
are properly tested. Four warnings are raised: one substantive data-integrity
gap (the `stripe_session_id` unique index allows multiple NULL rows on Postgres
and SQLite but the entity marks it Optional, which means every un-associated
row competes for that slot after phase 234 sets the value), one missing column
update in `mark_refunded` (`refund_amount_cents` is never set by any lifecycle
function), one test-code SQL injection in `seed_with_status`, and one missing
`mark_paid` parameter for `payment_intent_id`/`stripe_session_id` which are
not in scope for this phase but will become a correctness gap once phase 234
wires Stripe. Three info items cover minor idiom observations.

## Warnings

### WR-01: `stripe_session_id` unique index permits multiple NULLs — intended, but not documented

**File:** `ferro-payments/src/migration/m20260617_create_payment_intents.rs:124-135`
**Issue:** The migration creates a plain `UNIQUE` index on `stripe_session_id`
without a `WHERE stripe_session_id IS NOT NULL` filter. On Postgres and SQLite
a unique index does not deduplicate NULLs (each NULL is considered distinct),
so multiple rows with `stripe_session_id = NULL` are freely allowed — this is
almost certainly correct behaviour for the pre-payment `reserved` state. On
MySQL `UNIQUE` also allows multiple NULLs, so all three backends agree.
However, the comment at line 125 reads "each Stripe session maps to exactly
one payment intent", which implies the index enforces a 1:1 Stripe-session →
row mapping. That invariant only holds for non-NULL values. If the intent is
that the uniqueness guarantee is over non-NULL session IDs only, the comment
should say so; if a partial/filtered unique index over non-NULL values is
desired (to make the DB schema self-documenting), that can be added the same
way as `uq_payment_intents_active`.

**Fix:** Clarify the comment to state explicitly that the uniqueness applies
only to non-NULL values:
```rust
// Unique index on stripe_session_id — each non-NULL Stripe session ID maps to
// exactly one payment intent row. NULL is allowed for rows where no session
// has been created yet; NULLs are never deduplicated by this index.
```
If stricter schema documentation is desired, convert to a partial index on
Postgres/SQLite (mirroring the active-row index pattern already present):
```sql
-- Postgres / SQLite partial form (optional but self-documenting):
CREATE UNIQUE INDEX idx_payment_intents_stripe_session_id
    ON payment_intents (stripe_session_id)
    WHERE stripe_session_id IS NOT NULL;
```

---

### WR-02: `mark_refunded` never sets `refund_amount_cents`

**File:** `ferro-payments/src/intent/lifecycle.rs:111-127`
**Issue:** `mark_refunded` transitions `paid → refunded` and records
`refunded_at`, but the `refund_amount_cents` column — explicitly documented in
the entity as "may differ from `amount_cents` for partial refunds" — is never
written by any lifecycle function. The column is `Option<i64>` so it stays
`NULL` after every refund. Callers cannot pass a refund amount, and the column
can never reach a non-NULL state through the public API.

This is either a missing parameter or a deliberate decision (e.g. phase 235
fills it from the Stripe refund event). If deliberate, the function signature
and module-level doc should say so. If unintentional, the fix is:

**Fix:**
```rust
pub async fn mark_refunded<C: ConnectionTrait>(
    id: i64,
    refund_amount_cents: i64,   // add parameter
    conn: &C,
) -> Result<bool, PaymentError> {
    let now = Utc::now();
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::Status.eq(PaymentIntentStatus::Paid))
        .set_value(
            Column::Status,
            Value::String(Some(Box::new("refunded".to_string()))),
        )
        .set_value(
            Column::RefundedAt,
            Value::ChronoDateTimeUtc(Some(Box::new(now))),
        )
        .set_value(
            Column::RefundAmountCents,
            Value::BigInt(Some(refund_amount_cents)),
        )
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}
```
If the value is intentionally deferred to a Stripe webhook phase, add a
`// Phase 235: refund_amount_cents filled by webhook handler` comment.

---

### WR-03: `seed_with_status` uses string interpolation into `execute_unprepared` (test-only SQL injection)

**File:** `ferro-payments/src/intent/lifecycle.rs:202-209`
**Issue:** Inside `#[cfg(test)]`, `seed_with_status` formats the `status`
parameter directly into a raw SQL string via `&format!(...)`:
```rust
conn.execute_unprepared(&format!(
    "INSERT INTO payment_intents ... VALUES (1,'booking',99,500,'USD','{status}',...)"
))
```
In test code the caller supplies only the five known status literals
(`"reserved"`, `"paid"`, etc.), so there is no real injection risk. However
the pattern is inconsistent with the crate's security stance on
`execute_unprepared` (module docstring explicitly calls out that production DDL
strings are static). A future test author might copy the pattern with a
user-supplied value. The test is also coupled to SQLite's non-standard
`last_insert_rowid()` query on line 215-222, making it backend-specific.

**Fix:** Replace interpolation with a parameterised `Statement::from_sql_and_values`:
```rust
use sea_orm::{DatabaseBackend, Statement, Value};

let result = conn
    .execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO payment_intents \
         (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
          expires_at,reserved_at) \
         VALUES (1,'booking',99,500,'USD',?,\
         '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')",
        [Value::String(Some(Box::new(status.to_string())))],
    ))
    .await
    .expect("seed row");
```

---

### WR-04: `find_active_for` does not filter by `tenant_id`

**File:** `ferro-payments/src/intent/lifecycle.rs:135-147`
**Issue:** `find_active_for` takes `(kind, billable_id, conn)` but omits
`tenant_id` from its filter. In a multi-tenant deployment two tenants could
theoretically share the same `billable_id` for the same `billable_kind` (e.g.
tenant A and tenant B both have an `order` with `id = 42`). The function
would then return either tenant's row to whichever caller reaches it first,
depending on insertion order and no deterministic sort. The composite index
`idx_payment_intents_tenant_status` shows tenant scoping is expected.

**Fix:** Add `tenant_id` as a parameter and include it in the filter:
```rust
pub async fn find_active_for<C: ConnectionTrait>(
    tenant_id: i64,
    kind: &str,
    billable_id: i64,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::TenantId.eq(tenant_id))
        .filter(Column::BillableKind.eq(kind))
        .filter(Column::BillableId.eq(billable_id))
        .filter(Column::Status.is_in([PaymentIntentStatus::Reserved, PaymentIntentStatus::Paid]))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}
```
The corresponding public re-export in `lib.rs` and the test in
`lifecycle.rs:326-351` must be updated to pass `tenant_id`.

---

## Info

### IN-01: `mark_paid`, `mark_released`, `mark_refunded` use raw string literals for status values instead of the enum's `to_value()`

**File:** `ferro-payments/src/intent/lifecycle.rs:74,96,118`
**Issue:** The `set_value` calls for `Column::Status` pass `Value::String(Some(Box::new("paid".to_string())))` (and `"released"`, `"refunded"`) as raw string literals. These must stay in sync with `PaymentIntentStatus`'s `#[sea_orm(string_value = "...")]` annotations. If a string value is ever renamed in the enum, the lifecycle transitions silently write an unrecognised status string that the ORM will then fail to deserialise on the next read.

**Fix:** Use the enum's `ActiveEnum::to_value()` to derive the string:
```rust
use sea_orm::ActiveEnum;

.set_value(
    Column::Status,
    Value::String(Some(Box::new(
        PaymentIntentStatus::Paid.to_value(),
    ))),
)
```
This makes the lifecycle strings and enum declarations a single source of truth.

---

### IN-02: `BillableKind` is not used by any lifecycle function — its ergonomic value is unclear

**File:** `ferro-payments/src/lib.rs:18-29`
**Issue:** `BillableKind` wraps a `&'static str` and is exported from
`lib.rs`, but none of the lifecycle functions (`create_reserved`,
`find_active_for`, etc.) accept a `BillableKind` — they all take `&str`
directly. The type exists in the public API but has no use site within the
crate. Consumers receive no type-safety benefit from it because they can pass
any `&str` to the lifecycle functions without going through `BillableKind`.

**Fix:** Either thread `BillableKind` through the lifecycle API (replacing the
`billable_kind: &str` parameters with `kind: &BillableKind` and calling
`kind.as_str()` internally), or document explicitly that `BillableKind` is a
user-side constant-naming convention only and the crate deliberately accepts
`&str` for flexibility. In the latter case, move `BillableKind` behind a clear
module path (e.g. `ferro_payments::kind::BillableKind`) so the API signals its
optional, convention-only nature.

---

### IN-03: `migration/mod.rs` exports `migration_create_payment_intents()` function alongside the `CreatePaymentIntentsTable` re-export — duplicate surface

**File:** `ferro-payments/src/migration/mod.rs:13-15`
**Issue:** The module exposes two ways to obtain the same migration:
`CreatePaymentIntentsTable` (a type alias usable as `Box::new(CreatePaymentIntentsTable)`)
and `migration_create_payment_intents()` (a function returning the boxed
trait object). Both are re-exported from `lib.rs`. Having two paths for the
same thing means `lib.rs` line 14 (`pub use migration::CreatePaymentIntentsTable`)
and the function on line 13 of `migration/mod.rs` can drift. Pick one
convention and remove the other.

**Fix:** The `pub use … as CreatePaymentIntentsTable` type re-export is the
simpler pattern (callers wrap it themselves). Remove `migration_create_payment_intents()`
to keep the public surface minimal:
```rust
// migration/mod.rs — keep only:
pub(crate) mod m20260617_create_payment_intents;
pub use m20260617_create_payment_intents::Migration as CreatePaymentIntentsTable;
```

---

_Reviewed: 2026-06-17_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
