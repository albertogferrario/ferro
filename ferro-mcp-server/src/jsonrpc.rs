//! Pure JSON-RPC method dispatch for the MCP endpoint.
//!
//! Each function returns a full `{ "result": {...} }` or `{ "error": {...} }`
//! payload. The HTTP adapter (Plan 198-02) splices the `jsonrpc`/`id` fields
//! from the request onto the returned object before writing the response.

use crate::config::McpServerConfig;
use crate::{dispatch, render_exposed_tools, McpContext};
use ferro_projections::ServiceDef;
use serde_json::{json, Value};

/// Handle an MCP `initialize` request.
///
/// Returns `protocolVersion: "2025-03-26"` (rmcp 0.12 `ProtocolVersion::LATEST`),
/// `capabilities.tools` as an object, and `serverInfo` from `McpServerConfig`.
pub async fn handle_initialize(_params: Value, config: &McpServerConfig) -> Value {
    json!({
        "result": {
            "protocolVersion": "2025-03-26",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": config.app_name,
                "version": config.version
            }
        }
    })
}

/// Handle an MCP `tools/list` request.
///
/// Returns only the `mcp_exposed` projections, each named `list_<service.name>`.
pub async fn handle_tools_list(services: &[ServiceDef], _config: &McpServerConfig) -> Value {
    match render_exposed_tools(services, &McpContext) {
        Ok(tools) => json!({ "result": { "tools": tools } }),
        Err(e) => json!({ "error": { "code": -32603, "message": e.to_string() } }),
    }
}

/// Handle an MCP `tools/call` request.
///
/// `call_params` is the full MCP params object: `{ "name": "list_<svc>",
/// "arguments": { "limit": u64, "offset": u64, <filter_key>: <value>, ... } }`.
///
/// Strips the `"list_"` prefix from `name` to find the `ServiceDef`, then
/// delegates to `dispatch`. Pagination keys are removed from `arguments` before
/// passing the remainder as filters — the filter-key allowlist and limit clamp
/// live in `dispatch` (Phase 197 WR-01/WR-02) and are not re-implemented here.
pub async fn handle_tools_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &sea_orm::DatabaseConnection,
) -> Value {
    let tool_name = call_params["name"].as_str().unwrap_or("");
    let service_name = tool_name.strip_prefix("list_").unwrap_or(tool_name);

    let service = match services
        .iter()
        .find(|s| s.name == service_name && s.mcp_exposed)
    {
        Some(s) => s,
        None => {
            return json!({ "error": { "code": -32601, "message": "Method not found" } });
        }
    };

    let args = call_params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(25);
    let offset = args.get("offset").and_then(Value::as_u64).unwrap_or(0);

    // Filters = arguments minus pagination keys. dispatch enforces the
    // filter-key allowlist and limit clamp (Phase 197 WR-01/WR-02).
    let mut filters = args.clone();
    if let Some(obj) = filters.as_object_mut() {
        obj.remove("limit");
        obj.remove("offset");
    }

    match dispatch(service, filters, limit, offset, db).await {
        Ok(result) => json!({
            "result": {
                "content": result.rows,
                "total": result.total,
                "limit": result.limit,
                "offset": result.offset
            }
        }),
        Err(e) => json!({ "error": { "code": -32602, "message": e.to_string() } }),
    }
}
