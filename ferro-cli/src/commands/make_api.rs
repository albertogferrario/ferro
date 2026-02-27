//! `ferro make:api` command — scaffolds a complete REST API layer for existing models.
//!
//! Generates CRUD controllers, API resources, request validation types,
//! route registration with API key middleware, OpenAPI docs endpoint,
//! and API key migration.

use console::style;
use quote::ToTokens;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, Fields, ItemStruct, Type};
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Model metadata types
// ---------------------------------------------------------------------------

/// Parsed model information extracted from source files via syn.
#[derive(Debug, Clone)]
struct ModelInfo {
    /// PascalCase struct name (e.g., "User")
    name: String,
    /// Table name from `#[sea_orm(table_name = "...")]`
    table_name: Option<String>,
    /// All struct fields
    fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
struct FieldInfo {
    name: String,
    rust_type: String,
    is_primary_key: bool,
    is_nullable: bool,
}

// ---------------------------------------------------------------------------
// AST visitor for model detection
// ---------------------------------------------------------------------------

struct ModelVisitor {
    models: Vec<ModelInfo>,
}

impl ModelVisitor {
    fn new() -> Self {
        Self { models: Vec::new() }
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

    fn extract_table_name(attrs: &[Attribute]) -> Option<String> {
        for attr in attrs {
            if attr.path().is_ident("sea_orm") {
                if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args::<syn::Meta>() {
                    if nv.path.is_ident("table_name") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) = nv.value
                        {
                            return Some(s.value());
                        }
                    }
                }
            }
        }
        None
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

