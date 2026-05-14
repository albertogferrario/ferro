//! Backfill helpers. Filled in by Task 2.

use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::prelude::*;

/// Placeholder; Task 2 replaces with real implementation.
pub async fn backfill_random_hex(
    _manager: &SchemaManager<'_>,
    _table: &str,
    _column: &str,
    _hex_len: u32,
) -> Result<(), DbErr> {
    Ok(())
}

/// Placeholder; Task 2 replaces with real implementation.
pub async fn backfill_random_uuid(
    _manager: &SchemaManager<'_>,
    _table: &str,
    _column: &str,
) -> Result<(), DbErr> {
    Ok(())
}

/// Placeholder; Task 2 replaces with real implementation.
pub async fn backfill_current_timestamp(
    _manager: &SchemaManager<'_>,
    _table: &str,
    _column: &str,
) -> Result<(), DbErr> {
    Ok(())
}

/// Placeholder; Task 2 replaces with real implementation.
pub async fn backfill<F, Fut>(
    _manager: &SchemaManager<'_>,
    _sql_fn: F,
) -> Result<(), DbErr>
where
    F: FnOnce(DbBackend) -> Fut,
    Fut: std::future::Future<Output = Result<String, DbErr>>,
{
    Ok(())
}
