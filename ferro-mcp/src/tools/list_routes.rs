//! List routes tool - parse and list application routes
//!
//! This tool tries to fetch routes from the running application first via
//! the `/_ferro/routes` debug endpoint, falling back to static file parsing
//! when the app isn't running.

use crate::error::{McpError, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Timeout for HTTP requests to the running application
const HTTP_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Serialize)]
pub struct RoutesInfo {
    pub routes: Vec<RouteInfo>,
    /// Indicates whether routes came from runtime or static analysis
    pub source: RouteSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteSource {
    /// Routes fetched from running application via HTTP endpoint
    Runtime,
    /// Routes parsed from source files (fallback when app not running)
    StaticAnalysis,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub handler: String,
    pub name: Option<String>,
    #[serde(default)]
    pub middleware: Vec<String>,
}

/// Response format from the `/_ferro/routes` endpoint
#[derive(Debug, Deserialize)]
struct DebugResponse {
    success: bool,
    data: Vec<RuntimeRouteInfo>,
}

/// Route info as returned by the runtime endpoint
#[derive(Debug, Deserialize)]
struct RuntimeRouteInfo {
    method: String,
    path: String,
    name: Option<String>,
    middleware: Vec<String>,
}

/// Try to fetch routes from the running application
async fn fetch_runtime_routes(base_url: &str) -> Option<Vec<RouteInfo>> {
    let url = format!("{base_url}/_ferro/routes");

    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .ok()?;
    let response = client.get(&url).send().await.ok()?;

    if !response.status().is_success() {
        return None;
    }

    let debug_response: DebugResponse = response.json().await.ok()?;

    if !debug_response.success {
        return None;
    }

    Some(
        debug_response
            .data
            .into_iter()
            .map(|r| RouteInfo {
                method: r.method,
                path: r.path,
                handler: String::new(), // Runtime doesn't expose handler names yet
                name: r.name,
                middleware: r.middleware,
            })
            .collect(),
    )
}

pub async fn execute(project_root: &Path) -> Result<RoutesInfo> {
    // Try runtime endpoint first
    for base_url in ["http://localhost:8080", "http://127.0.0.1:8080"] {
        if let Some(routes) = fetch_runtime_routes(base_url).await {
            return Ok(RoutesInfo {
                routes,
                source: RouteSource::Runtime,
            });
        }
    }

    // Fall back to static analysis
    let routes = parse_routes_from_files(project_root)?;
    Ok(RoutesInfo {
        routes,
        source: RouteSource::StaticAnalysis,
    })
}

/// Parse routes from source files (static analysis fallback)
fn parse_routes_from_files(project_root: &Path) -> Result<Vec<RouteInfo>> {
    let routes_file = project_root.join("src/routes.rs");

    if !routes_file.exists() {
        return Err(McpError::FileNotFound("src/routes.rs".to_string()));
    }

    let content = fs::read_to_string(&routes_file).map_err(McpError::IoError)?;

    Ok(parse_routes(&content))
}

fn parse_routes(content: &str) -> Vec<RouteInfo> {
    let mut routes = Vec::new();

    // Pattern to match route definitions like:
    // get!("/path", controllers::module::function).name("route.name")
    // post!("/path/{id}", controllers::module::function)
    let route_pattern = Regex::new(
        r#"(get|post|put|patch|delete)!\s*\(\s*"([^"]+)"\s*,\s*([a-zA-Z_][a-zA-Z0-9_:]*)\s*\)(?:\s*\.name\s*\(\s*"([^"]+)"\s*\))?(?:\s*\.middleware\s*\(\s*([^)]+)\s*\))?"#
    ).unwrap();

    for cap in route_pattern.captures_iter(content) {
        let method = cap
            .get(1)
            .map(|m| m.as_str().to_uppercase())
            .unwrap_or_default();
        let path = cap
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let handler = cap
            .get(3)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let name = cap.get(4).map(|m| m.as_str().to_string());
        let middleware_str = cap.get(5).map(|m| m.as_str()).unwrap_or("");

        // Parse middleware list
        let middleware: Vec<String> = if middleware_str.is_empty() {
            Vec::new()
        } else {
            middleware_str
                .split(',')
                .map(|s| {
                    s.trim()
                        .trim_matches(|c| c == '[' || c == ']' || c == '"')
                        .to_string()
                })
                .filter(|s| !s.is_empty())
                .collect()
        };

        routes.push(RouteInfo {
            method,
            path,
            handler,
            name,
            middleware,
        });
    }

    // Also try to parse route groups
    let group_pattern =
        Regex::new(r#"group!\s*\(\s*"([^"]+)"\s*,\s*\[([^\]]+)\]\s*(?:,\s*\[([^\]]+)\])?\s*\)"#)
            .unwrap();

    for cap in group_pattern.captures_iter(content) {
        let prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let group_routes = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let group_middleware = cap.get(3).map(|m| m.as_str()).unwrap_or("");

        // Parse middleware for group
        let middleware: Vec<String> = if group_middleware.is_empty() {
            Vec::new()
        } else {
            group_middleware
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        // Parse nested routes in group
        for nested_cap in route_pattern.captures_iter(group_routes) {
            let method = nested_cap
                .get(1)
                .map(|m| m.as_str().to_uppercase())
                .unwrap_or_default();
            let path = nested_cap
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let handler = nested_cap
                .get(3)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let name = nested_cap.get(4).map(|m| m.as_str().to_string());

            let full_path = format!("{prefix}{path}");

            routes.push(RouteInfo {
                method,
                path: full_path,
                handler,
                name,
                middleware: middleware.clone(),
            });
        }
    }

    routes
}
