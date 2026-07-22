use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Places::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Places::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Places::Name).string().not_null())
                    .col(
                        ColumnDef::new(Places::Category)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(Places::Lat).double().not_null())
                    .col(ColumnDef::new(Places::Lng).double().not_null())
                    .col(
                        ColumnDef::new(Places::Premium)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Places::CreatedAt)
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
            .drop_table(Table::drop().table(Places::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Places {
    Table,
    Id,
    Name,
    Category,
    Lat,
    Lng,
    Premium,
    CreatedAt,
}
