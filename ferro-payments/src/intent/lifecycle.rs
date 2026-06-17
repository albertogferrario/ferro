//! Lifecycle functions for `payment_intents` rows.
//!
//! All state-transition functions (`mark_*`) use `ferro_orm::GuardedUpdate`
//! — a single atomic `UPDATE … WHERE …` statement. A `0`-rows-affected result
//! is a **no-op**, not an error: the stale-precondition caller simply returns
//! `Ok(false)` (D-09 "second writer no-ops" race semantics).

use chrono::Utc;
use ferro_orm::{GuardedUpdate, Value};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::error::PaymentError;
use crate::intent::entity::{self, Column, Entity};
use crate::intent::status::PaymentIntentStatus;

// ---------------------------------------------------------------------------
// INSERT
// ---------------------------------------------------------------------------

/// Insert a new `payment_intents` row with `status = reserved`.
///
/// `reserved_at` is set to `Utc::now()` in Rust (D-06 — no DB default).
/// If an active row for `(billable_kind, billable_id)` already exists, the
/// partial unique index from the migration raises a DB error (D-10).
#[allow(clippy::too_many_arguments)]
pub async fn create_reserved<C: ConnectionTrait>(
    tenant_id: i64,
    billable_kind: &str,
    billable_id: i64,
    amount_cents: i64,
    currency: &str,
    expires_at: chrono::DateTime<Utc>,
    conn: &C,
) -> Result<entity::Model, PaymentError> {
    let now = Utc::now();
    let row = entity::ActiveModel {
        tenant_id: Set(tenant_id),
        billable_kind: Set(billable_kind.to_string()),
        billable_id: Set(billable_id),
        amount_cents: Set(amount_cents),
        currency: Set(currency.to_string()),
        status: Set(PaymentIntentStatus::Reserved),
        stripe_session_id: Set(None),
        payment_intent_id: Set(None),
        charge_id: Set(None),
        application_fee_cents: Set(None),
        expires_at: Set(expires_at),
        reserved_at: Set(now),
        paid_at: Set(None),
        released_at: Set(None),
        refunded_at: Set(None),
        refund_amount_cents: Set(None),
        metadata: Set(None),
        ..Default::default()
    };
    row.insert(conn).await.map_err(PaymentError::Db)
}

// ---------------------------------------------------------------------------
// Atomic state transitions (GuardedUpdate / no-op semantics)
// ---------------------------------------------------------------------------

/// Transition `status: reserved → paid` and record `paid_at = now`.
///
/// Returns `Ok(true)` on success, `Ok(false)` when the row was not in the
/// `reserved` state (no-op — D-09).
pub async fn mark_paid<C: ConnectionTrait>(id: i64, conn: &C) -> Result<bool, PaymentError> {
    let now = Utc::now();
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::Status.eq(PaymentIntentStatus::Reserved))
        .set_value(
            Column::Status,
            Value::String(Some(Box::new("paid".to_string()))),
        )
        .set_value(
            Column::PaidAt,
            Value::ChronoDateTimeUtc(Some(Box::new(now))),
        )
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}

/// Transition `status: reserved → released` and record `released_at = now`.
///
/// Returns `Ok(true)` on success, `Ok(false)` when the row was not in the
/// `reserved` state (no-op — D-09).
pub async fn mark_released<C: ConnectionTrait>(id: i64, conn: &C) -> Result<bool, PaymentError> {
    let now = Utc::now();
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::Status.eq(PaymentIntentStatus::Reserved))
        .set_value(
            Column::Status,
            Value::String(Some(Box::new("released".to_string()))),
        )
        .set_value(
            Column::ReleasedAt,
            Value::ChronoDateTimeUtc(Some(Box::new(now))),
        )
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}

/// Transition `status: paid → refunded` and record `refunded_at = now`.
///
/// Returns `Ok(true)` on success, `Ok(false)` when the row was not in the
/// `paid` state (no-op — D-09).
pub async fn mark_refunded<C: ConnectionTrait>(id: i64, conn: &C) -> Result<bool, PaymentError> {
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
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}

