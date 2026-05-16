//! Code templates tool - returns copy-paste code templates for common patterns

use serde::Serialize;

/// Collection of code templates
#[derive(Debug, Serialize)]
pub struct CodeTemplates {
    pub templates: Vec<CodeTemplate>,
}

/// A single code template with metadata
#[derive(Debug, Serialize)]
pub struct CodeTemplate {
    pub name: String,
    pub category: String,
    pub description: String,
    pub code: String,
    pub imports: Vec<String>,
    pub placeholders: Vec<Placeholder>,
}

/// A placeholder in a template that needs to be replaced
#[derive(Debug, Serialize)]
pub struct Placeholder {
    pub name: String,
    pub description: String,
    pub example: String,
}

/// Execute the code templates tool
///
/// # Arguments
/// * `category` - Optional filter by category (handler, model, migration, middleware, validation, json_view, rate_limiting, broadcasting, api)
pub fn execute(category: Option<&str>) -> CodeTemplates {
    let all_templates = build_templates();

    let templates = match category {
        Some(cat) => all_templates
            .into_iter()
            .filter(|t| t.category == cat)
            .collect(),
        None => all_templates,
    };

    CodeTemplates { templates }
}

fn build_templates() -> Vec<CodeTemplate> {
    let mut templates = Vec::new();

    // Handler templates
    templates.extend(handler_templates());

    // Model templates
    templates.extend(model_templates());

    // Migration templates
    templates.extend(migration_templates());

    // Middleware templates
    templates.extend(middleware_templates());

    // Validation templates
    templates.extend(validation_templates());

    // JSON-UI view templates
    templates.extend(json_view_templates());

    // Rate limiting templates
    templates.extend(rate_limiting_templates());

    // Broadcasting templates
    templates.extend(broadcasting_templates());

    // API scaffold templates
    templates.extend(api_templates());

    // v1 → v2 migration patterns
    templates.extend(migration_v1_to_v2_templates());

    templates
}

fn handler_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "index_handler".to_string(),
            category: "handler".to_string(),
            description: "List all resources with pagination using ResourceCollection".to_string(),
            code: r#"#[handler]
pub async fn index(req: Request) -> Response {
    let db = req.db();
    let page: u64 = req.query("page").unwrap_or(1);
    let per_page: u64 = req.query("per_page").unwrap_or(20);

    let paginator = {{Entity}}::find()
        .order_by_desc({{entity}}::Column::Id)
        .paginate(db, per_page);

    let items = paginator.fetch_page(page - 1).await?;
    let total = paginator.num_items().await?;

    let resources: Vec<{{Entity}}Resource> = items.into_iter()
        .map({{Entity}}Resource::from)
        .collect();

    let meta = PaginationMeta::new(page, per_page, total);
    Ok(ResourceCollection::paginated(resources, meta).to_response(&req))
}"#
            .to_string(),
            imports: vec![
                "use ferro::{handler, Request, Response, HttpResponse, PaginationMeta, ResourceCollection};".to_string(),
                "use crate::entities::{{entity}};".to_string(),
                "use crate::entities::{{entity}}::Entity as {{Entity}};".to_string(),
                "use crate::resources::{{Entity}}Resource;".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Model name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Model name in snake_case".to_string(),
                    example: "user".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "show_handler".to_string(),
            category: "handler".to_string(),
            description: "Get single resource by ID".to_string(),
            code: r#"#[handler]
pub async fn show(req: Request, id: Path<i32>) -> Response {
    let db = req.db();
    let item = {{Entity}}::find_by_id(*id)
        .one(db)
        .await?
        .ok_or_else(|| not_found("{{Entity}} not found"))?;

    Ok(json!(item))
}"#
            .to_string(),
            imports: vec![
                "use ferro::{handler, Request, Response, HttpResponse, AppError};".to_string(),
                "use serde_json::json;".to_string(),
                "use crate::entities::{{entity}}::Entity as {{Entity}};".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Model name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Model name in snake_case".to_string(),
                    example: "user".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "create_handler".to_string(),
            category: "handler".to_string(),
            description: "Create resource with validation".to_string(),
            code: r#"#[handler]
pub async fn create(req: Request) -> Response {
    let db = req.db();
    let data = req.input::<Create{{Entity}}Request>().await?;

    Validator::new(&data)
        .rules("name", rules![required()])
        // Add more validation rules
        .validate()?;

    let model = {{entity}}::ActiveModel {
        name: Set(data.name),
        // Set other fields
        ..Default::default()
    };

    let result = model.insert(db).await?;

    Ok(json!(result).status(201))
}"#
            .to_string(),
            imports: vec![
                "use ferro::{handler, Request, Response, HttpResponse, ResponseExt, Validator, required, min, max};".to_string(),
                "use serde_json::json;".to_string(),
                "use serde::Deserialize;".to_string(),
                "use crate::entities::{{entity}};".to_string(),
                "use sea_orm::ActiveModelTrait;".to_string(),
                "use sea_orm::Set;".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Model name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Model name in snake_case".to_string(),
                    example: "user".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "update_handler".to_string(),
            category: "handler".to_string(),
            description: "Update resource with validation using UpdateBuilder".to_string(),
            code: r#"#[handler]
pub async fn update(req: Request, id: Path<i32>) -> Response {
    let db = req.db();
    let data = req.input::<Update{{Entity}}Request>().await?;

    Validator::new(&data)
        .rules("name", rules![required()])
        // Add more validation rules
        .validate()?;

    let existing = {{Entity}}::find_by_id(*id)
        .one(db)
        .await?
        .ok_or_else(|| not_found("{{Entity}} not found"))?;

    let result = existing
        .update()
        .set_name(data.name)
        // Chain other .set_*() calls
        .save()
        .await?;

    Ok(json!(result))
}"#
            .to_string(),
            imports: vec![
                "use ferro::{handler, Request, Response, HttpResponse, AppError, Validator, required, min, max};".to_string(),
                "use serde_json::json;".to_string(),
                "use serde::Deserialize;".to_string(),
                "use crate::entities::{{entity}}::Entity as {{Entity}};".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Model name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Model name in snake_case".to_string(),
                    example: "user".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "destroy_handler".to_string(),
            category: "handler".to_string(),
            description: "Delete resource by ID".to_string(),
            code: r#"#[handler]
pub async fn destroy(req: Request, id: Path<i32>) -> Response {
    let db = req.db();
    let existing = {{Entity}}::find_by_id(*id)
        .one(db)
        .await?
        .ok_or_else(|| not_found("{{Entity}} not found"))?;

    existing.delete(db).await?;

    Ok(json!({"deleted": true}).status(200))
}"#
            .to_string(),
            imports: vec![
                "use ferro::{handler, Request, Response, HttpResponse, ResponseExt, AppError};".to_string(),
                "use serde_json::json;".to_string(),
                "use crate::entities::{{entity}}::Entity as {{Entity}};".to_string(),
                "use sea_orm::ModelTrait;".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Model name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Model name in snake_case".to_string(),
                    example: "user".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "inertia_handler".to_string(),
            category: "handler".to_string(),
            description: "Render Inertia component with props".to_string(),
            code: r#"#[handler]
pub async fn show(req: Request, id: Path<i32>) -> Response {
    let db = req.db();
    let item = {{Entity}}::find_by_id(*id)
        .one(db)
        .await?
        .ok_or_else(|| not_found("{{Entity}} not found"))?;

    Inertia::render(&req, "{{Component}}", {{Props}}Props {
        {{entity}}: item,
    })
}

// For forms that consume request body:
#[handler]
pub async fn store(req: Request) -> Response {
    // IMPORTANT: Save context before consuming request
    let ctx = SavedInertiaContext::from(&req);
    let form = req.input::<{{Entity}}Form>().await?;

    // ... process form ...

    // Use saved context for render
    Inertia::render_ctx(&ctx, "{{Component}}", {{Props}}Props { /* ... */ })
}"#
            .to_string(),
            imports: vec![
                "use ferro::{handler, Request, Response, HttpResponse, AppError, Inertia, SavedInertiaContext};".to_string(),
                "use serde::Serialize;".to_string(),
                "use crate::entities::{{entity}}::Entity as {{Entity}};".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Model name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Model name in snake_case".to_string(),
                    example: "user".to_string(),
                },
                Placeholder {
                    name: "{{Component}}".to_string(),
                    description: "Inertia component path".to_string(),
                    example: "Users/Show".to_string(),
                },
                Placeholder {
                    name: "{{Props}}".to_string(),
                    description: "Props struct name prefix".to_string(),
                    example: "UserShow".to_string(),
                },
            ],
        },
    ]
}

fn model_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "entity_model".to_string(),
            category: "model".to_string(),
            description: "SeaORM DeriveEntityModel struct".to_string(),
            code: r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{{table_name}}")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // #[sea_orm(has_many = "super::post::Entity")]
    // Posts,
}

