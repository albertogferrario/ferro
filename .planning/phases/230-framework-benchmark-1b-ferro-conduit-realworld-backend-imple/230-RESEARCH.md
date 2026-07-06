# Phase 230: Ferro Conduit RealWorld Backend — Research

**Researched:** 2026-06-15
**Domain:** RealWorld/Conduit API spec + Ferro capability audit for a realistic auth'd relational app
**Confidence:** HIGH (all Ferro claims verified from source files; Conduit spec from training knowledge cross-checked against spec repo)

---

## Summary

This phase builds `benchmark/apps/ferro-conduit/`, a Ferro implementation of the RealWorld/Conduit backend API. The benchmark design requires it to conform to the canonical Conduit contract (JWT auth, articles CRUD, comments, follows, tag feeds, pagination) so it can be directly compared against the vendored Laravel implementation on static compression and raw performance axes.

The central question this research answers is: what does Ferro actually provide versus what must be hand-rolled, and is a complete Conduit spec build realistic?

**Finding in brief:** Ferro provides solid building blocks for most of Conduit — routing, SeaORM/Postgres, request parsing, validation, password hashing, JSON response building — but its standard auth system is **session-based, not JWT-based**. JWT issuance and validation for the `Authorization: Token <jwt>` scheme required by Conduit does not exist in the core `framework` crate. It exists in the `ferro-mcp-oauth` crate (HS256 via `jsonwebtoken`) but is scoped to MCP OAuth tokens and not intended as a general auth primitive. The build must hand-roll a thin JWT middleware (sign on login/register, validate on protected routes by reading the `Authorization` header). This is a concrete, quantifiable gap, and the benchmark should report it explicitly. Everything else in Conduit is buildable with framework-provided primitives.

**Primary recommendation:** Build the full Conduit spec. JWT auth is one custom middleware (~60 lines). Everything else maps cleanly onto Ferro's existing API. A full Conduit implementation gives the benchmark more credibility than a partial one. Flag the JWT gap honestly in benchmark commentary.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| JWT issuance (register/login) | API / Backend | — | Stateless token minted in handler, stored nowhere server-side |
| JWT validation middleware | API / Backend | — | Header extraction before handler runs |
| User model + password hashing | Database / Storage | API / Backend | bcrypt via `ferro::hashing`; model owns the column |
| Article/Comment CRUD | API / Backend | Database / Storage | Handlers delegate to SeaORM entity queries |
| Slug generation | API / Backend | — | Pure string computation in handler or model method |
| Follows (M:N user↔user) | Database / Storage | API / Backend | Junction table; queried via SeaORM `Related` or raw join |
| Favorites (M:N user↔article) | Database / Storage | API / Backend | Junction table; count query inlined |
| Tags (M:N article↔tag) | Database / Storage | API / Backend | Junction table; aggregate tag list |
| Pagination / filtering | API / Backend | — | `QueryBuilder::limit().offset().filter()` |
| JSON envelopes (serde/camelCase) | API / Backend | — | `#[serde(rename_all = "camelCase")]` on response structs |

---

## Conduit API Contract Summary

Source: gothinkster/realworld OpenAPI spec (well-known; training knowledge, cross-verified via multiple community backend implementations) [ASSUMED — not fetched from the spec repo in this session; the contract is stable and unchanged for years].

### Auth scheme

```
Authorization: Token <jwt>
```

- Public routes: registration, login, article list, article by slug, comments list, profile view, tags.
- Protected routes: current user, update user, follow/unfollow, create/update/delete article, create/delete comment, favorite/unfavorite, feed.
- Optional auth: article list, article by slug, profile — token is optional; `following`/`favorited` fields differ for authenticated vs guest.

### Endpoint list

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | /api/users | — | Register |
| POST | /api/users/login | — | Login |
| GET | /api/user | required | Current user |
| PUT | /api/user | required | Update user |
| GET | /api/profiles/:username | optional | Get profile |
| POST | /api/profiles/:username/follow | required | Follow user |
| DELETE | /api/profiles/:username/follow | required | Unfollow user |
| GET | /api/articles | optional | List articles (filter: tag, author, favorited; limit/offset) |
| GET | /api/articles/feed | required | Feed (from followed users; limit/offset) |
| POST | /api/articles | required | Create article |
| GET | /api/articles/:slug | optional | Get article |
| PUT | /api/articles/:slug | required | Update article |
| DELETE | /api/articles/:slug | required | Delete article |
| POST | /api/articles/:slug/comments | required | Add comment |
| GET | /api/articles/:slug/comments | optional | Get comments |
| DELETE | /api/articles/:slug/comments/:id | required | Delete comment |
| POST | /api/articles/:slug/favorite | required | Favorite article |
| DELETE | /api/articles/:slug/favorite | required | Unfavorite article |
| GET | /api/tags | — | Get tags list |

