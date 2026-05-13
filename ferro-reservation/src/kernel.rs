//! `ReservationKernel<R>` — the typed hold/commit/release/extend
//! orchestrator (D-09).
//!
//! This file is a STUB. Plan 154-05 lands the four state-transition
//! methods (`hold`, `commit`, `release`, `extend`) per D-10..D-15, the
//! `GuardedUpdate` + audit emission + event dispatch pattern from
//! PATTERNS.md §kernel.rs.

#![allow(dead_code)]

use sea_orm::DatabaseConnection;

use crate::resource::Resource;

/// Generic hold/commit/release/extend orchestrator over a consumer's
/// `Resource` implementation. The struct carries an owned
/// `DatabaseConnection` for the sweeper path (which has no caller-supplied
/// conn); per-call methods accept an explicit `&C: ConnectionTrait` so
/// consumers can run them inside their own transactions.
pub struct ReservationKernel<R: Resource> {
    pub(crate) db: DatabaseConnection,
    pub(crate) resource: R,
}

impl<R: Resource> ReservationKernel<R> {
    /// Construct a kernel from an owned `DatabaseConnection` and a
    /// consumer `Resource` impl. Cloning is cheap (`DatabaseConnection`
    /// is `Clone` by SeaORM's design).
    pub fn new(db: DatabaseConnection, resource: R) -> Self {
        Self { db, resource }
    }
}

impl<R: Resource> Clone for ReservationKernel<R>
where
    R: Clone,
{
    fn clone(&self) -> Self {
        Self { db: self.db.clone(), resource: self.resource.clone() }
    }
}
