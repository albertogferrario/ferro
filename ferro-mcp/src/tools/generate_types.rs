//! Generate types tool - generate TypeScript types from InertiaProps structs
//!
//! This provides direct introspection and type generation without running the CLI.

use crate::error::{McpError, Result};
use crate::tools::list_props::{self, PropsField, PropsStruct};
use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct GenerateTypesResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub message: String,
    /// Number of types generated
    pub types_generated: usize,
    /// List of generated interface names
    pub interfaces: Vec<String>,
    /// Preview of generated TypeScript (first 2000 chars)
    pub preview: Option<String>,
    /// Differences from existing file (if any)
    pub diff: Option<TypesDiff>,
}

#[derive(Debug, Serialize)]
pub struct TypesDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

pub fn execute(
    project_root: &Path,
    output_path: Option<&str>,
    dry_run: bool,
) -> Result<GenerateTypesResult> {
    let output = output_path.unwrap_or("frontend/src/types/inertia-props.ts");
    let full_output_path = project_root.join(output);

    // Get all InertiaProps structs
    let props_result = list_props::execute(project_root, None)?;

    if props_result.props.is_empty() {
        return Ok(GenerateTypesResult {
            success: true,
            output_path: Some(output.to_string()),
            message: "No InertiaProps structs found in the project".to_string(),
            types_generated: 0,
            interfaces: vec![],
            preview: None,
            diff: None,
        });
    }

    // Parse shared.ts types (to avoid regenerating)
    let shared_types = parse_shared_types(project_root);

    // Resolve nested types
    let mut all_props = props_result.props;
    let nested_types = resolve_nested_types(project_root, &all_props, &shared_types);
    all_props.extend(nested_types);

    // Sort topologically (dependencies first)
    let sorted = topological_sort(&all_props);

    // Generate self-contained TypeScript (no shared.ts imports/re-exports)
    let typescript = generate_typescript(&sorted);

    // Check for differences with existing file
    let diff = if full_output_path.exists() {
        let existing = fs::read_to_string(&full_output_path).ok();
        existing.and_then(|e| compute_diff(&e, &typescript, &sorted))
    } else {
        None
    };

    let interfaces: Vec<String> = sorted
        .iter()
        .map(|p| generate_namespaced_name(&p.module_path, &p.name))
        .collect();
    let types_generated = interfaces.len();

    // Preview (truncate if too long)
    let preview = if typescript.len() > 2000 {
        Some(format!(
            "{}...\n\n(truncated, {} total characters)",
            &typescript[..2000],
            typescript.len()
        ))
    } else {
        Some(typescript.clone())
    };

    if dry_run {
        return Ok(GenerateTypesResult {
            success: true,
            output_path: Some(output.to_string()),
            message: format!("Dry run: would generate {types_generated} interface(s)"),
            types_generated,
            interfaces,
            preview,
            diff,
        });
    }

    // Ensure output directory exists
    if let Some(parent) = full_output_path.parent() {
        fs::create_dir_all(parent).map_err(McpError::IoError)?;
    }

    // Write the file
    fs::write(&full_output_path, &typescript).map_err(McpError::IoError)?;

    Ok(GenerateTypesResult {
        success: true,
        output_path: Some(output.to_string()),
        message: format!("Generated {types_generated} interface(s) to {output}"),
        types_generated,
        interfaces,
        preview,
        diff,
    })
}

fn topological_sort(props: &[PropsStruct]) -> Vec<&PropsStruct> {
    let names: HashSet<String> = props.iter().map(|p| p.name.clone()).collect();

    // Build dependency graph using owned strings
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
    for p in props {
        let mut p_deps = HashSet::new();
        for field in &p.fields {
            // Check if this field's type is another InertiaProps struct
            let type_name = extract_base_type(&field.typescript_type);
            if names.contains(&type_name) {
                p_deps.insert(type_name);
            }
        }
        deps.insert(p.name.clone(), p_deps);
    }

    // Kahn's algorithm
    let mut in_degree: HashMap<String, usize> = names.iter().map(|n| (n.clone(), 0)).collect();
    for p_deps in deps.values() {
        for dep in p_deps {
            if let Some(count) = in_degree.get_mut(dep) {
                *count += 1;
            }
        }
    }

    let mut queue: Vec<String> = in_degree
        .iter()
        .filter(|(_, &count)| count == 0)
        .map(|(name, _)| name.clone())
        .collect();

    let mut result = Vec::new();
    let props_map: HashMap<_, _> = props.iter().map(|p| (p.name.clone(), p)).collect();

    while let Some(name) = queue.pop() {
        if let Some(p) = props_map.get(&name) {
            result.push(*p);
        }
        if let Some(p_deps) = deps.get(&name) {
            for dep in p_deps {
                if let Some(count) = in_degree.get_mut(dep) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        queue.push(dep.clone());
                    }
                }
            }
        }
    }

    result
}

