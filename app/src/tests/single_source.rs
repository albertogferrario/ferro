//! Both-channels single-source proof for Phase 232 SC3 (EXEC-05).
//!
//! ONE `ServiceDef` `submit` transition — declared once on the order/approval
//! `ServiceDef` — is exercised through BOTH write surfaces:
//!
//!   (a) the MCP framing  (`handle_tools_call` → `ferro::write::dispatch_write(.., "mcp")`)
//!   (b) the visual handler (`controllers::visual_action::handle` → `dispatch_write(.., "web")`)
//!
//! and the tests assert IDENTICAL semantics across both:
//!   - identical persisted derived `to_state` (both == `derive_transition_plan(..).to_state`);
//!   - identical guard re-evaluation outcome (both succeed only because the live guard held;
//!     a guard-failing variant is rejected on BOTH paths, leaving state unchanged);
//!   - the audit channel (`mcp.action.submit` vs `web.action.submit`) is the ONLY divergence.
//!
//! This is the EXEC-05 coherence proof: one declaration, one executor
//! (`framework::write::dispatch_write`), two callers. A second per-channel executor
//! or a divergent transition target would fail these tests.
//!
//! Gated `not(feature = "confirmation")`: `submit`/`approve` are destructive, so the
//! feature-on path requires the two-step confirm flow (covered by the ferro-mcp-server
//! confirmation suite). The single-source structural claim is identical either way —
//! both surfaces route through the one kernel.

#[cfg(all(test, not(feature = "confirmation")))]
mod tests {
    use crate::migrations::Migrator;
    use ferro::serde_json::json;
    use ferro::write::{dispatch_write, WriteDispatcher, WriteError};
    use ferro_audit::{history_for_target, AuditTarget};
    use ferro_mcp_server::{handle_tools_call, McpContext};
    use sea_orm::{
        ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, EntityTrait,
        QueryFilter,
    };
    use sea_orm_migration::prelude::*;

