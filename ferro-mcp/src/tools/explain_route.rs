//! explain_route tool - Explains the purpose and business context of a route
//!
//! Returns semantic information about a route to help agents understand
//! WHY a route exists, not just WHAT it does.

use crate::error::{McpError, Result};
use crate::tools::list_routes::{execute as list_routes, RouteInfo};
use serde::Serialize;
use std::path::Path;

/// Explanation of a route's purpose and context
#[derive(Debug, Serialize)]
pub struct RouteExplanation {
    /// The route path
    pub route: String,
    /// HTTP method
    pub method: String,
    /// Inferred purpose of this route
    pub purpose: String,
    /// Business context explanation
    pub business_context: String,
    /// Guards/middleware that protect this route
    pub guards: Vec<String>,
    /// Related routes that work with this one
    pub related_routes: Vec<String>,
    /// Example usage patterns
    pub usage_examples: Vec<String>,
    /// Route name if defined
    pub name: Option<String>,
    /// Handler function path
    pub handler: String,
}

/// Execute the explain_route tool
pub async fn execute(project_root: &Path, route_path: &str) -> Result<RouteExplanation> {
    // Get all routes
    let routes_info = list_routes(project_root).await?;
    let routes = routes_info.routes;

    // Find the matching route
    let route = routes
        .iter()
        .find(|r| r.path == route_path)
        .or_else(|| {
            // Try matching without leading slash
            let normalized = route_path.trim_start_matches('/');
            routes
                .iter()
                .find(|r| r.path.trim_start_matches('/') == normalized)
        })
        .ok_or_else(|| McpError::NotFound(format!("Route '{route_path}' not found")))?;

    // Find related routes (same resource or prefix)
    let related = find_related_routes(&routes, &route.path);

    // Generate explanation
    Ok(RouteExplanation {
        route: route.path.clone(),
        method: route.method.clone(),
        purpose: infer_purpose(&route.method, &route.path, &route.handler),
        business_context: infer_business_context(&route.path, &route.handler),
        guards: extract_guards(&route.middleware),
        related_routes: related,
        usage_examples: generate_usage_examples(&route.method, &route.path),
        name: route.name.clone(),
        handler: route.handler.clone(),
    })
}

/// Infer the purpose of a route from its method, path, and handler
fn infer_purpose(method: &str, path: &str, handler: &str) -> String {
    // Extract resource name from path
    let resource = extract_resource_name(path);

    // Check for common CRUD patterns
    let handler_fn = handler.split("::").last().unwrap_or("");

    match (method, handler_fn) {
        ("GET", "index") | ("GET", "list") => {
            format!("List all {resource} records")
        }
        ("GET", "show") | ("GET", "get") => {
            format!("Display a single {resource} by ID")
        }
        ("GET", "create") | ("GET", "new") => {
            format!("Show form to create a new {resource}")
        }
        ("GET", "edit") => {
            format!("Show form to edit an existing {resource}")
        }
        ("POST", "store") | ("POST", "create") => {
            format!("Create a new {resource} record")
        }
        ("PUT", "update") | ("PATCH", "update") => {
            format!("Update an existing {resource} record")
        }
        ("DELETE", "destroy") | ("DELETE", "delete") => {
            format!("Delete a {resource} record")
        }
        _ => {
            // Infer from path patterns
            if path.contains("login") {
                "Authenticate user credentials".to_string()
            } else if path.contains("logout") {
                "End user session".to_string()
            } else if path.contains("register") {
                "Create new user account".to_string()
            } else if path.contains("dashboard") {
                "Display main dashboard view".to_string()
            } else if path.contains("profile") {
                "Manage user profile".to_string()
            } else if path.contains("settings") {
                "Manage application settings".to_string()
            } else if path.contains("search") {
                format!("Search {resource} records")
            } else if path.contains("health") || path.contains("status") {
                "Health check endpoint for monitoring".to_string()
            } else if path.contains("{id}") || path.contains(":id") {
                format!("{} a specific {} by ID", capitalize(method), resource)
            } else {
                format!("{} {}", capitalize(method), resource)
            }
        }
    }
}

