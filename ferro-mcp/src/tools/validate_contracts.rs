//! Contract validation tool - validates backend/frontend data contracts
//!
//! This tool compares:
//! - Rust InertiaProps structs (what backend sends)
//! - TypeScript Props interfaces (what frontend expects)
//!
//! And reports any mismatches.

use crate::error::{McpError, Result};
use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ContractValidationResult {
    pub total_routes: usize,
    pub validated: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub validations: Vec<RouteValidation>,
    pub summary: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RouteValidation {
    pub route: String,
    pub component: String,
    pub status: ValidationStatus,
    pub rust_props: Option<PropsInfo>,
    pub typescript_props: Option<PropsInfo>,
    pub mismatches: Vec<Mismatch>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PropsInfo {
    pub name: String,
    pub fields: Vec<PropField>,
    pub source_file: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PropField {
    pub name: String,
    pub field_type: String,
    pub optional: bool,
    /// Nested fields for complex types (objects/structs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested: Option<Vec<PropField>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Serialize)]
pub struct Mismatch {
    pub kind: MismatchKind,
    pub field: String,
    pub details: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)] // TypeMismatch reserved for type comparison
pub enum MismatchKind {
    MissingInBackend,
    MissingInFrontend,
    TypeMismatch,
    NullabilityMismatch,
    StructureMismatch,
}

pub fn execute(
    project_root: &Path,
    route_filter: Option<&str>,
) -> Result<ContractValidationResult> {
    let mut validations = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    // Parse routes and their handlers
    let routes = parse_routes_with_components(project_root)?;

    for (route, handler, component) in &routes {
        // Apply filter if provided
        if let Some(filter) = route_filter {
            if !route.contains(filter) && !component.contains(filter) {
                continue;
            }
        }

        let validation = validate_route(project_root, route, handler, component);

        match validation.status {
            ValidationStatus::Passed => passed += 1,
            ValidationStatus::Failed => failed += 1,
            ValidationStatus::Skipped => skipped += 1,
        }

        validations.push(validation);
    }

    let total_routes = validations.len();
    let validated = passed + failed;

    // Generate summary
    let mut summary = Vec::new();
    if failed > 0 {
        summary.push(format!(
            "{failed} contract(s) have mismatches that need attention"
        ));
    }
    if passed > 0 {
        summary.push(format!("{passed} contract(s) validated successfully"));
    }
    if skipped > 0 {
        summary.push(format!(
            "{skipped} route(s) skipped (no props or component found)"
        ));
    }

    Ok(ContractValidationResult {
        total_routes,
        validated,
        passed,
        failed,
        skipped,
        validations,
        summary,
    })
}

fn parse_routes_with_components(project_root: &Path) -> Result<Vec<(String, String, String)>> {
    let routes_file = project_root.join("src/routes.rs");
    if !routes_file.exists() {
        return Err(McpError::FileNotFound("src/routes.rs".to_string()));
    }

    let content = fs::read_to_string(&routes_file).map_err(McpError::IoError)?;
    let mut routes = Vec::new();

    // Pattern to match route definitions
    let route_pattern = Regex::new(
        r#"(get|post|put|patch|delete)!\s*\(\s*"([^"]+)"\s*,\s*([a-zA-Z_][a-zA-Z0-9_:]*)\s*\)"#,
    )
    .unwrap();

    for cap in route_pattern.captures_iter(&content) {
        let path = cap
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let handler = cap
            .get(3)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        // Try to find the component this handler renders
        if let Some(component) = find_component_for_handler(project_root, &handler) {
            routes.push((path, handler, component));
        }
    }

    Ok(routes)
}

fn find_component_for_handler(project_root: &Path, handler: &str) -> Option<String> {
    // Parse handler path: controllers::shelter::animals::show -> src/controllers/shelter/animals.rs
    let parts: Vec<&str> = handler.split("::").collect();
    if parts.len() < 2 {
        return None;
    }

    let file_parts: Vec<&str> = parts[..parts.len() - 1].to_vec();
    let file_path = project_root
        .join("src")
        .join(file_parts.join("/"))
        .with_extension("rs");

    if !file_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&file_path).ok()?;
    let function_name = parts.last()?;

    // Find the inertia_response! call in the handler
    // Pattern: inertia_response!("Component/Path", ...)
    let inertia_pattern = Regex::new(&format!(
        r#"fn\s+{function_name}\s*\([^)]*\)[^{{]*\{{[^}}]*inertia_response!\s*\(\s*"([^"]+)""#
    ))
    .ok()?;

    // Try simpler pattern if complex one fails
    if let Some(cap) = inertia_pattern.captures(&content) {
        return cap.get(1).map(|m| m.as_str().to_string());
    }

    // Fallback: search for any inertia_response! after the function definition
    let func_start = content.find(&format!("fn {function_name}"))?;
    let after_func = &content[func_start..];

    let simple_pattern = Regex::new(r#"inertia_response!\s*\(\s*"([^"]+)""#).ok()?;
    if let Some(cap) = simple_pattern.captures(after_func) {
        return cap.get(1).map(|m| m.as_str().to_string());
    }

    None
}

fn validate_route(
    project_root: &Path,
    route: &str,
    handler: &str,
    component: &str,
) -> RouteValidation {
    // Extract Rust props from handler
    let rust_props = extract_rust_props(project_root, handler);

    // Extract TypeScript props from component
    let typescript_props = extract_typescript_props(project_root, component);

    // If we can't find either, skip validation
    if rust_props.is_none() || typescript_props.is_none() {
        return RouteValidation {
            route: route.to_string(),
            component: component.to_string(),
            status: ValidationStatus::Skipped,
            rust_props,
            typescript_props,
            mismatches: vec![],
        };
    }

    let rust = rust_props.as_ref().unwrap();
    let ts = typescript_props.as_ref().unwrap();

    // Compare fields
    let mismatches = compare_props(rust, ts);

    let status = if mismatches.is_empty() {
        ValidationStatus::Passed
    } else {
        ValidationStatus::Failed
    };

    RouteValidation {
        route: route.to_string(),
        component: component.to_string(),
        status,
        rust_props,
        typescript_props,
        mismatches,
    }
}

fn extract_rust_props(project_root: &Path, handler: &str) -> Option<PropsInfo> {
    let parts: Vec<&str> = handler.split("::").collect();
    if parts.len() < 2 {
        return None;
    }

    let file_parts: Vec<&str> = parts[..parts.len() - 1].to_vec();
    let function_name = parts.last()?;

    let file_path = project_root
        .join("src")
        .join(file_parts.join("/"))
        .with_extension("rs");

    if !file_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&file_path).ok()?;

    // Find the handler function and extract what it returns
    let func_start = content.find(&format!("fn {function_name}"))?;
    let after_func = &content[func_start..];

    // Look for the Props struct being used in inertia_response!
    // Pattern: inertia_response!("Component", PropsStruct { ... })
    let props_pattern =
        Regex::new(r#"inertia_response!\s*\(\s*"[^"]+"\s*,\s*([A-Z][a-zA-Z0-9]*)\s*\{"#).ok()?;

    let props_name = if let Some(cap) = props_pattern.captures(after_func) {
        cap.get(1).map(|m| m.as_str().to_string())
    } else {
        None
    };

    // Try to find the Props struct definition
    if let Some(name) = &props_name {
        // Search in the same file or common props locations
        if let Some(props) = find_props_struct(&content, name, &file_path) {
            return Some(props);
        }

        // Search in props module
        let props_file = project_root.join("src/props.rs");
        if props_file.exists() {
            if let Ok(props_content) = fs::read_to_string(&props_file) {
                if let Some(props) = find_props_struct(&props_content, name, &props_file) {
                    return Some(props);
                }
            }
        }
    }

    // Fallback: extract inline struct fields from inertia_response!
    extract_inline_props(after_func, &file_path)
}

fn find_props_struct(content: &str, name: &str, source_file: &Path) -> Option<PropsInfo> {
    // Pattern to find struct definition with #[derive(InertiaProps)] or similar
    let struct_pattern = Regex::new(&format!(
        r#"(?:#\[derive\([^\)]*\)\]\s*)*struct\s+{name}\s*\{{\s*([^}}]+)\}}"#
    ))
    .ok()?;

    let cap = struct_pattern.captures(content)?;
    let fields_str = cap.get(1)?.as_str();

    let fields = parse_rust_fields(fields_str);

    Some(PropsInfo {
        name: name.to_string(),
        fields,
        source_file: source_file.to_string_lossy().to_string(),
    })
}

fn extract_inline_props(handler_code: &str, source_file: &Path) -> Option<PropsInfo> {
    // Look for struct instantiation: SomeProps { field1: value, field2: value }
    let inline_pattern = Regex::new(r#"([A-Z][a-zA-Z0-9]*)\s*\{\s*([^}]+)\}"#).ok()?;

    for cap in inline_pattern.captures_iter(handler_code) {
        let name = cap.get(1)?.as_str();
        if name.ends_with("Props") || name.ends_with("Detail") || name.ends_with("Summary") {
            let fields_str = cap.get(2)?.as_str();
            let fields = parse_inline_fields(fields_str);

            if !fields.is_empty() {
                return Some(PropsInfo {
                    name: name.to_string(),
                    fields,
                    source_file: source_file.to_string_lossy().to_string(),
                });
            }
        }
    }

    None
}

fn parse_rust_fields(fields_str: &str) -> Vec<PropField> {
    let mut fields = Vec::new();
    let field_pattern = Regex::new(r#"(?:pub\s+)?(\w+)\s*:\s*([^,\n]+)"#).unwrap();

    for cap in field_pattern.captures_iter(fields_str) {
        let name = cap
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let field_type = cap
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        // Skip if it looks like a derive attribute or comment
        if name.starts_with('#') || name.starts_with('/') {
            continue;
        }

        let optional = field_type.starts_with("Option<");

        fields.push(PropField {
            name,
            field_type,
            optional,
            nested: None,
        });
    }

    fields
}

fn parse_inline_fields(fields_str: &str) -> Vec<PropField> {
    let mut fields = Vec::new();
    let field_pattern = Regex::new(r#"(\w+)\s*:"#).unwrap();

    for cap in field_pattern.captures_iter(fields_str) {
        let name = cap
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        fields.push(PropField {
            name,
            field_type: "unknown".to_string(),
            optional: false,
            nested: None,
        });
    }

    fields
}

fn extract_typescript_props(project_root: &Path, component: &str) -> Option<PropsInfo> {
    let component_path = project_root
        .join("frontend/src/pages")
        .join(format!("{component}.tsx"));

    if !component_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&component_path).ok()?;

    // Strategy 1: Look for imported Props type usage in function signature
    // export default function ComponentName({ prop1, prop2 }: PropsType)
    let func_pattern = Regex::new(
        r#"(?:export\s+default\s+)?function\s+\w+\s*\(\s*\{\s*([^}]+)\s*\}\s*:\s*(\w+)"#,
    )
    .ok()?;

    if let Some(cap) = func_pattern.captures(&content) {
        let destructured = cap.get(1)?.as_str();
        let props_type = cap.get(2)?.as_str();

        let fields = parse_destructured_props(destructured);

        // Also try to find the interface definition
        let mut all_fields = fields;
        if let Some(interface_fields) = find_interface_fields(&content, props_type) {
            // Merge fields, preferring interface definition
            let existing_names: HashSet<_> = all_fields.iter().map(|f| f.name.clone()).collect();
            for field in interface_fields {
                if !existing_names.contains(&field.name) {
                    all_fields.push(field);
                }
            }
        }

        // Check imported types from inertia-props.ts
        if let Some(imported_fields) = find_imported_props(project_root, &content, props_type) {
            let existing_names: HashSet<_> = all_fields.iter().map(|f| f.name.clone()).collect();
            for field in imported_fields {
                if !existing_names.contains(&field.name) {
                    all_fields.push(field);
                }
            }
        }

        return Some(PropsInfo {
            name: props_type.to_string(),
            fields: all_fields,
            source_file: component_path.to_string_lossy().to_string(),
        });
    }

    // Strategy 2: Look for interface definition in the file
    let interface_pattern = Regex::new(r#"interface\s+(\w*Props\w*)\s*\{([^}]+)\}"#).ok()?;
    if let Some(cap) = interface_pattern.captures(&content) {
        let name = cap.get(1)?.as_str();
        let fields_str = cap.get(2)?.as_str();

        return Some(PropsInfo {
            name: name.to_string(),
            fields: parse_typescript_interface_fields(fields_str),
            source_file: component_path.to_string_lossy().to_string(),
        });
    }

    None
}

fn parse_destructured_props(destructured: &str) -> Vec<PropField> {
    let mut fields = Vec::new();

    for part in destructured.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Handle renamed props: originalName: newName
        // Handle default values: propName = defaultValue
        let name = part
            .split(':')
            .next()
            .unwrap_or(part)
            .split('=')
            .next()
            .unwrap_or(part)
            .trim()
            .to_string();

        if !name.is_empty() && !name.starts_with("...") {
            fields.push(PropField {
                name,
                field_type: "unknown".to_string(),
                optional: part.contains('='), // Has default value = optional
                nested: None,
            });
        }
    }

    fields
}

fn find_interface_fields(content: &str, props_type: &str) -> Option<Vec<PropField>> {
    let pattern = Regex::new(&format!(
        r#"interface\s+{}\s*(?:extends\s+[^{{]+)?\{{\s*([^}}]+)\}}"#,
        regex::escape(props_type)
    ))
    .ok()?;

    let cap = pattern.captures(content)?;
    let fields_str = cap.get(1)?.as_str();

    Some(parse_typescript_interface_fields(fields_str))
}

fn find_imported_props(
    project_root: &Path,
    content: &str,
    props_type: &str,
) -> Option<Vec<PropField>> {
    // Check if this type is imported from types/inertia-props
    let pattern_str = format!(
        r#"import\s*\{{[^}}]*\b{}\b[^}}]*\}}\s*from\s*['"]([^'"]+)['"]"#,
        regex::escape(props_type)
    );
    let import_pattern = Regex::new(&pattern_str).ok()?;

    let cap = import_pattern.captures(content)?;
    let import_path = cap.get(1)?.as_str();

    // Resolve the import path
    let types_file = if import_path.contains("inertia-props") {
        project_root.join("frontend/src/types/inertia-props.ts")
    } else if import_path.contains("shared") {
        project_root.join("frontend/src/types/shared.ts")
    } else {
        return None;
    };

    if !types_file.exists() {
        return None;
    }

    let types_content = fs::read_to_string(&types_file).ok()?;
    find_interface_fields(&types_content, props_type)
}

/// Parse TypeScript interface fields with nested structure support
fn parse_typescript_interface_fields(fields_str: &str) -> Vec<PropField> {
    parse_typescript_fields_recursive(fields_str)
}

/// Recursively parse TypeScript fields, handling nested inline objects
fn parse_typescript_fields_recursive(fields_str: &str) -> Vec<PropField> {
    let mut fields = Vec::new();
    let chars = fields_str.chars().peekable();
    let mut current_field = String::new();
    let mut brace_depth = 0;

    for c in chars {
        match c {
            '{' => {
                brace_depth += 1;
                current_field.push(c);
            }
            '}' => {
                brace_depth -= 1;
                current_field.push(c);
            }
            ';' | ',' if brace_depth == 0 => {
                if let Some(field) = parse_single_typescript_field(&current_field) {
                    fields.push(field);
                }
                current_field.clear();
            }
            '\n' if brace_depth == 0 && !current_field.trim().is_empty() => {
                // Handle newline-terminated fields (common in TS)
                if current_field.contains(':') {
                    if let Some(field) = parse_single_typescript_field(&current_field) {
                        fields.push(field);
                    }
                    current_field.clear();
                }
            }
            _ => {
                current_field.push(c);
            }
        }
    }

    // Handle last field if exists
    if !current_field.trim().is_empty() {
        if let Some(field) = parse_single_typescript_field(&current_field) {
            fields.push(field);
        }
    }

    fields
}

/// Parse a single TypeScript field declaration
fn parse_single_typescript_field(field_str: &str) -> Option<PropField> {
    let field_str = field_str.trim();
    if field_str.is_empty() {
        return None;
    }

    // Pattern: name?: { nested } or name?: Type
    let field_pattern = Regex::new(r#"^(\w+)(\?)?:\s*(.+)$"#).ok()?;

    let cap = field_pattern.captures(field_str)?;
    let name = cap.get(1)?.as_str().to_string();
    let optional = cap.get(2).is_some();
    let type_part = cap.get(3)?.as_str().trim();

    // Check if the type is an inline object (starts with { and ends with })
    let (field_type, nested) = if type_part.starts_with('{') && type_part.ends_with('}') {
        // Extract nested object fields
        let inner = &type_part[1..type_part.len() - 1];
        let nested_fields = parse_typescript_fields_recursive(inner);
        if nested_fields.is_empty() {
            ("object".to_string(), None)
        } else {
            ("object".to_string(), Some(nested_fields))
        }
    } else {
        (type_part.to_string(), None)
    };

    Some(PropField {
        name,
        field_type,
        optional,
        nested,
    })
}

/// Compare Rust and TypeScript props
fn compare_props(rust: &PropsInfo, ts: &PropsInfo) -> Vec<Mismatch> {
    compare_fields(&rust.fields, &ts.fields, "")
}

/// Recursively compare fields between Rust and TypeScript
fn compare_fields(rust_fields: &[PropField], ts_fields: &[PropField], path: &str) -> Vec<Mismatch> {
    let mut mismatches = Vec::new();

    let rust_map: HashMap<_, _> = rust_fields.iter().map(|f| (f.name.clone(), f)).collect();
    let ts_map: HashMap<_, _> = ts_fields.iter().map(|f| (f.name.clone(), f)).collect();

    // Build a map of camelCase -> original rust field names for reverse lookup
    let rust_camel_to_snake: HashMap<String, String> = rust_map
        .keys()
        .map(|name| (to_camel_case(name), name.clone()))
        .collect();

    // Check for fields in TypeScript but missing in Rust
    for (name, ts_field) in &ts_map {
        let field_path = if path.is_empty() {
            name.clone()
        } else {
            format!("{path}.{name}")
        };

        // Check if the TS field exists in Rust (direct match or snake_case version)
        let rust_field = rust_map.get(name).or_else(|| {
            rust_camel_to_snake
                .get(name)
                .and_then(|sn| rust_map.get(sn))
        });

        if let Some(rf) = rust_field {
            // Check structural mismatch: TS has nested but Rust doesn't
            if ts_field.nested.is_some() && rf.nested.is_none() {
                mismatches.push(Mismatch {
                    kind: MismatchKind::StructureMismatch,
                    field: field_path.clone(),
                    details: format!(
                        "Frontend expects '{}' to have nested properties but backend sends flat type '{}'",
                        name, rf.field_type
                    ),
                });
            } else if ts_field.nested.is_none() && rf.nested.is_some() {
                mismatches.push(Mismatch {
                    kind: MismatchKind::StructureMismatch,
                    field: field_path.clone(),
                    details: format!(
                        "Backend sends '{}' with nested structure but frontend expects flat type '{}'",
                        name, ts_field.field_type
                    ),
                });
            } else if let (Some(ts_nested), Some(rust_nested)) = (&ts_field.nested, &rf.nested) {
                // Recursively compare nested structures
                mismatches.extend(compare_fields(rust_nested, ts_nested, &field_path));
            }

            // Check nullability mismatch
            if rf.optional && !ts_field.optional && !ts_field.field_type.contains("null") {
                mismatches.push(Mismatch {
                    kind: MismatchKind::NullabilityMismatch,
                    field: field_path,
                    details: format!(
                        "Backend sends Option<{}> but frontend expects non-nullable {}",
                        rf.field_type, ts_field.field_type
                    ),
                });
            }
        } else if !ts_field.optional {
            mismatches.push(Mismatch {
                kind: MismatchKind::MissingInBackend,
                field: field_path,
                details: format!("Frontend expects '{name}' but backend doesn't send it"),
            });
        }
    }

    // Check for fields in Rust but not used in TypeScript
    for name in rust_map.keys() {
        let field_path = if path.is_empty() {
            name.clone()
        } else {
            format!("{path}.{name}")
        };

        if !ts_map.contains_key(name) {
            let camel_name = to_camel_case(name);
            if !ts_map.contains_key(&camel_name) {
                mismatches.push(Mismatch {
                    kind: MismatchKind::MissingInFrontend,
                    field: field_path,
                    details: format!(
                        "Backend sends '{name}' but frontend doesn't use it (might be intentional)"
                    ),
                });
            }
        }
    }

    mismatches
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
