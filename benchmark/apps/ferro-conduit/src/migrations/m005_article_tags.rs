use sea_orm_migration::prelude::*;

use super::m002_articles::Articles;
use super::m004_tags::Tags;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ArticleTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ArticleTags::ArticleId).integer().not_null())
                    .col(ColumnDef::new(ArticleTags::TagId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(ArticleTags::ArticleId)
                            .col(ArticleTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_article_tags_article")
                            .from(ArticleTags::Table, ArticleTags::ArticleId)
                            .to(Articles::Table, Articles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_article_tags_tag")
                            .from(ArticleTags::Table, ArticleTags::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ArticleTags::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ArticleTags {
    Table,
    ArticleId,
    TagId,
}
