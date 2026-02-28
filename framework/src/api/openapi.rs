//! OpenAPI specification builder and documentation handlers.
//!
//! Generates OpenAPI specs from Ferro route metadata and serves
//! interactive API documentation via ReDoc.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{build_openapi_spec, OpenApiConfig, get_registered_routes};
//!
//! // Build spec from registered routes
//! let config = OpenApiConfig {
//!     title: "My API".to_string(),
//!     version: "1.0.0".to_string(),
//!     description: Some("My application API".to_string()),
//!     api_prefix: "/api/".to_string(),
//! };
//! let spec = build_openapi_spec(&config, &get_registered_routes());
//!
//! // Serve JSON spec
//! let json = openapi_json_response(&config, &get_registered_routes());
//!
//! // Serve ReDoc HTML
//! let html = openapi_docs_response(&config, &get_registered_routes());
//! ```

use crate::http::HttpResponse;
use crate::routing::RouteInfo;
use std::sync::OnceLock;
use utoipa::openapi::extensions::ExtensionsBuilder;
use utoipa::openapi::path::{HttpMethod, OperationBuilder, ParameterBuilder, ParameterIn};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::openapi::{
    ComponentsBuilder, InfoBuilder, OpenApiBuilder, PathItem, PathsBuilder, Required,
};

/// Configuration for OpenAPI spec generation.
#[derive(Debug, Clone)]
pub struct OpenApiConfig {
    /// API title displayed in documentation
    pub title: String,
    /// API version (e.g., "1.0.0")
    pub version: String,
    /// Optional description displayed in documentation
    pub description: Option<String>,
    /// Route path prefix filter (only routes starting with this are included)
    pub api_prefix: String,
}

impl Default for OpenApiConfig {
    fn default() -> Self {
        Self {
            title: "API Documentation".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            api_prefix: "/api/".to_string(),
        }
    }
}

/// Build an OpenAPI spec from route metadata.
///
/// Filters routes matching `config.api_prefix`, generates operations with
/// auto-summaries and path parameters, and adds API key security scheme.
pub fn build_openapi_spec(
    config: &OpenApiConfig,
    routes: &[RouteInfo],
) -> utoipa::openapi::OpenApi {
    let mut paths = PathsBuilder::new();

    for route in routes
        .iter()
        .filter(|r| r.path.starts_with(&config.api_prefix))
    {
        let http_method = parse_http_method(&route.method);
        let tag = extract_tag(&route.path);
        let summary = auto_summary(&route.method, &route.path);

        let mut op = OperationBuilder::new()
            .tag(tag)
            .summary(Some(summary))
            .operation_id(route.name.clone());

        // Add path parameters extracted from {param} patterns
        for param_name in extract_path_params(&route.path) {
            op = op.parameter(
                ParameterBuilder::new()
                    .name(param_name)
                    .parameter_in(ParameterIn::Path)
                    .required(Required::True)
                    .build(),
            );
        }

        // Emit x-mcp vendor extensions for AI tool discovery
        let extensions = ExtensionsBuilder::new()
            .add("x-mcp-tool-name", mcp_tool_name(&route.method, &route.path))
            .add(
                "x-mcp-description",
                mcp_description(&route.method, &route.path),
            )
            .build();
        op = op.extensions(Some(extensions));

        let operation = op.build();

        // Convert route path from {param} to OpenAPI format (already correct)
        let openapi_path = route.path.clone();

        let path_item = PathItem::new(http_method, operation);
        paths = paths.path(openapi_path, path_item);
    }

    let components = ComponentsBuilder::new()
        .security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
        )
        .build();

    let info = InfoBuilder::new()
        .title(&config.title)
        .version(&config.version)
        .description(config.description.as_deref())
        .build();

    OpenApiBuilder::new()
        .info(info)
        .paths(paths.build())
        .components(Some(components))
        .build()
}

/// Global cache for the generated OpenAPI spec.
static CACHED_SPEC_JSON: OnceLock<String> = OnceLock::new();
static CACHED_DOCS_HTML: OnceLock<String> = OnceLock::new();

/// Return an HttpResponse with the OpenAPI spec as JSON.
///
/// The spec is generated once and cached via OnceLock.
pub fn openapi_json_response(config: &OpenApiConfig, routes: &[RouteInfo]) -> HttpResponse {
    let json = CACHED_SPEC_JSON.get_or_init(|| {
        let spec = build_openapi_spec(config, routes);
        spec.to_json()
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize spec: {e}\"}}"))
    });

    HttpResponse::text(json.clone()).header("Content-Type", "application/json")
}

