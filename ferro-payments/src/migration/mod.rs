//! Migration module for `ferro-payments`.
//!
//! Consumers include the migration returned by
//! [`migration_create_payment_intents`] in their own `Migrator`.

mod m20260617_create_payment_intents;

pub use m20260617_create_payment_intents::Migration as CreatePaymentIntentsTable;

/// Returns a boxed [`sea_orm_migration::MigrationTrait`] for the
/// `payment_intents` table migration. Pass the return value to your
/// application's `Migrator::migrations()` list.
pub fn migration_create_payment_intents() -> Box<dyn sea_orm_migration::MigrationTrait> {
    Box::new(m20260617_create_payment_intents::Migration)
}