/// Attach `stripe_session_id` and snapshot `application_fee_cents` onto a reserved
/// row after a successful Stripe Checkout session creation.
///
/// Guard: `WHERE stripe_session_id IS NULL` — idempotent for retries. Returns
/// `Ok(true)` when the session was attached, `Ok(false)` when a session was already
/// attached (the guard excluded the row — do not overwrite).
pub async fn attach_session<C: ConnectionTrait>(
    id: i64,
    stripe_session_id: &str,
    application_fee_cents: Option<i64>,
    conn: &C,
) -> Result<bool, PaymentError> {
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::StripeSessionId.is_null())
        .set_value(
            Column::StripeSessionId,
            Value::String(Some(Box::new(stripe_session_id.to_string()))),
        )
        .set_value(
            Column::ApplicationFeeCents,
            match application_fee_cents {
                Some(f) => Value::BigInt(Some(f)),
                None => Value::BigInt(None),
            },
        )
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Return the active `payment_intents` row for `(billable_kind, billable_id)`,
/// if one exists. Active means `status IN ('reserved', 'paid')` (D-11).
pub async fn find_active_for<C: ConnectionTrait>(
    kind: &str,
    billable_id: i64,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::BillableKind.eq(kind))
        .filter(Column::BillableId.eq(billable_id))
        .filter(Column::Status.is_in([PaymentIntentStatus::Reserved, PaymentIntentStatus::Paid]))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}

/// Return the `payment_intents` row whose `stripe_session_id` matches
/// `session_id`, or `None` if no such row exists (D-11).
pub async fn find_by_stripe_session<C: ConnectionTrait>(
    session_id: &str,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::StripeSessionId.eq(session_id))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}

/// Return the `payment_intents` row whose `payment_intent_id` matches,
/// or `None` if absent.
///
/// Primary lookup for `handle_charge_refunded` — the refund event carries
/// no `session_id`.
pub async fn find_by_payment_intent<C: ConnectionTrait>(
    payment_intent_id: &str,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::PaymentIntentId.eq(payment_intent_id))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}

/// Return the `payment_intents` row whose `charge_id` matches,
/// or `None` if absent.
///
/// Fallback lookup for `handle_charge_refunded` when `payment_intent_id`
/// is absent from the event.
pub async fn find_by_charge_id<C: ConnectionTrait>(
    charge_id: &str,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::ChargeId.eq(charge_id))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}

