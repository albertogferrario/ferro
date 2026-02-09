use std::fs;
use std::path::Path;

use crate::analyzer::{
    FactoryPattern, ForeignKeyInfo, ProjectAnalyzer, ProjectConventions, TestPattern,
};
use crate::templates;
use dialoguer::Confirm;

/// Tracks which smart defaults were applied and why.
#[derive(Debug, Default)]
struct SmartDefaults {
    api_detected: bool,
    test_detected: bool,
    test_count: usize,
    factory_detected: bool,
    factory_count: usize,
    field_inferences: Vec<(String, String, String)>, // (name, type, reason)
}

impl SmartDefaults {
    fn has_any(&self) -> bool {
        self.api_detected
            || self.test_detected
            || self.factory_detected
            || !self.field_inferences.is_empty()
    }

    fn display(&self, api_only: bool, with_tests: bool, with_factory: bool) {
        println!("\n📊 Smart Defaults Detected:");
        println!("   ─────────────────────────");

        // Project type
        if self.api_detected {
            println!("   Project type: API-only (no Inertia pages found)");
        } else if api_only {
            println!("   Project type: API-only (explicit --api flag)");
        } else {
            println!("   Project type: Full-stack (Inertia pages present)");
        }

        // Test pattern
        if self.test_detected {
            println!(
                "   Test pattern: Per-controller ({} existing test files)",
                self.test_count
            );
        } else if with_tests {
            println!("   Test pattern: Enabled (explicit --with-tests flag)");
        }

        // Factory pattern
        if self.factory_detected {
            println!(
                "   Factory pattern: Per-model ({} existing factories)",
                self.factory_count
            );
        } else if with_factory {
            println!("   Factory pattern: Enabled (explicit --with-factory flag)");
        }

        // Applied flags summary
        let mut flags = Vec::new();
        if api_only {
            flags.push("--api");
        }
        if with_tests {
            flags.push("--with-tests");
        }
        if with_factory {
            flags.push("--with-factory");
        }
        if !flags.is_empty() {
            println!("\n   Applied flags: {}", flags.join(" "));
        }

        // Field inferences
        if !self.field_inferences.is_empty() {
            println!("\n   Field type inference:");
            for (name, field_type, reason) in &self.field_inferences {
                println!("     {} → {} ({})", name, field_type, reason);
            }
        }

        println!();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    name: String,
    fields: Vec<String>,
    with_tests: bool,
    with_factory: bool,
    auto_routes: bool,
    yes: bool,
    api_only: bool,
    no_smart_defaults: bool,
    quiet: bool,
) {
    // Track what smart defaults were applied
    let mut smart_defaults = SmartDefaults::default();

    // Apply smart defaults unless disabled
    let (api_only, with_tests, with_factory) = if no_smart_defaults {
        (api_only, with_tests, with_factory)
    } else {
        apply_smart_defaults(api_only, with_tests, with_factory, &mut smart_defaults)
    };

    // Validate resource name
    if !is_valid_identifier(&name) {
        eprintln!(
            "Error: '{}' is not a valid identifier. Use PascalCase (e.g., Post, UserProfile)",
            name
        );
        std::process::exit(1);
    }

    // Parse fields (with type inference tracking)
    let parsed_fields = match parse_fields(&fields, &mut smart_defaults, no_smart_defaults) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error parsing fields: {}", e);
            std::process::exit(1);
        }
    };

    let snake_name = to_snake_case(&name);
    let plural_snake = pluralize(&snake_name);

    // Display smart defaults summary (unless quiet or no smart defaults)
    if !quiet && !no_smart_defaults && smart_defaults.has_any() {
        smart_defaults.display(api_only, with_tests, with_factory);

        // Interactive confirmation unless --yes is passed
        if !yes {
            let confirmed = Confirm::new()
                .with_prompt("Proceed with generation?")
                .default(true)
                .interact()
                .unwrap_or(false);

            if !confirmed {
                println!("Aborted.");
                return;
            }
        }
    }

    println!("🚀 Scaffolding {}...\n", name);

    // Detect foreign keys from field names
    let analyzer = ProjectAnalyzer::current_dir();
    let field_tuples: Vec<(&str, &str)> = parsed_fields
        .iter()
        .map(|f| (f.name.as_str(), f.field_type.to_display_name()))
        .collect();
    let foreign_keys = analyzer.detect_foreign_keys(&field_tuples);

    // Generate migration
    generate_migration(
        &name,
        &snake_name,
        &plural_snake,
        &parsed_fields,
        &foreign_keys,
    );

    // Generate model (includes entity)
    generate_model(&name, &snake_name, &parsed_fields, &foreign_keys);

    // Generate controller
    generate_controller(
        &name,
        &snake_name,
        &plural_snake,
        &parsed_fields,
        &foreign_keys,
        api_only,
    );

    // Generate Inertia pages (skip for API-only scaffold)
    if !api_only {
        generate_inertia_pages(
            &name,
            &snake_name,
            &plural_snake,
            &parsed_fields,
            &foreign_keys,
        );
    }

    // Generate tests if requested
    if with_tests {
        generate_tests(
            &name,
            &snake_name,
            &plural_snake,
            &parsed_fields,
            with_factory,
        );
    }

    // Generate factory if requested
    if with_factory {
        generate_scaffold_factory(&name, &snake_name, &parsed_fields, &foreign_keys);
    }

    // Auto-register routes or print instructions
    if auto_routes {
        register_routes(&snake_name, &plural_snake, yes);
    } else {
        print_route_instructions(&name, &snake_name, &plural_snake);
    }

    if api_only {
        println!("\n✅ API scaffold for {} created successfully!", name);
    } else {
        println!("\n✅ Scaffold for {} created successfully!", name);
    }
}

#[derive(Debug, Clone)]
struct Field {
    name: String,
    field_type: FieldType,
}

#[derive(Debug, Clone)]
enum FieldType {
    String,
    Text,
    Integer,
    BigInteger,
    Float,
    Boolean,
    DateTime,
    Date,
    Uuid,
}

