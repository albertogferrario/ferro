# REST API Scaffold

Ferro generates a production-ready REST API from existing models with one command. The scaffold includes CRUD controllers, API key authentication, rate limiting, OpenAPI documentation, and request validation types.

## Quick Start

```bash
# Generate API for all models
ferro make:api --all

# Generate for specific models
ferro make:api User Post

# Skip confirmation prompts
ferro make:api --all --yes
```

After generation, wire the scaffold into your application:

1. Add `mod api;` to `src/main.rs` or `src/lib.rs`
2. Register `api_routes()` in your route configuration
3. Register `docs_routes()` for API documentation
4. Register `ApiKeyProviderImpl` as a service: `App::bind::<dyn ApiKeyProvider>(Box::new(ApiKeyProviderImpl));`
5. Run `ferro db:migrate` to create the `api_keys` table
6. Generate your first API key programmatically

## Generated Files

`ferro make:api` generates the following files for each model:

| File | Purpose |
|------|---------|
| `src/api/{model}_api.rs` | CRUD controller with index, show, store, update, destroy handlers |
| `src/resources/{model}_resource.rs` | API resource with `Resource` trait implementation and `From<Model>` conversion |
| `src/requests/{model}_request.rs` | `Create{Model}Request` and `Update{Model}Request` with validation |

Infrastructure files (generated once):

| File | Purpose |
|------|---------|
| `src/api/mod.rs` | Module declarations for all API controllers |
| `src/api/routes.rs` | Route group with `ApiKeyMiddleware` and `Throttle` middleware |
| `src/api/docs.rs` | OpenAPI JSON and ReDoc HTML handlers |
| `src/models/api_key.rs` | SeaORM entity for the `api_keys` table |
| `src/providers/api_key_provider.rs` | `ApiKeyProvider` implementation with revocation and expiry checks |
| `src/migrations/m*_create_api_keys_table.rs` | Migration for the `api_keys` table |

Existing files are never overwritten. If a file already exists, it is skipped with an info message.

## API Key Authentication

### Key Format

API keys follow the `fe_{env}_{random}` pattern:

- `fe_live_` prefix for production keys
- `fe_test_` prefix for test/development keys
- 43 random base62 characters for the secret portion

Full key length: 51 characters (e.g., `fe_live_aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789abcde`).

### Generating Keys

```rust
use ferro::generate_api_key;

let key = generate_api_key("live");

// Show the raw key to the user exactly once
println!("Your API key: {}", key.raw_key);

// Store these in the database
// key.prefix — first 16 chars, used for DB lookup (unhashed)
// key.hashed_key — SHA-256 hex digest, used for verification
```

The raw key is never stored. After creation, only the prefix and hash are persisted. If the user loses the key, a new one must be generated.

### Storage Schema

The generated `api_keys` migration creates:

| Column | Type | Description |
|--------|------|-------------|
| `id` | `BIGINT PK` | Auto-increment identifier |
| `name` | `VARCHAR` | Human-readable label (e.g., "Production Bot") |
| `prefix` | `VARCHAR(16)` | First 16 characters for indexed lookup |
| `hashed_key` | `VARCHAR(64)` | SHA-256 hex digest |
| `scopes` | `TEXT NULL` | JSON array of permission scopes |
| `last_used_at` | `TIMESTAMPTZ NULL` | Last request timestamp |
| `expires_at` | `TIMESTAMPTZ NULL` | Expiration timestamp |
| `revoked_at` | `TIMESTAMPTZ NULL` | Revocation timestamp |
| `created_at` | `TIMESTAMPTZ` | Creation timestamp |

An index on `prefix` enables fast key lookup.

### Verification Flow

1. Extract `Bearer {key}` from the `Authorization` header
2. Look up the key record by prefix (first 16 characters)
3. Check revocation (`revoked_at IS NULL`)
4. Check expiry (`expires_at` not passed)
5. Constant-time SHA-256 hash comparison via `subtle::ConstantTimeEq`
6. Check required scopes against granted scopes
7. Store `ApiKeyInfo` in request extensions for downstream handlers

### ApiKeyProvider Trait

The middleware resolves an `ApiKeyProvider` from the service container. Implement this trait to connect to any key store:

```rust
use ferro::{async_trait, ApiKeyProvider, ApiKeyInfo};

pub struct MyKeyProvider;

#[async_trait]
impl ApiKeyProvider for MyKeyProvider {
    async fn verify_key(&self, raw_key: &str) -> Result<ApiKeyInfo, ()> {
        let prefix = &raw_key[..16.min(raw_key.len())];
        // Look up by prefix, verify hash, return metadata
        // ...
        Ok(ApiKeyInfo {
            id: record.id,
            name: record.name,
            scopes: vec!["read".to_string(), "write".to_string()],
        })
    }
}
```

