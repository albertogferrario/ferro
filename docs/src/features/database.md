# Database

Ferro provides a SeaORM-based database layer with Laravel-like API, automatic route model binding, fluent query builder, and testing utilities.

## Configuration

### Environment Variables

Configure database in your `.env` file:

```env
# Database connection URL (required)
DATABASE_URL=postgres://user:pass@localhost:5432/mydb

# For SQLite:
# DATABASE_URL=sqlite://./database.db

# Connection pool settings
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=1
DB_CONNECT_TIMEOUT=30

# Enable SQL query logging
DB_LOGGING=false
```

### Bootstrap Setup

In `src/bootstrap.rs`, configure the database:

```rust
use ferro::{Config, DB, DatabaseConfig};

pub async fn register() {
    // Register database configuration
    Config::register(DatabaseConfig::from_env());

    // Initialize the database connection
    DB::init().await.expect("Failed to connect to database");
}
```

### Manual Configuration

```rust
use ferro::{Config, DB, DatabaseConfig};

// Build configuration programmatically
let config = DatabaseConfig::builder()
    .url("postgres://localhost/mydb")
    .max_connections(20)
    .min_connections(5)
    .connect_timeout(60)
    .logging(true)
    .build();

// Initialize with custom config
DB::init_with(config).await?;
```

## Basic Usage

### Getting a Connection

```rust
use ferro::DB;

// Get the database connection
let conn = DB::connection()?;

// Use with SeaORM queries
let users = User::find().all(conn.inner()).await?;

// Shorthand
let conn = DB::get()?;
```

### Checking Connection Status

```rust
use ferro::DB;

if DB::is_connected() {
    let conn = DB::connection()?;
    // Perform database operations
}
```

## Models

Ferro provides Laravel-like traits for SeaORM entities.

### Defining a Model

```rust
use sea_orm::entity::prelude::*;
use ferro::{Model, ModelMut};

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Add Ferro's Model traits for convenient methods
impl ferro::Model for Entity {}
impl ferro::ModelMut for Entity {}
```

### Reading Records

```rust
use ferro::models::user;

// Find all records
let users = user::Entity::all().await?;

// Find by primary key
let user = user::Entity::find_by_pk(1).await?;

// Find or return error
let user = user::Entity::find_or_fail(1).await?;

// Get first record
let first = user::Entity::first().await?;

// Count all records
let count = user::Entity::count_all().await?;

// Check if any exist
if user::Entity::exists_any().await? {
    println!("Users exist!");
}
```

### Creating Records

```rust
use sea_orm::Set;
use ferro::models::user;

let new_user = user::ActiveModel {
    name: Set("John Doe".to_string()),
    email: Set("john@example.com".to_string()),
    ..Default::default()
};

let user = user::Entity::insert_one(new_user).await?;
println!("Created user with id: {}", user.id);
```

### Updating Records

```rust
use ferro::models::user::User;

// Find and update with builder pattern
let user = User::find_or_fail(1).await?;
let updated = user
    .update()
    .set_name("Updated Name")
    .save()
    .await?;
```

The `UpdateBuilder` tracks which fields were modified and only sends those to the database. It automatically updates the `updated_at` timestamp.

For nullable fields, use `clear_*()` to set a column to NULL:

```rust
let updated = user
    .update()
    .clear_bio()
    .save()
    .await?;
```

### Deleting Records

```rust
use ferro::models::user;

// Delete by primary key
let rows_deleted = user::Entity::delete_by_pk(1).await?;
println!("Deleted {} rows", rows_deleted);
```

### Save (Insert or Update)

```rust
use sea_orm::Set;
use ferro::models::user;

// Save will insert if no primary key, update if present
let model = user::ActiveModel {
    name: Set("Jane Doe".to_string()),
    email: Set("jane@example.com".to_string()),
    ..Default::default()
};

let saved = user::Entity::save_one(model).await?;
```

## Query Builder

The fluent query builder provides an Eloquent-like API.

### Basic Queries

```rust
use ferro::models::todo::{self, Column};

// Get all records
let todos = todo::Entity::query().all().await?;

// Get first record
let todo = todo::Entity::query().first().await?;

// Get first or error
let todo = todo::Entity::query().first_or_fail().await?;
```

### Filtering