impl FieldType {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "string" | "str" => Ok(FieldType::String),
            "text" => Ok(FieldType::Text),
            "int" | "integer" | "i32" => Ok(FieldType::Integer),
            "bigint" | "biginteger" | "i64" => Ok(FieldType::BigInteger),
            "float" | "f64" | "double" => Ok(FieldType::Float),
            "bool" | "boolean" => Ok(FieldType::Boolean),
            "datetime" | "timestamp" => Ok(FieldType::DateTime),
            "date" => Ok(FieldType::Date),
            "uuid" => Ok(FieldType::Uuid),
            _ => Err(format!("Unknown field type: '{}'. Valid types: string, text, integer, bigint, float, bool, datetime, date, uuid", s)),
        }
    }

    fn to_display_name(&self) -> &'static str {
        match self {
            FieldType::String => "string",
            FieldType::Text => "text",
            FieldType::Integer => "integer",
            FieldType::BigInteger => "bigint",
            FieldType::Float => "float",
            FieldType::Boolean => "bool",
            FieldType::DateTime => "datetime",
            FieldType::Date => "date",
            FieldType::Uuid => "uuid",
        }
    }

    fn to_rust_type(&self) -> &'static str {
        match self {
            FieldType::String => "String",
            FieldType::Text => "String",
            FieldType::Integer => "i32",
            FieldType::BigInteger => "i64",
            FieldType::Float => "f64",
            FieldType::Boolean => "bool",
            FieldType::DateTime => "chrono::DateTime<chrono::Utc>",
            FieldType::Date => "chrono::NaiveDate",
            FieldType::Uuid => "uuid::Uuid",
        }
    }

    fn to_sea_orm_method(&self) -> &'static str {
        match self {
            FieldType::String => "string()",
            FieldType::Text => "text()",
            FieldType::Integer => "integer()",
            FieldType::BigInteger => "big_integer()",
            FieldType::Float => "double()",
            FieldType::Boolean => "boolean()",
            FieldType::DateTime => "timestamp_with_time_zone()",
            FieldType::Date => "date()",
            FieldType::Uuid => "uuid()",
        }
    }

    fn to_typescript_type(&self) -> &'static str {
        match self {
            FieldType::String => "string",
            FieldType::Text => "string",
            FieldType::Integer => "number",
            FieldType::BigInteger => "number",
            FieldType::Float => "number",
            FieldType::Boolean => "boolean",
            FieldType::DateTime => "string",
            FieldType::Date => "string",
            FieldType::Uuid => "string",
        }
    }

    fn to_form_input_type(&self) -> &'static str {
        match self {
            FieldType::String => "text",
            FieldType::Text => "textarea",
            FieldType::Integer => "number",
            FieldType::BigInteger => "number",
            FieldType::Float => "number",
            FieldType::Boolean => "checkbox",
            FieldType::DateTime => "datetime-local",
            FieldType::Date => "date",
            FieldType::Uuid => "text",
        }
    }

    fn to_validation_attr(&self) -> &'static str {
        match self {
            FieldType::String => "#[rule(required, string)]",
            FieldType::Text => "#[rule(required, string)]",
            FieldType::Integer => "#[rule(required, integer)]",
            FieldType::BigInteger => "#[rule(required, integer)]",
            FieldType::Float => "#[rule(required, numeric)]",
            FieldType::Boolean => "#[rule(required, boolean)]",
            FieldType::DateTime => "#[rule(required, date)]",
            FieldType::Date => "#[rule(required, date)]",
            FieldType::Uuid => "#[rule(required, string)]",
        }
    }

    fn to_scaffold_type(&self) -> &'static str {
        match self {
            FieldType::String => "string",
            FieldType::Text => "text",
            FieldType::Integer => "integer",
            FieldType::BigInteger => "bigint",
            FieldType::Float => "float",
            FieldType::Boolean => "bool",
            FieldType::DateTime => "datetime",
            FieldType::Date => "date",
            FieldType::Uuid => "uuid",
        }
    }
}

fn parse_fields(
    fields: &[String],
    tracking: &mut SmartDefaults,
    no_smart_defaults: bool,
) -> Result<Vec<Field>, String> {
    let mut parsed = Vec::new();

    for field_str in fields {
        let parts: Vec<&str> = field_str.split(':').collect();

        let (name, field_type) = match parts.len() {
            1 => {
                // Just field name, infer type from naming convention
                let name = parts[0].to_string();
                if !is_valid_field_name(&name) {
                    return Err(format!(
                        "Invalid field name: '{}'. Use snake_case (e.g., user_id)",
                        name
                    ));
                }
                let (field_type, reason) = infer_field_type(&name);
                if !no_smart_defaults {
                    tracking.field_inferences.push((
                        name.clone(),
                        field_type.to_display_name().to_string(),
                        reason.to_string(),
                    ));
                }
                (name, field_type)
            }
            2 => {
                // Explicit name:type format
                let name = parts[0].to_string();
                if !is_valid_field_name(&name) {
                    return Err(format!(
                        "Invalid field name: '{}'. Use snake_case (e.g., user_id)",
                        name
                    ));
                }
                let field_type = FieldType::from_str(parts[1])?;
                (name, field_type)
            }
            _ => {
                return Err(format!(
                    "Invalid field format: '{}'. Expected format: name or name:type (e.g., title or title:string)",
                    field_str
                ));
            }
        };

        parsed.push(Field { name, field_type });
    }

    Ok(parsed)
}

/// Infer field type from naming conventions.
///
/// Returns (FieldType, reason) tuple.
///
/// Patterns:
/// - `*_id` -> bigint (foreign key)
/// - `*_at` -> datetime (timestamp)
/// - `email` -> string
/// - `password` -> string
/// - `is_*` or `has_*` -> bool
/// - default -> string
fn infer_field_type(name: &str) -> (FieldType, &'static str) {
    // Foreign key pattern
    if name.ends_with("_id") {
        return (FieldType::BigInteger, "foreign key pattern");
    }

    // Timestamp pattern
    if name.ends_with("_at") {
        return (FieldType::DateTime, "timestamp pattern");
    }

    // Boolean patterns
    if name.starts_with("is_") || name.starts_with("has_") {
        return (FieldType::Boolean, "boolean pattern");
    }

    // Common field names
    match name {
        "email" => (FieldType::String, "common field"),
        "password" => (FieldType::String, "hashed field"),
        _ => (FieldType::String, "default"),
    }
}

fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_uppercase() {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_valid_field_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_lowercase() && first != '_' {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

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
    // Simple pluralization rules
    if name.ends_with('s') || name.ends_with('x') || name.ends_with("ch") || name.ends_with("sh") {
        format!("{}es", name)
    } else if name.ends_with('y')
        && !name.ends_with("ay")
        && !name.ends_with("ey")
        && !name.ends_with("oy")
        && !name.ends_with("uy")
    {
        format!("{}ies", &name[..name.len() - 1])
    } else {
        format!("{}s", name)
    }
}

fn generate_migration(
    _name: &str,
    _snake_name: &str,
    plural_snake: &str,
    fields: &[Field],
    foreign_keys: &[ForeignKeyInfo],
) {
    // Check for both possible migration directory locations
    let migrations_dir = if Path::new("src/migrations").exists() {
        Path::new("src/migrations")
    } else if Path::new("src/database/migrations").exists() {
        Path::new("src/database/migrations")
    } else {
        eprintln!("Error: migrations directory not found. Are you in a Ferro project?");
        eprintln!("Expected: src/migrations or src/database/migrations");
        std::process::exit(1);
    };

    // Generate timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let migration_name = format!("m{}_create_{}_table", timestamp, plural_snake);
    let file_name = format!("{}.rs", migration_name);
    let file_path = migrations_dir.join(&file_name);

    // Build column definitions
    let mut columns = String::new();
    for field in fields {
        columns.push_str(&format!(
            "            .col(ColumnDef::new({name}::{column}).{method}.not_null())\n",
            name = to_pascal_case(plural_snake),
            column = to_pascal_case(&field.name),
            method = field.field_type.to_sea_orm_method()
        ));
    }

    // Build foreign key constraints for validated FKs only
    let mut fk_constraints = String::new();
    let mut fk_comments = String::new();
    for fk in foreign_keys {
        if fk.validated {
            fk_constraints.push_str(&format!(
                r#"            .foreign_key(
                ForeignKey::create()
                    .name("fk_{table}_{field}")
                    .from({table_enum}::Table, {table_enum}::{column})
                    .to({target_table_enum}::Table, {target_table_enum}::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
"#,
                table = plural_snake,
                field = fk.field_name,
                table_enum = to_pascal_case(plural_snake),
                column = to_pascal_case(&fk.field_name),
                target_table_enum = to_pascal_case(&fk.target_table),
            ));
        } else {
            fk_comments.push_str(&format!(
                "// Note: {} model not found - FK constraint for {} skipped\n",
                fk.target_model, fk.field_name
            ));
        }
    }

    // Build FK table enum imports if we have validated FKs
    let fk_table_enums: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            format!(
                r#"
/// Reference to {target_table} table for FK constraint
#[derive(Iden)]
pub enum {target_table_enum} {{
    Table,
    Id,
}}
"#,
                target_table = fk.target_table,
                target_table_enum = to_pascal_case(&fk.target_table),
            )
        })
        .collect();

    let migration_content = format!(
        r#"use sea_orm_migration::prelude::*;

{fk_comments}#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .create_table(
                Table::create()
                    .table({table_enum}::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new({table_enum}::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
{columns}{fk_constraints}                    .col(
                        ColumnDef::new({table_enum}::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new({table_enum}::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .drop_table(Table::drop().table({table_enum}::Table).to_owned())
            .await
    }}
}}

#[derive(Iden)]
pub enum {table_enum} {{
    Table,
    Id,
{iden_columns}    CreatedAt,
    UpdatedAt,
}}
{fk_table_enums}"#,
        table_enum = to_pascal_case(plural_snake),
        columns = columns,
        fk_constraints = fk_constraints,
        fk_comments = fk_comments,
        iden_columns = fields
            .iter()
            .map(|f| format!("    {},\n", to_pascal_case(&f.name)))
            .collect::<String>(),
        fk_table_enums = fk_table_enums,
    );

    fs::write(&file_path, migration_content).expect("Failed to write migration file");

    // Update mod.rs
    update_migrations_mod(&migration_name);

    println!(
        "   📦 Created migration: {}/{}",
        migrations_dir.display(),
        file_name
    );
}

fn update_migrations_mod(migration_name: &str) {
    // Check for both possible mod.rs locations
    let mod_path = if Path::new("src/migrations/mod.rs").exists() {
        Path::new("src/migrations/mod.rs")
    } else if Path::new("src/database/migrations/mod.rs").exists() {
        Path::new("src/database/migrations/mod.rs")
    } else {
        eprintln!("Warning: migrations/mod.rs not found");
        return;
    };

    let content = fs::read_to_string(mod_path).expect("Failed to read mod.rs");

    // Add module declaration
    let mod_declaration = format!("pub mod {};", migration_name);
    if content.contains(&mod_declaration) {
        return;
    }

    // Find where to insert module declaration (after existing pub mod lines)
    let mut lines: Vec<&str> = content.lines().collect();
    let mut insert_index = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("pub mod m") {
            insert_index = i + 1;
        }
    }

    lines.insert(insert_index, &mod_declaration);

    // Also add to Migrator
    let migrator_addition = format!("            Box::new({}::Migration),", migration_name);

    let mut updated_lines = Vec::new();
    for line in lines {
        updated_lines.push(line.to_string());
        if line.contains("fn migrations()") {
            // Find the vec![ line and add before closing ]
        }
    }

    // Simple approach: find "]" line in migrations() and insert before it
    let content = updated_lines.join("\n");
    let content = if content.contains("vec![]") {
        content.replace(
            "vec![]",
            &format!("vec![\n{}\n        ]", migrator_addition),
        )
    } else if content.contains("vec![") {
        // Find last ] in migrations function and insert before it
        let mut result = String::new();
        let mut in_migrations = false;
        let mut bracket_depth = 0;

        for line in content.lines() {
            if line.contains("fn migrations()") {
                in_migrations = true;
            }

            if in_migrations {
                if line.contains("vec![") {
                    bracket_depth += 1;
                }
                if line.trim() == "]" && bracket_depth == 1 {
                    result.push_str(&migrator_addition);
                    result.push('\n');
                    bracket_depth = 0;
                    in_migrations = false;
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        result
    } else {
        content
    };

    fs::write(mod_path, content).expect("Failed to write mod.rs");
}

fn generate_model(name: &str, snake_name: &str, fields: &[Field], foreign_keys: &[ForeignKeyInfo]) {
    let models_dir = Path::new("src/models");

    if !models_dir.exists() {
        fs::create_dir_all(models_dir).expect("Failed to create models directory");
    }

    let file_path = models_dir.join(format!("{}.rs", snake_name));

    // Build field definitions for the entity
    let mut field_defs = String::new();
    for field in fields {
        field_defs.push_str(&format!(
            "    pub {}: {},\n",
            field.name,
            field.field_type.to_rust_type()
        ));
    }

    // Build Relation enum variants for validated FKs
    let validated_fks: Vec<&ForeignKeyInfo> =
        foreign_keys.iter().filter(|fk| fk.validated).collect();

    let relation_enum = if validated_fks.is_empty() {
        "#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]\npub enum Relation {}".to_string()
    } else {
        let variants: String = validated_fks
            .iter()
            .map(|fk| {
                let target_snake = to_snake_case(&fk.target_model);
                format!(
                    r#"    #[sea_orm(
        belongs_to = "super::{target_snake}::Entity",
        from = "Column::{fk_column}",
        to = "super::{target_snake}::Column::Id"
    )]
    {target_pascal},
"#,
                    target_snake = target_snake,
                    fk_column = to_pascal_case(&fk.field_name),
                    target_pascal = fk.target_model,
                )
            })
            .collect();

        format!(
            "#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]\npub enum Relation {{\n{}}}\n",
            variants
        )
    };

    // Build Related<T> impls for validated FKs
    let related_impls: String = validated_fks
        .iter()
        .map(|fk| {
            let target_snake = to_snake_case(&fk.target_model);
            format!(
                r#"
impl Related<super::{target_snake}::Entity> for Entity {{
    fn to() -> RelationDef {{
        Relation::{target_pascal}.def()
    }}
}}
"#,
                target_snake = target_snake,
                target_pascal = fk.target_model,
            )
        })
        .collect();

    let model_content = format!(
        r#"//! {name} model

use ferro::database::{{Model as DatabaseModel, ModelMut, QueryBuilder}};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "{table_name}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i64,
{field_defs}    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}}

{relation_enum}

impl ActiveModelBehavior for ActiveModel {{}}

impl DatabaseModel for Entity {{}}
impl ModelMut for Entity {{}}
{related_impls}
/// Type alias for convenient access
pub type {name} = Model;

impl Model {{
    /// Start a query builder
    pub fn query() -> QueryBuilder<Entity> {{
        QueryBuilder::new()
    }}
}}
"#,
        name = name,
        table_name = pluralize(snake_name),
        field_defs = field_defs,
        relation_enum = relation_enum,
        related_impls = related_impls,
    );

    fs::write(&file_path, model_content).expect("Failed to write model file");

    // Update mod.rs
    update_models_mod(snake_name);

    println!("   📦 Created model: src/models/{}.rs", snake_name);
}