Register in bootstrap:

```rust
App::bind::<dyn ApiKeyProvider>(Box::new(MyKeyProvider));
```

The generated `ApiKeyProviderImpl` in `src/providers/api_key_provider.rs` provides a complete database-backed implementation with revocation and expiry checks.

## API Key Management

### CLI Key Generation

Generate API keys from the command line without writing code:

```bash
ferro make:api-key "My App"
```

Options:

| Flag | Description | Default |
|------|-------------|---------|
| `--env` | Key environment: `live` or `test` | `live` |

Example with test environment:

```bash
ferro make:api-key "Dev Bot" --env test
```

Output includes:

- **Raw key** (e.g., `fe_live_aBcDeFg...`) -- shown once, store securely
- **Prefix** -- first 16 characters, used for database lookup
- **Hashed key** -- SHA-256 hex digest for verification
- **SQL insert** -- ready-to-run INSERT statement
- **Rust snippet** -- copy-paste SeaORM code

The raw key is displayed only once. If lost, generate a new key.

### Creating Keys Programmatically

```rust
use ferro::generate_api_key;
use sea_orm::{EntityTrait, Set};
use crate::models::api_key;

let key = generate_api_key("live");

let record = api_key::ActiveModel {
    name: Set("Production Bot".to_string()),
    prefix: Set(key.prefix),
    hashed_key: Set(key.hashed_key),
    scopes: Set(Some(serde_json::to_string(&["read", "write"]).unwrap())),
    ..Default::default()
};

api_key::Entity::insert(record)
    .exec(&db)
    .await?;

// Return key.raw_key to the user (show once)
```

### Revoking Keys

Set `revoked_at` to the current timestamp. The provider checks this before verifying the hash:

```rust
use sea_orm::{EntityTrait, Set, IntoActiveModel};
use chrono::Utc;

let mut key: api_key::ActiveModel = record.into_active_model();
key.revoked_at = Set(Some(Utc::now()));
key.update(&db).await?;
```

### Key Expiration

Set `expires_at` when creating the key. The provider rejects keys past their expiration:

```rust
use chrono::{Utc, Duration};

let record = api_key::ActiveModel {
    expires_at: Set(Some(Utc::now() + Duration::days(90))),
    // ...
    ..Default::default()
};
```

### Scope-Based Permissions

Scopes are stored as a JSON array in the `scopes` column. The middleware checks required scopes against granted scopes:

```rust
use ferro::ApiKeyMiddleware;

// Require any valid API key
group!("/api/v1")
    .middleware(ApiKeyMiddleware::new())
    .routes([...]);

// Require specific scopes
group!("/api/v1/admin")
    .middleware(ApiKeyMiddleware::scopes(&["admin"]))
    .routes([...]);
```

A key with `["*"]` in its scopes bypasses all scope checks (wildcard).

### Accessing Key Info in Handlers

After middleware verification, `ApiKeyInfo` is available in request extensions:

```rust
use ferro::{handler, Request, Response, HttpResponse, ApiKeyInfo};

#[handler]
pub async fn index(req: Request) -> Response {
    let key_info = req.get::<ApiKeyInfo>().unwrap();
    println!("Request from: {} (scopes: {:?})", key_info.name, key_info.scopes);
    // ...
}
```

## Field Selection

By default, `make:api` auto-excludes sensitive fields from generated API resources. Fields matching these patterns are omitted:

- `password`, `password_hash`, `hashed_password`
- `secret`, `token`, `api_key`, `hashed_key`
- `remember_token`

Matching is case-insensitive, exact match only. A field named `token_count` is not excluded.

### Custom Exclusion

Exclude additional fields with `--exclude`:

```bash
ferro make:api --all --exclude password_hash,secret_token
```

Multiple fields are comma-separated. Custom exclusions stack with auto-exclusion.

### Including All Fields

Override auto-exclusion with `--include-all`:

```bash
ferro make:api --all --include-all
```

When `--include-all` is set, no auto-exclusion is applied. Custom `--exclude` fields are still honored:

```bash
# Include all fields except internal_notes
ferro make:api --all --include-all --exclude internal_notes
```

## Verifying Your API

After scaffolding and wiring routes, verify the setup with `ferro api:check`:

```bash
ferro api:check
```

The command runs four sequential checks:

1. **Server connectivity** -- can the CLI reach your server?
2. **Spec available** -- does `/api/openapi.json` return a response?
3. **Spec valid** -- is the response a valid OpenAPI 3.x document?
4. **Auth working** -- does the API key authenticate successfully?

