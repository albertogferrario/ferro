//! # ferro-orm
//!
//! Atomic conditional updates and ORM primitives for the Ferro framework.
//!
//! `GuardedUpdate<E>` compiles to a single `UPDATE … WHERE …` SQL statement,
//! replacing the hand-rolled `read → check → write` pattern wherever a column's
//! value is conditionally mutated. The database is the authority on contention;
//! the call site is race-free by construction.
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_orm::{GuardedUpdate, ColumnTrait};
//! use sea_orm::sea_query::Expr;
//!
//! GuardedUpdate::new(inventory_units::Entity)
//!     .filter(inventory_units::Column::Id.eq(unit_id))
//!     .filter(inventory_units::Column::Quantity.gte(needed))
//!     .set_expr(
//!         inventory_units::Column::Quantity,
//!         Expr::col(inventory_units::Column::Quantity).sub(needed),
//!     )
//!     .exec_one(&txn)
//!     .await?;
//! // — exactly one row matched and was decremented atomically,
//! //   OR Err(NoRowsAffected) signalling capacity exhausted.
//! ```
//!
//! ## Atomicity guarantee (and its limit)
//!
//! `GuardedUpdate` guarantees atomicity *per statement*, not per builder.
//! A caller building `.set_expr(qty - 1)` and reading the resulting `qty`
//! in a separate query without a transaction re-introduces a race. The crate's
//! job is to make the conditional `UPDATE` race-free; bracketing it in a
//! transaction is the caller's responsibility.

mod error;
mod guarded;

pub use error::GuardedError;
pub use guarded::GuardedUpdate;

// Targeted re-exports — consumers calling the builder need these.
// Do NOT add `pub use sea_orm::*;` (D-03).
pub use sea_orm::sea_query::{Expr, IntoCondition, SimpleExpr, Value};
pub use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait};
