//! Pure JSON-RPC method dispatch for the MCP endpoint.
//!
//! Each function returns a full `{ "result": {...} }` or `{ "error": {...} }`
//! payload. The HTTP adapter (Plan 198-02) splices the `jsonrpc`/`id` fields
//! from the request onto the returned object before writing the response.

use crate::config::McpServerConfig;
use crate::write_dispatch::handle_write_call;
use crate::{dispatch, render_exposed_tools, McpContext, WriteDispatcher};
use ferro_projections::ServiceDef;
use rmcp::model::CallToolResult;
use serde_json::{json, Value};

#[cfg(feature = "confirmation")]
use ferro_ai::ConfirmationStore;

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
#[allow(clippy::too_many_arguments)]
pub async fn handle_tools_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &McpContext,
    dispatcher: &WriteDispatcher,
    #[cfg(feature = "confirmation")] store: &dyn ConfirmationStore,
    #[cfg(feature = "confirmation")] config: &McpServerConfig,
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

    // Phase 219/220: route write-tool calls to the write dispatch path.
    // Scope gate above stays in front of this routing.
    if is_write_tool {
        return handle_write_call(
            call_params,
            services,
            db,
            tenant_id,
            ctx,
            dispatcher,
            #[cfg(feature = "confirmation")]
            store,
            #[cfg(feature = "confirmation")]
            config,
        )
        .await;
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
        // Phase 241: CRUD dispatch writes to mcp_idempotency_keys + audit_log;
        // add them so tests that route through dispatch_write do not error.
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS mcp_idempotency_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tenant_id INTEGER NOT NULL,
                idempotency_key TEXT NOT NULL,
                result TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE (tenant_id, idempotency_key)
            )"
            .to_string(),
        ))
        .await
        .expect("create mcp_idempotency_keys");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY NOT NULL,
                tenant_id TEXT,
                actor_kind TEXT NOT NULL,
                actor_id TEXT,
                action TEXT NOT NULL,
                target_kind TEXT,
                target_id TEXT,
                before TEXT,
                after TEXT,
                reason TEXT,
                correlation_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
            .to_string(),
        ))
        .await
        .expect("create audit_log");
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

        // No-op dispatcher: read-path test uses list_ tool, write path is not reached.
        let noop_dispatcher = crate::WriteDispatcher::new(
            Box::new(|_, _, _, _| Box::pin(async { Ok(serde_json::json!({})) })),
            Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
        );
        let response = handle_tools_call(
            call_params,
            &services,
            &db,
            Some(1),
            &McpContext::default(),
            &noop_dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &test_config(),
        )
        .await;

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

    // -------------------------------------------------------------------------
    // Phase 218 RED test: SC#5 strict-deserialization of write-tool definitions
    // (T-218-03).
    //
    // Parallel to the Phase 205 test above, but for write tool *definitions*
    // emitted by tools/list rather than tools/call results. The load-bearing
    // assertion: every tool definition must deserialize via rmcp::model::Tool
    // (catching malformed inputSchema shape, missing "type" key, broken
    // annotation shape — the Phase 205 content-block-bug class applied to
    // write-tool definitions).
    //
    // This test is RED now: tools_json.len() == 1 (only list_order exists),
    // but the assertion expects 3 (1 read + 2 write tools). Turns GREEN in
    // Plan 02 when render_exposed_tools emits write tools.
    // -------------------------------------------------------------------------

    fn order_service_with_actions() -> ServiceDef {
        use ferro_projections::{ActionDef, InputDef};
        ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .action(
                ActionDef::new("submit_order")
                    .description("Submit an order for processing")
                    .input(
                        InputDef::new("notes", DataType::String, FieldMeaning::FreeText)
                            .required(false),
                    )
                    .transition_trigger("submit"),
            )
            .action(ActionDef::new("update_notes").input(InputDef::new(
                "notes",
                DataType::String,
                FieldMeaning::FreeText,
            )))
    }

    fn test_config() -> McpServerConfig {
        McpServerConfig {
            app_name: "TestApp".to_string(),
            app_url: "https://test.example".to_string(),
            version: "0.0.0".to_string(),
            confirmation_ttl_seconds: 300,
        }
    }

    /// SC#5 / T-218-03: every write tool definition emitted by tools/list must
    /// parse as rmcp::model::Tool (strict deserializer), and write tools must
    /// carry the correct annotation values.
    ///
    /// RED now: only 1 tool (list_order) exists; assertion expects 3.
    /// GREEN in Plan 02 after render_exposed_tools emits write tools.
    #[tokio::test]
    async fn write_tools_definitions_parse_as_valid_mcp_tool() {
        use rmcp::model::Tool;

        let service = order_service_with_actions();
        let ctx = McpContext::default();
        let config = test_config();
        let resp = handle_tools_list(&[service], &ctx, &config).await;

        let tools_json = resp["result"]["tools"]
            .as_array()
            .expect("tools must be an array");

        // 1 read tool + 2 write tools = 3 total; with confirmation feature, 2 extra
        // confirmation tools are synthesised for submit_order (the only destructive action)
        #[cfg(not(feature = "confirmation"))]
        assert_eq!(
            tools_json.len(),
            3,
            "expected list_order + submit_order + update_notes; got {}",
            tools_json.len()
        );
        #[cfg(feature = "confirmation")]
        assert_eq!(
            tools_json.len(),
            5,
            "expected list_order + submit_order + update_notes + request_confirm_submit_order + confirm_submit_order; got {}",
            tools_json.len()
        );

        for tool_json in tools_json {
            // Load-bearing: rmcp's strict Tool deserializer must accept each definition.
            let tool: Tool = serde_json::from_value(tool_json.clone())
                .expect("each tool definition must parse as rmcp::model::Tool");
            assert!(!tool.name.is_empty(), "tool name must not be empty");
            // inputSchema must have at minimum a "type" or "properties" key.
            assert!(
                tool.input_schema.contains_key("type")
                    || tool.input_schema.contains_key("properties"),
                "inputSchema must have 'type' or 'properties': {:?}",
                &*tool.input_schema
            );
        }

        // submit_order: transition action → readOnlyHint=false, destructiveHint=true
        let submit = tools_json
            .iter()
            .find(|t| t["name"] == "submit_order")
            .expect("submit_order tool must be present");
        assert_eq!(
            submit["annotations"]["readOnlyHint"], false,
            "submit_order readOnlyHint must be false"
        );
        assert_eq!(
            submit["annotations"]["destructiveHint"], true,
            "submit_order (transition) destructiveHint must be true"
        );

        // update_notes: non-transition → readOnlyHint=false, destructiveHint=false
        let update = tools_json
            .iter()
            .find(|t| t["name"] == "update_notes")
            .expect("update_notes tool must be present");
        assert_eq!(
            update["annotations"]["readOnlyHint"], false,
            "update_notes readOnlyHint must be false"
        );
        assert_eq!(
            update["annotations"]["destructiveHint"], false,
            "update_notes (non-transition) destructiveHint must be false"
        );
    }

    // -------------------------------------------------------------------------
    // Phase 241 Plan 03: CRUD verb tool call routes through the real dispatch
    // path and returns a structured success envelope — NOT a -32601 error and
    // NOT an NTI stub. Extends the Phase 205 regression guard.
    // -------------------------------------------------------------------------

    /// Phase 241-03 / D-10: calling a CRUD verb tool must return a well-formed
    /// CallToolResult with is_error=false and status=ok — NOT a JSON-RPC -32601
    /// error — keeping the Phase 205 regression guard green.
    ///
    /// Phase 240 asserted error_kind=not_yet_implemented; Phase 241 replaces
    /// that stub with real derive_crud_plan + dispatch_write, so the envelope
    /// now carries status=ok with the created record.
    #[tokio::test]
    async fn crud_tool_call_nti_parses_as_valid_mcp_content() {
        // Use a DB whose schema matches the minimal service below (id + status only).
        // setup_orders_db() has a richer schema with NOT NULL columns that the
        // minimal service would leave unset — the INSERT would fail.
        let db = {
            let d = Database::connect("sqlite::memory:")
                .await
                .expect("sqlite connect");
            d.execute(Statement::from_string(
                DatabaseBackend::Sqlite,
                "CREATE TABLE IF NOT EXISTS orders (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    status TEXT,
                    created_at TEXT DEFAULT (datetime('now'))
                )"
                .to_string(),
            ))
            .await
            .expect("create orders table");
            d.execute(Statement::from_string(
                DatabaseBackend::Sqlite,
                "CREATE TABLE IF NOT EXISTS mcp_idempotency_keys (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    tenant_id INTEGER NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    result TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    UNIQUE (tenant_id, idempotency_key)
                )"
                .to_string(),
            ))
            .await
            .expect("create idempotency table");
            d.execute(Statement::from_string(
                DatabaseBackend::Sqlite,
                "CREATE TABLE IF NOT EXISTS audit_log (
                    id TEXT PRIMARY KEY NOT NULL,
                    tenant_id TEXT,
                    actor_kind TEXT NOT NULL,
                    actor_id TEXT,
                    action TEXT NOT NULL,
                    target_kind TEXT,
                    target_id TEXT,
                    before TEXT,
                    after TEXT,
                    reason TEXT,
                    correlation_id TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                )"
                .to_string(),
            ))
            .await
            .expect("create audit_log table");
            d
        };

        // Service with creatable=true and mcp_write_ability declared (Track A flags).
        let service = ServiceDef::new("order")
            .mcp_exposed(true)
            .creatable(true)
            .mcp_write_ability("write-orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status);

        let call_params = serde_json::json!({
            "name": "create_order",
            "arguments": { "status": "pending" }
        });

        // read_write scope + write_authorized: Some(true) so the scope gate and the
        // write-ability gate (Phase 242) both pass — this test exercises CRUD dispatch.
        let ctx = McpContext {
            scope: Some("read_write".to_string()),
            write_authorized: Some(true),
            ..Default::default()
        };

        let noop_dispatcher = crate::WriteDispatcher::new(
            Box::new(|_, _, _, _| Box::pin(async { Ok(serde_json::json!({})) })),
            Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
        );

        let response = handle_tools_call(
            call_params,
            &[service],
            &db,
            Some(1),
            &ctx,
            &noop_dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &test_config(),
        )
        .await;

        // Load-bearing: CRUD response must parse as CallToolResult (not a -32601 error object).
        let parsed: CallToolResult = serde_json::from_value(response["result"].clone())
            .expect("CRUD result must parse as CallToolResult — NOT a -32601 error");

        assert_eq!(
            parsed.is_error,
            Some(false),
            "CRUD success envelope must have is_error=false"
        );
        assert_eq!(
            parsed.content.len(),
            1,
            "structured() produces exactly one content block"
        );

        let sc = parsed
            .structured_content
            .expect("structuredContent must be present in CRUD envelope");
        assert_eq!(
            sc["status"].as_str(),
            Some("ok"),
            "CRUD success envelope structuredContent.status must be 'ok'"
        );
    }

    /// Phase 240-03 WR-04 regression: the NTI envelope must be gated on the matching
    /// opt-in flag. A service that did NOT opt into create emits no `create_<svc>` tool,
    /// so a `create_<svc>` call must fall through to the genuine -32601 "unknown tool"
    /// path — returning a misleading not_yet_implemented envelope would advertise a tool
    /// that does not exist.
    #[tokio::test]
    async fn crud_nti_not_returned_when_verb_flag_disabled() {
        let db = setup_orders_db().await;

        // creatable is NOT set (default false) — no create_order tool is emitted.
        let service = ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status);

        let call_params = serde_json::json!({
            "name": "create_order",
            "arguments": { "status": "pending" }
        });

        // write_authorized: Some(true) so the write-ability gate (Phase 242) passes and the
        // CRUD prefix loop runs. The loop finds no matching service (creatable=false) → -32601.
        let ctx = McpContext {
            scope: Some("read_write".to_string()),
            write_authorized: Some(true),
            ..Default::default()
        };

        let noop_dispatcher = crate::WriteDispatcher::new(
            Box::new(|_, _, _, _| Box::pin(async { Ok(serde_json::json!({})) })),
            Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
        );

        let response = handle_tools_call(
            call_params,
            &[service],
            &db,
            Some(1),
            &ctx,
            &noop_dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &test_config(),
        )
        .await;

        // Must NOT be an NTI envelope — an unflagged verb has no tool, so this is -32601.
        assert!(
            response.get("result").is_none(),
            "an unflagged CRUD verb must not return a not_yet_implemented result envelope"
        );
        assert_eq!(
            response["error"]["code"].as_i64(),
            Some(-32601),
            "calling a non-emitted CRUD verb must return -32601 (unknown tool), not NTI"
        );
    }
}