### With Authentication

```bash
ferro api:check --api-key fe_live_...
```

### Custom URL

```bash
ferro api:check --url http://localhost:3000
```

### Custom Spec Path

```bash
ferro api:check --spec-path /api/docs/openapi.json
```

On success, `api:check` prints a ready-to-copy `ferro-api-mcp` command for MCP integration.

## Endpoints

The generated routes follow standard REST conventions under `/api/v1/`:

### List Resources

```
GET /api/v1/{resource}?page=1&per_page=15
```

```bash
curl -H "Authorization: Bearer fe_live_..." \
     "https://example.com/api/v1/users?page=1&per_page=15"
```

Response:

```json
{
  "data": [
    {"id": 1, "name": "Alice", "email": "alice@example.com"},
    {"id": 2, "name": "Bob", "email": "bob@example.com"}
  ],
  "meta": {
    "current_page": 1,
    "per_page": 15,
    "total": 42,
    "last_page": 3,
    "from": 1,
    "to": 15
  },
  "links": {
    "first": "/api/v1/users?page=1",
    "last": "/api/v1/users?page=3",
    "prev": null,
    "next": "/api/v1/users?page=2"
  }
}
```

`per_page` is capped at 100.

### Create Resource

```
POST /api/v1/{resource}
```

```bash
curl -X POST -H "Authorization: Bearer fe_live_..." \
     -H "Content-Type: application/json" \
     -d '{"name": "Charlie", "email": "charlie@example.com"}' \
     "https://example.com/api/v1/users"
```

Response (201):

```json
{
  "data": {"id": 3, "name": "Charlie", "email": "charlie@example.com"}
}
```

### Show Resource

```
GET /api/v1/{resource}/{id}
```

```bash
curl -H "Authorization: Bearer fe_live_..." \
     "https://example.com/api/v1/users/1"
```

Response:

```json
{
  "data": {"id": 1, "name": "Alice", "email": "alice@example.com"}
}
```

### Update Resource

```
PUT /api/v1/{resource}/{id}
```

```bash
curl -X PUT -H "Authorization: Bearer fe_live_..." \
     -H "Content-Type: application/json" \
     -d '{"name": "Alice Smith"}' \
     "https://example.com/api/v1/users/1"
```

Response:

```json
{
  "data": {"id": 1, "name": "Alice Smith", "email": "alice@example.com"}
}
```

Update handlers use conditional field assignment (`if let Some`), so only included fields are modified.

### Delete Resource

```
DELETE /api/v1/{resource}/{id}
```

```bash
curl -X DELETE -H "Authorization: Bearer fe_live_..." \
     "https://example.com/api/v1/users/1"
```

Response:

```json
{"message": "Deleted"}
```

## OpenAPI Documentation

The scaffold includes auto-generated OpenAPI 3.0 documentation:

| Endpoint | Description |
|----------|-------------|
| `/api/docs` | Interactive ReDoc UI |
| `/api/openapi.json` | Raw OpenAPI specification |

### How Specs Are Built

The OpenAPI spec builder reads from the Ferro route registry:

1. Filters routes matching the `/api/` prefix
2. Generates operations with auto-summaries (e.g., `GET /api/v1/users` becomes "List users")
3. Extracts path parameters from `{param}` patterns
4. Groups endpoints by resource name as tags
5. Adds API key security scheme (`Authorization` header)

Specs and HTML are cached via `OnceLock` -- generated once on first request, zero cost per subsequent request.

### Configuration

```rust
use ferro::{OpenApiConfig, build_openapi_spec, get_registered_routes};

let config = OpenApiConfig {
    title: "My API".to_string(),
    version: "1.0.0".to_string(),
    description: Some("Application API".to_string()),
    api_prefix: "/api/".to_string(),
};

let spec = build_openapi_spec(&config, &get_registered_routes());
```

The generated `src/api/docs.rs` reads the `APP_NAME` environment variable for the title.

## Rate Limiting

Generated API routes include `Throttle::named("api")` middleware. Define the limiter in `bootstrap.rs`:

```rust
use ferro::middleware::{RateLimiter, Limit};

RateLimiter::define("api", |_req| {
    Limit::per_minute(60)
});
```

The default configuration allows 60 requests per minute per client IP. Adjust the limit or segment by API key:

```rust
use ferro::ApiKeyInfo;

RateLimiter::define("api", |req| {
    match req.get::<ApiKeyInfo>() {
        Some(key) => Limit::per_minute(1000).by(format!("key:{}", key.id)),
        None => Limit::per_minute(60),
    }
});
```

