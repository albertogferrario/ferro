//! Database migrations for Nearly.

pub use sea_orm_migration::prelude::*;

mod m20260722_000001_create_users_table;
mod m20260722_000002_create_sessions_table;
mod m20260722_000003_create_profiles_table;
mod m20260722_000004_create_presences_table;
mod m20260722_000005_create_trillos_table;
mod m20260722_000006_create_places_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260722_000001_create_users_table::Migration),
            Box::new(m20260722_000002_create_sessions_table::Migration),
            Box::new(m20260722_000003_create_profiles_table::Migration),
            Box::new(m20260722_000004_create_presences_table::Migration),
            Box::new(m20260722_000005_create_trillos_table::Migration),
            Box::new(m20260722_000006_create_places_table::Migration),
        ]
    }
}
