//! # ferro-reservation
//!
//! Generic hold/commit/release resource reservation kernel for the Ferro
//! framework.
//!
//! Capacity-constrained apps (booking, ticketing, checkout, queue
//! admission, rate limiting) all hand-roll the same buggy `read → check →
//! write` pattern. ferro-reservation replaces it with a typed kernel that
//! is race-free by construction, automatically writes a before/after
//! audit entry on every transition, and emits typed domain events for
//! downstream live read-models or broadcast fanout.
//!
//! ferro-reservation is the *capacity* primitive. [`ferro-events`] says
//! *something happened*. [`ferro-audit`] says *here is the evidence
//! forever*. [`ferro_orm::GuardedUpdate`] says *only one writer wins*.
//! ferro-reservation composes the three: the resource is reserved, with a
//! deadline, race-free, with audit and broadcast — pick a side from the
//! trio at the right layer.
//!
//! ## State diagram
//!
//! ```text
//!                hold()                  commit()
//!     ──────────────▶ held ──────────────────────▶ committed
//!                      │
//!                      │ release(reason)
//!                      ▼
//!                  released
//!                      ▲
//!                      │ run_sweep_once()
//!                      │
//!     ──────────────▶ held ─── ttl ─────────────▶ expired
//! ```
//!
//! Terminal states (`committed`, `released`, `expired`) have no outgoing
//! transitions. Any attempt is a [`ReservationError::ConflictingState`].
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_reservation::{ReservationKernel, ReservationContext, Resource, ReleaseReason};
//! use std::time::Duration;
//!
//! // Consumer-defined Resource impl
//! struct InventoryUnitResource { /* db reference, business rules */ }
//!
//! #[async_trait::async_trait]
//! impl Resource for InventoryUnitResource {
//!     type Key = (TenantId, ProductId);
//!     type Window = BookingWindow;
//!     const KIND: &'static str = "inventory.unit";
//!
//!     async fn capacity<C: ConnectionTrait>(&self, conn: &C, key: &Self::Key, _w: &Self::Window)
//!         -> Result<u32, _> { /* ... */ }
//!     async fn held<C: ConnectionTrait>(&self, conn: &C, key: &Self::Key, w: &Self::Window)
//!         -> Result<u32, _> { /* ... */ }
//! }
//!
//! // Application setup
//! let kernel = ReservationKernel::new(db.clone(), InventoryUnitResource::new(/* ... */));
//!
//! // Online-checkout: hold a slot during payment
//! let ctx = ReservationContext::user(user_id.to_string()).with_correlation(request_id);
//! let handle = kernel.hold(&conn, key, window, /*qty=*/1, Duration::from_secs(15 * 60), &ctx).await?;
//!
//! match stripe_result {
//!     Ok(_)  => kernel.commit(&conn, handle, &ctx).await?,
//!     Err(_) => kernel.release(&conn, handle, ReleaseReason::PaymentFailed, &ctx).await?,
//! }
//! ```
//!
//! ## Audit and events — operational semantics
//!
//! - **Audit emission is unconditional.** Every successful state
//!   transition writes one `AuditEntry` with
//!   `action = "reservation.{held|committed|released|expired}"`. If the
//!   audit write fails, the DB state change is already committed
//!   ([`ferro_orm::GuardedUpdate`] is atomic at the SQL level); the
//!   kernel surfaces [`ReservationError::Audit`] so consumers can alarm
//!   on it, but does NOT attempt to roll back the state change.
//! - **Event dispatch is best-effort.** Events are emitted via
//!   [`ferro_events::dispatch`] AFTER the state change commits. If
//!   dispatch fails (no listeners, listener panic, bus disconnect), the
//!   kernel logs at `tracing::warn!` and returns `Ok(())`. Consumers
//!   can replay missed events from the audit log (which never depends
//!   on event dispatch).
//!
//! ## Schema and migration
//!
//! ferro-reservation ships a SeaORM migration as
//! [`CreateReservationsTable`]. Register it in your consumer-side
//! `Migrator`, alongside [`ferro_audit::CreateAuditLogTable`]:
//!
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(ferro_audit::CreateAuditLogTable),
//!             Box::new(ferro_reservation::CreateReservationsTable),
//!             // ... your app migrations
//!         ]
//!     }
//! }
//! ```
//!
//! ## Sweeper scheduling
//!
//! `ReservationKernel::run_sweep_once` is the sweeper primitive; it
//! transitions `held` rows whose `expires_at` is past to `expired`.
//! Schedule it from your consumer side — three idiomatic patterns:
//!
//! 1. **ferro-queue `Job`** — implement a `Job` that calls
//!    `kernel.run_sweep_once().await` and schedule it via the queue.
//! 2. **`tokio::time::interval` task** — spawn a 60-second loop on
//!    application start.
//! 3. **Cron-driven CLI** — `your-app reservation:sweep` calls
//!    `kernel.run_sweep_once()` and exits.
//!
//! ferro-reservation has no `ferro-queue` runtime dependency by design
//! (the choice of scheduler is consumer territory).

mod context;
mod entity;
mod error;
mod event;
mod handle;
mod kernel;
mod migration;
mod resource;
mod sweeper;

pub use context::ReservationContext;
pub use error::ReservationError;
pub use event::{ReleaseReason, ReservationEvent};
pub use handle::ReservationHandle;
pub use kernel::ReservationKernel;
pub use migration::Migration as CreateReservationsTable;
pub use resource::Resource;
pub use sweeper::SweepReport;

// SeaORM entity re-exports for consumers who need native SeaORM query access.
pub use entity::{
    ActiveModel as ReservationActiveModel, Entity as ReservationEntity, Model as ReservationModel,
};

// Re-export `AuditActor` so consumers building `ReservationContext`
// don't need a direct `ferro-audit` dependency for the common case.
pub use ferro_audit::AuditActor;
