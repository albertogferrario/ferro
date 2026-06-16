//! End-to-end write dispatch fixtures for Phase 219 success criteria.
//!
//! SC#2: Cross-tenant write denied — tenant A targeting tenant B's order is
//!       rejected with isError:true; tenant B's record is unmutated (T-219-01).
//!
//! SC#3: Idempotency e2e — two identical calls with the same idempotency_key
//!       through the full app path produce exactly one DB mutation.
//!
//! SC#4: Audit trail — a successful write call produces a recoverable
//!       ferro-audit entry with tool name, tenant_id, action, and record id.
//!
//! The harness mirrors `mcp_tenant_isolation.rs` exactly:
//!   - `setup_db()` → in-memory SQLite + full `Migrator::up` (which now includes
//!     `CreateMcpIdempotencyKeysTable` and `CreateAuditLogTable`)
//!   - `seed_two_tenants()` → 4 orders, 2 per tenant, explicit ids
//!   - Test-local `WriteDispatcher` closures use the explicit `db` arg so tests
//!     are fully isolated from the global connection pool.

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
    #[cfg(not(feature = "confirmation"))]
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[cfg(not(feature = "confirmation"))]
    use std::sync::Arc;

    // ── Fixture helpers ──────────────────────────────────────────────────────

    /// Open an in-memory SQLite DB and run the full Migrator.
    ///
    /// `Migrator` now includes `CreateMcpIdempotencyKeysTable` and
    /// `CreateAuditLogTable`, so `dispatch_write`'s idempotency store and
    /// `AuditEntry::write` both have tables to write to.
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
    /// Order ids: 1–2 (tenant 1), 3–4 (tenant 2).
    /// Initial status: "submitted" (matching existing seed pattern).
    async fn seed_two_tenants(db: &DatabaseConnection) {
        use crate::models::entities::orders::ActiveModel as OrderActive;
        use crate::models::entities::tenants::ActiveModel as TenantActive;
        use crate::models::entities::users::ActiveModel as UserActive;

        let now = "2026-06-14T00:00:00+00:00";

        // Tenants
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

        // One user per tenant (needed for the live is_manager check)
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

        // Orders (2 per tenant, explicit ids)
        for (id, tid, customer) in [
            (1i32, 1i64, "Alice Acme"),
            (2i32, 1i64, "Alice Acme"),
            (3i32, 2i64, "Bob Globex"),
            (4i32, 2i64, "Bob Globex"),
        ] {
            OrderActive {
                id: Set(id),
                customer_name: Set(customer.into()),
                total: Set(10.0 * id as f64),
                status: Set("submitted".into()),
                created_at: Set(now.into()),
                tenant_id: Set(tid),
            }
            .insert(db)
            .await
            .unwrap_or_else(|e| panic!("seed: insert order {id}: {e}"));
        }
    }

    /// The Order ServiceDef used in dispatch calls (mirrors the production registration).
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

    /// Build a test `WriteDispatcher` that applies state transitions using the
    /// explicit `db` arg (not the global connection pool).
    ///
    /// The `db` clone is Arc-backed (cheap). The closures capture no external
    /// state beyond the cloned `db` — matching PITFALLS §4.
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
                    let id = id_val
                        .ok_or_else(|| ferro_mcp_server::Error::Validation("missing id".into()))?;

                    // find_for_tenant inline: filter by id AND tenant_id (D-03 cross-tenant denial).
                    let order = Entity::find_by_id(id as i32)
                        .filter(Column::TenantId.eq(tenant_id))
                        .one(&db)
                        .await
                        .map_err(|e| ferro_mcp_server::Error::Database(e.to_string()))?
                        .ok_or_else(|| {
                            ferro_mcp_server::Error::Validation(
                                "not found or cross-tenant access denied".into(),
                            )
                        })?;

                    // Derive new_status from the declared StateMachine (no match).
                    let services = crate::controllers::mcp::exposed_services();
                    let svc = services
                        .iter()
                        .find(|s| s.actions.iter().any(|a| a.name == action_name))
                        .ok_or_else(|| {
                            ferro_mcp_server::Error::ActionNotFound(action_name.clone())
                        })?;
                    let plan = ferro::derive_transition_plan(svc, &action_name)
                        .map_err(|e| ferro_mcp_server::Error::Validation(e.to_string()))?;
                    let new_status = plan.to_state;

                    let mut active: OrderActive = order.into();
                    active.status = Set(new_status);
                    let updated = active
                        .update(&db)
                        .await
                        .map_err(|e| ferro_mcp_server::Error::Database(e.to_string()))?;

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
                            // Live DB check — never reads ctx.evaluated_guards (D-02).
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
                        // Fail-closed: unknown guard names deny, not allow.
                        // Any ActionDef referencing an unregistered guard name is a
                        // configuration error; silently passing it inverts fail-closed.
                        _ => Err(ferro_mcp_server::Error::GuardFailed(format!(
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

    /// Invoke a write tool through the full `handle_tools_call` path.
    async fn call_write_tool(
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

    // ── EXEC-01 (end-to-end): derived to_state, no hand-written match ─────────

    /// A `submit` write persists the DERIVED target state `"submitted"` —
    /// sourced from `derive_transition_plan(...).to_state` (Transition.to), with
    /// no `match action_name` anywhere in the path. Seeds the order as `"draft"`
    /// so the transition is observable (draft → submit → submitted).
    ///
    /// Gated feature-off: `submit` is destructive, so feature-on requires the
    /// two-step confirm flow (covered by the ferro-mcp-server confirmation tests).
    #[cfg(not(feature = "confirmation"))]
    #[tokio::test]
    async fn submit_persists_derived_to_state() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;

        // Move order 1 (tenant 1) to the "draft" source state.
        {
            use crate::models::entities::orders::ActiveModel as OrderActive;
            let mut active: OrderActive = load_order(1, &db).await.into();
            active.status = Set("draft".into());
            active
                .update(&db)
                .await
                .expect("seed: set order 1 to draft");
        }

        let dispatcher = make_test_write_dispatcher(db.clone());
        let result = call_write_tool("submit", json!({"id": 1}), Some(1), &db, &dispatcher).await;

        assert_ne!(
            result["result"]["isError"], true,
            "submit for owned order must succeed; got: {result}"
        );

        // The persisted status is the derived Transition.to ("submitted"), not a
        // hand-written value.
        let order = load_order(1, &db).await;
        assert_eq!(
            order.status, "submitted",
            "submit must persist the derived to_state 'submitted'; got: {}",
            order.status
        );
    }

    // ── SC#2: Cross-tenant write denied (T-219-01) ───────────────────────────

    /// SC#2 (T-219-01): Tenant A targeting tenant B's order is denied.
    ///
    /// The executor calls `find_for_tenant(id, tenant_id)` which returns `None`
    /// when `tenant_id` does not match the record's `tenant_id` column — this is
    /// the BOLA prevention primitive (D-03).
    ///
    /// Assertions:
    ///   1. Result has `isError == true`.
    ///   2. Tenant B's order status is UNCHANGED in the DB (no mutation occurred).
    #[tokio::test]
    async fn cross_tenant_write_denied() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let dispatcher = make_test_write_dispatcher(db.clone());

        // Order id=3 belongs to tenant 2; call as tenant 1 → find_for_tenant → None → denial.
        let result = call_write_tool("submit", json!({"id": 3}), Some(1), &db, &dispatcher).await;

        // Must be a tool-level error (isError:true in result, not a JSON-RPC -32xxx error).
        assert_eq!(
            result["result"]["isError"], true,
            "cross-tenant write must return isError:true; got result: {result}"
        );

        // Tenant B's order must be unmutated — status stays "submitted".
        let order = load_order(3, &db).await;
        assert_eq!(
            order.status, "submitted",
            "tenant B order must not be mutated by tenant A's denied write; got: {}",
            order.status
        );
    }

    // ── SC#4: Audit trail ────────────────────────────────────────────────────

    /// SC#4: A successful write call produces a recoverable ferro-audit entry.
    ///
    /// The `dispatch_write` pipeline (Plan 01) calls `AuditEntry::write` after
    /// every successful execution. This test recovers the entry via
    /// `history_for_target` and asserts: action name, tenant_id, and after present.
    ///
    /// Gated: with the confirmation feature on, `submit` (a destructive action) requires
    /// the two-step confirm flow before executing. The Phase 220 SC#1–#4 tests in
    /// ferro-mcp-server cover the confirmed-execution → audit-trail path feature-on.
    #[cfg(not(feature = "confirmation"))]
    #[tokio::test]
    async fn write_call_produces_audit_entry() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;
        let dispatcher = make_test_write_dispatcher(db.clone());

        // Call submit for order 1 (tenant 1 owns it).
        let result = call_write_tool("submit", json!({"id": 1}), Some(1), &db, &dispatcher).await;

        // Call must succeed before we look for the audit trail.
        // CallToolResult::structured sets isError:false explicitly.
        assert_ne!(
            result["result"]["isError"], true,
            "submit for owned order must succeed (isError must not be true); got: {result}"
        );

        // Recover the audit entry written by dispatch_write.
        // target_kind = action name ("submit"), target_id = record id ("1").
        let entries = history_for_target(&AuditTarget::new("submit", "1"), &db)
            .await
            .expect("history_for_target must not fail");

        assert!(
            !entries.is_empty(),
            "audit entry must be written after write call; history is empty"
        );

        let entry = &entries[0];
        assert_eq!(
            entry.action, "mcp.action.submit",
            "audit entry action must be 'mcp.action.submit', got: {}",
            entry.action
        );
        assert_eq!(
            entry.tenant_id,
            Some("1".to_string()),
            "audit entry tenant_id must be Some(\"1\"), got: {:?}",
            entry.tenant_id
        );
        assert!(
            entry.after.is_some(),
            "audit entry after must contain execution result"
        );
    }

    // ── SC#3: Idempotency e2e ────────────────────────────────────────────────

    /// SC#3 (T-219-03): Two identical calls with the same idempotency_key produce
    /// exactly one DB mutation.
    ///
    /// An `AtomicUsize` counter increments every time the executor runs. After two
    /// calls with the same key the counter must equal 1 (replay skipped execution).
    /// The DB order status must reflect exactly one transition, not two.
    ///
    /// Gated: with the confirmation feature on, `submit` (a destructive action) requires
    /// the two-step confirm flow before executing. The idempotency layer is exercised
    /// inside dispatch_write which is reached only after confirmation; the Phase 220
    /// SC#2 (exactly-once) tests in ferro-mcp-server cover the confirmed path feature-on.
    #[cfg(not(feature = "confirmation"))]
    #[tokio::test]
    async fn idempotent_write_e2e() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;

        let exec_count = Arc::new(AtomicUsize::new(0));
        let db_exec = db.clone();
        let counter = exec_count.clone();

        // Dispatcher with a counting executor.
        let dispatcher = WriteDispatcher::new(
            Box::new(move |action_name, inputs, tenant_id, _db_arg| {
                use crate::models::entities::orders::{ActiveModel as OrderActive, Column, Entity};
                let action_name = action_name.to_string();
                let id_val = inputs["id"].as_i64();
                let db = db_exec.clone();
                let counter = counter.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);

                    let id = id_val
                        .ok_or_else(|| ferro_mcp_server::Error::Validation("missing id".into()))?;

                    let order = Entity::find_by_id(id as i32)
                        .filter(Column::TenantId.eq(tenant_id))
                        .one(&db)
                        .await
                        .map_err(|e| ferro_mcp_server::Error::Database(e.to_string()))?
                        .ok_or_else(|| {
                            ferro_mcp_server::Error::Validation(
                                "not found or cross-tenant access denied".into(),
                            )
                        })?;

                    // Derive new_status from the declared StateMachine (no match).
                    let services = crate::controllers::mcp::exposed_services();
                    let svc = services
                        .iter()
                        .find(|s| s.actions.iter().any(|a| a.name == action_name))
                        .ok_or_else(|| {
                            ferro_mcp_server::Error::ActionNotFound(action_name.clone())
                        })?;
                    let plan = ferro::derive_transition_plan(svc, &action_name)
                        .map_err(|e| ferro_mcp_server::Error::Validation(e.to_string()))?;
                    let new_status = plan.to_state;

                    let mut active: OrderActive = order.into();
                    active.status = Set(new_status);
                    let updated = active
                        .update(&db)
                        .await
                        .map_err(|e| ferro_mcp_server::Error::Database(e.to_string()))?;

                    Ok(json!({ "id": updated.id, "status": updated.status }))
                })
            }),
            Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
        );

        let args = json!({ "id": 2, "idempotency_key": "e2e-idem-key-001" });

        let services = vec![order_service()];
        let ctx = McpContext {
            tenant_id: Some(1),
            scope: Some("read_write".to_string()),
            ..Default::default()
        };

        let result1 = handle_tools_call(
            json!({ "name": "submit", "arguments": args }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &test_config(),
        )
        .await;

        let result2 = handle_tools_call(
            json!({ "name": "submit", "arguments": args }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &test_config(),
        )
        .await;

        // Both calls must succeed (not isError:true).
        assert_ne!(
            result1["result"]["isError"], true,
            "first call must succeed; got: {result1}"
        );
        assert_ne!(
            result2["result"]["isError"], true,
            "second call (replay) must succeed; got: {result2}"
        );

        // Idempotent replay: both structured content payloads must be equal.
        assert_eq!(
            result1["result"]["structuredContent"], result2["result"]["structuredContent"],
            "idempotent replay must return identical structured content"
        );

        // Executor must have fired exactly once (SC#3 — single mutation proof).
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            1,
            "executor must fire exactly once after two identical calls with same idempotency_key"
        );
    }
}
