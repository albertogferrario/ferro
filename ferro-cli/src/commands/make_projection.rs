use crate::naming::{is_valid_identifier, to_snake_case};
use console::style;
use quote::ToTokens;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, Fields, ItemStruct, Type};
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Model field metadata (self-contained, mirrors make_api::FieldInfo)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct ModelField {
    name: String,
    rust_type: String,
    is_primary_key: bool,
    is_nullable: bool,
}

// ---------------------------------------------------------------------------
// AST visitor for model detection
// ---------------------------------------------------------------------------

struct ModelVisitor {
    fields: Vec<ModelField>,
    found: bool,
}

impl ModelVisitor {
    fn new() -> Self {
        Self {
            fields: Vec::new(),
            found: false,
        }
    }

    fn has_model_derive(attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        let ident = path.segments.last().map(|s| s.ident.to_string());
                        if matches!(
                            ident.as_deref(),
                            Some("DeriveEntityModel") | Some("FerroModel")
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_field_primary_key(attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("sea_orm") {
                let tokens = attr.meta.to_token_stream().to_string();
                if tokens.contains("primary_key") {
                    return true;
                }
            }
        }
        false
    }

    fn type_to_string(ty: &Type) -> String {
        ty.to_token_stream().to_string().replace(' ', "")
    }

    fn extract_fields(fields: &Fields) -> Vec<ModelField> {
        let mut result = Vec::new();
        if let Fields::Named(named) = fields {
            for field in &named.named {
                if let Some(ident) = &field.ident {
                    let name = ident.to_string();
                    let rust_type = Self::type_to_string(&field.ty);
                    let is_nullable = rust_type.starts_with("Option<");
                    let is_primary_key = Self::is_field_primary_key(&field.attrs);
                    result.push(ModelField {
                        name,
                        rust_type,
                        is_primary_key,
                        is_nullable,
                    });
                }
            }
        }
        result
    }
}

impl<'ast> Visit<'ast> for ModelVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if Self::has_model_derive(&node.attrs) && node.ident == "Model" {
            self.fields = Self::extract_fields(&node.fields);
            self.found = true;
        }
        syn::visit::visit_item_struct(self, node);
    }
}

// ---------------------------------------------------------------------------
// Model scanning
// ---------------------------------------------------------------------------

/// Scan `src/models/` for model files and return matching model fields.
fn scan_models(project_root: &Path) -> Vec<(String, Vec<ModelField>)> {
    let models_dir = project_root.join("src/models");
    if !models_dir.exists() || !models_dir.is_dir() {
        return Vec::new();
    }

    let mut results = Vec::new();

    for entry in WalkDir::new(&models_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let file_stem = entry
            .path()
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        if file_stem == "mod" {
            continue;
        }

        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(syntax) = syn::parse_file(&content) else {
            continue;
        };

        let mut visitor = ModelVisitor::new();
        visitor.visit_file(&syntax);

        if visitor.found {
            let is_entity_file = entry
                .path()
                .parent()
                .and_then(|p| p.file_name())
                .is_some_and(|dir| dir == "entities");

            let singular_stem = if is_entity_file {
                singularize(&file_stem)
            } else {
                file_stem.clone()
            };

            results.push((singular_stem, visitor.fields));
        }
    }

    results
}

/// Scan models from a specific directory (for testing).
#[cfg(test)]
fn scan_models_from_dir(models_dir: &Path) -> Vec<(String, Vec<ModelField>)> {
    if !models_dir.exists() || !models_dir.is_dir() {
        return Vec::new();
    }

    let mut results = Vec::new();

    for entry in WalkDir::new(models_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let file_stem = entry
            .path()
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        if file_stem == "mod" {
            continue;
        }

        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(syntax) = syn::parse_file(&content) else {
            continue;
        };

        let mut visitor = ModelVisitor::new();
        visitor.visit_file(&syntax);

        if visitor.found {
            results.push((file_stem, visitor.fields));
        }
    }

    results
}

