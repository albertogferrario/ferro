use sea_orm_migration::prelude::*;

use super::m001_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Articles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Articles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Articles::Slug)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Articles::Title).string().not_null())
                    .col(ColumnDef::new(Articles::Description).string().not_null())
                    .col(ColumnDef::new(Articles::Body).text().not_null())
                    .col(ColumnDef::new(Articles::AuthorId).integer().not_null())
                    .col(
                        ColumnDef::new(Articles::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Articles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_articles_author")
                            .from(Articles::Table, Articles::AuthorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Articles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Articles {
    Table,
    Id,
    Slug,
    Title,
    Description,
    Body,
    AuthorId,
    CreatedAt,
    UpdatedAt,
}