fn update_models_mod(snake_name: &str) {
    let mod_path = Path::new("src/models/mod.rs");

    if !mod_path.exists() {
        let content = format!(
            "pub mod {snake};\npub use {snake}::*;\n",
            snake = snake_name
        );
        fs::write(mod_path, content).expect("Failed to write mod.rs");
        return;
    }

    let content = fs::read_to_string(mod_path).expect("Failed to read mod.rs");
    let mod_declaration = format!("pub mod {};", snake_name);

    if content.contains(&mod_declaration) {
        return;
    }

    let updated = format!(
        "{}{}\npub use {}::*;\n",
        content, mod_declaration, snake_name
    );
    fs::write(mod_path, updated).expect("Failed to write mod.rs");
}

fn generate_controller(
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    fields: &[Field],
    foreign_keys: &[ForeignKeyInfo],
    api_only: bool,
) {
    let controllers_dir = Path::new("src/controllers");

    if !controllers_dir.exists() {
        fs::create_dir_all(controllers_dir).expect("Failed to create controllers directory");
    }

    let file_path = controllers_dir.join(format!("{}_controller.rs", snake_name));

    // Build update field assignments (builder setter calls)
    let mut update_fields = String::new();
    for field in fields {
        update_fields.push_str(&format!(
            "        .set_{}(form.{}.clone())\n",
            field.name, field.name
        ));
    }

    // Build form struct fields with validation attributes
    let mut form_fields = String::new();
    for field in fields {
        let rule_attr = field.field_type.to_validation_attr();
        form_fields.push_str(&format!(
            "    {}\n    pub {}: {},\n",
            rule_attr,
            field.name,
            field.field_type.to_rust_type()
        ));
    }

    let insert_fields: String = fields
        .iter()
        .map(|f| {
            format!(
                "        {}: ActiveValue::Set(form.{}.clone()),\n",
                f.name, f.name
            )
        })
        .collect();

    // Convert ForeignKeyInfo to ForeignKeyField for templates
    let fk_fields: Vec<templates::ForeignKeyField> = foreign_keys
        .iter()
        .map(|fk| templates::ForeignKeyField {
            field_name: fk.field_name.clone(),
            target_model: fk.target_model.clone(),
            target_table: fk.target_table.clone(),
            validated: fk.validated,
        })
        .collect();

    let controller_content = if api_only {
        // Use FK-aware API template if there are foreign keys
        if !fk_fields.is_empty() {
            templates::api_controller_with_fk_template(
                name,
                snake_name,
                plural_snake,
                &form_fields,
                &update_fields,
                &insert_fields,
                &fk_fields,
            )
        } else {
            templates::api_controller_template(
                name,
                snake_name,
                plural_snake,
                &form_fields,
                &update_fields,
                &insert_fields,
            )
        }
    } else if !fk_fields.is_empty() {
        // Use FK-aware template if there are foreign keys
        templates::scaffold_controller_with_fk_template(
            name,
            snake_name,
            plural_snake,
            &form_fields,
            &update_fields,
            &insert_fields,
            &fk_fields,
        )
    } else {
        templates::scaffold_controller_template(
            name,
            snake_name,
            plural_snake,
            &form_fields,
            &update_fields,
            &insert_fields,
        )
    };

    fs::write(&file_path, controller_content).expect("Failed to write controller file");

    // Update mod.rs
    update_controllers_mod(snake_name);

    let controller_type = if api_only {
        "API controller"
    } else {
        "controller"
    };
    println!(
        "   📦 Created {}: src/controllers/{}_controller.rs",
        controller_type, snake_name
    );
}

fn update_controllers_mod(snake_name: &str) {
    let mod_path = Path::new("src/controllers/mod.rs");
    let module_name = format!("{}_controller", snake_name);

    if !mod_path.exists() {
        let content = format!("pub mod {};\n", module_name);
        fs::write(mod_path, content).expect("Failed to write mod.rs");
        return;
    }

    let content = fs::read_to_string(mod_path).expect("Failed to read mod.rs");
    let mod_declaration = format!("pub mod {};", module_name);

    if content.contains(&mod_declaration) {
        return;
    }

    let updated = format!("{}{}\n", content, mod_declaration);
    fs::write(mod_path, updated).expect("Failed to write mod.rs");
}

fn generate_inertia_pages(
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    fields: &[Field],
    foreign_keys: &[ForeignKeyInfo],
) {
    let pages_dir = Path::new("frontend/src/pages").join(plural_snake);

    if !pages_dir.exists() {
        fs::create_dir_all(&pages_dir).expect("Failed to create pages directory");
    }

    // Generate Index page
    generate_index_page(&pages_dir, name, snake_name, plural_snake, fields);

    // Generate Show page
    generate_show_page(&pages_dir, name, snake_name, plural_snake, fields);

    // Generate Create page
    generate_create_page(
        &pages_dir,
        name,
        snake_name,
        plural_snake,
        fields,
        foreign_keys,
    );

    // Generate Edit page
    generate_edit_page(
        &pages_dir,
        name,
        snake_name,
        plural_snake,
        fields,
        foreign_keys,
    );

    println!(
        "   📦 Created Inertia pages: frontend/src/pages/{}/",
        plural_snake
    );
}