/// Return an HttpResponse with ReDoc HTML documentation.
///
/// The HTML is generated once and cached via OnceLock.
pub fn openapi_docs_response(config: &OpenApiConfig, routes: &[RouteInfo]) -> HttpResponse {
    let html = CACHED_DOCS_HTML.get_or_init(|| {
        let spec = build_openapi_spec(config, routes);
        utoipa_redoc::Redoc::new(spec).to_html()
    });

    HttpResponse::text(html.clone()).header("Content-Type", "text/html; charset=utf-8")
}

fn parse_http_method(method: &str) -> HttpMethod {
    match method {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "PATCH" => HttpMethod::Patch,
        "DELETE" => HttpMethod::Delete,
        "OPTIONS" => HttpMethod::Options,
        "HEAD" => HttpMethod::Head,
        _ => HttpMethod::Get,
    }
}

/// Extract tag from route path (first non-version segment after api prefix).
///
/// Examples:
/// - "/api/v1/users/{id}" -> "users"
/// - "/api/v1/posts" -> "posts"
/// - "/api/users" -> "users"
fn extract_tag(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Skip "api" prefix and optional version segment ("v1", "v2", etc.)
    let start = segments.iter().position(|s| *s == "api").unwrap_or(0) + 1;
    let start = if segments
        .get(start)
        .is_some_and(|s| s.starts_with('v') && s[1..].chars().all(|c| c.is_ascii_digit()))
    {
        start + 1
    } else {
        start
    };

    segments
        .get(start)
        .filter(|s| !s.starts_with('{'))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "default".to_string())
}

/// Extract the resource name from the last non-parameter path segment.
///
/// Examples:
/// - "/api/v1/users" -> "users"
/// - "/api/v1/users/{id}" -> "user" (singularized)
/// - "/api/v1/posts/{id}/comments" -> "comments"
fn extract_resource_name(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    let resource = segments
        .iter()
        .rev()
        .find(|s| !s.starts_with('{'))
        .unwrap_or(&"resource");

    resource.to_string()
}

/// Check if the path ends with a parameter (e.g., "/{id}").
fn has_path_param(path: &str) -> bool {
    path.split('/')
        .next_back()
        .is_some_and(|s| s.starts_with('{'))
}

/// Generate a human-readable summary from HTTP method and path.
///
/// Examples:
/// - ("GET", "/api/v1/users") -> "List users"
/// - ("GET", "/api/v1/users/{id}") -> "Get user"
/// - ("POST", "/api/v1/users") -> "Create user"
/// - ("PUT", "/api/v1/users/{id}") -> "Update user"
/// - ("DELETE", "/api/v1/users/{id}") -> "Delete user"
fn auto_summary(method: &str, path: &str) -> String {
    let resource = extract_resource_name(path);
    let singular = singularize(&resource);

    match (method, has_path_param(path)) {
        ("GET", false) => format!("List {resource}"),
        ("GET", true) => format!("Get {singular}"),
        ("POST", _) => format!("Create {singular}"),
        ("PUT" | "PATCH", _) => format!("Update {singular}"),
        ("DELETE", _) => format!("Delete {singular}"),
        _ => format!("{method} {path}"),
    }
}

/// Naive singularization: strip trailing 's' if present.
fn singularize(word: &str) -> String {
    if word.ends_with('s') && word.len() > 1 {
        word[..word.len() - 1].to_string()
    } else {
        word.to_string()
    }
}

/// Generate a snake_case tool name for MCP consumption.
///
/// Examples:
/// - ("GET", "/api/v1/users") -> "list_users"
/// - ("GET", "/api/v1/users/{id}") -> "get_user"
/// - ("POST", "/api/v1/users") -> "create_user"
/// - ("DELETE", "/api/v1/users/{id}") -> "delete_user"
fn mcp_tool_name(method: &str, path: &str) -> String {
    let resource = extract_resource_name(path);
    let singular = singularize(&resource);

    match (method, has_path_param(path)) {
        ("GET", false) => format!("list_{resource}"),
        ("GET", true) => format!("get_{singular}"),
        ("POST", _) => format!("create_{singular}"),
        ("PUT" | "PATCH", _) => format!("update_{singular}"),
        ("DELETE", _) => format!("delete_{singular}"),
        _ => format!("{}_{resource}", method.to_lowercase()),
    }
}

