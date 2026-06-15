pub use sea_orm_migration::prelude::*;

mod m001_users;
mod m002_articles;
mod m003_comments;
mod m004_tags;
mod m005_article_tags;
mod m006_follows;
mod m007_favorites;

/// Migrator for the Conduit schema. Order is dependency-driven:
/// users/tags first, then articles, then comments and the junction tables.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_users::Migration),
            Box::new(m002_articles::Migration),
            Box::new(m003_comments::Migration),
            Box::new(m004_tags::Migration),
            Box::new(m005_article_tags::Migration),
            Box::new(m006_follows::Migration),
            Box::new(m007_favorites::Migration),
        ]
    }
}