```rust
use ferro::models::todo::{self, Column};

// Single filter
let todos = todo::Entity::query()
    .filter(Column::Active.eq(true))
    .all()
    .await?;

// Multiple filters (AND)
let todo = todo::Entity::query()
    .filter(Column::Title.eq("My Task"))
    .filter(Column::Id.gt(5))
    .first()
    .await?;

// Using SeaORM conditions
use sea_orm::Condition;

let todos = todo::Entity::query()
    .filter(
        Condition::any()
            .add(Column::Priority.eq("high"))
            .add(Column::DueDate.lt(chrono::Utc::now()))
    )
    .all()
    .await?;
```

### Ordering

```rust
use ferro::models::todo::{self, Column};

// Order ascending
let todos = todo::Entity::query()
    .order_by_asc(Column::Title)
    .all()
    .await?;

// Order descending
let todos = todo::Entity::query()
    .order_by_desc(Column::CreatedAt)
    .all()
    .await?;

// Multiple orderings
let todos = todo::Entity::query()
    .order_by_desc(Column::Priority)
    .order_by_asc(Column::Title)
    .all()
    .await?;
```

### Pagination

```rust
use ferro::models::todo;

// Limit results
let todos = todo::Entity::query()
    .limit(10)
    .all()
    .await?;

// Offset and limit (pagination)
let page = 2;
let per_page = 10;

let todos = todo::Entity::query()
    .offset((page - 1) * per_page)
    .limit(per_page)
    .all()
    .await?;
```

### Counting and Existence

```rust
use ferro::models::todo::{self, Column};

// Count matching records
let count = todo::Entity::query()
    .filter(Column::Active.eq(true))
    .count()
    .await?;

// Check if any exist
let has_active = todo::Entity::query()
    .filter(Column::Active.eq(true))
    .exists()
    .await?;
```

### Advanced Queries

```rust
use ferro::models::todo;

// Get underlying SeaORM Select for advanced operations
let select = todo::Entity::query()
    .filter(Column::Active.eq(true))
    .into_select();

// Use with SeaORM directly
let conn = DB::connection()?;
let todos = select
    .join(JoinType::LeftJoin, todo::Relation::User.def())
    .all(conn.inner())
    .await?;
```

## Route Model Binding

Ferro automatically resolves models from route parameters.

### Automatic Binding

```rust
use ferro::{handler, json_response, Response};
use ferro::models::user;

// The 'user' parameter is automatically resolved from the route
#[handler]
pub async fn show(user: user::Model) -> Response {
    json_response!({ "name": user.name, "email": user.email })
}

// Route definition: get!("/users/{user}", controllers::user::show)
// The {user} parameter is parsed as the primary key and the model is fetched
```

### How It Works

1. Route parameter `{user}` is extracted from the URL
2. The parameter value is parsed as the model's primary key type
3. The model is fetched from the database
4. If not found, a 404 response is returned automatically
5. If the parameter can't be parsed, a 400 response is returned

### Custom Route Binding

For custom binding logic, implement the `RouteBinding` trait:

```rust
use ferro::{RouteBinding, FrameworkError};
use async_trait::async_trait;

#[async_trait]
impl RouteBinding for user::Model {
    fn param_name() -> &'static str {
        "user"
    }

    async fn from_route_param(value: &str) -> Result<Self, FrameworkError> {
        // Custom logic: find by email instead of ID
        let conn = DB::connection()?;
        user::Entity::find()
            .filter(user::Column::Email.eq(value))
            .one(conn.inner())
            .await?
            .ok_or_else(|| FrameworkError::model_not_found("User"))
    }
}
```

## Migrations

Ferro uses SeaORM migrations with a timestamp-based naming convention.

### Creating Migrations

```bash
# Create a new migration
ferro make:migration create_posts_table

# Creates: src/migrations/m20240115_143052_create_posts_table.rs
```