fn extract_base_type(ts_type: &str) -> String {
    // Remove array suffix, null union, etc
    ts_type
        .replace("[]", "")
        .replace(" | null", "")
        .replace(" | undefined", "")
        .trim()
        .to_string()
}

/// Parse shared.ts to find exported type names
fn parse_shared_types(project_root: &Path) -> HashSet<String> {
    let shared_path = project_root.join("frontend/src/types/shared.ts");

    if !shared_path.exists() {
        return HashSet::new();
    }

    let content = match fs::read_to_string(&shared_path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };

    let mut types = HashSet::new();

    // Match: export interface Name, export type Name, export enum Name
    let patterns = [
        r"export\s+interface\s+(\w+)",
        r"export\s+type\s+(\w+)",
        r"export\s+enum\s+(\w+)",
    ];

    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.captures_iter(&content) {
                if let Some(name) = cap.get(1) {
                    types.insert(name.as_str().to_string());
                }
            }
        }
    }

    types
}

/// Generate self-contained TypeScript output (no shared.ts imports/re-exports)
fn generate_typescript(props: &[&PropsStruct]) -> String {
    // Build name map for namespacing
    let name_map = build_name_map(props);

    let mut output = String::new();
    output.push_str("// This file is auto-generated by Ferro. Do not edit manually.\n");
    output.push_str("// Run `ferro generate-types` to regenerate.\n\n");

    // Utility types (inline to avoid circular imports with shared.ts)
    output.push_str("// Utility types\n\n");
    output.push_str("/** Represents any valid JSON value */\n");
    output.push_str("export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };\n\n");
    output.push_str("/** Validation error messages keyed by field name */\n");
    output.push_str("export type ValidationErrors = Record<string, string[]>;\n\n");

    // Generate interfaces with namespaced names
    for p in props {
        let namespaced_name = generate_namespaced_name(&p.module_path, &p.name);
        let interface = generate_interface_with_mapping(
            &namespaced_name,
            &p.fields,
            p.serde_rename_all.as_deref(),
            &name_map,
        );
        output.push_str(&interface);
        output.push_str("\n\n");
    }

    output.trim_end().to_string()
}

fn compute_diff(existing: &str, _new: &str, props: &[&PropsStruct]) -> Option<TypesDiff> {
    // Extract existing interface names
    let existing_interfaces: HashSet<_> = regex::Regex::new(r"(?:export\s+)?interface\s+(\w+)")
        .ok()?
        .captures_iter(existing)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect();

    // Use namespaced names for new interfaces
    let new_interfaces: HashSet<_> = props
        .iter()
        .map(|p| generate_namespaced_name(&p.module_path, &p.name))
        .collect();

    let added: Vec<_> = new_interfaces
        .difference(&existing_interfaces)
        .cloned()
        .collect();

    let removed: Vec<_> = existing_interfaces
        .difference(&new_interfaces)
        .cloned()
        .collect();

    // Check for modifications (same name but different content)
    let mut modified = Vec::new();
    for p in props {
        let namespaced_name = generate_namespaced_name(&p.module_path, &p.name);
        if existing_interfaces.contains(&namespaced_name) {
            // Simple check: see if the interface body changed
            let existing_pattern = regex::Regex::new(&format!(
                r"(?:export\s+)?interface\s+{}\s*\{{\s*([^}}]+)\}}",
                regex::escape(&namespaced_name)
            ))
            .ok()?;

            if let Some(cap) = existing_pattern.captures(existing) {
                let existing_body = cap.get(1)?.as_str().trim();
                // Normalize for comparison
                let existing_normalized = normalize_interface_body(existing_body);
                let new_normalized = normalize_interface_body(&p.typescript);

                if existing_normalized != new_normalized {
                    modified.push(namespaced_name);
                }
            }
        }
    }

    if added.is_empty() && removed.is_empty() && modified.is_empty() {
        return None;
    }

    Some(TypesDiff {
        added,
        removed,
        modified,
    })
}

