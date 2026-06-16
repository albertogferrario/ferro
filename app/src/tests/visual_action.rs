//! Visual/form write-surface fixtures for Phase 232 SC2 (EXEC-05).
//!
//! These prove the `POST /{service}/{action}` handler (`controllers::visual_action`)
//! drives the SAME `framework::write` kernel as the MCP path, with the audit
//! channel set to `"web"`. Each test exercises the EXACT kernel call the handler
//! makes — `ferro::write::dispatch_write(action, &inputs, tenant_id, db,
//! &dispatcher, transition_guard, "web", ..)` — using the reused dispatcher
//! closures (mirrored here against the explicit in-memory `db`, as the MCP
//! fixtures do, so tests are isolated from the global connection pool).
//!
//! SC2 coverage:
//!   - `visual_action_drives_derived_transition` — derived to_state persisted via the shared kernel.
//!   - `visual_guard_rejects_illegal_transition` — live guard re-eval rejects; state unchanged.
//!   - `visual_audit_channel_is_web` — audit prefix is `web.action.*`, NOT `mcp.action.*`.
//!   - `visual_cross_tenant_denied` — tenant from auth; cross-tenant write denied, unchanged.
//!   - `visual_action_rejects_form_supplied_to_state` — to_state from Transition.to only.

#[cfg(all(test, not(feature = "confirmation")))]
mod tests {
    use crate::migrations::Migrator;
    use ferro::serde_json::json;
    use ferro::write::{dispatch_write, WriteDispatcher, WriteError};
    use ferro_audit::{history_for_target, AuditTarget};
    use sea_orm::{
        ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, EntityTrait,
        QueryFilter,
    };
    use sea_orm_migration::prelude::*;

