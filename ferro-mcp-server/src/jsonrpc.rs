//! Pure JSON-RPC method dispatch for the MCP endpoint.
//!
//! Each function returns a full `{ "result": {...} }` or `{ "error": {...} }`
//! payload. The HTTP adapter (Plan 198-02) splices the `jsonrpc`/`id` fields
//! from the request onto the returned object before writing the response.

use crate::config::McpServerConfig;
use crate::{dispatch, render_exposed_tools, McpContext};
use ferro_projections::ServiceDef;
use rmcp::model::CallToolResult;
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
/// `ctx` carries the resolved tenant identity and scope for filtering.
pub async fn handle_tools_list(
    services: &[ServiceDef],
    ctx: &McpContext,
    _config: &McpServerConfig,
) -> Value {
    match render_exposed_tools(services, ctx) {
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
    tenant_id: Option<i64>,
    ctx: &McpContext,
) -> Value {
    let tool_name = call_params["name"].as_str().unwrap_or("");
    let service_name = tool_name.strip_prefix("list_").unwrap_or(tool_name);

    // Scope enforcement (D-06 / SC#3): re-check at call time, independent of listing filter.
    // Fires before service lookup so write-tool calls are rejected even for unknown tool names.
    // Absent scope (OAuth JWT path) = full access. "read" key cannot call non-read tools.
    let is_write_tool = !tool_name.starts_with("list_");
    let key_scope = ctx.scope.as_deref().unwrap_or("read_write");
    if is_write_tool && key_scope == "read" {
        return json!({
            "error": {
                "code": -32603,
                "message": crate::Error::Auth(
                    "scope insufficient: read key cannot call write tools".to_string()
                ).to_string()
            }
        });
    }

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

    match dispatch(service, filters, limit, offset, db, tenant_id).await {
        // D-01 + D-02 + D-03: single structured value → valid camelCase MCP envelope.
        // structured() emits one text content block + structuredContent + isError:false.
        // total/limit/offset are nested INSIDE the payload (D-02), never as extra
        // top-level keys on the outer "result" object.
        Ok(result) => {
            let payload = serde_json::json!({
                "rows": result.rows,
                "total": result.total,
                "limit": result.limit,
                "offset": result.offset
            });
            let tool_result = CallToolResult::structured(payload);
            json!({ "result": tool_result })
        }
        // A bad filter key is a client parameter problem (-32602); any other
        // failure (DB/render/serialization) is internal (-32603). Clients use
        // the code to decide whether to fix the request or retry (WR-02).
        Err(crate::Error::InvalidFilter(msg)) => {
            json!({ "error": { "code": -32602, "message": msg } })
        }
        Err(e) => json!({ "error": { "code": -32603, "message": e.to_string() } }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::{DataType, FieldMeaning, ServiceDef};
    use rmcp::model::CallToolResult;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};

    // Copied verbatim from dispatch.rs:234-269 — identical in-memory orders fixture.
    async fn setup_orders_db() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("sqlite connect");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_name TEXT NOT NULL,
                total REAL NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                tenant_id INTEGER NOT NULL
            )"
            .to_string(),
        ))
        .await
        .expect("create table");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "INSERT INTO orders (customer_name, total, status, tenant_id) VALUES
                ('Alice', 100.0, 'pending', 1),
                ('Bob',   200.0, 'shipped', 1),
                ('Carol', 150.0, 'pending', 2),
                ('Dave',  250.0, 'shipped', 2)"
                .to_string(),
        ))
        .await
        .expect("seed rows");
        db
    }

    // Copied verbatim from dispatch.rs:271-282.
    fn order_service_with_tenant() -> ServiceDef {
        ServiceDef::new("order")
            .mcp_exposed(true)
            .tenant_column("tenant_id")
            .mcp_ability("view-orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_name", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::String, FieldMeaning::CreatedAt)
            .field("tenant_id", DataType::Integer, FieldMeaning::ForeignKey)
    }

    /// D-04 interop regression: parse the EMITTED result with the MCP client's own
    /// type (CallToolResult custom Deserialize, rmcp model.rs:1646). The prior unit
    /// tests asserted the server's own broken shape and so missed the original bug.
    #[tokio::test]
    async fn tools_call_result_parses_as_valid_mcp_content() {
        let db = setup_orders_db().await;
        let services = vec![order_service_with_tenant()];
        let call_params = serde_json::json!({
            "name": "list_order",
            "arguments": { "limit": 10 }
        });

        let response = handle_tools_call(call_params, &services, &db, Some(1), &McpContext::default()).await;

        // The load-bearing assertion: the client's own type must deserialize it.
        let parsed: CallToolResult = serde_json::from_value(response["result"].clone())
            .expect("result must parse as CallToolResult (D-04 interop)");

        assert_eq!(parsed.is_error, Some(false));
        assert_eq!(
            parsed.content.len(),
            1,
            "structured() produces exactly one content block (D-03)"
        );

        let content_json = serde_json::to_value(&parsed.content).unwrap();
        assert_eq!(
            content_json[0]["type"].as_str(),
            Some("text"),
            "content[0] must have type=text (was missing before fix)"
        );

        let sc = parsed
            .structured_content
            .expect("structuredContent must be present (D-02)");
        assert!(sc.get("rows").is_some(), "structuredContent.rows present");
        assert!(sc.get("total").is_some(), "structuredContent.total present");
        assert!(sc.get("limit").is_some(), "structuredContent.limit present");
        assert!(
            sc.get("offset").is_some(),
            "structuredContent.offset present"
        );

        let rows = sc["rows"].as_array().expect("rows is an array");
        assert_eq!(rows.len(), 2, "tenant 1 has exactly 2 rows");
    }
}