fn singularize(name: &str) -> String {
    if name.ends_with("ies") && name.len() > 3 {
        format!("{}y", &name[..name.len() - 3])
    } else if name.ends_with("ses")
        || name.ends_with("xes")
        || name.ends_with("ches")
        || name.ends_with("shes")
    {
        name[..name.len() - 2].to_string()
    } else if name.ends_with('s') && !name.ends_with("ss") {
        name[..name.len() - 1].to_string()
    } else {
        name.to_string()
    }
}

// ---------------------------------------------------------------------------
// Type and meaning mapping
// ---------------------------------------------------------------------------

/// Map a Rust type string to a DataType name for code generation.
fn rust_type_to_data_type(rust_type: &str) -> &'static str {
    let inner = unwrap_option(rust_type);

    match inner {
        "String" | "&str" => "DataType::String",
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize" | "isize" => {
            "DataType::Integer"
        }
        "f32" | "f64" | "Decimal" => "DataType::Float",
        "bool" => "DataType::Boolean",
        "DateTime"
        | "NaiveDateTime"
        | "DateTimeUtc"
        | "DateTimeWithTimeZone"
        | "chrono::DateTime<chrono::Utc>"
        | "chrono::NaiveDateTime" => "DataType::DateTime",
        "NaiveDate" | "Date" | "chrono::NaiveDate" => "DataType::Date",
        "Uuid" | "uuid::Uuid" => "DataType::Uuid",
        "Vec<u8>" => "DataType::Binary",
        "Json" | "serde_json::Value" | "JsonValue" => "DataType::Json",
        _ => "DataType::String",
    }
}

/// Unwrap `Option<T>` to get the inner type string.
fn unwrap_option(ty: &str) -> &str {
    if let Some(inner) = ty.strip_prefix("Option<") {
        if let Some(inner) = inner.strip_suffix('>') {
            return inner;
        }
    }
    ty
}

/// Infer a FieldMeaning variant name from a field name.
/// Replicates the logic from ferro-projections/src/field.rs.
fn infer_meaning(field_name: &str) -> &'static str {
    match field_name {
        "id" => "FieldMeaning::Identifier",
        "email" => "FieldMeaning::Email",
        "created_at" => "FieldMeaning::CreatedAt",
        "updated_at" => "FieldMeaning::UpdatedAt",
        _ => {
            if field_name.ends_with("_id") {
                return "FieldMeaning::ForeignKey";
            }
            if field_name.ends_with("_at") {
                return "FieldMeaning::DateTime";
            }
            if field_name.starts_with("is_") || field_name.starts_with("has_") {
                return "FieldMeaning::Boolean";
            }
            if is_sensitive_field(field_name) {
                return "FieldMeaning::Sensitive";
            }
            "custom"
        }
    }
}

/// Check if a field name matches sensitive patterns.
fn is_sensitive_field(field_name: &str) -> bool {
    const SENSITIVE: &[&str] = &["password", "secret", "token", "api_key", "hashed_key"];
    SENSITIVE.iter().any(|s| field_name.contains(s))
}

/// Determine the builder method for a field.
/// Returns None if the field should be skipped (sensitive).
fn field_builder_method(field: &ModelField, meaning: &str) -> Option<&'static str> {
    if meaning == "FieldMeaning::Sensitive" {
        return None;
    }

    if field.is_primary_key
        || meaning == "FieldMeaning::CreatedAt"
        || meaning == "FieldMeaning::UpdatedAt"
        || meaning == "FieldMeaning::ForeignKey"
    {
        return Some("read_only_field");
    }

    if field.is_nullable {
        return Some("optional_field");
    }

    Some("field")
}

// ---------------------------------------------------------------------------
// Model-aware template generation
// ---------------------------------------------------------------------------

