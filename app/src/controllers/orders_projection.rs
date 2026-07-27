//! Dogfood: projection-native Inertia page for the `order` service (SUBST-03).
//!
//! The entire page — the field schema, the tenant-scoped rows, and the
//! guard-filtered list of permitted actions — is derived from ONE `ServiceDef`
//! (`crate::projections::order::service_def()`) by `Inertia::from_projection`.
//! There are no hand-written columns and no hand-written action list: the same
//! declaration that drives the MCP and visual renderers drives this Inertia page.
//!
//! Security envelope mirrors the visual write surface:
//!   - `tenant_id` comes from `ferro::current_tenant()` (auth), never the request.
//!   - The data read is tenant-scoped inside `framework::projection_read::dispatch`.
//!   - `permitted_actions` is advisory display data; write authorization is still
//!     re-evaluated server-side at `dispatch_write` time.

use std::collections::HashMap;

use ferro::{handler, HttpResponse, Inertia, ProjectionQuery, Request, Response};

/// GET /orders — render the order projection as an Inertia page.
#[handler]
pub async fn index(req: Request) -> Response {
    // Tenant from the authenticated session ONLY — never the request body.
    let tenant_id = ferro::current_tenant()
        .map(|t| t.id)
        .ok_or_else(|| HttpResponse::new().status(403))?;

    let db = ferro::DB::connection().map_err(|_| HttpResponse::new().status(500))?;
    let service = crate::projections::order::service_def();

    // Pre-evaluate the service's declared guards ONCE for this request, reusing
    // the SAME live check the MCP write path uses — no duplicate guard logic.
    let is_manager = crate::controllers::mcp::check_is_manager(tenant_id, db.inner()).await;
    let evaluated_guards: HashMap<String, bool> = [("is_manager".to_string(), is_manager)].into();

    Inertia::from_projection(
        &req,
        "OrderList",
        &service,
        ProjectionQuery::default(),
        db.inner(),
        Some(tenant_id),
        &evaluated_guards,
    )
    .await
}