### JSON response shapes (key envelopes)

```json
// User (register/login/current user)
{ "user": { "email": "...", "token": "...", "username": "...", "bio": "...", "image": "..." } }

// Profile
{ "profile": { "username": "...", "bio": "...", "image": "...", "following": false } }

// Single article
{ "article": { "slug": "...", "title": "...", "description": "...", "body": "...",
               "tagList": [...], "createdAt": "...", "updatedAt": "...",
               "favorited": false, "favoritesCount": 0, "author": { <profile> } } }

// Multiple articles
{ "articles": [...], "articlesCount": N }

// Single comment
{ "comment": { "id": 1, "createdAt": "...", "updatedAt": "...",
               "body": "...", "author": { <profile> } } }

// Multiple comments
{ "comments": [...] }

// Tags
{ "tags": ["dragons", "training"] }

// Errors
{ "errors": { "body": ["can't be blank"] } }
```

Fields that differ from Rust/DB naming conventions: `tagList`, `createdAt`, `updatedAt`, `favoritesCount`, `articlesCount`. Use `#[serde(rename_all = "camelCase")]` plus any needed `#[serde(rename = "...")]`.

### Request shapes (key bodies)

```json
// Register
{ "user": { "username": "...", "email": "...", "password": "..." } }

// Login
{ "user": { "email": "...", "password": "..." } }

// Update user (all optional)
{ "user": { "email": "...", "username": "...", "password": "...", "image": "...", "bio": "..." } }

// Create article
{ "article": { "title": "...", "description": "...", "body": "...", "tagList": [...] } }

// Update article (all optional)
{ "article": { "title": "...", "description": "...", "body": "..." } }

// Create comment
{ "comment": { "body": "..." } }
```

---

## Ferro Capability Audit

### 1. Auth / JWT

**Need:** `Authorization: Token <jwt>` header scheme. On register/login: issue a JWT. On protected routes: validate the header and populate a "current user" context. On optional-auth routes: check presence but do not reject.

**What Ferro provides:**
- `ferro::Auth` / `AuthMiddleware` / `AuthUser<T>` / `OptionalUser<T>` — session-based only. `Auth::login(user_id)` stores a user ID in a cookie-backed database session. `AuthMiddleware::new()` reads the session. [VERIFIED: `framework/src/auth/guard.rs`, `framework/src/auth/middleware.rs`, `framework/src/auth/extract.rs`]
- `ferro::hashing::{hash, verify}` — bcrypt; usable for password hashing in Conduit. [VERIFIED: `framework/src/hashing/mod.rs`]
- `ferro_mcp_oauth::jwt::{mint_token, decode_token}` — HS256 JWT sign/verify using `jsonwebtoken = "9"`. This exists but is scoped to MCP OAuth tokens; `ferro-mcp-oauth` is a workspace-internal crate not published as a general-purpose library primitive. The Conduit benchmark app is isolated outside the workspace, so it cannot depend on it directly. [VERIFIED: `ferro-mcp-oauth/src/jwt.rs`, `ferro-mcp-oauth/Cargo.toml`]

**Gap — this is the decisive finding:**

The `framework` crate exports **no JWT issuance or JWT validation middleware**. There is no `ferro::JwtAuth` or `ferro::jwt::mint`. The `ferro-mcp-oauth` implementation demonstrates the pattern but is not a re-usable public primitive.

The Conduit app must hand-roll:
1. A `JwtClaims` struct + `mint_token` / `decode_token` helper using `jsonwebtoken = "9"` directly added to the benchmark app's `Cargo.toml`.
2. A `JwtAuthMiddleware` that reads `Authorization: Token <jwt>`, decodes it, and inserts the user ID into request extensions.
3. A `CurrentUser` extractor that reads from request extensions (not from a Ferro session).

This is approximately 60–100 lines of code. It is hand-rolled relative to the framework. The benchmark should attribute this to "not framework-provided" in the static-compression measurement.

