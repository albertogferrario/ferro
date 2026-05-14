//! Backend-portable migration helpers for ferro applications.
//!
//! These helpers dispatch on `SchemaManager::get_database_backend()` so a single
//! call works identically on SQLite and Postgres. Use them inside
//! `MigrationTrait::up` implementations to avoid hand-rolled
//! `DbBackend::Sqlite`-pinned raw SQL.
//!
//! See the crate README for the pgcrypto requirement note.

#![warn(missing_docs)]

pub mod backfill;
pub mod error;

pub use crate::backfill::{
    backfill, backfill_current_timestamp, backfill_random_hex, backfill_random_uuid,
};
pub use crate::error::Error;
