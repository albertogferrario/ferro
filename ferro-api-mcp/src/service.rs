use std::sync::Arc;

use rmcp::{
    handler::server::{
        router::tool::{ToolRoute, ToolRouter},
        tool::ToolCallContext,
    },
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
        PaginatedRequestParam, ServerCapabilities, ServerInfo, Tool, ToolAnnotations,
    },
    service::RequestContext,
    RoleServer, ServerHandler,
};

use crate::http::HttpClient;
use crate::types::ApiOperation;

/// MCP service that dynamically registers one tool per OpenAPI operation.
pub struct ApiMcpService {
    api_name: String,
    tool_router: ToolRouter<Self>,
    tool_count: usize,
}

impl ApiMcpService {
    /// Build an MCP service from parsed API operations.
    ///
    /// Each `ApiOperation` becomes one MCP tool. The `http_client` is shared
    /// across all tool handlers for executing API calls.
    pub fn new(
        api_name: String,
        operations: Vec<ApiOperation>,
        http_client: Arc<HttpClient>,
    ) -> Self {
        let tool_count = operations.len();
        let mut router = ToolRouter::new();

        for op in operations {
            let annotations = annotations_for_method(&op.method);
            let input_schema = input_schema_to_arc_map(&op.input_schema);

            let tool = Tool::new(op.tool_name.clone(), op.description.clone(), input_schema)
                .annotate(annotations);

            let client = Arc::clone(&http_client);
            let route = ToolRoute::new_dyn(tool, move |ctx: ToolCallContext<'_, Self>| {
                let client = Arc::clone(&client);
                let op = op.clone();
                Box::pin(async move {
                    let args = ctx.arguments.unwrap_or_default();
                    match client.execute(&op, &args).await {
                        Ok(response) => {
                            let text = serde_json::to_string_pretty(&response)
                                .unwrap_or_else(|_| response.to_string());
                            Ok(CallToolResult::success(vec![Content::text(text)]))
                        }
                        Err(err) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "API call failed: {err}"
                        ))])),
                    }
                })
            });

            router.add_route(route);
        }

        Self {
            api_name,
            tool_router: router,
            tool_count,
        }
    }
}

impl ServerHandler for ApiMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "ferro-api-mcp".to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            instructions: Some(format!(
                "API tools for {}. {} tools available. Use these tools to interact with the API.",
                self.api_name, self.tool_count
            )),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult::with_all_items(
            self.tool_router.list_all(),
        )))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_ {
        let tcc = ToolCallContext::new(self, request, context);
        async move { self.tool_router.call(tcc).await }
    }
}

/// Map HTTP method to MCP tool annotations.
fn annotations_for_method(method: &str) -> ToolAnnotations {
    match method.to_uppercase().as_str() {
        "GET" => ToolAnnotations::new()
            .read_only(true)
            .idempotent(true)
            .open_world(true),
        "POST" => ToolAnnotations::new().read_only(false).open_world(true),
        "PUT" | "PATCH" => ToolAnnotations::new()
            .read_only(false)
            .idempotent(true)
            .open_world(true),
        "DELETE" => ToolAnnotations::new()
            .read_only(false)
            .destructive(true)
            .open_world(true),
        _ => ToolAnnotations::new().open_world(true),
    }
}

/// Convert a `serde_json::Value` (expected to be an object) into
/// `Arc<serde_json::Map<String, serde_json::Value>>` for the MCP `Tool` input schema.
fn input_schema_to_arc_map(
    value: &serde_json::Value,
) -> Arc<serde_json::Map<String, serde_json::Value>> {
    match value {
        serde_json::Value::Object(map) => Arc::new(map.clone()),
        _ => Arc::new(serde_json::Map::new()),
    }
}

// Required for async trait methods in ServerHandler
use std::future::Future;