/// Generate an AI-optimized description for MCP consumption.
///
/// Examples:
/// - ("GET", "/api/v1/users") -> "List all users with optional pagination."
/// - ("GET", "/api/v1/users/{id}") -> "Get a single user by ID."
/// - ("POST", "/api/v1/users") -> "Create a new user."
fn mcp_description(method: &str, path: &str) -> String {
    let resource = extract_resource_name(path);
    let singular = singularize(&resource);

    match (method, has_path_param(path)) {
        ("GET", false) => format!("List all {resource} with optional pagination."),
        ("GET", true) => format!("Get a single {singular} by ID."),
        ("POST", _) => format!("Create a new {singular}."),
        ("PUT" | "PATCH", _) => format!("Update an existing {singular} by ID."),
        ("DELETE", _) => format!("Permanently delete a {singular} by ID."),
        _ => format!("{method} {path}"),
    }
}

/// Extract path parameter names from `{param}` patterns.
fn extract_path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|s| s.starts_with('{') && s.ends_with('}'))
        .map(|s| s[1..s.len() - 1].to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_summary_list() {
        assert_eq!(auto_summary("GET", "/api/v1/users"), "List users");
    }

    #[test]
    fn auto_summary_get() {
        assert_eq!(auto_summary("GET", "/api/v1/users/{id}"), "Get user");
    }

    #[test]
    fn auto_summary_create() {
        assert_eq!(auto_summary("POST", "/api/v1/users"), "Create user");
    }

    #[test]
    fn auto_summary_update_put() {
        assert_eq!(auto_summary("PUT", "/api/v1/users/{id}"), "Update user");
    }

    #[test]
    fn auto_summary_update_patch() {
        assert_eq!(auto_summary("PATCH", "/api/v1/users/{id}"), "Update user");
    }

    #[test]
    fn auto_summary_delete() {
        assert_eq!(auto_summary("DELETE", "/api/v1/users/{id}"), "Delete user");
    }

    #[test]
    fn auto_summary_fallback() {
        assert_eq!(
            auto_summary("OPTIONS", "/api/v1/health"),
            "OPTIONS /api/v1/health"
        );
    }

    #[test]
    fn extract_resource_name_collection() {
        assert_eq!(extract_resource_name("/api/v1/users"), "users");
    }

    #[test]
    fn extract_resource_name_with_param() {
        assert_eq!(extract_resource_name("/api/v1/users/{id}"), "users");
    }

    #[test]
    fn extract_resource_name_nested() {
        assert_eq!(
            extract_resource_name("/api/v1/posts/{id}/comments"),
            "comments"
        );
    }

    #[test]
    fn extract_tag_simple() {
        assert_eq!(extract_tag("/api/v1/users"), "users");
    }

    #[test]
    fn extract_tag_with_param() {
        assert_eq!(extract_tag("/api/v1/users/{id}"), "users");
    }

    #[test]
    fn extract_tag_no_version() {
        assert_eq!(extract_tag("/api/users"), "users");
    }

    #[test]
    fn extract_tag_nested() {
        assert_eq!(extract_tag("/api/v1/posts/{id}/comments"), "posts");
    }

    #[test]
    fn extract_tag_default() {
        assert_eq!(extract_tag("/api/v1/{id}"), "default");
    }

    #[test]
    fn extract_path_params_none() {
        assert!(extract_path_params("/api/v1/users").is_empty());
    }

    #[test]
    fn extract_path_params_single() {
        assert_eq!(extract_path_params("/api/v1/users/{id}"), vec!["id"]);
    }

    #[test]
    fn extract_path_params_multiple() {
        assert_eq!(
            extract_path_params("/api/v1/users/{user_id}/posts/{post_id}"),
            vec!["user_id", "post_id"]
        );
    }

    #[test]
    fn singularize_plural() {
        assert_eq!(singularize("users"), "user");
    }

    #[test]
    fn singularize_already_singular() {
        assert_eq!(singularize("user"), "user");
    }

    #[test]
    fn singularize_single_char() {
        assert_eq!(singularize("s"), "s");
    }

    #[test]
    fn mcp_tool_name_list() {
        assert_eq!(mcp_tool_name("GET", "/api/v1/users"), "list_users");
    }

    #[test]
    fn mcp_tool_name_get() {
        assert_eq!(mcp_tool_name("GET", "/api/v1/users/{id}"), "get_user");
    }

    #[test]
    fn mcp_tool_name_create() {
        assert_eq!(mcp_tool_name("POST", "/api/v1/users"), "create_user");
    }

    #[test]
    fn mcp_tool_name_update_put() {
        assert_eq!(mcp_tool_name("PUT", "/api/v1/users/{id}"), "update_user");
    }

    #[test]
    fn mcp_tool_name_update_patch() {
        assert_eq!(mcp_tool_name("PATCH", "/api/v1/users/{id}"), "update_user");
    }

    #[test]
    fn mcp_tool_name_delete() {
        assert_eq!(mcp_tool_name("DELETE", "/api/v1/users/{id}"), "delete_user");
    }

    #[test]
    fn mcp_tool_name_nested() {
        assert_eq!(
            mcp_tool_name("GET", "/api/v1/posts/{id}/comments"),
            "list_comments"
        );
    }

    #[test]
    fn mcp_tool_name_fallback() {
        assert_eq!(mcp_tool_name("OPTIONS", "/api/v1/health"), "options_health");
    }

    #[test]
    fn mcp_description_list() {
        assert_eq!(
            mcp_description("GET", "/api/v1/users"),
            "List all users with optional pagination."
        );
    }

    #[test]
    fn mcp_description_get() {
        assert_eq!(
            mcp_description("GET", "/api/v1/users/{id}"),
            "Get a single user by ID."
        );
    }

    #[test]
    fn mcp_description_create() {
        assert_eq!(
            mcp_description("POST", "/api/v1/users"),
            "Create a new user."
        );
    }

    #[test]
    fn mcp_description_update_put() {
        assert_eq!(
            mcp_description("PUT", "/api/v1/users/{id}"),
            "Update an existing user by ID."
        );
    }

    #[test]
    fn mcp_description_update_patch() {
        assert_eq!(
            mcp_description("PATCH", "/api/v1/users/{id}"),
            "Update an existing user by ID."
        );
    }

    #[test]
    fn mcp_description_delete() {
        assert_eq!(
            mcp_description("DELETE", "/api/v1/users/{id}"),
            "Permanently delete a user by ID."
        );
    }

    #[test]
    fn mcp_description_nested() {
        assert_eq!(
            mcp_description("GET", "/api/v1/posts/{id}/comments"),
            "List all comments with optional pagination."
        );
    }

    #[test]
    fn mcp_description_fallback() {
        assert_eq!(
            mcp_description("OPTIONS", "/api/v1/health"),
            "OPTIONS /api/v1/health"
        );
    }

    #[test]
    fn build_spec_basic() {
        let config = OpenApiConfig {
            title: "Test API".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test description".to_string()),
            api_prefix: "/api/".to_string(),
        };

        let routes = vec![
            RouteInfo {
                method: "GET".to_string(),
                path: "/api/v1/users".to_string(),
                name: Some("api.users.index".to_string()),
                middleware: vec![],
            },
            RouteInfo {
                method: "POST".to_string(),
                path: "/api/v1/users".to_string(),
                name: Some("api.users.store".to_string()),
                middleware: vec![],
            },
            RouteInfo {
                method: "GET".to_string(),
                path: "/api/v1/users/{id}".to_string(),
                name: Some("api.users.show".to_string()),
                middleware: vec![],
            },
            RouteInfo {
                method: "PUT".to_string(),
                path: "/api/v1/users/{id}".to_string(),
                name: Some("api.users.update".to_string()),
                middleware: vec![],
            },
            RouteInfo {
                method: "DELETE".to_string(),
                path: "/api/v1/users/{id}".to_string(),
                name: Some("api.users.destroy".to_string()),
                middleware: vec![],
            },
        ];

        let spec = build_openapi_spec(&config, &routes);

        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
        assert_eq!(spec.info.description, Some("Test description".to_string()));

        // Should have paths for both /api/v1/users and /api/v1/users/{id}
        assert_eq!(spec.paths.paths.len(), 2);
        assert!(spec.paths.paths.contains_key("/api/v1/users"));
        assert!(spec.paths.paths.contains_key("/api/v1/users/{id}"));

        // Check GET /api/v1/users has correct operation
        let users_path = spec.paths.paths.get("/api/v1/users").unwrap();
        let get_op = users_path.get.as_ref().unwrap();
        assert_eq!(get_op.summary, Some("List users".to_string()));
        assert_eq!(get_op.operation_id, Some("api.users.index".to_string()));

        // Check POST /api/v1/users
        let post_op = users_path.post.as_ref().unwrap();
        assert_eq!(post_op.summary, Some("Create user".to_string()));

        // Check path parameters on /api/v1/users/{id}
        let user_path = spec.paths.paths.get("/api/v1/users/{id}").unwrap();
        let get_user_op = user_path.get.as_ref().unwrap();
        assert_eq!(get_user_op.summary, Some("Get user".to_string()));
        let params = get_user_op.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "id");

        // Check x-mcp extensions on GET /api/v1/users
        let get_ext = get_op.extensions.as_ref().unwrap();
        assert_eq!(
            get_ext.get("x-mcp-tool-name").unwrap(),
            &serde_json::Value::String("list_users".to_string())
        );
        assert_eq!(
            get_ext.get("x-mcp-description").unwrap(),
            &serde_json::Value::String("List all users with optional pagination.".to_string())
        );

        // Check x-mcp extensions on DELETE /api/v1/users/{id}
        let delete_op = user_path.delete.as_ref().unwrap();
        let delete_ext = delete_op.extensions.as_ref().unwrap();
        assert_eq!(
            delete_ext.get("x-mcp-tool-name").unwrap(),
            &serde_json::Value::String("delete_user".to_string())
        );
        assert_eq!(
            delete_ext.get("x-mcp-description").unwrap(),
            &serde_json::Value::String("Permanently delete a user by ID.".to_string())
        );

        // Check security scheme
        let components = spec.components.as_ref().unwrap();
        assert!(components.security_schemes.contains_key("api_key"));
    }

    #[test]
    fn build_spec_filters_non_api_routes() {
        let config = OpenApiConfig::default();

        let routes = vec![
            RouteInfo {
                method: "GET".to_string(),
                path: "/api/v1/users".to_string(),
                name: None,
                middleware: vec![],
            },
            RouteInfo {
                method: "GET".to_string(),
                path: "/dashboard".to_string(),
                name: None,
                middleware: vec![],
            },
            RouteInfo {
                method: "GET".to_string(),
                path: "/login".to_string(),
                name: None,
                middleware: vec![],
            },
        ];

        let spec = build_openapi_spec(&config, &routes);

        // Only the /api/ route should be included
        assert_eq!(spec.paths.paths.len(), 1);
        assert!(spec.paths.paths.contains_key("/api/v1/users"));
    }

    #[test]
    fn build_spec_empty_routes() {
        let config = OpenApiConfig::default();
        let spec = build_openapi_spec(&config, &[]);

        assert!(spec.paths.paths.is_empty());
        assert_eq!(spec.info.title, "API Documentation");
    }

    #[test]
    fn spec_serializes_to_json() {
        let config = OpenApiConfig::default();
        let routes = vec![RouteInfo {
            method: "GET".to_string(),
            path: "/api/v1/health".to_string(),
            name: None,
            middleware: vec![],
        }];

        let spec = build_openapi_spec(&config, &routes);
        let json = spec.to_json().unwrap();

        assert!(json.contains("\"openapi\""));
        assert!(json.contains("\"info\""));
        assert!(json.contains("\"paths\""));
        assert!(json.contains("/api/v1/health"));
    }

    #[test]
    fn spec_extensions_in_json() {
        let config = OpenApiConfig::default();
        let routes = vec![
            RouteInfo {
                method: "GET".to_string(),
                path: "/api/v1/users".to_string(),
                name: Some("users.index".to_string()),
                middleware: vec![],
            },
            RouteInfo {
                method: "POST".to_string(),
                path: "/api/v1/users".to_string(),
                name: Some("users.store".to_string()),
                middleware: vec![],
            },
        ];

        let spec = build_openapi_spec(&config, &routes);
        let json_str = spec.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Verify GET operation has x-mcp extensions
        let get_op = &parsed["paths"]["/api/v1/users"]["get"];
        assert_eq!(get_op["x-mcp-tool-name"], "list_users");
        assert_eq!(
            get_op["x-mcp-description"],
            "List all users with optional pagination."
        );

        // Verify POST operation has x-mcp extensions
        let post_op = &parsed["paths"]["/api/v1/users"]["post"];
        assert_eq!(post_op["x-mcp-tool-name"], "create_user");
        assert_eq!(post_op["x-mcp-description"], "Create a new user.");
    }
}
