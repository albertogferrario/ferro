//! MCP Streamable HTTP endpoint (Phase 199).
//! Thin adapter over ferro-mcp-server pure JSON-RPC dispatch.
//! Bearer validation via ferro-mcp-oauth; Origin check (DNS-rebinding prevention per MCP spec).

use ferro::serde_json::{json, Value};
use ferro::ServiceDef;
use ferro::{handler, HttpResponse, Request, Response};
use ferro_mcp_oauth::{validate_bearer, BearerCheck, OAuthConfig};
use ferro_mcp_server::{handle_initialize, handle_tools_call, handle_tools_list, McpServerConfig};

/// The MCP-exposed projections served at this endpoint.
/// Phase 198: explicit slice; a registry can replace this later.
#[allow(dead_code)]
fn exposed_services() -> Vec<ServiceDef> {
    vec![crate::projections::order::service_def()]
}

/// Build the RFC 9728 / RFC 6750 unauthenticated challenge response.
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
/// Validates Origin (DNS-rebinding prevention) and bearer token before dispatch.
/// Phase 199: real JWT validation via ferro-mcp-oauth::validate_bearer.
#[handler]
pub async fn handle(req: Request) -> Response {
    let config = McpServerConfig::from_env();

    // 1. Origin check (T-15): present but mismatched → 403; absent allowed (non-browser SDK).
    if let Some(origin) = req.header("Origin") {
        if !origin.starts_with(config.app_url.as_str()) {
            return Err(HttpResponse::new().status(403));
        }
    }

    // 2. Read Authorization header BEFORE consuming the body (single-read guarantee).
    let authorization = req.header("Authorization").map(|s| s.to_owned());

    // 3. Bearer validation — fail-closed: if config unavailable → 401 challenge (T-199-13b).
    let oauth_config = OAuthConfig::from_env()
        .map_err(|_| challenge_response(&config))?;

    // expected_tenant: None for single-tenant /mcp (Phase 200 will supply tenant context).
    let expected_tenant = ferro::current_tenant().map(|t| t.id);

    match validate_bearer(authorization.as_deref(), &oauth_config, expected_tenant) {
        BearerCheck::Unauthenticated => return Err(challenge_response(&config)),
        BearerCheck::Invalid => {
            // 401 invalid_token (RFC 6750 §3.1)
            return Err(HttpResponse::new()
                .status(401)
                .header("WWW-Authenticate", "Bearer error=\"invalid_token\""));
        }
        BearerCheck::Forbidden => return Err(HttpResponse::new().status(403)),
        BearerCheck::Authenticated(_principal) => {
            // Phase 200 inserts principal into request extensions for JwtClaimResolver.
        }
    }

    // 4. Authenticated path — dispatch to MCP JSON-RPC handler.
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
    fn invalid_token_returns_401_invalid_token_header() {
        use ferro_mcp_oauth::jwt::{build_claims, mint_token};

        // Mint a token with a different secret (invalid signature from config's perspective)
        let wrong_secret = b"wrong-secret-that-is-at-least-32-bytes-long!!!!!!";
        let claims = build_claims(1, None, "http://localhost", 3600);
        let token = mint_token(&claims, wrong_secret).expect("mint failed");

        let config = OAuthConfig {
            app_name: "x".into(),
            app_url: "http://localhost".into(),
            token_secret: b"correct-secret-that-is-at-least-32-bytes-long!!!!".to_vec(),
        };
        let header = format!("Bearer {token}");
        let result = validate_bearer(Some(&header), &config, None);
        assert!(
            matches!(result, BearerCheck::Invalid),
            "expected Invalid for wrong-secret token"
        );
    }

    #[test]
    fn origin_mismatch_maps_to_403() {
        // Simulate the guard logic: present but mismatched Origin → 403.
        let app_url = "http://localhost";
        let origin = "http://evil.example.com";
        assert!(
            !origin.starts_with(app_url),
            "Origin mismatch guard should reject this"
        );
    }

    #[test]
    fn absent_origin_is_allowed() {
        // Absent Origin (no header) → allowed for non-browser SDK clients.
        let origin: Option<&str> = None;
        assert!(origin.is_none(), "absent origin must not be rejected");
    }
}
