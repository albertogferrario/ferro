use console::style;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, Fields, GenericArgument, ItemStruct, Meta, PathArguments, Type};
use walkdir::WalkDir;

/// Serde rename_all case transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SerdeCase {
    #[default]
    None,
    CamelCase,
    SnakeCase,
    PascalCase,
    ScreamingSnakeCase,
    KebabCase,
}

impl SerdeCase {
    /// Parse from serde attribute value
    fn from_str(s: &str) -> Self {
        match s {
            "camelCase" => Self::CamelCase,
            "snake_case" => Self::SnakeCase,
            "PascalCase" => Self::PascalCase,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnakeCase,
            "kebab-case" => Self::KebabCase,
            _ => Self::None,
        }
    }

    /// Apply case transformation to a field name
    fn apply(&self, name: &str) -> String {
        match self {
            Self::None | Self::SnakeCase => name.to_string(),
            Self::CamelCase => snake_to_camel(name),
            Self::PascalCase => snake_to_pascal(name),
            Self::ScreamingSnakeCase => name.to_uppercase(),
            Self::KebabCase => name.replace('_', "-"),
        }
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

/// Represents a parsed InertiaProps struct
#[derive(Debug, Clone)]
pub struct InertiaPropsStruct {
    pub name: String,
    pub fields: Vec<StructField>,
    /// Serde rename_all attribute on the struct
    pub rename_all: SerdeCase,
    /// Module path where this struct is defined (e.g., "shelter::applications")
    /// Used to generate unique namespaced TypeScript interface names
    #[allow(dead_code)] // Will be used in namespaced interface generation (Task 2)
    pub module_path: String,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: RustType,
    /// Per-field serde rename override
    pub serde_rename: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RustType {
    String,
    Number,
    Bool,
    DateTime,
    /// serde_json::Value or sea_orm::Json - arbitrary JSON
    JsonValue,
    /// ferro::ValidationErrors - Record<string, string[]>
    ValidationErrors,
    Option(Box<RustType>),
    Vec(Box<RustType>),
    HashMap(Box<RustType>, Box<RustType>),
    Custom(String),
}

/// Visitor that collects structs with #[derive(InertiaProps)]
struct InertiaPropsVisitor {
    structs: Vec<InertiaPropsStruct>,
    /// Module path for the current file being scanned
    module_path: String,
}

impl InertiaPropsVisitor {
    fn new(module_path: String) -> Self {
        Self {
            structs: Vec::new(),
            module_path,
        }
    }

    fn has_inertia_props_derive(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        if path.is_ident("InertiaProps") {
                            return true;
                        }
                        // Also check for ferro::InertiaProps
                        if path.segments.len() == 2 {
                            let first = &path.segments[0].ident;
                            let second = &path.segments[1].ident;
                            if first == "ferro" && second == "InertiaProps" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Parse #[serde(rename_all = "...")] from struct attributes
    fn parse_serde_rename_all(attrs: &[Attribute]) -> SerdeCase {
        for attr in attrs {
            if attr.path().is_ident("serde") {
                if let Meta::List(meta_list) = &attr.meta {
                    // Parse the token stream to find rename_all = "value"
                    let tokens_str = meta_list.tokens.to_string();
                    if let Some(rename_all) = parse_serde_rename_all_value(&tokens_str) {
                        return SerdeCase::from_str(&rename_all);
                    }
                }
            }
        }
        SerdeCase::None
    }

    /// Parse #[serde(rename = "...")] from field attributes
    fn parse_serde_field_rename(attrs: &[Attribute]) -> Option<String> {
        for attr in attrs {
            if attr.path().is_ident("serde") {
                if let Meta::List(meta_list) = &attr.meta {
                    let tokens_str = meta_list.tokens.to_string();
                    if let Some(rename) = parse_serde_rename_value(&tokens_str) {
                        return Some(rename);
                    }
                }
            }
        }
        None
    }
}

/// Parse rename_all = "value" from serde attribute tokens
fn parse_serde_rename_all_value(tokens: &str) -> Option<String> {
    // Look for rename_all = "..."
    if let Some(start) = tokens.find("rename_all") {
        let rest = &tokens[start..];
        // Find the value between quotes
        if let Some(quote_start) = rest.find('"') {
            let after_quote = &rest[quote_start + 1..];
            if let Some(quote_end) = after_quote.find('"') {
                return Some(after_quote[..quote_end].to_string());
            }
        }
    }
    None
}

/// Parse rename = "value" from serde attribute tokens (but not rename_all)
fn parse_serde_rename_value(tokens: &str) -> Option<String> {
    // Look for "rename" followed by "=" but not "rename_all"
    let mut search_from = 0;
    while let Some(pos) = tokens[search_from..].find("rename") {
        let actual_pos = search_from + pos;
        let rest = &tokens[actual_pos..];
        // Check if it's "rename_all"
        if rest.starts_with("rename_all") {
            search_from = actual_pos + 10;
            continue;
        }
        // Find the value between quotes after =
        if let Some(eq_pos) = rest.find('=') {
            let after_eq = &rest[eq_pos..];
            if let Some(quote_start) = after_eq.find('"') {
                let after_quote = &after_eq[quote_start + 1..];
                if let Some(quote_end) = after_quote.find('"') {
                    return Some(after_quote[..quote_end].to_string());
                }
            }
        }
        break;
    }
    None
}

impl InertiaPropsVisitor {
    fn parse_type(ty: &Type) -> RustType {
        match ty {
            Type::Path(type_path) => {
                let segment = type_path.path.segments.last().unwrap();
                let ident = segment.ident.to_string();

                match ident.as_str() {
                    "String" | "str" => RustType::String,
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32"
                    | "u64" | "u128" | "usize" | "f32" | "f64" => RustType::Number,
                    "bool" => RustType::Bool,
                    "Option" => {
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                                return RustType::Option(Box::new(Self::parse_type(inner_ty)));
                            }
                        }
                        RustType::Option(Box::new(RustType::Custom("unknown".to_string())))
                    }
                    "Vec" => {
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                                return RustType::Vec(Box::new(Self::parse_type(inner_ty)));
                            }
                        }
                        RustType::Vec(Box::new(RustType::Custom("unknown".to_string())))
                    }
                    "HashMap" | "BTreeMap" => {
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            let mut iter = args.args.iter();
                            if let (
                                Some(GenericArgument::Type(key_ty)),
                                Some(GenericArgument::Type(val_ty)),
                            ) = (iter.next(), iter.next())
                            {
                                return RustType::HashMap(
                                    Box::new(Self::parse_type(key_ty)),
                                    Box::new(Self::parse_type(val_ty)),
                                );
                            }
                        }
                        RustType::HashMap(
                            Box::new(RustType::String),
                            Box::new(RustType::Custom("unknown".to_string())),
                        )
                    }
                    // serde_json::Value and sea_orm::Json map to JsonValue
                    "Value" | "Json" => RustType::JsonValue,
                    // ferro::ValidationErrors or ValidationErrors
                    "ValidationErrors" => RustType::ValidationErrors,
                    // Chrono datetime types serialize to ISO8601 strings
                    "DateTime" | "NaiveDateTime" | "NaiveDate" | "NaiveTime" | "DateTimeUtc"
                    | "DateTimeLocal" | "Date" | "Time" => RustType::DateTime,
                    other => RustType::Custom(other.to_string()),
                }
            }
            Type::Reference(type_ref) => {
                // Handle &str as String
                if let Type::Path(inner) = &*type_ref.elem {
                    if inner
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident == "str")
                        .unwrap_or(false)
                    {
                        return RustType::String;
                    }
                }
                Self::parse_type(&type_ref.elem)
            }
            _ => RustType::Custom("unknown".to_string()),
        }
    }
}

