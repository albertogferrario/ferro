pub use sea_orm_migration::prelude::*;

/// Migrator. Plan 02 replaces the empty migration vec with the Conduit schema.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}
