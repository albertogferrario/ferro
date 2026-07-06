use sea_orm_migration::prelude::*;

use super::m001_users::Users;
use super::m002_articles::Articles;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Favorites::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Favorites::UserId).integer().not_null())
                    .col(ColumnDef::new(Favorites::ArticleId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(Favorites::UserId)
                            .col(Favorites::ArticleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_favorites_user")
                            .from(Favorites::Table, Favorites::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_favorites_article")
                            .from(Favorites::Table, Favorites::ArticleId)
                            .to(Articles::Table, Articles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Favorites::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Favorites {
    Table,
    UserId,
    ArticleId,
}
