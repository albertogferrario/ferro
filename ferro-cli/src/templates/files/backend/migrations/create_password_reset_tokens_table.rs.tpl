use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PasswordResetTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PasswordResetTokens::Email)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PasswordResetTokens::Token).string().not_null())
                    .col(
                        ColumnDef::new(PasswordResetTokens::CreatedAt)
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
                    .name("idx_password_reset_tokens_token")
                    .table(PasswordResetTokens::Table)
                    .col(PasswordResetTokens::Token)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(PasswordResetTokens::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum PasswordResetTokens {
    Table,
    Email,
    Token,
    CreatedAt,
}
