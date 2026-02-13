mod entity;
mod make;
mod project;

pub use entity::*;
pub use make::*;
pub use project::*;

use entity::to_pascal_case;
use entity::to_snake_case;

// ============================================================================
// Docker Templates
// ============================================================================

/// Generate Dockerfile for production deployment
pub fn dockerfile_template(package_name: &str) -> String {
    include_str!("files/docker/Dockerfile.tpl").replace("{package_name}", package_name)
}

/// Generate .dockerignore file
pub fn dockerignore_template() -> &'static str {
    include_str!("files/docker/dockerignore.tpl")
}

/// Generate docker-compose.yml for local development
pub fn docker_compose_template(
    project_name: &str,
    include_mailpit: bool,
    include_minio: bool,
) -> String {
    let mailpit_service = if include_mailpit {
        include_str!("files/docker/mailpit.service.tpl").replace("{project_name}", project_name)
    } else {
        String::new()
    };

    let minio_service = if include_minio {
        include_str!("files/docker/minio.service.tpl").replace("{project_name}", project_name)
    } else {
        String::new()
    };

    let additional_volumes = if include_minio {
        "\n  minio_data:".to_string()
    } else {
        String::new()
    };

    include_str!("files/docker/docker-compose.yml.tpl")
        .replace("{project_name}", project_name)
        .replace("{mailpit_service}", &mailpit_service)
        .replace("{minio_service}", &minio_service)
        .replace("{additional_volumes}", &additional_volumes)
}

// ============================================================================
// DigitalOcean App Platform Templates
// ============================================================================

/// Generate app.yaml for DigitalOcean App Platform deployment
pub fn do_app_yaml_template(package_name: &str, github_repo: &str) -> String {
    include_str!("files/do/app.yaml.tpl")
        .replace("{package_name}", package_name)
        .replace("{github_repo}", github_repo)
}

// ============================================================================
// AI Development Boost Templates
// ============================================================================

/// Ferro framework guidelines for AI assistants
pub fn ferro_guidelines_template() -> &'static str {
    r#"# Ferro Framework Guidelines

Ferro is a Rust web framework inspired by Laravel, providing a familiar developer experience with Rust's performance and safety.

## Project Structure

```
app/
├── src/
│   ├── main.rs           # Application entry point
│   ├── routes.rs         # Route definitions
│   ├── bootstrap.rs      # Application bootstrap
│   ├── controllers/      # Request handlers
│   ├── middleware/       # HTTP middleware
│   ├── models/           # Database models (SeaORM entities)
│   │   └── entities/     # Auto-generated entities (do not edit)
│   ├── actions/          # Business logic actions
│   ├── events/           # Domain events
│   ├── listeners/        # Event listeners
│   ├── jobs/             # Background jobs
│   ├── notifications/    # Multi-channel notifications
│   ├── tasks/            # Scheduled tasks
│   ├── config/           # Configuration modules
│   └── migrations/       # Database migrations
└── Cargo.toml
frontend/
├── src/
│   ├── main.tsx          # Frontend entry
│   └── pages/            # Inertia.js pages (React/TypeScript)
└── package.json
```

## Key Conventions

### Controllers
- Use the `#[handler]` macro for route handlers
- Return `Response` type using helper macros

```rust
use ferro::{handler, json_response, Request, Response};

#[handler]
pub async fn index(_req: Request) -> Response {
    json_response!({ "message": "Hello" })
}
```

### Middleware
- Implement the `Middleware` trait
- Use `#[async_trait]` for async methods

```rust
use ferro::{async_trait, Middleware, Next, Request, Response};

pub struct MyMiddleware;

#[async_trait]
impl Middleware for MyMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        // Before request
        let response = next(request).await;
        // After request
        response
    }
}
```

### Models (SeaORM)
- Models use SeaORM with an Eloquent-like API
- Entity files in `models/entities/` are auto-generated
- Custom logic goes in `models/{table_name}.rs`

```rust
// Query builder pattern
let users = User::query()
    .filter(Column::Active.eq(true))
    .all()
    .await?;

// Fluent create
let user = User::create()
    .set_email("user@example.com")
    .set_name("John")
    .insert()
    .await?;

// Fluent update
let updated = user
    .set_name("Jane")
    .update()
    .await?;
```

### Inertia.js Integration
- Backend sends data via `inertia_response!` macro
- Frontend receives as props in React components
- TypeScript types auto-generated from Rust structs

```rust
// Backend
#[handler]
pub async fn show(req: Request) -> Response {
    inertia_response!("Users/Show", {
        "user": user,
        "posts": posts
    })
}
```

### Database Migrations
- Create with `ferro make:migration <name>`
- Run with `ferro migrate`
- Sync models with `ferro db:sync`

### Error Handling
- Use `#[domain_error]` macro for custom errors
- Errors automatically convert to appropriate HTTP responses

```rust
use ferro::domain_error;

#[domain_error(status = 404, message = "User not found")]
pub struct UserNotFound;
```

## CLI Commands

- `ferro new <name>` - Create new project
- `ferro serve` - Start dev servers
- `ferro make:controller <name>` - Generate controller
- `ferro make:middleware <name>` - Generate middleware
- `ferro make:migration <name>` - Generate migration
- `ferro make:event <name>` - Generate event
- `ferro make:job <name>` - Generate background job
- `ferro migrate` - Run migrations
- `ferro db:sync` - Sync DB schema to entities
- `ferro mcp` - Start MCP server for AI assistance

## Best Practices

1. **Use Actions for Business Logic**: Keep controllers thin, move logic to action classes
2. **Leverage the Type System**: Use Rust's types for validation and safety
3. **Auto-generate Types**: Run `ferro generate-types` to sync Rust structs to TypeScript
4. **Database Sync**: Use `ferro db:sync` after migrations to update entity files
5. **Middleware Order**: Register middleware in the correct order in routes.rs
"#
}

/// Cursor-specific rules file
pub fn cursor_rules_template() -> &'static str {
    r#"# Ferro Framework - Cursor Rules

You are working on a Ferro framework project. Ferro is a Rust web framework inspired by Laravel.

## Framework Knowledge

- Ferro uses Rust with async/await for the backend
- Frontend uses React + TypeScript with Inertia.js
- Database layer uses SeaORM with an Eloquent-like API
- The project follows Laravel conventions adapted for Rust

## Code Style

- Use `#[handler]` macro for route handlers
- Use `#[async_trait]` for middleware
- Use `#[domain_error]` for custom errors
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)

## When Generating Code

1. Controllers go in `app/src/controllers/`
2. Middleware goes in `app/src/middleware/`
3. Models go in `app/src/models/`
4. React pages go in `frontend/src/pages/`

## Available MCP Tools

Use the Ferro MCP tools for introspection:
- `application_info` - Get app info, versions, crates
- `list_routes` - See all defined routes
- `db_schema` - Get database schema
- `db_query` - Run read-only SQL queries
- `list_migrations` - Check migration status
- `list_middleware` - See registered middleware
- `read_logs` - Read application logs
- `last_error` - Get recent errors
- `tinker` - Execute Rust code in app context
- `browser_logs` - Read frontend error logs

## Common Patterns

### Adding a new page
1. Create controller handler in `app/src/controllers/`
2. Add route in `app/src/routes.rs`
3. Create React page in `frontend/src/pages/`
4. Run `ferro generate-types` to sync types

### Adding a database table
1. `ferro make:migration create_table_name`
2. Edit migration file
3. `ferro migrate`
4. `ferro db:sync`
"#
}

/// CLAUDE.md template for Claude Code
pub fn claude_md_template() -> &'static str {
    r#"# Project Instructions

This is a Ferro framework project - a Rust web framework inspired by Laravel.

## Quick Reference

- **Backend**: Rust with async/await, SeaORM for database
- **Frontend**: React + TypeScript with Inertia.js
- **CLI**: Use `ferro` command for scaffolding

## MCP Tools Available

The Ferro MCP server provides these introspection tools:
- `application_info`, `list_routes`, `db_schema`, `db_query`
- `list_migrations`, `list_middleware`, `list_events`, `list_jobs`
- `read_logs`, `last_error`, `browser_logs`, `tinker`

## Development Workflow

1. Use `ferro serve` to start dev servers
2. Use `ferro make:*` commands for scaffolding
3. Use `ferro db:sync` after migrations to update models
4. Use `ferro generate-types` to sync TypeScript types

## Ferro Framework Guidelines

See `.ai/guidelines/ferro.md` for detailed framework conventions.
"#
}

/// Section to append to existing CLAUDE.md
pub fn claude_md_ferro_section() -> &'static str {
    r#"
---

# Ferro Framework

This is a Ferro framework project - a Rust web framework inspired by Laravel.

## MCP Tools Available

The Ferro MCP server provides introspection tools:
- `application_info`, `list_routes`, `db_schema`, `db_query`
- `list_migrations`, `list_middleware`, `list_events`, `list_jobs`
- `read_logs`, `last_error`, `browser_logs`, `tinker`

## Framework Conventions

See `.ai/guidelines/ferro.md` for detailed framework conventions.
"#
}

/// GitHub Copilot instructions
pub fn copilot_instructions_template() -> &'static str {
    r#"# GitHub Copilot Instructions

## Project Type
This is a Ferro framework project (Rust web framework inspired by Laravel).

## Key Files
- `app/src/routes.rs` - Route definitions
- `app/src/controllers/` - Request handlers
- `app/src/models/` - Database models (SeaORM)
- `frontend/src/pages/` - React/TypeScript pages

## Code Patterns