impl ActiveModelBehavior for ActiveModel {}"#
                .to_string(),
            imports: vec![], // Already in code
            placeholders: vec![Placeholder {
                name: "{{table_name}}".to_string(),
                description: "Database table name (snake_case plural)".to_string(),
                example: "users".to_string(),
            }],
        },
        CodeTemplate {
            name: "active_model".to_string(),
            category: "model".to_string(),
            description: "Model create and update operations".to_string(),
            code: r#"use crate::entities::{{entity}};
use sea_orm::{ActiveModelTrait, Set};

// Create (using ActiveModel)
let model = {{entity}}::ActiveModel {
    name: Set("Example".to_string()),
    ..Default::default()
};
let result = model.insert(db).await?;

// Update (using UpdateBuilder)
let result = existing
    .update()
    .set_name("New Name")
    .save()
    .await?;

// Clear an optional field
let result = existing
    .update()
    .clear_description()
    .save()
    .await?;"#
                .to_string(),
            imports: vec![
                "use crate::entities::{{entity}};".to_string(),
                "use sea_orm::{ActiveModelTrait, Set};".to_string(),
            ],
            placeholders: vec![Placeholder {
                name: "{{entity}}".to_string(),
                description: "Entity module name (snake_case)".to_string(),
                example: "user".to_string(),
            }],
        },
        CodeTemplate {
            name: "query_example".to_string(),
            category: "model".to_string(),
            description: "Common SeaORM query patterns".to_string(),
            code: r#"use crate::entities::{{entity}};
use crate::entities::{{entity}}::Entity as {{Entity}};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

// Find by ID
let item = {{Entity}}::find_by_id(id).one(db).await?;

// Find all
let items = {{Entity}}::find().all(db).await?;

// Find with filter
let items = {{Entity}}::find()
    .filter({{entity}}::Column::Status.eq("active"))
    .all(db)
    .await?;

// Find with ordering
let items = {{Entity}}::find()
    .order_by_desc({{entity}}::Column::CreatedAt)
    .all(db)
    .await?;

// Find with pagination
let paginator = {{Entity}}::find().paginate(db, 20);
let items = paginator.fetch_page(0).await?;
let total = paginator.num_items().await?;

// Find with relation
let items = {{Entity}}::find()
    .find_with_related(Related{{Entity}})
    .all(db)
    .await?;

// Count
let count = {{Entity}}::find()
    .filter({{entity}}::Column::Status.eq("active"))
    .count(db)
    .await?;"#
                .to_string(),
            imports: vec![
                "use crate::entities::{{entity}};".to_string(),
                "use crate::entities::{{entity}}::Entity as {{Entity}};".to_string(),
                "use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Entity struct name (PascalCase)".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Entity module name (snake_case)".to_string(),
                    example: "user".to_string(),
                },
            ],
        },
    ]
}