fn generate_index_page(
    pages_dir: &Path,
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    fields: &[Field],
) {
    let file_path = pages_dir.join("Index.tsx");

    // Build table headers
    let headers: String = fields
        .iter()
        .map(|f| format!("              <th>{}</th>\n", to_pascal_case(&f.name)))
        .collect();

    // Build table cells
    let cells: String = fields
        .iter()
        .map(|f| {
            format!(
                "                <td>{{{snake}.{}}}</td>\n",
                f.name,
                snake = snake_name
            )
        })
        .collect();

    let content = format!(
        r#"import {{ Link }} from '@inertiajs/react';

interface {name} {{
  id: number;
{ts_fields}  created_at: string;
  updated_at: string;
}}

interface Props {{
  {plural}: {name}[];
}}

export default function Index({{ {plural} }}: Props) {{
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">{name_display}</h1>
        <Link
          href="/{plural}/create"
          className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
        >
          Create {name}
        </Link>
      </div>

      <table className="min-w-full bg-white border border-gray-200">
        <thead>
          <tr className="bg-gray-100">
            <th className="px-4 py-2 text-left">ID</th>
{headers}            <th className="px-4 py-2 text-left">Actions</th>
          </tr>
        </thead>
        <tbody>
          {{{plural}.map(({snake}) => (
            <tr key={{{snake}.id}} className="border-t">
              <td className="px-4 py-2">{{{snake}.id}}</td>
{cells}              <td className="px-4 py-2">
                <Link
                  href={{`/{plural}/${{{snake}.id}}`}}
                  className="text-blue-500 hover:underline mr-2"
                >
                  View
                </Link>
                <Link
                  href={{`/{plural}/${{{snake}.id}}/edit`}}
                  className="text-green-500 hover:underline"
                >
                  Edit
                </Link>
              </td>
            </tr>
          ))}}
        </tbody>
      </table>
    </div>
  );
}}
"#,
        name = name,
        name_display = pluralize(name),
        snake = snake_name,
        plural = plural_snake,
        headers = headers,
        cells = cells,
        ts_fields = fields
            .iter()
            .map(|f| format!("  {}: {};\n", f.name, f.field_type.to_typescript_type()))
            .collect::<String>()
    );

    fs::write(file_path, content).expect("Failed to write Index.tsx");
}

fn generate_show_page(
    pages_dir: &Path,
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    fields: &[Field],
) {
    let file_path = pages_dir.join("Show.tsx");

    // Build field displays
    let field_displays: String = fields
        .iter()
        .map(|f| {
            format!(
                r#"        <div className="mb-4">
          <label className="block text-gray-700 font-bold">{label}</label>
          <p>{{{snake}.{field}}}</p>
        </div>
"#,
                label = to_pascal_case(&f.name),
                snake = snake_name,
                field = f.name
            )
        })
        .collect();

    let content = format!(
        r#"import {{ Link, router }} from '@inertiajs/react';

interface {name} {{
  id: number;
{ts_fields}  created_at: string;
  updated_at: string;
}}

interface Props {{
  {snake}: {name};
}}

export default function Show({{ {snake} }}: Props) {{
  const handleDelete = () => {{
    if (confirm('Are you sure you want to delete this {snake}?')) {{
      router.delete(`/{plural}/${{{snake}.id}}`);
    }}
  }};

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-2xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold">{name} Details</h1>
          <div>
            <Link
              href="/{plural}"
              className="text-gray-500 hover:underline mr-4"
            >
              Back to list
            </Link>
            <Link
              href={{`/{plural}/${{{snake}.id}}/edit`}}
              className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 mr-2"
            >
              Edit
            </Link>
            <button
              onClick={{handleDelete}}
              className="bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600"
            >
              Delete
            </button>
          </div>
        </div>

        <div className="bg-white shadow rounded-lg p-6">
          <div className="mb-4">
            <label className="block text-gray-700 font-bold">ID</label>
            <p>{{{snake}.id}}</p>
          </div>
{field_displays}
          <div className="mb-4">
            <label className="block text-gray-700 font-bold">Created At</label>
            <p>{{new Date({snake}.created_at).toLocaleString()}}</p>
          </div>
          <div className="mb-4">
            <label className="block text-gray-700 font-bold">Updated At</label>
            <p>{{new Date({snake}.updated_at).toLocaleString()}}</p>
          </div>
        </div>
      </div>
    </div>
  );
}}
"#,
        name = name,
        snake = snake_name,
        plural = plural_snake,
        field_displays = field_displays,
        ts_fields = fields
            .iter()
            .map(|f| format!("  {}: {};\n", f.name, f.field_type.to_typescript_type()))
            .collect::<String>()
    );

    fs::write(file_path, content).expect("Failed to write Show.tsx");
}

