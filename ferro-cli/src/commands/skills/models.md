---
name: ferro:models
description: List all models with their fields
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
---

<objective>
List all models in the Ferro application with their fields and relationships.

Uses ferro-mcp `list_models` and `explain_model` tools for detailed information.
</objective>

<arguments>
Optional:
- `[name]` - Show details for specific model
- `--relations` - Include relationship information
- `--json` - Output as JSON

Examples:
- `/ferro:models` - List all models
- `/ferro:models User` - Show User model details
- `/ferro:models --relations` - Include relationships
</arguments>

<process>

<step name="get_models">

**Primary method:** Use ferro-mcp `list_models` tool.

**Fallback method:** Scan model files:

```bash
# Find model files
ls src/models/*.rs 2>/dev/null | grep -v "mod.rs" | grep -v "entities"
```

Then read each file to extract model information.

</step>

<step name="list_all">

**When no specific model requested:**

Display all models in a list:

```
# Models

## User
- id: i32 (primary key)
- email: String (unique)
- name: String
- password: String
- created_at: DateTime
- updated_at: DateTime

Relations: has_many Posts, has_many Comments

## Post
- id: i32 (primary key)
- user_id: i32 (foreign key -> users)
- title: String
- body: String
- published_at: Option<DateTime>
- created_at: DateTime
- updated_at: DateTime

Relations: belongs_to User, has_many Comments

## Comment
- id: i32 (primary key)
- post_id: i32 (foreign key -> posts)
- user_id: i32 (foreign key -> users)
- body: String
- created_at: DateTime
- updated_at: DateTime

Relations: belongs_to User, belongs_to Post

---
Total: 3 models
```

</step>

<step name="show_detail">

**When specific model requested:**

Use ferro-mcp `explain_model` tool for detailed view:

```
# User Model

## Fields
┌─────────────┬──────────────┬──────────┬─────────────────┐
│ Field       │ Type         │ Nullable │ Attributes      │
├─────────────┼──────────────┼──────────┼─────────────────┤
│ id          │ i32          │ NO       │ primary_key, ai │
│ email       │ String       │ NO       │ unique          │
│ name        │ String       │ NO       │                 │
│ password    │ String       │ NO       │                 │
│ avatar      │ Option<String>│ YES     │                 │
│ created_at  │ DateTime     │ NO       │ auto            │
│ updated_at  │ DateTime     │ NO       │ auto            │
└─────────────┴──────────────┴──────────┴─────────────────┘

## Relations
- has_many: Posts (user_id)
- has_many: Comments (user_id)

## Indexes
- PRIMARY: id
- UNIQUE: email

## Custom Methods
- find_by_email(email: &str) -> Option<Self>
- verify_password(password: &str) -> bool

## File
src/models/users.rs

## Usage
```rust
// Find by ID
let user = User::find(1).await?;

// Query
let users = User::query()
    .filter(Column::Email.contains("@example.com"))
    .all()
    .await?;

// Create
let user = User::create(CreateUser {
    email: "test@example.com".into(),
    name: "Test".into(),
    password: hash("secret"),
}).await?;
```
```

</step>

</process>
