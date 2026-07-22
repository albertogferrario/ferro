use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Trillos::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Trillos::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Trillos::FromUserId).integer().not_null())
                    .col(ColumnDef::new(Trillos::ToUserId).integer().not_null())
                    .col(
                        ColumnDef::new(Trillos::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(Trillos::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_trillos_to_user_id")
                    .table(Trillos::Table)
                    .col(Trillos::ToUserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Trillos::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Trillos {
    Table,
    Id,
    FromUserId,
    ToUserId,
    Status,
    CreatedAt,
}
