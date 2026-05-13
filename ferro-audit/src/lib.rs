//! # ferro-audit
//!
//! Append-only structured before/after audit log for the Ferro framework.
//!
//! Audit entries record *what happened* — for forensic investigation,
//! regulatory evidence, and state replay. They are the historical twin
//! of [`ferro-events`]: events are "something happened, react now";
//! audit entries are "something happened, here is the evidence forever".
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_audit::{AuditEntry, AuditActor, AuditTarget};
//! use serde_json::json;
//!
//! AuditEntry::record("inventory.stock.adjust")
//!     .actor(AuditActor::User(user_id.to_string()))
//!     .target(AuditTarget::new("inventory.unit", unit_id.to_string()))
//!     .before(json!({ "quantity": old }))
//!     .after(json!({ "quantity": new }))
//!     .reason("order_committed")
//!     .write(&conn)
//!     .await?;
//! ```
//!
//! ## Replay
//!
//! `AuditEntry::history_for_target(&target, &conn).await?` returns the
//! sequence of entries ordered ascending by `created_at`. Passing that
//! sequence to [`reconstruct_state`] folds each entry's `after` JSON into
//! a running object — the *replay* primitive.
//!
//! The fold is a **shallow object merge**: newer keys overwrite older
//! keys at the top level only. Nested objects and arrays are replaced
//! wholesale, not deep-merged. A consumer needing deep-merge runs its
//! own fold over the `Vec<AuditEntry>`.
//!
//! ## Schema and Migration
//!
//! ferro-audit ships a SeaORM migration as [`CreateAuditLogTable`].
//! Register it in your consumer-side `Migrator`:
//!
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(ferro_audit::CreateAuditLogTable),
//!             // ... your app migrations
//!         ]
//!     }
//! }
//! ```

mod actor;
mod entity;
mod entry;
mod error;
mod migration;
mod prune;
mod query;
mod replay;
mod target;

pub use actor::AuditActor;
pub use entry::AuditEntry;
pub use error::AuditError;
pub use migration::Migration as CreateAuditLogTable;
pub use prune::prune_older_than;
pub use query::{history_for_target, recent, recent_by_actor};
pub use replay::reconstruct_state;
pub use target::AuditTarget;

// Entity re-export for SeaORM-native consumer queries (D-25 — consumers
// needing pagination / custom filters drop down to sea-orm directly).
pub use entity::Entity as AuditLogEntity;
