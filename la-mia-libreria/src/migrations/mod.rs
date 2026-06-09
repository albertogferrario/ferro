pub use sea_orm_migration::prelude::*;

mod m20260609_000000_create_books_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260609_000000_create_books_table::Migration)]
    }
}