fn generate_create_page(
    pages_dir: &Path,
    name: &str,
    _snake_name: &str,
    plural_snake: &str,
    fields: &[Field],
    foreign_keys: &[ForeignKeyInfo],
) {
    let file_path = pages_dir.join("Create.tsx");

    // Build form inputs with FK select dropdowns
    let form_inputs: String = fields
        .iter()
        .map(|f| {
            // Check if this field is a foreign key
            if let Some(fk) = foreign_keys.iter().find(|fk| fk.field_name == f.name) {
                if fk.validated {
                    // Validated FK: render select dropdown
                    let target_plural = pluralize(&to_snake_case(&fk.target_model));
                    let target_snake = to_snake_case(&fk.target_model);
                    format!(
                        r#"        <div className="mb-4">
          <label className="block text-gray-700 mb-2">{label}</label>
          <select
            value={{data.{field}}}
            onChange={{e => setData('{field}', parseInt(e.target.value) || 0)}}
            className="w-full border rounded px-3 py-2"
          >
            <option value="">Select {target_label}...</option>
            {{{target_plural}.map(({target_snake}) => (
              <option key={{{target_snake}.id}} value={{{target_snake}.id}}>
                {{{target_snake}.name ?? {target_snake}.title ?? {target_snake}.email ?? {target_snake}.id}}
              </option>
            ))}}
          </select>
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name,
                        target_label = fk.target_model,
                        target_plural = target_plural,
                        target_snake = target_snake
                    )
                } else {
                    // Unvalidated FK: render number input with TODO
                    format!(
                        r#"        {{/* TODO: Replace with select once {target_model} model exists */}}
        <div className="mb-4">
          <label className="block text-gray-700 mb-2">{label}</label>
          <input
            type="number"
            value={{data.{field}}}
            onChange={{e => setData('{field}', parseInt(e.target.value) || 0)}}
            className="w-full border rounded px-3 py-2"
          />
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name,
                        target_model = fk.target_model
                    )
                }
            } else {
                // Regular field
                let input_type = f.field_type.to_form_input_type();
                if input_type == "textarea" {
                    format!(
                        r#"        <div className="mb-4">
          <label className="block text-gray-700 mb-2">{label}</label>
          <textarea
            value={{data.{field}}}
            onChange={{e => setData('{field}', e.target.value)}}
            className="w-full border rounded px-3 py-2"
            rows={{4}}
          />
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name
                    )
                } else if input_type == "checkbox" {
                    format!(
                        r#"        <div className="mb-4">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={{data.{field}}}
              onChange={{e => setData('{field}', e.target.checked)}}
              className="mr-2"
            />
            <span className="text-gray-700">{label}</span>
          </label>
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name
                    )
                } else {
                    format!(
                        r#"        <div className="mb-4">
          <label className="block text-gray-700 mb-2">{label}</label>
          <input
            type="{input_type}"
            value={{data.{field}}}
            onChange={{e => setData('{field}', e.target.value)}}
            className="w-full border rounded px-3 py-2"
          />
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name,
                        input_type = input_type
                    )
                }
            }
        })
        .collect();

    // Build initial data
    let initial_data: String = fields
        .iter()
        .map(|f| {
            let default_value = match f.field_type {
                FieldType::String | FieldType::Text => "''",
                FieldType::Integer | FieldType::BigInteger | FieldType::Float => "0",
                FieldType::Boolean => "false",
                FieldType::DateTime | FieldType::Date => "''",
                FieldType::Uuid => "''",
            };
            format!("    {}: {},\n", f.name, default_value)
        })
        .collect();

    // Build TypeScript interfaces for related data
    let validated_fks: Vec<_> = foreign_keys.iter().filter(|fk| fk.validated).collect();
    let fk_interfaces: String = validated_fks
        .iter()
        .map(|fk| {
            format!(
                r#"
interface {target_model} {{
  id: number;
  name?: string;
  title?: string;
  email?: string;
}}
"#,
                target_model = fk.target_model
            )
        })
        .collect();

    // Build props interface with related data
    let fk_props: String = validated_fks
        .iter()
        .map(|fk| {
            let target_plural = pluralize(&to_snake_case(&fk.target_model));
            format!("  {}: {}[];\n", target_plural, fk.target_model)
        })
        .collect();

    // Build props destructuring
    let fk_destructure: String = if validated_fks.is_empty() {
        String::new()
    } else {
        validated_fks
            .iter()
            .map(|fk| pluralize(&to_snake_case(&fk.target_model)))
            .collect::<Vec<_>>()
            .join(", ")
            + ", "
    };

    let content = format!(
        r#"import {{ Link, useForm }} from '@inertiajs/react';
{fk_interfaces}
interface Props {{
{fk_props}  errors?: Record<string, string[]>;
}}

export default function Create({{ {fk_destructure}errors: serverErrors }}: Props) {{
  const {{ data, setData, post, processing, errors }} = useForm({{
{initial_data}  }});

  const handleSubmit = (e: React.FormEvent) => {{
    e.preventDefault();
    post('/{plural}');
  }};

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-2xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold">Create {name}</h1>
          <Link href="/{plural}" className="text-gray-500 hover:underline">
            Back to list
          </Link>
        </div>

        <form onSubmit={{handleSubmit}} className="bg-white shadow rounded-lg p-6">
{form_inputs}
          <div className="flex justify-end">
            <button
              type="submit"
              disabled={{processing}}
              className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:opacity-50"
            >
              {{processing ? 'Creating...' : 'Create {name}'}}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}}
"#,
        name = name,
        plural = plural_snake,
        form_inputs = form_inputs,
        initial_data = initial_data,
        fk_interfaces = fk_interfaces,
        fk_props = fk_props,
        fk_destructure = fk_destructure
    );

    fs::write(file_path, content).expect("Failed to write Create.tsx");
}