fn normalize_interface_body(body: &str) -> String {
    body.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Scan for structs with #[derive(Serialize)] matching target type names
fn scan_serialize_structs(project_root: &Path, target_types: &HashSet<String>) -> Vec<PropsStruct> {
    if target_types.is_empty() {
        return Vec::new();
    }

    let src_path = project_root.join("src");
    let mut found_structs = Vec::new();

    // Pattern to find #[derive(...Serialize...)]
    let derive_pattern = Regex::new(r#"#\[derive\([^\)]*(?:serde::)?Serialize[^\)]*\)\]"#).unwrap();
    let struct_pattern = Regex::new(r#"(?:pub\s+)?struct\s+(\w+)\s*\{"#).unwrap();
    let rename_all_pattern =
        Regex::new(r#"#\[serde\([^\)]*rename_all\s*=\s*"([^"]+)"[^\)]*\)\]"#).unwrap();
    let field_pattern = Regex::new(r#"(?:pub\s+)?(\w+)\s*:\s*([^,\n]+)"#).unwrap();
    let field_rename_pattern =
        Regex::new(r#"#\[serde\([^\)]*rename\s*=\s*"([^"]+)"[^\)]*\)\]"#).unwrap();

    for entry in WalkDir::new(&src_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        let path = entry.path();
        // Compute module path for this file
        let module_path = compute_module_path(path, &src_path);

        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines().collect();

            for (i, line) in lines.iter().enumerate() {
                if derive_pattern.is_match(line) {
                    // Look for serde(rename_all) between derive and struct
                    let mut serde_rename_all = None;
                    for check_line in lines.iter().take(std::cmp::min(i + 5, lines.len())).skip(i) {
                        if let Some(cap) = rename_all_pattern.captures(check_line) {
                            serde_rename_all = Some(cap.get(1).unwrap().as_str().to_string());
                            break;
                        }
                    }

                    // Look for struct definition in next few lines
                    for j in (i + 1)..std::cmp::min(i + 5, lines.len()) {
                        if let Some(cap) = struct_pattern.captures(lines[j]) {
                            let name = cap.get(1).unwrap().as_str().to_string();

                            // Only process if this is a target type
                            if !target_types.contains(&name) {
                                break;
                            }

                            // Extract struct body
                            if let Some(struct_body) = extract_struct_body(&lines, j) {
                                let fields = parse_struct_fields_for_serialize(
                                    &struct_body,
                                    &field_pattern,
                                    &field_rename_pattern,
                                );
                                let typescript =
                                    generate_interface(&name, &fields, serde_rename_all.as_deref());

                                let relative_path = path
                                    .strip_prefix(project_root)
                                    .unwrap_or(path)
                                    .to_string_lossy()
                                    .to_string();

                                found_structs.push(PropsStruct {
                                    name,
                                    file_path: relative_path,
                                    line_number: j + 1,
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
        }
    }

    found_structs
}

/// Compute module path from file path, stripping src/ prefix and .rs extension.
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

fn parse_struct_fields_for_serialize(
    body: &str,
    field_pattern: &Regex,
    rename_pattern: &Regex,
) -> Vec<PropsField> {
    let mut fields = Vec::new();
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

    // Handle DateTime<Tz> - chrono DateTime with timezone generic
    if ty.starts_with("DateTime<") && ty.ends_with('>') {
        return "string".to_string();
    }

    // Primitive types and special framework types
    match ty {
        "String" | "&str" | "str" => "string".to_string(),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "Value" | "serde_json::Value" => "JsonValue".to_string(),
        "ValidationErrors" | "ferro::ValidationErrors" => "ValidationErrors".to_string(),
        // Chrono datetime types serialize to ISO8601 strings
        "DateTime" | "NaiveDateTime" | "NaiveDate" | "NaiveTime" | "DateTimeUtc"
        | "DateTimeLocal" | "Date" | "Time" => "string".to_string(),
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

/// Generate a namespaced TypeScript name from module path and struct name
fn generate_namespaced_name(module_path: &str, struct_name: &str) -> String {
    if module_path.is_empty() {
        return struct_name.to_string();
    }
    let namespace: String = module_path.split("::").map(snake_to_pascal).collect();
    format!("{namespace}{struct_name}")
}

/// Build a map from original struct name to namespaced name, with collision detection
fn build_name_map(structs: &[&PropsStruct]) -> HashMap<String, String> {
    let mut name_map = HashMap::new();
    let mut reverse_map: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for s in structs {
        let namespaced = generate_namespaced_name(&s.module_path, &s.name);
        name_map.insert(s.name.clone(), namespaced.clone());
        reverse_map
            .entry(namespaced)
            .or_default()
            .push((s.name.clone(), s.module_path.clone()));
    }

    // Collision detection
    for (namespaced, sources) in &reverse_map {
        if sources.len() > 1 {
            eprintln!(
                "Warning: TypeScript name collision detected for '{namespaced}': {sources:?}"
            );
        }
    }

    name_map
}

/// Convert Rust type to TypeScript with namespaced name mapping for custom types
fn rust_type_to_ts_with_mapping(rust_type: &str, name_map: &HashMap<String, String>) -> String {
    let ty = rust_type.trim();

    // Handle Option<T>
    if ty.starts_with("Option<") && ty.ends_with('>') {
        let inner = &ty[7..ty.len() - 1];
        return format!("{} | null", rust_type_to_ts_with_mapping(inner, name_map));
    }

    // Handle Vec<T>
    if ty.starts_with("Vec<") && ty.ends_with('>') {
        let inner = &ty[4..ty.len() - 1];
        return format!("{}[]", rust_type_to_ts_with_mapping(inner, name_map));
    }

    // Handle HashMap<K, V>
    if (ty.starts_with("HashMap<") || ty.starts_with("BTreeMap<")) && ty.ends_with('>') {
        let start = if ty.starts_with("HashMap<") { 8 } else { 9 };
        let inner = &ty[start..ty.len() - 1];
        if let Some(comma_pos) = inner.find(',') {
            let key = inner[..comma_pos].trim();
            let val = inner[comma_pos + 1..].trim();
            return format!(
                "Record<{}, {}>",
                rust_type_to_ts_with_mapping(key, name_map),
                rust_type_to_ts_with_mapping(val, name_map)
            );
        }
    }

    // Primitive types and special framework types
    match ty {
        "String" | "&str" | "str" => "string".to_string(),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "Value" | "serde_json::Value" => "JsonValue".to_string(),
        "ValidationErrors" | "ferro::ValidationErrors" => "ValidationErrors".to_string(),
        // Check if it's a known custom type that needs namespacing
        _ => name_map.get(ty).cloned().unwrap_or_else(|| ty.to_string()),
    }
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

fn generate_interface(name: &str, fields: &[PropsField], serde_rename_all: Option<&str>) -> String {
    let mut ts = format!("export interface {name} {{\n");
    for field in fields {
        let optional_marker = if field.optional { "?" } else { "" };
        // Per-field rename takes precedence over rename_all
        let ts_field_name = if let Some(ref rename) = field.serde_rename {
            rename.clone()
        } else {
            apply_serde_rename(&field.name, serde_rename_all)
        };
        ts.push_str(&format!(
            "  {}{}: {};\n",
            ts_field_name, optional_marker, field.typescript_type
        ));
    }
    ts.push('}');
    ts
}

/// Generate interface with namespaced type references
fn generate_interface_with_mapping(
    name: &str,
    fields: &[PropsField],
    serde_rename_all: Option<&str>,
    name_map: &HashMap<String, String>,
) -> String {
    let mut ts = format!("export interface {name} {{\n");
    for field in fields {
        let optional_marker = if field.optional { "?" } else { "" };
        // Per-field rename takes precedence over rename_all
        let ts_field_name = if let Some(ref rename) = field.serde_rename {
            rename.clone()
        } else {
            apply_serde_rename(&field.name, serde_rename_all)
        };
        // Use namespaced type references
        let ts_type = rust_type_to_ts_with_mapping(&field.rust_type, name_map);
        ts.push_str(&format!("  {ts_field_name}{optional_marker}: {ts_type};\n"));
    }
    ts.push('}');
    ts
}

/// Collect custom type names referenced in props fields
fn collect_nested_types(props: &[PropsStruct]) -> HashSet<String> {
    let mut types = HashSet::new();
    let type_pattern = Regex::new(r"\b([A-Z][a-zA-Z0-9]+)\b").unwrap();

    for p in props {
        for field in &p.fields {
            for cap in type_pattern.captures_iter(&field.typescript_type) {
                if let Some(name) = cap.get(1) {
                    let type_name = name.as_str();
                    // Filter out TypeScript built-in types
                    if !matches!(
                        type_name,
                        "Record" | "Array" | "Partial" | "Required" | "Readonly" | "Map" | "Set"
                    ) {
                        types.insert(type_name.to_string());
                    }
                }
            }
        }
    }

    types
}

/// Resolve all nested types referenced by InertiaProps structs
fn resolve_nested_types(
    project_root: &Path,
    initial_props: &[PropsStruct],
    shared_types: &HashSet<String>,
) -> Vec<PropsStruct> {
    let mut known_types: HashSet<String> = initial_props.iter().map(|p| p.name.clone()).collect();
    let mut all_nested = Vec::new();
    let mut types_to_find = collect_nested_types(initial_props);

    // Filter out types we already know about
    types_to_find.retain(|t| !known_types.contains(t) && !shared_types.contains(t));

    // Fixed-point iteration
    while !types_to_find.is_empty() {
        let found = scan_serialize_structs(project_root, &types_to_find);

        if found.is_empty() {
            break;
        }

        // Mark found types as known
        for s in &found {
            known_types.insert(s.name.clone());
        }

        // Collect types referenced by newly found structs
        let mut next_types = collect_nested_types(&found);
        next_types.retain(|t| !known_types.contains(t) && !shared_types.contains(t));

        all_nested.extend(found);
        types_to_find = next_types;
    }

    all_nested
}
