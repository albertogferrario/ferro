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
                    .table(Follows::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Follows::FollowerId).integer().not_null())
                    .col(ColumnDef::new(Follows::FollowedId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(Follows::FollowerId)
                            .col(Follows::FollowedId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_follows_follower")
                            .from(Follows::Table, Follows::FollowerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_follows_followed")
                            .from(Follows::Table, Follows::FollowedId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Follows::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Follows {
    Table,
    FollowerId,
    FollowedId,
}