### Controller Handler
```rust
use ferro::{handler, json_response, Request, Response};

#[handler]
pub async fn index(_req: Request) -> Response {
    json_response!({ "data": value })
}
```

### SeaORM Query
```rust
let items = Model::query()
    .filter(Column::Field.eq(value))
    .all()
    .await?;
```

### Inertia Response
```rust
inertia_response!("PageName", { "prop": value })
```

## Conventions
- Controllers are async handlers with `#[handler]` macro
- Models use SeaORM with Eloquent-like query builder
- Frontend pages receive data as Inertia props
- TypeScript types are auto-generated from Rust structs
"#
}

// ============================================================================
// Scaffold Factory Template
// ============================================================================

/// Scaffold field information for factory generation
pub struct ScaffoldField {
    pub name: String,
    pub field_type: String,
}

/// Foreign key information for scaffold generation
pub struct ScaffoldForeignKey {
    /// The field name (e.g., "user_id")
    pub field_name: String,
    /// The target model name in PascalCase (e.g., "User")
    pub target_model: String,
    /// The target model name in snake_case (e.g., "user")
    pub target_snake: String,
    /// Whether the target model exists in the project
    pub validated: bool,
}

/// Template for generating factory with pre-populated fields from scaffold definition
pub fn scaffold_factory_template(
    _file_name: &str,
    struct_name: &str,
    model_name: &str,
    fields: &[ScaffoldField],
    foreign_keys: &[ScaffoldForeignKey],
) -> String {
    // Separate FK fields from regular fields for special handling
    let fk_field_names: Vec<&str> = foreign_keys
        .iter()
        .map(|fk| fk.field_name.as_str())
        .collect();

    // Build field definitions
    let field_defs: String = fields
        .iter()
        .map(|f| {
            format!(
                "    pub {}: {},\n",
                f.name,
                rust_type_for_factory(&f.field_type)
            )
        })
        .collect();

    // Build Fake::* assignments - handle FK fields specially
    let fake_assignments: String = fields
        .iter()
        .map(|f| {
            if fk_field_names.contains(&f.name.as_str()) {
                // Find the FK info
                let fk = foreign_keys.iter().find(|fk| fk.field_name == f.name);
                if let Some(fk) = fk {
                    if fk.validated {
                        format!(
                            "            {}: 0, // Set via with_{target}() or create will make one\n",
                            f.name,
                            target = fk.target_snake
                        )
                    } else {
                        format!(
                            "            {}: Fake::integer(1, 1000000) as i64, // TODO: Create {target} first\n",
                            f.name,
                            target = fk.target_model
                        )
                    }
                } else {
                    format!("            {}: {},\n", f.name, fake_value_for_type(&f.field_type))
                }
            } else {
                format!("            {}: {},\n", f.name, fake_value_for_type(&f.field_type))
            }
        })
        .collect();

    // Build factory imports for validated FKs
    let fk_imports: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            format!(
                "use crate::factories::{target_snake}_factory::{target_pascal}Factory;\n",
                target_snake = fk.target_snake,
                target_pascal = fk.target_model
            )
        })
        .collect();

    // Build with_* methods for validated FKs
    let with_methods: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            format!(
                r#"
    /// Set the {target_snake} for this factory
    pub fn with_{target_snake}(mut self, {target_snake}_id: i64) -> Self {{
        self.{field_name} = {target_snake}_id;
        self
    }}
"#,
                target_snake = fk.target_snake,
                field_name = fk.field_name
            )
        })
        .collect();

    // Build create method that creates related records first (for validated FKs)
    let validated_fks: Vec<&ScaffoldForeignKey> =
        foreign_keys.iter().filter(|fk| fk.validated).collect();
    let create_method = if validated_fks.is_empty() {
        String::new()
    } else {
        let create_relations: String = validated_fks
            .iter()
            .map(|fk| {
                format!(
                    "        let {target_snake} = {target_pascal}Factory::factory().create(db).await;\n",
                    target_snake = fk.target_snake,
                    target_pascal = fk.target_model
                )
            })
            .collect();

        let set_fk_fields: String = validated_fks
            .iter()
            .map(|fk| {
                format!(
                    "        result.{field_name} = {target_snake}.id;\n",
                    field_name = fk.field_name,
                    target_snake = fk.target_snake
                )
            })
            .collect();

        format!(
            r#"
    /// Create related records and set FK fields
    pub async fn create_with_relations(&self, db: &DatabaseConnection) -> Self {{
{create_relations}        let mut result = self.clone();
{set_fk_fields}        result
    }}
"#,
            create_relations = create_relations,
            set_fk_fields = set_fk_fields
        )
    };

    format!(
        r#"//! {struct_name} factory
//!
//! Generated with `ferro make:scaffold --with-factory`

use ferro::testing::{{Factory, FactoryTraits, Fake}};
{fk_imports}// use ferro::testing::DatabaseFactory;
// use crate::models::{model_lower}::{{self, Model as {model_name}}};
// use sea_orm::DatabaseConnection;

/// Factory for creating {model_name} instances in tests
#[derive(Clone)]
pub struct {struct_name} {{
    pub id: i64,
{field_defs}    pub created_at: String,
    pub updated_at: String,
}}

impl {struct_name} {{{with_methods}{create_method}}}

impl Factory for {struct_name} {{
    fn definition() -> Self {{
        Self {{
            id: 0, // Will be set by database
{fake_assignments}            created_at: Fake::datetime(),
            updated_at: Fake::datetime(),
        }}
    }}

    fn traits() -> FactoryTraits<Self> {{
        FactoryTraits::new()
    }}
}}

// Uncomment to enable database persistence with create():
//
// #[ferro::async_trait]
// impl DatabaseFactory for {struct_name} {{
//     type Entity = {model_lower}::Entity;
//     type ActiveModel = {model_lower}::ActiveModel;
// }}

// Usage in tests:
//
// // Make without persisting:
// let model = {struct_name}::factory().make();
//
// // Apply named trait:
// let custom = {struct_name}::factory().trait_("custom").make();
//
// // With inline state:
// let model = {struct_name}::factory()
//     .state(|m| m.id = 42)
//     .make();
//
// // Create with database persistence:
// let model = {struct_name}::factory().create().await?;
//
// // Create multiple:
// let models = {struct_name}::factory().count(5).create_many().await?;
"#,
        struct_name = struct_name,
        model_name = model_name,
        model_lower = model_name.to_lowercase(),
        field_defs = field_defs,
        fake_assignments = fake_assignments,
        fk_imports = fk_imports,
        with_methods = with_methods,
        create_method = create_method,
    )
}

/// Convert scaffold field type to Rust type for factory
fn rust_type_for_factory(field_type: &str) -> &'static str {
    match field_type.to_lowercase().as_str() {
        "string" | "str" | "text" => "String",
        "int" | "integer" | "i32" => "i32",
        "bigint" | "biginteger" | "i64" => "i64",
        "float" | "f64" | "double" => "f64",
        "bool" | "boolean" => "bool",
        "datetime" | "timestamp" => "String",
        "date" => "String",
        "uuid" => "String",
        _ => "String",
    }
}

/// Generate Fake::* value based on field type
fn fake_value_for_type(field_type: &str) -> &'static str {
    match field_type.to_lowercase().as_str() {
        "string" | "str" => "Fake::word()",
        "text" => "Fake::sentence()",
        "int" | "integer" | "i32" => "Fake::integer(1, 1000)",
        "bigint" | "biginteger" | "i64" => "Fake::integer(1, 1000000) as i64",
        "float" | "f64" | "double" => "Fake::float(0.0, 1000.0)",
        "bool" | "boolean" => "Fake::boolean()",
        "datetime" | "timestamp" => "Fake::datetime()",
        "date" => "Fake::date()",
        "uuid" => "Fake::uuid()",
        _ => "Fake::word()",
    }
}

// ============================================================================
// Scaffold Test Template
// ============================================================================

/// Template for generating controller tests with make:scaffold --with-tests
pub fn scaffold_test_template(snake_name: &str, plural_snake: &str) -> String {
    format!(
        r#"//! {plural_pascal} controller tests
//!
//! Generated with `ferro make:scaffold --with-tests`

use ferro::testing::{{TestClient, TestResponse}};

/// Test that the {plural} index endpoint returns success
#[tokio::test]
async fn test_{plural}_index() {{
    let client = TestClient::new();

    let response = client.get("/{plural}").send().await;

    // TODO: Configure TestClient with your app's router
    // response.assert_ok();
    assert!(response.status().is_success());
}}

/// Test that showing a single {snake} returns success
#[tokio::test]
async fn test_{plural}_show() {{
    let client = TestClient::new();

    let response = client.get("/{plural}/1").send().await;

    // TODO: Create a test record first, then verify response
    // response.assert_ok().assert_json_has("{snake}");
    assert!(response.status().is_success());
}}

/// Test that creating a {snake} works
#[tokio::test]
async fn test_{plural}_store() {{
    let client = TestClient::new();

    let response = client
        .post("/{plural}")
        .json(&serde_json::json!({{
            // TODO: Add your model fields here
        }}))
        .send()
        .await;

    // TODO: Verify redirect or JSON response
    // response.assert_status(302);
    assert!(response.status().is_success());
}}

/// Test that updating a {snake} works
#[tokio::test]
async fn test_{plural}_update() {{
    let client = TestClient::new();

    let response = client
        .put("/{plural}/1")
        .json(&serde_json::json!({{
            // TODO: Add your model fields here
        }}))
        .send()
        .await;

    // TODO: Verify redirect or JSON response
    // response.assert_status(302);
    assert!(response.status().is_success());
}}

/// Test that deleting a {snake} works
#[tokio::test]
async fn test_{plural}_destroy() {{
    let client = TestClient::new();

    let response = client.delete("/{plural}/1").send().await;

    // TODO: Verify redirect or JSON response
    // response.assert_status(302);
    assert!(response.status().is_success());
}}
"#,
        snake = snake_name,
        plural = plural_snake,
        plural_pascal = to_pascal_case(plural_snake),
    )
}

