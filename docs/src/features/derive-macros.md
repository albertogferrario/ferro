# Derive Macros

Ferro provides two derive macros that eliminate boilerplate for database models and validation. `FerroModel` scaffolds CRUD operations on SeaORM entities; `ValidateRules` co-locates validation rules with struct field definitions.

## FerroModel

`FerroModel` generates `create`, `update`, `delete`, and `query` methods from a SeaORM entity model struct. Place it alongside `DeriveEntityModel` in the derive list.

### Entity Definition

```rust
use ferro::FerroModel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, FerroModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub author_id: i32,
    pub slug: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

### Generated API

**Create** — returns a builder with a setter for each non-primary-key field:

```rust
let post = Post::create()
    .set_title("Hello")
    .set_body("World")
    .set_published(false)
    .set_author_id(1)
    .insert()
    .await?;
```

**Update** — selectively updates fields on an existing model instance:

```rust
let post = post.update()
    .set_title("Updated Title")
    .save()
    .await?;
```

**Clear optional fields** — `clear_<field>()` sets a nullable field to `None`:

```rust
let post = post.update()
    .clear_slug()
    .save()
    .await?;
```

**Delete** — removes the row from the database:

```rust
post.delete().await?;
```

**Query** — returns a SeaORM `Select<Entity>` for building queries:

```rust
let published_posts = Post::query()
    .filter(Column::Published.eq(true))
    .all(&db)
    .await?;
```

### Generated Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `Entity::create()` | `CreateBuilder` | Builder for inserting a new row |
| `instance.update()` | `UpdateBuilder` | Builder for selectively updating fields |
| `instance.delete()` | `Result` | Deletes the row from the database |
| `Entity::query()` | `Select<Entity>` | SeaORM Select for building queries |

The primary key field is excluded from `CreateBuilder`. `Option<T>` fields gain both `set_<field>()` and `clear_<field>()` on `UpdateBuilder`.

## ValidateRules

`ValidateRules` generates a `.validate()` method from `#[rule(...)]` attributes on struct fields. It uses Ferro's built-in rule functions — the same functions available in the fluent `Validator::new()` API (documented in [Validation](validation.md)).

Note: `#[rule(...)]` is Ferro's own attribute syntax. It is not the same as the `#[validate(...)]` attribute from the `validator` crate.

### Struct Definition

```rust
use ferro::ValidateRules;

#[derive(ValidateRules)]
struct RegistrationRequest {
    #[rule(required, email)]
    email: String,

    #[rule(required, min(8))]
    password: String,

    #[rule(required, string, min(2), max(50))]
    name: String,

    #[rule(required, integer, min(18), max(120))]
    age: i32,

    #[rule(nullable, url)]
    website: Option<String>,
}
```

### Usage

```rust
let req = RegistrationRequest { /* ... */ };
req.validate()?;  // Returns Result<(), ValidationErrors>
```

The `.validate()` method returns `Ok(())` when all rules pass, or `Err(ValidationErrors)` with per-field error messages when any rule fails.

### Available Rules

| Rule | Description | Example |
|------|-------------|---------|
| `required` | Field must be present and non-empty | `#[rule(required)]` |
| `email` | Must be a valid email address | `#[rule(required, email)]` |
| `string` | Must be a string value | `#[rule(string)]` |
| `integer` | Must be an integer value | `#[rule(integer)]` |
| `min(n)` | Minimum value or minimum string length | `#[rule(min(8))]` |
| `max(n)` | Maximum value or maximum string length | `#[rule(max(100))]` |
| `between(a, b)` | Value between `a` and `b` inclusive | `#[rule(between(1, 10))]` |
| `url` | Must be a valid URL | `#[rule(url)]` |
| `nullable` | Field may be `None`; skips other rules when absent | `#[rule(nullable, url)]` |

Rules are evaluated in declaration order. For `Option<T>` fields, use `nullable` first to skip subsequent rules when the value is `None`.

## See Also

- [Database](database.md) — SeaORM entity setup and query patterns
- [Validation](validation.md) — fluent `Validator::new()` API

## MCP Tools

Use `explain_model` to inspect the generated CRUD API for a `FerroModel`-derived entity.

### `explain_model`

Returns the full structure of a SeaORM entity, including which fields are included in the `CreateBuilder` and `UpdateBuilder`, optional fields that gain `clear_*()` methods, and any `RouteBinding` implementation. This is the same tool documented in [Database](database.md) — it works on any entity, whether or not it uses `#[derive(FerroModel)]`.
