use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LineItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LineItems::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LineItems::OrderId).big_integer().not_null())
                    .col(ColumnDef::new(LineItems::Amount).double().not_null())
                    .col(ColumnDef::new(LineItems::TenantId).big_integer().not_null())
                    // created_at: injected by the CRUD Create kernel (execute_crud_plan always
                    // adds this column); required for INSERT … RETURNING * to succeed.
                    .col(
                        ColumnDef::new(LineItems::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(LineItems::DeletedAt).timestamp().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(LineItems::Table, LineItems::OrderId)
                            .to(Orders::Table, Orders::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LineItems::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LineItems {
    Table,
    Id,
    OrderId,
    Amount,
    TenantId,
    CreatedAt,
    DeletedAt,
}

// FK target reference (mirrors the orders → tenants pattern).
#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
}