/// Template for generating controller tests that use factories
///
/// Generated when both --with-tests and --with-factory flags are used.
/// Tests create model instances using the factory for realistic test data.
pub fn scaffold_test_with_factory_template(
    snake_name: &str,
    plural_snake: &str,
    pascal_name: &str,
    fields: &[ScaffoldField],
) -> String {
    // Build JSON fields for store/update tests from factory data
    let json_fields: String = fields
        .iter()
        .map(|f| format!("            \"{}\": factory.{}.clone(),\n", f.name, f.name))
        .collect();

    format!(
        r#"//! {plural_pascal} controller tests
//!
//! Generated with `ferro make:scaffold --with-tests --with-factory`

use ferro::testing::{{Factory, TestClient, TestDatabase, TestResponse}};
use crate::factories::{snake}_factory::{pascal}Factory;

/// Test that the {plural} index endpoint returns a list
#[tokio::test]
async fn test_{plural}_index() {{
    let db = TestDatabase::new().await;
    let client = TestClient::with_db(db.clone());

    // Create 3 {plural} using factory
    for _ in 0..3 {{
        let model = {pascal}Factory::factory().create(&db).await.unwrap();
    }}

    let response = client.get("/{plural}").send().await;

    response.assert_ok();
    // response.assert_json_path("data").assert_count(3);
}}

/// Test that showing a single {snake} returns the correct record
#[tokio::test]
async fn test_{plural}_show() {{
    let db = TestDatabase::new().await;
    let client = TestClient::with_db(db.clone());

    // Create a {snake} using factory
    let {snake} = {pascal}Factory::factory().create(&db).await.unwrap();

    let response = client.get(&format!("/{plural}/{{}}", {snake}.id)).send().await;

    response.assert_ok();
    // response.assert_json_path("data.id").assert_eq({snake}.id);
}}

/// Test that creating a {snake} persists to database
#[tokio::test]
async fn test_{plural}_store() {{
    let db = TestDatabase::new().await;
    let client = TestClient::with_db(db.clone());

    // Use factory to generate valid input data
    let factory = {pascal}Factory::definition();

    let response = client
        .post("/{plural}")
        .json(&serde_json::json!({{
{json_fields}        }}))
        .send()
        .await;

    response.assert_created();
    // Verify record was created in database
    // let count = {pascal}::query().count(&db).await.unwrap();
    // assert_eq!(count, 1);
}}

/// Test that updating a {snake} modifies the record
#[tokio::test]
async fn test_{plural}_update() {{
    let db = TestDatabase::new().await;
    let client = TestClient::with_db(db.clone());

    // Create initial {snake}
    let {snake} = {pascal}Factory::factory().create(&db).await.unwrap();

    // Use factory for updated data
    let factory = {pascal}Factory::definition();

    let response = client
        .put(&format!("/{plural}/{{}}", {snake}.id))
        .json(&serde_json::json!({{
{json_fields}        }}))
        .send()
        .await;

    response.assert_ok();
    // Verify record was updated
    // let updated = {pascal}::find({snake}.id, &db).await.unwrap();
    // assert_ne!(updated.field, {snake}.field);
}}

/// Test that deleting a {snake} removes the record
#[tokio::test]
async fn test_{plural}_destroy() {{
    let db = TestDatabase::new().await;
    let client = TestClient::with_db(db.clone());

    // Create a {snake} using factory
    let {snake} = {pascal}Factory::factory().create(&db).await.unwrap();

    let response = client.delete(&format!("/{plural}/{{}}", {snake}.id)).send().await;

    response.assert_ok();
    // Verify record was deleted
    // let exists = {pascal}::find({snake}.id, &db).await.is_ok();
    // assert!(!exists);
}}
"#,
        snake = snake_name,
        plural = plural_snake,
        pascal = pascal_name,
        plural_pascal = to_pascal_case(plural_snake),
        json_fields = json_fields,
    )
}

// ============================================================================
// FK-Aware Scaffold Templates
// ============================================================================

/// Foreign key information for template generation.
/// Mirrors the ForeignKeyInfo from analyzer.rs for use in templates.
#[derive(Debug, Clone)]
pub struct ForeignKeyField {
    /// The field name (e.g., "user_id")
    pub field_name: String,
    /// The target model name in PascalCase (e.g., "User")
    pub target_model: String,
    /// The target table name in snake_case plural (e.g., "users")
    pub target_table: String,
    /// Whether the target model exists in the project
    pub validated: bool,
}

