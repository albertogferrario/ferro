use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Profiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Profiles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Profiles::UserId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Profiles::DisplayName).string().not_null())
                    .col(
                        ColumnDef::new(Profiles::Status)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(Profiles::AvatarUrl).string().null())
                    .col(
                        ColumnDef::new(Profiles::Visible)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Profiles::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Profiles::UpdatedAt)
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
            .drop_table(Table::drop().table(Profiles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Profiles {
    Table,
    Id,
    UserId,
    DisplayName,
    Status,
    AvatarUrl,
    Visible,
    CreatedAt,
    UpdatedAt,
}
