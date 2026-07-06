//! Get handler tool - returns the source code of a handler function

use crate::error::{McpError, Result};
use crate::tools::list_routes;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct HandlerInfo {
    pub handler: String,
    pub path: String,
    pub method: String,
    pub file_path: String,
    pub source_code: String,
    pub line_start: usize,
    pub line_end: usize,
    /// The Inertia component this handler renders (if any)
    pub component: Option<String>,
    /// The props struct name being sent (if detected)
    pub props_struct: Option<String>,
    /// Fields being sent to the frontend
    pub props_fields: Vec<PropsField>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PropsField {
    pub name: String,
    pub value_source: String,
}

pub async fn execute(project_root: &Path, route_path: &str) -> Result<HandlerInfo> {
    // First find the handler for this route
    let routes = list_routes::execute(project_root).await?;

    let route = routes
        .routes
        .iter()
        .find(|r| r.path == route_path)
        .ok_or_else(|| McpError::NotFound(format!("Route not found: {route_path}")))?;

    let handler = &route.handler;

    // Parse handler path like "controllers::shelter::animals::show"
    let parts: Vec<&str> = handler.split("::").collect();
    if parts.len() < 2 {
        return Err(McpError::ParseError(format!(
            "Invalid handler path: {handler}"
        )));
    }

    // Build the file path
    // controllers::shelter::animals::show -> src/controllers/shelter/animals.rs
    let file_parts: Vec<&str> = parts[..parts.len() - 1].to_vec();
    let function_name = parts[parts.len() - 1];

    let file_path = project_root
        .join("src")
        .join(file_parts.join("/"))
        .with_extension("rs");

    if !file_path.exists() {
        // Try as directory with mod.rs
        let mod_path = project_root
            .join("src")
            .join(file_parts.join("/"))
            .join("mod.rs");

        if mod_path.exists() {
            return extract_handler(&mod_path, function_name, handler, route, project_root);
        }

        return Err(McpError::FileNotFound(format!(
            "Handler file not found: {}",
            file_path.display()
        )));
    }

    extract_handler(&file_path, function_name, handler, route, project_root)
}

fn extract_handler(
    file_path: &Path,
    function_name: &str,
    handler: &str,
    route: &list_routes::RouteInfo,
    project_root: &Path,
) -> Result<HandlerInfo> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| McpError::FileReadError(format!("{}: {}", file_path.display(), e)))?;

    let lines: Vec<&str> = content.lines().collect();

    // Find the function with #[handler] attribute
    let mut line_start = None;
    let mut in_handler = false;
    let mut brace_count = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Look for #[handler] attribute followed by the function
        if trimmed.starts_with("#[handler") {
            // Check if next non-empty, non-attribute line is our function
            for next_line in lines.iter().skip(i + 1) {
                let next = next_line.trim();
                if next.is_empty() || next.starts_with("#[") {
                    continue;
                }
                if next.contains(&format!("fn {function_name}"))
                    || next.contains(&format!("pub fn {function_name}"))
                    || next.contains(&format!("pub async fn {function_name}"))
                    || next.contains(&format!("async fn {function_name}"))
                {
                    line_start = Some(i);
                    in_handler = true;
                    brace_count = 0;
                }
                break;
            }
        }

        if in_handler {
            brace_count += line.chars().filter(|&c| c == '{').count();
            brace_count = brace_count.saturating_sub(line.chars().filter(|&c| c == '}').count());

            if brace_count == 0 && line.contains('}') {
                let start = line_start.unwrap();
                let end = i + 1;

                let source_code = lines[start..end].join("\n");
                let relative_path = file_path
                    .strip_prefix(project_root)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string();

                // Extract component and props info
                let (component, props_struct, props_fields) = extract_inertia_info(&source_code);

                return Ok(HandlerInfo {
                    handler: handler.to_string(),
                    path: route.path.clone(),
                    method: route.method.clone(),
                    file_path: relative_path,
                    source_code,
                    line_start: start + 1, // 1-indexed
                    line_end: end,
                    component,
                    props_struct,
                    props_fields,
                });
            }
        }
    }

    Err(McpError::NotFound(format!(
        "Handler function '{}' not found in {}",
        function_name,
        file_path.display()
    )))
}

/// Extract Inertia component name, props struct, and fields from handler source code
fn extract_inertia_info(source_code: &str) -> (Option<String>, Option<String>, Vec<PropsField>) {
    let mut component = None;
    let mut props_struct = None;
    let mut props_fields = Vec::new();

    // Pattern to find inertia_response!("ComponentName", PropsStruct { ... })
    let inertia_pattern =
        Regex::new(r#"inertia_response!\s*\(\s*"([^"]+)"\s*,\s*([A-Z][a-zA-Z0-9]*)\s*\{([^}]*)\}"#);

    if let Ok(pattern) = inertia_pattern {
        if let Some(cap) = pattern.captures(source_code) {
            component = cap.get(1).map(|m| m.as_str().to_string());
            props_struct = cap.get(2).map(|m| m.as_str().to_string());

            if let Some(fields_match) = cap.get(3) {
                props_fields = parse_props_fields(fields_match.as_str());
            }
        }
    }

    // Fallback: just try to find the component name
    if component.is_none() {
        let simple_pattern = Regex::new(r#"inertia_response!\s*\(\s*"([^"]+)""#);
        if let Ok(pattern) = simple_pattern {
            if let Some(cap) = pattern.captures(source_code) {
                component = cap.get(1).map(|m| m.as_str().to_string());
            }
        }
    }

    (component, props_struct, props_fields)
}

/// Parse props fields from struct instantiation
fn parse_props_fields(fields_str: &str) -> Vec<PropsField> {
    let mut fields = Vec::new();

    // Pattern: field_name: value or field_name (shorthand)
    let field_pattern = Regex::new(r#"(\w+)\s*(?::\s*([^,\n]+))?"#);

    if let Ok(pattern) = field_pattern {
        for cap in pattern.captures_iter(fields_str) {
            let name = cap
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            if name.is_empty() {
                continue;
            }

            let value_source = cap
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_else(|| name.clone()); // shorthand: field = field

            fields.push(PropsField { name, value_source });
        }
    }

    fields
}