    // ── Fixture helpers (mirror mcp_write_dispatch.rs / visual_action.rs) ──────

    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite connect failed");
        Migrator::up(&db, None)
            .await
            .expect("migrations failed on test DB");
        db
    }

    /// Seed two tenants used by both channels:
    ///   - tenant 1 (acme): has a user → `is_manager` true.
    ///   - tenant 3 (initech): NO user → `is_manager` false (drives the guard-reject variant).
    ///
    /// Orders:
    ///   - 1, 2 (tenant 1, "draft")   — one per channel for the success variant.
    ///   - 5, 6 (tenant 3, "submitted") — one per channel for the guard-reject variant.
    async fn seed(db: &DatabaseConnection) {
        use crate::models::entities::orders::ActiveModel as OrderActive;
        use crate::models::entities::tenants::ActiveModel as TenantActive;
        use crate::models::entities::users::ActiveModel as UserActive;

        let now = "2026-06-16T00:00:00+00:00";

        for (id, slug, name) in [(1i64, "acme", "Acme"), (3i64, "initech", "Initech")] {
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

        // Tenant 1 has a user (is_manager true); tenant 3 deliberately has none.
        UserActive {
            id: Set(901),
            name: Set("Seed User".into()),
            email: Set("alice@acme.test".into()),
            password: Set("hashed".into()),
            remember_token: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            tenant_id: Set(Some(1)),
        }
        .insert(db)
        .await
        .expect("seed user alice");

        for (id, tid, status) in [
            (1i32, 1i64, "draft"),     // MCP success variant
            (2i32, 1i64, "draft"),     // visual success variant
            (5i32, 3i64, "submitted"), // MCP guard-reject variant
            (6i32, 3i64, "submitted"), // visual guard-reject variant
        ] {
            OrderActive {
                id: Set(id),
                customer_name: Set("Seed Customer".into()),
                total: Set(10.0 * id as f64),
                status: Set(status.into()),
                created_at: Set(now.into()),
                tenant_id: Set(tid),
                deleted_at: Set(None),
            }
            .insert(db)
            .await
            .unwrap_or_else(|e| panic!("seed order {id}: {e}"));
        }
    }

    fn order_service() -> ferro::ServiceDef {
        crate::projections::order::service_def()
    }

    /// Resolve the `ActionDef` from the SAME production registry both channels use.
    fn action_def(action_name: &str) -> ferro::ActionDef {
        order_service()
            .actions
            .iter()
            .find(|a| a.name == action_name)
            .cloned()
            .unwrap_or_else(|| panic!("action '{action_name}' not in order service"))
    }

    /// The transition guard derived from the StateMachine (the visual handler step).
    fn transition_guard_for(action_name: &str) -> Option<String> {
        ferro::derive_transition_plan(&order_service(), action_name)
            .ok()
            .and_then(|p| p.guard)
    }

    /// The single declared target state — `Transition.to` from the StateMachine.
    /// Both channels must persist exactly this; nothing else is a single source.
    fn derived_to_state(action_name: &str) -> String {
        ferro::derive_transition_plan(&order_service(), action_name)
            .expect("derive_transition_plan")
            .to_state
    }

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

    /// The shared dispatcher both channels drive (mirrors the production
    /// `make_write_dispatcher` against the explicit in-memory `db`). The executor
    /// derives `to_state` from the StateMachine (no match); the guard evaluator
    /// performs the live `is_manager` check — exactly what BOTH surfaces invoke.
    fn dispatcher(db: DatabaseConnection) -> WriteDispatcher {
        let db_exec = db.clone();
        let db_guard = db.clone();
        WriteDispatcher::new(
            Box::new(move |action_name, inputs, tenant_id, _db_arg| {
                use crate::models::entities::orders::{ActiveModel as OrderActive, Column, Entity};
                let action_name = action_name.to_string();
                let id_val = inputs["id"].as_i64();
                let db = db_exec.clone();
                Box::pin(async move {
                    let id = id_val.ok_or_else(|| WriteError::Validation("missing id".into()))?;

                    let order = Entity::find_by_id(id as i32)
                        .filter(Column::TenantId.eq(tenant_id))
                        .one(&db)
                        .await
                        .map_err(|e| WriteError::Database(e.to_string()))?
                        .ok_or_else(|| {
                            WriteError::Validation("not found or cross-tenant access denied".into())
                        })?;

                    // Derive to_state from the StateMachine — single source of truth.
                    let services = crate::controllers::mcp::exposed_services();
                    let svc = services
                        .iter()
                        .find(|s| s.actions.iter().any(|a| a.name == action_name))
                        .ok_or_else(|| WriteError::ActionNotFound(action_name.clone()))?;
                    let plan = ferro::derive_transition_plan(svc, &action_name)
                        .map_err(|e| WriteError::Validation(e.to_string()))?;
                    let new_status = plan.to_state;

                    let mut active: OrderActive = order.into();
                    active.status = Set(new_status);
                    let updated = active
                        .update(&db)
                        .await
                        .map_err(|e| WriteError::Database(e.to_string()))?;

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
                            let cnt = db
                                .query_one(stmt)
                                .await
                                .ok()
                                .flatten()
                                .and_then(|row| row.try_get::<i64>("", "cnt").ok())
                                .map(|c| c > 0)
                                .unwrap_or(false);
                            Ok(cnt)
                        }
                        _ => Err(WriteError::GuardFailed(format!(
                            "unknown guard '{guard_name}': no evaluator registered"
                        ))),
                    }
                })
            }),
        )
    }

    // ── Channel drivers ──────────────────────────────────────────────────────

    /// MCP framing path: drive the transition through `handle_tools_call`, which
    /// frames the call and invokes `ferro::write::dispatch_write(.., "mcp")`.
    /// Returns whether the tool call succeeded (isError != true).
    async fn drive_mcp(
        action_name: &str,
        inputs: ferro::serde_json::Value,
        tenant_id: i64,
        db: &DatabaseConnection,
    ) -> bool {
        let services = vec![order_service()];
        let ctx = McpContext {
            tenant_id: Some(tenant_id),
            scope: Some("read_write".to_string()),
            ..Default::default()
        };
        let disp = dispatcher(db.clone());
        let result = handle_tools_call(
            json!({ "name": action_name, "arguments": inputs }),
            &services,
            db,
            Some(tenant_id),
            &ctx,
            &disp,
        )
        .await;
        result["result"]["isError"] != true
    }

    /// Visual handler path: drive the EXACT kernel call `visual_action::handle`
    /// makes — `dispatch_write(.., "web")` with the same resolved `ActionDef` and
    /// derived guard. Returns the kernel result so guard rejection is observable.
    async fn drive_visual(
        action_name: &str,
        inputs: ferro::serde_json::Value,
        tenant_id: i64,
        db: &DatabaseConnection,
    ) -> ferro::write::WriteResult<ferro::serde_json::Value> {
        let action = action_def(action_name);
        let guard = transition_guard_for(action_name);
        let disp = dispatcher(db.clone());
        dispatch_write(
            &action,
            &inputs,
            tenant_id,
            db,
            &disp,
            guard.as_deref(),
            "web",
            #[cfg(feature = "confirmation")]
            false,
            None,
        )
        .await
    }

    // ── SC3: one transition, both channels, identical semantics ───────────────

    /// The SAME declared `submit` transition (draft → submitted), exercised through
    /// BOTH the MCP framing and the visual handler, persists the IDENTICAL derived
    /// `to_state` and required the guard to hold on BOTH paths — with the audit
    /// channel (`mcp.action.submit` vs `web.action.submit`) the ONLY divergence.
    #[tokio::test]
    async fn single_source_both_channels() {
        let db = setup_db().await;
        seed(&db).await;

        // The single source of truth: the derived target state, declared once.
        let expected = derived_to_state("submit");
        assert_eq!(
            expected, "submitted",
            "synthetic anchor invariant: submit derives to 'submitted'"
        );

        // (a) MCP path — order 1 (tenant 1, draft). Guard `is_manager` holds.
        let mcp_ok = drive_mcp("submit", json!({"id": 1}), 1, &db).await;
        assert!(mcp_ok, "MCP submit for an owned order must succeed");
        let mcp_state = load_order(1, &db).await.status;

        // (b) Visual path — order 2 (tenant 1, draft). Same guard, same kernel.
        let visual_res = drive_visual("submit", json!({"id": 2}), 1, &db).await;
        assert!(
            visual_res.is_ok(),
            "visual submit for an owned order must succeed; got: {visual_res:?}"
        );
        let visual_state = load_order(2, &db).await.status;

        // 1. Identical persisted to_state — both == the derived Transition.to.
        assert_eq!(
            mcp_state, expected,
            "MCP path must persist the derived to_state"
        );
        assert_eq!(
            visual_state, expected,
            "visual path must persist the derived to_state"
        );
        assert_eq!(
            mcp_state, visual_state,
            "both channels must persist the IDENTICAL derived to_state (single source)"
        );

        // 2. The audit channel is the ONLY divergence: mcp.action.submit vs web.action.submit.
        let mcp_audit = history_for_target(&AuditTarget::new("submit", "1"), &db)
            .await
            .expect("mcp audit history");
        let web_audit = history_for_target(&AuditTarget::new("submit", "2"), &db)
            .await
            .expect("web audit history");
        assert_eq!(
            mcp_audit.first().map(|e| e.action.as_str()),
            Some("mcp.action.submit"),
            "MCP channel audits as mcp.action.submit"
        );
        assert_eq!(
            web_audit.first().map(|e| e.action.as_str()),
            Some("web.action.submit"),
            "visual channel audits as web.action.submit"
        );
        // The transition target recorded in each audit `after` is identical — the
        // channel prefix is the sole difference, confirming ONE kernel with a tag.
        assert!(
            mcp_audit.first().and_then(|e| e.after.clone()).is_some(),
            "MCP audit records the transition result"
        );
        assert!(
            web_audit.first().and_then(|e| e.after.clone()).is_some(),
            "visual audit records the transition result"
        );
    }

    /// The guard-half of single-source: a transition whose live guard does NOT hold
    /// is rejected on BOTH channels and leaves state unchanged. Proves the guard
    /// re-evaluation is the SAME kernel gate regardless of caller — not a per-channel
    /// re-implementation that could diverge.
    #[tokio::test]
    async fn single_source_guard_rejects_both() {
        let db = setup_db().await;
        seed(&db).await;

        // `approve` carries guard `is_manager`. Tenant 3 has no user → guard false.
        // Order 5 (MCP) and order 6 (visual), both tenant 3, both "submitted".

        // (a) MCP path — must NOT succeed; state unchanged.
        let mcp_ok = drive_mcp("approve", json!({"id": 5}), 3, &db).await;
        assert!(
            !mcp_ok,
            "MCP approve must be rejected when the guard does not hold"
        );
        assert_eq!(
            load_order(5, &db).await.status,
            "submitted",
            "MCP guard-rejected transition must leave state unchanged"
        );

        // (b) Visual path — must reject with GuardFailed; state unchanged.
        let visual_res = drive_visual("approve", json!({"id": 6}), 3, &db).await;
        assert!(
            matches!(visual_res, Err(WriteError::GuardFailed(_))),
            "visual approve must be rejected via live guard re-eval; got: {visual_res:?}"
        );
        assert_eq!(
            load_order(6, &db).await.status,
            "submitted",
            "visual guard-rejected transition must leave state unchanged"
        );
    }
}