/// Infer business context from route path and handler
fn infer_business_context(path: &str, handler: &str) -> String {
    let resource = extract_resource_name(path);

    // Check for common domain patterns
    let path_lower = path.to_lowercase();

    if path_lower.contains("user")
        || path_lower.contains("profile")
        || path_lower.contains("account")
    {
        "User management and authentication workflow".to_string()
    } else if path_lower.contains("admin") {
        "Administrative functions requiring elevated privileges".to_string()
    } else if path_lower.contains("api") {
        format!("API endpoint for programmatic {resource} access")
    } else if path_lower.contains("auth")
        || path_lower.contains("login")
        || path_lower.contains("logout")
    {
        "Authentication and session management".to_string()
    } else if path_lower.contains("rifugio") {
        "Core shelter/refuge entity for visitor browsing and booking".to_string()
    } else if path_lower.contains("prenotazione")
        || path_lower.contains("booking")
        || path_lower.contains("reservation")
    {
        "Reservation workflow for booking resources".to_string()
    } else if path_lower.contains("todo") || path_lower.contains("task") {
        "Task management and productivity tracking".to_string()
    } else if path_lower.contains("notification") {
        "User notification and alert system".to_string()
    } else if path_lower.contains("upload")
        || path_lower.contains("file")
        || path_lower.contains("media")
    {
        "File management and media handling".to_string()
    } else if path_lower.contains("report") || path_lower.contains("analytics") {
        "Reporting and analytics functionality".to_string()
    } else {
        // Extract from handler path
        if handler.contains("controllers::") {
            let parts: Vec<&str> = handler.split("::").collect();
            if parts.len() >= 2 {
                format!(
                    "Core {} functionality in {} domain",
                    resource,
                    parts.get(1).unwrap_or(&"application")
                )
            } else {
                format!("Core {resource} functionality")
            }
        } else {
            format!("Domain functionality for {resource}")
        }
    }
}

/// Extract guards (authentication/authorization middleware) from middleware list
fn extract_guards(middleware: &[String]) -> Vec<String> {
    middleware
        .iter()
        .filter(|m| {
            let lower = m.to_lowercase();
            lower.contains("auth")
                || lower.contains("guard")
                || lower.contains("admin")
                || lower.contains("permission")
                || lower.contains("role")
                || lower.contains("verify")
        })
        .cloned()
        .collect()
}

/// Find routes related to this one (same resource prefix)
fn find_related_routes(routes: &[RouteInfo], current_path: &str) -> Vec<String> {
    let resource = extract_resource_name(current_path);
    let prefix = extract_prefix(current_path);

    routes
        .iter()
        .filter(|r| {
            r.path != current_path
                && (r.path.contains(&resource)
                    || (!prefix.is_empty() && r.path.starts_with(&prefix)))
        })
        .map(|r| format!("{} {}", r.method, r.path))
        .take(5) // Limit to 5 related routes
        .collect()
}

/// Generate usage examples for the route
fn generate_usage_examples(method: &str, path: &str) -> Vec<String> {
    let mut examples = Vec::new();

    // Replace path parameters with example values
    let example_path = path
        .replace("{id}", "123")
        .replace(":id", "123")
        .replace("{slug}", "example-slug")
        .replace(":slug", "example-slug")
        .replace("{uuid}", "550e8400-e29b-41d4-a716-446655440000");

    examples.push(format!("{method} {example_path}"));

    // Add curl example for non-GET methods
    if method != "GET" {
        match method {
            "POST" | "PUT" | "PATCH" => {
                examples.push(format!(
                    "curl -X {method} {example_path} -H 'Content-Type: application/json' -d '{{}}"
                ));
            }
            "DELETE" => {
                examples.push(format!("curl -X DELETE {example_path}"));
            }
            _ => {}
        }
    }

    examples
}

/// Extract resource name from route path
fn extract_resource_name(path: &str) -> String {
    path.split('/')
        .find(|s| {
            !s.is_empty()
                && !s.starts_with('{')
                && !s.starts_with(':')
                && !["api", "v1", "v2", "admin"].contains(s)
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| "resource".to_string())
}

/// Extract common prefix from path
fn extract_prefix(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return String::new();
    }

    // Find first non-parameter segment
    let first_resource = parts
        .iter()
        .find(|s| !s.starts_with('{') && !s.starts_with(':'));

    first_resource.map(|s| format!("/{s}")).unwrap_or_default()
}

/// Capitalize first letter of a string
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_resource_name() {
        assert_eq!(extract_resource_name("/users"), "users");
        assert_eq!(extract_resource_name("/users/{id}"), "users");
        assert_eq!(extract_resource_name("/api/v1/posts"), "posts");
        assert_eq!(extract_resource_name("/admin/users"), "users");
    }

    #[test]
    fn test_extract_prefix() {
        assert_eq!(extract_prefix("/users"), "/users");
        assert_eq!(extract_prefix("/users/{id}"), "/users");
        assert_eq!(extract_prefix("/api/posts"), "/api");
    }

    #[test]
    fn test_infer_purpose() {
        assert!(infer_purpose("GET", "/users", "controllers::user::index").contains("List"));
        assert!(infer_purpose("POST", "/users", "controllers::user::store").contains("Create"));
        assert!(infer_purpose("GET", "/users/{id}", "controllers::user::show").contains("Display"));
    }

    #[test]
    fn test_extract_guards() {
        let middleware = vec![
            "web".to_string(),
            "auth".to_string(),
            "cors".to_string(),
            "AdminGuard".to_string(),
        ];
        let guards = extract_guards(&middleware);
        assert_eq!(guards.len(), 2);
        assert!(guards.contains(&"auth".to_string()));
        assert!(guards.contains(&"AdminGuard".to_string()));
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("get"), "Get");
        assert_eq!(capitalize("POST"), "Post");
        assert_eq!(capitalize(""), "");
    }
}
