//! `Resource` — consumer-implemented capacity model (D-05).
//!
//! Implement this trait once per resource kind. The kernel calls
//! [`Resource::capacity`] and [`Resource::held`] inside
//! [`crate::ReservationKernel::hold`] to compute available capacity.
//!
//! `Key` identifies a resource instance; `Window` scopes capacity to a
//! time range, seat category, or any other dimension. Use `Window = ()`
//! for non-windowed resources (atomic counters, simple capacity).
//!
//! `KIND` is a `&'static str` const (D-08) — dotted-namespace convention:
//! `"inventory.unit"`, `"checkout.slot"`, `"api.quota"`.
//!
//! # Example
//!
//! ```rust,ignore
//! use async_trait::async_trait;
//! use ferro_reservation::{Resource, ReservationError};
//! use sea_orm::ConnectionTrait;
//!
//! struct SeatResource { /* venue rules, db reference */ }
//!
//! #[async_trait]
//! impl Resource for SeatResource {
//!     type Key = ShowId;
//!     type Window = ();
//!     const KIND: &'static str = "ticketing.seat";
//!
//!     async fn capacity<C: ConnectionTrait>(
//!         &self, _conn: &C, _key: &Self::Key, _w: &Self::Window,
//!     ) -> Result<u32, ReservationError> {
//!         Ok(self.venue.seat_count())
//!     }
//!
//!     async fn held<C: ConnectionTrait>(
//!         &self, conn: &C, key: &Self::Key, _w: &Self::Window,
//!     ) -> Result<u32, ReservationError> {
//!         // Sum of `quantity` for rows where status IN ('held','committed')
//!         // and resource_key = key.to_json() and resource_kind = Self::KIND.
//!         todo!("query reservations table")
//!     }
//! }
//! ```
//!
//! Multi-tenancy is a `Key` concern (D-37): the kernel does not scope
//! `capacity`/`held` queries by tenant — the consumer adds tenant
//! filtering inside the impl, typically by including the tenant id in
//! `Key`.

use async_trait::async_trait;
use sea_orm::ConnectionTrait;
use serde::{de::DeserializeOwned, Serialize};
use std::hash::Hash;

use crate::error::ReservationError;

#[async_trait]
pub trait Resource: Send + Sync + 'static {
    type Key: Hash + Eq + Clone + Send + Sync + Serialize + DeserializeOwned;
    type Window: PartialEq + Clone + Send + Sync + Serialize + DeserializeOwned;

    const KIND: &'static str;

    async fn capacity<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: &Self::Key,
        window: &Self::Window,
    ) -> Result<u32, ReservationError>;

    async fn held<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: &Self::Key,
        window: &Self::Window,
    ) -> Result<u32, ReservationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DatabaseConnection};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(crate::migration::Migration)]
        }
    }

    async fn fresh_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate");
        conn
    }

    /// Minimal Resource impl with fixed capacity, used in unit + property tests
    /// throughout the crate. `Window = ()` (non-windowed); `Key = String`.
    struct TestResource {
        capacity_value: u32,
    }

    #[async_trait]
    impl Resource for TestResource {
        type Key = String;
        type Window = ();
        const KIND: &'static str = "test.resource";

        async fn capacity<C: ConnectionTrait>(
            &self,
            _conn: &C,
            _key: &Self::Key,
            _window: &Self::Window,
        ) -> Result<u32, ReservationError> {
            Ok(self.capacity_value)
        }

        async fn held<C: ConnectionTrait>(
            &self,
            _conn: &C,
            _key: &Self::Key,
            _window: &Self::Window,
        ) -> Result<u32, ReservationError> {
            // Stub: real impls query the reservations table.
            // For this trait-shape sanity test we return 0.
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_resource_impl_capacity_and_held() {
        let conn = fresh_db().await;
        let r = TestResource { capacity_value: 10 };
        let key = "k1".to_string();
        let cap = r.capacity(&conn, &key, &()).await.expect("capacity");
        let held = r.held(&conn, &key, &()).await.expect("held");
        assert_eq!(cap, 10);
        assert_eq!(held, 0);
        assert_eq!(<TestResource as Resource>::KIND, "test.resource");
    }
}
