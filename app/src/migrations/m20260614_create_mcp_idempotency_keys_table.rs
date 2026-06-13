use sea_orm_migration::prelude::*;

/// Local wrapper for `ferro_mcp_oauth::CreateMcpIdempotencyKeysTable`.
///
/// Using a local file gives this migration a unique version name derived from
/// the file stem ("m20260614_create_mcp_idempotency_keys_table") rather than
/// the external crate's "migration" stem, which would collide with the audit
/// migration registered below.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        ferro_mcp_oauth::CreateMcpIdempotencyKeysTable
            .up(manager)
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        ferro_mcp_oauth::CreateMcpIdempotencyKeysTable
            .down(manager)
            .await
    }
}
