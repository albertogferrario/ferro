//! The `BillableLoader` trait — the consumer registers a polymorphic loader so the
//! payment layer can resolve a `(kind, id)` to a `Box<dyn Billable>`.

use async_trait::async_trait;

use crate::billable::Billable;
use crate::error::PaymentError;
use crate::BillableKind;

/// Resolves a `(kind, id)` pair to the concrete billable entity.
///
/// - `Ok(Some(_))` — the billable was loaded.
/// - `Ok(None)` — the billable no longer exists (a DELETE in flight); the phase-235
///   webhook handler treats this as a trigger for the auto-refund fallback.
/// - `Err(PaymentError::Loader(..))` — a consumer-side failure (DB error, unknown kind).
///
/// No separate `tenant_id` argument (D-08): tenant scoping is the loader's concern,
/// and the loaded `Billable` exposes `tenant_id()`.
#[async_trait]
pub trait BillableLoader: Send + Sync {
    async fn load(
        &self,
        kind: BillableKind,
        id: i64,
    ) -> Result<Option<Box<dyn Billable>>, PaymentError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::billable::Billable;
    use crate::error::PaymentError;
    use crate::BillableKind;
    use async_trait::async_trait;

    struct MockLoader;

    #[async_trait]
    impl BillableLoader for MockLoader {
        async fn load(
            &self,
            _kind: BillableKind,
            _id: i64,
        ) -> Result<Option<Box<dyn Billable>>, PaymentError> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn mock_loader_returns_ok_none() {
        let loader = MockLoader;
        let result = loader.load(BillableKind::new("test"), 1).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn mock_loader_usable_as_dyn_billable_loader() {
        let loader: &dyn BillableLoader = &MockLoader;
        let result = loader.load(BillableKind::new("order"), 42).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