    fn extract_fields(fields: &Fields) -> Vec<FieldInfo> {
        let mut result = Vec::new();
        if let Fields::Named(named) = fields {
            for field in &named.named {
                if let Some(ident) = &field.ident {
                    let name = ident.to_string();
                    let rust_type = Self::type_to_string(&field.ty);
                    let is_nullable = rust_type.starts_with("Option<");
                    let is_primary_key = Self::is_field_primary_key(&field.attrs);
                    result.push(FieldInfo {
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
        if Self::has_model_derive(&node.attrs) {
            let name = node.ident.to_string();
            // The struct is typically "Model"; skip unless it's an entity model
            // (DeriveEntityModel is on the "Model" struct inside the entity module)
            if name == "Model" {
                // Capture parent module name from attributes instead
                let table = Self::extract_table_name(&node.attrs);
                let fields = Self::extract_fields(&node.fields);
                self.models.push(ModelInfo {
                    name: name.clone(),
                    table_name: table,
                    fields,
                });
            }
        }
        syn::visit::visit_item_struct(self, node);
    }
}

// ---------------------------------------------------------------------------
// Model scanning
// ---------------------------------------------------------------------------

/// Scan `src/models/` for model files and extract metadata via syn AST parsing.
fn scan_models(project_root: &Path) -> Vec<(String, ModelInfo)> {
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

        // Skip mod.rs
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

        for mut model in visitor.models {
            // Derive the PascalCase model name from the file name
            let pascal_name = to_pascal_case(&file_stem);
            model.name = pascal_name.clone();
            // If no table name was extracted, derive from file name
            if model.table_name.is_none() {
                model.table_name = Some(pluralize(&file_stem));
            }
            results.push((file_stem.clone(), model));
        }
    }

    results
}

/// Resolve which models to generate API for.
fn resolve_models(
    requested: &[String],
    all: bool,
    available: &[(String, ModelInfo)],
) -> Vec<(String, ModelInfo)> {
    if all {
        return available.to_vec();
    }

    let mut resolved = Vec::new();
    for name in requested {
        let snake = to_snake_case(name);
        let pascal = to_pascal_case(&snake);
        if let Some(found) = available
            .iter()
            .find(|(sn, mi)| *sn == snake || mi.name == pascal || mi.name == *name)
        {
            resolved.push(found.clone());
        } else {
            eprintln!(
                "{} Model '{}' not found in src/models/",
                style("Error:").red().bold(),
                name
            );
            std::process::exit(1);
        }
    }
    resolved
}

// ---------------------------------------------------------------------------
// Per-model code generation
// ---------------------------------------------------------------------------

/// Generate the API controller for a model.
fn generate_controller(snake_name: &str, model: &ModelInfo) {
    let api_dir = Path::new("src/api");
    if !api_dir.exists() {
        fs::create_dir_all(api_dir).expect("Failed to create src/api/ directory");
    }

    let file_path = api_dir.join(format!("{snake_name}_api.rs"));
    if file_path.exists() {
        println!(
            "   {} src/api/{snake_name}_api.rs (already exists)",
            style("skip").yellow()
        );
        return;
    }

    let pascal = &model.name;
    let plural_default = pluralize(snake_name);
    let plural = model.table_name.as_deref().unwrap_or(&plural_default);

    // Build set_field calls for store (non-PK, non-auto fields)
    let store_fields = build_store_fields(&model.fields);
    // Build optional set_field calls for update
    let update_fields = build_update_fields(&model.fields);

    let content = format!(
        r#"//! {pascal} API controller
//!
//! Generated with `ferro make:api`

use ferro::{{handler, Request, Response, HttpResponse}};
use crate::models::{snake_name}::{{self, Entity as {pascal}}};
use sea_orm::{{EntityTrait, PaginatorTrait}};
use crate::resources::{snake_name}_resource::{pascal}Resource;
use crate::requests::{snake_name}_request::{{Create{pascal}Request, Update{pascal}Request}};

/// List {plural} with pagination
///
/// GET /api/v1/{plural}
#[handler]
pub async fn index(req: Request) -> Response {{
    let page: u64 = req.query("page").unwrap_or(1);
    let per_page: u64 = req.query("per_page").unwrap_or(15).min(100);
    let db = ferro::DB::connection()
        .map_err(|e| HttpResponse::json(serde_json::json!({{"error": e.to_string()}})).status(500))?;
    let paginator = {pascal}::find().paginate(&db, per_page);
    let total = paginator
        .num_items()
        .await
        .map_err(|e| HttpResponse::json(serde_json::json!({{"error": e.to_string()}})).status(500))?;
    let items = paginator
        .fetch_page(page - 1)
        .await
        .map_err(|e| HttpResponse::json(serde_json::json!({{"error": e.to_string()}})).status(500))?;
    let resources: Vec<{pascal}Resource> = items.iter().map(|m| {pascal}Resource::from(m)).collect();
    let meta = ferro::PaginationMeta::new(page, per_page, total);
    Ok(ferro::ResourceCollection::paginated(resources, meta).to_response(&req))
}}

/// Show a single {snake_name}
///
/// GET /api/v1/{plural}/{{id}}
#[handler]
pub async fn show(req: Request, {snake_name}: {snake_name}::Model) -> Response {{
    Ok(ferro::Resource::to_wrapped_response(&{pascal}Resource::from(&{snake_name}), &req))
}}

/// Create a new {snake_name}
///
/// POST /api/v1/{plural}
#[handler]
pub async fn store(req: Request, form: Create{pascal}Request) -> Response {{
    let model = {pascal}::create()
{store_fields}        .insert()
        .await
        .map_err(|e| HttpResponse::json(serde_json::json!({{"error": e.to_string()}})).status(500))?;
    Ok(ferro::Resource::to_wrapped_response(&{pascal}Resource::from(&model), &req).status(201))
}}

/// Update an existing {snake_name}
///
/// PUT /api/v1/{plural}/{{id}}
#[handler]
pub async fn update(req: Request, {snake_name}: {snake_name}::Model, form: Update{pascal}Request) -> Response {{
    let mut builder = {snake_name}.update();
{update_fields}    let updated = builder
        .save()
        .await
        .map_err(|e| HttpResponse::json(serde_json::json!({{"error": e.to_string()}})).status(500))?;
    Ok(ferro::Resource::to_wrapped_response(&{pascal}Resource::from(&updated), &req))
}}

/// Delete a {snake_name}
///
/// DELETE /api/v1/{plural}/{{id}}
#[handler]
pub async fn destroy({snake_name}: {snake_name}::Model) -> Response {{
    {snake_name}
        .delete()
        .await
        .map_err(|e| HttpResponse::json(serde_json::json!({{"error": e.to_string()}})).status(500))?;
    Ok(HttpResponse::json(serde_json::json!({{"message": "Deleted"}})).status(200))
}}
"#,
    );

    fs::write(&file_path, content).expect("Failed to write API controller file");
    println!(
        "   {} Created src/api/{snake_name}_api.rs",
        style("✓").green()
    );
}

/// Generate the API resource for a model.
fn generate_resource(snake_name: &str, model: &ModelInfo) {
    let resources_dir = Path::new("src/resources");
    if !resources_dir.exists() {
        fs::create_dir_all(resources_dir).expect("Failed to create src/resources/ directory");
    }

    let file_path = resources_dir.join(format!("{snake_name}_resource.rs"));
    if file_path.exists() {
        println!(
            "   {} src/resources/{snake_name}_resource.rs (already exists)",
            style("skip").yellow()
        );
        return;
    }

    let pascal = &model.name;

    // Build resource fields and From impl assignments
    let resource_fields = build_resource_fields(&model.fields);
    let from_assignments = build_from_assignments(&model.fields);

    let content = format!(
        r#"//! {pascal} API resource
//!
//! Generated with `ferro make:api`

use ferro::{{Resource, ResourceMap, Request}};
use crate::models::{snake_name};

/// API representation of {pascal}.
pub struct {pascal}Resource {{
{resource_fields}
}}

impl Resource for {pascal}Resource {{
    fn to_resource(&self, _req: &Request) -> serde_json::Value {{
        let mut map = ResourceMap::new();
{from_assignments}        map.build()
    }}
}}

impl From<&{snake_name}::Model> for {pascal}Resource {{
    fn from(model: &{snake_name}::Model) -> Self {{
        Self {{
{model_to_resource}        }}
    }}
}}
"#,
        model_to_resource = build_model_to_resource(&model.fields),
    );

    fs::write(&file_path, content).expect("Failed to write API resource file");
    println!(
        "   {} Created src/resources/{snake_name}_resource.rs",
        style("✓").green()
    );
}

/// Generate request types for a model.
fn generate_request(snake_name: &str, model: &ModelInfo) {
    let requests_dir = Path::new("src/requests");
    if !requests_dir.exists() {
        fs::create_dir_all(requests_dir).expect("Failed to create src/requests/ directory");
    }

    let file_path = requests_dir.join(format!("{snake_name}_request.rs"));
    if file_path.exists() {
        println!(
            "   {} src/requests/{snake_name}_request.rs (already exists)",
            style("skip").yellow()
        );
        return;
    }

    let pascal = &model.name;

    let create_fields = build_create_request_fields(&model.fields);
    let update_fields = build_update_request_fields(&model.fields);

    let content = format!(
        r#"//! {pascal} API request types
//!
//! Generated with `ferro make:api`

use ferro::request;

/// Request body for creating a new {pascal}.
#[request]
pub struct Create{pascal}Request {{
{create_fields}}}

/// Request body for updating an existing {pascal} (all fields optional).
#[request]
pub struct Update{pascal}Request {{
{update_fields}}}
"#,
    );

    fs::write(&file_path, content).expect("Failed to write API request file");
    println!(
        "   {} Created src/requests/{snake_name}_request.rs",
        style("✓").green()
    );
}

// ---------------------------------------------------------------------------
// Field mapping helpers
// ---------------------------------------------------------------------------

/// Fields to skip in generated request/store/update code.
fn is_auto_field(field: &FieldInfo) -> bool {
    field.is_primary_key
        || field.name == "created_at"
        || field.name == "updated_at"
        || field.name == "deleted_at"
}

/// Build `.set_field(form.field)` lines for the store handler.
fn build_store_fields(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .filter(|f| !is_auto_field(f))
        .map(|f| format!("        .set_{}(form.{}.clone())\n", f.name, f.name))
        .collect()
}

/// Build conditional set_field lines for the update handler.
fn build_update_fields(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .filter(|f| !is_auto_field(f))
        .map(|f| {
            format!(
                "    if let Some(ref v) = form.{name} {{ builder = builder.set_{name}(v.clone()); }}\n",
                name = f.name
            )
        })
        .collect()
}

/// Build struct field definitions for the resource.
fn build_resource_fields(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .map(|f| format!("    pub {}: {},", f.name, resource_rust_type(&f.rust_type)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build ResourceMap field calls for to_resource.
fn build_from_assignments(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .map(|f| {
            format!(
                "        map = map.field(\"{name}\", serde_json::json!(self.{name}));\n",
                name = f.name
            )
        })
        .collect()
}

/// Build From<&Model> field assignments.
fn build_model_to_resource(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .map(|f| format!("            {name}: model.{name}.clone(),\n", name = f.name))
        .collect()
}

/// Build create request struct fields with validation attributes.
fn build_create_request_fields(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .filter(|f| !is_auto_field(f))
        .map(|f| {
            let validation = validation_attr_for_field(f, false);
            let rust_type = request_rust_type(&f.rust_type, f.is_nullable);
            if validation.is_empty() {
                format!("    pub {}: {},\n", f.name, rust_type)
            } else {
                format!("    {}\n    pub {}: {},\n", validation, f.name, rust_type)
            }
        })
        .collect()
}

/// Build update request struct fields (all optional).
fn build_update_request_fields(fields: &[FieldInfo]) -> String {
    fields
        .iter()
        .filter(|f| !is_auto_field(f))
        .map(|f| {
            let inner = request_rust_type(&f.rust_type, false);
            format!("    pub {}: Option<{}>,\n", f.name, inner)
        })
        .collect()
}

/// Map a Rust type from the model to a suitable resource field type.
fn resource_rust_type(rust_type: &str) -> String {
    // Strip Option wrapper if present for display, keep as-is
    rust_type.to_string()
}

/// Map a model's Rust type to a request field type.
fn request_rust_type(rust_type: &str, is_nullable: bool) -> String {
    if is_nullable {
        // Already Option<T>, keep it
        return rust_type.to_string();
    }
    // Map DateTime types to String for request input
    if rust_type.contains("DateTime") || rust_type.contains("DateTimeUtc") {
        return "String".to_string();
    }
    if rust_type.contains("NaiveDate") || rust_type == "Date" {
        return "String".to_string();
    }
    rust_type.to_string()
}

/// Generate a validation attribute for a request field.
fn validation_attr_for_field(field: &FieldInfo, _is_update: bool) -> String {
    if field.is_nullable {
        return String::new();
    }

    let ty = &field.rust_type;

    // Email fields
    if field.name == "email" || field.name.ends_with("_email") {
        return "#[validate(email)]".to_string();
    }

    // String/text types
    if ty == "String" || ty.contains("String") && !ty.starts_with("Option") {
        return "#[validate(length(min = 1))]".to_string();
    }

    String::new()
}

// ---------------------------------------------------------------------------
// String utilities
// ---------------------------------------------------------------------------

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn to_pascal_case(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn pluralize(name: &str) -> String {
    if name.ends_with('s') || name.ends_with('x') || name.ends_with("ch") || name.ends_with("sh") {
        format!("{name}es")
    } else if name.ends_with('y')
        && !name.ends_with("ay")
        && !name.ends_with("ey")
        && !name.ends_with("oy")
        && !name.ends_with("uy")
    {
        format!("{}ies", &name[..name.len() - 1])
    } else {
        format!("{name}s")
    }
}

// ---------------------------------------------------------------------------
// Public entry point (Task 1: model detection + per-model generation only)
// ---------------------------------------------------------------------------

/// Run the `make:api` command.
///
/// Generates API controllers, resources, and request types for the specified
/// models (or all models if `--all` is set).
pub fn run(models: Vec<String>, all: bool, yes: bool) {
    if models.is_empty() && !all {
        eprintln!(
            "{} Specify model names or use --all to scaffold API for all models",
            style("Error:").red().bold()
        );
        eprintln!("  Usage: ferro make:api User Post");
        eprintln!("  Usage: ferro make:api --all");
        std::process::exit(1);
    }

    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let available = scan_models(&project_root);

    if available.is_empty() {
        eprintln!(
            "{} No models found in src/models/. Create models first with `ferro make:scaffold`.",
            style("Error:").red().bold()
        );
        std::process::exit(1);
    }

    let selected = resolve_models(&models, all, &available);

    if selected.is_empty() {
        eprintln!("{} No matching models found", style("Error:").red().bold());
        std::process::exit(1);
    }

    // Confirmation prompt unless --yes
    if !yes {
        let names: Vec<&str> = selected.iter().map(|(_, m)| m.name.as_str()).collect();
        println!(
            "\n{} Scaffold API for: {}",
            style("?").cyan().bold(),
            names.join(", ")
        );
        let confirmed = dialoguer::Confirm::new()
            .with_prompt("Proceed with generation?")
            .default(true)
            .interact()
            .unwrap_or(false);
        if !confirmed {
            println!("Aborted.");
            return;
        }
    }

    println!(
        "\n{} Generating API scaffold...\n",
        style("▸").cyan().bold()
    );

    let mut generated_files: Vec<String> = Vec::new();

    for (snake_name, model) in &selected {
        println!("  {} {}", style("Model:").bold(), style(&model.name).cyan());
        generate_controller(snake_name, model);
        generate_resource(snake_name, model);
        generate_request(snake_name, model);
        generated_files.push(format!("src/api/{snake_name}_api.rs"));
        generated_files.push(format!("src/resources/{snake_name}_resource.rs"));
        generated_files.push(format!("src/requests/{snake_name}_request.rs"));
        println!();
    }

    // Generate infrastructure files
    generate_api_mod(&selected);
    generate_api_routes(&selected);
    generate_api_docs();
    generate_api_key_migration();
    generate_api_key_model();
    generate_api_key_provider();

    // Print summary
    let model_names: Vec<&str> = selected.iter().map(|(_, m)| m.name.as_str()).collect();
    println!(
        "\n{} API scaffold generated for: {}",
        style("✓").green().bold(),
        model_names.join(", ")
    );
    println!("\n  Generated files:");
    for (snake_name, _) in &selected {
        println!("    {}  src/api/{snake_name}_api.rs", style("—").dim());
        println!(
            "    {}  src/resources/{snake_name}_resource.rs",
            style("—").dim()
        );
        println!(
            "    {}  src/requests/{snake_name}_request.rs",
            style("—").dim()
        );
    }
    println!("    {}  src/api/mod.rs", style("—").dim());
    println!("    {}  src/api/routes.rs", style("—").dim());
    println!("    {}  src/api/docs.rs", style("—").dim());
    println!("    {}  src/models/api_key.rs", style("—").dim());
    println!(
        "    {}  src/providers/api_key_provider.rs",
        style("—").dim()
    );
    println!(
        "    {}  src/migrations/m..._create_api_keys_table.rs",
        style("—").dim()
    );

    println!("\n  Next steps:");
    println!("    1. Add `mod api;` to src/main.rs or src/lib.rs");
    println!("    2. Register api_routes() in your route configuration");
    println!("    3. Register docs_routes() for API documentation");
    println!("    4. Register ApiKeyProviderImpl as a service");
    println!("    5. Run `ferro db:migrate` to create api_keys table");
    println!("    6. Generate your first API key (see docs)");
    println!();
}

// ---------------------------------------------------------------------------
// Infrastructure file generation (called from run)
// ---------------------------------------------------------------------------

/// Generate src/api/mod.rs with module declarations.
fn generate_api_mod(models: &[(String, ModelInfo)]) {
    let api_dir = Path::new("src/api");
    if !api_dir.exists() {
        fs::create_dir_all(api_dir).expect("Failed to create src/api/ directory");
    }

    let mod_path = api_dir.join("mod.rs");
    if mod_path.exists() {
        // Append new model modules if they're not already declared
        let existing = fs::read_to_string(&mod_path).unwrap_or_default();
        let mut additions = String::new();
        for (snake_name, _) in models {
            let decl = format!("pub mod {snake_name}_api;");
            if !existing.contains(&decl) {
                additions.push_str(&decl);
                additions.push('\n');
            }
        }
        // Ensure routes and docs modules are declared
        if !existing.contains("pub mod routes;") {
            additions.push_str("pub mod routes;\n");
        }
        if !existing.contains("pub mod docs;") {
            additions.push_str("pub mod docs;\n");
        }
        if !additions.is_empty() {
            let updated = format!("{existing}{additions}");
            fs::write(&mod_path, updated).expect("Failed to update src/api/mod.rs");
            println!("   {} Updated src/api/mod.rs", style("✓").green());
        } else {
            println!(
                "   {} src/api/mod.rs (already up-to-date)",
                style("skip").yellow()
            );
        }
    } else {
        let mut content = String::from("// Auto-generated API modules\n");
        for (snake_name, _) in models {
            content.push_str(&format!("pub mod {snake_name}_api;\n"));
        }
        content.push_str("pub mod routes;\n");
        content.push_str("pub mod docs;\n");
        fs::write(&mod_path, content).expect("Failed to write src/api/mod.rs");
        println!("   {} Created src/api/mod.rs", style("✓").green());
    }
}

/// Generate src/api/routes.rs with route registration.
fn generate_api_routes(models: &[(String, ModelInfo)]) {
    let file_path = Path::new("src/api/routes.rs");
    if file_path.exists() {
        println!(
            "   {} src/api/routes.rs (already exists)",
            style("skip").yellow()
        );
        return;
    }

    let mut route_blocks = String::new();
    for (snake_name, model) in models {
        let plural_default = pluralize(snake_name);
        let plural = model.table_name.as_deref().unwrap_or(&plural_default);
        let pk = model
            .fields
            .iter()
            .find(|f| f.is_primary_key)
            .map(|f| f.name.as_str())
            .unwrap_or("id");

        route_blocks.push_str(&format!(
            r#"
            // {pascal} CRUD
            get!("/{plural}", {snake_name}_api::index).name("api.{plural}.index"),
            post!("/{plural}", {snake_name}_api::store).name("api.{plural}.store"),
            get!("/{plural}/:{pk}", {snake_name}_api::show).name("api.{plural}.show"),
            put!("/{plural}/:{pk}", {snake_name}_api::update).name("api.{plural}.update"),
            delete!("/{plural}/:{pk}", {snake_name}_api::destroy).name("api.{plural}.destroy"),
"#,
            pascal = model.name,
        ));
    }

    let content = format!(
        r#"//! API route registration
//!
//! Generated with `ferro make:api`

use ferro::*;
use crate::api::*;

pub fn api_routes() -> GroupBuilder {{
    group!("/api/v1")
        .middleware(ApiKeyMiddleware::new())
        .middleware(Throttle::named("api"))
        .routes([{route_blocks}        ])
}}
"#,
    );

    fs::write(file_path, content).expect("Failed to write src/api/routes.rs");
    println!("   {} Created src/api/routes.rs", style("✓").green());
}

/// Generate src/api/docs.rs with OpenAPI documentation handlers.
fn generate_api_docs() {
    let file_path = Path::new("src/api/docs.rs");
    if file_path.exists() {
        println!(
            "   {} src/api/docs.rs (already exists)",
            style("skip").yellow()
        );
        return;
    }

    let content = r#"//! API documentation routes
//!
//! Generated with `ferro make:api`

use ferro::*;

pub fn docs_routes() -> Vec<RouteDefBuilder> {
    vec![
        get!("/api/docs", api_docs).name("api.docs"),
        get!("/api/openapi.json", openapi_json).name("api.openapi"),
    ]
}

#[handler]
pub async fn api_docs() -> Response {
    let config = OpenApiConfig {
        title: ferro::env("APP_NAME", "API"),
        version: "1.0.0".to_string(),
        description: Some("Auto-generated API documentation".to_string()),
        api_prefix: "/api/".to_string(),
    };
    let routes = get_registered_routes();
    let resp = openapi_docs_response(&config, &routes);
    Ok(resp)
}

#[handler]
pub async fn openapi_json() -> Response {
    let config = OpenApiConfig {
        title: ferro::env("APP_NAME", "API"),
        version: "1.0.0".to_string(),
        description: Some("Auto-generated API documentation".to_string()),
        api_prefix: "/api/".to_string(),
    };
    let routes = get_registered_routes();
    let resp = openapi_json_response(&config, &routes);
    Ok(resp)
}
"#;

    fs::write(file_path, content).expect("Failed to write src/api/docs.rs");
    println!("   {} Created src/api/docs.rs", style("✓").green());
}

/// Generate the API keys migration.
fn generate_api_key_migration() {
    let migrations_dir = if Path::new("src/migrations").exists() {
        Path::new("src/migrations")
    } else if Path::new("src/database/migrations").exists() {
        Path::new("src/database/migrations")
    } else {
        println!(
            "   {} migrations directory not found, skipping migration generation",
            style("warn").yellow()
        );
        return;
    };

    // Check if migration already exists
    if let Ok(entries) = fs::read_dir(migrations_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("create_api_keys_table") {
                println!(
                    "   {} API keys migration (already exists)",
                    style("skip").yellow()
                );
                return;
            }
        }
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let migration_name = format!("m{timestamp}_create_api_keys_table");
    let file_name = format!("{migration_name}.rs");
    let file_path = migrations_dir.join(&file_name);

    let content = r#"use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ApiKeys::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ApiKeys::Name).string().not_null())
                    .col(ColumnDef::new(ApiKeys::Prefix).string_len(16).not_null())
                    .col(ColumnDef::new(ApiKeys::HashedKey).string_len(64).not_null())
                    .col(ColumnDef::new(ApiKeys::Scopes).text().null())
                    .col(ColumnDef::new(ApiKeys::LastUsedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(ApiKeys::ExpiresAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(ApiKeys::RevokedAt).timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new(ApiKeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_api_keys_prefix")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::Prefix)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiKeys::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ApiKeys {
    Table,
    Id,
    Name,
    Prefix,
    HashedKey,
    Scopes,
    LastUsedAt,
    ExpiresAt,
    RevokedAt,
    CreatedAt,
}
"#
    .to_string();

    fs::write(&file_path, content).expect("Failed to write migration file");

    // Update migrations mod.rs
    update_migrations_mod(&migration_name);

    println!(
        "   {} Created {}/{}",
        style("✓").green(),
        migrations_dir.display(),
        file_name
    );
}

/// Update the migrations mod.rs to include the new migration.
fn update_migrations_mod(migration_name: &str) {
    let mod_path = if Path::new("src/migrations/mod.rs").exists() {
        Path::new("src/migrations/mod.rs")
    } else if Path::new("src/database/migrations/mod.rs").exists() {
        Path::new("src/database/migrations/mod.rs")
    } else {
        return;
    };

    let content = fs::read_to_string(mod_path).unwrap_or_default();
    let mod_declaration = format!("pub mod {migration_name};");
    if content.contains(&mod_declaration) {
        return;
    }

    // Find where to insert (after existing pub mod m* lines)
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut insert_index = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("pub mod m") {
            insert_index = i + 1;
        }
    }
    lines.insert(insert_index, mod_declaration.clone());

    // Add to Migrator vec
    let migrator_addition = format!("            Box::new({migration_name}::Migration),");
    let mut result = lines.join("\n");

    if result.contains("vec![]") {
        result = result.replace("vec![]", &format!("vec![\n{migrator_addition}\n        ]"));
    } else if result.contains("vec![") {
        let mut final_result = String::new();
        let mut in_migrations = false;
        let mut bracket_depth = 0;

        for line in result.lines() {
            if line.contains("fn migrations()") {
                in_migrations = true;
            }
            if in_migrations {
                if line.contains("vec![") {
                    bracket_depth += 1;
                }
                if line.trim() == "]" && bracket_depth == 1 {
                    final_result.push_str(&migrator_addition);
                    final_result.push('\n');
                    bracket_depth = 0;
                    in_migrations = false;
                }
            }
            final_result.push_str(line);
            final_result.push('\n');
        }
        result = final_result;
    }

    fs::write(mod_path, result).expect("Failed to update migrations mod.rs");
}

/// Generate src/models/api_key.rs.
fn generate_api_key_model() {
    let models_dir = Path::new("src/models");
    if !models_dir.exists() {
        fs::create_dir_all(models_dir).expect("Failed to create src/models/ directory");
    }

    let file_path = models_dir.join("api_key.rs");
    if file_path.exists() {
        println!(
            "   {} src/models/api_key.rs (already exists)",
            style("skip").yellow()
        );
        return;
    }

    let content = r#"//! API key model

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "api_keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub prefix: String,
    pub hashed_key: String,
    pub scopes: Option<String>,
    pub last_used_at: Option<DateTimeUtc>,
    pub expires_at: Option<DateTimeUtc>,
    pub revoked_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl DatabaseModel for Entity {}
impl ModelMut for Entity {}

pub type ApiKey = Model;

impl Model {
    pub fn query() -> QueryBuilder<Entity> {
        QueryBuilder::new()
    }
}
"#;

    fs::write(&file_path, content).expect("Failed to write API key model file");

    // Update models mod.rs
    let mod_path = models_dir.join("mod.rs");
    if mod_path.exists() {
        let existing = fs::read_to_string(&mod_path).unwrap_or_default();
        if !existing.contains("pub mod api_key;") {
            let updated = format!("{existing}pub mod api_key;\npub use api_key::*;\n");
            fs::write(&mod_path, updated).expect("Failed to update models mod.rs");
        }
    }

    println!("   {} Created src/models/api_key.rs", style("✓").green());
}

/// Generate src/providers/api_key_provider.rs.
fn generate_api_key_provider() {
    let providers_dir = Path::new("src/providers");
    if !providers_dir.exists() {
        fs::create_dir_all(providers_dir).expect("Failed to create src/providers/ directory");
    }

    let file_path = providers_dir.join("api_key_provider.rs");
    if file_path.exists() {
        println!(
            "   {} src/providers/api_key_provider.rs (already exists)",
            style("skip").yellow()
        );
        return;
    }

    let content = r#"//! API key provider implementation
//!
//! Generated with `ferro make:api`

use ferro::{async_trait, ApiKeyInfo, ApiKeyProvider, verify_api_key_hash};
use crate::models::api_key::{self, Entity as ApiKey};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

/// Database-backed API key provider.
///
/// Register this as a service in your bootstrap:
/// ```rust,ignore
/// App::bind::<dyn ApiKeyProvider>(Box::new(ApiKeyProviderImpl));
/// ```
pub struct ApiKeyProviderImpl;

#[async_trait]
impl ApiKeyProvider for ApiKeyProviderImpl {
    async fn verify_key(&self, raw_key: &str) -> Result<ApiKeyInfo, ()> {
        let prefix = &raw_key[..16.min(raw_key.len())];

        let db = ferro::DB::connection().map_err(|_| ())?;
        let record = ApiKey::find()
            .filter(api_key::Column::Prefix.eq(prefix))
            .one(&db)
            .await
            .map_err(|_| ())?
            .ok_or(())?;

        // Check revocation
        if record.revoked_at.is_some() {
            return Err(());
        }

        // Check expiry
        if let Some(expires_at) = record.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(());
            }
        }

        // Constant-time hash verification
        if !verify_api_key_hash(raw_key, &record.hashed_key) {
            return Err(());
        }

        let scopes: Vec<String> = record
            .scopes
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        Ok(ApiKeyInfo {
            id: record.id,
            name: record.name,
            scopes,
        })
    }
}
"#;

    fs::write(&file_path, content).expect("Failed to write API key provider file");
    println!(
        "   {} Created src/providers/api_key_provider.rs",
        style("✓").green()
    );
}
