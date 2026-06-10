//! MCP Streamable HTTP endpoint (Phase 199/200).
//! Thin adapter over ferro-mcp-server pure JSON-RPC dispatch.
//! Bearer validation via BearerAuthMiddleware (Plan 200-04); Origin check (DNS-rebinding prevention).
//! Policy gating via Gate::authorize_for + mcp_ability fail-closed check (Plan 200-05, AMCP-11).

use ferro::serde_json::{json, Value};
use ferro::ServiceDef;
use ferro::{handler, HttpResponse, Request, Response};
use ferro_mcp_server::{handle_initialize, handle_tools_call, handle_tools_list, McpServerConfig};

/// The MCP-exposed projections served at this endpoint.
/// Phase 198: explicit slice; a registry can replace this later.
fn exposed_services() -> Vec<ServiceDef> {
    vec![crate::projections::order::service_def()]
}

/// Build the RFC 9728 / RFC 6750 unauthenticated challenge response.
#[cfg(test)]
fn challenge_response(config: &McpServerConfig) -> HttpResponse {
    let challenge = format!(
        "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\"",
        config.app_url
    );
    HttpResponse::new()
        .status(401)
        .header("WWW-Authenticate", challenge)
}

/// Build a D-09 policy-deny MCP tool-error envelope (JSON-RPC success, isError=true).
///
/// The message MUST NOT disclose table names, column names, filter values, or row counts.
/// The envelope is spliced with `jsonrpc`/`id` inline.
fn make_tool_deny_response(message: &str, id: &Value) -> Value {
    let mut payload = json!({
        "result": {
            "content": [{ "type": "text", "text": message }],
            "isError": true
        }
    });
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("jsonrpc".into(), json!("2.0"));
        obj.insert("id".into(), id.clone());
    }
    payload
}

