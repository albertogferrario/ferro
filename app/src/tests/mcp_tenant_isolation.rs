//! Integration tests: per-tenant MCP isolation (SC-1) and middleware-chain parity (SC-3).
//!
//! SC-1: A JWT principal scoped to tenant A returns only A's orders; tenant B returns only B's.
//! SC-3: The tenant context on /mcp is set by JwtClaimResolver inside TenantMiddleware — the
//!       same path as the web surface — not by a separate hand-set task-local.
//!
//! Both tests exercise a real in-memory SQLite DB seeded with two tenants and two orders each.

#[cfg(test)]
mod tests {
    use crate::migrations::Migrator;
    use ferro::serde_json::json;
    use ferro::{
        DbTenantLookup, JwtClaimResolver, Middleware, Next, TenantContext, TenantFailureMode,
        TenantMiddleware, TenantResolver,
    };
    #[cfg(feature = "confirmation")]
    use ferro_mcp_server::McpServerConfig;
    use ferro_mcp_server::{handle_tools_call, McpContext, WriteDispatcher};

    fn noop_dispatcher() -> WriteDispatcher {
        WriteDispatcher {
            executor: Box::new(|_, _, _, _| Box::pin(async { Ok(ferro::serde_json::json!({})) })),
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
        }
    }
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database, DatabaseConnection};
    use sea_orm_migration::prelude::*;
    use std::sync::Arc;

    // ── Fixture helpers ──────────────────────────────────────────────────────

    /// Open an in-memory SQLite DB and run the full Migrator.
    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite connect failed");
        Migrator::up(&db, None)
            .await
            .expect("migrations failed on test DB");
        db
    }

    /// Seed two tenants, one user each, and two orders each.
    ///
    /// Tenant ids: 1 (acme), 2 (globex).
    /// User ids: 901 (alice, tenant 1), 902 (bob, tenant 2).
    /// Order ids: 1-2 (tenant 1), 3-4 (tenant 2).
    async fn seed_two_tenants(db: &DatabaseConnection) {
        use crate::models::entities::orders::ActiveModel as OrderActive;
        use crate::models::entities::tenants::ActiveModel as TenantActive;
        use crate::models::entities::users::ActiveModel as UserActive;

        let now = "2026-06-10T00:00:00+00:00";

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

        // Users (password irrelevant for these tests)
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

        // Orders (2 per tenant)
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

    /// Build a `DbTenantLookup` that resolves against the given DatabaseConnection.
    ///
    /// This is structurally identical to `app::tenant_lookup::build()` but scoped
    /// to a test-local DB so tests are fully isolated.
    fn build_test_lookup(db: DatabaseConnection) -> Arc<dyn ferro::TenantLookup> {
        use crate::models::entities::tenants::Entity as TenantEntity;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter as _};

        // Clone db into each closure — DatabaseConnection is Clone (Arc-backed).
        let db_slug = db.clone();
        let db_id = db.clone();

        Arc::new(DbTenantLookup::new(
            move |slug| {
                let db = db_slug.clone();
                Box::pin(async move {
                    use crate::models::entities::tenants::Column;
                    TenantEntity::find()
                        .filter(Column::Slug.eq(slug))
                        .one(&db)
                        .await
                        .ok()
                        .flatten()
                        .map(|t| TenantContext::new(t.id, t.slug, t.name, None))
                })
            },
            move |id| {
                let db = db_id.clone();
                Box::pin(async move {
                    use crate::models::entities::tenants::Column;
                    TenantEntity::find()
                        .filter(Column::Id.eq(id))
                        .one(&db)
                        .await
                        .ok()
                        .flatten()
                        .map(|t| TenantContext::new(t.id, t.slug, t.name, None))
                })
            },
        ))
    }

    /// Build a ferro::Request via TCP loopback with the given principal inserted
    /// into request extensions (mirrors BearerAuthMiddleware's insertion).
    async fn make_request_with_principal(principal: ferro::serde_json::Value) -> ferro::Request {
        use bytes::Bytes;
        use http_body_util::Empty;
        use hyper_util::rt::TokioIo;
        use std::sync::{Arc, Mutex};
        use tokio::sync::oneshot;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = oneshot::channel::<ferro::Request>();
        let tx_holder = Arc::new(Mutex::new(Some(tx)));
        let principal_clone = principal.clone();

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let io = TokioIo::new(stream);
                let tx_holder = tx_holder.clone();
                let principal_inner = principal_clone.clone();
                hyper::server::conn::http1::Builder::new()
                    .serve_connection(
                        io,
                        hyper::service::service_fn(move |req| {
                            let tx_holder = tx_holder.clone();
                            let p = principal_inner.clone();
                            async move {
                                if let Some(tx) = tx_holder.lock().unwrap().take() {
                                    let mut ferro_req = ferro::Request::new(req);
                                    // Insert principal exactly as BearerAuthMiddleware does.
                                    ferro_req.insert::<ferro::serde_json::Value>(p);
                                    let _ = tx.send(ferro_req);
                                }
                                Ok::<_, hyper::Error>(hyper::Response::new(Empty::<Bytes>::new()))
                            }
                        }),
                    )
                    .await
                    .ok();
            }
        });

        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
        tokio::spawn(async move { conn.await.ok() });

        let req = hyper::Request::builder()
            .uri("/mcp")
            .body(Empty::<Bytes>::new())
            .unwrap();
        let _ = sender.send_request(req).await;
        rx.await.unwrap()
    }

    /// The `ServiceDef` for orders used in dispatch calls.
    fn order_service() -> ferro::ServiceDef {
        crate::projections::order::service_def()
    }

    // ── SC-1 Isolation tests ─────────────────────────────────────────────────

    /// SC-1 — Tenant A isolation: principal with tenant_id=1 returns only tenant 1 orders.
    ///
    /// Drives JwtClaimResolver::resolve (the same call TenantMiddleware makes) to get the
    /// TenantContext, then passes tenant_id to handle_tools_call, which enforces the
    /// tenant predicate at the SQL level. All returned rows must have tenant_id == 1;
    /// none may have tenant_id == 2.
    #[tokio::test]
    async fn tenant_a_isolation() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;

        // Build the resolver exactly as the /mcp route does.
        let lookup = build_test_lookup(db.clone());
        let resolver = JwtClaimResolver::new("tenant_id", lookup);

        // Insert principal as BearerAuthMiddleware would.
        let alice_principal = json!({"sub": "901", "tenant_id": 1});
        let req = make_request_with_principal(alice_principal.clone()).await;

        // Resolve via JwtClaimResolver — this is the same call TenantMiddleware makes.
        let tenant_ctx = resolver
            .resolve(&req)
            .await
            .expect("JwtClaimResolver must resolve tenant 1");
        assert_eq!(
            tenant_ctx.id, 1,
            "resolver must return tenant id=1 for Alice"
        );

        // Call dispatch with the resolved tenant_id (mirrors handler's current_tenant().map(|t| t.id)).
        let call_params = json!({"name": "list_order", "arguments": {"limit": 10}});
        let services = vec![order_service()];
        let result = handle_tools_call(
            call_params,
            &services,
            &db,
            Some(tenant_ctx.id),
            &McpContext::default(),
            &noop_dispatcher(),
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &McpServerConfig::default(),
        )
        .await;

        // Post-fix envelope: content is a valid text block, rows live in structuredContent.
        let content = result["result"]["content"]
            .as_array()
            .expect("result.content must be an array");
        assert_eq!(
            content[0]["type"].as_str(),
            Some("text"),
            "content[0] must be a text block (type=text) — locks the post-fix shape"
        );

        let rows = result["result"]["structuredContent"]["rows"]
            .as_array()
            .expect("structuredContent.rows must be an array");
        assert!(
            !rows.is_empty(),
            "tenant 1 must have at least one order in the result"
        );

        // Every row must belong to tenant 1.
        for row in rows {
            let tid = row["tenant_id"]
                .as_i64()
                .expect("each row must have a tenant_id field");
            assert_eq!(
                tid, 1,
                "tenant A isolation: row tenant_id must be 1, got {tid}"
            );
        }

        // No row may belong to tenant 2.
        let tenant_2_leak = rows.iter().any(|r| r["tenant_id"].as_i64() == Some(2));
        assert!(
            !tenant_2_leak,
            "tenant A isolation: no row must have tenant_id == 2 (cross-tenant leak)"
        );
    }

    /// SC-1 — Tenant B isolation: principal with tenant_id=2 returns only tenant 2 orders.
    ///
    /// Mirror of tenant_a_isolation. Both directions must pass for SC-1 to be satisfied.
    #[tokio::test]
    async fn tenant_b_isolation() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;

        let lookup = build_test_lookup(db.clone());
        let resolver = JwtClaimResolver::new("tenant_id", lookup);

        let bob_principal = json!({"sub": "902", "tenant_id": 2});
        let req = make_request_with_principal(bob_principal.clone()).await;

        let tenant_ctx = resolver
            .resolve(&req)
            .await
            .expect("JwtClaimResolver must resolve tenant 2");
        assert_eq!(tenant_ctx.id, 2, "resolver must return tenant id=2 for Bob");

        let call_params = json!({"name": "list_order", "arguments": {"limit": 10}});
        let services = vec![order_service()];
        let result = handle_tools_call(
            call_params,
            &services,
            &db,
            Some(tenant_ctx.id),
            &McpContext::default(),
            &noop_dispatcher(),
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &McpServerConfig::default(),
        )
        .await;

        // Post-fix envelope: content is a valid text block, rows live in structuredContent.
        let content = result["result"]["content"]
            .as_array()
            .expect("result.content must be an array");
        assert_eq!(
            content[0]["type"].as_str(),
            Some("text"),
            "content[0] must be a text block (type=text) — locks the post-fix shape"
        );

        let rows = result["result"]["structuredContent"]["rows"]
            .as_array()
            .expect("structuredContent.rows must be an array");
        assert!(
            !rows.is_empty(),
            "tenant 2 must have at least one order in the result"
        );

        // Every row must belong to tenant 2.
        for row in rows {
            let tid = row["tenant_id"]
                .as_i64()
                .expect("each row must have a tenant_id field");
            assert_eq!(
                tid, 2,
                "tenant B isolation: row tenant_id must be 2, got {tid}"
            );
        }

        // No row may belong to tenant 1.
        let tenant_1_leak = rows.iter().any(|r| r["tenant_id"].as_i64() == Some(1));
        assert!(
            !tenant_1_leak,
            "tenant B isolation: no row must have tenant_id == 1 (cross-tenant leak)"
        );
    }

    // ── SC-3 Parity test ─────────────────────────────────────────────────────

    /// SC-3 — Middleware-chain parity: current_tenant() on the /mcp path is set by
    /// JwtClaimResolver inside TenantMiddleware, not by a hand-set task-local.
    ///
    /// Drives TenantMiddleware::handle() with a JwtClaimResolver and a Next closure
    /// that captures current_tenant(). Asserts:
    ///   1. current_tenant() inside the Next closure is Some.
    ///   2. The id matches the tenant_id claim in the principal.
    ///   3. The id was set by the resolver, not by any secondary path.
    ///
    /// No hand-set task-local is used here — the test relies exclusively on
    /// TenantMiddleware driving the resolver and storing the result.
    #[tokio::test]
    async fn tenant_context_parity() {
        let db = setup_db().await;
        seed_two_tenants(&db).await;

        // Build the same middleware stack as /mcp (Plan 200-04):
        //   BearerAuthMiddleware → TenantMiddleware(JwtClaimResolver("tenant_id"), Forbidden)
        // BearerAuthMiddleware is skipped here (its only job is inserting the principal,
        // which we do manually via make_request_with_principal, mirroring its TypeId contract).
        let lookup = build_test_lookup(db.clone());
        let middleware = TenantMiddleware::new()
            .resolver(JwtClaimResolver::new("tenant_id", lookup))
            .on_failure(TenantFailureMode::Forbidden);

        // Principal for Alice (tenant 1).
        let alice_principal = json!({"sub": "901", "tenant_id": 1});
        let req = make_request_with_principal(alice_principal).await;

        // The Next closure captures current_tenant() — set exclusively by TenantMiddleware.
        let captured_tenant: Arc<tokio::sync::Mutex<Option<TenantContext>>> =
            Arc::new(tokio::sync::Mutex::new(None));
        let captured_clone = captured_tenant.clone();

        let next: Next = Arc::new(move |_req| {
            let captured = captured_clone.clone();
            Box::pin(async move {
                // current_tenant() is set by TenantMiddleware via with_tenant_scope.
                // If this returns Some, the middleware chain set it (not a hand-set).
                let ctx = ferro::current_tenant();
                *captured.lock().await = ctx;
                Ok(ferro::HttpResponse::text("ok"))
            })
        });

        // Drive the middleware chain.
        let result = middleware.handle(req, next).await;
        assert!(
            result.is_ok(),
            "middleware chain must not fail for a valid principal: {result:?}"
        );

        // Verify the captured tenant matches the JWT claim.
        let captured = captured_tenant.lock().await;
        let ctx = captured
            .as_ref()
            .expect("current_tenant() must be Some inside the /mcp middleware chain (SC-3 parity)");

        assert_eq!(
            ctx.id, 1,
            "SC-3: current_tenant().id must equal the tenant_id claim (1), got {}",
            ctx.id
        );

        // Structural guarantee: the tenant was established by the resolver path (JwtClaimResolver),
        // not by any parallel hand-set mechanism. If JwtClaimResolver returned a different
        // id than the claim, this assertion would catch the drift.
        assert_eq!(
            ctx.slug, "acme",
            "SC-3: resolved tenant slug must match the DB record for tenant_id=1"
        );
    }
}
