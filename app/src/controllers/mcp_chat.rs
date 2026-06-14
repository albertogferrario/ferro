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
use ferro_mcp_server::{McpContext, McpServerConfig};

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

    // Parse user_id and scope from principal before req.json() consumes the body.
    let _user_id: i64 = principal["sub"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| HttpResponse::new().status(400))?;
    let key_scope: Option<String> = principal
        .get("scope")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Parse `{ "message": "..." }` from the JSON body.
    let body: Value = req.json().await.map_err(|e| {
        HttpResponse::json(json!({ "error": e.to_string() }))
    })?;
    let nl_message = body["message"].as_str().unwrap_or("").to_string();
    if nl_message.is_empty() {
        return Err(HttpResponse::json(
            json!({ "error": "message field is required and must be a non-empty string" }),
        )
        .status(400));
    }

    // Resolve db and tenant_id from the authenticated principal (never from body).
    let db = ferro::DB::connection().map_err(|e| {
        HttpResponse::json(json!({ "error": e.to_string() }))
    })?;
    let tenant_id = ferro::current_tenant().map(|t| t.id);

    let ctx = McpContext {
        tenant_id,
        scope: key_scope,
        ..Default::default()
    };
    let services = super::mcp::exposed_services();
    let dispatcher = super::mcp::make_write_dispatcher();

    // Delegate to process_nl_turn with the live AnthropicProvider.
    // Compiled only under the `ai-live` feature; endpoints built without `ai-live`
    // will not include this handler.
    #[cfg(feature = "ai-live")]
    let result = {
        let provider: Arc<dyn ferro_ai::ClassificationProvider> =
            Arc::new(ferro_ai::AnthropicProvider::from_env().map_err(|e| {
                HttpResponse::json(json!({ "error": e.to_string() }))
            })?);

        ferro_mcp_server::intent::process_nl_turn(
            &nl_message,
            &services,
            db.inner(),
            tenant_id,
            &ctx,
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

    // When built without `ai-live`, the /mcp/chat route is not registered, so this
    // branch is unreachable at runtime. The compile-time guard below ensures the
    // handler body always returns a value regardless of feature combination.
    #[cfg(not(feature = "ai-live"))]
    let result: Value = json!({
        "result": {
            "content": [{ "type": "text", "text": "NL intent loop requires the ai-live feature" }],
            "isError": true
        }
    });

    Ok(HttpResponse::json(result))
}