fn migration_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "create_table".to_string(),
            category: "migration".to_string(),
            description: "Create a new database table".to_string(),
            code: r#"use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table({{Entity}}::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new({{Entity}}::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new({{Entity}}::Name).string().not_null())
                    .col(
                        ColumnDef::new({{Entity}}::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new({{Entity}}::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table({{Entity}}::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum {{Entity}} {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}"#
            .to_string(),
            imports: vec!["use sea_orm_migration::prelude::*;".to_string()],
            placeholders: vec![Placeholder {
                name: "{{Entity}}".to_string(),
                description: "Entity enum name (PascalCase)".to_string(),
                example: "User".to_string(),
            }],
        },
        CodeTemplate {
            name: "add_column".to_string(),
            category: "migration".to_string(),
            description: "Add column to existing table".to_string(),
            code: r#"use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table({{Entity}}::Table)
                    .add_column(
                        ColumnDef::new({{Entity}}::{{NewColumn}})
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table({{Entity}}::Table)
                    .drop_column({{Entity}}::{{NewColumn}})
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum {{Entity}} {
    Table,
    {{NewColumn}},
}"#
            .to_string(),
            imports: vec!["use sea_orm_migration::prelude::*;".to_string()],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Entity enum name (PascalCase)".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{NewColumn}}".to_string(),
                    description: "New column name (PascalCase)".to_string(),
                    example: "Bio".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "create_index".to_string(),
            category: "migration".to_string(),
            description: "Add index to table".to_string(),
            code: r#"use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_{{table}}_{{column}}")
                    .table({{Entity}}::Table)
                    .col({{Entity}}::{{Column}})
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_{{table}}_{{column}}")
                    .table({{Entity}}::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum {{Entity}} {
    Table,
    {{Column}},
}"#
            .to_string(),
            imports: vec!["use sea_orm_migration::prelude::*;".to_string()],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Entity enum name (PascalCase)".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{Column}}".to_string(),
                    description: "Column enum variant (PascalCase)".to_string(),
                    example: "Email".to_string(),
                },
                Placeholder {
                    name: "{{table}}".to_string(),
                    description: "Table name for index naming (snake_case)".to_string(),
                    example: "users".to_string(),
                },
                Placeholder {
                    name: "{{column}}".to_string(),
                    description: "Column name for index naming (snake_case)".to_string(),
                    example: "email".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "add_foreign_key".to_string(),
            category: "migration".to_string(),
            description: "Add foreign key relationship".to_string(),
            code: r#"use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table({{ChildEntity}}::Table)
                    .add_column(
                        ColumnDef::new({{ChildEntity}}::{{ParentEntity}}Id)
                            .integer()
                            .not_null(),
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_{{child_table}}_{{parent_table}}")
                            .from_tbl({{ChildEntity}}::Table)
                            .from_col({{ChildEntity}}::{{ParentEntity}}Id)
                            .to_tbl({{ParentEntity}}::Table)
                            .to_col({{ParentEntity}}::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table({{ChildEntity}}::Table)
                    .drop_foreign_key(Alias::new("fk_{{child_table}}_{{parent_table}}"))
                    .drop_column({{ChildEntity}}::{{ParentEntity}}Id)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum {{ParentEntity}} {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum {{ChildEntity}} {
    Table,
    {{ParentEntity}}Id,
}"#
            .to_string(),
            imports: vec!["use sea_orm_migration::prelude::*;".to_string()],
            placeholders: vec![
                Placeholder {
                    name: "{{ParentEntity}}".to_string(),
                    description: "Parent entity name (PascalCase)".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{ChildEntity}}".to_string(),
                    description: "Child entity name (PascalCase)".to_string(),
                    example: "Post".to_string(),
                },
                Placeholder {
                    name: "{{parent_table}}".to_string(),
                    description: "Parent table name for FK naming (snake_case)".to_string(),
                    example: "users".to_string(),
                },
                Placeholder {
                    name: "{{child_table}}".to_string(),
                    description: "Child table name for FK naming (snake_case)".to_string(),
                    example: "posts".to_string(),
                },
            ],
        },
    ]
}

fn middleware_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "auth_middleware".to_string(),
            category: "middleware".to_string(),
            description: "Authentication check middleware".to_string(),
            code: r#"use ferro::{Middleware, Next, Request, HttpResponse};

pub struct AuthMiddleware;

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn handle(&self, req: Request, next: Next) -> HttpResponse {
        // Check for authenticated user
        let user = req.auth().user::<User>().await;

        match user {
            Some(_) => next.run(req).await,
            None => HttpResponse::json(serde_json::json!({"message": "Unauthenticated."})).status(401),
        }
    }
}"#
            .to_string(),
            imports: vec![
                "use ferro::{Middleware, Next, Request, HttpResponse};".to_string(),
                "use crate::models::User;".to_string(),
            ],
            placeholders: vec![],
        },
        CodeTemplate {
            name: "basic_middleware".to_string(),
            category: "middleware".to_string(),
            description: "Basic middleware structure".to_string(),
            code: r#"use ferro::{Middleware, Next, Request, HttpResponse};

pub struct {{Name}}Middleware;

#[async_trait::async_trait]
impl Middleware for {{Name}}Middleware {
    async fn handle(&self, req: Request, next: Next) -> HttpResponse {
        // Before request processing
        // ...

        // Call next middleware/handler
        let response = next.run(req).await;

        // After request processing
        // ...

        response
    }
}"#
            .to_string(),
            imports: vec![
                "use ferro::{Middleware, Next, Request, HttpResponse};".to_string(),
            ],
            placeholders: vec![Placeholder {
                name: "{{Name}}".to_string(),
                description: "Middleware name (PascalCase)".to_string(),
                example: "RateLimit".to_string(),
            }],
        },
    ]
}

fn validation_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "form_validation".to_string(),
            category: "validation".to_string(),
            description: "Full form validation with multiple fields".to_string(),
            code: r#"use ferro::{Validator, required, email, min, max, integer, nullable};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct {{Form}}Request {
    pub name: String,
    pub email: String,
    pub password: String,
    pub age: Option<i32>,
}

