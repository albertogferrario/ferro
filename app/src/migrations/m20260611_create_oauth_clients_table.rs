use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OauthClients::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OauthClients::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OauthClients::ClientId).string().not_null())
                    .col(ColumnDef::new(OauthClients::ClientName).string().null())
                    .col(
                        ColumnDef::new(OauthClients::RedirectUris)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OauthClients::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_oauth_clients_client_id")
                    .table(OauthClients::Table)
                    .col(OauthClients::ClientId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OauthClients::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OauthClients {
    Table,
    Id,
    ClientId,
    ClientName,
    RedirectUris,
    CreatedAt,
}