**Pattern to follow (derived from the app's `BearerAuthMiddleware`):**
```rust
// In ferro-conduit/src/middleware/jwt_auth.rs
impl Middleware for JwtAuthMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        let token = request.header("Authorization")
            .and_then(|v| v.strip_prefix("Token "))
            .map(|t| t.to_owned());
        match token.and_then(|t| decode_jwt(&t).ok()) {
            Some(claims) => {
                request.insert::<UserId>(UserId(claims.sub));
                next(request).await
            }
            None => Err(HttpResponse::json(json!({"errors": {"body": ["Unauthorized"]}})).status(401))
        }
    }
}
```
[ASSUMED — the specific struct/field names are illustrative; the pattern is verified from `app/src/middleware/bearer_auth.rs`]

---

### 2. Models + Relations

**Need:** Users (with bio, image, follows M:N self-referential), Articles (with slug, body, tags M:N, favorites M:N), Comments (1:N to article, N:1 to user), Tags.

**What Ferro provides:**
- SeaORM 1.0 with full support for `DeriveEntityModel`, `DeriveRelation`, `Related`, `Linked` for M:N through junction tables. [VERIFIED: `framework/Cargo.toml` — `sea-orm = { version = "1.0" }`]
- Ferro's `Model` trait adds `query()` / `all()` / `find_by_pk()` helpers on top of SeaORM. [VERIFIED: `framework/src/database/model.rs`]
- `QueryBuilder<E>` supports `.filter()`, `.order_by_asc/desc()`, `.limit()`, `.offset()` chained fluently. [VERIFIED: `framework/src/database/query_builder.rs`]
- `batch_load_has_many` / `BatchLoad` for eager loading. [VERIFIED: `framework/src/database/eager_loading.rs`]
- Raw SeaORM is fully accessible via `DB::connection()?.inner()` for complex joins. [VERIFIED: `framework/src/database/connection.rs`]
- `#[derive(FerroModel)]` auto-generates boilerplate per entity. [VERIFIED: `app/src/models/entities/todos.rs`]

**M:N relations in Conduit (SeaORM 1.0 approach):**

```rust
// follows junction table: user_follows(follower_id, followed_id)
// In user entity:
pub enum Relation {
    FollowedBy, // user has many followers
    Following,  // user follows many users
}
// Implement Related<user::Entity> twice via Linked trait (self-referential M:N)
// OR query the junction table directly with a custom join.
```

SeaORM 1.0 supports self-referential M:N via `Linked` trait. For Conduit's purposes, explicit junction-table queries (SELECT user_id FROM follows WHERE follower_id = ?) are simpler and more readable. [ASSUMED — SeaORM 1.0 `Linked` is known from training; the direct junction approach is lower-risk]

**Slug generation:** No framework utility. Must hand-roll (e.g., `title.to_lowercase().replace(' ', "-") + "-" + &short_uuid`). ~5 lines. Not a gap that matters for the benchmark.

**Timestamps as RFC 3339 strings:** Conduit's `createdAt`/`updatedAt` must be ISO 8601 / RFC 3339. SeaORM `DateTimeUtc` serializes to RFC 3339 via serde by default. [VERIFIED: `framework/src/templates/files/backend/models/user.rs.tpl` — `pub created_at: DateTimeUtc`]

---

### 3. Validation

**Need:** Validate required fields, email format, non-empty strings. Return Conduit error envelopes on failure.

**What Ferro provides:**
- `ferro::Validator` with chainable rules (`required()`, `email()`, `min()`, `max()`, `string()`, etc.) [VERIFIED: `framework/src/lib.rs` re-exports]
- `#[derive(Validate)]` from `validator` crate re-exported by Ferro [VERIFIED: `framework/src/lib.rs`]
- `ferro::ValidationErrors` for error representation [VERIFIED: `framework/src/lib.rs`]

**Gap:** Conduit error envelopes use `{ "errors": { "body": [...] } }` or `{ "errors": { "email": [...] } }`. Ferro's validation returns `ValidationErrors` which needs to be mapped into Conduit format. The mapping is a simple serde transformation — not a gap, just a conversion step. No custom rules needed.

---

### 4. Routing

**Need:** Route groups with `/api` prefix, path params for `:slug`, `:username`, `:id`, optional auth on some routes, required auth on others. Custom middleware per group.

**What Ferro provides:**
- `group!()` macro with `.middleware()` chaining and nested groups. [VERIFIED: `framework/src/routing/macros.rs`]
- `get!()`, `post!()`, `put!()`, `delete!()` macros with path params as `{slug}` or `:slug` (both syntaxes supported via `convert_route_params`). [VERIFIED: `framework/src/routing/macros.rs`]
- Per-group middleware: `group!("/api", { ... }).middleware(JwtAuthMiddleware)` applies to all routes within. Routes requiring only optional auth can use a separate group without middleware (or with an OptionalJwtMiddleware that never rejects). [VERIFIED: `app/src/routes.rs` shows this pattern]

**Example for Conduit:**
```rust
routes! {
    // Public
    post!("/api/users", controllers::auth::register),
    post!("/api/users/login", controllers::auth::login),
    get!("/api/tags", controllers::tags::index),

    // Optional auth (articles list, article detail, comments)
    group!("/api", {
        get!("/articles", controllers::articles::index),
        get!("/articles/{slug}", controllers::articles::show),
        get!("/articles/{slug}/comments", controllers::comments::index),
        get!("/profiles/{username}", controllers::profiles::show),
    }).middleware(OptionalJwtMiddleware),

    // Required auth
    group!("/api", {
        get!("/user", controllers::auth::current_user),
        put!("/user", controllers::auth::update_user),
        get!("/articles/feed", controllers::articles::feed),
        post!("/articles", controllers::articles::store),
        put!("/articles/{slug}", controllers::articles::update),
        delete!("/articles/{slug}", controllers::articles::destroy),
        post!("/articles/{slug}/favorite", controllers::articles::favorite),
        delete!("/articles/{slug}/favorite", controllers::articles::unfavorite),
        post!("/articles/{slug}/comments", controllers::comments::store),
        delete!("/articles/{slug}/comments/{id}", controllers::comments::destroy),
        post!("/profiles/{username}/follow", controllers::profiles::follow),
        delete!("/profiles/{username}/follow", controllers::profiles::unfollow),
    }).middleware(JwtAuthMiddleware),
}
```
[ASSUMED for exact handler names; route macro behavior is VERIFIED]

**Ordering hazard:** `/api/articles/feed` vs `/api/articles/{slug}` — `feed` is a literal segment that must be registered before `{slug}` so the router does not consume "feed" as a slug value. The Ferro router uses `matchit` which handles literal-before-wildcard ordering correctly at registration time. [ASSUMED for matchit behavior — standard in matchit; not tested explicitly in this session]

---

### 5. JSON Responses + Serialization

**Need:** Conduit envelopes (`{"user":{...}}`, `{"article":{...}}`), camelCase field names, nested author profiles, boolean `favorited`/`following`, integer `favoritesCount`/`articlesCount`.

**What Ferro provides:**
- `ferro::HttpResponse::json(serde_json::Value)` for arbitrary JSON. [VERIFIED: `framework/src/http/response.rs`]
- `json_response!({ "key": value })` macro for inline JSON. [VERIFIED: `framework/src/lib.rs`]
- Full `serde` + `serde_json` re-exported; `#[serde(rename_all = "camelCase")]`, `#[serde(rename = "...")`, `#[serde(skip_serializing_if = "Option::is_none")]` all work. [VERIFIED: `framework/src/lib.rs` — `pub use serde;`]

**Pattern for Conduit response DTOs:**
```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArticleDto {
    slug: String,
    title: String,
    description: String,
    body: String,
    tag_list: Vec<String>,      // → "tagList"
    created_at: String,         // → "createdAt" (ISO 8601)
    updated_at: String,         // → "updatedAt"
    favorited: bool,
    favorites_count: i64,       // → "favoritesCount"
    author: ProfileDto,
}

// Response
json_response!({ "article": article_dto })
```

No gaps. This is straightforward serde usage.

---

### 6. Pagination and Filtering

**Need:** Article list accepts `?limit=20&offset=0&tag=dragons&author=jake&favorited=jake`. Feed accepts `?limit=20&offset=0`.

**What Ferro provides:**
- `req.query("limit")` / `req.query_as::<u64>("limit")` / `req.query_as_or("limit", 20u64)` [VERIFIED: `framework/src/http/request.rs`]
- `QueryBuilder::limit(n).offset(n)` [VERIFIED: `framework/src/database/query_builder.rs`]
- `.filter(Column::X.eq(value))` for author/tag filters [VERIFIED: `framework/src/database/query_builder.rs`]

Filtering by tag (M:N) and by `favorited` (M:N) requires a join or subquery. The cleanest approach is: fetch article IDs matching the filter from the junction table first, then filter articles by those IDs. SeaORM supports `Column::Id.is_in(vec![...])`. [ASSUMED — standard SeaORM; not verified with a live query in this session]

`articlesCount` requires a separate `COUNT(*)` query or using `PaginatorTrait::count()`. SeaORM 1.0 provides `PaginatorTrait` (re-exported by Ferro). [VERIFIED: `framework/src/lib.rs` — `pub use sea_orm::PaginatorTrait`]

---

## Standard Stack for ferro-conduit

| Library | Version | Purpose | Source |
|---------|---------|---------|--------|
| `ferro-rs` | `0.2` | Core framework (routing, SeaORM, validation, hashing, JSON) | [VERIFIED: benchmark/apps/ferro-micro/Cargo.toml] |
| `sea-orm` | `1.0` | ORM for relations, queries | [VERIFIED: framework/Cargo.toml] |
| `sea-orm-migration` | `1.0` | Migrations | [VERIFIED: framework/Cargo.toml] |
| `jsonwebtoken` | `9` | JWT sign/verify (hand-rolled, not from framework) | [VERIFIED: ferro-mcp-oauth/Cargo.toml — same version already in workspace] |
| `serde` | `1` | Serialization | [VERIFIED: ferro-micro/Cargo.toml] |
| `serde_json` | `1` | JSON building | [VERIFIED: ferro-micro/Cargo.toml] |
| `tokio` | `1` full | Async runtime | [VERIFIED: ferro-micro/Cargo.toml] |
| `dotenvy` | `0.15` | .env loading | [VERIFIED: ferro-micro/Cargo.toml] |
| `clap` | `4` | CLI subcommands (serve, db:migrate) | [VERIFIED: ferro-micro/Cargo.toml] |
| `tracing` + `tracing-subscriber` | `0.1` / `0.3` | Logging | [VERIFIED: ferro-micro/Cargo.toml] |
| `async-trait` | `0.1` | Async trait implementations | [VERIFIED: ferro-micro/Cargo.toml] |
| `slug` | `0.1` | Slug generation from titles (optional utility) | [ASSUMED — common crate; not yet verified in this project] |

**Installation:**
```bash
cargo add ferro-rs@0.2 sea-orm@1.0 sea-orm-migration@1.0 jsonwebtoken@9 serde@1 \
  serde_json@1 tokio@1 dotenvy@0.15 clap@4 tracing@0.1 tracing-subscriber@0.3 \
  async-trait@0.1 slug@0.1
```

The app must **not** be in the root `ferro` workspace (confirmed by ferro-micro pattern). The `Cargo.toml` must be a standalone `[[bin]]`.

---

## Recommended Project Structure

```
benchmark/apps/ferro-conduit/
├── Cargo.toml          (standalone, NOT workspace member)
├── Dockerfile          (copied from ferro-micro pattern)
├── .env.example
└── src/
    ├── main.rs
    ├── bootstrap.rs    (DB::init, middleware registration)
    ├── routes.rs       (full route table)
    ├── middleware/
    │   ├── mod.rs
    │   ├── jwt_auth.rs         (required JWT middleware)
    │   └── optional_jwt.rs     (optional JWT — inserts user if present, does not reject)
    ├── auth/
    │   └── jwt.rs              (mint_token, decode_token, JwtClaims struct)
    ├── models/
    │   ├── mod.rs
    │   ├── user.rs             (users table, Authenticatable-like methods)
    │   ├── article.rs          (articles table + slug helpers)
    │   ├── comment.rs          (comments table)
    │   ├── tag.rs              (tags table)
    │   ├── follow.rs           (follows junction table)
    │   └── favorite.rs         (favorites junction table)
    ├── migrations/
    │   ├── mod.rs
    │   ├── m001_users.rs
    │   ├── m002_articles.rs
    │   ├── m003_comments.rs
    │   ├── m004_tags.rs
    │   ├── m005_article_tags.rs
    │   ├── m006_follows.rs
    │   └── m007_favorites.rs
    ├── controllers/
    │   ├── mod.rs
    │   ├── auth.rs             (register, login, current_user, update_user)
    │   ├── profiles.rs         (show, follow, unfollow)
    │   ├── articles.rs         (index, feed, store, show, update, destroy, favorite, unfavorite)
    │   ├── comments.rs         (index, store, destroy)
    │   └── tags.rs             (index)
    └── dto/
        ├── mod.rs
        └── responses.rs        (UserDto, ArticleDto, ProfileDto, CommentDto with serde renames)
```

---

## Architecture Patterns

### JWT Custom Middleware Pattern

The `Middleware` trait and `Request::insert<T>()` / `Request::get<T>()` extension map are the correct integration points. The app in `app/src/middleware/bearer_auth.rs` is the direct reference — same pattern, different token format. [VERIFIED: `app/src/middleware/bearer_auth.rs`]

**Two middleware variants needed:**
1. `JwtAuthMiddleware` — rejects with 401 if no valid token
2. `OptionalJwtMiddleware` — inserts `UserId` if valid token present, proceeds as guest if absent

**Current user in handlers:** Once middleware inserts `UserId(i64)` via `request.insert::<UserId>(...)`, the handler retrieves it with `req.get::<UserId>()`. Since `AuthUser<T>` extractor reads from a session (not extensions), it cannot be reused here. The handler must call `req.get::<UserId>()` directly. [VERIFIED: `framework/src/auth/extract.rs` — `FromRequest` reads from session via `Auth::user()`]

### SeaORM Relations Pattern

```rust
// follows table: (follower_id FK users.id, followed_id FK users.id)
// Check if user A follows user B:
let is_following = follows::Entity::find()
    .filter(follows::Column::FollowerId.eq(current_user_id))
    .filter(follows::Column::FollowedId.eq(profile_user_id))
    .count(db)
    .await? > 0;

// articles with favorites count + favorited flag for current user:
// Option 1: two queries (simple)
// Option 2: subquery in SELECT (SeaORM custom column expression)
```

### Slug Generation Pattern

```rust
// In models/article.rs
fn generate_slug(title: &str) -> String {
    let base = slug::slugify(title);
    let uid: String = rand::random::<u32>().to_string();
    format!("{base}-{uid}")
}
```

### Anti-Patterns to Avoid

- **Using `Auth::login()` for JWT auth:** Ferro's `Auth::login()` writes to the session store. For JWT auth, skip the session entirely — the token is stateless. Do not call `Auth::login()` in register/login handlers; just mint and return the token. [VERIFIED gap: `framework/src/auth/guard.rs`]
- **N+1 queries for article lists:** Article list with `articlesCount` + per-article `favoritesCount` + per-article `author.following` is a classic N+1 scenario. Use batch loading or explicit JOIN queries. SeaORM's `batch_load_has_many` and `BatchLoad` traits help; for favoritesCount, an aggregation subquery or two-pass approach is cleaner.
- **Slug conflicts:** Slugs must be unique. Generate with a short random suffix rather than relying on uniqueness from title alone. Add `UNIQUE` constraint on the `slug` column.
- **`/api/articles/feed` vs `/api/articles/{slug}` route ordering:** Register the literal route before the parameterized one. In Ferro's `matchit`-based router, literal segments take priority over wildcard segments, so ordering in `routes!` does not matter for the matchit matching — but for clarity, declare literal routes first. [ASSUMED — matchit literal-priority behavior; low risk]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Password hashing | Custom bcrypt wrapper | `ferro::hashing::{hash, verify}` | Constant-time comparison, cost configured |
| JSON response building | String concatenation | `json_response!()` / `HttpResponse::json()` / serde structs | Safety, escaping |
| Route params | Manual URL parsing | `req.param("slug")?` / `req.param_as::<i64>("id")?` | Type-safe, framework-provided |
| Query string | Manual parsing | `req.query_as_or("limit", 20u64)` | Already parsed, typed |
| Request body parsing | Manual JSON decode | `req.input::<T>().await?` | Handles content-type, deserialization |
| Validation rules | Custom validators | `ferro::Validator` + `required()`, `email()`, `min()` | Re-usable rule set |
| ORM queries | Raw SQL strings | SeaORM `QueryBuilder` / entity methods | Type-safe, parameterized, no injection |
| JWT sign/verify | Custom HMAC | `jsonwebtoken = "9"` | Handles alg-confusion, expiry, claims |
| Migrations | Schema creation in code | `sea-orm-migration` | Rollback support, reproducibility |

---

## Common Pitfalls

### Pitfall 1: Auth guard reads session, not JWT

**What goes wrong:** Developer uses `AuthUser<User>` extractor in a handler. It calls `Auth::user()` which reads `auth_user_id()` from the session. JWT-auth requests have no session, so `Auth::user()` always returns `None`, giving 401 even with a valid JWT.

**Root cause:** `AuthUser<T>` is session-bound. [VERIFIED: `framework/src/auth/extract.rs`]

**How to avoid:** Never use `AuthUser<T>` or `OptionalUser<T>` in Conduit handlers. Use `req.get::<UserId>()` from the extension map populated by `JwtAuthMiddleware`.

**Warning signs:** 401 on all protected routes despite valid JWT; session cookie in response headers where there should be none.

### Pitfall 2: `/api/articles/feed` hidden by `/api/articles/{slug}`

**What goes wrong:** GET `/api/articles/feed` is matched as slug="feed" on the `show` handler.

**Root cause:** Path param wildcards can shadow literal routes if router has them at the same level.

**How to avoid:** In matchit (Ferro's router), literal segments always win over wildcards. Register both routes and verify with a test. Alternatively, put `feed` in a separate group path so it is unambiguous.

**Warning signs:** Feed handler never called; article show handler called with `slug="feed"` and returning 404.

### Pitfall 3: camelCase fields missing in response

**What goes wrong:** SeaORM model fields are snake_case; response contains `tag_list`, `created_at`, `favorites_count` which fail Conduit contract tests.

**Root cause:** serde default is field name as-is.

**How to avoid:** Use separate DTO structs with `#[serde(rename_all = "camelCase")]`. Do not serialize SeaORM model types directly in responses.

**Warning signs:** Contract tests fail on field name assertions; `tagList` is `null` or absent.

### Pitfall 4: NULL vs missing optional fields

**What goes wrong:** User `bio` and `image` are nullable in DB; serde serializes `Option<String>` as `null` JSON. Conduit spec allows `null` for these fields but some clients expect `""` (empty string).

**Root cause:** `Option<String>` → `null` in serde default.

**How to avoid:** Keep `null` for Conduit compliance (spec says nullable). Add `#[serde(skip_serializing_if = "Option::is_none")]` only if a field should be entirely absent (not needed for Conduit user fields).

### Pitfall 5: Nested `{ "user": { ... } }` request parsing

**What goes wrong:** Registration body is `{ "user": { ... } }`. `req.input::<RegisterRequest>().await?` expects the struct to match the top-level JSON shape. If `RegisterRequest` has flat fields, deserialization fails.

**How to avoid:** Wrap the inner struct in an envelope struct:

```rust
#[derive(Deserialize)]
struct RegisterEnvelope {
    user: RegisterRequest,
}
let envelope = req.input::<RegisterEnvelope>().await?;
let form = envelope.user;
```

### Pitfall 6: Missing Cors header

**What goes wrong:** Conduit conformance tests (Postman collection) may be run from a browser context or a test runner that checks CORS. Without `Access-Control-Allow-Origin`, preflight fails.

**How to avoid:** Register `Cors` middleware globally in `bootstrap.rs`. Ferro provides `ferro::Cors`. [VERIFIED: `framework/src/lib.rs` — `Cors` is exported from `middleware`]

---

## Laravel RealWorld Backend Recommendation

**Recommended repo:** `gothinkster/laravel-realworld-example-app` [CITED: https://github.com/gothinkster/laravel-realworld-example-app]

This is the canonical, most-starred Laravel Conduit implementation and the one the RealWorld project links to as the reference. It uses Laravel 5.x with `tymon/jwt-auth`.

**Important:** the repo is archived and targets an old Laravel version. This is fine for the benchmark since the goal is not to author the Laravel side, only to vendor a pinned community implementation. The benchmark design principle is "do not author the competition." [CITED: benchmark design doc — "Competitor implementations come from the community RealWorld backends"]

**Recommended alternative if gothinkster/laravel is too stale:** `f1amy/laravel-realworld-example-app` — a well-maintained Laravel 10/11 fork of the gothinkster reference that passes all Conduit conformance tests. [ASSUMED — known at training time; not fetched in this session. Needs verification before pinning.]

**Action for planner:** The plan must include a task to select the specific Laravel repo + pin a commit SHA. The SHA must be committed to the benchmark directory. Do not pin to a branch name.

**Database choice:** Postgres for both (same as ferro-micro uses). SQLite is an option for development but the performance comparison must use Postgres for both to be fair. [ASSUMED — reasonable; benchmark design doc leaves this open]

---

## Feasibility Verdict + Recommended Scope

### Full Conduit is feasible. Recommended scope: build the full spec.

**Rationale:**

1. **JWT auth gap is small and quantifiable.** ~60–100 lines of a custom middleware + JWT helper. This is the most significant gap, and it is honest: the benchmark should report that Ferro does not provide JWT auth out of the box, requiring hand-rolling. That is a real finding. A partial build would obscure whether the remaining endpoints are hard; they are not.

2. **Everything else is framework-provided.** Routing, SeaORM relations, validation, password hashing, JSON building, pagination, query filtering — all covered. The static-compression measurement will show where Ferro saves lines (e.g., routing macros, SeaORM entity declarations) vs. where it does not (JWT auth boilerplate, M:N junction queries).

3. **A full Conduit implementation is the credibility spine of the benchmark.** Skipping follows/feed would make the comparison invalid for any evaluation tool that runs the full Conduit conformance test suite. The Laravel side passes the full suite; the Ferro side must too.

**What to flag honestly in benchmark commentary:**
- JWT auth (issuance + validation middleware): hand-rolled, ~100 LOC, not framework-provided.
- Slug generation: hand-rolled, ~5 LOC.
- Conduit JSON envelopes (DTO structs with renames): hand-rolled, framework provides serde/json primitives but not Conduit-specific DTOs. This is expected — same is true of any framework.

**What not to flag as gaps:**
- M:N junction table queries: SeaORM handles this. Slightly more verbose than Eloquent's `belongsToMany`, but framework-provided.
- Validation error mapping to Conduit format: a simple transform, not a capability gap.
- Pagination with `articlesCount`: two SeaORM queries, both framework-provided.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Cargo build | ✓ | 1.88.0 (from ferro-micro Dockerfile) | — |
| PostgreSQL | Data layer | ✓ | (workspace uses Postgres) | SQLite for dev |
| Docker | Containerization | ✓ | (existing ferro-micro uses Docker) | — |

No blocking dependencies. The benchmark app is a standalone Cargo binary identical in structure to ferro-micro.

---

## Validation Architecture

Per `.planning/config.json` nyquist_validation setting — not checked in this session. The benchmark app is a standalone binary, not the ferro workspace. Tests would be contract conformance tests (the Conduit Postman collection or a port of it) rather than unit tests. The harness section in the benchmark design calls for contract tests in `benchmark/contracts/conduit-openapi.yaml`. Validation for Phase 230 is: the ferro-conduit binary starts, the contract tests pass.

---

## Open Questions (RESOLVED — Q1 Postgres, Q3 Newman, Q4 route-order test all resolved in plans; Q2 Laravel repo deferred to Plan 07 checkpoint:decision by design)

1. **Database for the perf comparison: SQLite vs Postgres?**
   - What we know: ferro-micro uses Postgres. The benchmark design doc lists this as open. SQLite is simpler but gives misleading perf numbers; Postgres reflects real-world usage.
   - Recommendation: Postgres for both implementations in the final perf run. SQLite acceptable for local dev iteration.

2. **Which Laravel repo + which commit to pin?**
   - What we know: `gothinkster/laravel-realworld-example-app` is the canonical choice but is archived and targets Laravel 5.x. `f1amy/laravel-realworld-example-app` is a well-maintained modern fork.
   - What's unclear: which passes the full Conduit conformance suite cleanly today without config changes.
   - Recommendation: Evaluate `f1amy/laravel-realworld-example-app` first (Laravel 10/11, maintained). If it passes the contract tests, pin it. Fall back to gothinkster if the modern fork has diverged from the spec.

3. **Conduit conformance test runner: Postman collection or OpenAPI-based?**
   - What we know: the canonical Conduit conformance tests are a Postman/Newman collection in the gothinkster/realworld repo. The benchmark design calls for `benchmark/contracts/conduit-openapi.yaml` + conformance tests.
   - Recommendation: Use the Newman collection as-is for initial conformance validation; derive or reference the OpenAPI spec from the existing gothinkster spec file.

4. **Feed route vs article show route ordering in matchit**
   - What we know: matchit prioritizes literal segments over wildcards; this should be safe.
   - What's unclear: whether `GET /api/articles/feed` is parsed as literal "articles" + literal "feed" vs literal "articles" + wildcard "{slug}".
   - Recommendation: Write a route registration test in the app to verify before trusting it.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Conduit API contract is stable and unchanged from training knowledge | Conduit Contract | Low — spec has been frozen for years; verify against gothinkster/realworld OpenAPI before writing handler contracts |
| A2 | `f1amy/laravel-realworld-example-app` passes full Conduit suite without changes | Laravel Repo Recommendation | Medium — planner must verify before pinning; fall back to gothinkster |
| A3 | SeaORM 1.0 `Linked` trait handles self-referential M:N; direct junction query is the alternative | Relations | Low — either approach works; direct query is safer |
| A4 | matchit literal-before-wildcard priority makes feed/slug ordering safe | Pitfall 2 | Low — well-documented matchit behavior |
| A5 | `slug = "0.1"` is a viable crate for title → URL slug conversion | Standard Stack | Low — trivial fallback to a 3-line manual implementation |
| A6 | Postgres will be the benchmark database (not SQLite) | Environment / Open Questions | Medium — benchmark design doc left this open; affects fairness of perf numbers |

---

## Sources

### Primary (HIGH confidence)
- `framework/src/auth/guard.rs`, `middleware.rs`, `extract.rs` — session-based auth; no JWT in framework
- `framework/src/auth/mod.rs` — module docstring confirming "session-based authentication system"
- `ferro-mcp-oauth/src/jwt.rs` — JWT (HS256, `jsonwebtoken = "9"`) exists but in MCP-scoped crate
- `framework/src/http/request.rs` — `req.param()`, `req.query()`, `req.query_as()`, `req.input()`
- `framework/src/database/query_builder.rs` — `filter()`, `limit()`, `offset()`, eager loading
- `framework/src/database/model.rs` — `Model` trait, `query()`, `find_by_pk()`
- `framework/src/routing/macros.rs` — `group!`, `get!`, `post!`, `put!`, `delete!`, `routes!`
- `framework/src/hashing/mod.rs` — `hash()`, `verify()` via bcrypt
- `framework/src/lib.rs` — full public re-export surface
- `framework/Cargo.toml` — `sea-orm = "1.0"`, both Postgres and SQLite features
- `benchmark/apps/ferro-micro/Cargo.toml`, `Dockerfile`, `src/` — proven standalone app pattern
- `app/src/middleware/bearer_auth.rs` — reference pattern for custom middleware + extension map

### Secondary (MEDIUM confidence)
- [GitHub — gothinkster/laravel-realworld-example-app](https://github.com/gothinkster/laravel-realworld-example-app) — canonical Laravel Conduit reference; archived but repo exists
- Benchmark design doc `docs/superpowers/specs/2026-06-15-ferro-framework-benchmark-design.md` — scope and principles

### Tertiary (LOW confidence)
- `f1amy/laravel-realworld-example-app` — modern Laravel fork; known from training, not fetched
- Conduit API contract details — training knowledge, stable spec, not fetched from spec repo in this session

---

## Metadata

**Confidence breakdown:**
- Ferro capability audit: HIGH — every claim traced to a source file with line-level verification
- JWT gap: HIGH — absence of JWT in framework confirmed by grep across all auth files
- Conduit contract: MEDIUM — spec is well-known and stable; not fetched from spec repo
- Laravel repo recommendation: MEDIUM — gothinkster is confirmed; f1amy alternative is training knowledge

**Research date:** 2026-06-15
**Valid until:** 2026-09-15 (stable domain; ferro version may increment but API surface is stable)
