//! Data tenant-scoping test (SUBST-03 / SUBST-05 / T-263-15).
//!
//! Proves that `framework::projection_read::dispatch` enforces tenant isolation:
//!
//! - Querying with `tenant_id = Some(1)` returns ONLY tenant 1's rows.
//! - Querying with `tenant_id = Some(2)` returns ONLY tenant 2's rows.
//! - Looking up an id belonging to tenant 2 while scoped to tenant 1 returns
//!   ZERO rows (cross-tenant id is "not found" — no data disclosure).
//!
//! This test exercises `framework::projection_read::dispatch` directly, not the
//! MCP surface (`ferro_mcp_server::handle_tools_call`). Both surfaces delegate to
//! the same kernel; this test pins the framework-level tenant predicate.
//!
//! Uses the app's `Migrator` for DB setup (same pattern as `single_source.rs` and
//! `crud_e2e.rs`). The `ServiceDef` includes `tenant_column("tenant_id")` so the
//! tenant predicate is active on every query.

#[cfg(test)]
mod tests {
    use crate::migrations::Migrator;
    use ferro::projection_read::dispatch;
    use ferro::serde_json::json;
    use ferro_projections::{DataType, FieldMeaning, ServiceDef};
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database, DatabaseConnection};
    use sea_orm_migration::prelude::*;

    // ── Fixture helpers ──────────────────────────────────────────────────────

    /// Open an in-memory SQLite DB and run the full app Migrator (includes orders,
    /// tenants, users, mcp_idempotency_keys, audit_log, deleted_at, etc.).
    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite connect failed");
        Migrator::up(&db, None)
            .await
            .expect("migrations failed on test DB");
        db
    }

    /// Seed two tenants with two orders each.
    ///
    /// Tenants: 1 (acme), 2 (globex).
    /// Orders:
    ///   - id=1, id=2 → tenant 1
    ///   - id=3, id=4 → tenant 2
    async fn seed_two_tenants(db: &DatabaseConnection) {
        use crate::models::entities::orders::ActiveModel as OrderActive;
        use crate::models::entities::tenants::ActiveModel as TenantActive;

        let now = "2026-07-27T00:00:00+00:00";

        for (id, slug, name) in [(1i64, "acme", "Acme"), (2i64, "globex", "Globex")] {
            TenantActive {
                id: Set(id),
                slug: Set(slug.into()),
                name: Set(name.into()),
                created_at: Set(now.into()),
            }
            .insert(db)
            .await
            .unwrap_or_else(|e| panic!("seed tenant {slug}: {e}"));
        }

        for (id, tid, customer) in [
            (1i32, 1i64, "Acme Order 1"),
            (2i32, 1i64, "Acme Order 2"),
            (3i32, 2i64, "Globex Order 1"),
            (4i32, 2i64, "Globex Order 2"),
        ] {
            OrderActive {
                id: Set(id),
                customer_name: Set(customer.into()),
                total: Set(10.0 * id as f64),
                status: Set("draft".into()),
                created_at: Set(now.into()),
                tenant_id: Set(tid),
                deleted_at: Set(None),
            }
            .insert(db)
            .await
            .unwrap_or_else(|e| panic!("seed order {id}: {e}"));
        }
    }

    /// A minimal `ServiceDef` backed by the `orders` table with tenant isolation active.
    ///
    /// `tenant_column("tenant_id")` activates the WHERE tenant_id=? predicate in
    /// `framework::projection_read::dispatch`.
    fn order_service_with_tenant() -> ServiceDef {
        ServiceDef::new("order")
            .mcp_exposed(true)
            .tenant_column("tenant_id")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_name", DataType::String, FieldMeaning::EntityName)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("tenant_id", DataType::Integer, FieldMeaning::ForeignKey)
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    /// SUBST-03 / T-263-15: tenant 1 only sees its own rows.
    ///
    /// `dispatch` with `tenant_id = Some(1)` must return exactly 2 rows,
    /// all with `tenant_id = 1`.
    #[tokio::test]
    async fn data_tenant_scoping() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let service = order_service_with_tenant();

        let result = dispatch(&service, json!({}), 10, 0, &db, Some(1))
            .await
            .expect("dispatch ok for tenant 1");

        assert_eq!(
            result.rows.len(),
            2,
            "tenant 1 must see exactly 2 rows; got: {:?}",
            result.rows
        );
        for row in &result.rows {
            assert_eq!(
                row["tenant_id"].as_i64(),
                Some(1),
                "every row must belong to tenant 1; got: {row}"
            );
        }
    }

    /// Symmetric: tenant 2 only sees its own rows.
    #[tokio::test]
    async fn tenant_isolation_symmetric() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let service = order_service_with_tenant();

        let result = dispatch(&service, json!({}), 10, 0, &db, Some(2))
            .await
            .expect("dispatch ok for tenant 2");

        assert_eq!(
            result.rows.len(),
            2,
            "tenant 2 must see exactly 2 rows; got: {:?}",
            result.rows
        );
        for row in &result.rows {
            assert_eq!(
                row["tenant_id"].as_i64(),
                Some(2),
                "every row must belong to tenant 2; got: {row}"
            );
        }
    }

    /// T-263-15 (cross-tenant id not found): filtering by an id that belongs to
    /// tenant 2 while scoped to tenant 1 returns ZERO rows.
    ///
    /// This is the security regression pin: cross-tenant id lookup must not leak
    /// data (the row is invisible, not an error). `dispatch` returns an empty
    /// `rows` list.
    #[tokio::test]
    async fn cross_tenant_id_not_found() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let service = order_service_with_tenant();

        // Order id=3 belongs to tenant 2; querying as tenant 1 must yield zero rows.
        let result = dispatch(&service, json!({ "id": 3 }), 10, 0, &db, Some(1))
            .await
            .expect("dispatch ok");

        assert!(
            result.rows.is_empty(),
            "cross-tenant id lookup must return zero rows (not found, no disclosure); \
             got: {:?}",
            result.rows
        );
    }
}
