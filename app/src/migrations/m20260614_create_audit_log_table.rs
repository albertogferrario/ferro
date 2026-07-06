use sea_orm_migration::prelude::*;

/// Local wrapper for `ferro_audit::CreateAuditLogTable`.
///
/// Using a local file gives this migration a unique version name derived from
/// the file stem ("m20260614_create_audit_log_table") rather than the external
/// crate's "migration" stem, which would collide with the idempotency-keys
/// migration registered above.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        ferro_audit::CreateAuditLogTable.up(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        ferro_audit::CreateAuditLogTable.down(manager).await
    }
}