    // ── Fixture helpers (mirror mcp_write_dispatch.rs) ───────────────────────

    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite connect failed");
        Migrator::up(&db, None)
            .await
            .expect("migrations failed on test DB");
        db
    }

    /// Seed three tenants:
    ///   - tenant 1 (acme): has a user → `is_manager` true.
    ///   - tenant 2 (globex): has a user → `is_manager` true.
    ///   - tenant 3 (initech): NO user → `is_manager` false (drives the guard-reject test).
    ///
    /// Orders: 1–2 (tenant 1, "draft"), 3 (tenant 2, "submitted"), 5 (tenant 3, "submitted").
    async fn seed(db: &DatabaseConnection) {
        use crate::models::entities::orders::ActiveModel as OrderActive;
        use crate::models::entities::tenants::ActiveModel as TenantActive;
        use crate::models::entities::users::ActiveModel as UserActive;

        let now = "2026-06-16T00:00:00+00:00";

        for (id, slug, name) in [
            (1i64, "acme", "Acme"),
            (2i64, "globex", "Globex"),
            (3i64, "initech", "Initech"),
        ] {
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

        // One user each for tenants 1 and 2; tenant 3 deliberately has none.
        for (uid, tid, email) in [
            (901i32, 1i64, "alice@acme.test"),
            (902, 2, "bob@globex.test"),
        ] {
            UserActive {
                id: Set(uid),
                name: Set("Seed User".into()),
                email: Set(email.into()),
                password: Set("hashed".into()),
                remember_token: Set(None),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
                tenant_id: Set(Some(tid)),
            }
            .insert(db)
            .await
            .unwrap_or_else(|e| panic!("seed user {email}: {e}"));
        }

        for (id, tid, status) in [
            (1i32, 1i64, "draft"),
            (2i32, 1i64, "draft"),
            (3i32, 2i64, "submitted"),
            (5i32, 3i64, "submitted"),
        ] {
            OrderActive {
                id: Set(id),
                customer_name: Set("Seed Customer".into()),
                total: Set(10.0 * id as f64),
                status: Set(status.into()),
                created_at: Set(now.into()),
                tenant_id: Set(tid),
            }
            .insert(db)
            .await
            .unwrap_or_else(|e| panic!("seed order {id}: {e}"));
        }
    }

    fn order_service() -> ferro::ServiceDef {
        crate::projections::order::service_def()
    }

    /// Resolve the `ActionDef` for `action_name` from the production registry —
    /// the SAME resolution the visual handler performs.
    fn action_def(action_name: &str) -> ferro::ActionDef {
        order_service()
            .actions
            .iter()
            .find(|a| a.name == action_name)
            .cloned()
            .unwrap_or_else(|| panic!("action '{action_name}' not in order service"))
    }

    /// The transition guard derived from the StateMachine (handler step 4).
    fn transition_guard_for(action_name: &str) -> Option<String> {
        let svc = order_service();
        ferro::derive_transition_plan(&svc, action_name)
            .ok()
            .and_then(|p| p.guard)
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

    /// Build the reused dispatcher (mirrors `make_write_dispatcher` against the
    /// explicit in-memory `db`). The handler reuses the production dispatcher;
    /// the closures are identical in shape (find_for_tenant + derive to_state /
    /// live is_manager check).
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

                    // Derive to_state from the StateMachine — no match, not from inputs.
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

    /// Drive the exact kernel call the visual handler makes (channel `"web"`).
    async fn visual_dispatch(
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
        )
        .await
    }

    // ── SC2: visual path drives the derived transition through the kernel ─────

    #[tokio::test]
    async fn visual_action_drives_derived_transition() {
        let db = setup_db().await;
        seed(&db).await;

        // Order 1 (tenant 1) is "draft"; submit → derived to_state "submitted".
        let result = visual_dispatch("submit", json!({"id": 1}), 1, &db).await;
        assert!(
            result.is_ok(),
            "visual submit must succeed; got: {result:?}"
        );

        let order = load_order(1, &db).await;
        assert_eq!(
            order.status, "submitted",
            "visual path must persist the derived to_state 'submitted'; got: {}",
            order.status
        );
    }

    // ── SC2: live guard re-eval rejects on the visual path ───────────────────

    #[tokio::test]
    async fn visual_guard_rejects_illegal_transition() {
        let db = setup_db().await;
        seed(&db).await;

        // Order 5 (tenant 3) is "submitted". `approve` carries guard `is_manager`,
        // which is false for tenant 3 (no users). The shared kernel re-evaluates
        // the guard LIVE → GuardFailed; the record must NOT transition.
        let result = visual_dispatch("approve", json!({"id": 5}), 3, &db).await;
        assert!(
            matches!(result, Err(WriteError::GuardFailed(_))),
            "guard-false transition must be rejected via live re-eval; got: {result:?}"
        );

        let order = load_order(5, &db).await;
        assert_eq!(
            order.status, "submitted",
            "guard-rejected transition must leave state unchanged; got: {}",
            order.status
        );
    }

    // ── SC2: audit channel is `web`, not `mcp` ───────────────────────────────

    #[tokio::test]
    async fn visual_audit_channel_is_web() {
        let db = setup_db().await;
        seed(&db).await;

        let result = visual_dispatch("submit", json!({"id": 2}), 1, &db).await;
        assert!(
            result.is_ok(),
            "visual submit must succeed; got: {result:?}"
        );

        let entries = history_for_target(&AuditTarget::new("submit", "2"), &db)
            .await
            .expect("history_for_target must not fail");
        assert!(
            !entries.is_empty(),
            "audit entry must be written after the visual write"
        );
        assert_eq!(
            entries[0].action, "web.action.submit",
            "visual write must audit as 'web.action.submit', NOT 'mcp.action.*'; got: {}",
            entries[0].action
        );
    }

    // ── SC2: cross-tenant write denied (tenant from auth) ────────────────────

    #[tokio::test]
    async fn visual_cross_tenant_denied() {
        let db = setup_db().await;
        seed(&db).await;

        // Order 3 belongs to tenant 2; the authenticated tenant is 1.
        // find_for_tenant → None → denial. Order 3 must be unmutated.
        let result = visual_dispatch("approve", json!({"id": 3}), 1, &db).await;
        assert!(
            result.is_err(),
            "cross-tenant visual write must be denied; got: {result:?}"
        );

        let order = load_order(3, &db).await;
        assert_eq!(
            order.status, "submitted",
            "another tenant's order must not be mutated; got: {}",
            order.status
        );
    }

    // ── SC2: to_state from the StateMachine only (form value ignored) ─────────

    #[tokio::test]
    async fn visual_action_rejects_form_supplied_to_state() {
        let db = setup_db().await;
        seed(&db).await;

        // The body carries a bogus `status`/`to_state`. The persisted state must
        // still be the DERIVED Transition.to ("submitted"), not the form value.
        let body = json!({ "id": 1, "status": "delivered", "to_state": "approved" });
        let result = visual_dispatch("submit", body, 1, &db).await;
        assert!(
            result.is_ok(),
            "visual submit must succeed; got: {result:?}"
        );

        let order = load_order(1, &db).await;
        assert_eq!(
            order.status, "submitted",
            "form-supplied to_state must be IGNORED; persisted must be derived 'submitted'; got: {}",
            order.status
        );
    }
}