/// Generate a projection template populated from model fields.
fn model_aware_template(name: &str, display_name: &str, fields: &[ModelField]) -> String {
    let mut field_lines = Vec::new();
    let mut belongs_to_lines = Vec::new();

    for field in fields {
        let meaning = infer_meaning(&field.name);
        let data_type = rust_type_to_data_type(&field.rust_type);

        let Some(builder) = field_builder_method(field, meaning) else {
            continue;
        };

        let meaning_str = if meaning == "custom" {
            // Use {:?} debug-formatting (same as ai_make.rs) so field names with
            // special characters produce valid Rust source rather than broken code.
            format!("FieldMeaning::Custom({:?}.into())", field.name)
        } else {
            meaning.to_string()
        };

        field_lines.push(format!(
            "        .{builder}(\"{}\", {data_type}, {meaning_str})",
            field.name
        ));

        if field.name.ends_with("_id") && field.name.len() > 3 {
            let rel_name = &field.name[..field.name.len() - 3];
            belongs_to_lines.push(format!(
                "        .belongs_to(\"{rel_name}\", \"{rel_name}\")"
            ));
        }
    }

    let all_lines: Vec<String> = field_lines.into_iter().chain(belongs_to_lines).collect();

    let builder_calls = all_lines.join("\n");

    format!(
        r#"use ferro::{{
    DataType, FieldMeaning, ServiceDef,
}};

