//! POST /mcp/chat — thin NL conversational-turn endpoint (Phase 221, AMCP-06).
//!
//! Authenticates via the same bearer middleware as `/mcp`, derives `tenant_id`
//! from the authenticated principal (never from the request body), parses the
//! `{ "message": "..." }` body, instantiates the live `AnthropicProvider` (behind
//! the `ai-live` Cargo feature), and delegates to `process_nl_turn`.
//!
//! This handler adds no dispatch, guard, or confirmation logic of its own — all of
//! that lives inside `ferro_mcp_server::intent::process_nl_turn` (D-01, T-221-10).
//!
//! App identity (app_url for the origin check) comes from `McpServerConfig::from_env()`;
//! no strings are hardcoded here (CLAUDE.md project-agnostic rule, T-221-11).

use ferro::serde_json::{json, Value};
use ferro::{handler, HttpResponse, Request, Response};
use ferro_mcp_server::McpServerConfig;

#[cfg(feature = "ai-live")]
use ferro_mcp_server::McpContext;
#[cfg(feature = "ai-live")]
use std::sync::Arc;

/// POST /mcp/chat — single conversational NL turn.
///
/// Auth: same bearer middleware + tenant middleware as `/mcp`.
/// Body: `{ "message": "<nl string>" }`
/// Response: `CallToolResult`-shaped JSON (content + isError + structuredContent).
#[handler]
pub async fn handle_chat(req: Request) -> Response {
    let config = McpServerConfig::from_env();

    // Origin check (mirrors mcp.rs lines 177-180): present but mismatched → 403.
    if let Some(origin) = req.header("Origin") {
        if !origin.starts_with(config.app_url.as_str()) {
            return Err(HttpResponse::new().status(403));
        }
    }

    // Retrieve principal inserted by BearerAuthMiddleware upstream.
    let principal = req
        .get::<ferro::serde_json::Value>()
        .ok_or_else(|| HttpResponse::new().status(401))?;

    // Extract fields from principal BEFORE req.json() moves `req`.
    // user_id validates the principal is well-formed and (under ai-live) loads the
    // concrete User for the read-tool authorization gate; scope feeds McpContext.
    #[cfg_attr(not(feature = "ai-live"), allow(unused_variables))]
    let user_id: i64 = principal["sub"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| HttpResponse::new().status(400))?;
    // key_scope is only used under ai-live but must be extracted here before req moves.
    #[cfg(feature = "ai-live")]
    let key_scope: Option<String> = principal
        .get("scope")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Parse `{ "message": "..." }` from the JSON body (moves req).
    let body: Value = req
        .json()
        .await
        .map_err(|e| HttpResponse::json(json!({ "error": e.to_string() })))?;
    let nl_message = body["message"].as_str().unwrap_or("").to_string();
    if nl_message.is_empty() {
        return Err(HttpResponse::json(
            json!({ "error": "message field is required and must be a non-empty string" }),
        )
        .status(400));
    }

    // Delegate to process_nl_turn with the live AnthropicProvider.
    // Compiled only under the `ai-live` feature.
    #[cfg(feature = "ai-live")]
    let result = {
        // Resolve db and tenant_id from the authenticated principal (never from body).
        let db = ferro::DB::connection()
            .map_err(|e| HttpResponse::json(json!({ "error": e.to_string() })))?;
        let tenant_id = ferro::current_tenant().map(|t| t.id);

        let ctx = McpContext {
            tenant_id,
            scope: key_scope,
            ..Default::default()
        };
        let services = super::mcp::exposed_services();
        let dispatcher = super::mcp::make_write_dispatcher();

        let provider: Arc<dyn ferro_ai::ClassificationProvider> = Arc::new(
            ferro_ai::AnthropicProvider::from_env()
                .map_err(|e| HttpResponse::json(json!({ "error": e.to_string() })))?,
        );

        // Read-tool authorization gate (WR-01 / AMCP-11): mirror the direct /mcp path's
        // Gate::authorize_for + mcp_ability fail-closed check. Load the concrete User
        // (Pitfall 7: Gate::authorize_for takes an explicit user — the MCP bearer path
        // has no session Auth::id()) and build a fail-closed predicate: a service with
        // no declared mcp_ability (None) is denied; otherwise defer to the policy Gate.
        let user = crate::models::users::User::find_by_id(user_id)
            .await
            .map_err(|e| HttpResponse::json(json!({ "error": e.to_string() })))?
            .ok_or_else(|| HttpResponse::new().status(401))?;
        let authorize_read = move |ability: Option<&str>| -> bool {
            match ability {
                None => false,
                Some(a) => ferro::authorization::Gate::authorize_for(&user, a, None).is_ok(),
            }
        };

        ferro_mcp_server::intent::process_nl_turn(
            &nl_message,
            &services,
            db.inner(),
            tenant_id,
            &ctx,
            &authorize_read,
            provider,
            ferro_ai::ClassifierConfig::default(),
            &dispatcher,
            #[cfg(feature = "confirmation")]
            super::mcp::confirmation_store(),
            #[cfg(feature = "confirmation")]
            &config,
        )
        .await
    };

    // Built without `ai-live`: /mcp/chat is non-functional in this build. Return
    // 501 Not Implemented rather than a 200 `isError` envelope, so a non-ai-live build
    // does not advertise the NL-intent feature surface to authenticated callers (WR-02).
    #[cfg(not(feature = "ai-live"))]
    return Err(HttpResponse::new().status(501));

    #[cfg(feature = "ai-live")]
    Ok(HttpResponse::json(result))
}