fn generate_edit_page(
    pages_dir: &Path,
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    fields: &[Field],
    foreign_keys: &[ForeignKeyInfo],
) {
    let file_path = pages_dir.join("Edit.tsx");

    // Build form inputs with FK select dropdowns
    let form_inputs: String = fields
        .iter()
        .map(|f| {
            // Check if this field is a foreign key
            if let Some(fk) = foreign_keys.iter().find(|fk| fk.field_name == f.name) {
                if fk.validated {
                    // Validated FK: render select dropdown
                    let target_plural = pluralize(&to_snake_case(&fk.target_model));
                    let target_snake = to_snake_case(&fk.target_model);
                    format!(
                        r#"        <div className="mb-4">
          <label className="block text-gray-700 mb-2">{label}</label>
          <select
            value={{data.{field}}}
            onChange={{e => setData('{field}', parseInt(e.target.value) || 0)}}
            className="w-full border rounded px-3 py-2"
          >
            <option value="">Select {target_label}...</option>
            {{{target_plural}.map(({target_snake}) => (
              <option key={{{target_snake}.id}} value={{{target_snake}.id}}>
                {{{target_snake}.name ?? {target_snake}.title ?? {target_snake}.email ?? {target_snake}.id}}
              </option>
            ))}}
          </select>
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name,
                        target_label = fk.target_model,
                        target_plural = target_plural,
                        target_snake = target_snake
                    )
                } else {
                    // Unvalidated FK: render number input with TODO
                    format!(
                        r#"        {{/* TODO: Replace with select once {target_model} model exists */}}
        <div className="mb-4">
          <label className="block text-gray-700 mb-2">{label}</label>
          <input
            type="number"
            value={{data.{field}}}
            onChange={{e => setData('{field}', parseInt(e.target.value) || 0)}}
            className="w-full border rounded px-3 py-2"
          />
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name,
                        target_model = fk.target_model
                    )
                }
            } else {
                // Regular field
                let input_type = f.field_type.to_form_input_type();
                if input_type == "textarea" {
                    format!(
                        r#"        <div className="mb-4">
          <label className="block text-gray-700 mb-2">{label}</label>
          <textarea
            value={{data.{field}}}
            onChange={{e => setData('{field}', e.target.value)}}
            className="w-full border rounded px-3 py-2"
            rows={{4}}
          />
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name
                    )
                } else if input_type == "checkbox" {
                    format!(
                        r#"        <div className="mb-4">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={{data.{field}}}
              onChange={{e => setData('{field}', e.target.checked)}}
              className="mr-2"
            />
            <span className="text-gray-700">{label}</span>
          </label>
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name
                    )
                } else {
                    format!(
                        r#"        <div className="mb-4">
          <label className="block text-gray-700 mb-2">{label}</label>
          <input
            type="{input_type}"
            value={{data.{field}}}
            onChange={{e => setData('{field}', e.target.value)}}
            className="w-full border rounded px-3 py-2"
          />
          {{errors.{field} && <p className="text-red-500 text-sm mt-1">{{errors.{field}}}</p>}}
        </div>
"#,
                        label = to_pascal_case(&f.name),
                        field = f.name,
                        input_type = input_type
                    )
                }
            }
        })
        .collect();

    // Build initial data from prop
    let initial_data: String = fields
        .iter()
        .map(|f| format!("    {}: {}.{},\n", f.name, snake_name, f.name))
        .collect();

    // Build TypeScript interfaces for related data
    let validated_fks: Vec<_> = foreign_keys.iter().filter(|fk| fk.validated).collect();
    let fk_interfaces: String = validated_fks
        .iter()
        .map(|fk| {
            format!(
                r#"
interface {target_model} {{
  id: number;
  name?: string;
  title?: string;
  email?: string;
}}
"#,
                target_model = fk.target_model
            )
        })
        .collect();

    // Build props interface with related data
    let fk_props: String = validated_fks
        .iter()
        .map(|fk| {
            let target_plural = pluralize(&to_snake_case(&fk.target_model));
            format!("  {}: {}[];\n", target_plural, fk.target_model)
        })
        .collect();

    // Build props destructuring
    let fk_destructure: String = if validated_fks.is_empty() {
        String::new()
    } else {
        validated_fks
            .iter()
            .map(|fk| pluralize(&to_snake_case(&fk.target_model)))
            .collect::<Vec<_>>()
            .join(", ")
            + ", "
    };

    let content = format!(
        r#"import {{ Link, useForm }} from '@inertiajs/react';
{fk_interfaces}
interface {name} {{
  id: number;
{ts_fields}  created_at: string;
  updated_at: string;
}}

interface Props {{
  {snake}: {name};
{fk_props}  errors?: Record<string, string[]>;
}}

export default function Edit({{ {snake}, {fk_destructure}errors: serverErrors }}: Props) {{
  const {{ data, setData, put, processing, errors }} = useForm({{
{initial_data}  }});

  const handleSubmit = (e: React.FormEvent) => {{
    e.preventDefault();
    put(`/{plural}/${{{snake}.id}}`);
  }};

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-2xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold">Edit {name}</h1>
          <Link href="/{plural}" className="text-gray-500 hover:underline">
            Back to list
          </Link>
        </div>

        <form onSubmit={{handleSubmit}} className="bg-white shadow rounded-lg p-6">
{form_inputs}
          <div className="flex justify-end">
            <button
              type="submit"
              disabled={{processing}}
              className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:opacity-50"
            >
              {{processing ? 'Saving...' : 'Save Changes'}}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}}
"#,
        name = name,
        snake = snake_name,
        plural = plural_snake,
        form_inputs = form_inputs,
        initial_data = initial_data,
        ts_fields = fields
            .iter()
            .map(|f| format!("  {}: {};\n", f.name, f.field_type.to_typescript_type()))
            .collect::<String>(),
        fk_interfaces = fk_interfaces,
        fk_props = fk_props,
        fk_destructure = fk_destructure
    );

    fs::write(file_path, content).expect("Failed to write Edit.tsx");
}

fn print_route_instructions(name: &str, snake_name: &str, plural_snake: &str) {
    println!("\n📝 Add these routes to src/routes.rs:\n");
    println!(
        r#"use crate::controllers::{snake}_controller;

// {name} routes
route("/{plural}", {snake}_controller::index);
route("/{plural}/create", {snake}_controller::create);
route_post("/{plural}", {snake}_controller::store);
route("/{plural}/{{id}}", {snake}_controller::show);
route("/{plural}/{{id}}/edit", {snake}_controller::edit);
route_put("/{plural}/{{id}}", {snake}_controller::update);
route_delete("/{plural}/{{id}}", {snake}_controller::destroy);"#,
        name = name,
        snake = snake_name,
        plural = plural_snake
    );
}