/// Template for generating full-stack controller with FK eager loading
pub fn scaffold_controller_with_fk_template(
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    form_fields: &str,
    update_fields: &str,
    insert_fields: &str,
    foreign_keys: &[ForeignKeyField],
) -> String {
    // Build FK imports
    let fk_imports: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            format!(
                "use crate::models::{}::{{Entity as {}Entity, Model as {}}};\n",
                fk.target_table.trim_end_matches('s'), // singularize for module name
                fk.target_model,
                fk.target_model
            )
        })
        .collect();

    // Build props for related data in Index
    let fk_index_props: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| format!("    pub {}: Vec<{}>,\n", fk.target_table, fk.target_model))
        .collect();

    // Build fetching code for index
    let fk_index_fetches: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            format!(
                "    let {} = {}Entity::find().all(db).await\n        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;\n",
                fk.target_table,
                fk.target_model
            )
        })
        .collect();

    // Build props assignment for index
    let fk_index_props_assign: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| format!(", {}", fk.target_table))
        .collect();

    // Build props for Create page
    let fk_create_props: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| format!("    pub {}: Vec<{}>,\n", fk.target_table, fk.target_model))
        .collect();

    // Build fetching code for create
    let fk_create_fetches: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            format!(
                "    let {} = {}Entity::find().all(db).await\n        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;\n",
                fk.target_table,
                fk.target_model
            )
        })
        .collect();

    // Build props assignment for create
    let fk_create_props_assign: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| format!(", {}", fk.target_table))
        .collect();

    // Build props for Edit page
    let fk_edit_props: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| format!("    pub {}: Vec<{}>,\n", fk.target_table, fk.target_model))
        .collect();

    // Build fetching code for edit
    let fk_edit_fetches: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            format!(
                "    let {} = {}Entity::find().all(db).await\n        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;\n",
                fk.target_table,
                fk.target_model
            )
        })
        .collect();

    // Build props assignment for edit
    let fk_edit_props_assign: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| format!(", {}", fk.target_table))
        .collect();

    // Generate validated FK comment if there are unvalidated FKs
    let unvalidated_fks: Vec<_> = foreign_keys.iter().filter(|fk| !fk.validated).collect();
    let unvalidated_comment = if !unvalidated_fks.is_empty() {
        let fk_list: String = unvalidated_fks
            .iter()
            .map(|fk| {
                format!(
                    "// - {} (model {} not found)",
                    fk.field_name, fk.target_model
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "\n// TODO: The following FK fields have no corresponding model:\n{}\n// Create these models to enable relationship loading.\n",
            fk_list
        )
    } else {
        String::new()
    };

    format!(
        r#"//! {name} controller
//!
//! Generated with `ferro make:scaffold`
{unvalidated_comment}
use ferro::{{
    http::{{Request, Response, HttpResponse}},
    inertia::{{Inertia, SavedInertiaContext}},
    validation::Validatable,
    ValidateRules,
}};
use sea_orm::{{EntityTrait, ActiveModelTrait, ActiveValue}};
use serde::{{Deserialize, Serialize}};

use crate::models::{snake_name}::{{self, Entity, Model as {name}}};
{fk_imports}
#[derive(Debug, Deserialize, Serialize, ValidateRules)]
pub struct {name}Form {{
{form_fields}}}

#[derive(Debug, Serialize)]
pub struct {plural_pascal}IndexProps {{
    pub {plural}: Vec<{name}>,
{fk_index_props}}}

#[derive(Debug, Serialize)]
pub struct {name}ShowProps {{
    pub {snake}: {name},
}}

#[derive(Debug, Serialize)]
pub struct {name}CreateProps {{
    pub errors: Option<std::collections::HashMap<String, Vec<String>>>,
{fk_create_props}}}

#[derive(Debug, Serialize)]
pub struct {name}EditProps {{
    pub {snake}: {name},
    pub errors: Option<std::collections::HashMap<String, Vec<String>>>,
{fk_edit_props}}}

/// List all {plural}
pub async fn index(req: Request) -> Response {{
    let db = req.db();
    let {plural} = {snake_name}::Entity::find()
        .all(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;

{fk_index_fetches}
    Inertia::render(&req, "{plural_pascal}/Index", {plural_pascal}IndexProps {{ {plural}{fk_index_props_assign} }})
}}

/// Show a single {snake}
pub async fn show(req: Request, id: i64) -> Response {{
    let db = req.db();
    let {snake} = {snake_name}::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?
        .ok_or_else(|| HttpResponse::not_found("{name} not found"))?;

    Inertia::render(&req, "{plural_pascal}/Show", {name}ShowProps {{ {snake} }})
}}

/// Show create form
pub async fn create(req: Request) -> Response {{
    let db = req.db();
{fk_create_fetches}
    Inertia::render(&req, "{plural_pascal}/Create", {name}CreateProps {{ errors: None{fk_create_props_assign} }})
}}

/// Store a new {snake}
pub async fn store(req: Request) -> Response {{
    let ctx = SavedInertiaContext::from(&req);
    let db = req.db();
    let form: {name}Form = req.input().await.map_err(|e| {{
        HttpResponse::bad_request(format!("Invalid form data: {{}}", e))
    }})?;

    // Validate using derive macro
    if let Err(errors) = form.validate() {{
{fk_create_fetches}        return Inertia::render_ctx(&ctx, "{plural_pascal}/Create", {name}CreateProps {{
            errors: Some(errors.into_messages()){fk_create_props_assign}
        }});
    }}

    let model = {snake_name}::ActiveModel {{
        id: ActiveValue::NotSet,
{insert_fields}        created_at: ActiveValue::Set(chrono::Utc::now()),
        updated_at: ActiveValue::Set(chrono::Utc::now()),
    }};

    let result = model.insert(db).await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;

    HttpResponse::redirect(&format!("/{plural}/{{}}", result.id))
}}

/// Show edit form
pub async fn edit(req: Request, id: i64) -> Response {{
    let db = req.db();
    let {snake} = {snake_name}::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?
        .ok_or_else(|| HttpResponse::not_found("{name} not found"))?;

{fk_edit_fetches}
    Inertia::render(&req, "{plural_pascal}/Edit", {name}EditProps {{ {snake}, errors: None{fk_edit_props_assign} }})
}}

/// Update an existing {snake}
pub async fn update(req: Request, id: i64) -> Response {{
    let ctx = SavedInertiaContext::from(&req);
    let db = req.db();
    let form: {name}Form = req.input().await.map_err(|e| {{
        HttpResponse::bad_request(format!("Invalid form data: {{}}", e))
    }})?;

    let {snake} = {snake_name}::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?
        .ok_or_else(|| HttpResponse::not_found("{name} not found"))?;

    // Validate using derive macro
    if let Err(errors) = form.validate() {{
{fk_edit_fetches}        return Inertia::render_ctx(&ctx, "{plural_pascal}/Edit", {name}EditProps {{
            {snake},
            errors: Some(errors.into_messages()){fk_edit_props_assign}
        }});
    }}

    {snake}
        .update()
{update_fields}        .save()
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;

    HttpResponse::redirect(&format!("/{plural}/{{}}", id))
}}

/// Delete a {snake}
pub async fn destroy(req: Request, id: i64) -> Response {{
    let db = req.db();
    {snake_name}::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;

    HttpResponse::redirect("/{plural}")
}}
"#,
        name = name,
        snake = snake_name,
        snake_name = snake_name,
        plural = plural_snake,
        plural_pascal = to_pascal_case(plural_snake),
        form_fields = form_fields,
        update_fields = update_fields,
        insert_fields = insert_fields,
        fk_imports = fk_imports,
        fk_index_props = fk_index_props,
        fk_index_fetches = fk_index_fetches,
        fk_index_props_assign = fk_index_props_assign,
        fk_create_props = fk_create_props,
        fk_create_fetches = fk_create_fetches,
        fk_create_props_assign = fk_create_props_assign,
        fk_edit_props = fk_edit_props,
        fk_edit_fetches = fk_edit_fetches,
        fk_edit_props_assign = fk_edit_props_assign,
        unvalidated_comment = unvalidated_comment,
    )
}

/// Template for generating full-stack controller without FK relationships
pub fn scaffold_controller_template(
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    form_fields: &str,
    update_fields: &str,
    insert_fields: &str,
) -> String {
    format!(
        r#"//! {name} controller
//!
//! Generated with `ferro make:scaffold`

use ferro::{{
    http::{{Request, Response, HttpResponse}},
    inertia::{{Inertia, SavedInertiaContext}},
    validation::Validatable,
    ValidateRules,
}};
use sea_orm::{{EntityTrait, ActiveModelTrait, ActiveValue}};
use serde::{{Deserialize, Serialize}};

use crate::models::{snake_name}::{{self, Entity, Model as {name}}};

#[derive(Debug, Deserialize, Serialize, ValidateRules)]
pub struct {name}Form {{
{form_fields}}}

#[derive(Debug, Serialize)]
pub struct {plural_pascal}IndexProps {{
    pub {plural}: Vec<{name}>,
}}

#[derive(Debug, Serialize)]
pub struct {name}ShowProps {{
    pub {snake}: {name},
}}

#[derive(Debug, Serialize)]
pub struct {name}CreateProps {{
    pub errors: Option<std::collections::HashMap<String, Vec<String>>>,
}}

#[derive(Debug, Serialize)]
pub struct {name}EditProps {{
    pub {snake}: {name},
    pub errors: Option<std::collections::HashMap<String, Vec<String>>>,
}}

/// List all {plural}
pub async fn index(req: Request) -> Response {{
    let db = req.db();
    let {plural} = {snake_name}::Entity::find()
        .all(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;

    Inertia::render(&req, "{plural_pascal}/Index", {plural_pascal}IndexProps {{ {plural} }})
}}

/// Show a single {snake}
pub async fn show(req: Request, id: i64) -> Response {{
    let db = req.db();
    let {snake} = {snake_name}::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?
        .ok_or_else(|| HttpResponse::not_found("{name} not found"))?;

    Inertia::render(&req, "{plural_pascal}/Show", {name}ShowProps {{ {snake} }})
}}

/// Show create form
pub async fn create(req: Request) -> Response {{
    Inertia::render(&req, "{plural_pascal}/Create", {name}CreateProps {{ errors: None }})
}}

/// Store a new {snake}
pub async fn store(req: Request) -> Response {{
    let ctx = SavedInertiaContext::from(&req);
    let form: {name}Form = req.input().await.map_err(|e| {{
        HttpResponse::bad_request(format!("Invalid form data: {{}}", e))
    }})?;

    // Validate using derive macro
    if let Err(errors) = form.validate() {{
        return Inertia::render_ctx(&ctx, "{plural_pascal}/Create", {name}CreateProps {{
            errors: Some(errors.into_messages()),
        }});
    }}

    let db = req.db();
    let model = {snake_name}::ActiveModel {{
        id: ActiveValue::NotSet,
{insert_fields}        created_at: ActiveValue::Set(chrono::Utc::now()),
        updated_at: ActiveValue::Set(chrono::Utc::now()),
    }};

    let result = model.insert(db).await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;

    HttpResponse::redirect(&format!("/{plural}/{{}}", result.id))
}}

/// Show edit form
pub async fn edit(req: Request, id: i64) -> Response {{
    let db = req.db();
    let {snake} = {snake_name}::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?
        .ok_or_else(|| HttpResponse::not_found("{name} not found"))?;

    Inertia::render(&req, "{plural_pascal}/Edit", {name}EditProps {{ {snake}, errors: None }})
}}

/// Update an existing {snake}
pub async fn update(req: Request, id: i64) -> Response {{
    let ctx = SavedInertiaContext::from(&req);
    let form: {name}Form = req.input().await.map_err(|e| {{
        HttpResponse::bad_request(format!("Invalid form data: {{}}", e))
    }})?;

    let db = req.db();
    let {snake} = {snake_name}::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?
        .ok_or_else(|| HttpResponse::not_found("{name} not found"))?;

    // Validate using derive macro
    if let Err(errors) = form.validate() {{
        return Inertia::render_ctx(&ctx, "{plural_pascal}/Edit", {name}EditProps {{
            {snake},
            errors: Some(errors.into_messages()),
        }});
    }}

    {snake}
        .update()
{update_fields}        .save()
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;

    HttpResponse::redirect(&format!("/{plural}/{{}}", id))
}}

/// Delete a {snake}
pub async fn destroy(req: Request, id: i64) -> Response {{
    let db = req.db();
    {snake_name}::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?;

    HttpResponse::redirect("/{plural}")
}}
"#,
        name = name,
        snake = snake_name,
        snake_name = snake_name,
        plural = plural_snake,
        plural_pascal = to_pascal_case(plural_snake),
        form_fields = form_fields,
        update_fields = update_fields,
        insert_fields = insert_fields,
    )
}

// ============================================================================
// API Controller Template
// ============================================================================

/// Template for generating API-only controller with make:scaffold --api
pub fn api_controller_template(
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    form_fields: &str,
    update_fields: &str,
    insert_fields: &str,
) -> String {
    format!(
        r#"//! {name} API controller
//!
//! Generated with `ferro make:scaffold --api`

use ferro::{{handler, json_response, Request, Response}};
use crate::models::{snake_name}::{{self, Column, Entity, Model as {name}}};
use sea_orm::{{ColumnTrait, EntityTrait, QueryFilter}};

/// Form data for creating/updating {name}
#[derive(serde::Deserialize)]
pub struct {name}Form {{
{form_fields}
}}

/// List all {plural_snake}
///
/// GET /{plural_snake}
#[handler]
pub async fn index(req: Request) -> Response {{
    let db = req.db();
    let {plural_snake} = Entity::find().all(db).await.map_err(|e| {{
        tracing::error!("Failed to fetch {plural_snake}: {{:?}}", e);
        ferro::error_response!(500, "Failed to fetch {plural_snake}")
    }})?;

    let total = {plural_snake}.len();

    json_response!({{
        "data": {plural_snake},
        "meta": {{
            "total": total
        }}
    }})
}}

/// Get a single {snake_name}
///
/// GET /{plural_snake}/{{id}}
#[handler]
pub async fn show(req: Request) -> Response {{
    let db = req.db();
    let id: i64 = req.param("id").unwrap_or_default();

    let {snake_name} = Entity::find_by_id(id as i32)
        .one(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch {snake_name}")
        }})?
        .ok_or_else(|| ferro::error_response!(404, "{name} not found"))?;

    json_response!({{
        "data": {snake_name}
    }})
}}

/// Create a new {snake_name}
///
/// POST /{plural_snake}
#[handler]
pub async fn store(req: Request) -> Response {{
    let db = req.db();
    let form: {name}Form = req.input().await?;

    let {snake_name} = {snake_name}::ActiveModel {{
{insert_fields}
        ..Default::default()
    }};

    let result = Entity::insert({snake_name})
        .exec(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to create {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to create {snake_name}")
        }})?;

    let created = Entity::find_by_id(result.last_insert_id)
        .one(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch created {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch created {snake_name}")
        }})?
        .ok_or_else(|| ferro::error_response!(500, "Failed to retrieve created {snake_name}"))?;

    json_response!({{
        "data": created,
        "message": "{name} created successfully"
    }})
}}

/// Update an existing {snake_name}
///
/// PUT /{plural_snake}/{{id}}
#[handler]
pub async fn update(req: Request) -> Response {{
    let db = req.db();
    let id: i64 = req.param("id").unwrap_or_default();
    let form: {name}Form = req.input().await?;

    let existing = Entity::find_by_id(id as i32)
        .one(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch {snake_name}")
        }})?
        .ok_or_else(|| ferro::error_response!(404, "{name} not found"))?;

    let updated = existing
        .update()
{update_fields}        .save()
        .await
        .map_err(|e| {{
            tracing::error!("Failed to update {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to update {snake_name}")
        }})?;

    json_response!({{
        "data": updated,
        "message": "{name} updated successfully"
    }})
}}