// In handler:
let data = req.input::<{{Form}}Request>().await?;

Validator::new(&data)
    .rules("name", rules![required(), min(2.0), max(100.0)])
    .rules("email", rules![required(), email()])
    .rules("password", rules![required(), min(8.0)])
    .rules("age", rules![nullable(), integer(), min(0.0), max(150.0)])
    .validate()?;

// Validation passes, data is valid"#
                .to_string(),
            imports: vec![
                "use ferro::{Validator, required, email, min, max, integer, nullable};".to_string(),
                "use serde::Deserialize;".to_string(),
            ],
            placeholders: vec![Placeholder {
                name: "{{Form}}".to_string(),
                description: "Form/request struct name prefix".to_string(),
                example: "CreateUser".to_string(),
            }],
        },
        CodeTemplate {
            name: "field_rules".to_string(),
            category: "validation".to_string(),
            description: "Common validation rule combinations".to_string(),
            code: r#"use ferro::{required, min, max, email, url, integer, numeric, nullable, confirmed, required_if};

// String fields
rules![required(), min(1.0), max(255.0)]  // Required string
rules![nullable(), min(1.0), max(255.0)]  // Optional string
rules![required(), email()]  // Email
rules![required(), url()]  // URL

// Numeric fields
rules![required(), integer()]  // Required integer
rules![required(), numeric(), min(0.0)]  // Non-negative number
rules![nullable(), integer(), min(1.0), max(100.0)]  // Optional int 1-100

// Confirmation
rules![required(), confirmed()]  // password + password_confirmation

// Conditional
rules![required_if("type", "premium")]  // Required if type == premium

// Array/List
rules![required(), min(1.0)]  // At least one item"#
                .to_string(),
            imports: vec!["use ferro::{required, min, max, email, url, integer, numeric, nullable, confirmed, required_if};".to_string()],
            placeholders: vec![],
        },
    ]
}

fn json_view_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "basic_view".to_string(),
            category: "json_view".to_string(),
            description: "A minimal JSON-UI v2 spec file with title, heading text, and one card component. Save as src/views/{{view_name}}.json.".to_string(),
            code: r#"{
  "$schema": "ferro-json-ui/v2",
  "title": "{{title}}",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "{{title}}" },
      "children": ["heading"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "{{title}}", "element": "h1" }
    }
  }
}"#
            .to_string(),
            imports: vec![],
            placeholders: vec![
                Placeholder {
                    name: "{{view_name}}".to_string(),
                    description: "View file name (snake_case, without .json extension)".to_string(),
                    example: "dashboard".to_string(),
                },
                Placeholder {
                    name: "{{title}}".to_string(),
                    description: "Page title displayed in the view".to_string(),
                    example: "Dashboard".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "list_view".to_string(),
            category: "json_view".to_string(),
            description: "A JSON-UI v2 spec for listing resources with a data table, pagination, and create action button. Save as src/views/{{view_name}}.json.".to_string(),
            code: r#"{
  "$schema": "ferro-json-ui/v2",
  "title": "{{title}}",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "{{title}}" },
      "children": ["heading", "create-btn", "{{entity}}-table", "pagination"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "{{title}}", "element": "h1" }
    },
    "create-btn": {
      "type": "Button",
      "props": { "label": "Create {{Entity}}", "variant": "default" },
      "action": { "handler": "{{entity}}.create", "method": "GET" }
    },
    "{{entity}}-table": {
      "type": "DataTable",
      "props": {
        "columns": [
          { "key": "id", "label": "ID" },
          { "key": "name", "label": "Name" }
        ],
        "data_path": "/data/{{entity}}s",
        "empty_message": "No {{entity}}s found"
      }
    },
    "pagination": {
      "type": "Pagination",
      "props": { "current_page": 1, "per_page": 20, "total": 0 }
    }
  }
}"#
            .to_string(),
            imports: vec![],
            placeholders: vec![
                Placeholder {
                    name: "{{view_name}}".to_string(),
                    description: "View file name (snake_case, without .json extension)".to_string(),
                    example: "users_index".to_string(),
                },
                Placeholder {
                    name: "{{title}}".to_string(),
                    description: "Page title displayed in the view".to_string(),
                    example: "Users".to_string(),
                },
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Entity name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Entity name in snake_case".to_string(),
                    example: "user".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "form_view".to_string(),
            category: "json_view".to_string(),
            description: "A JSON-UI v2 spec for a form with input fields and a submit button. Save as src/views/{{view_name}}.json.".to_string(),
            code: r#"{
  "$schema": "ferro-json-ui/v2",
  "title": "{{title}}",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "{{title}}" },
      "children": ["heading", "{{entity}}-form"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "{{title}}", "element": "h1" }
    },
    "{{entity}}-form": {
      "type": "Form",
      "props": {},
      "children": ["name-field", "email-field"],
      "action": { "handler": "{{action_handler}}", "method": "POST" }
    },
    "name-field": {
      "type": "Input",
      "props": {
        "field": "name",
        "label": "Name",
        "input_type": "text",
        "placeholder": "Enter name",
        "required": true
      }
    },
    "email-field": {
      "type": "Input",
      "props": {
        "field": "email",
        "label": "Email",
        "input_type": "email",
        "placeholder": "Enter email",
        "required": true
      }
    }
  }
}"#
            .to_string(),
            imports: vec![],
            placeholders: vec![
                Placeholder {
                    name: "{{view_name}}".to_string(),
                    description: "View file name (snake_case, without .json extension)".to_string(),
                    example: "users_create".to_string(),
                },
                Placeholder {
                    name: "{{title}}".to_string(),
                    description: "Page title displayed in the view".to_string(),
                    example: "Create User".to_string(),
                },
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Entity name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Entity name in snake_case".to_string(),
                    example: "user".to_string(),
                },
                Placeholder {
                    name: "{{action_handler}}".to_string(),
                    description: "Route handler name for form submission (controller.method)".to_string(),
                    example: "users.store".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "json_view_handler".to_string(),
            category: "json_view".to_string(),
            description: "Rust handler that loads and renders a JSON-UI v2 spec file via JsonUi::render_file.".to_string(),
            code: r#"#[handler]
pub async fn {{view_name}}(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/{{view_name}}.json", data)
}"#
            .to_string(),
            imports: vec![
                "use ferro::{JsonUi, Response};".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{view_name}}".to_string(),
                    description: "Handler and view file name (snake_case)".to_string(),
                    example: "dashboard".to_string(),
                },
            ],
        },
    ]
}

