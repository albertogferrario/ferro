use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Presences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Presences::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Presences::UserId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Presences::Lat).double().not_null())
                    .col(ColumnDef::new(Presences::Lng).double().not_null())
                    .col(
                        ColumnDef::new(Presences::LastSeen)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Presences::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Presences {
    Table,
    Id,
    UserId,
    Lat,
    Lng,
    LastSeen,
}