/// Delete a {snake_name}
///
/// DELETE /{plural_snake}/{{id}}
#[handler]
pub async fn destroy(req: Request) -> Response {{
    let db = req.db();
    let id: i64 = req.param("id").unwrap_or_default();

    let existing = Entity::find_by_id(id as i32)
        .one(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch {snake_name}")
        }})?
        .ok_or_else(|| ferro::error_response!(404, "{name} not found"))?;

    Entity::delete_by_id(existing.id)
        .exec(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to delete {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to delete {snake_name}")
        }})?;

    json_response!({{
        "message": "{name} deleted successfully"
    }})
}}
"#,
        name = name,
        snake_name = snake_name,
        plural_snake = plural_snake,
        form_fields = form_fields,
        update_fields = update_fields,
        insert_fields = insert_fields,
    )
}

/// Template for generating API-only controller with FK nested data support
pub fn api_controller_with_fk_template(
    name: &str,
    snake_name: &str,
    plural_snake: &str,
    form_fields: &str,
    update_fields: &str,
    insert_fields: &str,
    foreign_keys: &[ForeignKeyField],
) -> String {
    // Build FK imports for validated foreign keys
    let fk_imports: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            let target_snake = to_snake_case(&fk.target_model);
            format!(
                "use crate::models::{}::{{Entity as {}Entity, Model as {}}};\n",
                target_snake, fk.target_model, fk.target_model
            )
        })
        .collect();

    // Build FK fetch code for index
    let fk_index_fetches: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            format!(
                r#"
    // Fetch {} for nested data
    let {}_map: std::collections::HashMap<i64, {}> = {}Entity::find()
        .all(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch {}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch {}")
        }})?
        .into_iter()
        .map(|r| (r.id, r))
        .collect();
"#,
                fk.target_model,
                fk.target_table,
                fk.target_model,
                fk.target_model,
                fk.target_table,
                fk.target_table
            )
        })
        .collect();

    // Build response data enrichment for index
    let fk_index_enrich: String = if foreign_keys.iter().any(|fk| fk.validated) {
        let enrichments: String = foreign_keys
            .iter()
            .filter(|fk| fk.validated)
            .map(|fk| {
                let target_snake = to_snake_case(&fk.target_model);
                format!(
                    r#"                "{target_snake}": {target_table}_map.get(&item.{fk_field}).cloned(),"#,
                    target_snake = target_snake,
                    target_table = fk.target_table,
                    fk_field = fk.field_name
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"
    // Enrich data with related entities
    let enriched: Vec<serde_json::Value> = {plural_snake}
        .into_iter()
        .map(|item| {{
            serde_json::json!({{
                "id": item.id,
{enrichments}
                // Include all model fields
                ..serde_json::to_value(&item).unwrap_or_default().as_object().cloned().unwrap_or_default()
            }})
        }})
        .collect();
"#,
            plural_snake = plural_snake,
            enrichments = enrichments
        )
    } else {
        String::new()
    };

    // Build FK fetch code for show
    let fk_show_fetches: String = foreign_keys
        .iter()
        .filter(|fk| fk.validated)
        .map(|fk| {
            let target_snake = to_snake_case(&fk.target_model);
            format!(
                r#"
    // Fetch related {target_model}
    let related_{target_snake} = {target_model}Entity::find_by_id({snake_name}.{fk_field})
        .one(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch related {target_model}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch related {target_model}")
        }})?;
"#,
                target_model = fk.target_model,
                snake_name = snake_name,
                fk_field = fk.field_name,
                target_snake = target_snake,
            )
        })
        .collect();

    // Build show response with nested data
    let fk_show_response: String = if foreign_keys.iter().any(|fk| fk.validated) {
        let nested_fields: String = foreign_keys
            .iter()
            .filter(|fk| fk.validated)
            .map(|fk| {
                let target_snake = to_snake_case(&fk.target_model);
                format!(
                    r#"            "{}": related_{},"#,
                    target_snake, target_snake
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"json_response!({{
        "data": {{
            ..serde_json::to_value(&{snake_name}).unwrap_or_default().as_object().cloned().unwrap_or_default(),
{nested_fields}
        }}
    }})"#,
            snake_name = snake_name,
            nested_fields = nested_fields
        )
    } else {
        format!(
            r#"json_response!({{
        "data": {snake_name}
    }})"#,
            snake_name = snake_name
        )
    };

    // Generate validated FK comment if there are unvalidated FKs
    let unvalidated_fks: Vec<_> = foreign_keys.iter().filter(|fk| !fk.validated).collect();
    let unvalidated_comment = if !unvalidated_fks.is_empty() {
        let fk_list: String = unvalidated_fks
            .iter()
            .map(|fk| {
                format!(
                    "// - {} (model {} not found)",
                    fk.field_name, fk.target_model
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "\n// TODO: The following FK fields have no corresponding model:\n{}\n// Create these models to enable nested data in responses.\n",
            fk_list
        )
    } else {
        String::new()
    };

    // Determine if we use enriched data or raw data in index
    let has_validated_fks = foreign_keys.iter().any(|fk| fk.validated);
    let index_data_var = if has_validated_fks {
        "enriched"
    } else {
        plural_snake
    };

    format!(
        r#"//! {name} API controller
//!
//! Generated with `ferro make:scaffold --api`
{unvalidated_comment}
use ferro::{{handler, json_response, Request, Response}};
use crate::models::{snake_name}::{{self, Column, Entity, Model as {name}}};
use sea_orm::{{ColumnTrait, EntityTrait, QueryFilter}};
{fk_imports}
/// Form data for creating/updating {name}
#[derive(serde::Deserialize)]
pub struct {name}Form {{
{form_fields}
}}

/// List all {plural_snake} with nested related data
///
/// GET /{plural_snake}
#[handler]
pub async fn index(req: Request) -> Response {{
    let db = req.db();
    let {plural_snake} = Entity::find().all(db).await.map_err(|e| {{
        tracing::error!("Failed to fetch {plural_snake}: {{:?}}", e);
        ferro::error_response!(500, "Failed to fetch {plural_snake}")
    }})?;
{fk_index_fetches}{fk_index_enrich}
    let total = {index_data_var}.len();

    json_response!({{
        "data": {index_data_var},
        "meta": {{
            "total": total
        }}
    }})
}}

/// Get a single {snake_name} with nested related data
///
/// GET /{plural_snake}/{{id}}
#[handler]
pub async fn show(req: Request) -> Response {{
    let db = req.db();
    let id: i64 = req.param("id").unwrap_or_default();

    let {snake_name} = Entity::find_by_id(id as i32)
        .one(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch {snake_name}")
        }})?
        .ok_or_else(|| ferro::error_response!(404, "{name} not found"))?;
{fk_show_fetches}
    {fk_show_response}
}}

/// Create a new {snake_name}
///
/// POST /{plural_snake}
#[handler]
pub async fn store(req: Request) -> Response {{
    let db = req.db();
    let form: {name}Form = req.input().await?;

    let {snake_name} = {snake_name}::ActiveModel {{
{insert_fields}
        ..Default::default()
    }};

    let result = Entity::insert({snake_name})
        .exec(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to create {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to create {snake_name}")
        }})?;

    let created = Entity::find_by_id(result.last_insert_id)
        .one(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch created {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch created {snake_name}")
        }})?
        .ok_or_else(|| ferro::error_response!(500, "Failed to retrieve created {snake_name}"))?;

    json_response!({{
        "data": created,
        "message": "{name} created successfully"
    }})
}}