fn rate_limiting_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "define_rate_limiters".to_string(),
            category: "rate_limiting".to_string(),
            description: "Define named rate limiters in bootstrap.rs".to_string(),
            code: r#"use ferro::middleware::{RateLimiter, Limit};
use ferro::Auth;

pub fn register_rate_limiters() {
    // API rate limiter: authenticated users get higher limit
    RateLimiter::define("api", |req| {
        match Auth::id() {
            Some(id) => Limit::per_minute(120).by(format!("user:{}", id)),
            None => Limit::per_minute(60),
        }
    });

    // Auth rate limiter: strict per-IP limit on login attempts
    RateLimiter::define("auth", |req| {
        let ip = req.header("X-Forwarded-For")
            .and_then(|s| s.split(',').next())
            .unwrap_or("unknown")
            .trim()
            .to_string();
        Limit::per_minute(5).by(ip)
    });
}"#
            .to_string(),
            imports: vec![
                "use ferro::middleware::{RateLimiter, Limit};".to_string(),
                "use ferro::Auth;".to_string(),
            ],
            placeholders: vec![],
        },
        CodeTemplate {
            name: "throttle_routes".to_string(),
            category: "rate_limiting".to_string(),
            description: "Apply named throttle middleware to route groups".to_string(),
            code: r#"use ferro::middleware::Throttle;

routes! {
    group!("/api", {
        get!("/{{entity}}s", controllers::{{entity}}s::index),
        get!("/{{entity}}s/{id}", controllers::{{entity}}s::show),
    }).middleware(Throttle::named("api")),

    group!("/auth", {
        post!("/login", controllers::auth::login),
        post!("/register", controllers::auth::register),
    }).middleware(Throttle::named("auth")),
}"#
            .to_string(),
            imports: vec!["use ferro::middleware::Throttle;".to_string()],
            placeholders: vec![Placeholder {
                name: "{{entity}}".to_string(),
                description: "Entity name in snake_case".to_string(),
                example: "user".to_string(),
            }],
        },
        CodeTemplate {
            name: "inline_throttle".to_string(),
            category: "rate_limiting".to_string(),
            description: "Apply inline rate limit without named registration".to_string(),
            code: r#"use ferro::middleware::Throttle;

// Inline throttle: no need to register with RateLimiter::define()
get!("/health", controllers::health::check)
    .middleware(Throttle::per_minute(10))

// Other inline options:
// Throttle::per_second(5)
// Throttle::per_hour(1000)
// Throttle::per_day(10000)"#
                .to_string(),
            imports: vec!["use ferro::middleware::Throttle;".to_string()],
            placeholders: vec![],
        },
    ]
}

fn broadcasting_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "broadcasting_setup".to_string(),
            category: "broadcasting".to_string(),
            description: "Register Broadcaster with ChannelAuthorizer in bootstrap.rs".to_string(),
            code: r#"use ferro::{Broadcaster, BroadcastConfig, AuthData, ChannelAuthorizer};
use ferro::container::App;

pub struct AppChannelAuth;

#[async_trait::async_trait]
impl ChannelAuthorizer for AppChannelAuth {
    async fn authorize(&self, data: &AuthData) -> bool {
        match data.channel.as_str() {
            c if c.starts_with("private-{{entity}}s.") => {
                // Verify user has access to this resource
                data.auth_token.is_some()
            }
            c if c.starts_with("presence-") => {
                // Allow all authenticated users
                data.auth_token.is_some()
            }
            _ => false,
        }
    }
}

pub async fn register() {
    let broadcaster = Broadcaster::with_config(BroadcastConfig::from_env())
        .with_authorizer(AppChannelAuth);
    App::singleton(broadcaster);
}"#
            .to_string(),
            imports: vec![
                "use ferro::{Broadcaster, BroadcastConfig, AuthData, ChannelAuthorizer};"
                    .to_string(),
                "use ferro::container::App;".to_string(),
            ],
            placeholders: vec![Placeholder {
                name: "{{entity}}".to_string(),
                description: "Entity name in snake_case for channel prefix".to_string(),
                example: "order".to_string(),
            }],
        },
        CodeTemplate {
            name: "broadcasting_routes".to_string(),
            category: "broadcasting".to_string(),
            description: "Register broadcasting auth endpoint with session middleware".to_string(),
            code: r#"use ferro::broadcasting_auth;

// In routes.rs - add to your route definitions
Route::post("/broadcasting/auth", broadcasting_auth)
    .middleware(SessionAuthMiddleware);

// WebSocket endpoint is automatic at /_ferro/ws
// No route registration needed for WebSocket connections"#
                .to_string(),
            imports: vec!["use ferro::broadcasting_auth;".to_string()],
            placeholders: vec![],
        },
        CodeTemplate {
            name: "broadcasting_send".to_string(),
            category: "broadcasting".to_string(),
            description: "Send broadcast events from handlers using Broadcast builder".to_string(),
            code: r#"use ferro::{Broadcast, Broadcaster};
use ferro::container::App;
use std::sync::Arc;

#[handler]
pub async fn update(req: Request, id: Path<i32>) -> Response {
    let db = req.db();
    let {{entity}} = update_{{entity}}_in_db(db, *id).await?;

    // Broadcast the update to channel subscribers
    let broadcaster = App::get::<Broadcaster>().unwrap();
    let broadcast = Broadcast::new(Arc::new(broadcaster));

    broadcast
        .channel(&format!("{{entity}}s.{}", id))
        .event("{{Entity}}Updated")
        .data(&{{entity}})
        .send()
        .await
        .ok();

    Ok(json!({{entity}}))
}

// To exclude the triggering client from the broadcast:
// broadcast
//     .channel("{{entity}}s.1")
//     .event("{{Entity}}Updated")
//     .data(&{{entity}})
//     .except(&socket_id)
//     .send()
//     .await?;"#
                .to_string(),
            imports: vec![
                "use ferro::{Broadcast, Broadcaster};".to_string(),
                "use ferro::container::App;".to_string(),
                "use std::sync::Arc;".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Entity name in PascalCase".to_string(),
                    example: "Order".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Entity name in snake_case".to_string(),
                    example: "order".to_string(),
                },
            ],
        },
    ]
}

