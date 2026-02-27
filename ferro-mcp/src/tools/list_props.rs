//! List props tool - list all InertiaProps structs in the project
//!
//! This provides introspection into the props types without running the CLI.

use crate::error::{McpError, Result};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct ListPropsResult {
    pub total: usize,
    pub props: Vec<PropsStruct>,
    /// Map of component name to props struct name (from inertia_response! calls)
    pub component_mappings: HashMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PropsStruct {
    pub name: String,
    pub file_path: String,
    pub line_number: usize,
    pub fields: Vec<PropsField>,
    /// TypeScript interface preview
    pub typescript: String,
    /// Components that use this props struct
    pub used_by_components: Vec<String>,
    /// Serde rename_all attribute value (e.g., "camelCase")
    pub serde_rename_all: Option<String>,
    /// Module path where this struct is defined (e.g., "shelter::applications")
    pub module_path: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PropsField {
    pub name: String,
    pub rust_type: String,
    pub typescript_type: String,
    pub optional: bool,
    /// Per-field serde rename override
    pub serde_rename: Option<String>,
}

pub fn execute(project_root: &Path, filter: Option<&str>) -> Result<ListPropsResult> {
    let src_path = project_root.join("src");
    if !src_path.exists() {
        return Err(McpError::FileNotFound("src directory".to_string()));
    }

    let mut all_props = Vec::new();
    let mut component_mappings = HashMap::new();

    // Scan all Rust files
    for entry in WalkDir::new(&src_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            // Find InertiaProps structs
            let structs = find_inertia_props_structs(&content, path, project_root);
            all_props.extend(structs);

            // Find component mappings from inertia_response! calls
            let mappings = find_component_mappings(&content);
            component_mappings.extend(mappings);
        }
    }

    // Link props to components
    for props in &mut all_props {
        for (component, props_name) in &component_mappings {
            if props_name == &props.name {
                props.used_by_components.push(component.clone());
            }
        }
    }

    // Apply filter if provided
    if let Some(filter) = filter {
        all_props.retain(|p| {
            p.name.to_lowercase().contains(&filter.to_lowercase())
                || p.file_path.to_lowercase().contains(&filter.to_lowercase())
        });
    }

    let total = all_props.len();

    Ok(ListPropsResult {
        total,
        props: all_props,
        component_mappings,
    })
}

fn find_inertia_props_structs(
    content: &str,
    file_path: &Path,
    project_root: &Path,
) -> Vec<PropsStruct> {
    let mut result = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Pattern to find #[derive(...InertiaProps...)]
    let derive_pattern = Regex::new(r#"#\[derive\([^\)]*InertiaProps[^\)]*\)\]"#).unwrap();

    // Pattern to find struct definition
    let struct_pattern = Regex::new(r#"(?:pub\s+)?struct\s+(\w+)\s*\{"#).unwrap();

    // Pattern to find #[serde(rename_all = "...")]
    let rename_all_pattern =
        Regex::new(r#"#\[serde\([^\)]*rename_all\s*=\s*"([^"]+)"[^\)]*\)\]"#).unwrap();

    // Compute module path for this file
    let src_path = project_root.join("src");
    let module_path = compute_module_path(file_path, &src_path);

    for (i, line) in lines.iter().enumerate() {
        if derive_pattern.is_match(line) {
            // Look for serde(rename_all) between derive and struct
            let mut serde_rename_all = None;
            for line in lines.iter().take(std::cmp::min(i + 5, lines.len())).skip(i) {
                if let Some(cap) = rename_all_pattern.captures(line) {
                    serde_rename_all = Some(cap.get(1).unwrap().as_str().to_string());
                    break;
                }
            }

            // Look for struct definition in next few lines
            for j in (i + 1)..std::cmp::min(i + 5, lines.len()) {
                if let Some(cap) = struct_pattern.captures(lines[j]) {
                    let name = cap.get(1).unwrap().as_str().to_string();

                    // Extract struct body
                    if let Some(struct_body) = extract_struct_body(&lines, j) {
                        let fields = parse_struct_fields(&struct_body);
                        let typescript = generate_typescript_interface(
                            &name,
                            &fields,
                            serde_rename_all.as_deref(),
                        );

                        let relative_path = file_path
                            .strip_prefix(project_root)
                            .unwrap_or(file_path)
                            .to_string_lossy()
                            .to_string();

                        result.push(PropsStruct {
                            name,
                            file_path: relative_path,
                            line_number: j + 1, // 1-indexed
                            fields,
                            typescript,
                            used_by_components: Vec::new(),
                            serde_rename_all,
                            module_path: module_path.clone(),
                        });
                    }
                    break;
                }
            }
        }
    }

    result
}

/// Compute module path from file path, stripping src/ prefix and .rs extension.
///
/// Examples:
/// - "src/controllers/shelter/applications.rs" -> "shelter::applications"
/// - "src/controllers/user.rs" -> "user"
/// - "src/models/animal.rs" -> "models::animal"
fn compute_module_path(file_path: &Path, src_path: &Path) -> String {
    let relative = file_path
        .strip_prefix(src_path)
        .unwrap_or(file_path)
        .with_extension("");

    let path_str = relative
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "::");

    // Remove "mod" suffix if the file is mod.rs
    let path_str = path_str.strip_suffix("::mod").unwrap_or(&path_str);

    // Strip "controllers::" prefix for cleaner namespacing
    path_str
        .strip_prefix("controllers::")
        .unwrap_or(path_str)
        .to_string()
}

fn extract_struct_body(lines: &[&str], start_line: usize) -> Option<String> {
    let mut brace_count = 0;
    let mut body = String::new();
    let mut started = false;

    for line in lines.iter().skip(start_line) {
        for c in line.chars() {
            if c == '{' {
                brace_count += 1;
                started = true;
            } else if c == '}' {
                brace_count -= 1;
            }
        }

        if started {
            body.push_str(line);
            body.push('\n');

            if brace_count == 0 {
                return Some(body);
            }
        }
    }

    None
}

fn parse_struct_fields(body: &str) -> Vec<PropsField> {
    let mut fields = Vec::new();

    // Pattern: pub field_name: Type, or field_name: Type,
    let field_pattern = Regex::new(r#"(?:pub\s+)?(\w+)\s*:\s*([^,\n]+)"#).unwrap();

    // Pattern for #[serde(rename = "...")] - to detect field renames
    let rename_pattern = Regex::new(r#"#\[serde\([^\)]*rename\s*=\s*"([^"]+)"[^\)]*\)\]"#).unwrap();

    let lines: Vec<&str> = body.lines().collect();
    let mut pending_rename: Option<String> = None;

    for line in &lines {
        // Check for serde rename attribute
        if let Some(cap) = rename_pattern.captures(line) {
            // Make sure it's not rename_all
            if !line.contains("rename_all") {
                pending_rename = Some(cap.get(1).unwrap().as_str().to_string());
            }
            continue;
        }

        // Check for field definition
        if let Some(cap) = field_pattern.captures(line) {
            let name = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let rust_type = cap
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            // Skip if it looks like an attribute or comment
            if name.starts_with('#') || name.starts_with('/') || name.is_empty() {
                continue;
            }

            let optional = rust_type.starts_with("Option<");
            let typescript_type = rust_to_typescript(&rust_type);

            fields.push(PropsField {
                name,
                rust_type,
                typescript_type,
                optional,
                serde_rename: pending_rename.take(),
            });
        }
    }

    fields
}

fn rust_to_typescript(rust_type: &str) -> String {
    let ty = rust_type.trim();

    // Handle Option<T>
    if ty.starts_with("Option<") && ty.ends_with('>') {
        let inner = &ty[7..ty.len() - 1];
        return format!("{} | null", rust_to_typescript(inner));
    }

    // Handle Vec<T>
    if ty.starts_with("Vec<") && ty.ends_with('>') {
        let inner = &ty[4..ty.len() - 1];
        return format!("{}[]", rust_to_typescript(inner));
    }

    // Handle HashMap<K, V>
    if (ty.starts_with("HashMap<") || ty.starts_with("BTreeMap<")) && ty.ends_with('>') {
        let start = if ty.starts_with("HashMap<") { 8 } else { 9 };
        let inner = &ty[start..ty.len() - 1];
        // Simple split on comma (doesn't handle nested generics perfectly)
        if let Some(comma_pos) = inner.find(',') {
            let key = inner[..comma_pos].trim();
            let val = inner[comma_pos + 1..].trim();
            return format!(
                "Record<{}, {}>",
                rust_to_typescript(key),
                rust_to_typescript(val)
            );
        }
    }

    // Primitive types
    match ty {
        "String" | "&str" | "str" => "string".to_string(),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "Value" | "Json" | "serde_json::Value" | "sea_orm::Json" | "sea_orm::JsonValue"
        | "JsonValue" => "unknown".to_string(),
        "ValidationErrors" | "ferro::ValidationErrors" => "Record<string, string[]>".to_string(),
        // Custom types pass through
        _ => ty.to_string(),
    }
}

/// Convert snake_case to camelCase
fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert snake_case to PascalCase
fn snake_to_pascal(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Apply serde rename transformation to a field name
fn apply_serde_rename(field_name: &str, rename_all: Option<&str>) -> String {
    match rename_all {
        Some("camelCase") => snake_to_camel(field_name),
        Some("PascalCase") => snake_to_pascal(field_name),
        Some("SCREAMING_SNAKE_CASE") => field_name.to_uppercase(),
        Some("kebab-case") => field_name.replace('_', "-"),
        Some("snake_case") | None => field_name.to_string(),
        Some(_) => field_name.to_string(),
    }
}

fn generate_typescript_interface(
    name: &str,
    fields: &[PropsField],
    serde_rename_all: Option<&str>,
) -> String {
    let mut ts = format!("export interface {name} {{\n");
    for field in fields {
        // Per-field rename takes precedence over rename_all
        let ts_field_name = if let Some(ref rename) = field.serde_rename {
            rename.clone()
        } else {
            apply_serde_rename(&field.name, serde_rename_all)
        };
        ts.push_str(&format!(
            "  {}: {};\n",
            ts_field_name, field.typescript_type
        ));
    }
    ts.push('}');
    ts
}

fn find_component_mappings(content: &str) -> HashMap<String, String> {
    let mut mappings = HashMap::new();

    // Pattern: inertia_response!("Component/Path", PropsStruct { ... })
    let pattern =
        Regex::new(r#"inertia_response!\s*\(\s*"([^"]+)"\s*,\s*([A-Z][a-zA-Z0-9]*)\s*\{"#).unwrap();

    for cap in pattern.captures_iter(content) {
        if let (Some(component), Some(props)) = (cap.get(1), cap.get(2)) {
            mappings.insert(component.as_str().to_string(), props.as_str().to_string());
        }
    }

    mappings
}
