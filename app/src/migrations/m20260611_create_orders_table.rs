use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Orders::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Orders::CustomerName).string().not_null())
                    // Derived field: excluded from CRUD write input (read-only), so the
                    // INSERT omits it. Default 0 lets an order be created with no line
                    // items; the recompute hook updates it as line items are added.
                    .col(ColumnDef::new(Orders::Total).double().not_null().default(0.0))
                    .col(ColumnDef::new(Orders::Status).string().not_null())
                    .col(
                        ColumnDef::new(Orders::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Orders::TenantId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Orders::Table, Orders::TenantId)
                            .to(Tenants::Table, Tenants::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Orders::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
    CustomerName,
    Total,
    Status,
    CreatedAt,
    TenantId,
}

// Minimal IdenStatic for the FK target reference
#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}