fn register_routes(snake_name: &str, plural_snake: &str, skip_confirm: bool) {
    let routes_path = Path::new("src/routes.rs");

    if !routes_path.exists() {
        eprintln!("Warning: src/routes.rs not found. Skipping route registration.");
        return;
    }

    let content = fs::read_to_string(routes_path).expect("Failed to read routes.rs");

    // Check if resource already registered
    let resource_pattern = format!("resource!(\"/{}\"", plural_snake);
    if content.contains(&resource_pattern) {
        println!("   ⏭️  Route already registered for /{}", plural_snake);
        return;
    }

    // Show what will be added and confirm
    let route_entry = format!(
        "\n    // {} routes\n    resource!(\"/{}\", controllers::{}),",
        to_pascal_case(snake_name),
        plural_snake,
        snake_name
    );
    let use_statement = format!("{}::{}_controller", "controllers", snake_name);

    println!("\n📝 Route registration:");
    println!(
        "   Will add: resource!(\"/{}\", controllers::{})",
        plural_snake, snake_name
    );

    if !skip_confirm {
        let confirmed = Confirm::new()
            .with_prompt("Register route in src/routes.rs?")
            .default(true)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            println!("   ⏭️  Skipped route registration");
            return;
        }
    }

    // Find routes! macro and insert before its closing brace
    // Note: resource! macro accesses controllers::name module directly
    // No additional use statement needed since routes already imports controllers
    let _ = use_statement; // Mark as intentionally unused

    // Find the closing brace of routes! macro
    // Strategy: Find "routes! {" and then find its matching "}"
    if let Some(routes_start) = content.find("routes!") {
        if let Some(brace_start) = content[routes_start..].find('{') {
            let routes_content_start = routes_start + brace_start + 1;
            let mut depth = 1;
            let mut insert_pos = None;

            for (i, c) in content[routes_content_start..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            insert_pos = Some(routes_content_start + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if let Some(pos) = insert_pos {
                let updated_content =
                    format!("{}{}\n{}", &content[..pos], route_entry, &content[pos..]);

                fs::write(routes_path, updated_content).expect("Failed to write routes.rs");
                println!("   ✅ Registered route in src/routes.rs");
                return;
            }
        }
    }

    eprintln!("Warning: Could not find routes! macro. Skipping route registration.");
}

fn generate_tests(
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    fields: &[Field],
    with_factory: bool,
) {
    let tests_dir = Path::new("src/tests");

    if !tests_dir.exists() {
        fs::create_dir_all(tests_dir).expect("Failed to create tests directory");
    }

    let file_path = tests_dir.join(format!("{}_controller_test.rs", snake_name));

    // Choose template based on whether factory is also being generated
    let test_content = if with_factory {
        // Convert Field to ScaffoldField for template
        let scaffold_fields: Vec<templates::ScaffoldField> = fields
            .iter()
            .map(|f| templates::ScaffoldField {
                name: f.name.clone(),
                field_type: f.field_type.to_scaffold_type().to_string(),
            })
            .collect();

        templates::scaffold_test_with_factory_template(
            snake_name,
            plural_snake,
            name,
            &scaffold_fields,
        )
    } else {
        templates::scaffold_test_template(snake_name, plural_snake)
    };

    fs::write(&file_path, test_content).expect("Failed to write test file");

    // Update tests/mod.rs
    update_tests_mod(snake_name);

    let test_type = if with_factory {
        "test (with factory usage)"
    } else {
        "test"
    };
    println!(
        "   📦 Created {}: src/tests/{}_controller_test.rs",
        test_type, snake_name
    );
}

fn update_tests_mod(snake_name: &str) {
    let mod_path = Path::new("src/tests/mod.rs");
    let module_name = format!("{}_controller_test", snake_name);

    if !mod_path.exists() {
        let content = format!("pub mod {};\n", module_name);
        fs::write(mod_path, content).expect("Failed to write mod.rs");
        return;
    }

    let content = fs::read_to_string(mod_path).expect("Failed to read mod.rs");
    let mod_declaration = format!("pub mod {};", module_name);

    if content.contains(&mod_declaration) {
        return;
    }

    let updated = format!("{}{}\n", content, mod_declaration);
    fs::write(mod_path, updated).expect("Failed to write mod.rs");
}

fn generate_scaffold_factory(
    name: &str,
    snake_name: &str,
    fields: &[Field],
    foreign_keys: &[ForeignKeyInfo],
) {
    let factories_dir = Path::new("src/factories");

    if !factories_dir.exists() {
        fs::create_dir_all(factories_dir).expect("Failed to create factories directory");
    }

    let file_name = format!("{}_factory", snake_name);
    let struct_name = format!("{}Factory", name);
    let file_path = factories_dir.join(format!("{}.rs", file_name));

    // Convert Field to ScaffoldField for template
    let scaffold_fields: Vec<templates::ScaffoldField> = fields
        .iter()
        .map(|f| templates::ScaffoldField {
            name: f.name.clone(),
            field_type: f.field_type.to_scaffold_type().to_string(),
        })
        .collect();

    // Convert ForeignKeyInfo to ScaffoldForeignKey for template
    let scaffold_fks: Vec<templates::ScaffoldForeignKey> = foreign_keys
        .iter()
        .map(|fk| templates::ScaffoldForeignKey {
            field_name: fk.field_name.clone(),
            target_model: fk.target_model.clone(),
            target_snake: fk.target_table.trim_end_matches('s').to_string(), // users -> user
            validated: fk.validated,
        })
        .collect();

    let factory_content = templates::scaffold_factory_template(
        &file_name,
        &struct_name,
        name,
        &scaffold_fields,
        &scaffold_fks,
    );

    fs::write(&file_path, factory_content).expect("Failed to write factory file");

    // Update factories/mod.rs
    update_factories_mod(&file_name);

    println!("   📦 Created factory: src/factories/{}.rs", file_name);
}

fn update_factories_mod(file_name: &str) {
    let mod_path = Path::new("src/factories/mod.rs");

    if !mod_path.exists() {
        let content = format!(
            "{}pub mod {};\npub use {}::*;\n",
            templates::factories_mod(),
            file_name,
            file_name
        );
        fs::write(mod_path, content).expect("Failed to write mod.rs");
        return;
    }

    let content = fs::read_to_string(mod_path).expect("Failed to read mod.rs");
    let mod_declaration = format!("pub mod {};", file_name);

    if content.contains(&mod_declaration) {
        return;
    }

    let updated = format!(
        "{}{}\npub use {}::*;\n",
        content, mod_declaration, file_name
    );
    fs::write(mod_path, updated).expect("Failed to write mod.rs");
}

/// Apply smart defaults for scaffold generation based on project structure.
///
/// Returns (api_only, with_tests, with_factory) tuple with detected defaults.
/// User-explicit flags are preserved (e.g., if --api is passed, always use API mode).
/// Apply smart defaults for scaffold generation based on project structure.
///
/// Returns (api_only, with_tests, with_factory) tuple with detected defaults.
/// User-explicit flags are preserved (e.g., if --api is passed, always use API mode).
fn apply_smart_defaults(
    explicit_api: bool,
    explicit_tests: bool,
    explicit_factory: bool,
    tracking: &mut SmartDefaults,
) -> (bool, bool, bool) {
    // Analyze project once for all detections
    let analyzer = ProjectAnalyzer::current_dir();
    let conventions = analyzer.analyze();

    let api_only = apply_api_smart_default_from_conventions(explicit_api, &conventions, tracking);
    let with_tests = apply_test_smart_default(explicit_tests, &conventions, tracking);
    let with_factory = apply_factory_smart_default(explicit_factory, &conventions, tracking);

    (api_only, with_tests, with_factory)
}

/// Apply smart default for API-only mode based on analyzed conventions.
fn apply_api_smart_default_from_conventions(
    explicit_api: bool,
    conventions: &ProjectConventions,
    tracking: &mut SmartDefaults,
) -> bool {
    // If user explicitly requested API mode, honor that
    if explicit_api {
        return true;
    }

    // If no Inertia pages found, suggest API-only mode
    if !conventions.has_inertia_pages {
        tracking.api_detected = true;
        return true;
    }

    false
}

/// Apply smart default for --with-tests based on existing test patterns.
fn apply_test_smart_default(
    explicit_tests: bool,
    conventions: &ProjectConventions,
    tracking: &mut SmartDefaults,
) -> bool {
    // If user explicitly requested tests, honor that
    if explicit_tests {
        return true;
    }

    // If existing test pattern detected, suggest --with-tests
    if conventions.test_pattern == TestPattern::PerController && conventions.test_file_count > 0 {
        tracking.test_detected = true;
        tracking.test_count = conventions.test_file_count;
        return true;
    }

    false
}

/// Apply smart default for --with-factory based on existing factory patterns.
fn apply_factory_smart_default(
    explicit_factory: bool,
    conventions: &ProjectConventions,
    tracking: &mut SmartDefaults,
) -> bool {
    // If user explicitly requested factory, honor that
    if explicit_factory {
        return true;
    }

    // If existing factory pattern detected, suggest --with-factory
    if conventions.factory_pattern == FactoryPattern::PerModel && conventions.factory_file_count > 0
    {
        tracking.factory_detected = true;
        tracking.factory_count = conventions.factory_file_count;
        return true;
    }

    false
}
