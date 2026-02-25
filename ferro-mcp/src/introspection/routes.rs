//! Route introspection

use crate::tools::list_routes::RouteInfo;
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn scan_routes(project_root: &Path) -> Vec<RouteInfo> {
    let routes_file = project_root.join("src/routes.rs");

    if !routes_file.exists() {
        return Vec::new();
    }

    let Ok(content) = fs::read_to_string(&routes_file) else {
        return Vec::new();
    };

    parse_routes(&content)
}

fn parse_routes(content: &str) -> Vec<RouteInfo> {
    let mut routes = Vec::new();

    // Pattern for route definitions like:
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

        let middleware = parse_middleware(middleware_str);

        routes.push(RouteInfo {
            method,
            path,
            handler,
            name,
            middleware,
        });
    }

    // Also try to parse route groups
    parse_route_groups(content, &mut routes);

    routes
}

fn parse_middleware(middleware_str: &str) -> Vec<String> {
    if middleware_str.is_empty() {
        return Vec::new();
    }

    middleware_str
        .split(',')
        .map(|s| {
            s.trim()
                .trim_matches(|c| c == '[' || c == ']' || c == '"')
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_route_groups(content: &str, routes: &mut Vec<RouteInfo>) {
    let group_pattern =
        Regex::new(r#"group!\s*\(\s*"([^"]+)"\s*,\s*\[([^\]]+)\]\s*(?:,\s*\[([^\]]+)\])?\s*\)"#)
            .unwrap();

    let route_pattern = Regex::new(
        r#"(get|post|put|patch|delete)!\s*\(\s*"([^"]+)"\s*,\s*([a-zA-Z_][a-zA-Z0-9_:]*)\s*\)(?:\s*\.name\s*\(\s*"([^"]+)"\s*\))?"#
    ).unwrap();

    for cap in group_pattern.captures_iter(content) {
        let prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let group_routes = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let group_middleware = cap.get(3).map(|m| m.as_str()).unwrap_or("");

        let middleware = parse_middleware(group_middleware);

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
}