/// Update an existing {snake_name}
///
/// PUT /{plural_snake}/{{id}}
#[handler]
pub async fn update(req: Request) -> Response {{
    let db = req.db();
    let id: i64 = req.param("id").unwrap_or_default();
    let form: {name}Form = req.input().await?;

    let existing = Entity::find_by_id(id as i32)
        .one(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch {snake_name}")
        }})?
        .ok_or_else(|| ferro::error_response!(404, "{name} not found"))?;

    let updated = existing
        .update()
{update_fields}        .save()
        .await
        .map_err(|e| {{
            tracing::error!("Failed to update {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to update {snake_name}")
        }})?;

    json_response!({{
        "data": updated,
        "message": "{name} updated successfully"
    }})
}}

/// Delete a {snake_name}
///
/// DELETE /{plural_snake}/{{id}}
#[handler]
pub async fn destroy(req: Request) -> Response {{
    let db = req.db();
    let id: i64 = req.param("id").unwrap_or_default();

    let existing = Entity::find_by_id(id as i32)
        .one(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to fetch {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to fetch {snake_name}")
        }})?
        .ok_or_else(|| ferro::error_response!(404, "{name} not found"))?;

    Entity::delete_by_id(existing.id)
        .exec(db)
        .await
        .map_err(|e| {{
            tracing::error!("Failed to delete {snake_name}: {{:?}}", e);
            ferro::error_response!(500, "Failed to delete {snake_name}")
        }})?;

    json_response!({{
        "message": "{name} deleted successfully"
    }})
}}
"#,
        name = name,
        snake_name = snake_name,
        plural_snake = plural_snake,
        form_fields = form_fields,
        update_fields = update_fields,
        insert_fields = insert_fields,
        fk_imports = fk_imports,
        fk_index_fetches = fk_index_fetches,
        fk_index_enrich = fk_index_enrich,
        fk_show_fetches = fk_show_fetches,
        fk_show_response = fk_show_response,
        unvalidated_comment = unvalidated_comment,
        index_data_var = index_data_var,
    )
}

// ============================================================================
// Auth scaffolding templates
// ============================================================================

/// Migration template that adds auth fields to an existing users table.
///
/// Uses ALTER TABLE to add name, email (unique), password, and remember_token.
pub fn auth_migration_template() -> String {
    r#"use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add auth fields to existing users table
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::Name).string().not_null().default(""))
                    .add_column(ColumnDef::new(Users::Email).string().not_null().default(""))
                    .add_column(ColumnDef::new(Users::Password).string().not_null().default(""))
                    .add_column(ColumnDef::new(Users::RememberToken).string().null())
                    .to_owned(),
            )
            .await?;

        // Add unique index on email
        manager
            .create_index(
                Index::create()
                    .name("idx_users_email_unique")
                    .table(Users::Table)
                    .col(Users::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_email_unique")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Name)
                    .drop_column(Users::Email)
                    .drop_column(Users::Password)
                    .drop_column(Users::RememberToken)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Name,
    Email,
    Password,
    RememberToken,
}
"#
    .to_string()
}

/// Auth controller template with register, login, and logout handlers.
pub fn auth_controller_template() -> String {
    r#"//! Authentication controller
//!
//! Handles user registration, login, and logout.
//!
//! Tip: Use AuthUser<users::Model> to auto-extract the authenticated user:
//!
//!   use ferro::AuthUser;
//!
//!   #[handler]
//!   pub async fn profile(user: AuthUser<users::Model>) -> Response {
//!       Ok(HttpResponse::json(serde_json::json!({"user": user.name})))
//!   }

use ferro::database::ModelMut;
use ferro::http::{HttpResponse, Request, Response};
use ferro::{handler, hash, json_response, rules, verify};
use ferro::{Auth, Validator, required, string, email, min};
use sea_orm::ActiveValue;
use serde::Deserialize;

use crate::models::users;

#[derive(Deserialize)]
struct RegisterInput {
    name: String,
    email: String,
    password: String,
    password_confirmation: String,
}

#[derive(Deserialize)]
struct LoginInput {
    email: String,
    password: String,
}

/// Register a new user
#[handler]
pub async fn register(req: Request) -> Response {
    let input: RegisterInput = req.input().await.map_err(|_| {
        HttpResponse::json(serde_json::json!({
            "message": "Invalid request body."
        }))
        .status(422)
    })?;

    // Validate input
    let data = serde_json::json!({
        "name": input.name,
        "email": input.email,
        "password": input.password,
        "password_confirmation": input.password_confirmation,
    });

    let mut validator = Validator::new(&data)
        .rules("name", rules![required(), string()])
        .rules("email", rules![required(), email()])
        .rules("password", rules![required(), string(), min(8)]);

    // Check password confirmation
    if input.password != input.password_confirmation {
        validator = validator.with_error("password_confirmation", "Passwords do not match.");
    }

    // Check email uniqueness
    if let Some(_existing) = users::Model::find_by_email(&input.email).await.map_err(|e| {
        HttpResponse::json(serde_json::json!({
            "message": format!("Database error: {}", e)
        }))
        .status(500)
    })? {
        validator = validator.with_error("email", "This email is already registered.");
    }

    if let Err(errors) = validator.validate() {
        return Err(HttpResponse::json(serde_json::json!({
            "message": "Validation failed.",
            "errors": errors,
        }))
        .status(422));
    }

    // Hash password
    let password_hash = hash(&input.password).map_err(|e| {
        HttpResponse::json(serde_json::json!({
            "message": format!("Failed to hash password: {}", e)
        }))
        .status(500)
    })?;

    // Create user
    let user = users::ActiveModel {
        name: ActiveValue::Set(input.name.clone()),
        email: ActiveValue::Set(input.email.clone()),
        password: ActiveValue::Set(password_hash),
        remember_token: ActiveValue::Set(None),
        ..Default::default()
    };

    let user = users::Entity::insert(user)
        .exec_with_returning(&ferro::database::connection().await)
        .await
        .map_err(|e| {
            HttpResponse::json(serde_json::json!({
                "message": format!("Failed to create user: {}", e)
            }))
            .status(500)
        })?;

    // Log in the new user
    Auth::login(user.id as i64);

    Ok(HttpResponse::json(serde_json::json!({
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
        }
    }))
    .status(201))
}