See [Rate Limiting](rate-limiting.md) for full documentation on time windows, multiple limits, custom responses, and cache backends.

## MCP Tools

Ferro's MCP server provides four CRUD tools for direct database access without the HTTP API layer. These enable AI agents to manage application data programmatically.

### `crud_create`

Create a new record for any model:

```json
{
  "model": "User",
  "data": {"name": "Alice", "email": "alice@example.com"}
}
```

Returns the created record as JSON.

### `crud_list`

List records with optional filtering and pagination:

```json
{
  "model": "User",
  "filters": {"status": "active"},
  "page": 1,
  "per_page": 25
}
```

Returns records array with `total`, `page`, and `per_page` metadata. Per-page is capped at 100.

### `crud_update`

Update an existing record by primary key:

```json
{
  "model": "User",
  "id": 1,
  "data": {"name": "Alice Smith"}
}
```

Returns the updated record as JSON.

### `crud_delete`

Delete a record by primary key:

```json
{
  "model": "User",
  "id": 1
}
```

Returns a confirmation message.

### How It Works

The MCP CRUD tools:

1. Parse model metadata via syn AST visitor (same pattern as `ferro make:api`)
2. Validate field names against the model struct definition
3. Build parameterized SQL using `sea_orm::Statement::from_sql_and_values`
4. Execute against the project's configured database
5. Use `RETURNING *` on Postgres, `last_insert_rowid()` fallback on SQLite

All queries use parameterized statements to prevent SQL injection. Timestamp fields (`created_at`, `updated_at`) are excluded from required-field validation since they typically have database defaults.

## Customization

### Adding Custom Endpoints

Add routes to the generated `src/api/routes.rs`:

```rust
pub fn api_routes() -> GroupBuilder {
    group!("/api/v1")
        .middleware(ApiKeyMiddleware::new())
        .middleware(Throttle::named("api"))
        .routes([
            // Generated CRUD routes...
            get!("/users", user_api::index),
            post!("/users", user_api::store),
            // Custom endpoints
            post!("/users/:id/activate", user_api::activate),
            get!("/stats", stats_api::overview),
        ])
}
```

### Modifying Response Format

Edit the generated resource in `src/resources/{model}_resource.rs`. Use `ResourceMap` for conditional fields:

```rust
impl Resource for UserResource {
    fn to_resource(&self, _req: &Request) -> serde_json::Value {
        ResourceMap::new()
            .field("id", json!(self.id))
            .field("name", json!(self.name))
            .when("email", self.is_admin, || json!(self.email))
            .build()
    }
}
```

See [API Resources](api-resources.md) for full documentation on `ResourceMap`, conditional fields, and relationship inclusion.

### Adding Relationships

Load related data in the controller before constructing resources:

```rust
#[handler]
pub async fn show(req: Request, user: user::Model) -> Response {
    let db = ferro::DB::connection()
        .map_err(|e| HttpResponse::json(json!({"error": e.to_string()})).status(500))?;
    let posts = Post::find()
        .filter(posts::Column::UserId.eq(user.id))
        .all(&db)
        .await
        .map_err(|e| HttpResponse::json(json!({"error": e.to_string()})).status(500))?;

    let mut resource = UserResource::from(&user);
    // Add posts to the response
    // ...
    Ok(resource.to_wrapped_response(&req))
}
```

### Custom Validation Rules

Edit the generated request types in `src/requests/{model}_request.rs`:

```rust
#[request]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
```

See [Validation](validation.md) for all available validation rules.

### Custom Scopes

Define application-specific scopes and check them in handlers:

```rust
// In route registration
group!("/api/v1/admin")
    .middleware(ApiKeyMiddleware::scopes(&["admin"]))
    .routes([...]);

// In handler, check additional granular permissions
#[handler]
pub async fn destroy(req: Request, user: user::Model) -> Response {
    let key = req.get::<ApiKeyInfo>().unwrap();
    if !key.scopes.contains(&"users:delete".to_string()) {
        return Err(HttpResponse::json(json!({"error": "Missing users:delete scope"})).status(403));
    }
    // proceed with deletion
}
```

## Security

| Protection | Mechanism |
|-----------|-----------|
| Key storage | SHA-256 hash only; raw key never persisted |
| Timing attacks | Constant-time comparison via `subtle::ConstantTimeEq` |
| Key rotation | Revocation via `revoked_at` timestamp |
| Key expiry | `expires_at` checked on every request |
| SQL injection | Parameterized queries in all CRUD operations |
| Rate limiting | Per-key or per-IP throttling with configurable windows |
| Scope enforcement | Middleware-level scope checking with wildcard support |
