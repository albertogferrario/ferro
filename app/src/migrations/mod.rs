pub use sea_orm_migration::prelude::*;

mod m20251208_160100_create_users_table;
mod m20251208_200000_create_todos_table;
mod m20260228_create_api_keys_table;
mod m20260611_add_tenant_id_to_users;
mod m20260611_create_oauth_clients_table;
mod m20260611_create_orders_table;
mod m20260611_create_sessions_table;
mod m20260611_create_tenants_table;
// MCP write-dispatch tables: local wrappers give unique version names derived
// from the file stem, avoiding collisions with external crate "migration" stems.
mod m20260614_create_mcp_idempotency_keys_table;
mod m20260614_create_audit_log_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251208_160100_create_users_table::Migration),
            Box::new(m20251208_200000_create_todos_table::Migration),
            Box::new(m20260228_create_api_keys_table::Migration),
            Box::new(m20260611_create_oauth_clients_table::Migration),
            Box::new(m20260611_create_tenants_table::Migration),
            Box::new(m20260611_add_tenant_id_to_users::Migration),
            Box::new(m20260611_create_orders_table::Migration),
            Box::new(m20260611_create_sessions_table::Migration),
            Box::new(m20260614_create_mcp_idempotency_keys_table::Migration),
            Box::new(m20260614_create_audit_log_table::Migration),
        ]
    }
}