/// Log in an existing user
#[handler]
pub async fn login(req: Request) -> Response {
    let input: LoginInput = req.input().await.map_err(|_| {
        HttpResponse::json(serde_json::json!({
            "message": "Invalid request body."
        }))
        .status(422)
    })?;

    // Validate input
    let data = serde_json::json!({
        "email": input.email,
        "password": input.password,
    });

    if let Err(errors) = Validator::new(&data)
        .rules("email", rules![required(), email()])
        .rules("password", rules![required()])
        .validate()
    {
        return Err(HttpResponse::json(serde_json::json!({
            "message": "Validation failed.",
            "errors": errors,
        }))
        .status(422));
    }

    // Attempt authentication
    let email = input.email.clone();
    let password = input.password.clone();

    let result = Auth::attempt(|| async {
        let user = users::Model::find_by_email(&email).await?;
        match user {
            Some(user) => {
                if verify(&password, &user.password)? {
                    Ok(Some(user.id as i64))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    })
    .await;

    match result {
        Ok(Some(_id)) => {
            // Re-fetch user for response
            let user = users::Model::find_by_email(&input.email)
                .await
                .map_err(|e| {
                    HttpResponse::json(serde_json::json!({
                        "message": format!("Database error: {}", e)
                    }))
                    .status(500)
                })?;

            match user {
                Some(user) => json_response!({
                    "user": {
                        "id": user.id,
                        "name": user.name,
                        "email": user.email,
                    }
                }),
                None => Err(HttpResponse::json(serde_json::json!({
                    "email": ["These credentials do not match our records."]
                }))
                .status(422)),
            }
        }
        Ok(None) => Err(HttpResponse::json(serde_json::json!({
            "email": ["These credentials do not match our records."]
        }))
        .status(422)),
        Err(e) => Err(HttpResponse::json(serde_json::json!({
            "message": format!("Authentication error: {}", e)
        }))
        .status(500)),
    }
}

/// Log out the current user
#[handler]
pub async fn logout(_req: Request) -> Response {
    Auth::logout();
    json_response!({
        "message": "Logged out successfully."
    })
}
"#
    .to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Backend Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cargo_toml_substitution() {
        let result = cargo_toml("my_app", "A test app", "Test Author <test@example.com>");
        assert!(result.contains("name = \"my_app\""));
        assert!(result.contains("description = \"A test app\""));
        assert!(result.contains("authors = [\"Test Author <test@example.com>\"]"));
    }

    #[test]
    fn test_cargo_toml_empty_author() {
        let result = cargo_toml("my_app", "A test app", "");
        assert!(result.contains("name = \"my_app\""));
        assert!(!result.contains("authors = "));
    }

    #[test]
    fn test_main_rs_substitution() {
        let result = main_rs("my_app");
        assert!(result.contains("my_app"));
    }

    #[test]
    fn test_routes_rs_not_empty() {
        assert!(!routes_rs().is_empty());
        assert!(routes_rs().contains("routes"));
    }

    #[test]
    fn test_controllers_mod_not_empty() {
        assert!(!controllers_mod().is_empty());
        assert!(controllers_mod().contains("auth"));
        assert!(controllers_mod().contains("dashboard"));
        assert!(controllers_mod().contains("profile"));
        assert!(controllers_mod().contains("settings"));
    }

    #[test]
    fn test_home_controller_not_empty() {
        assert!(!home_controller().is_empty());
        assert!(home_controller().contains("async fn index"));
    }

    #[test]
    fn test_auth_controller_not_empty() {
        assert!(!auth_controller().is_empty());
        assert!(auth_controller().contains("login"));
        assert!(auth_controller().contains("register"));
    }

    #[test]
    fn test_dashboard_controller_not_empty() {
        assert!(!dashboard_controller().is_empty());
        assert!(dashboard_controller().contains("Dashboard"));
    }

    #[test]
    fn test_profile_controller_not_empty() {
        let content = profile_controller();
        assert!(!content.is_empty());
        assert!(content.contains("Profile"));
        assert!(content.contains("async fn"));
    }

    #[test]
    fn test_settings_controller_not_empty() {
        let content = settings_controller();
        assert!(!content.is_empty());
        assert!(content.contains("Settings"));
        assert!(content.contains("async fn"));
    }

    // -------------------------------------------------------------------------
    // Middleware Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_middleware_mod_not_empty() {
        assert!(!middleware_mod().is_empty());
        assert!(middleware_mod().contains("logging"));
    }

    #[test]
    fn test_middleware_template_substitution() {
        let result = middleware_template("auth", "AuthMiddleware");
        assert!(result.contains("auth middleware"));
        assert!(result.contains("pub struct AuthMiddleware"));
        assert!(result.contains("impl Middleware for AuthMiddleware"));
    }

    #[test]
    fn test_authenticate_middleware_not_empty() {
        assert!(!authenticate_middleware().is_empty());
        assert!(authenticate_middleware().contains("Middleware"));
    }

    // -------------------------------------------------------------------------
    // Model Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_models_mod_not_empty() {
        let content = models_mod();
        assert!(!content.is_empty());
        assert!(content.contains("user"));
        assert!(content.contains("password_reset_tokens"));
    }

    #[test]
    fn test_user_model_not_empty() {
        assert!(!user_model().is_empty());
        assert!(user_model().contains("User"));
    }

    #[test]
    fn test_password_reset_tokens_model_not_empty() {
        let content = password_reset_tokens_model();
        assert!(!content.is_empty());
        assert!(content.contains("password_reset_tokens"));
        assert!(content.contains("email"));
        assert!(content.contains("token"));
    }

    // -------------------------------------------------------------------------
    // Migration Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_migrations_mod_not_empty() {
        let content = migrations_mod();
        assert!(!content.is_empty());
        assert!(content.contains("create_users_table"));
        assert!(content.contains("create_sessions_table"));
        assert!(content.contains("create_password_reset_tokens_table"));
    }

    #[test]
    fn test_create_users_migration_not_empty() {
        assert!(!create_users_migration().is_empty());
        assert!(create_users_migration().contains("Users"));
    }

    #[test]
    fn test_create_sessions_migration_not_empty() {
        assert!(!create_sessions_migration().is_empty());
        assert!(create_sessions_migration().contains("sessions"));
    }

    #[test]
    fn test_create_password_reset_tokens_migration_not_empty() {
        let content = create_password_reset_tokens_migration();
        assert!(!content.is_empty());
        assert!(content.contains("password_reset_tokens"));
    }

    // -------------------------------------------------------------------------
    // Config Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_config_mod_not_empty() {
        assert!(!config_mod().is_empty());
        assert!(config_mod().contains("database"));
    }

    #[test]
    fn test_config_database_not_empty() {
        assert!(!config_database().is_empty());
    }

    #[test]
    fn test_config_mail_not_empty() {
        assert!(!config_mail().is_empty());
    }

    // -------------------------------------------------------------------------
    // Frontend Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_package_json_substitution() {
        let result = package_json("my-project");
        assert!(result.contains("\"name\": \"my-project-frontend\""));
    }

    #[test]
    fn test_vite_config_not_empty() {
        assert!(!vite_config().is_empty());
        assert!(vite_config().contains("vite"));
    }

    #[test]
    fn test_tsconfig_not_empty() {
        assert!(!tsconfig().is_empty());
        assert!(tsconfig().contains("compilerOptions"));
    }

    #[test]
    fn test_index_html_substitution() {
        let result = index_html("My App");
        assert!(result.contains("<title>My App</title>"));
    }

    #[test]
    fn test_main_tsx_not_empty() {
        assert!(!main_tsx().is_empty());
        assert!(main_tsx().contains("createInertiaApp"));
    }

    #[test]
    fn test_home_page_not_empty() {
        assert!(!home_page().is_empty());
        assert!(home_page().contains("Home"));
    }

    #[test]
    fn test_inertia_props_types_not_empty() {
        let content = inertia_props_types();
        assert!(!content.is_empty());
        assert!(content.contains("User"));
        assert!(content.contains("DashboardProps"));
        assert!(content.contains("ProfileProps"));
        assert!(content.contains("SettingsProps"));
    }

    // -------------------------------------------------------------------------
    // Frontend Layout Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_app_layout_not_empty() {
        let content = app_layout();
        assert!(!content.is_empty());
        assert!(content.contains("AppLayout"));
        assert!(content.contains("Sidebar"));
    }

    #[test]
    fn test_auth_layout_not_empty() {
        let content = auth_layout();
        assert!(!content.is_empty());
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_layouts_index_not_empty() {
        let content = layouts_index();
        assert!(!content.is_empty());
        assert!(content.contains("AppLayout"));
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_globals_css_not_empty() {
        let content = globals_css();
        assert!(!content.is_empty());
        // Tailwind CSS v4 uses @import "tailwindcss" instead of @tailwind directives
        assert!(content.contains("tailwindcss"));
    }

    // -------------------------------------------------------------------------
    // Frontend Auth Page Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_login_page_not_empty() {
        let content = login_page();
        assert!(!content.is_empty());
        assert!(content.contains("Login"));
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_register_page_not_empty() {
        let content = register_page();
        assert!(!content.is_empty());
        assert!(content.contains("Register"));
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_forgot_password_page_not_empty() {
        let content = forgot_password_page();
        assert!(!content.is_empty());
        assert!(content.contains("ForgotPassword"));
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_reset_password_page_not_empty() {
        let content = reset_password_page();
        assert!(!content.is_empty());
        assert!(content.contains("ResetPassword"));
        assert!(content.contains("AuthLayout"));
    }

    // -------------------------------------------------------------------------
    // Frontend User Page Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dashboard_page_not_empty() {
        let content = dashboard_page();
        assert!(!content.is_empty());
        assert!(content.contains("Dashboard"));
        assert!(content.contains("AppLayout"));
    }

    #[test]
    fn test_profile_page_not_empty() {
        let content = profile_page();
        assert!(!content.is_empty());
        assert!(content.contains("Profile"));
        assert!(content.contains("AppLayout"));
    }

    #[test]
    fn test_settings_page_not_empty() {
        let content = settings_page();
        assert!(!content.is_empty());
        assert!(content.contains("Settings"));
        assert!(content.contains("AppLayout"));
    }

    // -------------------------------------------------------------------------
    // Controller Template Generation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_controller_template_substitution() {
        let result = controller_template("users");
        assert!(result.contains("users controller"));
        assert!(result.contains("#[handler]"));
    }

    // -------------------------------------------------------------------------
    // Action Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_action_template_substitution() {
        let result = action_template("create_user", "CreateUser");
        assert!(result.contains("create_user action"));
        assert!(result.contains("pub struct CreateUser"));
        assert!(result.contains("#[injectable]"));
    }

    #[test]
    fn test_actions_mod_not_empty() {
        assert!(!actions_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Error Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_error_template_substitution() {
        let result = error_template("UserNotFound");
        assert!(result.contains("UserNotFound error"));
        assert!(result.contains("pub struct UserNotFound"));
        assert!(result.contains("#[domain_error"));
    }

    // -------------------------------------------------------------------------
    // Inertia Page Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_inertia_page_template_substitution() {
        let result = inertia_page_template("Users");
        assert!(result.contains("export default function Users()"));
        assert!(result.contains("<h1"));
    }

    // -------------------------------------------------------------------------
    // Event/Listener/Job Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_event_template_substitution() {
        let result = event_template("user_registered", "UserRegistered");
        assert!(result.contains("UserRegistered"));
        assert!(result.contains("impl Event for UserRegistered"));
    }

    #[test]
    fn test_listener_template_substitution() {
        let result = listener_template("send_welcome_email", "SendWelcomeEmail", "UserRegistered");
        assert!(result.contains("SendWelcomeEmail"));
        assert!(result.contains("impl Listener<UserRegistered>"));
    }

    #[test]
    fn test_job_template_substitution() {
        let result = job_template("send_email", "SendEmail");
        assert!(result.contains("SendEmail"));
        assert!(result.contains("impl Job for SendEmail"));
    }

    #[test]
    fn test_events_mod_not_empty() {
        assert!(!events_mod().is_empty());
    }

    #[test]
    fn test_listeners_mod_not_empty() {
        assert!(!listeners_mod().is_empty());
    }

    #[test]
    fn test_jobs_mod_not_empty() {
        assert!(!jobs_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Notification Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_notification_template_substitution() {
        let result = notification_template("order_shipped", "OrderShipped");
        assert!(result.contains("OrderShipped"));
        assert!(result.contains("impl Notification for OrderShipped"));
    }

    #[test]
    fn test_notifications_mod_not_empty() {
        assert!(!notifications_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Task Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_task_template_substitution() {
        let result = task_template("cleanup_old_sessions", "CleanupOldSessions");
        assert!(result.contains("CleanupOldSessions"));
        assert!(result.contains("impl Task for CleanupOldSessions"));
    }

    #[test]
    fn test_tasks_mod_not_empty() {
        assert!(!tasks_mod().is_empty());
    }

    #[test]
    fn test_schedule_rs_not_empty() {
        assert!(!schedule_rs().is_empty());
    }

    // -------------------------------------------------------------------------
    // Seeder Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_seeder_template_substitution() {
        let result = seeder_template("users_seeder", "UsersSeeder");
        assert!(result.contains("UsersSeeder"));
        assert!(result.contains("impl Seeder for UsersSeeder"));
    }

    #[test]
    fn test_seeders_mod_not_empty() {
        assert!(!seeders_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Factory Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_factory_template_substitution() {
        let result = factory_template("user_factory", "UserFactory", "User");
        assert!(result.contains("UserFactory"));
        assert!(result.contains("impl Factory for UserFactory"));
    }

    #[test]
    fn test_factories_mod_not_empty() {
        assert!(!factories_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Policy Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_policy_template_substitution() {
        let result = policy_template("post_policy", "PostPolicy", "Post");
        assert!(result.contains("PostPolicy"));
        assert!(result.contains("impl Policy<Post>"));
    }

    #[test]
    fn test_policies_mod_not_empty() {
        assert!(!policies_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Docker Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dockerfile_template_substitution() {
        let result = dockerfile_template("my_app");
        assert!(result.contains("my_app"));
    }

    #[test]
    fn test_dockerignore_template_not_empty() {
        assert!(!dockerignore_template().is_empty());
    }

    #[test]
    fn test_docker_compose_template_basic() {
        let result = docker_compose_template("my_project", false, false);
        assert!(result.contains("my_project"));
        assert!(result.contains("postgres"));
    }

    #[test]
    fn test_docker_compose_template_with_mailpit() {
        let result = docker_compose_template("my_project", true, false);
        assert!(result.contains("mailpit"));
    }

    #[test]
    fn test_docker_compose_template_with_minio() {
        let result = docker_compose_template("my_project", false, true);
        assert!(result.contains("minio"));
    }

    // -------------------------------------------------------------------------
    // Root File Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_gitignore_not_empty() {
        assert!(!gitignore().is_empty());
        assert!(gitignore().contains("target"));
    }

    #[test]
    fn test_env_substitution() {
        let result = env("my_project");
        assert!(result.contains("my_project"));
    }

    #[test]
    fn test_env_example_not_empty() {
        assert!(!env_example().is_empty());
    }

    // -------------------------------------------------------------------------
    // AI Development Boost Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_ferro_guidelines_template_not_empty() {
        let content = ferro_guidelines_template();
        assert!(!content.is_empty());
        assert!(content.contains("Ferro Framework"));
    }

    #[test]
    fn test_cursor_rules_template_not_empty() {
        let content = cursor_rules_template();
        assert!(!content.is_empty());
        assert!(content.contains("Ferro"));
    }

    #[test]
    fn test_claude_md_template_not_empty() {
        let content = claude_md_template();
        assert!(!content.is_empty());
        assert!(content.contains("Ferro"));
    }

    #[test]
    fn test_copilot_instructions_template_not_empty() {
        let content = copilot_instructions_template();
        assert!(!content.is_empty());
        assert!(content.contains("Ferro"));
    }

    // -------------------------------------------------------------------------
    // Entity Generation Helper Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_entity_template_generates_valid_rust() {
        let columns = vec![
            ColumnInfo {
                name: "id".to_string(),
                col_type: "INTEGER".to_string(),
                is_nullable: false,
                is_primary_key: true,
            },
            ColumnInfo {
                name: "name".to_string(),
                col_type: "VARCHAR".to_string(),
                is_nullable: false,
                is_primary_key: false,
            },
            ColumnInfo {
                name: "email".to_string(),
                col_type: "VARCHAR".to_string(),
                is_nullable: true,
                is_primary_key: false,
            },
        ];

        let result = entity_template("users", &columns);
        assert!(result.contains("table_name = \"users\""));
        assert!(result.contains("pub id: i32"));
        assert!(result.contains("pub name: String"));
        assert!(result.contains("pub email: Option<String>"));
        assert!(result.contains("#[sea_orm(primary_key)]"));
    }

    #[test]
    fn test_entity_template_handles_reserved_keywords() {
        let columns = vec![
            ColumnInfo {
                name: "id".to_string(),
                col_type: "INTEGER".to_string(),
                is_nullable: false,
                is_primary_key: true,
            },
            ColumnInfo {
                name: "type".to_string(),
                col_type: "VARCHAR".to_string(),
                is_nullable: false,
                is_primary_key: false,
            },
        ];

        let result = entity_template("items", &columns);
        assert!(result.contains("pub r#type: String"));
        assert!(result.contains("column_name = \"type\""));
    }

    #[test]
    fn test_user_model_template_generates_minimal_file() {
        let columns = vec![
            ColumnInfo {
                name: "id".to_string(),
                col_type: "INTEGER".to_string(),
                is_nullable: false,
                is_primary_key: true,
            },
            ColumnInfo {
                name: "name".to_string(),
                col_type: "VARCHAR".to_string(),
                is_nullable: false,
                is_primary_key: false,
            },
        ];

        let result = user_model_template("users", "User", &columns);
        // Type alias for convenient access
        assert!(result.contains("pub type User = Model"));
        // Re-exports entity module
        assert!(result.contains("pub use super::entities::users::*"));
        // Users table should have Authenticatable impl
        assert!(result.contains("impl ferro::auth::Authenticatable for Model"));
        // Should NOT contain manual method implementations (now generated by FerroModel macro)
        assert!(!result.contains("pub fn query()"));
        assert!(!result.contains("pub fn create()"));
        assert!(!result.contains("pub struct UserBuilder"));
    }

    #[test]
    fn test_entity_template_includes_ferro_model_derive() {
        let columns = vec![ColumnInfo {
            name: "id".to_string(),
            col_type: "INTEGER".to_string(),
            is_nullable: false,
            is_primary_key: true,
        }];

        let result = entity_template("users", &columns);
        // Should include FerroModel in derives
        assert!(result.contains("FerroModel"));
        assert!(result.contains("use ferro::FerroModel"));
    }

    #[test]
    fn test_entities_mod_template() {
        let tables = vec![
            TableInfo {
                name: "users".to_string(),
                columns: vec![],
            },
            TableInfo {
                name: "posts".to_string(),
                columns: vec![],
            },
        ];

        let result = entities_mod_template(&tables);
        assert!(result.contains("pub mod users;"));
        assert!(result.contains("pub mod posts;"));
    }

    // -------------------------------------------------------------------------
    // SQL Type Conversion Tests (via entity_template)
    // -------------------------------------------------------------------------

    #[test]
    fn test_sql_type_conversions() {
        let test_cases = vec![
            ("BIGINT", "i64"),
            ("INT8", "i64"),
            ("SMALLINT", "i16"),
            ("INT2", "i16"),
            ("INTEGER", "i32"),
            ("INT", "i32"),
            ("TEXT", "String"),
            ("VARCHAR(255)", "String"),
            ("CHAR(10)", "String"),
            ("BOOLEAN", "bool"),
            ("BOOL", "bool"),
            ("REAL", "f32"),
            ("FLOAT4", "f32"),
            ("DOUBLE", "f64"),
            ("FLOAT8", "f64"),
            ("TIMESTAMP", "DateTimeUtc"),
            ("DATETIME", "DateTimeUtc"),
            ("DATE", "Date"),
            ("TIME", "Time"),
            ("UUID", "Uuid"),
            ("JSON", "Json"),
            ("JSONB", "Json"),
            ("BYTEA", "Vec<u8>"),
            ("BLOB", "Vec<u8>"),
            ("DECIMAL", "Decimal"),
            ("NUMERIC", "Decimal"),
        ];

        for (sql_type, expected_rust_type) in test_cases {
            let columns = vec![ColumnInfo {
                name: "test_col".to_string(),
                col_type: sql_type.to_string(),
                is_nullable: false,
                is_primary_key: false,
            }];

            let result = entity_template("test_table", &columns);
            assert!(
                result.contains(&format!("pub test_col: {}", expected_rust_type)),
                "Failed for SQL type '{}': expected Rust type '{}' not found in:\n{}",
                sql_type,
                expected_rust_type,
                result
            );
        }
    }

    #[test]
    fn test_nullable_types() {
        let columns = vec![ColumnInfo {
            name: "optional_name".to_string(),
            col_type: "VARCHAR".to_string(),
            is_nullable: true,
            is_primary_key: false,
        }];

        let result = entity_template("test_table", &columns);
        assert!(result.contains("pub optional_name: Option<String>"));
    }

    // -------------------------------------------------------------------------
    // API Controller Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_api_controller_template_substitution() {
        let result = api_controller_template(
            "Post",
            "post",
            "posts",
            "    pub title: String,\n    pub body: String,",
            "        .set_title(form.title.clone())\n        .set_body(form.body.clone())\n",
            "        title: sea_orm::ActiveValue::Set(form.title.clone()),\n        body: sea_orm::ActiveValue::Set(form.body.clone()),",
        );
        assert!(result.contains("Post API controller"));
        assert!(result.contains("pub async fn index"));
        assert!(result.contains("pub async fn show"));
        assert!(result.contains("pub async fn store"));
        assert!(result.contains("pub async fn update"));
        assert!(result.contains("pub async fn destroy"));
        assert!(result.contains("json_response!"));
        assert!(!result.contains("Inertia"));
        // Verify builder pattern is used in update handler
        assert!(result.contains(".update()"));
        assert!(result.contains(".set_title("));
        assert!(result.contains(".save()"));
        // Verify old ActiveModel pattern is NOT used in update handler
        assert!(!result.contains("let mut post: post::ActiveModel"));
    }
}