fn api_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "api_controller".to_string(),
            category: "api".to_string(),
            description: "CRUD API controller with pagination, resource responses, and error handling. Sensitive fields (password_hash, etc.) are auto-excluded by make:api."
                .to_string(),
            code: r#"use ferro::{handler, Request, Response, HttpResponse};
use crate::models::{{entity}}::{self, Entity as {{Entity}}};
use sea_orm::{EntityTrait, PaginatorTrait};
// Sensitive fields (password_hash, etc.) are auto-excluded by make:api
use crate::resources::{{entity}}_resource::{{Entity}}Resource;
use crate::requests::{{entity}}_request::{Create{{Entity}}Request, Update{{Entity}}Request};

#[handler]
pub async fn index(req: Request) -> Response {
    let page: u64 = req.query("page").unwrap_or(1);
    let per_page: u64 = req.query("per_page").unwrap_or(15).min(100);
    let db = ferro::DB::connection()
        .map_err(|e| HttpResponse::json(serde_json::json!({"error": e.to_string()})).status(500))?;
    let paginator = {{Entity}}::find().paginate(&db, per_page);
    let total = paginator.num_items().await
        .map_err(|e| HttpResponse::json(serde_json::json!({"error": e.to_string()})).status(500))?;
    let items = paginator.fetch_page(page - 1).await
        .map_err(|e| HttpResponse::json(serde_json::json!({"error": e.to_string()})).status(500))?;
    let resources: Vec<{{Entity}}Resource> = items.into_iter().map({{Entity}}Resource::from).collect();
    let meta = ferro::PaginationMeta::new(page, per_page, total);
    Ok(ferro::ResourceCollection::paginated(resources, meta).to_response(&req))
}

#[handler]
pub async fn show(req: Request, {{entity}}: {{entity}}::Model) -> Response {
    Ok(ferro::Resource::to_wrapped_response(&{{Entity}}Resource::from({{entity}}), &req))
}

#[handler]
pub async fn store(req: Request, form: Create{{Entity}}Request) -> Response {
    let model = {{Entity}}::create()
        // .set_field(form.field.clone())
        .insert()
        .await
        .map_err(|e| HttpResponse::json(serde_json::json!({"error": e.to_string()})).status(500))?;
    Ok(ferro::Resource::to_wrapped_response(&{{Entity}}Resource::from(&model), &req).status(201))
}

#[handler]
pub async fn update(req: Request, {{entity}}: {{entity}}::Model, form: Update{{Entity}}Request) -> Response {
    let mut builder = {{entity}}.update();
    // if let Some(ref v) = form.field { builder = builder.set_field(v.clone()); }
    let updated = builder.save().await
        .map_err(|e| HttpResponse::json(serde_json::json!({"error": e.to_string()})).status(500))?;
    Ok(ferro::Resource::to_wrapped_response(&{{Entity}}Resource::from(&updated), &req))
}

#[handler]
pub async fn destroy({{entity}}: {{entity}}::Model) -> Response {
    {{entity}}.delete().await
        .map_err(|e| HttpResponse::json(serde_json::json!({"error": e.to_string()})).status(500))?;
    Ok(HttpResponse::json(serde_json::json!({"message": "Deleted"})).status(200))
}"#
            .to_string(),
            imports: vec![
                "use ferro::{handler, Request, Response, HttpResponse};".to_string(),
                "use sea_orm::{EntityTrait, PaginatorTrait};".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Entity name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Entity name in snake_case".to_string(),
                    example: "user".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "api_key_middleware".to_string(),
            category: "api".to_string(),
            description: "API key middleware configuration with optional scope requirements"
                .to_string(),
            code: r#"use ferro::ApiKeyMiddleware;

// Require any valid API key
group!("/api/v1")
    .middleware(ApiKeyMiddleware::new())
    .routes([...]);

// Require specific scopes
group!("/api/v1/admin")
    .middleware(ApiKeyMiddleware::scopes(&["admin"]))
    .routes([...]);

// Access key info in handlers
use ferro::ApiKeyInfo;

#[handler]
pub async fn index(req: Request) -> Response {
    let key_info = req.get::<ApiKeyInfo>().unwrap();
    println!("Key: {} (scopes: {:?})", key_info.name, key_info.scopes);
    // ...
}"#
            .to_string(),
            imports: vec!["use ferro::ApiKeyMiddleware;".to_string()],
            placeholders: vec![],
        },
        CodeTemplate {
            name: "api_route_group".to_string(),
            category: "api".to_string(),
            description:
                "API route group with ApiKeyMiddleware and Throttle for CRUD resources"
                    .to_string(),
            code: r#"use ferro::*;
use crate::api::*;

pub fn api_routes() -> GroupBuilder {
    group!("/api/v1")
        .middleware(ApiKeyMiddleware::new())
        .middleware(Throttle::named("api"))
        .routes([
            // {{Entity}} CRUD
            get!("/{{entities}}", {{entity}}_api::index).name("api.{{entities}}.index"),
            post!("/{{entities}}", {{entity}}_api::store).name("api.{{entities}}.store"),
            get!("/{{entities}}/:id", {{entity}}_api::show).name("api.{{entities}}.show"),
            put!("/{{entities}}/:id", {{entity}}_api::update).name("api.{{entities}}.update"),
            delete!("/{{entities}}/:id", {{entity}}_api::destroy).name("api.{{entities}}.destroy"),
        ])
}"#
            .to_string(),
            imports: vec!["use ferro::*;".to_string()],
            placeholders: vec![
                Placeholder {
                    name: "{{Entity}}".to_string(),
                    description: "Entity name in PascalCase".to_string(),
                    example: "User".to_string(),
                },
                Placeholder {
                    name: "{{entity}}".to_string(),
                    description: "Entity name in snake_case".to_string(),
                    example: "user".to_string(),
                },
                Placeholder {
                    name: "{{entities}}".to_string(),
                    description: "Plural entity name for URL paths".to_string(),
                    example: "users".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "api_openapi_docs".to_string(),
            category: "api".to_string(),
            description: "OpenAPI documentation handlers with ReDoc UI and JSON spec endpoints"
                .to_string(),
            code: r#"use ferro::*;

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
    Ok(openapi_docs_response(&config, &routes))
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
    Ok(openapi_json_response(&config, &routes))
}"#
            .to_string(),
            imports: vec!["use ferro::*;".to_string()],
            placeholders: vec![],
        },
    ]
}

