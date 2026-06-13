use super::entity::to_pascal_case;
use super::entity::to_snake_case;

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
"#
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
            "\n// TODO: The following FK fields have no corresponding model:\n{fk_list}\n// Create these models to enable relationship loading.\n"
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
    ActiveValue, ValidateRules,
}};
use sea_orm::{{EntityTrait, ActiveModelTrait}};
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
    ActiveValue, ValidateRules,
}};
use sea_orm::{{EntityTrait, ActiveModelTrait}};
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

use ferro::{{handler, json_response, ActiveValue, Request, Response, ValidateRules}};
use crate::models::{snake_name}::{{self, Column, Entity, Model as {name}}};
use sea_orm::{{ColumnTrait, EntityTrait, QueryFilter}};

/// Form data for creating/updating {name}
#[derive(Debug, serde::Deserialize, ValidateRules)]
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
"#
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
                format!(r#"            "{target_snake}": related_{target_snake},"#)
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"json_response!({{
        "data": {{
            ..serde_json::to_value(&{snake_name}).unwrap_or_default().as_object().cloned().unwrap_or_default(),
{nested_fields}
        }}
    }})"#
        )
    } else {
        format!(
            r#"json_response!({{
        "data": {snake_name}
    }})"#
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
            "\n// TODO: The following FK fields have no corresponding model:\n{fk_list}\n// Create these models to enable nested data in responses.\n"
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
use ferro::{{handler, json_response, ActiveValue, Request, Response, ValidateRules}};
use crate::models::{snake_name}::{{self, Column, Entity, Model as {name}}};
use sea_orm::{{ColumnTrait, EntityTrait, QueryFilter}};
{fk_imports}
/// Form data for creating/updating {name}
#[derive(Debug, serde::Deserialize, ValidateRules)]
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
    )
}
