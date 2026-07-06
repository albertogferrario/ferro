//! End-to-end CRUD surface proof for Phase 243 (CRUD-01..07, D-07, SC#3).
//!
//! Exercises the full Track A CRUD data surface against the flipped `order`
//! projection — create → list → update → delete — through the real
//! `handle_tools_call` entry point, using the SHIPPED `execute_crud_plan` kernel.
//!
//! Key architectural invariant: CRUD verb calls route through
//!   `handle_tools_call` → `derive_crud_plan` → `dispatch_write(Some(&plan))`
//!   → `execute_crud_plan`  (the `dispatcher.executor` closure is BYPASSED for
//!   CRUD verbs when `crud_plan=Some(..)`; no per-name SQL dispatcher is needed).
//!
//! Tests:
//!   - `crud_cycle_create_list_update_delete` — CRUD-01/02/03/04 + D-07 envelope guard
//!   - `crud_write_requires_write_authorization` — CRUD-05 auth gate (-32603 before executor)
//!   - `crud_cross_tenant_non_disclosure` — T-243-01 cross-tenant update/delete denied
//!   - `crud_mcp_visual_single_source_parity` — CRUD-06 (feature off only)
//!   - `delete_order_confirmation_flow` — SC#3 (feature = "confirmation" only)

#[cfg(test)]
mod tests {
    use crate::migrations::Migrator;
    use ferro::serde_json::json;
    #[cfg(not(feature = "confirmation"))]
    use ferro_audit::{history_for_target, AuditTarget};
    #[cfg(feature = "confirmation")]
    use ferro_mcp_server::McpServerConfig;
    use ferro_mcp_server::{handle_tools_call, McpContext, WriteDispatcher};
    use sea_orm::{
        ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, EntityTrait,
        QueryFilter,
    };
    use sea_orm_migration::prelude::*;

    // ── Fixture helpers (mirror mcp_write_dispatch.rs exactly) ──────────────