fn migration_v1_to_v2_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "render_file_migration".to_string(),
            category: "migration_v1_to_v2".to_string(),
            description: "Replace v1 JsonUiView builder with v2 JsonUi::render_file. \
                          v1 types (JsonUiView, Component, ComponentNode) are removed."
                .to_string(),
            code: r#"// v2: load spec from JSON file and merge handler data
JsonUi::render_file("src/views/{{module}}/{{page}}.json", serde_json::json!({
    // your handler data
}))"#
            .to_string(),
            imports: vec!["use ferro::{JsonUi, serde_json};".to_string()],
            placeholders: vec![
                Placeholder {
                    name: "{{module}}".to_string(),
                    description: "Controller module name".to_string(),
                    example: "account".to_string(),
                },
                Placeholder {
                    name: "{{page}}".to_string(),
                    description: "Page name".to_string(),
                    example: "settings".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "card_children_flat_map".to_string(),
            category: "migration_v1_to_v2".to_string(),
            description:
                "v2 Card with children as IDs into the flat elements map (was nested Vec<Component> in v1)."
                    .to_string(),
            code: r#"{
  "root": "card_main",
  "elements": {
    "card_main": { "type": "Card", "props": {"title": "{{title}}"}, "children": ["heading", "body"] },
    "heading":   { "type": "Text", "props": {"content": "{{heading}}", "element": "h2"} },
    "body":      { "type": "Text", "props": {"content": "{{body}}"} }
  }
}"#
            .to_string(),
            imports: vec![],
            placeholders: vec![
                Placeholder {
                    name: "{{title}}".to_string(),
                    description: "Card title".to_string(),
                    example: "Welcome".to_string(),
                },
                Placeholder {
                    name: "{{heading}}".to_string(),
                    description: "Heading text".to_string(),
                    example: "Sign in".to_string(),
                },
                Placeholder {
                    name: "{{body}}".to_string(),
                    description: "Body text".to_string(),
                    example: "Enter your credentials.".to_string(),
                },
            ],
        },
        CodeTemplate {
            name: "datatable_row_actions_interpolation".to_string(),
            category: "migration_v1_to_v2".to_string(),
            description:
                "DataTable per-row action URL with column-key interpolation (D-03/D-04). \
                 Any row column key such as {slug_path}, {label}, {status} is substituted at render time."
                    .to_string(),
            code: r#"{
  "type": "DataTable",
  "props": {
    "data_path": "/pages",
    "row_key": "slug_path",
    "row_actions": [
      { "label": "Edit",   "action": { "url": "/p/{slug_path}/edit" } },
      { "label": "Delete", "action": { "url": "/p/{slug_path}/delete",
                                        "confirm": { "message": "Delete this page?" } } }
    ]
  }
}"#
            .to_string(),
            imports: vec![],
            placeholders: vec![],
        },
        CodeTemplate {
            name: "inline_view_edit_pattern".to_string(),
            category: "migration_v1_to_v2".to_string(),
            description:
                "Read+edit detail pattern using Form + visible conditions on a ?mode= query param \
                 (replaces v1 DetailFormProps / DetailField / EditMode)."
                    .to_string(),
            code: r#"{
  "type": "Form",
  "props": { "method": "POST", "max_width": "md" },
  "children": ["name_view", "name_edit", "save_btn"],
  "action": { "handler": "{{handler}}", "method": "POST" },
  "elements": {
    "name_view": {
      "type": "DescriptionList",
      "props": { "items": [{ "label": "Name", "value": { "$data": "/user/name" } }] },
      "visible": { "ne": ["query.mode", "edit"] }
    },
    "name_edit": {
      "type": "Input",
      "props": { "field": "name", "label": "Name", "data_path": "/user/name" },
      "visible": { "eq": ["query.mode", "edit"] }
    },
    "save_btn": {
      "type": "Button",
      "props": { "label": "Save", "button_type": "submit" },
      "visible": { "eq": ["query.mode", "edit"] }
    }
  }
}"#
            .to_string(),
            imports: vec![],
            placeholders: vec![Placeholder {
                name: "{{handler}}".to_string(),
                description: "Route handler name for form submission".to_string(),
                example: "profile.update".to_string(),
            }],
        },
        CodeTemplate {
            name: "checkbox_list_data_driven".to_string(),
            category: "migration_v1_to_v2".to_string(),
            description:
                "Data-driven CheckboxList (D-01). Options resolved from options_path; \
                 pre-selected values from selected_path."
                    .to_string(),
            code: r#"{
  "type": "CheckboxList",
  "props": {
    "field": "services",
    "options_path": "/available_services",
    "selected_path": "/user/selected_services",
    "label": "Choose services"
  }
}"#
            .to_string(),
            imports: vec![],
            placeholders: vec![],
        },
        CodeTemplate {
            name: "variant_strum_round_trip".to_string(),
            category: "migration_v1_to_v2".to_string(),
            description:
                "Use typed variant enums via strum AsRefStr (D-11) instead of hand-typed strings \
                 in Rust spec-builder code. Wire format unchanged."
                    .to_string(),
            code: r#"use ferro_json_ui::{AlertVariant, AlertProps};

let props = AlertProps {
    variant: Some(AlertVariant::Success),
    message: "Saved.".to_string(),
    title: None,
};
// Serializes to: { "variant": "success", "message": "Saved." }

// AsRefStr call site:
assert_eq!(AlertVariant::Success.as_ref(), "success");
assert_eq!(AlertVariant::Warning.as_ref(), "warning");"#
            .to_string(),
            imports: vec!["use ferro_json_ui::{AlertVariant, AlertProps};".to_string()],
            placeholders: vec![],
        },
        CodeTemplate {
            name: "verify_action_mcp".to_string(),
            category: "migration_v1_to_v2".to_string(),
            description:
                "Verify a handler name via the json_ui_verify_action MCP tool (D-09). \
                 Returns route info on hit; Levenshtein candidate on miss."
                    .to_string(),
            code: r#"// MCP tool call (agent context):
// mcp__ferro__json_ui_verify_action({ "handler": "{{handler}}", "method": "{{method}}" })
//
// Hit:  { "found": true,  "route": { "name": "...", "method": "GET", "path": "..." }, "candidate": null }
// Miss: { "found": false, "route": null, "candidate": "dashboard.show" }"#
            .to_string(),
            imports: vec![],
            placeholders: vec![
                Placeholder {
                    name: "{{handler}}".to_string(),
                    description: "Registered route name".to_string(),
                    example: "dashboard.show".to_string(),
                },
                Placeholder {
                    name: "{{method}}".to_string(),
                    description: "HTTP method".to_string(),
                    example: "GET".to_string(),
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_categories_present() {
        let templates = execute(None);

        let categories: std::collections::HashSet<_> = templates
            .templates
            .iter()
            .map(|t| t.category.as_str())
            .collect();

        assert!(
            categories.contains("handler"),
            "Should have handler templates"
        );
        assert!(categories.contains("model"), "Should have model templates");
        assert!(
            categories.contains("migration"),
            "Should have migration templates"
        );
        assert!(
            categories.contains("middleware"),
            "Should have middleware templates"
        );
        assert!(
            categories.contains("validation"),
            "Should have validation templates"
        );
        assert!(
            categories.contains("json_view"),
            "Should have json_view templates"
        );
        assert!(
            categories.contains("rate_limiting"),
            "Should have rate_limiting templates"
        );
        assert!(
            categories.contains("broadcasting"),
            "Should have broadcasting templates"
        );
        assert!(categories.contains("api"), "Should have api templates");
    }

    #[test]
    fn test_filter_by_category() {
        let handler_templates = execute(Some("handler"));
        assert!(!handler_templates.templates.is_empty());
        assert!(
            handler_templates
                .templates
                .iter()
                .all(|t| t.category == "handler"),
            "All templates should be handlers"
        );

        let model_templates = execute(Some("model"));
        assert!(!model_templates.templates.is_empty());
        assert!(
            model_templates
                .templates
                .iter()
                .all(|t| t.category == "model"),
            "All templates should be models"
        );
    }

    #[test]
    fn test_templates_have_required_fields() {
        let templates = execute(None);

        for template in &templates.templates {
            assert!(!template.name.is_empty(), "Template should have name");
            assert!(
                !template.category.is_empty(),
                "Template should have category"
            );
            assert!(
                !template.description.is_empty(),
                "Template should have description"
            );
            assert!(!template.code.is_empty(), "Template should have code");
            // imports can be empty for self-contained templates
        }
    }

    #[test]
    fn test_handler_templates_count() {
        let handler_templates = execute(Some("handler"));
        assert!(
            handler_templates.templates.len() >= 5,
            "Should have at least 5 handler templates, got {}",
            handler_templates.templates.len()
        );
    }

    #[test]
    fn test_serialization() {
        let templates = execute(None);
        let json = serde_json::to_string(&templates);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("templates"));
        assert!(json_str.contains("handler"));
        assert!(json_str.contains("placeholders"));
    }

    #[test]
    fn test_unknown_category_returns_empty() {
        let templates = execute(Some("nonexistent"));
        assert!(
            templates.templates.is_empty(),
            "Unknown category should return empty"
        );
    }

    #[test]
    fn code_templates_returns_migration_patterns() {
        let templates = execute(Some("migration_v1_to_v2"));
        assert!(
            templates.templates.len() >= 7,
            "expected at least 7 migration_v1_to_v2 templates, got {}",
            templates.templates.len()
        );
        for t in &templates.templates {
            assert_eq!(t.category, "migration_v1_to_v2");
        }
        let names: Vec<&str> = templates.templates.iter().map(|t| t.name.as_str()).collect();
        for required in [
            "render_file_migration",
            "card_children_flat_map",
            "datatable_row_actions_interpolation",
            "inline_view_edit_pattern",
            "checkbox_list_data_driven",
            "variant_strum_round_trip",
            "verify_action_mcp",
        ] {
            assert!(names.contains(&required), "missing migration template: {required}");
        }
    }
}
