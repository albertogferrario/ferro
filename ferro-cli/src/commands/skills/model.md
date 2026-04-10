---
name: ferro:model
description: Generate a new model with migration
allowed-tools:
  - Bash
  - Read
  - Write
  - Glob
  - AskUserQuestion
---

<objective>
Generate a new Ferro model with corresponding database migration.

Creates:
1. Model file in `src/models/{name}.rs`
2. Entity file in `src/models/entities/{name}.rs` (if using db:sync)
3. Migration file in `migrations/`
4. Updates `src/models/mod.rs` to include the new model
</objective>

<arguments>
Required:
- `name` - Model name in PascalCase (e.g., `Post`, `UserProfile`)

Optional:
- `fields` - Field definitions in `name:type` format
  - Types: string, text, integer, bigint, boolean, datetime, date, decimal, json, uuid

Examples:
- `/ferro:model Post`
- `/ferro:model Post title:string body:text published:boolean`
- `/ferro:model Comment post_id:integer user_id:integer body:text`
</arguments>

<process>

<step name="parse_args">

Parse the model name and fields from arguments.

If no fields provided, ask:
```
What fields should this model have?

Common patterns:
- Blog post: title:string body:text slug:string published_at:datetime
- User profile: user_id:integer bio:text avatar:string
- Product: name:string description:text price:decimal stock:integer
```

</step>

<step name="check_existing">

Check if model already exists:

```bash
if [ -f "src/models/{snake_name}.rs" ]; then
    echo "EXISTS"
fi
```

If exists, ask whether to overwrite or abort.

</step>

<step name="generate_migration">

Generate migration file with timestamp:

```bash
TIMESTAMP=$(date +%Y%m%d%H%M%S)
MIGRATION_NAME="${TIMESTAMP}_create_{table_name}_table"
```

Migration content:
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
                    .table({TableName}::Table)
                    .if_not_exists()
                    .col(ColumnDef::new({TableName}::Id).integer().not_null().auto_increment().primary_key())
                    {field_columns}
                    .col(ColumnDef::new({TableName}::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new({TableName}::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table({TableName}::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum {TableName} {
    Table,
    Id,
    {field_idents}
    CreatedAt,
    UpdatedAt,
}
```

</step>

<step name="generate_model">

Generate model file at `src/models/{snake_name}.rs`:

```rust
//! {ModelName} model
//!
//! This file contains custom implementations for the {ModelName} model.
//! The base entity is auto-generated in src/models/entities/{snake_name}.rs
//!
//! This file is NEVER overwritten by `ferro db:sync` - your custom code is safe here.

// Re-export the auto-generated entity
pub use super::entities::{snake_name}::*;

/// Type alias for convenient access
#[allow(dead_code)]
pub type {ModelName} = Model;

// ============================================================================
// CUSTOM METHODS
// Add your custom query and mutation methods below
// ============================================================================

// impl Model {
//     pub async fn find_by_slug(slug: &str) -> Result<Option<Self>, ferro::FrameworkError> {
//         Self::query().filter(Column::Slug.eq(slug)).first().await
//     }
// }

// ============================================================================
// RELATIONS
// Define relationships to other entities here
// ============================================================================

// impl Entity {
//     pub fn belongs_to_user() -> RelationDef {
//         Entity::belongs_to(super::users::Entity)
//             .from(Column::UserId)
//             .to(super::users::Column::Id)
//             .into()
//     }
// }
```

</step>

<step name="update_mod">

Update `src/models/mod.rs` to include the new model:

```rust
pub mod {snake_name};
```

</step>

<step name="summary">

Output summary:
```
Created model: {ModelName}

Files created:
  - src/models/{snake_name}.rs
  - migrations/{timestamp}_create_{table_name}_table.rs

Next steps:
  1. Run migration: ferro db:migrate (or cargo run -- db:migrate)
  2. Sync entities: ferro db:sync (generates src/models/entities/{snake_name}.rs)
  3. Add relationships in src/models/{snake_name}.rs
```

</step>

</process>

<field_type_mapping>
| Input Type | Rust Type      | SeaORM Column              |
|------------|----------------|----------------------------|
| string     | String         | ColumnDef::string()        |
| text       | String         | ColumnDef::text()          |
| integer    | i32            | ColumnDef::integer()       |
| bigint     | i64            | ColumnDef::big_integer()   |
| boolean    | bool           | ColumnDef::boolean()       |
| datetime   | DateTimeUtc    | ColumnDef::timestamp()     |
| date       | Date           | ColumnDef::date()          |
| decimal    | Decimal        | ColumnDef::decimal()       |
| json       | Json           | ColumnDef::json()          |
| uuid       | Uuid           | ColumnDef::uuid()          |
</field_type_mapping>
