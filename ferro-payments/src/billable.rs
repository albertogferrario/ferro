//! The `Billable` trait — domain entities expose their amount, currency, line
//! description, and per-status side effects to the payment layer without coupling
//! the payment layer to any concrete table.

use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

use crate::error::PaymentError;
use crate::BillableKind;

/// A domain entity that can be paid for via the payment layer.
///
/// Object-safe (`Send + Sync`, all methods take `&self`, no associated types) so a
/// `Box<dyn Billable>` can be returned by [`crate::loader::BillableLoader::load`].
/// Intentionally NOT `Clone` (D-06) — everything passes `&dyn Billable`.
#[async_trait]
pub trait Billable: Send + Sync {
    /// Open-set kind discriminator (stored in `payment_intents.billable_kind`).
    fn kind(&self) -> BillableKind;
    /// The billable entity's primary key.
    fn id(&self) -> i64;
    /// Owning tenant (the loader is responsible for tenant scoping — D-08).
    fn tenant_id(&self) -> i64;
    /// Charge amount in the smallest currency unit.
    fn amount_cents(&self) -> i64;
    /// ISO 4217 currency code, e.g. `"EUR"`.
    fn currency(&self) -> &str;
    /// Human-readable Stripe Checkout line-item description.
    fn checkout_line_description(&self) -> String;

    /// Stripe Connect destination account, when this billable routes funds to a
    /// connected account. Default `None` keeps non-Connect billables trivial; Connect
    /// billables override so `start_checkout` can snapshot `application_fee_cents` (D-05).
    fn connect_account_id(&self) -> Option<String> {
        None
    }

    /// Side effect after the payment is confirmed paid. Runs inside the caller's txn.
    async fn on_paid(&self, txn: &DatabaseTransaction) -> Result<(), PaymentError>;
    /// Side effect after a reserved intent is released (expired/unpaid).
    async fn on_released(&self, txn: &DatabaseTransaction) -> Result<(), PaymentError>;
    /// Side effect after a refund is confirmed, carrying the refunded amount.
    async fn on_refunded(
        &self,
        txn: &DatabaseTransaction,
        amount_cents: i64,
    ) -> Result<(), PaymentError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal concrete `Billable` impl — proves object-safety and the default
    /// `connect_account_id` behaviour.
    struct TestBillable;

    #[async_trait]
    impl Billable for TestBillable {
        fn kind(&self) -> BillableKind {
            BillableKind::new("test")
        }
        fn id(&self) -> i64 {
            1
        }
        fn tenant_id(&self) -> i64 {
            1
        }
        fn amount_cents(&self) -> i64 {
            1000
        }
        fn currency(&self) -> &str {
            "EUR"
        }
        fn checkout_line_description(&self) -> String {
            "Test item".to_string()
        }
        async fn on_paid(&self, _txn: &DatabaseTransaction) -> Result<(), PaymentError> {
            Ok(())
        }
        async fn on_released(&self, _txn: &DatabaseTransaction) -> Result<(), PaymentError> {
            Ok(())
        }
        async fn on_refunded(
            &self,
            _txn: &DatabaseTransaction,
            _amount_cents: i64,
        ) -> Result<(), PaymentError> {
            Ok(())
        }
    }

    #[test]
    fn connect_account_id_defaults_to_none() {
        let b = TestBillable;
        assert_eq!(b.connect_account_id(), None);
    }

    #[test]
    fn box_dyn_billable_is_constructible() {
        let b: Box<dyn Billable> = Box::new(TestBillable);
        assert_eq!(b.amount_cents(), 1000);
        assert_eq!(b.connect_account_id(), None);
    }
}