/// Build the {display_name} service projection.
///
/// Derived from the {display_name} model.
/// Describes the {display_name} entity's fields, relationships,
/// and behavioral semantics for intent derivation and UI rendering.
pub fn {name}_service() -> ServiceDef {{
    ServiceDef::new("{name}")
        .display_name("{display_name}")
{builder_calls}
}}
"#
    )
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn execute(name: &str, from_model: bool) {
    let file_name = to_snake_case(name);

    if !is_valid_identifier(&file_name) {
        eprintln!(
            "{} '{}' is not a valid projection name",
            style("Error:").red().bold(),
            name
        );
        std::process::exit(1);
    }

    let display_name = to_pascal_case(name);
    let projections_dir = Path::new("src/projections");
    let projection_file = projections_dir.join(format!("{file_name}.rs"));
    let mod_file = projections_dir.join("mod.rs");

    if !projections_dir.exists() {
        if let Err(e) = fs::create_dir_all(projections_dir) {
            eprintln!(
                "{} Failed to create src/projections directory: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/projections/", style("✓").green());
    }

    if projection_file.exists() {
        eprintln!(
            "{} Projection '{}' already exists at {}",
            style("Info:").yellow().bold(),
            file_name,
            projection_file.display()
        );
        std::process::exit(0);
    }

    if mod_file.exists() {
        let mod_content = fs::read_to_string(&mod_file).unwrap_or_default();
        let mod_decl = format!("mod {file_name};");
        let pub_mod_decl = format!("pub mod {file_name};");
        if mod_content.contains(&mod_decl) || mod_content.contains(&pub_mod_decl) {
            eprintln!(
                "{} Module '{}' is already declared in src/projections/mod.rs",
                style("Info:").yellow().bold(),
                file_name
            );
            std::process::exit(0);
        }
    }

    let content = if from_model {
        let project_root = Path::new(".");
        let available = scan_models(project_root);

        let matched = available
            .iter()
            .find(|(sn, _)| sn.eq_ignore_ascii_case(&file_name));

        match matched {
            Some((_, fields)) => {
                println!(
                    "{} Found model '{}' with {} fields",
                    style("✓").green(),
                    display_name,
                    fields.len()
                );
                model_aware_template(&file_name, &display_name, fields)
            }
            None => {
                let model_names: Vec<&str> = available.iter().map(|(n, _)| n.as_str()).collect();
                if model_names.is_empty() {
                    eprintln!(
                        "{} No models found in src/models/",
                        style("Error:").red().bold()
                    );
                } else {
                    eprintln!(
                        "{} Model '{}' not found. Available models: {}",
                        style("Error:").red().bold(),
                        file_name,
                        model_names.join(", ")
                    );
                }
                std::process::exit(1);
            }
        }
    } else {
        projection_template(&file_name, &display_name)
    };

    if let Err(e) = fs::write(&projection_file, &content) {
        eprintln!(
            "{} Failed to write projection file: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!(
        "{} Created {}",
        style("✓").green(),
        projection_file.display()
    );

    if mod_file.exists() {
        if let Err(e) = update_mod_file(&mod_file, &file_name) {
            eprintln!(
                "{} Failed to update mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Updated src/projections/mod.rs", style("✓").green());
    } else {
        let mod_content = format!("pub mod {file_name};\n");
        if let Err(e) = fs::write(&mod_file, mod_content) {
            eprintln!(
                "{} Failed to create mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/projections/mod.rs", style("✓").green());
    }

    println!();
    println!(
        "Projection {} created successfully!",
        style(&file_name).cyan().bold()
    );
    println!();
    println!("Usage:");
    println!(
        "  {} Define fields matching your model in src/projections/{file_name}.rs",
        style("1.").dim()
    );
    println!("  {} Use in a handler:", style("2.").dim());
    println!("     use crate::projections::{file_name};");
    println!();
    println!("     let service = {file_name}::{file_name}_service();");
    println!("     let intents = derive_intents(&service);");
    println!();
}

fn projection_template(name: &str, display_name: &str) -> String {
    format!(
        r#"use ferro::{{
    DataType, FieldMeaning, ServiceDef,
}};

/// Build the {display_name} service projection.
///
/// Describes the {display_name} entity's fields, relationships,
/// and behavioral semantics for intent derivation and UI rendering.
pub fn {name}_service() -> ServiceDef {{
    ServiceDef::new("{name}")
        .display_name("{display_name}")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        // Add fields matching your model:
        // .field("name", DataType::String, FieldMeaning::EntityName)
        // .field("email", DataType::String, FieldMeaning::Email)
        // .field("status", DataType::String, FieldMeaning::Status)
        // .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        // .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
}}
"#
    )
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Generate projection files at the given base directory.
/// Returns (projection_file_path, mod_file_path) on success.
#[cfg(test)]
fn generate_in_dir(
    base_dir: &Path,
    name: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    let file_name = to_snake_case(name);
    let display_name = to_pascal_case(name);
    let projections_dir = base_dir.join("src/projections");
    let projection_file = projections_dir.join(format!("{file_name}.rs"));
    let mod_file = projections_dir.join("mod.rs");

    fs::create_dir_all(&projections_dir)
        .map_err(|e| format!("Failed to create projections directory: {e}"))?;

    let content = projection_template(&file_name, &display_name);
    fs::write(&projection_file, content)
        .map_err(|e| format!("Failed to write projection file: {e}"))?;

    if mod_file.exists() {
        let mod_content = fs::read_to_string(&mod_file).unwrap_or_default();
        let pub_mod_decl = format!("pub mod {file_name};");
        if !mod_content.contains(&pub_mod_decl) {
            update_mod_file(&mod_file, &file_name)?;
        }
    } else {
        let mod_content = format!("pub mod {file_name};\n");
        fs::write(&mod_file, mod_content).map_err(|e| format!("Failed to create mod.rs: {e}"))?;
    }

    Ok((projection_file, mod_file))
}

pub(crate) fn update_mod_file(mod_file: &Path, file_name: &str) -> Result<(), String> {
    let content =
        fs::read_to_string(mod_file).map_err(|e| format!("Failed to read mod.rs: {e}"))?;

    let pub_mod_decl = format!("pub mod {file_name};");

    let mut lines: Vec<&str> = content.lines().collect();

    let mut last_pub_mod_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("pub mod ") {
            last_pub_mod_idx = Some(i);
        }
    }

    let insert_idx = match last_pub_mod_idx {
        Some(idx) => idx + 1,
        None => {
            let mut insert_idx = 0;
            for (i, line) in lines.iter().enumerate() {
                if line.starts_with("//!") || line.is_empty() {
                    insert_idx = i + 1;
                } else {
                    break;
                }
            }
            insert_idx
        }
    };
    lines.insert(insert_idx, &pub_mod_decl);

    let new_content = lines.join("\n");
    fs::write(mod_file, new_content).map_err(|e| format!("Failed to write mod.rs: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_template_generation() {
        let template = projection_template("user", "User");

        assert!(template.contains("pub fn user_service() -> ServiceDef"));
        assert!(template.contains("ServiceDef::new(\"user\")"));
        assert!(template.contains(".display_name(\"User\")"));
        assert!(template.contains("DataType::Integer, FieldMeaning::Identifier"));
        assert!(template.contains("use ferro::{"));
        assert!(template.contains("/// Build the User service projection."));
    }

    #[test]
    fn test_creates_directory_and_file() {
        let tmp = TempDir::new().unwrap();
        let (proj_file, _mod_file) = generate_in_dir(tmp.path(), "order").unwrap();

        assert!(tmp.path().join("src/projections").exists());
        assert!(proj_file.exists());

        let content = fs::read_to_string(&proj_file).unwrap();
        assert!(content.contains("pub fn order_service() -> ServiceDef"));
        assert!(content.contains(".display_name(\"Order\")"));
    }

    #[test]
    fn test_mod_rs_creation() {
        let tmp = TempDir::new().unwrap();
        let (_proj_file, mod_file) = generate_in_dir(tmp.path(), "product").unwrap();

        assert!(mod_file.exists());
        let mod_content = fs::read_to_string(&mod_file).unwrap();
        assert!(mod_content.contains("pub mod product;"));
    }

    #[test]
    fn test_mod_rs_append() {
        let tmp = TempDir::new().unwrap();

        generate_in_dir(tmp.path(), "user").unwrap();
        let mod_file = tmp.path().join("src/projections/mod.rs");
        let content = fs::read_to_string(&mod_file).unwrap();
        assert!(content.contains("pub mod user;"));

        generate_in_dir(tmp.path(), "order").unwrap();
        let content = fs::read_to_string(&mod_file).unwrap();
        assert!(content.contains("pub mod user;"));
        assert!(content.contains("pub mod order;"));

        generate_in_dir(tmp.path(), "order").unwrap();
        let content = fs::read_to_string(&mod_file).unwrap();
        let count = content.matches("pub mod order;").count();
        assert_eq!(count, 1, "pub mod order; should appear exactly once");
    }

    // -- Model-aware tests --

    fn write_mock_model(dir: &Path, filename: &str, content: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn test_model_aware_template_basic() {
        let tmp = TempDir::new().unwrap();
        let models_dir = tmp.path().join("models");
        write_mock_model(
            &models_dir,
            "user.rs",
            r#"
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
"#,
        );

        let models = scan_models_from_dir(&models_dir);
        assert_eq!(models.len(), 1);
        let (name, fields) = &models[0];
        assert_eq!(name, "user");

        let output = model_aware_template("user", "User", fields);

        assert!(output.contains("pub fn user_service() -> ServiceDef"));
        assert!(output.contains("Derived from the User model"));
        assert!(output
            .contains(r#".read_only_field("id", DataType::Integer, FieldMeaning::Identifier)"#));
        assert!(output
            .contains(r#".field("name", DataType::String, FieldMeaning::Custom("name".into()))"#));
        assert!(output.contains(r#".field("email", DataType::String, FieldMeaning::Email)"#));
        assert!(output.contains(
            r#".read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)"#
        ));
        assert!(output.contains(
            r#".read_only_field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)"#
        ));
    }

    #[test]
    fn test_model_aware_excludes_sensitive() {
        let tmp = TempDir::new().unwrap();
        let models_dir = tmp.path().join("models");
        write_mock_model(
            &models_dir,
            "user.rs",
            r#"
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub password_hash: String,
    pub remember_token: Option<String>,
}
"#,
        );

        let models = scan_models_from_dir(&models_dir);
        let (_, fields) = &models[0];
        let output = model_aware_template("user", "User", fields);

        assert!(output.contains(r#".read_only_field("id""#));
        assert!(output.contains(r#".field("name""#));
        assert!(!output.contains("password_hash"));
        assert!(!output.contains("remember_token"));
    }

    #[test]
    fn test_model_aware_foreign_keys() {
        let tmp = TempDir::new().unwrap();
        let models_dir = tmp.path().join("models");
        write_mock_model(
            &models_dir,
            "order.rs",
            r#"
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub total: f64,
}
"#,
        );

        let models = scan_models_from_dir(&models_dir);
        let (_, fields) = &models[0];
        let output = model_aware_template("order", "Order", fields);

        assert!(output.contains(
            r#".read_only_field("user_id", DataType::Integer, FieldMeaning::ForeignKey)"#
        ));
        assert!(output.contains(r#".belongs_to("user", "user")"#));
    }

    #[test]
    fn test_model_aware_optional_fields() {
        let tmp = TempDir::new().unwrap();
        let models_dir = tmp.path().join("models");
        write_mock_model(
            &models_dir,
            "post.rs",
            r#"
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub notes: Option<String>,
}
"#,
        );

        let models = scan_models_from_dir(&models_dir);
        let (_, fields) = &models[0];
        let output = model_aware_template("post", "Post", fields);

        assert!(output.contains(
            r#".optional_field("notes", DataType::String, FieldMeaning::Custom("notes".into()))"#
        ));
    }

    #[test]
    fn test_rust_type_to_data_type() {
        assert_eq!(rust_type_to_data_type("String"), "DataType::String");
        assert_eq!(rust_type_to_data_type("i32"), "DataType::Integer");
        assert_eq!(rust_type_to_data_type("i64"), "DataType::Integer");
        assert_eq!(rust_type_to_data_type("u32"), "DataType::Integer");
        assert_eq!(rust_type_to_data_type("f32"), "DataType::Float");
        assert_eq!(rust_type_to_data_type("f64"), "DataType::Float");
        assert_eq!(rust_type_to_data_type("bool"), "DataType::Boolean");
        assert_eq!(rust_type_to_data_type("DateTime"), "DataType::DateTime");
        assert_eq!(
            rust_type_to_data_type("NaiveDateTime"),
            "DataType::DateTime"
        );
        assert_eq!(rust_type_to_data_type("NaiveDate"), "DataType::Date");
        assert_eq!(rust_type_to_data_type("Uuid"), "DataType::Uuid");
        assert_eq!(rust_type_to_data_type("Vec<u8>"), "DataType::Binary");
        assert_eq!(rust_type_to_data_type("Json"), "DataType::Json");
        assert_eq!(rust_type_to_data_type("Option<String>"), "DataType::String");
        assert_eq!(rust_type_to_data_type("Option<i32>"), "DataType::Integer");
        assert_eq!(
            rust_type_to_data_type("SomeUnknownType"),
            "DataType::String"
        );
    }

    #[test]
    fn test_infer_field_meaning() {
        assert_eq!(infer_meaning("id"), "FieldMeaning::Identifier");
        assert_eq!(infer_meaning("email"), "FieldMeaning::Email");
        assert_eq!(infer_meaning("created_at"), "FieldMeaning::CreatedAt");
        assert_eq!(infer_meaning("updated_at"), "FieldMeaning::UpdatedAt");
        assert_eq!(infer_meaning("user_id"), "FieldMeaning::ForeignKey");
        assert_eq!(infer_meaning("deleted_at"), "FieldMeaning::DateTime");
        assert_eq!(infer_meaning("is_active"), "FieldMeaning::Boolean");
        assert_eq!(infer_meaning("has_premium"), "FieldMeaning::Boolean");
        assert_eq!(infer_meaning("password"), "FieldMeaning::Sensitive");
        assert_eq!(infer_meaning("remember_token"), "FieldMeaning::Sensitive");
        assert_eq!(infer_meaning("title"), "custom");
    }
}
