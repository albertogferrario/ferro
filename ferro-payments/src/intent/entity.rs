//! SeaORM entity for the `payment_intents` table.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::intent::status::PaymentIntentStatus;

/// A single polymorphic payment intent, linked to any billable entity via
/// `(billable_kind, billable_id)`. Enforces at most one active row per
/// billable through a partial unique index on those columns filtered to
/// `status IN ('reserved','paid')`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "payment_intents")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Consumer-side tenant identifier. No FK — consumers add their own constraints.
    pub tenant_id: i64,
    /// Discriminator string for the billable entity type (e.g. `"order"`, `"booking"`).
    /// Raw TEXT; the crate never enumerates kinds.
    pub billable_kind: String,
    /// Primary key of the billable entity in the consumer's table. No FK.
    pub billable_id: i64,
    /// Charge amount in the smallest currency unit (e.g. cents for EUR/USD).
    pub amount_cents: i64,
    /// ISO 4217 currency code (e.g. `"EUR"`, `"USD"`).
    pub currency: String,
    /// Lifecycle status. Stored as TEXT; enforces the partial unique index invariant.
    pub status: PaymentIntentStatus,
    /// Stripe Checkout Session ID (set on `create_reserved`; unique across rows).
    pub stripe_session_id: Option<String>,
    /// Stripe PaymentIntent ID (set on `mark_paid`).
    pub payment_intent_id: Option<String>,
    /// Stripe Charge ID (set on `mark_paid` when available).
    pub charge_id: Option<String>,
    /// Connect destination charge application fee in the smallest currency unit.
    pub application_fee_cents: Option<i64>,
    /// Reservation expiry timestamp. Set in Rust at `create_reserved` time.
    pub expires_at: DateTimeUtc,
    /// Timestamp when the reservation was created.
    pub reserved_at: DateTimeUtc,
    /// Timestamp when the payment was confirmed. Set on `mark_paid`.
    pub paid_at: Option<DateTimeUtc>,
    /// Timestamp when the reservation was released. Set on `mark_released`.
    pub released_at: Option<DateTimeUtc>,
    /// Timestamp when the charge was refunded. Set on `mark_refunded`.
    pub refunded_at: Option<DateTimeUtc>,
    /// Actual refund amount in the smallest currency unit (may differ from `amount_cents`
    /// for partial refunds).
    pub refund_amount_cents: Option<i64>,
    /// Free-form JSON metadata. Convention: no PII. No column-level enforcement.
    pub metadata: Option<JsonValue>,
}

/// No FK relations — `tenant_id` and `billable_id` reference consumer tables unknown
/// to this crate. Consumers add their own FK constraints at the migration level.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
