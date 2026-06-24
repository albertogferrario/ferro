//! End-to-end field test for Phase 243.1 — derived `order.total`.
//!
//! Proves that:
//!   1. `create_order` input schema excludes the read-only `total` field.
//!   2. Creating an order with no line items yields `total == 0`.
//!   3. `order.total` equals `SUM(line_items.amount)` after line-item writes.
//!   4. Deleting a line item recomputes `order.total` (read-your-writes).
//!
//! The lifecycle assertions (2–4) and their fixtures live in a
//! `#[cfg(not(feature = "confirmation"))]` submodule — matching `crud_e2e.rs`'s
//! cycle test, since `delete_line_item` is a two-step confirm flow when the
//! `confirmation` feature is on. The schema assertion (1) needs no gate.

#[cfg(test)]
mod tests {
    /// Truth 1: create_order input schema must not include the read-only `total` field.
    ///
    /// Static schema check — runs under every feature configuration.
    #[tokio::test]
    async fn create_order_schema_omits_derived_total() {
        let svc = crate::projections::order::service_def();
        let schema =
            ferro_mcp_server::schema::build_create_input_schema(&svc).expect("create schema");
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("customer_name"));
        assert!(
            !props.contains_key("total"),
            "create_order must not accept a derived `total`"
        );
    }

    /// Truths 2–4: full derived-total lifecycle through the live kernel.
    ///
    /// Gated `not(feature = "confirmation")`: `delete_line_item` is destructive and
    /// uses the two-step confirm flow when the feature is on. The fixtures below are
    /// only referenced here, so they live inside this gate to stay dead-code-free
    /// under `--all-features`.
    #[cfg(not(feature = "confirmation"))]
    mod lifecycle {
        use crate::migrations::Migrator;
        use ferro::serde_json::json;
        use ferro_mcp_server::{handle_tools_call, McpContext, WriteDispatcher};
        use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database, DatabaseConnection};
        use sea_orm_migration::prelude::*;

        /// Open an in-memory SQLite DB and run the full Migrator (includes line_items,
        /// deleted_at, mcp_idempotency_keys, and audit_log tables).
        async fn setup_db() -> DatabaseConnection {
            let db = Database::connect("sqlite::memory:")
                .await
                .expect("in-memory SQLite connect failed");
            Migrator::up(&db, None)
                .await
                .expect("migrations failed on test DB");
            db
        }

        /// Seed two tenants and their users (acme=1/alice=901, globex=2/bob=902).
        async fn seed_two_tenants(db: &DatabaseConnection) {
            use crate::models::entities::tenants::ActiveModel as TenantActive;
            use crate::models::entities::users::ActiveModel as UserActive;

            let now = "2026-06-24T00:00:00+00:00";

            TenantActive {
                id: Set(1),
                slug: Set("acme".into()),
                name: Set("Acme".into()),
                created_at: Set(now.into()),
            }
            .insert(db)
            .await
            .expect("seed: insert acme");

            TenantActive {
                id: Set(2),
                slug: Set("globex".into()),
                name: Set("Globex".into()),
                created_at: Set(now.into()),
            }
            .insert(db)
            .await
            .expect("seed: insert globex");

            UserActive {
                id: Set(901),
                name: Set("Alice Acme".into()),
                email: Set("alice@acme.test".into()),
                password: Set("hashed".into()),
                remember_token: Set(None),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
                tenant_id: Set(Some(1)),
            }
            .insert(db)
            .await
            .expect("seed: insert alice");

            UserActive {
                id: Set(902),
                name: Set("Bob Globex".into()),
                email: Set("bob@globex.test".into()),
                password: Set("hashed".into()),
                remember_token: Set(None),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
                tenant_id: Set(Some(2)),
            }
            .insert(db)
            .await
            .expect("seed: insert bob");
        }

        /// Invoke a CRUD write tool with an authorized `McpContext`.
        ///
        /// Uses `exposed_services()` (order + line_item) and `make_write_dispatcher()`
        /// (carries the recompute hook) so the hook fires on every line-item write.
        async fn call_write(
            db: &DatabaseConnection,
            tool_name: &str,
            arguments: ferro::serde_json::Value,
        ) -> ferro::serde_json::Value {
            let services = crate::controllers::mcp::exposed_services();
            let dispatcher = crate::controllers::mcp::make_write_dispatcher();
            let ctx = McpContext {
                tenant_id: Some(1),
                scope: Some("read_write".to_string()),
                write_authorized: Some(true),
                ..Default::default()
            };
            let params = json!({ "name": tool_name, "arguments": arguments });
            handle_tools_call(params, &services, db, Some(1), &ctx, &dispatcher).await
        }

        /// Invoke a read tool with a noop dispatcher (reads bypass the write kernel).
        async fn call_read(
            db: &DatabaseConnection,
            tool_name: &str,
            arguments: ferro::serde_json::Value,
        ) -> ferro::serde_json::Value {
            let services = crate::controllers::mcp::exposed_services();
            let noop = WriteDispatcher::new(
                Box::new(|_, _, _, _| Box::pin(async { Ok(json!({})) })),
                Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            );
            let ctx = McpContext {
                tenant_id: Some(1),
                ..Default::default()
            };
            let params = json!({ "name": tool_name, "arguments": arguments });
            handle_tools_call(params, &services, db, Some(1), &ctx, &noop).await
        }

        #[tokio::test]
        async fn order_total_is_derived_from_line_items() {
            let db = setup_db().await;
            seed_two_tenants(&db).await;

            // Create an order — no `total` supplied; server defaults it to 0.
            let created =
                call_write(&db, "create_order", json!({ "customer_name": "Mario Rossi" })).await;
            let order_id = created["result"]["structuredContent"]["result"]["id"]
                .as_i64()
                .unwrap_or_else(|| panic!("order id missing; got: {created}"));
            assert_eq!(
                created["result"]["structuredContent"]["result"]["total"].as_f64(),
                Some(0.0),
                "new order total must be 0"
            );

            // Add two line items.
            call_write(&db, "create_line_item", json!({ "order_id": order_id, "amount": 10.0 }))
                .await;
            call_write(&db, "create_line_item", json!({ "order_id": order_id, "amount": 5.5 }))
                .await;

            // Read the order back — total must equal the sum (read-your-writes).
            let listed = call_read(&db, "list_order", json!({ "id": order_id })).await;
            let rows = listed["result"]["structuredContent"]["rows"]
                .as_array()
                .unwrap_or_else(|| panic!("rows missing; got: {listed}"));
            let total = rows[0]["total"].as_f64().expect("total");
            assert_eq!(total, 15.5, "order total must equal SUM(line_items.amount)");

            // Delete one line item — total must drop.
            let li_rows = call_read(&db, "list_line_item", json!({ "order_id": order_id })).await;
            let li_id = li_rows["result"]["structuredContent"]["rows"][0]["id"]
                .as_i64()
                .unwrap_or_else(|| panic!("line item id missing; got: {li_rows}"));
            call_write(&db, "delete_line_item", json!({ "id": li_id })).await;

            let after = call_read(&db, "list_order", json!({ "id": order_id })).await;
            let total_after = after["result"]["structuredContent"]["rows"][0]["total"]
                .as_f64()
                .expect("total after");
            assert_eq!(
                total_after, 5.5,
                "deleting a line item must recompute the parent total"
            );
        }
    }
}