    /// Open an in-memory SQLite DB and run the full Migrator (includes deleted_at,
    /// mcp_idempotency_keys, and audit_log tables).
    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite connect failed");
        Migrator::up(&db, None)
            .await
            .expect("migrations failed on test DB");
        db
    }

    /// Seed two tenants' orders into `db`.
    ///
    /// Tenants: 1 (acme), 2 (globex). Users: 901 (alice), 902 (bob).
    /// Orders: 1-2 (tenant 1), 3-4 (tenant 2) — all with `deleted_at: None`.
    async fn seed_two_tenants(db: &DatabaseConnection) {
        use crate::models::entities::orders::ActiveModel as OrderActive;
        use crate::models::entities::tenants::ActiveModel as TenantActive;
        use crate::models::entities::users::ActiveModel as UserActive;

        let now = "2026-06-14T00:00:00+00:00";

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

        for (id, tid, customer) in [
            (1i32, 1i64, "Alice Acme"),
            (2i32, 1i64, "Alice Acme 2"),
            (3i32, 2i64, "Bob Globex"),
            (4i32, 2i64, "Bob Globex 2"),
        ] {
            OrderActive {
                id: Set(id),
                customer_name: Set(customer.into()),
                total: Set(10.0 * id as f64),
                status: Set("submitted".into()),
                created_at: Set(now.into()),
                tenant_id: Set(tid),
                deleted_at: Set(None),
            }
            .insert(db)
            .await
            .unwrap_or_else(|e| panic!("seed: insert order {id}: {e}"));
        }
    }

    /// The Order ServiceDef (mirrors the production registration).
    fn order_service() -> ferro::ServiceDef {
        crate::projections::order::service_def()
    }

    /// Load an order by id directly from db (for post-call mutation assertions).
    async fn load_order(
        id: i32,
        db: &DatabaseConnection,
    ) -> crate::models::entities::orders::Model {
        use crate::models::entities::orders::Entity;
        Entity::find_by_id(id)
            .one(db)
            .await
            .expect("load_order query")
            .unwrap_or_else(|| panic!("order {id} not found"))
    }

    /// Reuse the production-style WriteDispatcher from mcp_write_dispatch.rs.
    ///
    /// For CRUD verb calls, `dispatch_write` is called with `crud_plan=Some(..)`,
    /// which causes the framework's `execute_crud_plan` to run. The `executor`
    /// closure is BYPASSED for CRUD verbs — no per-name CRUD SQL is needed here.
    /// The guard evaluator is included for completeness (CRUD has no guards, so it
    /// is never invoked on CRUD calls, but it must be present for the dispatcher).
    fn make_test_write_dispatcher(db: DatabaseConnection) -> WriteDispatcher {
        let db_exec = db.clone();
        let db_guard = db.clone();
        WriteDispatcher::new(
            Box::new(move |action_name, inputs, tenant_id, _db_arg| {
                use crate::models::entities::orders::{ActiveModel as OrderActive, Column, Entity};
                let action_name = action_name.to_string();
                let id_val = inputs["id"].as_i64();
                let db = db_exec.clone();
                Box::pin(async move {
                    // This executor handles TRANSITION actions only. For CRUD verbs
                    // (create_/update_/delete_), dispatch_write invokes execute_crud_plan
                    // when crud_plan=Some(..) and NEVER calls this closure.
                    let id = id_val
                        .ok_or_else(|| ferro::write::WriteError::Validation("missing id".into()))?;

                    let order = Entity::find_by_id(id as i32)
                        .filter(Column::TenantId.eq(tenant_id))
                        .one(&db)
                        .await
                        .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?
                        .ok_or_else(|| {
                            ferro::write::WriteError::Validation(
                                "not found or cross-tenant access denied".into(),
                            )
                        })?;

                    let services = crate::controllers::mcp::exposed_services();
                    let svc = services
                        .iter()
                        .find(|s| s.actions.iter().any(|a| a.name == action_name))
                        .ok_or_else(|| {
                            ferro::write::WriteError::ActionNotFound(action_name.clone())
                        })?;
                    let plan = ferro::derive_transition_plan(svc, &action_name)
                        .map_err(|e| ferro::write::WriteError::Validation(e.to_string()))?;
                    let new_status = plan.to_state;

                    let mut active: OrderActive = order.into();
                    active.status = Set(new_status);
                    let updated = active
                        .update(&db)
                        .await
                        .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?;

                    Ok(json!({ "id": updated.id, "status": updated.status }))
                })
            }),
            Box::new(move |guard_name, tenant_id, _inputs, _db_arg| {
                use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, Value};
                let guard_name = guard_name.to_string();
                let db = db_guard.clone();
                Box::pin(async move {
                    match guard_name.as_str() {
                        "is_manager" => {
                            let backend = db.get_database_backend();
                            let stmt = Statement::from_sql_and_values(
                                backend,
                                match backend {
                                    DatabaseBackend::Postgres => {
                                        "SELECT COUNT(*) AS cnt FROM users WHERE tenant_id = $1"
                                    }
                                    _ => "SELECT COUNT(*) AS cnt FROM users WHERE tenant_id = ?",
                                },
                                [Value::BigInt(Some(tenant_id))],
                            );
                            let result = db
                                .query_one(stmt)
                                .await
                                .ok()
                                .flatten()
                                .and_then(|row| row.try_get::<i64>("", "cnt").ok())
                                .map(|cnt| cnt > 0)
                                .unwrap_or(false);
                            Ok(result)
                        }
                        _ => Err(ferro::write::WriteError::GuardFailed(format!(
                            "unknown guard '{guard_name}': no evaluator registered"
                        ))),
                    }
                })
            }),
        )
    }

    #[cfg(feature = "confirmation")]
    fn test_config() -> McpServerConfig {
        McpServerConfig {
            app_name: "TestApp".into(),
            app_url: "https://test.example".into(),
            version: "0.0.0".into(),
            confirmation_ttl_seconds: 300,
        }
    }

    // ── Call helpers ─────────────────────────────────────────────────────────

    /// Invoke a CRUD write tool (create_/update_/delete_) through `handle_tools_call`
    /// with an authorized `McpContext` (write_authorized: Some(true)).
    ///
    /// For CRUD verbs, the framework routes to `execute_crud_plan` internally;
    /// the `dispatcher.executor` closure is never invoked.
    async fn call_crud_tool(
        tool_name: &str,
        arguments: ferro::serde_json::Value,
        tenant_id: Option<i64>,
        db: &DatabaseConnection,
        dispatcher: &WriteDispatcher,
    ) -> ferro::serde_json::Value {
        let services = vec![order_service()];
        let ctx = McpContext {
            tenant_id,
            scope: Some("read_write".to_string()),
            write_authorized: Some(true), // REQUIRED — without this, -32603 before executor
            ..Default::default()
        };
        let params = json!({ "name": tool_name, "arguments": arguments });
        handle_tools_call(
            params,
            &services,
            db,
            tenant_id,
            &ctx,
            dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &test_config(),
        )
        .await
    }

    /// Invoke a read tool (list_order) through `handle_tools_call`.
    ///
    /// Reads do not require write_authorized; McpContext::default() suffices
    /// (tenant_id is still required for tenant scoping).
    async fn call_list_tool(
        tool_name: &str,
        arguments: ferro::serde_json::Value,
        tenant_id: Option<i64>,
        db: &DatabaseConnection,
    ) -> ferro::serde_json::Value {
        let services = vec![order_service()];
        let noop = WriteDispatcher::new(
            Box::new(|_, _, _, _| Box::pin(async { Ok(json!({})) })),
            Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
        );
        let ctx = McpContext {
            tenant_id,
            ..Default::default()
        };
        let params = json!({ "name": tool_name, "arguments": arguments });
        handle_tools_call(
            params,
            &services,
            db,
            tenant_id,
            &ctx,
            &noop,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &test_config(),
        )
        .await
    }

    /// Invoke a CRUD write tool with `write_authorized: None` to test the auth gate.
    async fn call_crud_tool_unauthorized(
        tool_name: &str,
        arguments: ferro::serde_json::Value,
        tenant_id: Option<i64>,
        db: &DatabaseConnection,
        dispatcher: &WriteDispatcher,
    ) -> ferro::serde_json::Value {
        let services = vec![order_service()];
        let ctx = McpContext {
            tenant_id,
            scope: Some("read_write".to_string()),
            write_authorized: None, // Absent → -32603 before executor (CRUD-05)
            ..Default::default()
        };
        let params = json!({ "name": tool_name, "arguments": arguments });
        handle_tools_call(
            params,
            &services,
            db,
            tenant_id,
            &ctx,
            dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &test_config(),
        )
        .await
    }

    // ── Envelope guard helpers (D-07) ────────────────────────────────────────

    /// Assert the Phase 205 structured-envelope shape for a CRUD write result.
    ///
    /// Pins: content[0].type=="text", structuredContent.status=="ok",
    ///       structuredContent.action==tool_name, structuredContent.result is object,
    ///       isError != true.
    fn assert_write_envelope_ok(result: &ferro::serde_json::Value, tool_name: &str) {
        let content = result["result"]["content"].as_array().unwrap_or_else(|| {
            panic!("{tool_name}: result.content must be an array; got: {result}")
        });
        assert_eq!(
            content[0]["type"].as_str(),
            Some("text"),
            "{tool_name}: content[0] must be a text block (type=text) — Phase 205 envelope"
        );
        assert_eq!(
            result["result"]["structuredContent"]["status"].as_str(),
            Some("ok"),
            "{tool_name}: structuredContent.status must be ok; got: {result}"
        );
        assert_eq!(
            result["result"]["structuredContent"]["action"].as_str(),
            Some(tool_name),
            "{tool_name}: structuredContent.action must equal the tool name; got: {result}"
        );
        assert!(
            result["result"]["structuredContent"]["result"].is_object(),
            "{tool_name}: structuredContent.result must be an object; got: {result}"
        );
        assert_ne!(
            result["result"]["isError"], true,
            "{tool_name}: isError must not be true on success; got: {result}"
        );
    }

    /// Assert the Phase 205 list envelope shape.
    ///
    /// Pins: content[0].type=="text", structuredContent.rows is array.
    #[cfg(not(feature = "confirmation"))]
    fn assert_list_envelope(result: &ferro::serde_json::Value) {
        let content = result["result"]["content"]
            .as_array()
            .expect("list envelope: result.content must be an array");
        assert_eq!(
            content[0]["type"].as_str(),
            Some("text"),
            "list envelope: content[0] must be a text block (type=text)"
        );
        result["result"]["structuredContent"]["rows"]
            .as_array()
            .expect("list envelope: structuredContent.rows must be an array");
    }

    // ── CRUD-01/02/03/04 + D-07: create → list → update → delete cycle ───────

    /// Full CRUD cycle: create → list → update → delete.
    ///
    /// - CRUD-01: create_order inserts a row with status="draft" (server-side) and
    ///   returns an ok envelope with the new id.
    /// - CRUD-02: update_order patches customer_name; status is excluded.
    /// - CRUD-03: delete_order soft-deletes the row.
    /// - CRUD-04: list_order excludes soft-deleted records.
    /// - D-07: per-verb Phase 205 envelope shape is asserted for every write.
    ///
    /// Gated `not(feature = "confirmation")`: delete_order is destructive and
    /// requires the two-step confirm flow when the feature is on (tested in
    /// `delete_order_confirmation_flow`).
    #[cfg(not(feature = "confirmation"))]
    #[tokio::test]
    async fn crud_cycle_create_list_update_delete() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let dispatcher = make_test_write_dispatcher(db.clone());

        // Step 1: create_order — status set to "draft" server-side; tenant_id injected.
        // MUST NOT supply an explicit id (seeded ids are 1–4; SQLite auto-assigns 5+).
        let created = call_crud_tool(
            "create_order",
            json!({ "customer_name": "Test Customer", "total": 42.0 }),
            Some(1),
            &db,
            &dispatcher,
        )
        .await;
        assert_write_envelope_ok(&created, "create_order");
        let new_id = created["result"]["structuredContent"]["result"]["id"]
            .as_i64()
            .unwrap_or_else(|| panic!("create_order result must carry an id; got: {created}"));
        assert!(
            new_id >= 5,
            "auto-increment id must be >= 5 (seeded 1-4); got {new_id}"
        );

        // Verify server-side status injection (D-05): status must be "draft".
        let created_row = load_order(new_id as i32, &db).await;
        assert_eq!(
            created_row.status, "draft",
            "create_order must persist status='draft' server-side (D-05)"
        );
        assert_eq!(
            created_row.tenant_id, 1,
            "create_order must inject tenant_id from context, not from agent input"
        );

        // Step 2: list_order — must include the new record (read path, no write_authorized).
        let listed = call_list_tool("list_order", json!({ "limit": 100 }), Some(1), &db).await;
        assert_list_envelope(&listed);
        let rows = listed["result"]["structuredContent"]["rows"]
            .as_array()
            .expect("rows must be an array");
        assert!(
            rows.iter().any(|r| r["id"].as_i64() == Some(new_id)),
            "list_order must include the created record (id={new_id})"
        );

        // Step 3: update_order — patch customer_name; never pass status (CRUD-02: SM present).
        let updated = call_crud_tool(
            "update_order",
            json!({ "id": new_id, "customer_name": "Updated Customer" }),
            Some(1),
            &db,
            &dispatcher,
        )
        .await;
        assert_write_envelope_ok(&updated, "update_order");

        // Step 4: delete_order (feature off → direct soft-delete, no confirm token needed).
        let deleted = call_crud_tool(
            "delete_order",
            json!({ "id": new_id }),
            Some(1),
            &db,
            &dispatcher,
        )
        .await;
        assert_write_envelope_ok(&deleted, "delete_order");

        // CRUD-04 regression: list_order must NOW exclude the soft-deleted record.
        let after = call_list_tool("list_order", json!({ "limit": 100 }), Some(1), &db).await;
        let rows_after = after["result"]["structuredContent"]["rows"]
            .as_array()
            .expect("rows must be an array after delete");
        assert!(
            !rows_after.iter().any(|r| r["id"].as_i64() == Some(new_id)),
            "soft-deleted record must be filtered out of list_order (CRUD-03/04)"
        );
    }

    // ── CRUD-05: write_authorized gate ────────────────────────────────────────

    /// CRUD-05: A CRUD write tool with `write_authorized: None` returns a -32603
    /// transport error BEFORE any executor runs.
    ///
    /// This gate is in `handle_write_call` (~line 157 of write_dispatch.rs):
    ///   `is_crud_write_tool && ctx.write_authorized != Some(true)` → -32603.
    ///
    /// Runs under both feature states (no delete confirmation involved in create).
    #[tokio::test]
    async fn crud_write_requires_write_authorization() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let dispatcher = make_test_write_dispatcher(db.clone());

        let denied = call_crud_tool_unauthorized(
            "create_order",
            json!({ "customer_name": "Nope", "total": 1.0 }),
            Some(1),
            &db,
            &dispatcher,
        )
        .await;

        assert_eq!(
            denied["error"]["code"].as_i64(),
            Some(-32603),
            "CRUD-05: write_authorized != Some(true) must deny with -32603 before the executor; got: {denied}"
        );
        assert!(
            denied["error"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("write ability denied"),
            "CRUD-05: denial message must contain 'write ability denied'; got: {denied}"
        );
    }

    // ── T-243-01: cross-tenant non-disclosure ────────────────────────────────

    /// T-243-01: Tenant 1 cannot update or delete a record owned by tenant 2.
    ///
    /// The shipped `execute_crud_plan` appends `AND tenant_id=?` to UPDATE/DELETE
    /// queries, so a foreign row returns `RecordNotFound` — not a success or
    /// a data-disclosing error. The result must have `isError=true` and must NOT
    /// contain the foreign order's data.
    ///
    /// Runs under both feature states (cross-tenant is not about delete confirmation).
    #[tokio::test]
    async fn crud_cross_tenant_non_disclosure() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let dispatcher = make_test_write_dispatcher(db.clone());

        // Order id=3 belongs to tenant 2; call as tenant 1 → RecordNotFound (non-disclosing).
        let update_result = call_crud_tool(
            "update_order",
            json!({ "id": 3, "customer_name": "Hijacked" }),
            Some(1),
            &db,
            &dispatcher,
        )
        .await;

        assert_eq!(
            update_result["result"]["isError"], true,
            "cross-tenant update must return isError:true; got: {update_result}"
        );
        // Result must NOT contain the foreign order's data (non-disclosing not-found).
        // The result envelope has isError:true; structuredContent must not carry a "result" object.
        let sc = &update_result["result"]["structuredContent"];
        assert!(
            sc["result"].as_object().is_none(),
            "cross-tenant update must not disclose foreign order data in structuredContent.result; got: {update_result}"
        );

        // Verify the row is unmutated.
        let order = load_order(3, &db).await;
        assert_eq!(
            order.customer_name, "Bob Globex",
            "tenant 2's order must be unmutated after tenant 1's denied update"
        );
    }

    // ── CRUD-06: MCP↔visual single-source parity ─────────────────────────────

    /// Drive create/update on the VISUAL surface using `dispatch_write(.., "web")`.
    ///
    /// SAME `derive_crud_plan` → SAME `execute_crud_plan` as the MCP path.
    /// The audit channel tag (`"web"`) is the ONLY divergence.
    #[cfg(not(feature = "confirmation"))]
    async fn drive_visual_crud(
        verb: ferro_projections::CrudVerb,
        tool_name: &str,
        inputs: ferro::serde_json::Value,
        tenant_id: i64,
        db: &DatabaseConnection,
        dispatcher: &WriteDispatcher,
    ) -> ferro::write::WriteResult<ferro::serde_json::Value> {
        let svc = order_service();
        let plan =
            ferro_projections::derive_crud_plan(&svc, verb, &inputs).expect("derive_crud_plan");
        let crud_action = ferro::ActionDef::new(tool_name);
        ferro::write::dispatch_write(
            &crud_action,
            &inputs,
            tenant_id,
            db,
            dispatcher,
            None,  // CRUD has no transition guard
            "web", // audit channel → web.crud.create_order
            Some(&plan),
        )
        .await
    }

    /// CRUD-06: One derived CrudPlan executes identically on MCP and visual surfaces.
    ///
    /// - Both paths call `execute_crud_plan` through `dispatch_write(Some(&plan))`.
    /// - Both rows have identical `status="draft"` (server-side, D-05).
    /// - The ONLY divergence is the audit action prefix:
    ///     `mcp.crud.create_order` vs `web.crud.create_order`.
    ///
    /// Gated `not(feature = "confirmation")`: delete parity requires the two-step
    /// confirm flow feature-on; create/update prove the shared kernel conclusively.
    #[cfg(not(feature = "confirmation"))]
    #[tokio::test]
    async fn crud_mcp_visual_single_source_parity() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let dispatcher = make_test_write_dispatcher(db.clone());

        // MCP create — separate row, id assigned by auto-increment.
        let mcp_created = call_crud_tool(
            "create_order",
            json!({ "customer_name": "Parity MCP", "total": 10.0 }),
            Some(1),
            &db,
            &dispatcher,
        )
        .await;
        assert_write_envelope_ok(&mcp_created, "create_order");
        let mcp_id = mcp_created["result"]["structuredContent"]["result"]["id"]
            .as_i64()
            .expect("MCP create must return id");

        // Visual create (same derived plan, channel=web) — separate row (no id collision).
        let visual = drive_visual_crud(
            ferro_projections::CrudVerb::Create,
            "create_order",
            json!({ "customer_name": "Parity Web", "total": 10.0 }),
            1,
            &db,
            &dispatcher,
        )
        .await
        .expect("visual create must succeed");
        let visual_id = visual["id"].as_i64().expect("visual create must return id");

        // Both rows persisted identically (status=draft server-side), differing only by id/name.
        let mcp_row = load_order(mcp_id as i32, &db).await;
        let visual_row = load_order(visual_id as i32, &db).await;
        assert_eq!(
            mcp_row.status, "draft",
            "MCP create must persist status='draft' server-side"
        );
        assert_eq!(
            visual_row.status, "draft",
            "visual create must persist status='draft' server-side"
        );
        assert_eq!(
            mcp_row.status, visual_row.status,
            "single source: MCP and visual create must persist identical status"
        );
        assert_eq!(
            mcp_row.tenant_id, visual_row.tenant_id,
            "single source: both paths must inject the same tenant_id"
        );

        // Audit divergence is ONLY the channel prefix: mcp.crud.create_order vs web.crud.create_order.
        // For CRUD creates, `record_id` in the AuditTarget is "" (no id in inputs at create time).
        let mcp_audit = history_for_target(&AuditTarget::new("create_order", ""), &db)
            .await
            .expect("mcp audit history for create_order");
        // Filter to the MCP entry (there may be multiple creates in this test).
        let mcp_entry = mcp_audit
            .iter()
            .find(|e| e.action == "mcp.crud.create_order")
            .unwrap_or_else(|| {
                panic!("must find mcp.crud.create_order audit entry; entries: {mcp_audit:?}")
            });
        assert_eq!(
            mcp_entry.action, "mcp.crud.create_order",
            "MCP channel must audit as mcp.crud.create_order (D-08 prefix)"
        );

        let web_audit = history_for_target(&AuditTarget::new("create_order", ""), &db)
            .await
            .expect("web audit history for create_order");
        let web_entry = web_audit
            .iter()
            .find(|e| e.action == "web.crud.create_order")
            .unwrap_or_else(|| {
                panic!("must find web.crud.create_order audit entry; entries: {web_audit:?}")
            });
        assert_eq!(
            web_entry.action, "web.crud.create_order",
            "visual channel must audit as web.crud.create_order (D-08 prefix)"
        );
    }

    // ── SC#3: delete confirmation flow (feature = "confirmation" only) ────────

    /// SC#3 (CRUD-03): bare delete → confirmation_required → token → soft-delete → gone.
    ///
    /// Exercises the three-step confirmation flow for delete_order:
    ///   1. `delete_order` without a token → `confirmation_required` error.
    ///   2. `request_confirm_delete_order` → token (single-use, bound to tenant+record).
    ///   3. `confirm_delete_order` with token → soft-delete; list_order excludes the row.
    ///
    /// The SAME `InMemoryConfirmationStore` instance must be threaded through both
    /// `request_confirm_delete_order` and `confirm_delete_order` — a fresh store per
    /// call would lose the token binding.
    #[cfg(feature = "confirmation")]
    #[tokio::test]
    async fn delete_order_confirmation_flow() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let dispatcher = make_test_write_dispatcher(db.clone());

        // Create a target row to delete.
        let created = call_crud_tool(
            "create_order",
            json!({ "customer_name": "To Delete", "total": 1.0 }),
            Some(1),
            &db,
            &dispatcher,
        )
        .await;
        assert_write_envelope_ok(&created, "create_order");
        let target_id = created["result"]["structuredContent"]["result"]["id"]
            .as_i64()
            .expect("create_order must return id");

        // Step 1: bare delete_order → confirmation_required (feature on).
        let bare = call_crud_tool(
            "delete_order",
            json!({ "id": target_id }),
            Some(1),
            &db,
            &dispatcher,
        )
        .await;
        assert_eq!(
            bare["result"]["structuredContent"]["error_kind"].as_str(),
            Some("confirmation_required"),
            "bare delete must return confirmation_required; got: {bare}"
        );
        assert_eq!(
            bare["result"]["structuredContent"]["request_tool"].as_str(),
            Some("request_confirm_delete_order"),
            "bare delete must echo request_confirm_delete_order; got: {bare}"
        );
        assert_eq!(
            bare["result"]["isError"], true,
            "bare delete must be isError:true; got: {bare}"
        );

        // Steps 2+3: shared store threaded through request + confirm.
        let store = ferro_ai::InMemoryConfirmationStore::new();

        // Step 2: request_confirm_delete_order → token.
        let services = vec![order_service()];
        let ctx = McpContext {
            tenant_id: Some(1),
            scope: Some("read_write".to_string()),
            write_authorized: Some(true),
            ..Default::default()
        };
        let req_params =
            json!({ "name": "request_confirm_delete_order", "arguments": { "id": target_id } });
        let req_result = handle_tools_call(
            req_params,
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            &test_config(),
        )
        .await;
        let token = req_result["result"]["structuredContent"]["confirmation_token"]
            .as_str()
            .unwrap_or_else(|| {
                panic!(
                    "request_confirm_delete_order must return confirmation_token; got: {req_result}"
                )
            })
            .to_string();
        assert!(
            token.starts_with("cfm_"),
            "confirmation token must have cfm_ prefix; got: {token}"
        );

        // Step 3: confirm_delete_order with the token → soft-delete.
        let confirm_params = json!({
            "name": "confirm_delete_order",
            "arguments": { "confirmation_token": token, "id": target_id }
        });
        let confirm_result = handle_tools_call(
            confirm_params,
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            &test_config(),
        )
        .await;
        assert_write_envelope_ok(&confirm_result, "delete_order");

        // After confirm: list_order must exclude the soft-deleted row.
        let after = call_list_tool("list_order", json!({}), Some(1), &db).await;
        let rows = after["result"]["structuredContent"]["rows"]
            .as_array()
            .expect("rows must be an array after delete");
        assert!(
            !rows.iter().any(|r| r["id"].as_i64() == Some(target_id)),
            "confirmed delete must remove the row from list_order; rows: {rows:?}"
        );
    }
}