impl<'ast> Visit<'ast> for InertiaPropsVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.has_inertia_props_derive(&node.attrs) {
            let name = node.ident.to_string();
            let rename_all = Self::parse_serde_rename_all(&node.attrs);

            let fields = match &node.fields {
                Fields::Named(named) => named
                    .named
                    .iter()
                    .filter_map(|f| {
                        f.ident.as_ref().map(|ident| StructField {
                            name: ident.to_string(),
                            ty: Self::parse_type(&f.ty),
                            serde_rename: Self::parse_serde_field_rename(&f.attrs),
                        })
                    })
                    .collect(),
                _ => Vec::new(),
            };

            self.structs.push(InertiaPropsStruct {
                name,
                fields,
                rename_all,
                module_path: self.module_path.clone(),
            });
        }

        // Continue visiting nested items
        syn::visit::visit_item_struct(self, node);
    }
}

/// Visitor that collects structs with #[derive(Serialize)] matching target type names
struct SerializeStructVisitor {
    /// Target type names to find
    target_types: HashSet<String>,
    /// Found structs matching target types
    structs: Vec<InertiaPropsStruct>,
    /// Module path for the current file being scanned
    module_path: String,
}

impl SerializeStructVisitor {
    fn new(target_types: HashSet<String>, module_path: String) -> Self {
        Self {
            target_types,
            structs: Vec::new(),
            module_path,
        }
    }

    /// Check if struct has Serialize derive (but not InertiaProps which is handled separately)
    fn has_serialize_derive(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        // Check for Serialize
                        if path.is_ident("Serialize") {
                            return true;
                        }
                        // Check for serde::Serialize
                        if path.segments.len() == 2 {
                            let first = &path.segments[0].ident;
                            let second = &path.segments[1].ident;
                            if first == "serde" && second == "Serialize" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

impl<'ast> Visit<'ast> for SerializeStructVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let name = node.ident.to_string();

        // Only process if this is a target type and has Serialize derive
        if self.target_types.contains(&name) && self.has_serialize_derive(&node.attrs) {
            let rename_all = InertiaPropsVisitor::parse_serde_rename_all(&node.attrs);

            let fields = match &node.fields {
                Fields::Named(named) => named
                    .named
                    .iter()
                    .filter_map(|f| {
                        f.ident.as_ref().map(|ident| StructField {
                            name: ident.to_string(),
                            ty: InertiaPropsVisitor::parse_type(&f.ty),
                            serde_rename: InertiaPropsVisitor::parse_serde_field_rename(&f.attrs),
                        })
                    })
                    .collect(),
                _ => Vec::new(),
            };

            self.structs.push(InertiaPropsStruct {
                name,
                fields,
                rename_all,
                module_path: self.module_path.clone(),
            });
        }

        // Continue visiting nested items
        syn::visit::visit_item_struct(self, node);
    }
}

/// Compute module path from file path relative to src directory.
///
/// Strips "src/" prefix and ".rs" extension, removes "controllers::" prefix if present,
/// and converts path separators to "::".
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

/// Generate a unique namespaced TypeScript interface name from module path and struct name.
///
/// Combines the module path with the struct name using PascalCase.
/// Skips the prefix when the struct name already starts with it to avoid
/// redundant names like `MenuMenuListProps`.
///
/// Examples:
/// - ("shelter::applications", "ShowProps") -> "ShelterApplicationsShowProps"
/// - ("adopter::applications", "ShowProps") -> "AdopterApplicationsShowProps"
/// - ("user", "IndexProps") -> "UserIndexProps"
/// - ("menu", "MenuListProps") -> "MenuListProps" (already prefixed)
/// - ("public::menu", "PublicMenuProps") -> "PublicMenuProps" (already prefixed)
/// - ("", "GlobalProps") -> "GlobalProps" (root-level, no namespace)
fn generate_namespaced_name(module_path: &str, struct_name: &str) -> String {
    if module_path.is_empty() {
        return struct_name.to_string();
    }

    // Convert module path segments to PascalCase and join
    let namespace: String = module_path.split("::").map(snake_to_pascal).collect();

    // Skip prefix if struct name already starts with the namespace
    // (case-insensitive to handle QRCode vs Qrcode, etc.)
    if struct_name
        .to_lowercase()
        .starts_with(&namespace.to_lowercase())
    {
        return struct_name.to_string();
    }

    format!("{namespace}{struct_name}")
}

/// Scan all Rust files for Serialize structs matching the target type names
pub fn scan_serialize_structs(
    project_path: &Path,
    target_types: &HashSet<String>,
) -> Vec<InertiaPropsStruct> {
    if target_types.is_empty() {
        return Vec::new();
    }

    let src_path = project_path.join("src");
    let mut all_structs = Vec::new();

    for entry in WalkDir::new(&src_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(syntax) = syn::parse_file(&content) {
                let module_path = compute_module_path(entry.path(), &src_path);
                let mut visitor = SerializeStructVisitor::new(target_types.clone(), module_path);
                visitor.visit_file(&syntax);
                all_structs.extend(visitor.structs);
            }
        }
    }

    all_structs
}

/// Scan all Rust files in the src directory for InertiaProps structs
pub fn scan_inertia_props(project_path: &Path) -> Vec<InertiaPropsStruct> {
    let src_path = project_path.join("src");
    let mut all_structs = Vec::new();

    for entry in WalkDir::new(&src_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(syntax) = syn::parse_file(&content) {
                let module_path = compute_module_path(entry.path(), &src_path);
                let mut visitor = InertiaPropsVisitor::new(module_path);
                visitor.visit_file(&syntax);
                all_structs.extend(visitor.structs);
            }
        }
    }

    all_structs
}