/// Persist `payment_intent_id` onto the row after marking paid.
/// Guard: `WHERE payment_intent_id IS NULL` — idempotent for Stripe retries.
///
/// Returns `Ok(true)` when written, `Ok(false)` when already set (no-op).
pub async fn attach_payment_intent<C: ConnectionTrait>(
    id: i64,
    payment_intent_id: &str,
    conn: &C,
) -> Result<bool, PaymentError> {
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::PaymentIntentId.is_null())
        .set_value(
            Column::PaymentIntentId,
            Value::String(Some(Box::new(payment_intent_id.to_string()))),
        )
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database};
    use sea_orm_migration::MigratorTrait;

    use crate::migration::m20260617_create_payment_intents::Migration as CreateTable;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(CreateTable)]
        }
    }

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");
        TestMigrator::up(&conn, None).await.expect("migrate up");
        conn
    }

    /// Seed a reserved row directly via `create_reserved` and return it.
    async fn seed_reserved(conn: &sea_orm::DatabaseConnection) -> entity::Model {
        let expires_at = chrono::DateTime::parse_from_rfc3339("2030-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        create_reserved(1, "order", 42, 1000, "EUR", expires_at, conn)
            .await
            .expect("create_reserved")
    }

    /// Seed a row with an arbitrary status via raw SQL (bypasses lifecycle guards).
    async fn seed_with_status(conn: &sea_orm::DatabaseConnection, status: &str) -> i64 {
        conn.execute_unprepared(&format!(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'booking',99,500,'USD','{status}',\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')"
        ))
        .await
        .expect("seed row");

        // Return the last inserted id
        let row = conn
            .query_one(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT last_insert_rowid() AS id".to_string(),
            ))
            .await
            .expect("query")
            .expect("row");
        row.try_get::<i64>("", "id").expect("id")
    }

    #[tokio::test]
    async fn create_reserved_inserts_reserved_row() {
        let conn = fresh_db().await;
        let model = seed_reserved(&conn).await;

        assert_eq!(model.billable_kind, "order");
        assert_eq!(model.billable_id, 42);
        assert_eq!(model.amount_cents, 1000);
        assert_eq!(model.currency, "EUR");
        assert_eq!(model.status, PaymentIntentStatus::Reserved);
        assert!(model.paid_at.is_none());
        assert!(model.released_at.is_none());
        assert!(model.refunded_at.is_none());
    }

    #[tokio::test]
    async fn mark_paid_transitions_reserved_to_paid() {
        let conn = fresh_db().await;
        let model = seed_reserved(&conn).await;

        let updated = mark_paid(model.id, &conn).await.expect("mark_paid");
        assert!(updated, "mark_paid on a reserved row must return Ok(true)");

        let reloaded = Entity::find_by_id(model.id)
            .one(&conn)
            .await
            .unwrap()
            .expect("row still exists");
        assert_eq!(reloaded.status, PaymentIntentStatus::Paid);
        assert!(
            reloaded.paid_at.is_some(),
            "paid_at must be set after mark_paid"
        );
    }

    #[tokio::test]
    async fn mark_paid_noop_on_wrong_status() {
        let conn = fresh_db().await;
        let model = seed_reserved(&conn).await;

        // First call succeeds
        mark_paid(model.id, &conn).await.expect("first mark_paid");

        // Second call on an already-paid row must no-op
        let noop = mark_paid(model.id, &conn)
            .await
            .expect("second mark_paid must not err");
        assert!(
            !noop,
            "mark_paid on a non-reserved row must return Ok(false)"
        );

        // Status must still be Paid (not changed twice)
        let reloaded = Entity::find_by_id(model.id)
            .one(&conn)
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(reloaded.status, PaymentIntentStatus::Paid);
    }

    #[tokio::test]
    async fn mark_released_and_mark_refunded_guards() {
        let conn = fresh_db().await;

        // --- mark_released: reserved → released (happy path) ---
        let m1 = seed_reserved(&conn).await;
        let ok = mark_released(m1.id, &conn).await.expect("mark_released");
        assert!(ok, "mark_released on reserved must return Ok(true)");
        let r1 = Entity::find_by_id(m1.id).one(&conn).await.unwrap().unwrap();
        assert_eq!(r1.status, PaymentIntentStatus::Released);
        assert!(r1.released_at.is_some());

        // mark_released again is a no-op (source status is now released, not reserved)
        let noop = mark_released(m1.id, &conn)
            .await
            .expect("second mark_released");
        assert!(!noop, "mark_released on non-reserved must return Ok(false)");

        // --- mark_refunded: paid → refunded (happy path) ---
        // Seed a paid row via raw SQL to bypass mark_paid
        let paid_id = seed_with_status(&conn, "paid").await;
        let ok2 = mark_refunded(paid_id, &conn).await.expect("mark_refunded");
        assert!(ok2, "mark_refunded on paid must return Ok(true)");
        let r2 = Entity::find_by_id(paid_id)
            .one(&conn)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(r2.status, PaymentIntentStatus::Refunded);
        assert!(r2.released_at.is_none());
        assert!(r2.refunded_at.is_some());

        // mark_refunded on a reserved row is a no-op (wrong source status)
        let m2 = seed_reserved(&conn).await;
        let noop2 = mark_refunded(m2.id, &conn)
            .await
            .expect("mark_refunded on reserved");
        assert!(!noop2, "mark_refunded on non-paid must return Ok(false)");
    }

    #[tokio::test]
    async fn find_active_for_excludes_terminal_rows() {
        let conn = fresh_db().await;

        // A released row must not appear
        let released_id = seed_with_status(&conn, "released").await;
        let _ = released_id; // id not needed for assertions
        let none = find_active_for("booking", 99, &conn)
            .await
            .expect("find_active_for");
        assert!(
            none.is_none(),
            "released row must not be returned by find_active_for"
        );

        // A reserved row must appear
        let model = seed_reserved(&conn).await;
        let some = find_active_for("order", 42, &conn)
            .await
            .expect("find_active_for");
        assert!(
            some.is_some(),
            "reserved row must be returned by find_active_for"
        );
        assert_eq!(some.unwrap().id, model.id);
    }

    #[tokio::test]
    async fn attach_session_sets_session_and_fee() {
        let conn = fresh_db().await;
        let model = seed_reserved(&conn).await;

        let ok = attach_session(model.id, "cs_test", Some(50), &conn)
            .await
            .expect("attach_session");
        assert!(ok, "attach_session on a reserved row must return Ok(true)");

        let reloaded = Entity::find_by_id(model.id)
            .one(&conn)
            .await
            .unwrap()
            .expect("row still exists");
        assert_eq!(reloaded.stripe_session_id, Some("cs_test".to_string()));
        assert_eq!(reloaded.application_fee_cents, Some(50));
    }

    #[tokio::test]
    async fn attach_session_idempotent_second_call_noops() {
        let conn = fresh_db().await;
        let model = seed_reserved(&conn).await;

        // First attach succeeds
        attach_session(model.id, "cs_first", Some(100), &conn)
            .await
            .expect("first attach_session");

        // Second attach is a no-op (StripeSessionId IS NULL guard excludes the row)
        let noop = attach_session(model.id, "cs_other", None, &conn)
            .await
            .expect("second attach_session must not err");
        assert!(
            !noop,
            "attach_session on a row with session already attached must return Ok(false)"
        );

        // Original session_id must not have been overwritten
        let reloaded = Entity::find_by_id(model.id)
            .one(&conn)
            .await
            .unwrap()
            .expect("row still exists");
        assert_eq!(reloaded.stripe_session_id, Some("cs_first".to_string()));
        assert_eq!(reloaded.application_fee_cents, Some(100));
    }

    #[tokio::test]
    async fn find_by_payment_intent_matches() {
        let conn = fresh_db().await;
        let row = seed_reserved(&conn).await;
        mark_paid(row.id, &conn).await.unwrap();
        assert!(
            attach_payment_intent(row.id, "pi_abc", &conn)
                .await
                .unwrap(),
            "first attach must return Ok(true)"
        );

        let found = find_by_payment_intent("pi_abc", &conn)
            .await
            .expect("find_by_payment_intent");
        assert_eq!(
            found.map(|r| r.id),
            Some(row.id),
            "must find the row by payment_intent_id"
        );

        let miss = find_by_payment_intent("pi_missing", &conn)
            .await
            .expect("find_by_payment_intent miss");
        assert!(miss.is_none(), "non-matching id must return None");
    }

    #[tokio::test]
    async fn find_by_charge_id_matches() {
        let conn = fresh_db().await;
        let expires_at = chrono::DateTime::parse_from_rfc3339("2030-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let now = Utc::now();
        // Insert a row with a charge_id set directly.
        let inserted = entity::ActiveModel {
            tenant_id: Set(1),
            billable_kind: Set("order".to_string()),
            billable_id: Set(55),
            amount_cents: Set(500),
            currency: Set("EUR".to_string()),
            status: Set(PaymentIntentStatus::Paid),
            stripe_session_id: Set(None),
            payment_intent_id: Set(None),
            charge_id: Set(Some("ch_testcharge".to_string())),
            application_fee_cents: Set(None),
            expires_at: Set(expires_at),
            reserved_at: Set(now),
            paid_at: Set(Some(now)),
            released_at: Set(None),
            refunded_at: Set(None),
            refund_amount_cents: Set(None),
            metadata: Set(None),
            ..Default::default()
        }
        .insert(&conn)
        .await
        .expect("insert with charge_id");

        let found = find_by_charge_id("ch_testcharge", &conn)
            .await
            .expect("find_by_charge_id");
        assert!(found.is_some(), "must find row by charge_id");
        assert_eq!(found.unwrap().id, inserted.id);

        let miss = find_by_charge_id("ch_notexist", &conn)
            .await
            .expect("find_by_charge_id miss");
        assert!(miss.is_none(), "non-matching charge_id must return None");
    }

    #[tokio::test]
    async fn attach_payment_intent_idempotent_second_call_noops() {
        let conn = fresh_db().await;
        let row = seed_reserved(&conn).await;
        mark_paid(row.id, &conn).await.unwrap();

        // First call: written.
        assert!(
            attach_payment_intent(row.id, "pi_x", &conn).await.unwrap(),
            "first attach_payment_intent must return Ok(true)"
        );

        // Second call: IS NULL guard → no-op.
        assert!(
            !attach_payment_intent(row.id, "pi_y", &conn).await.unwrap(),
            "second attach_payment_intent must return Ok(false) (IS NULL guard)"
        );

        // The first value must still be present.
        let found = find_by_payment_intent("pi_x", &conn)
            .await
            .expect("find after idempotent double attach");
        assert!(
            found.is_some(),
            "original payment_intent_id must not be overwritten"
        );
    }

    #[tokio::test]
    async fn find_by_stripe_session_matches() {
        let conn = fresh_db().await;

        // Insert a row with a stripe_session_id via direct ActiveModel
        let expires_at = chrono::DateTime::parse_from_rfc3339("2030-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let now = Utc::now();
        let inserted = entity::ActiveModel {
            tenant_id: Set(1),
            billable_kind: Set("order".to_string()),
            billable_id: Set(77),
            amount_cents: Set(2000),
            currency: Set("EUR".to_string()),
            status: Set(PaymentIntentStatus::Reserved),
            stripe_session_id: Set(Some("cs_test_abc123".to_string())),
            payment_intent_id: Set(None),
            charge_id: Set(None),
            application_fee_cents: Set(None),
            expires_at: Set(expires_at),
            reserved_at: Set(now),
            paid_at: Set(None),
            released_at: Set(None),
            refunded_at: Set(None),
            refund_amount_cents: Set(None),
            metadata: Set(None),
            ..Default::default()
        }
        .insert(&conn)
        .await
        .expect("insert with session id");

        let found = find_by_stripe_session("cs_test_abc123", &conn)
            .await
            .expect("find_by_stripe_session");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, inserted.id);

        let miss = find_by_stripe_session("cs_test_nonexistent", &conn)
            .await
            .expect("find_by_stripe_session miss");
        assert!(miss.is_none());
    }
}