### Migration Structure

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Posts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Posts::Title).string().not_null())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .col(
                        ColumnDef::new(Posts::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Posts::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Posts {
    Table,
    Id,
    Title,
    Content,
    CreatedAt,
    UpdatedAt,
}
```

### Running Migrations

```bash
# Run all pending migrations
ferro db:migrate

# Rollback the last batch
ferro db:rollback

# Rollback all and re-run
ferro db:fresh

# Check migration status
ferro db:status
```

## Testing

Ferro provides utilities for isolated database testing.

### Test Database

```rust
use ferro::test_database;
use ferro::models::user;

#[tokio::test]
async fn test_create_user() {
    // Creates fresh in-memory SQLite with migrations
    let db = test_database!();

    // Your test code - DB::connection() automatically uses test database
    let new_user = user::ActiveModel {
        name: Set("Test User".to_string()),
        email: Set("test@example.com".to_string()),
        ..Default::default()
    };

    let user = user::Entity::insert_one(new_user).await.unwrap();
    assert!(user.id > 0);

    // Query directly
    let found = user::Entity::find_by_id(user.id)
        .one(db.conn())
        .await
        .unwrap();
    assert!(found.is_some());
}
```

### Custom Migrator

```rust
use ferro::TestDatabase;

#[tokio::test]
async fn test_with_custom_migrator() {
    let db = TestDatabase::fresh::<my_crate::CustomMigrator>()
        .await
        .unwrap();

    // Test code here
}
```

### Isolation

Each `TestDatabase` creates a completely isolated in-memory database:
- Fresh database for each test
- Migrations are run automatically
- No interference between tests
- Automatically cleaned up when dropped

## Dependency Injection

Use the `Database` type alias with dependency injection:

```rust
use ferro::{injectable, Database};

#[injectable]
pub struct CreateUserAction {
    #[inject]
    db: Database,
}

impl CreateUserAction {
    pub async fn execute(&self, email: &str) -> Result<user::Model, Error> {
        let new_user = user::ActiveModel {
            email: Set(email.to_string()),
            ..Default::default()
        };

        new_user.insert(self.db.conn()).await
    }
}
```

## Environment Variables Reference

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | Database connection URL | `sqlite://./database.db` |
| `DB_MAX_CONNECTIONS` | Maximum pool connections | `10` |
| `DB_MIN_CONNECTIONS` | Minimum pool connections | `1` |
| `DB_CONNECT_TIMEOUT` | Connection timeout (seconds) | `30` |
| `DB_LOGGING` | Enable SQL query logging | `false` |

## Supported Databases

| Database | URL Format | Notes |
|----------|------------|-------|
| PostgreSQL | `postgres://user:pass@host:5432/db` | Recommended for production |
| SQLite | `sqlite://./path/to/db.sqlite` | Great for development |
| SQLite (memory) | `sqlite::memory:` | For testing |

## Best Practices

1. **Use migrations** - Never modify database schema manually
2. **Implement Model traits** - Get convenient static methods for free
3. **Use QueryBuilder** - Cleaner API than raw SeaORM queries
4. **Leverage route binding** - Automatic 404 handling for missing models
5. **Test with test_database!** - Isolated, repeatable tests
6. **Use dependency injection** - Cleaner code with `#[inject] db: Database`
7. **Enable logging in development** - `DB_LOGGING=true` for debugging
8. **Set appropriate pool sizes** - Match your expected concurrency

## MCP Tools

Ferro provides seven database-focused MCP tools covering schema inspection, query execution, migration status, model analysis, and relationship mapping.

### `database_schema`

- **Returns:** All tables with column names, types, nullability, defaults, and indexes
- **When to use:** Understand the current database structure; verify a migration ran correctly; check column types before writing a query

### `database_query`

- **Returns:** Query results as JSON rows
- **When to use:** Run ad-hoc SELECT queries against the live database to inspect data; debug unexpected values; validate seeded test data. Read-only — does not support INSERT, UPDATE, or DELETE.

### `list_migrations`

- **Returns:** All migration files with their status (applied or pending), applied-at timestamp, and the migration name
- **When to use:** Check whether pending migrations need to be run; verify the current migration state before deployment

### `list_models`

- **Returns:** All SeaORM entity structs found in `src/models/`, with field names, types, and SeaORM column annotations
- **When to use:** Discover all models in the project; find the correct field names before writing a query or resource

### `explain_model`

- **Returns:** Detailed breakdown of one model: fields, types, relations, and any Ferro trait implementations (`Model`, `ModelMut`, `RouteBinding`)
- **When to use:** Deep-dive into a single model's structure; verify that the correct traits are implemented; understand relation definitions

### `model_usages`

- **Returns:** All locations in source code that reference a given model, grouped by file and usage type (query, insert, update, delete, relation)
- **When to use:** Understand how widely a model is used before changing its schema; find all query sites when adding a new filter condition

### `relation_map`

- **Returns:** A graph of all model relationships: belongs-to, has-many, and many-to-many edges derived from SeaORM `Relation` enums
- **When to use:** Visualize the entity relationship diagram; verify that relations are defined correctly on both sides; plan batch loading strategies to avoid N+1 queries