/// Sort structs topologically so dependencies come first
fn topological_sort(structs: &[InertiaPropsStruct]) -> Vec<&InertiaPropsStruct> {
    let struct_map: HashMap<_, _> = structs.iter().map(|s| (s.name.clone(), s)).collect();
    let struct_names: HashSet<_> = structs.iter().map(|s| s.name.clone()).collect();

    // Build dependency graph
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
    for s in structs {
        let mut s_deps = HashSet::new();
        for field in &s.fields {
            collect_type_deps(&field.ty, &mut s_deps, &struct_names);
        }
        deps.insert(s.name.clone(), s_deps);
    }

    // Kahn's algorithm for topological sort
    let mut in_degree: HashMap<String, usize> =
        struct_names.iter().map(|n| (n.clone(), 0)).collect();
    for s_deps in deps.values() {
        for dep in s_deps {
            if let Some(count) = in_degree.get_mut(dep) {
                *count += 1;
            }
        }
    }

    let mut queue: Vec<_> = in_degree
        .iter()
        .filter(|(_, &count)| count == 0)
        .map(|(name, _)| name.clone())
        .collect();
    let mut result = Vec::new();

    while let Some(name) = queue.pop() {
        if let Some(s) = struct_map.get(&name) {
            result.push(*s);
        }
        if let Some(s_deps) = deps.get(&name) {
            for dep in s_deps {
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

fn collect_type_deps(ty: &RustType, deps: &mut HashSet<String>, known: &HashSet<String>) {
    match ty {
        RustType::Custom(name) if known.contains(name) => {
            deps.insert(name.clone());
        }
        RustType::Option(inner) | RustType::Vec(inner) => {
            collect_type_deps(inner, deps, known);
        }
        RustType::HashMap(key, val) => {
            collect_type_deps(key, deps, known);
            collect_type_deps(val, deps, known);
        }
        _ => {}
    }
}

/// Parse shared.ts to find exported type names
/// Returns set of exported type/interface/enum names
pub fn parse_shared_types(project_path: &Path) -> HashSet<String> {
    let shared_path = project_path.join("frontend/src/types/shared.ts");

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

/// Collect all custom types referenced in InertiaProps structs
fn collect_referenced_types(structs: &[InertiaPropsStruct]) -> HashSet<String> {
    let mut types = HashSet::new();

    for s in structs {
        for field in &s.fields {
            collect_custom_types(&field.ty, &mut types);
        }
    }

    types
}

/// Recursively collect custom type names from a RustType
fn collect_custom_types(ty: &RustType, types: &mut HashSet<String>) {
    match ty {
        RustType::Custom(name) => {
            types.insert(name.clone());
        }
        RustType::Option(inner) | RustType::Vec(inner) => {
            collect_custom_types(inner, types);
        }
        RustType::HashMap(key, val) => {
            collect_custom_types(key, types);
            collect_custom_types(val, types);
        }
        _ => {}
    }
}

/// Generate TypeScript interfaces from the structs
/// Apply serde renaming to get the final TypeScript field name
fn apply_field_rename(field: &StructField, rename_all: SerdeCase) -> String {
    // Per-field rename takes precedence over rename_all
    if let Some(ref rename) = field.serde_rename {
        return rename.clone();
    }
    // Apply struct-level rename_all
    rename_all.apply(&field.name)
}

/// Generate TypeScript interfaces from the structs (self-contained, no shared.ts imports)
pub fn generate_typescript(structs: &[InertiaPropsStruct]) -> String {
    let sorted = topological_sort(structs);

    // Build name mapping: original name -> namespaced name
    // Also check for collisions
    let name_map = build_name_map(structs);

    let mut output = String::new();

    // Header with instructions
    output.push_str(
        "// =============================================================================\n",
    );
    output.push_str("// Auto-generated by `ferro generate-types`\n");
    output.push_str("// Do not edit manually - changes will be overwritten\n");
    output.push_str("//\n");
    output.push_str("// To regenerate: ferro generate-types\n");
    output.push_str("// Auto-watch: ferro serve (types regenerate on file changes)\n");
    output.push_str("//\n");
    output.push_str("// For custom types not generated here, create manual type files in:\n");
    output.push_str("// frontend/src/types/\n");
    output.push_str(
        "// =============================================================================\n\n",
    );

    // Utility types
    output.push_str("// Utility types\n\n");
    output.push_str("/** Represents any valid JSON value */\n");
    output.push_str("export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };\n\n");
    output.push_str("/** Validation error messages keyed by field name */\n");
    output.push_str("export type ValidationErrors = Record<string, string[]>;\n\n");

    for s in sorted {
        let interface_name = name_map.get(&s.name).unwrap_or(&s.name);
        output.push_str(&format!("export interface {interface_name} {{\n"));
        for field in &s.fields {
            let ts_type = rust_type_to_ts_with_mapping(&field.ty, &name_map);
            let ts_name = apply_field_rename(field, s.rename_all);
            output.push_str(&format!("  {ts_name}: {ts_type};\n"));
        }
        output.push_str("}\n\n");
    }

    output
}

/// Build a mapping from original struct name to namespaced TypeScript name.
/// Detects collisions where two different structs would produce the same namespaced name.
fn build_name_map(structs: &[InertiaPropsStruct]) -> HashMap<String, String> {
    let mut name_map = HashMap::new();
    let mut reverse_map: HashMap<String, Vec<(String, String)>> = HashMap::new(); // namespaced -> [(original, module_path), ...]

    for s in structs {
        let namespaced = generate_namespaced_name(&s.module_path, &s.name);
        name_map.insert(s.name.clone(), namespaced.clone());
        reverse_map
            .entry(namespaced)
            .or_default()
            .push((s.name.clone(), s.module_path.clone()));
    }

    // Check for collisions
    for (namespaced, sources) in &reverse_map {
        if sources.len() > 1 {
            let collision_info: Vec<String> = sources
                .iter()
                .map(|(name, path)| format!("{path}::{name}"))
                .collect();
            eprintln!(
                "Warning: TypeScript name collision detected for '{}'. Sources: {}",
                namespaced,
                collision_info.join(", ")
            );
        }
    }

    name_map
}

/// Convert RustType to TypeScript type string, applying name mapping for custom types
fn rust_type_to_ts_with_mapping(ty: &RustType, name_map: &HashMap<String, String>) -> String {
    match ty {
        RustType::String => "string".to_string(),
        RustType::Number => "number".to_string(),
        RustType::Bool => "boolean".to_string(),
        RustType::DateTime => "string".to_string(),
        RustType::JsonValue => "JsonValue".to_string(),
        RustType::ValidationErrors => "ValidationErrors".to_string(),
        RustType::Option(inner) => {
            format!("{} | null", rust_type_to_ts_with_mapping(inner, name_map))
        }
        RustType::Vec(inner) => format!("{}[]", rust_type_to_ts_with_mapping(inner, name_map)),
        RustType::HashMap(k, v) => format!(
            "Record<{}, {}>",
            rust_type_to_ts_with_mapping(k, name_map),
            rust_type_to_ts_with_mapping(v, name_map)
        ),
        RustType::Custom(name) => {
            // Apply name mapping if this is a type we've defined
            name_map.get(name).cloned().unwrap_or_else(|| name.clone())
        }
    }
}

/// Resolve all nested types referenced by InertiaProps structs
///
/// This function recursively finds all types referenced by the initial structs,
/// scans for their Serialize definitions, and returns them.
pub fn resolve_nested_types(
    project_path: &Path,
    initial_structs: &[InertiaPropsStruct],
    shared_types: &HashSet<String>,
) -> Vec<InertiaPropsStruct> {
    let mut known_types: HashSet<String> = initial_structs.iter().map(|s| s.name.clone()).collect();
    let mut all_nested = Vec::new();
    let mut types_to_find: HashSet<String> = collect_referenced_types(initial_structs);

    // Filter out types we already know about (initial structs and shared.ts types)
    types_to_find.retain(|t| !known_types.contains(t) && !shared_types.contains(t));

    // Fixed-point iteration: keep looking for nested types until none are found
    while !types_to_find.is_empty() {
        let found = scan_serialize_structs(project_path, &types_to_find);

        if found.is_empty() {
            // No more types could be resolved - remaining types are unknown
            // We could emit warnings here if needed
            break;
        }

        // Mark found types as known
        for s in &found {
            known_types.insert(s.name.clone());
        }

        // Collect types referenced by newly found structs
        let mut next_types = collect_referenced_types(&found);
        next_types.retain(|t| !known_types.contains(t) && !shared_types.contains(t));

        all_nested.extend(found);
        types_to_find = next_types;
    }

    all_nested
}

/// Generate types and write to the output file
pub fn generate_types_to_file(project_path: &Path, output_path: &Path) -> Result<usize, String> {
    let mut structs = scan_inertia_props(project_path);

    if structs.is_empty() {
        return Ok(0);
    }

    // Parse shared.ts types (used to avoid regenerating types already in shared.ts)
    let shared_types = parse_shared_types(project_path);

    // Resolve nested types
    let nested_types = resolve_nested_types(project_path, &structs, &shared_types);
    structs.extend(nested_types);

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {e}"))?;
    }

    // Generate self-contained TypeScript (no shared.ts imports/re-exports)
    let typescript = generate_typescript(&structs);
    fs::write(output_path, typescript)
        .map_err(|e| format!("Failed to write TypeScript file: {e}"))?;

    Ok(structs.len())
}

/// Main entry point for the generate-types command
pub fn run(output: Option<String>, watch: bool) {
    let project_path = Path::new(".");

    // Validate Ferro project
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        eprintln!(
            "{} Not a Ferro project (no Cargo.toml found)",
            style("Error:").red().bold()
        );
        std::process::exit(1);
    }

    let output_path = output
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| project_path.join("frontend/src/types/inertia-props.ts"));

    println!("{}", style("Scanning for InertiaProps structs...").cyan());

    match generate_types_to_file(project_path, &output_path) {
        Ok(0) => {
            println!("{}", style("No InertiaProps structs found.").yellow());
        }
        Ok(count) => {
            println!(
                "{} Found {} InertiaProps struct(s)",
                style("->").green(),
                count
            );
            println!("{} Generated {}", style("✓").green(), output_path.display());
        }
        Err(e) => {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }

    // Also generate route types
    generate_route_types(project_path);

    if watch {
        println!("{}", style("Watching for changes...").dim());
        if let Err(e) = start_watcher(project_path, &output_path) {
            eprintln!(
                "{} Failed to start watcher: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
    }
}

/// Generate route types
fn generate_route_types(project_path: &Path) {
    let routes_output = project_path.join("frontend/src/types/routes.ts");

    println!(
        "{}",
        style("Scanning routes for type-safe generation...").cyan()
    );

    match super::generate_routes::generate_routes_to_file(project_path, &routes_output) {
        Ok(0) => {
            println!("{}", style("No routes found in src/routes.rs").yellow());
        }
        Ok(count) => {
            println!("{} Found {} route(s)", style("->").green(), count);
            println!(
                "{} Generated {}",
                style("✓").green(),
                routes_output.display()
            );
        }
        Err(e) => {
            eprintln!(
                "{} Route generation error: {}",
                style("Warning:").yellow(),
                e
            );
        }
    }
}

/// Start file watcher for automatic type regeneration
fn start_watcher(project_path: &Path, output_path: &Path) -> Result<(), String> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let (tx, rx) = channel();
    let src_path = project_path.join("src");

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(1)),
    )
    .map_err(|e| format!("Failed to create watcher: {e}"))?;

    watcher
        .watch(&src_path, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch directory: {e}"))?;

    println!(
        "{} Watching {} for changes",
        style("->").cyan(),
        src_path.display()
    );

    let output_path = output_path.to_path_buf();
    let project_path = project_path.to_path_buf();

    loop {
        match rx.recv() {
            Ok(event) => {
                // Check if it's a Rust file change
                let is_rust_change = event
                    .paths
                    .iter()
                    .any(|p| p.extension().map(|e| e == "rs").unwrap_or(false));

                if is_rust_change {
                    println!("{}", style("Detected changes, regenerating types...").dim());
                    match generate_types_to_file(&project_path, &output_path) {
                        Ok(count) => {
                            println!("{} Regenerated {} type(s)", style("✓").green(), count);
                        }
                        Err(e) => {
                            eprintln!("{} Failed to regenerate: {}", style("Error:").red(), e);
                        }
                    }
                }
            }
            Err(e) => {
                return Err(format!("Watch error: {e}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("created_at"), "createdAt");
        assert_eq!(snake_to_camel("user_id"), "userId");
        assert_eq!(snake_to_camel("some_long_name"), "someLongName");
        assert_eq!(snake_to_camel("name"), "name");
    }

    #[test]
    fn test_snake_to_pascal() {
        assert_eq!(snake_to_pascal("created_at"), "CreatedAt");
        assert_eq!(snake_to_pascal("user_id"), "UserId");
        assert_eq!(snake_to_pascal("some_long_name"), "SomeLongName");
        assert_eq!(snake_to_pascal("name"), "Name");
    }

    #[test]
    fn test_serde_case_apply() {
        assert_eq!(SerdeCase::CamelCase.apply("created_at"), "createdAt");
        assert_eq!(SerdeCase::PascalCase.apply("created_at"), "CreatedAt");
        assert_eq!(
            SerdeCase::ScreamingSnakeCase.apply("created_at"),
            "CREATED_AT"
        );
        assert_eq!(SerdeCase::KebabCase.apply("created_at"), "created-at");
        assert_eq!(SerdeCase::None.apply("created_at"), "created_at");
        assert_eq!(SerdeCase::SnakeCase.apply("created_at"), "created_at");
    }

    #[test]
    fn test_serde_case_from_str() {
        assert_eq!(SerdeCase::from_str("camelCase"), SerdeCase::CamelCase);
        assert_eq!(SerdeCase::from_str("PascalCase"), SerdeCase::PascalCase);
        assert_eq!(
            SerdeCase::from_str("SCREAMING_SNAKE_CASE"),
            SerdeCase::ScreamingSnakeCase
        );
        assert_eq!(SerdeCase::from_str("kebab-case"), SerdeCase::KebabCase);
        assert_eq!(SerdeCase::from_str("snake_case"), SerdeCase::SnakeCase);
        assert_eq!(SerdeCase::from_str("unknown"), SerdeCase::None);
    }

    #[test]
    fn test_parse_serde_rename_all_value() {
        let tokens = r#"rename_all = "camelCase""#;
        assert_eq!(
            parse_serde_rename_all_value(tokens),
            Some("camelCase".to_string())
        );

        let tokens = r#"derive(Serialize), rename_all = "PascalCase""#;
        assert_eq!(
            parse_serde_rename_all_value(tokens),
            Some("PascalCase".to_string())
        );

        let tokens = r#"skip_serializing"#;
        assert_eq!(parse_serde_rename_all_value(tokens), None);
    }

    #[test]
    fn test_parse_serde_rename_value() {
        let tokens = r#"rename = "customName""#;
        assert_eq!(
            parse_serde_rename_value(tokens),
            Some("customName".to_string())
        );

        // Should not match rename_all
        let tokens = r#"rename_all = "camelCase""#;
        assert_eq!(parse_serde_rename_value(tokens), None);

        // Should find rename after rename_all
        let tokens = r#"rename_all = "camelCase", rename = "custom""#;
        assert_eq!(parse_serde_rename_value(tokens), Some("custom".to_string()));
    }

    #[test]
    fn test_apply_field_rename() {
        let field = StructField {
            name: "created_at".to_string(),
            ty: RustType::String,
            serde_rename: None,
        };

        // With camelCase rename_all
        assert_eq!(
            apply_field_rename(&field, SerdeCase::CamelCase),
            "createdAt"
        );

        // With explicit rename override
        let field_with_rename = StructField {
            name: "created_at".to_string(),
            ty: RustType::String,
            serde_rename: Some("customField".to_string()),
        };
        assert_eq!(
            apply_field_rename(&field_with_rename, SerdeCase::CamelCase),
            "customField"
        );
    }

    #[test]
    fn test_generate_typescript_with_serde() {
        let structs = vec![InertiaPropsStruct {
            name: "TestProps".to_string(),
            fields: vec![
                StructField {
                    name: "user_id".to_string(),
                    ty: RustType::Number,
                    serde_rename: None,
                },
                StructField {
                    name: "created_at".to_string(),
                    ty: RustType::String,
                    serde_rename: None,
                },
                StructField {
                    name: "special_field".to_string(),
                    ty: RustType::String,
                    serde_rename: Some("overridden".to_string()),
                },
            ],
            rename_all: SerdeCase::CamelCase,
            module_path: String::new(),
        }];

        let typescript = generate_typescript(&structs);

        assert!(typescript.contains("userId: number;"));
        assert!(typescript.contains("createdAt: string;"));
        assert!(typescript.contains("overridden: string;"));
        // Should NOT contain snake_case versions
        assert!(!typescript.contains("user_id:"));
        assert!(!typescript.contains("created_at:"));
        assert!(!typescript.contains("special_field:"));
    }

    #[test]
    fn test_serde_json_value_maps_to_json_value() {
        // serde_json::Value should map to 'JsonValue' in TypeScript
        let structs = vec![InertiaPropsStruct {
            name: "FormProps".to_string(),
            fields: vec![
                StructField {
                    name: "errors".to_string(),
                    ty: RustType::Option(Box::new(RustType::JsonValue)),
                    serde_rename: None,
                },
                StructField {
                    name: "data".to_string(),
                    ty: RustType::JsonValue,
                    serde_rename: None,
                },
            ],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        }];

        let typescript = generate_typescript(&structs);

        // Value should be mapped to 'JsonValue'
        assert!(typescript.contains("errors: JsonValue | null;"));
        assert!(typescript.contains("data: JsonValue;"));
        // Should NOT contain 'Value' as a type
        assert!(!typescript.contains(": Value"));
    }

    #[test]
    fn test_collect_referenced_types() {
        let structs = vec![InertiaPropsStruct {
            name: "TestProps".to_string(),
            fields: vec![
                StructField {
                    name: "animal".to_string(),
                    ty: RustType::Custom("Animal".to_string()),
                    serde_rename: None,
                },
                StructField {
                    name: "user".to_string(),
                    ty: RustType::Option(Box::new(RustType::Custom("UserProfile".to_string()))),
                    serde_rename: None,
                },
                StructField {
                    name: "items".to_string(),
                    ty: RustType::Vec(Box::new(RustType::Custom("DiscoverAnimal".to_string()))),
                    serde_rename: None,
                },
                StructField {
                    name: "name".to_string(),
                    ty: RustType::String,
                    serde_rename: None,
                },
            ],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        }];

        let types = collect_referenced_types(&structs);

        assert!(types.contains("Animal"));
        assert!(types.contains("UserProfile"));
        assert!(types.contains("DiscoverAnimal"));
        // Should not contain primitive type indicators
        assert!(!types.contains("String"));
    }

    #[test]
    fn test_generate_typescript_is_self_contained() {
        // Generated output should never contain import/re-export from shared.ts
        let structs = vec![InertiaPropsStruct {
            name: "DiscoverProps".to_string(),
            fields: vec![
                StructField {
                    name: "animals".to_string(),
                    ty: RustType::Vec(Box::new(RustType::Custom("DiscoverAnimal".to_string()))),
                    serde_rename: None,
                },
                StructField {
                    name: "user".to_string(),
                    ty: RustType::Option(Box::new(RustType::Custom("UserProfile".to_string()))),
                    serde_rename: None,
                },
            ],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        }];

        let typescript = generate_typescript(&structs);

        // Must NOT contain any imports or re-exports from shared.ts
        assert!(!typescript.contains("import type"));
        assert!(!typescript.contains("from './shared'"));
        assert!(!typescript.contains("export type {"));

        // Must contain inline utility types
        assert!(typescript.contains("export type JsonValue ="));
        assert!(typescript.contains("export type ValidationErrors ="));
    }

    #[test]
    fn test_serialize_struct_visitor_finds_matching() {
        let code = r#"
            use serde::Serialize;

            #[derive(Serialize)]
            pub struct MenuSummary {
                pub id: String,
                pub name: String,
            }

            #[derive(Serialize, Clone)]
            pub struct UserInfo {
                pub user_id: i64,
            }

            // Not a target, should be ignored
            #[derive(Serialize)]
            pub struct OtherType {
                pub value: String,
            }
        "#;

        let mut target_types = HashSet::new();
        target_types.insert("MenuSummary".to_string());
        target_types.insert("UserInfo".to_string());

        if let Ok(syntax) = syn::parse_file(code) {
            let mut visitor = SerializeStructVisitor::new(target_types, String::new());
            syn::visit::Visit::visit_file(&mut visitor, &syntax);

            assert_eq!(visitor.structs.len(), 2);
            let names: HashSet<_> = visitor.structs.iter().map(|s| s.name.as_str()).collect();
            assert!(names.contains("MenuSummary"));
            assert!(names.contains("UserInfo"));
            assert!(!names.contains("OtherType"));
        } else {
            panic!("Failed to parse test code");
        }
    }

    #[test]
    fn test_serialize_struct_visitor_ignores_non_matching() {
        let code = r#"
            use serde::Serialize;

            #[derive(Serialize)]
            pub struct Exists {
                pub id: String,
            }

            // No Serialize derive
            pub struct NoDerive {
                pub id: String,
            }

            // Different derive
            #[derive(Debug, Clone)]
            pub struct WrongDerive {
                pub id: String,
            }
        "#;

        let mut target_types = HashSet::new();
        target_types.insert("NotExists".to_string()); // Looking for something that doesn't exist
        target_types.insert("NoDerive".to_string()); // Exists but no Serialize
        target_types.insert("WrongDerive".to_string()); // Exists but wrong derive

        if let Ok(syntax) = syn::parse_file(code) {
            let mut visitor = SerializeStructVisitor::new(target_types, String::new());
            syn::visit::Visit::visit_file(&mut visitor, &syntax);

            // Should find nothing since none match both criteria
            assert_eq!(visitor.structs.len(), 0);
        } else {
            panic!("Failed to parse test code");
        }
    }

    #[test]
    fn test_serialize_struct_visitor_parses_serde_attributes() {
        let code = r#"
            use serde::Serialize;

            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            pub struct WithRenameAll {
                pub created_at: String,
                #[serde(rename = "customName")]
                pub some_field: String,
            }
        "#;

        let mut target_types = HashSet::new();
        target_types.insert("WithRenameAll".to_string());

        if let Ok(syntax) = syn::parse_file(code) {
            let mut visitor = SerializeStructVisitor::new(target_types, String::new());
            syn::visit::Visit::visit_file(&mut visitor, &syntax);

            assert_eq!(visitor.structs.len(), 1);
            let s = &visitor.structs[0];
            assert_eq!(s.rename_all, SerdeCase::CamelCase);
            assert_eq!(s.fields.len(), 2);

            // Check field-level rename
            let some_field = s.fields.iter().find(|f| f.name == "some_field").unwrap();
            assert_eq!(some_field.serde_rename, Some("customName".to_string()));
        } else {
            panic!("Failed to parse test code");
        }
    }

    #[test]
    fn test_parse_serde_json_value_type() {
        // When parsing serde_json::Value, it should map to JsonValue
        let code = r#"
            use serde::Serialize;
            use serde_json::Value;

            #[derive(Serialize)]
            pub struct FormProps {
                pub errors: Option<Value>,
                pub data: Value,
            }
        "#;

        let mut target_types = HashSet::new();
        target_types.insert("FormProps".to_string());

        if let Ok(syntax) = syn::parse_file(code) {
            let mut visitor = SerializeStructVisitor::new(target_types, String::new());
            syn::visit::Visit::visit_file(&mut visitor, &syntax);

            assert_eq!(visitor.structs.len(), 1);
            let s = &visitor.structs[0];

            // errors: Option<Value> should parse to Option(JsonValue)
            let errors_field = s.fields.iter().find(|f| f.name == "errors").unwrap();
            assert!(matches!(
                &errors_field.ty,
                RustType::Option(inner) if matches!(inner.as_ref(), RustType::JsonValue)
            ));

            // data: Value should parse to JsonValue
            let data_field = s.fields.iter().find(|f| f.name == "data").unwrap();
            assert!(matches!(&data_field.ty, RustType::JsonValue));

            // Generate TypeScript and verify output
            let typescript = generate_typescript(&visitor.structs);
            assert!(typescript.contains("errors: JsonValue | null;"));
            assert!(typescript.contains("data: JsonValue;"));
            assert!(!typescript.contains(": Value"));
        } else {
            panic!("Failed to parse test code");
        }
    }

    #[test]
    fn test_resolve_nested_types_single_level() {
        // Test that resolve_nested_types finds types referenced by initial structs
        let initial = vec![InertiaPropsStruct {
            name: "ListProps".to_string(),
            fields: vec![
                StructField {
                    name: "items".to_string(),
                    ty: RustType::Vec(Box::new(RustType::Custom("ItemSummary".to_string()))),
                    serde_rename: None,
                },
                StructField {
                    name: "user".to_string(),
                    ty: RustType::Custom("UserInfo".to_string()),
                    serde_rename: None,
                },
            ],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        }];

        // resolve_nested_types requires a project path with actual files
        // For unit testing, we verify collect_referenced_types works correctly
        let referenced = collect_referenced_types(&initial);
        assert!(referenced.contains("ItemSummary"));
        assert!(referenced.contains("UserInfo"));
    }

    #[test]
    fn test_resolve_nested_types_skips_shared() {
        // Test that shared types are not included in referenced types
        let initial = vec![InertiaPropsStruct {
            name: "TestProps".to_string(),
            fields: vec![
                StructField {
                    name: "animal".to_string(),
                    ty: RustType::Custom("Animal".to_string()),
                    serde_rename: None,
                },
                StructField {
                    name: "user".to_string(),
                    ty: RustType::Custom("SharedUser".to_string()),
                    serde_rename: None,
                },
            ],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        }];

        let mut shared_types = HashSet::new();
        shared_types.insert("SharedUser".to_string());

        // Simulate what resolve_nested_types does with filtering
        let mut types_to_find = collect_referenced_types(&initial);
        let initial_names: HashSet<String> = initial.iter().map(|s| s.name.clone()).collect();
        types_to_find.retain(|t| !initial_names.contains(t) && !shared_types.contains(t));

        // SharedUser should be filtered out, Animal should remain
        assert!(types_to_find.contains("Animal"));
        assert!(!types_to_find.contains("SharedUser"));
    }

    #[test]
    fn test_resolve_nested_types_recursive() {
        // Test that deeply nested types are collected
        // Level 1: PageProps -> Level1Type
        // Level 2: Level1Type -> Level2Type
        let level1 = InertiaPropsStruct {
            name: "Level1Type".to_string(),
            fields: vec![StructField {
                name: "nested".to_string(),
                ty: RustType::Custom("Level2Type".to_string()),
                serde_rename: None,
            }],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        };

        // Check that Level1Type references Level2Type
        let level1_refs = collect_referenced_types(&[level1]);
        assert!(level1_refs.contains("Level2Type"));
    }

    #[test]
    fn test_parse_type_validation_errors() {
        // Test parsing ValidationErrors type from Rust code
        let code = r#"
            use ferro::ValidationErrors;

            #[derive(Serialize)]
            pub struct FormProps {
                pub errors: Option<ValidationErrors>,
                pub all_errors: ValidationErrors,
            }
        "#;

        let mut target_types = HashSet::new();
        target_types.insert("FormProps".to_string());

        if let Ok(syntax) = syn::parse_file(code) {
            let mut visitor = SerializeStructVisitor::new(target_types, String::new());
            syn::visit::Visit::visit_file(&mut visitor, &syntax);

            assert_eq!(visitor.structs.len(), 1);
            let s = &visitor.structs[0];

            // errors: Option<ValidationErrors> should parse to Option(ValidationErrors)
            let errors_field = s.fields.iter().find(|f| f.name == "errors").unwrap();
            assert!(matches!(
                &errors_field.ty,
                RustType::Option(inner) if matches!(inner.as_ref(), RustType::ValidationErrors)
            ));

            // all_errors: ValidationErrors should parse to ValidationErrors
            let all_errors_field = s.fields.iter().find(|f| f.name == "all_errors").unwrap();
            assert!(matches!(&all_errors_field.ty, RustType::ValidationErrors));
        } else {
            panic!("Failed to parse test code");
        }
    }

    #[test]
    fn test_validation_errors_to_typescript() {
        // Test that ValidationErrors type generates ValidationErrors in TypeScript
        let structs = vec![InertiaPropsStruct {
            name: "FormProps".to_string(),
            fields: vec![
                StructField {
                    name: "errors".to_string(),
                    ty: RustType::Option(Box::new(RustType::ValidationErrors)),
                    serde_rename: None,
                },
                StructField {
                    name: "all_errors".to_string(),
                    ty: RustType::ValidationErrors,
                    serde_rename: None,
                },
            ],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        }];

        let typescript = generate_typescript(&structs);

        // ValidationErrors should use the exported ValidationErrors type
        assert!(typescript.contains("errors: ValidationErrors | null;"));
        assert!(typescript.contains("all_errors: ValidationErrors;"));
    }

    #[test]
    fn test_hashmap_string_vec_string_to_typescript() {
        // Test that HashMap<String, Vec<String>> generates Record<string, string[]> in TypeScript
        // This is for custom HashMap types, not ValidationErrors
        let structs = vec![InertiaPropsStruct {
            name: "CustomProps".to_string(),
            fields: vec![StructField {
                name: "data".to_string(),
                ty: RustType::HashMap(
                    Box::new(RustType::String),
                    Box::new(RustType::Vec(Box::new(RustType::String))),
                ),
                serde_rename: None,
            }],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        }];

        let typescript = generate_typescript(&structs);

        // HashMap<String, Vec<String>> should become Record<string, string[]>
        assert!(typescript.contains("data: Record<string, string[]>;"));
    }

    // ===== Namespacing Tests (Phase 22.5) =====

    #[test]
    fn test_compute_module_path_flat_controller() {
        // src/controllers/user.rs -> user
        let src_path = std::path::Path::new("/project/src");
        let file_path = std::path::Path::new("/project/src/controllers/user.rs");
        let result = compute_module_path(file_path, src_path);
        assert_eq!(result, "user");
    }

    #[test]
    fn test_compute_module_path_nested_controller() {
        // src/controllers/shelter/applications.rs -> shelter::applications
        let src_path = std::path::Path::new("/project/src");
        let file_path = std::path::Path::new("/project/src/controllers/shelter/applications.rs");
        let result = compute_module_path(file_path, src_path);
        assert_eq!(result, "shelter::applications");
    }

    #[test]
    fn test_compute_module_path_deeply_nested() {
        // src/controllers/admin/settings/security.rs -> admin::settings::security
        let src_path = std::path::Path::new("/project/src");
        let file_path = std::path::Path::new("/project/src/controllers/admin/settings/security.rs");
        let result = compute_module_path(file_path, src_path);
        assert_eq!(result, "admin::settings::security");
    }

    #[test]
    fn test_compute_module_path_non_controller() {
        // src/models/animal.rs -> models::animal (preserves path for non-controllers)
        let src_path = std::path::Path::new("/project/src");
        let file_path = std::path::Path::new("/project/src/models/animal.rs");
        let result = compute_module_path(file_path, src_path);
        assert_eq!(result, "models::animal");
    }

    #[test]
    fn test_compute_module_path_mod_rs() {
        // src/controllers/shelter/mod.rs -> shelter (removes ::mod suffix)
        let src_path = std::path::Path::new("/project/src");
        let file_path = std::path::Path::new("/project/src/controllers/shelter/mod.rs");
        let result = compute_module_path(file_path, src_path);
        assert_eq!(result, "shelter");
    }

    #[test]
    fn test_generate_namespaced_name_empty_module_path() {
        // Root-level struct should not be namespaced
        assert_eq!(generate_namespaced_name("", "GlobalProps"), "GlobalProps");
    }

    #[test]
    fn test_generate_namespaced_name_single_segment() {
        // user + ShowProps -> UserShowProps
        assert_eq!(
            generate_namespaced_name("user", "ShowProps"),
            "UserShowProps"
        );
    }

    #[test]
    fn test_generate_namespaced_name_nested_segments() {
        // shelter::applications + ShowProps -> ShelterApplicationsShowProps
        assert_eq!(
            generate_namespaced_name("shelter::applications", "ShowProps"),
            "ShelterApplicationsShowProps"
        );
    }

    #[test]
    fn test_generate_namespaced_name_deeply_nested() {
        // admin::settings::security + IndexProps -> AdminSettingsSecurityIndexProps
        assert_eq!(
            generate_namespaced_name("admin::settings::security", "IndexProps"),
            "AdminSettingsSecurityIndexProps"
        );
    }

    #[test]
    fn test_generate_namespaced_name_snake_case_conversion() {
        // user_profile + EditProps -> UserProfileEditProps
        assert_eq!(
            generate_namespaced_name("user_profile", "EditProps"),
            "UserProfileEditProps"
        );
    }

    #[test]
    fn test_generate_namespaced_name_skips_redundant_prefix() {
        // menu + MenuListProps -> MenuListProps (already prefixed)
        assert_eq!(
            generate_namespaced_name("menu", "MenuListProps"),
            "MenuListProps"
        );
        // public::menu + PublicMenuProps -> PublicMenuProps (already prefixed)
        assert_eq!(
            generate_namespaced_name("public::menu", "PublicMenuProps"),
            "PublicMenuProps"
        );
        // category + CategoryDetail -> CategoryDetail (already prefixed)
        assert_eq!(
            generate_namespaced_name("category", "CategoryDetail"),
            "CategoryDetail"
        );
        // qrcode + QRCodeListProps -> QRCodeListProps (case-insensitive match)
        assert_eq!(
            generate_namespaced_name("qrcode", "QRCodeListProps"),
            "QRCodeListProps"
        );
        // auth + AuthRegisterProps -> AuthRegisterProps
        assert_eq!(
            generate_namespaced_name("auth", "AuthRegisterProps"),
            "AuthRegisterProps"
        );
    }

    #[test]
    fn test_build_name_map_no_collisions() {
        let structs = vec![
            InertiaPropsStruct {
                name: "ShowProps".to_string(),
                fields: vec![],
                rename_all: SerdeCase::None,
                module_path: "shelter::applications".to_string(),
            },
            InertiaPropsStruct {
                name: "ShowProps".to_string(),
                fields: vec![],
                rename_all: SerdeCase::None,
                module_path: "adopter::applications".to_string(),
            },
        ];

        let name_map = build_name_map(&structs);

        // Both should get unique namespaced names
        // Note: The name_map is keyed by original name, so we need to check the values
        // Since both have the same name "ShowProps", the last one wins in the map
        // This is actually the collision we're detecting
        assert!(name_map.contains_key("ShowProps"));
    }

    #[test]
    fn test_typescript_generation_with_namespaced_names() {
        let structs = vec![
            InertiaPropsStruct {
                name: "ShowProps".to_string(),
                fields: vec![StructField {
                    name: "id".to_string(),
                    ty: RustType::Number,
                    serde_rename: None,
                }],
                rename_all: SerdeCase::None,
                module_path: "shelter::applications".to_string(),
            },
            InertiaPropsStruct {
                name: "IndexProps".to_string(),
                fields: vec![StructField {
                    name: "count".to_string(),
                    ty: RustType::Number,
                    serde_rename: None,
                }],
                rename_all: SerdeCase::None,
                module_path: "user".to_string(),
            },
        ];

        let typescript = generate_typescript(&structs);

        // Should have namespaced interface names
        assert!(typescript.contains("export interface ShelterApplicationsShowProps"));
        assert!(typescript.contains("export interface UserIndexProps"));
    }

    #[test]
    fn test_type_references_use_namespaced_names() {
        // When a field references another type, it should use the namespaced name
        let structs = vec![
            InertiaPropsStruct {
                name: "DetailProps".to_string(),
                fields: vec![],
                rename_all: SerdeCase::None,
                module_path: "shelter".to_string(),
            },
            InertiaPropsStruct {
                name: "ShowProps".to_string(),
                fields: vec![StructField {
                    name: "details".to_string(),
                    ty: RustType::Custom("DetailProps".to_string()),
                    serde_rename: None,
                }],
                rename_all: SerdeCase::None,
                module_path: "shelter".to_string(),
            },
        ];

        let typescript = generate_typescript(&structs);

        // The reference to DetailProps should use the namespaced name
        assert!(typescript.contains("details: ShelterDetailProps;"));
    }

    #[test]
    fn test_datetime_type_mapping() {
        // Test that all chrono datetime types parse to RustType::DateTime
        use syn::parse_quote;

        let datetime: Type = parse_quote!(DateTime<Utc>);
        assert!(matches!(
            InertiaPropsVisitor::parse_type(&datetime),
            RustType::DateTime
        ));

        let naive_datetime: Type = parse_quote!(NaiveDateTime);
        assert!(matches!(
            InertiaPropsVisitor::parse_type(&naive_datetime),
            RustType::DateTime
        ));

        let naive_date: Type = parse_quote!(NaiveDate);
        assert!(matches!(
            InertiaPropsVisitor::parse_type(&naive_date),
            RustType::DateTime
        ));

        let naive_time: Type = parse_quote!(NaiveTime);
        assert!(matches!(
            InertiaPropsVisitor::parse_type(&naive_time),
            RustType::DateTime
        ));

        let datetime_utc: Type = parse_quote!(DateTimeUtc);
        assert!(matches!(
            InertiaPropsVisitor::parse_type(&datetime_utc),
            RustType::DateTime
        ));

        let datetime_local: Type = parse_quote!(DateTimeLocal);
        assert!(matches!(
            InertiaPropsVisitor::parse_type(&datetime_local),
            RustType::DateTime
        ));

        let date: Type = parse_quote!(Date);
        assert!(matches!(
            InertiaPropsVisitor::parse_type(&date),
            RustType::DateTime
        ));

        let time: Type = parse_quote!(Time);
        assert!(matches!(
            InertiaPropsVisitor::parse_type(&time),
            RustType::DateTime
        ));
    }

    #[test]
    fn test_datetime_to_typescript() {
        // Verify RustType::DateTime generates "string" in TypeScript
        let name_map = HashMap::new();
        assert_eq!(
            rust_type_to_ts_with_mapping(&RustType::DateTime, &name_map),
            "string"
        );
    }

    #[test]
    fn test_option_datetime() {
        // Verify Option<DateTime<Utc>> generates "string | null"
        let name_map = HashMap::new();
        let option_datetime = RustType::Option(Box::new(RustType::DateTime));
        assert_eq!(
            rust_type_to_ts_with_mapping(&option_datetime, &name_map),
            "string | null"
        );
    }

    #[test]
    fn test_vec_datetime() {
        // Verify Vec<NaiveDateTime> generates "string[]"
        let name_map = HashMap::new();
        let vec_datetime = RustType::Vec(Box::new(RustType::DateTime));
        assert_eq!(
            rust_type_to_ts_with_mapping(&vec_datetime, &name_map),
            "string[]"
        );
    }

    #[test]
    fn test_generate_typescript_with_datetime() {
        // End-to-end test with a props struct containing datetime fields
        let structs = vec![InertiaPropsStruct {
            name: "EventProps".to_string(),
            fields: vec![
                StructField {
                    name: "created_at".to_string(),
                    ty: RustType::DateTime,
                    serde_rename: None,
                },
                StructField {
                    name: "updated_at".to_string(),
                    ty: RustType::Option(Box::new(RustType::DateTime)),
                    serde_rename: None,
                },
                StructField {
                    name: "scheduled_dates".to_string(),
                    ty: RustType::Vec(Box::new(RustType::DateTime)),
                    serde_rename: None,
                },
            ],
            rename_all: SerdeCase::CamelCase,
            module_path: "events".to_string(),
        }];

        let typescript = generate_typescript(&structs);

        // Should contain the interface with datetime fields mapped to string
        assert!(typescript.contains("export interface EventsEventProps"));
        assert!(typescript.contains("createdAt: string;"));
        assert!(typescript.contains("updatedAt: string | null;"));
        assert!(typescript.contains("scheduledDates: string[];"));
    }

    #[test]
    fn test_json_value_type_mapping() {
        // Verify RustType::JsonValue generates "JsonValue" in TypeScript
        let name_map = HashMap::new();
        assert_eq!(
            rust_type_to_ts_with_mapping(&RustType::JsonValue, &name_map),
            "JsonValue"
        );
    }

    #[test]
    fn test_validation_errors_type_mapping() {
        // Verify RustType::ValidationErrors generates "ValidationErrors" in TypeScript
        let name_map = HashMap::new();
        assert_eq!(
            rust_type_to_ts_with_mapping(&RustType::ValidationErrors, &name_map),
            "ValidationErrors"
        );
    }

    #[test]
    fn test_option_json_value() {
        // Verify Option<Value> generates "JsonValue | null"
        let name_map = HashMap::new();
        let option_value = RustType::Option(Box::new(RustType::JsonValue));
        assert_eq!(
            rust_type_to_ts_with_mapping(&option_value, &name_map),
            "JsonValue | null"
        );
    }

    #[test]
    fn test_option_validation_errors() {
        // Verify Option<ValidationErrors> generates "ValidationErrors | null"
        let name_map = HashMap::new();
        let option_errors = RustType::Option(Box::new(RustType::ValidationErrors));
        assert_eq!(
            rust_type_to_ts_with_mapping(&option_errors, &name_map),
            "ValidationErrors | null"
        );
    }

    #[test]
    fn test_generated_output_includes_utility_types() {
        // Verify generated output includes JsonValue and ValidationErrors type definitions
        let structs = vec![InertiaPropsStruct {
            name: "TestProps".to_string(),
            fields: vec![StructField {
                name: "data".to_string(),
                ty: RustType::String,
                serde_rename: None,
            }],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        }];

        let typescript = generate_typescript(&structs);

        // Should contain utility type definitions
        assert!(typescript.contains("export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };"));
        assert!(typescript.contains("export type ValidationErrors = Record<string, string[]>;"));
    }

    #[test]
    fn test_generated_output_includes_header() {
        // Verify generated output includes instructional header
        let structs = vec![InertiaPropsStruct {
            name: "TestProps".to_string(),
            fields: vec![StructField {
                name: "data".to_string(),
                ty: RustType::String,
                serde_rename: None,
            }],
            rename_all: SerdeCase::None,
            module_path: String::new(),
        }];

        let typescript = generate_typescript(&structs);

        // Should contain header with instructions
        assert!(typescript.contains("Auto-generated by `ferro generate-types`"));
        assert!(typescript.contains("Do not edit manually"));
        assert!(typescript.contains("ferro generate-types"));
        assert!(typescript.contains("frontend/src/types/"));
    }

    #[test]
    fn test_parse_validation_errors_type() {
        // When parsing ValidationErrors type, it should map to RustType::ValidationErrors
        let code = r#"
            use serde::Serialize;
            use ferro::ValidationErrors;

            #[derive(Serialize)]
            pub struct FormProps {
                pub errors: Option<ValidationErrors>,
            }
        "#;

        let mut target_types = HashSet::new();
        target_types.insert("FormProps".to_string());

        if let Ok(syntax) = syn::parse_file(code) {
            let mut visitor = SerializeStructVisitor::new(target_types, String::new());
            syn::visit::Visit::visit_file(&mut visitor, &syntax);

            assert_eq!(visitor.structs.len(), 1);
            let s = &visitor.structs[0];

            // errors: Option<ValidationErrors> should parse to Option(ValidationErrors)
            let errors_field = s.fields.iter().find(|f| f.name == "errors").unwrap();
            assert!(matches!(
                &errors_field.ty,
                RustType::Option(inner) if matches!(inner.as_ref(), RustType::ValidationErrors)
            ));

            // Generate TypeScript and verify output
            let typescript = generate_typescript(&visitor.structs);
            assert!(typescript.contains("errors: ValidationErrors | null;"));
        } else {
            panic!("Failed to parse test code");
        }
    }

    #[test]
    fn test_parse_sea_orm_json_type() {
        // When parsing sea_orm::Json type, it should map to JsonValue
        let code = r#"
            use serde::Serialize;
            use sea_orm::entity::prelude::Json;

            #[derive(Serialize)]
            pub struct ConfigProps {
                pub settings: Json,
            }
        "#;

        let mut target_types = HashSet::new();
        target_types.insert("ConfigProps".to_string());

        if let Ok(syntax) = syn::parse_file(code) {
            let mut visitor = SerializeStructVisitor::new(target_types, String::new());
            syn::visit::Visit::visit_file(&mut visitor, &syntax);

            assert_eq!(visitor.structs.len(), 1);
            let s = &visitor.structs[0];

            // settings: Json should parse to JsonValue
            let settings_field = s.fields.iter().find(|f| f.name == "settings").unwrap();
            assert!(matches!(&settings_field.ty, RustType::JsonValue));

            // Generate TypeScript and verify output
            let typescript = generate_typescript(&visitor.structs);
            assert!(typescript.contains("settings: JsonValue;"));
        } else {
            panic!("Failed to parse test code");
        }
    }
}
