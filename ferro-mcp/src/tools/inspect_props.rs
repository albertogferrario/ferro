//! Inspect props tool - detailed inspection of a single InertiaProps struct
//!
//! Shows the Rust struct, TypeScript equivalent, usage in handlers, and validation status.

use crate::error::Result;
use crate::tools::list_props::{self, PropsStruct};
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct InspectPropsResult {
    pub found: bool,
    pub props: Option<PropsStruct>,
    /// Source code of the struct definition
    pub source_code: Option<String>,
    /// Handlers that use this props struct
    pub handlers: Vec<HandlerUsage>,
    /// TypeScript interface that should exist
    pub expected_typescript: Option<String>,
    /// Actual TypeScript interface if it exists
    pub actual_typescript: Option<TypeScriptMatch>,
    /// Validation issues
    pub issues: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HandlerUsage {
    pub handler: String,
    pub component: String,
    pub file_path: String,
    pub line_number: usize,
}

#[derive(Debug, Serialize)]
pub struct TypeScriptMatch {
    pub file_path: String,
    pub interface_name: String,
    pub fields: Vec<TypeScriptField>,
}

#[derive(Debug, Serialize)]
pub struct TypeScriptField {
    pub name: String,
    pub field_type: String,
    pub optional: bool,
}

pub fn execute(project_root: &Path, props_name: &str) -> Result<InspectPropsResult> {
    // First, find the props struct
    let all_props = list_props::execute(project_root, None)?;

    let props = all_props.props.iter().find(|p| p.name == props_name);

    if props.is_none() {
        // Try partial match
        let partial = all_props
            .props
            .iter()
            .find(|p| p.name.to_lowercase().contains(&props_name.to_lowercase()));

        if partial.is_none() {
            return Ok(InspectPropsResult {
                found: false,
                props: None,
                source_code: None,
                handlers: Vec::new(),
                expected_typescript: None,
                actual_typescript: None,
                issues: vec![format!(
                    "Props struct '{}' not found. Available: {}",
                    props_name,
                    all_props
                        .props
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )],
            });
        }
    }

    let props = props
        .or_else(|| {
            all_props
                .props
                .iter()
                .find(|p| p.name.to_lowercase().contains(&props_name.to_lowercase()))
        })
        .unwrap()
        .clone();

    // Get source code
    let source_code = extract_source_code(project_root, &props);

    // Find handlers using this props
    let handlers = find_handlers_using_props(project_root, &props.name);

    // Generate expected TypeScript
    let expected_typescript = Some(props.typescript.clone());

    // Find actual TypeScript interface
    let actual_typescript = find_actual_typescript(project_root, &props.name);

    // Validate
    let issues = validate_props(&props, actual_typescript.as_ref());

    Ok(InspectPropsResult {
        found: true,
        props: Some(props),
        source_code,
        handlers,
        expected_typescript,
        actual_typescript,
        issues,
    })
}

fn extract_source_code(project_root: &Path, props: &PropsStruct) -> Option<String> {
    let file_path = project_root.join(&props.file_path);
    let content = fs::read_to_string(&file_path).ok()?;
    let lines: Vec<&str> = content.lines().collect();

    // Start from line before the struct (to include derive)
    let start_line = props.line_number.saturating_sub(2);
    let mut end_line = props.line_number;

    // Find end of struct
    let mut brace_count = 0;
    for (i, line) in lines
        .iter()
        .enumerate()
        .skip(props.line_number.saturating_sub(1))
    {
        for c in line.chars() {
            if c == '{' {
                brace_count += 1;
            } else if c == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    end_line = i + 1;
                    break;
                }
            }
        }
        if brace_count == 0 && end_line > props.line_number {
            break;
        }
    }

    let source: Vec<&str> = lines
        .iter()
        .skip(start_line)
        .take(end_line - start_line)
        .cloned()
        .collect();

    Some(source.join("\n"))
}