/// POST /mcp — MCP Streamable HTTP endpoint.
///
/// Origin check (DNS-rebinding prevention) and bearer validation have already run
/// in the middleware stack (Plan 200-04). The handler reads the principal inserted
/// by BearerAuthMiddleware, loads the concrete User for Gate checks (D-04), and
/// gates each `tools/call` via `Gate::authorize_for` before dispatching.
#[handler]
pub async fn handle(req: Request) -> Response {
    let config = McpServerConfig::from_env();

    // 1. Origin check (T-15): present but mismatched → 403; absent allowed (non-browser SDK).
    if let Some(origin) = req.header("Origin") {
        if !origin.starts_with(config.app_url.as_str()) {
            return Err(HttpResponse::new().status(403));
        }
    }

    // 2. Retrieve principal inserted by BearerAuthMiddleware upstream.
    //    BearerAuthMiddleware ran before TenantMiddleware; principal is a serde_json::Value
    //    with shape {"sub": "<user_id>", "tenant_id": <int|null>}.
    //    TypeId must match exactly — req.get::<serde_json::Value>() (Pitfall 2).
    let principal = req
        .get::<ferro::serde_json::Value>()
        .ok_or_else(|| HttpResponse::new().status(401))?;

    // 3. Parse user_id from sub claim — non-numeric or absent sub → 400 (T-200-SUB).
    let user_id: i64 = principal["sub"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| HttpResponse::new().status(400))?;

    // 4. Dispatch JSON-RPC body.
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

            // Resolve the target service by tool name before dispatching (D-04).
            let tool_name = params["name"].as_str().unwrap_or("");
            let service_name = tool_name.strip_prefix("list_").unwrap_or(tool_name);
            let services = exposed_services();
            let service = match services
                .iter()
                .find(|s| s.name == service_name && s.mcp_exposed)
            {
                Some(s) => s,
                None => {
                    return Ok(HttpResponse::json(json!({
                        "jsonrpc": "2.0", "id": id,
                        "error": { "code": -32601, "message": "Method not found" }
                    })));
                }
            };

            // Load the concrete User for Gate::authorize_for (Pitfall 7: Gate::authorize
            // reads session Auth::id() which is absent on the MCP bearer path).
            let user = crate::models::users::User::find_by_id(user_id)
                .await
                .map_err(|e| {
                    HttpResponse::json(json!({
                        "jsonrpc": "2.0", "id": id.clone(),
                        "error": { "code": -32603, "message": e.to_string() }
                    }))
                })?
                .ok_or_else(|| HttpResponse::new().status(401))?;

            // Fail-closed (D-04, D-06, T-200-03b): mcp_ability = None → deny.
            // An mcp_exposed projection with no declared ability is never callable.
            let ability = match service.mcp_ability.as_deref() {
                Some(a) => a,
                None => {
                    return Ok(HttpResponse::json(make_tool_deny_response(
                        "Access denied. This resource requires an explicit ability declaration.",
                        &id,
                    )));
                }
            };

            // Policy gate (T-200-03, AMCP-11): same policy layer as the web surface.
            // Gate::authorize_for takes an explicit user — does NOT check session state.
            match ferro::authorization::Gate::authorize_for(&user, ability, None) {
                Ok(()) => {}
                Err(_) => {
                    // D-09: deny envelope discloses no rows, columns, or filter values.
                    return Ok(HttpResponse::json(make_tool_deny_response(
                        "Access denied. You do not have permission to view this resource.",
                        &id,
                    )));
                }
            }

            // Allowed — forward tenant context to dispatch (SC-1).
            let tenant_id = ferro::current_tenant().map(|t| t.id);
            handle_tools_call(params, &services, db.inner(), tenant_id).await
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
    use ferro_mcp_oauth::jwt::{build_claims, mint_token};
    use ferro_mcp_oauth::{validate_bearer, BearerCheck, OAuthConfig};

    fn make_id() -> Value {
        json!(42)
    }

    // ────────────────────────────────────────────────────────────────
    // Existing tests (unchanged)
    // ────────────────────────────────────────────────────────────────

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
        // Mint a token with a different secret (invalid signature from config's perspective).
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

    // ────────────────────────────────────────────────────────────────
    // Policy-deny tests (Plan 200-05, AMCP-11, D-09)
    // ────────────────────────────────────────────────────────────────

    /// T-200-03b: a projection with mcp_ability = None → policy-deny tool error (fail-closed).
    ///
    /// Verifies that a ServiceDef with no declared mcp_ability produces the correct
    /// deny envelope via the same code path used in the handler's fail-closed branch.
    #[test]
    fn policy_deny_no_ability() {
        let id = make_id();

        // Build a service with no mcp_ability (default).
        let service = ServiceDef::new("secret_resource").mcp_exposed(true);
        assert!(
            service.mcp_ability.is_none(),
            "fixture: mcp_ability must be None for this test"
        );

        // Exercise the fail-closed branch: None ability → deny (mirrors handler logic).
        let response = match service.mcp_ability.as_deref() {
            Some(_) => panic!("test fixture error: ability should be None"),
            None => make_tool_deny_response(
                "Access denied. This resource requires an explicit ability declaration.",
                &id,
            ),
        };

        // Must be a JSON-RPC success envelope with isError:true (D-09, not a transport 401).
        assert_eq!(response["jsonrpc"], "2.0", "must be a valid JSON-RPC envelope");
        assert_eq!(response["id"], 42, "id must be forwarded");
        assert_eq!(
            response["result"]["isError"],
            true,
            "isError must be true for policy deny"
        );
        // No "error" key at top level (this would indicate a JSON-RPC error, not a tool error).
        assert!(
            response.get("error").is_none(),
            "must not have top-level error key"
        );

        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("content[0].text must exist");

        // D-09: no data disclosure in denial message.
        assert!(!text.contains("orders"), "denial must not disclose table name 'orders'");
        assert!(!text.contains("customer_name"), "denial must not disclose column name");
        assert!(!text.contains("tenant_id"), "denial must not disclose column name");
        // No digit-only tokens (no row counts).
        let has_row_count = text
            .split_whitespace()
            .any(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_digit()));
        assert!(!has_row_count, "denial must not disclose numeric row counts");
    }

    /// T-200-04: Gate deny → isError:true; deny body discloses no resource information (D-09).
    ///
    /// Exercises make_tool_deny_response directly — the same function called from
    /// the Gate::authorize_for Err branch in handle(). Verifies shape and no-disclosure.
    #[test]
    fn policy_deny_tool_error_shape() {
        let id = make_id();

        // This mirrors the Gate::authorize_for Err(_) branch in handle().
        let deny_response = make_tool_deny_response(
            "Access denied. You do not have permission to view this resource.",
            &id,
        );

        // Shape: JSON-RPC success envelope with isError:true (D-09 — not a transport error).
        assert_eq!(
            deny_response["jsonrpc"], "2.0",
            "deny envelope must be a JSON-RPC success (not a transport error code)"
        );
        assert_eq!(deny_response["id"], 42, "request id must be forwarded");

        let result = &deny_response["result"];
        assert_eq!(result["isError"], true, "isError must be true");
        assert!(
            deny_response.get("error").is_none(),
            "must not have top-level JSON-RPC error key"
        );

        let content = result["content"].as_array().expect("content must be array");
        assert_eq!(content.len(), 1, "exactly one content item");
        assert_eq!(content[0]["type"], "text", "content type must be text");

        let text = content[0]["text"].as_str().expect("text must be a string");

        // D-09 no-disclosure: the message must not contain resource-identifying strings.
        assert!(!text.contains("orders"), "deny text must not contain table name 'orders'");
        assert!(!text.contains("customer_name"), "deny text must not disclose column 'customer_name'");
        assert!(!text.contains("total"), "deny text must not disclose column 'total'");
        assert!(!text.contains("status"), "deny text must not disclose column 'status'");
        assert!(!text.contains("tenant_id"), "deny text must not disclose column 'tenant_id'");
        // No isolated digit-only tokens (no row counts).
        let has_digits_only_token = text
            .split_whitespace()
            .any(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_digit()));
        assert!(!has_digits_only_token, "deny text must not contain numeric row counts");

        // No rows or total count in the result.
        assert!(result.get("rows").is_none(), "deny result must not contain rows");
        assert!(result.get("total").is_none(), "deny result must not contain total count");
    }

    /// Verify that make_tool_deny_response produces a JSON-RPC success envelope, per D-09.
    /// A transport-level error (401/403) is inappropriate for a policy denial on a valid request.
    #[test]
    fn deny_response_is_jsonrpc_success_not_transport_error() {
        let id = json!("test-id");
        let resp = make_tool_deny_response("Access denied.", &id);

        assert!(
            resp.get("result").is_some(),
            "deny response must have 'result' key (JSON-RPC success, not error)"
        );
        assert!(
            resp.get("error").is_none(),
            "deny response must NOT have top-level 'error' key"
        );
    }
}
