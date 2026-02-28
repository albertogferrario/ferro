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

            let description = match &op.hint {
                Some(hint) => format!("{}\n\nHint: {hint}", op.description),
                None => op.description.clone(),
            };

            let tool =
                Tool::new(op.tool_name.clone(), description, input_schema).annotate(annotations);

            let client = Arc::clone(&http_client);
            let route = ToolRoute::new_dyn(tool, move |ctx: ToolCallContext<'_, Self>| {
                let client = Arc::clone(&client);
                let op = op.clone();
                Box::pin(async move {
                    let args = ctx.arguments.unwrap_or_default();

                    let validation_errors = validate_args(&op.input_schema, &args);
                    if !validation_errors.is_empty() {
                        let msg = format!(
                            "Invalid arguments:\n{}",
                            validation_errors
                                .iter()
                                .map(|e| format!("  - {e}"))
                                .collect::<Vec<_>>()
                                .join("\n")
                        );
                        return Ok(CallToolResult::error(vec![Content::text(msg)]));
                    }

                    match client.execute(&op, &args).await {
                        Ok(response) => {
                            let text = serde_json::to_string_pretty(&response)
                                .unwrap_or_else(|_| response.to_string());
                            Ok(CallToolResult::success(vec![Content::text(text)]))
                        }
                        Err(err) => {
                            let msg = match &err {
                                crate::error::Error::ApiError { status, body } => {
                                    format!("API returned HTTP {status}:\n{body}")
                                }
                                crate::error::Error::HttpClient(detail) => {
                                    format!("Connection error: {detail}")
                                }
                                other => format!("Error: {other}"),
                            };
                            Ok(CallToolResult::error(vec![Content::text(msg)]))
                        }
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

/// Validate tool arguments against the operation's input schema.
///
/// Checks required fields are present and basic type correctness.
/// Returns a list of validation errors, empty if valid.
fn validate_args(
    input_schema: &serde_json::Value,
    args: &serde_json::Map<String, serde_json::Value>,
) -> Vec<String> {
    let mut errors = Vec::new();

    // Check required fields
    if let Some(required) = input_schema.get("required").and_then(|r| r.as_array()) {
        for field in required {
            if let Some(name) = field.as_str() {
                if !args.contains_key(name) {
                    errors.push(format!("missing required field: '{name}'"));
                }
            }
        }
    }

    // Check type correctness for provided fields
    if let Some(properties) = input_schema.get("properties").and_then(|p| p.as_object()) {
        for (name, value) in args {
            if let Some(prop_schema) = properties.get(name) {
                if let Some(expected_type) = prop_schema.get("type").and_then(|t| t.as_str()) {
                    let type_ok = match expected_type {
                        "string" => value.is_string(),
                        "integer" => value.is_i64() || value.is_u64(),
                        "number" => value.is_number(),
                        "boolean" => value.is_boolean(),
                        "object" => value.is_object(),
                        "array" => value.is_array(),
                        _ => true,
                    };
                    if !type_ok {
                        errors.push(format!(
                            "field '{name}' expects type '{expected_type}', got {}",
                            json_type_name(value)
                        ));
                    }
                }
            }
        }
    }

    errors
}

/// Returns a human-readable type name for a JSON value.
fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// Required for async trait methods in ServerHandler
use std::future::Future;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_args_catches_missing_required_field() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "email": {"type": "string"}
            },
            "required": ["name", "email"]
        });
        let mut args = serde_json::Map::new();
        args.insert("name".to_string(), json!("Alice"));
        // email missing

        let errors = validate_args(&schema, &args);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("email"));
    }

    #[test]
    fn validate_args_catches_wrong_type() {
        let schema = json!({
            "type": "object",
            "properties": {
                "count": {"type": "integer"}
            },
            "required": []
        });
        let mut args = serde_json::Map::new();
        args.insert("count".to_string(), json!("not a number"));

        let errors = validate_args(&schema, &args);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("count"));
        assert!(errors[0].contains("integer"));
        assert!(errors[0].contains("string"));
    }

    #[test]
    fn validate_args_passes_valid_args() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"},
                "active": {"type": "boolean"}
            },
            "required": ["name"]
        });
        let mut args = serde_json::Map::new();
        args.insert("name".to_string(), json!("Alice"));
        args.insert("age".to_string(), json!(30));
        args.insert("active".to_string(), json!(true));

        let errors = validate_args(&schema, &args);
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_args_ignores_unknown_fields() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        });
        let mut args = serde_json::Map::new();
        args.insert("name".to_string(), json!("Alice"));
        args.insert("extra_field".to_string(), json!(42));

        let errors = validate_args(&schema, &args);
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_args_passes_empty_required() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": []
        });
        let args = serde_json::Map::new();

        let errors = validate_args(&schema, &args);
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_args_checks_all_types() {
        let schema = json!({
            "type": "object",
            "properties": {
                "s": {"type": "string"},
                "n": {"type": "number"},
                "b": {"type": "boolean"},
                "a": {"type": "array"},
                "o": {"type": "object"}
            },
            "required": []
        });
        let mut args = serde_json::Map::new();
        args.insert("s".to_string(), json!(123)); // wrong: number instead of string
        args.insert("n".to_string(), json!("text")); // wrong: string instead of number
        args.insert("b".to_string(), json!("true")); // wrong: string instead of boolean
        args.insert("a".to_string(), json!({})); // wrong: object instead of array
        args.insert("o".to_string(), json!([])); // wrong: array instead of object

        let errors = validate_args(&schema, &args);
        assert_eq!(errors.len(), 5);
    }

    #[test]
    fn validate_args_number_accepts_integers() {
        let schema = json!({
            "type": "object",
            "properties": {
                "value": {"type": "number"}
            },
            "required": []
        });
        let mut args = serde_json::Map::new();
        args.insert("value".to_string(), json!(42));

        let errors = validate_args(&schema, &args);
        assert!(errors.is_empty());
    }

    #[test]
    fn json_type_name_returns_correct_names() {
        assert_eq!(json_type_name(&json!(null)), "null");
        assert_eq!(json_type_name(&json!(true)), "boolean");
        assert_eq!(json_type_name(&json!(42)), "number");
        assert_eq!(json_type_name(&json!("hello")), "string");
        assert_eq!(json_type_name(&json!([])), "array");
        assert_eq!(json_type_name(&json!({})), "object");
    }
}
