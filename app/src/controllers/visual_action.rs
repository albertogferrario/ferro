//! Visual/form transition-write surface (Phase 232, EXEC-05).
//!
//! Receives `POST /{service}/{action}` — the action-button URL the projection
//! renderer EMITS (`ferro-json-ui/src/projection/builder.rs:685`) but for which
//! no handler previously existed. This is the visual half of the single-source
//! write surface: it drives the SAME `framework::write` kernel as the MCP path,
//! differing only in the audit channel (`"web"` instead of `"mcp"`).
//!
//! Security envelope (mirrors the MCP path; see the threat register in the plan):
//!   - `to_state` is derived ONLY from `ferro::derive_transition_plan(...)` —
//!     never read from the form body (T-232-05).
//!   - `tenant_id` comes from `ferro::current_tenant()` (auth), never the body
//!     (T-232-07). The reused executor's `find_for_tenant` enforces cross-tenant
//!     denial.
//!   - Guards are re-evaluated server-side inside the shared `dispatch_write`
//!     (T-232-06) — not advisory.
//!   - Error responses are redacted; no SQL/table/column names leak (T-232-09).
//!
//! This is FRAMING only — it resolves the action and calls the shared kernel.
//! There is no `match action_name`, no second executor, and no second guard loop.

use ferro::serde_json::{json, Value};
use ferro::write::{WriteError, WriteResult};
use ferro::{handler, ActionDef, HttpResponse, Request, Response, ServiceDef};

/// POST /{service}/{action} — drive a declared StateMachine transition through
/// the shared `framework::write` kernel with the `"web"` audit channel.
#[handler]
pub async fn handle(req: Request) -> Response {
    // 1. Path params identify the target service + action.
    let service_name = req
        .param("service")
        .map_err(|_| HttpResponse::new().status(400))?
        .to_string();
    let action_name = req
        .param("action")
        .map_err(|_| HttpResponse::new().status(400))?
        .to_string();

    // 2. Tenant from auth context ONLY — never the form body (T-232-07).
    let tenant_id = ferro::current_tenant()
        .map(|t| t.id)
        .ok_or_else(|| HttpResponse::new().status(403))?;

    // 3. Resolve (ServiceDef, ActionDef) from the single service registry the
    //    MCP path uses. Unknown service/action → 404 (no panic, no write).
    let services = crate::controllers::mcp::exposed_services();
    let (svc, action) = resolve_action(&services, &service_name, &action_name)
        .ok_or_else(|| HttpResponse::new().status(404))?;

    // 4. Derive the transition guard from the StateMachine — IDENTICAL to the
    //    MCP path (`write_dispatch.rs:541`). No `match action_name`; the form
    //    never supplies `to_state`/`status`.
    let plan = ferro::derive_transition_plan(svc, &action.name).ok();
    let transition_guard = plan.as_ref().and_then(|p| p.guard.as_deref());

    // 5. Read the form/JSON body as opaque inputs. The kernel + reused executor
    //    consume only `inputs["id"]`; any `status`/`to_state` in the body is
    //    ignored (T-232-05) because `to_state` is derived in the executor via
    //    `derive_transition_plan`.
    let inputs: Value = req.input::<Value>().await.unwrap_or_else(|_| json!({}));

    // 6. Obtain the tenant-scoped DB connection the same way the MCP path does.
    let db = ferro::DB::connection().map_err(|_| HttpResponse::new().status(500))?;

    // 7. REUSE the single dispatcher (same executor + guard evaluator as MCP).
    //    No second WriteDispatcher with different closures.
    let dispatcher = crate::controllers::mcp::make_write_dispatcher();

    // 8. Call the SHARED kernel with channel "web" → `web.action.{name}` audit.
    let outcome = ferro::write::dispatch_write(
        action,
        &inputs,
        tenant_id,
        db.inner(),
        &dispatcher,
        transition_guard,
        "web",
        #[cfg(feature = "confirmation")]
        false,
    )
    .await;

    // 9. Map the outcome to the redacted HTTP response (status + error_kind).
    outcome_to_response(&action.name, outcome)
}

/// Resolve `(ServiceDef, ActionDef)` for `service_name` / `action_name` from the
/// exposed service registry — the SAME registry the MCP path resolves against.
/// Returns `None` when either the service or the action is unknown; the handler
/// maps that to a 404 (no panic, no write).
pub(crate) fn resolve_action<'a>(
    services: &'a [ServiceDef],
    service_name: &str,
    action_name: &str,
) -> Option<(&'a ServiceDef, &'a ActionDef)> {
    let svc = services.iter().find(|s| s.name == service_name)?;
    let action = svc.actions.iter().find(|a| a.name == action_name)?;
    Some((svc, action))
}

/// Map the shared kernel's [`WriteResult`] to the redacted HTTP response the
/// visual surface returns. Error responses disclose no SQL/table/column names
/// (T-232-09): every variant collapses to a fixed `error_kind` + generic message,
/// and a guard rejection is a 4xx, never a 500.
pub(crate) fn outcome_to_response(action_name: &str, outcome: WriteResult<Value>) -> Response {
    match outcome {
        Ok(result) => Ok(HttpResponse::json(json!({
            "status": "ok",
            "action": action_name,
            "result": result,
        }))),
        Err(WriteError::GuardFailed(_)) => Ok(HttpResponse::json(json!({
            "status": "error",
            "error_kind": "guard_denied",
            "message": "precondition not met",
        }))
        .status(403)),
        Err(WriteError::Validation(_)) => Ok(HttpResponse::json(json!({
            "status": "error",
            "error_kind": "invalid_request",
            "message": "request could not be processed",
        }))
        .status(422)),
        Err(WriteError::ActionNotFound(_)) => Err(HttpResponse::new().status(404)),
        #[cfg(feature = "confirmation")]
        Err(WriteError::ConfirmationRequired(_)) => Ok(HttpResponse::json(json!({
            "status": "error",
            "error_kind": "confirmation_required",
            "message": "this action requires confirmation",
        }))
        .status(409)),
        Err(_) => Ok(HttpResponse::json(json!({
            "status": "error",
            "error_kind": "execution_error",
            "message": "write operation failed",
        }))
        .status(500)),
    }
}
