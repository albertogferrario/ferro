//! MCP Streamable HTTP endpoint (Phase 198).
//! Thin adapter over ferro-mcp-server pure JSON-RPC dispatch.
//! TODO(phase-199): validate Origin header (DNS-rebinding prevention per MCP spec).

use ferro::serde_json::{json, Value};
use ferro::ServiceDef;
use ferro::{handler, HttpResponse, Request, Response};
use ferro_mcp_server::{
    extract_bearer, handle_initialize, handle_tools_call, handle_tools_list, BearerOutcome,
    McpServerConfig,
};

/// The MCP-exposed projections served at this endpoint.
/// Phase 198: explicit slice; a registry can replace this later.
// Phase 199 authenticated path calls this; unreachable in Phase 198 since seam always challenges.
#[allow(dead_code)]
fn exposed_services() -> Vec<ServiceDef> {
    vec![crate::projections::order::service_def()]
}

/// Build the RFC 9728 / RFC 6750 unauthenticated challenge response.
// Called from the handle handler body; #[allow] required because the #[handler] macro
// wraps the function body and the dead-code lint does not see the call through the expansion.
#[allow(dead_code)]
fn challenge_response(config: &McpServerConfig) -> HttpResponse {
    let challenge = format!(
        "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\"",
        config.app_url
    );
    HttpResponse::new()
        .status(401)
        .header("WWW-Authenticate", challenge)
}

/// POST /mcp — MCP Streamable HTTP endpoint.
///
/// Reads the Authorization header before consuming the request body (single-read guarantee).
/// Phase 198: always returns 401 challenge (bearer seam returns Unauthenticated for all requests).
#[handler]
pub async fn handle(req: Request) -> Response {
    let config = McpServerConfig::from_env();

    // 1. Read headers BEFORE consuming the body (Request::json consumes self).
    let authorization = req.header("Authorization").map(|s| s.to_owned());

    // 2. Bearer seam — Phase 198 always Unauthenticated → 401 challenge.
    match extract_bearer(authorization.as_deref()) {
        BearerOutcome::Unauthenticated => return Err(challenge_response(&config)),
        BearerOutcome::Authenticated(_principal) => { /* Phase 199+ */ }
    }

    // 3. Authenticated path (unreachable in Phase 198, but wired for Phase 199).
    let body: Value = req.json().await.map_err(|e| {
        HttpResponse::json(json!({
            "jsonrpc": "2.0", "id": null,
            "error": { "code": -32700, "message": e.to_string() }
        }))
    })?;
    let id = body.get("id").cloned().unwrap_or(json!(null));
    let method = body["method"].as_str().unwrap_or("");
    let params = body.get("params").cloned().unwrap_or_else(|| json!({}));

    let mut payload = match method {
        "initialize" => handle_initialize(params, &config).await,
        "tools/list" => handle_tools_list(&exposed_services(), &config).await,
        "tools/call" => {
            let db = ferro::DB::connection().map_err(|e| {
                HttpResponse::json(json!({
                    "jsonrpc": "2.0", "id": id.clone(),
                    "error": { "code": -32603, "message": e.to_string() }
                }))
            })?;
            handle_tools_call(params, &exposed_services(), db.inner()).await
        }
        _ => json!({ "error": { "code": -32601, "message": "Method not found" } }),
    };
    // Splice the JSON-RPC envelope onto the handler payload.
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("jsonrpc".into(), json!("2.0"));
        obj.insert("id".into(), id);
    }
    Ok(HttpResponse::json(payload))
}

/// MCP spec: GET /mcp must return 405 when the server does not offer an SSE stream.
/// Ferro's router returns 404 on method mismatch, so this explicit handler is required.
#[handler]
pub async fn method_not_allowed(_req: Request) -> Response {
    Err(HttpResponse::new().status(405).header("Allow", "POST"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_response_has_correct_header() {
        let config = McpServerConfig {
            app_name: "x".into(),
            app_url: "http://localhost".into(),
            version: "0".into(),
        };
        let resp = challenge_response(&config);
        assert_eq!(resp.status_code(), 401);
        let hv = resp
            .headers()
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("WWW-Authenticate"));
        assert_eq!(
            hv.map(|(_, v)| v.as_str()),
            Some("Bearer resource_metadata=\"http://localhost/.well-known/oauth-protected-resource\"")
        );
    }

    #[test]
    fn bearer_seam_always_challenges() {
        // Any Authorization header value returns Unauthenticated in Phase 198.
        assert!(matches!(
            extract_bearer(Some("Bearer some-token")),
            BearerOutcome::Unauthenticated
        ));
        assert!(matches!(
            extract_bearer(None),
            BearerOutcome::Unauthenticated
        ));
    }
}