fn find_handlers_using_props(project_root: &Path, props_name: &str) -> Vec<HandlerUsage> {
    let mut handlers = Vec::new();
    let src_path = project_root.join("src");

    // Pattern: inertia_response!("Component", PropsName { ... })
    let pattern = Regex::new(&format!(
        r#"inertia_response!\s*\(\s*"([^"]+)"\s*,\s*{}\s*\{{"#,
        regex::escape(props_name)
    ))
    .ok();

    if pattern.is_none() {
        return handlers;
    }
    let pattern = pattern.unwrap();

    // Pattern to find handler function name
    let handler_pattern = Regex::new(r#"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)"#).unwrap();

    for entry in walkdir::WalkDir::new(&src_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            for cap in pattern.captures_iter(&content) {
                let component = cap
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();

                // Find the line number
                let match_start = cap.get(0).unwrap().start();
                let line_number = content[..match_start].matches('\n').count() + 1;

                // Find the handler function name (look backwards from the match)
                let before_match = &content[..match_start];
                let handler_name = handler_pattern
                    .captures_iter(before_match)
                    .last()
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let relative_path = path
                    .strip_prefix(project_root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                handlers.push(HandlerUsage {
                    handler: handler_name,
                    component,
                    file_path: relative_path,
                    line_number,
                });
            }
        }
    }

    handlers
}

fn find_actual_typescript(project_root: &Path, props_name: &str) -> Option<TypeScriptMatch> {
    // Check common TypeScript locations
    let type_files = [
        project_root.join("frontend/src/types/inertia-props.ts"),
        project_root.join("frontend/src/types/shared.ts"),
        project_root.join("frontend/src/types/index.ts"),
    ];

    let interface_pattern =
        Regex::new(&format!(r#"(?:export\s+)?interface\s+{props_name}\s*\{{"#)).ok()?;

    for type_file in &type_files {
        if !type_file.exists() {
            continue;
        }

        let content = fs::read_to_string(type_file).ok()?;

        if interface_pattern.is_match(&content) {
            // Extract the interface
            let fields = extract_typescript_fields(&content, props_name);

            return Some(TypeScriptMatch {
                file_path: type_file
                    .strip_prefix(project_root)
                    .unwrap_or(type_file)
                    .to_string_lossy()
                    .to_string(),
                interface_name: props_name.to_string(),
                fields,
            });
        }
    }

    None
}

fn extract_typescript_fields(content: &str, interface_name: &str) -> Vec<TypeScriptField> {
    let mut fields = Vec::new();

    // Find the interface and extract its body
    let pattern = Regex::new(&format!(
        r#"(?:export\s+)?interface\s+{}\s*(?:extends\s+[^{{]+)?\{{\s*([^}}]+)\}}"#,
        regex::escape(interface_name)
    ))
    .ok();

    if let Some(pattern) = pattern {
        if let Some(cap) = pattern.captures(content) {
            let body = cap.get(1).map(|m| m.as_str()).unwrap_or("");

            // Parse fields
            let field_pattern = Regex::new(r#"(\w+)(\?)?:\s*([^;,\n]+)"#).unwrap();
            for field_cap in field_pattern.captures_iter(body) {
                let name = field_cap
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let optional = field_cap.get(2).is_some();
                let field_type = field_cap
                    .get(3)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();

                fields.push(TypeScriptField {
                    name,
                    field_type,
                    optional,
                });
            }
        }
    }

    fields
}

fn validate_props(props: &PropsStruct, actual_ts: Option<&TypeScriptMatch>) -> Vec<String> {
    let mut issues = Vec::new();

    if actual_ts.is_none() {
        issues.push(format!(
            "TypeScript interface '{}' not found in type files. Run `ferro generate-types` to create it.",
            props.name
        ));
        return issues;
    }

    let actual = actual_ts.unwrap();

    // Compare fields
    let rust_fields: std::collections::HashSet<_> =
        props.fields.iter().map(|f| f.name.as_str()).collect();
    let ts_fields: std::collections::HashSet<_> =
        actual.fields.iter().map(|f| f.name.as_str()).collect();

    // Fields in Rust but not in TypeScript
    for field in rust_fields.difference(&ts_fields) {
        // Check for camelCase equivalent
        let camel = to_camel_case(field);
        if !actual.fields.iter().any(|f| f.name == camel) {
            issues.push(format!(
                "Field '{field}' exists in Rust but not in TypeScript interface"
            ));
        }
    }

    // Fields in TypeScript but not in Rust
    for field in ts_fields.difference(&rust_fields) {
        // Check for snake_case equivalent
        let snake = to_snake_case(field);
        if !props.fields.iter().any(|f| f.name == snake) {
            let ts_field = actual.fields.iter().find(|f| f.name == *field).unwrap();
            if !ts_field.optional {
                issues.push(format!(
                    "Required field '{field}' in TypeScript but not in Rust struct"
                ));
            }
        }
    }

    issues
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
            result.extend(c.to_lowercase());
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    result
}
