# Phase 199: OAuth Browser Login - Pattern Map

**Mapped:** 2026-06-10
**Files analyzed:** 17
**Analogs found:** 17 / 17

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-mcp-oauth/Cargo.toml` | config | — | `ferro-mcp-server/Cargo.toml` | exact |
| `ferro-mcp-oauth/src/lib.rs` | config | — | `ferro-mcp-server/src/lib.rs` | exact |
| `ferro-mcp-oauth/src/config.rs` | config | request-response | `ferro-mcp-server/src/config.rs` | exact |
| `ferro-mcp-oauth/src/discovery.rs` | controller | request-response | `ferro-mcp-server/src/config.rs` (JSON from config) | role-match |
| `ferro-mcp-oauth/src/register.rs` | controller | CRUD | `app/src/controllers/auth_controller.rs` (register) | role-match |
| `ferro-mcp-oauth/src/authorize.rs` | controller | request-response | `framework/src/auth/guard.rs` + `framework/src/session/middleware.rs` | role-match |
| `ferro-mcp-oauth/src/consent.rs` | controller | request-response | `framework/src/session/middleware.rs` (CSRF) + `framework/src/http/response.rs` | role-match |
| `ferro-mcp-oauth/src/token.rs` | controller | request-response | `framework/src/cache/mod.rs` (get/forget) | role-match |
| `ferro-mcp-oauth/src/validate.rs` | service | request-response | `framework/src/api/api_key.rs` (bearer extract + constant-time) | role-match |
| `ferro-mcp-oauth/src/jwt.rs` | service | transform | `ferro-wallet/src/google/jwt.rs` (jsonwebtoken encode pattern) | role-match |
| `ferro-mcp-oauth/src/pkce.rs` | utility | transform | `framework/src/api/api_key.rs` (sha2 + subtle::ConstantTimeEq) | role-match |
| `ferro-mcp-oauth/src/store.rs` | model | CRUD | `framework/src/cache/mod.rs` + migration pattern | role-match |
| `ferro-mcp-oauth/src/migration.rs` | migration | CRUD | `ferro-audit/src/migration.rs` (crate-shipped migration) | exact |
| `ferro-mcp-oauth/tests/flow_integration.rs` | test | request-response | `ferro-mcp-server/tests/dispatch_integration.rs` | role-match |
| `ferro-mcp-server/src/auth.rs` | service | request-response | current file (reduction: delete body, keep enum) | exact |
| `app/src/controllers/mcp.rs` | controller | request-response | current file (seam replacement) | exact |
| `app/src/controllers/auth_controller.rs` | controller | request-response | current file (add session return-to) | exact |
| `app/src/routes.rs` | config | — | current file | exact |
| `app/src/migrations/m20260611_create_oauth_clients_table.rs` | migration | CRUD | `app/src/migrations/m20260228_create_api_keys_table.rs` | exact |
| `app/src/migrations/mod.rs` | config | — | current file | exact |
| `app/Cargo.toml` | config | — | current file | exact |
| `Cargo.toml` (workspace) | config | — | current file | exact |
| `.github/workflows/publish.yml` | config | — | current file (WAVE2_CRATES line) | exact |

---

## Pattern Assignments

### `ferro-mcp-oauth/Cargo.toml` (config)

**Analog:** `ferro-mcp-server/Cargo.toml`

**Crate header pattern** (`ferro-mcp-server/Cargo.toml` lines 1-11):
```toml
[package]
name = "ferro-mcp-server"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "MCP tool rendering target for Ferro projections"
repository = "https://github.com/albertogferrario/ferro"
homepage = "https://ferro-rs.dev"
readme = "README.md"
keywords = ["mcp", "ferro", "projections", "server-driven"]
categories = ["web-programming", "web-programming::http-server"]
```

Apply for `ferro-mcp-oauth` with adjusted description/keywords. Dependencies from RESEARCH.md §1:
```toml
[dependencies]
ferro-rs = { path = "../framework", version = "0.2" }
jsonwebtoken = "9"
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }
sea-orm-migration = "1.0"
rand = "0.8"
base64 = "0.22"
sha2 = "0.10"
subtle = "2.5"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
async-trait = "0.1"
```

Note: `ferro-mcp-server` is NOT a dependency. All versions match workspace usage.

---

### `ferro-mcp-oauth/src/lib.rs` (pub re-exports)

**Analog:** `ferro-mcp-server/src/lib.rs`

**Module declaration + pub-use pattern** (`ferro-mcp-server/src/lib.rs` lines 1-19):
```rust
pub mod auth;
pub mod config;
// ... other modules

pub use auth::{extract_bearer, BearerOutcome};
pub use config::McpServerConfig;
```

For `ferro-mcp-oauth`, the lib.rs declares all modules and re-exports the public surface:
```rust
pub mod config;
pub mod discovery;
pub mod register;
pub mod authorize;
pub mod consent;
pub mod token;
pub mod validate;
pub mod pkce;
pub mod jwt;
pub mod store;
pub mod migration;

pub use config::{OAuthConfig, OAuthConfigError};
pub use validate::validate_bearer;
pub use migration::Migration as CreateOauthClientsTable;
// pub use handlers module (discovery, register, authorize_get, authorize_post, token_exchange)
```

---

### `ferro-mcp-oauth/src/config.rs` (config, request-response)

**Analog:** `ferro-mcp-server/src/config.rs` (exact role match)

**Config struct + sanitize_identity + from_env pattern** (`ferro-mcp-server/src/config.rs` lines 9-49):
```rust
fn sanitize_identity(raw: String) -> String {
    raw.chars().filter(|c| !c.is_ascii_control()).collect()
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            app_name: sanitize_identity(
                std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string()),
            ),
            app_url: sanitize_identity(
                std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string()),
            ),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl McpServerConfig {
    pub fn from_env() -> Self {
        Self::default()
    }
}
```

For `OAuthConfig`, mirror this exactly but add `MCP_TOKEN_SECRET` with fail-closed behavior:
```rust
pub struct OAuthConfig {
    pub app_name: String,
    pub app_url: String,
    pub token_secret: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthConfigError {
    #[error("MCP_TOKEN_SECRET env var not set")]
    MissingSecret,
    #[error("MCP_TOKEN_SECRET must be at least 32 bytes")]
    SecretTooShort,
}

impl OAuthConfig {
    pub fn from_env() -> Result<Self, OAuthConfigError> {
        let app_name = sanitize_identity(
            std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string()),
        );
        let app_url = sanitize_identity(
            std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string()),
        );
        let secret_str = std::env::var("MCP_TOKEN_SECRET")
            .map_err(|_| OAuthConfigError::MissingSecret)?;
        if secret_str.len() < 32 {
            return Err(OAuthConfigError::SecretTooShort);
        }
        Ok(Self { app_name, app_url, token_secret: secret_str.into_bytes() })
    }
}
```

**Test pattern** (`ferro-mcp-server/src/config.rs` lines 52-71): copy the `sanitize_strips_crlf_and_control_chars` test verbatim; add tests for `MissingSecret` and `SecretTooShort`.

---

### `ferro-mcp-oauth/src/discovery.rs` (controller, request-response)

**Analog:** `ferro-mcp-server/src/config.rs` + `app/src/controllers/mcp.rs` (JSON from config)

**Handler macro + JSON response pattern** (`app/src/controllers/mcp.rs` lines 39-51):
```rust
#[handler]
pub async fn handle(req: Request) -> Response {
    let config = McpServerConfig::from_env();
    // ... build JSON from config fields
    Ok(HttpResponse::json(json!({ ... })))
}
```

Both discovery handlers call `OAuthConfig::from_env()` (infallible for `app_url`/`app_name`) and return static JSON constructed from config. No DB, no auth check, no body parsing. The `#[handler]` macro is the only annotation needed.

---

### `ferro-mcp-oauth/src/register.rs` (controller, CRUD)

**Analog:** `app/src/controllers/auth_controller.rs::register` (lines 33-90)

**Imports pattern** (`app/src/controllers/auth_controller.rs` lines 1-12):
```rust
use ferro::serde_json::json;
use ferro::{handler, HttpResponse, Request, Response};
use serde::Deserialize;
```

**JSON input + validation + DB insert + JSON 201 response pattern** (`app/src/controllers/auth_controller.rs` lines 33-90):
```rust
#[handler]
pub async fn register(req: Request) -> Response {
    let input: RegisterInput = req.json().await?;

    // Validate
    if let Err(errors) = Validator::new(&data)
        .rules("redirect_uris", ferro::rules![required()])
        .validate()
    {
        return Err(HttpResponse::json(errors.to_json()).status(422));
    }

    // Insert (SeaORM pattern from auth_controller.rs line 70-76)
    let new_client = oauth_clients::ActiveModel {
        client_id: Set(uuid::Uuid::new_v4().to_string()),
        ...
        ..Default::default()
    };
    let client = oauth_clients::Entity::insert_one(new_client).await?;

    // Return 201
    Ok(HttpResponse::json(json!({ "client_id": client.client_id, ... })).status(201))
}
```

Error for invalid `redirect_uri` scheme: `400 {"error":"invalid_client_metadata","error_description":"..."}` — use `HttpResponse::json(json!({...})).status(400)`.

---

### `ferro-mcp-oauth/src/authorize.rs` (controller, request-response)

**Analog:** `framework/src/auth/guard.rs` (Auth::check/Auth::id) + `framework/src/session/middleware.rs` (session_mut)

**Auth::check pattern** (`framework/src/auth/guard.rs` lines 62-63):
```rust
pub fn check() -> bool {
    Self::id().is_some()
}
```

**Auth::id pattern** (`framework/src/auth/guard.rs` lines 40-43):
```rust
pub fn id() -> Option<i64> {
    auth_user_id()
}
```

**session_mut for storing OAuth params** (`framework/src/session/middleware.rs` lines 56-69):
```rust
session_mut(|session| {
    session.put("oauth_return_to", "/authorize?client_id=...&...");
});
```

**Redirect pattern** (`framework/src/http/response.rs` — `status` + `header`):
```rust
// Redirect to login if not authenticated
if !Auth::check() {
    session_mut(|s| { s.put("oauth_return_to", full_authorize_url); });
    return Err(HttpResponse::new()
        .status(302)
        .header("Location", "/auth/login"));
}
// After validation, redirect to consent (render it directly or redirect)
let user_id = Auth::id().expect("auth check passed");
let tenant_id: Option<i64> = current_tenant().map(|t| t.id);
```

**current_tenant pattern** (`framework/src/tenant/context.rs` lines 32-37):
```rust
pub fn current_tenant() -> Option<TenantContext> {
    TENANT_CONTEXT
        .try_with(|ctx| ctx.try_read().ok().and_then(|guard| guard.clone()))
        .ok()
        .flatten()
}
```

---

### `ferro-mcp-oauth/src/consent.rs` (controller, request-response)

**Analog:** `framework/src/session/middleware.rs` (get_csrf_token) + `framework/src/http/response.rs` (text + header override)

**get_csrf_token pattern** (`framework/src/session/middleware.rs` lines 221-224):
```rust
pub fn get_csrf_token() -> Option<String> {
    session().map(|s| s.csrf_token)
}
```

**HTML response pattern** (`framework/src/http/response.rs` lines 28-35 + line 122-127):
```rust
// HttpResponse::text() sets Content-Type: text/plain — must override:
Ok(HttpResponse::text(html_string)
    .header("Content-Type", "text/html; charset=utf-8"))
```

Note: `HttpResponse::text()` is the only constructor for HTML responses; `.header()` replaces the Content-Type (case-insensitive, see `framework/src/http/response.rs` line 124).

**CSRF constant-time validation** (`framework/src/api/api_key.rs` lines 145-151):
```rust
use subtle::ConstantTimeEq;

// Validate CSRF in POST /authorize:
let submitted = form._token.as_bytes();
let session_csrf = get_csrf_token().ok_or_else(|| csrf_error())?;
if !submitted.ct_eq(session_csrf.as_bytes()).into() {
    return Err(HttpResponse::new().status(400));
}
```

**Cache::put for OAuthCode** (`framework/src/cache/mod.rs` lines 145-154):
```rust
Cache::put(
    &format!("mcp:code:{code}"),
    &oauth_code,   // must implement Serialize
    Some(Duration::from_secs(60)),
).await?;
```

---

### `ferro-mcp-oauth/src/token.rs` (controller, request-response)

**Analog:** `framework/src/cache/mod.rs` (get/forget single-use pattern)

**Cache::get + Cache::forget single-use pattern** (`framework/src/cache/mod.rs` lines 123-134 and 190-193):
```rust
// Retrieve (returns Option<T> where T: DeserializeOwned)
let record: Option<OAuthCode> = Cache::get(&format!("mcp:code:{code}")).await?;
// Single-use: forget BEFORE validation so replay is impossible even on failure
Cache::forget(&format!("mcp:code:{code}")).await.ok();
let record = record.ok_or_else(|| {
    HttpResponse::json(json!({"error":"invalid_grant"})).status(400)
})?;
```

**form body parsing** — from RESEARCH.md §6: `req.form::<TokenRequest>()` (`framework/src/http/request.rs` line 485). The input type must derive `Deserialize`.

**Error response shape** (RFC 6749 §5.2, same JSON shape throughout):
```rust
Err(HttpResponse::json(json!({
    "error": "invalid_grant",
    "error_description": "authorization code expired or already used"
})).status(400))
```

---

### `ferro-mcp-oauth/src/validate.rs` (service, request-response)

**Analog:** `framework/src/api/api_key.rs` (bearer extraction + constant-time compare pattern)

**Bearer extraction from header** (`framework/src/api/api_key.rs` lines 199-223):
```rust
fn extract_bearer_token(request: &Request) -> Result<&str, HttpResponse> {
    let header = request.header("Authorization").ok_or_else(|| { ... 401 })?;
    let token = header.strip_prefix("Bearer ").ok_or_else(|| { ... 401 })?;
    if token.is_empty() { return Err(... 401); }
    Ok(token)
}
```

For `validate_bearer`, the input is `Option<&str>` (header already extracted before body-consume in mcp.rs), not a `&Request`:
```rust
pub fn validate_bearer(authorization_header: Option<&str>, config: &OAuthConfig) -> BearerOutcome {
    let header = match authorization_header {
        None => return BearerOutcome::Unauthenticated,
        Some(h) => h,
    };
    let token = match header.strip_prefix("Bearer ") {
        None => return BearerOutcome::Unauthenticated,
        Some(t) => t,
    };
    match decode_token(token, &config.token_secret, &format!("{}/mcp", config.app_url)) {
        Ok(claims) => BearerOutcome::Authenticated(json!({
            "sub": claims.sub,
            "tenant_id": claims.tenant_id,
        })),
        Err(e) => e.into_bearer_outcome(),
    }
}
```

`BearerOutcome` stays in `ferro-mcp-server`; import it from there.

**Error → HTTP status mapping** (D-07, RESEARCH.md §3): `TokenError::expired` → 401 `WWW-Authenticate: Bearer error="invalid_token"`; `TokenError::invalid_audience` → 403; tenant mismatch → 403.

---

### `ferro-mcp-oauth/src/jwt.rs` (service, transform)

**Analog:** `ferro-wallet/src/google/jwt.rs` (jsonwebtoken v9 encode pattern, lines 8-65)

**Imports + encode pattern** (`ferro-wallet/src/google/jwt.rs` lines 8-64):
```rust
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

fn sign_save_jwt(...) -> Result<String, WalletError> {
    let header = Header::new(Algorithm::RS256);   // use HS256 here instead
    let key = EncodingKey::from_rsa_pem(...)      // use EncodingKey::from_secret(&secret)
        .map_err(|e| WalletError::GoogleJwt(format!("private key parse: {e}")))?;
    encode(&header, &claims, &key)
        .map_err(|e| WalletError::GoogleJwt(format!("encode: {e}")))
}
```

The wallet analog is RS256/encode-only. For HS256 mint + decode, use:
```rust
// Mint (HS256)
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
let header = Header::new(Algorithm::HS256);
let key = EncodingKey::from_secret(secret);
encode(&header, claims, &key).map_err(Error::Jwt)

// Decode (HS256) — no analog in codebase; use RESEARCH.md §3 directly:
use jsonwebtoken::{decode, DecodingKey, Validation};
let mut validation = Validation::new(Algorithm::HS256);
validation.algorithms = vec![Algorithm::HS256];      // pin algorithm — T-06
validation.set_audience(&[expected_aud]);
validation.validate_exp = true;
validation.leeway = 0;
let key = DecodingKey::from_secret(secret);
let data = decode::<McpTokenClaims>(token, &key, &validation)?;
Ok(data.claims)
```

**Claims struct** (RESEARCH.md §3, claim name `tenant_id` is load-bearing):
```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct McpTokenClaims {
    sub: String,
    tenant_id: Option<i64>,   // EXACT name — matches JwtClaimResolver field (resolver.rs line 211)
    aud: Vec<String>,
    iss: String,
    iat: i64,
    exp: i64,
}
```

`tenant_id` verified from `framework/src/tenant/resolver.rs` line 211: `claims[&self.claim_field].as_i64()` where `claim_field = "tenant_id"`.

---

### `ferro-mcp-oauth/src/pkce.rs` (utility, transform)

**Analog:** `framework/src/api/api_key.rs` (sha2 + subtle::ConstantTimeEq pattern, lines 38-151)

**sha2 + subtle imports** (`framework/src/api/api_key.rs` lines 38-40):
```rust
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
```

**Hash + constant-time compare pattern** (`framework/src/api/api_key.rs` lines 136-151):
```rust
pub fn hash_api_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn verify_api_key_hash(raw_key: &str, stored_hash: &str) -> bool {
    let incoming_hash = hash_api_key(raw_key);
    incoming_hash.as_bytes().ct_eq(stored_hash.as_bytes()).into()
}
```

For PKCE S256, the output is base64url not hex — see RESEARCH.md §4:
```rust
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

pub fn verify_s256(code_verifier: &str, stored_challenge: &str) -> bool {
    let hash = Sha256::digest(code_verifier.as_bytes());
    let recomputed = URL_SAFE_NO_PAD.encode(hash);
    recomputed.as_bytes().ct_eq(stored_challenge.as_bytes()).into()
}
```

**Code generation** (`framework/src/api/api_key.rs` lines 114-133 as pattern for rand usage):
```rust
use rand::Rng;
let bytes: [u8; 32] = rand::thread_rng().gen();
URL_SAFE_NO_PAD.encode(bytes)  // 43 URL-safe base64 chars
```

---

### `ferro-mcp-oauth/src/store.rs` (model, CRUD)

**Analog:** `framework/src/cache/mod.rs` for cache structs; `app/src/migrations/m20260228_create_api_keys_table.rs` for the DB model shape.

**Cached code struct** (uses `serde::Serialize + Deserialize` for `Cache::put`/`Cache::get`):
```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OAuthCode {
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub user_id: i64,
    pub tenant_id: Option<i64>,
    pub created_at: i64,
}
```

**SeaORM model for oauth_clients** — no existing SeaORM model analog in the crate itself (the DB entity lives in `app/src/models/`); follow the existing model structure in `app/src/models/users.rs` if a SeaORM entity is needed within the crate for DCR queries.

---

### `ferro-mcp-oauth/src/migration.rs` (migration, CRUD)

**Analog:** `ferro-audit/src/migration.rs` (crate-ships-its-own-migration pattern, lines 1-101)

**Crate-shipped migration pattern** (`ferro-audit/src/migration.rs` lines 18-84):
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
                    .table(OauthClients::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(OauthClients::Id)
                        .big_integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(OauthClients::ClientId).string().not_null())
                    .col(ColumnDef::new(OauthClients::ClientName).string().null())
                    .col(ColumnDef::new(OauthClients::RedirectUris).text().not_null())
                    .col(ColumnDef::new(OauthClients::CreatedAt)
                        .timestamp_with_time_zone().not_null()
                        .default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(Index::create()
                .name("idx_oauth_clients_client_id")
                .table(OauthClients::Table)
                .col(OauthClients::ClientId)
                .unique()
                .to_owned())
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(OauthClients::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum OauthClients {
    Table, Id, ClientId, ClientName, RedirectUris, CreatedAt,
}
```

Re-exported in lib.rs as `pub use migration::Migration as CreateOauthClientsTable` (mirrors `ferro-audit/src/lib.rs` line 67: `pub use migration::Migration as CreateAuditLogTable`).

The test at `ferro-audit/src/migration.rs` lines 103-176 (in-memory SQLite migration round-trip) is the exact test pattern to copy.

---

### `ferro-mcp-oauth/tests/flow_integration.rs` (test, request-response)

**Analog:** `ferro-mcp-server/tests/dispatch_integration.rs` (in-process handler invocation pattern per RESEARCH.md §9)

No need to read the full test file; the RESEARCH.md §9 describes the pattern:
```rust
#[tokio::test]
async fn full_pkce_flow() {
    // 1. in-memory SQLite + in-memory Cache bootstrap
    // 2. call handler functions directly (not via HTTP server)
    // 3. assert on Response values
}
```

Unit tests for `pkce.rs`, `jwt.rs`, and `validate.rs` are synchronous `#[test]` — no tokio needed for pure functions.

---

### `ferro-mcp-server/src/auth.rs` (reduction — keep enum, delete function)

**Current file:** `ferro-mcp-server/src/auth.rs` lines 1-44

**Action:** Delete `extract_bearer` function (lines 20-23) and its two `#[test]` bodies (lines 30-43). Keep only `BearerOutcome` enum (lines 8-14) and the module-level doc comment.

**After deletion, the file contains only:**
```rust
//! Bearer-token extraction seam for the MCP endpoint.

/// Outcome of resolving a request's bearer credential.
pub enum BearerOutcome {
    /// No `Authorization` header, or token not validated.
    Unauthenticated,
    /// Token validated; principal attached.
    Authenticated(serde_json::Value),
}
```

`ferro-mcp-server/src/lib.rs` line 14 (`pub use auth::{extract_bearer, BearerOutcome}`) must be updated to remove `extract_bearer` from the re-export.

---

### `app/src/controllers/mcp.rs` (seam replacement)

**Current file:** `app/src/controllers/mcp.rs` lines 1-127

**Import change** (lines 9-11 → replace `extract_bearer` import):
```rust
// Before:
use ferro_mcp_server::{
    extract_bearer, handle_initialize, handle_tools_call, handle_tools_list, BearerOutcome,
    McpServerConfig,
};

// After:
use ferro_mcp_server::{BearerOutcome, McpServerConfig, handle_initialize, handle_tools_call, handle_tools_list};
use ferro_mcp_oauth::{validate_bearer, OAuthConfig};
```

**Handler body change** (lines 40-50):
```rust
#[handler]
pub async fn handle(req: Request) -> Response {
    let config = McpServerConfig::from_env();

    // Origin validation (Phase 198 TODO, now required)
    if let Some(origin) = req.header("Origin") {
        if !origin.starts_with(config.app_url.as_str()) {
            return Err(HttpResponse::new().status(403));
        }
    }

    // Read Authorization header BEFORE consuming body
    let authorization = req.header("Authorization").map(|s| s.to_owned());

    // Phase 199: real bearer validation via ferro-mcp-oauth
    let oauth_config = OAuthConfig::from_env()
        .map_err(|_| challenge_response(&config))?;

    match validate_bearer(authorization.as_deref(), &oauth_config) {
        BearerOutcome::Unauthenticated => return Err(challenge_response(&config)),
        BearerOutcome::Authenticated(_principal) => { /* Phase 200 inserts into extensions */ }
    }

    // ... rest unchanged (lines 53-83)
}
```

**Test to delete:** `bearer_seam_always_challenges` (lines 116-126) — it tests the stub behavior that no longer exists.

---

### `app/src/controllers/auth_controller.rs` (return-to addition)

**Current file:** `app/src/controllers/auth_controller.rs` lines 96-154

**Addition to `login` handler after `Auth::attempt` returns `Some(_)` (after line 130)**:

```rust
match result {
    Some(_) => {
        // Check for OAuth return-to before returning JSON
        let return_to: Option<String> = session()
            .and_then(|s| s.get("oauth_return_to"));
        if let Some(url) = return_to {
            session_mut(|s| { s.remove("oauth_return_to"); });
            return Ok(HttpResponse::new()
                .status(302)
                .header("Location", url));
        }

        // Existing JSON response path (unchanged)
        let user = User::find_by_email(&input.email)...
        json_response!({ "user": { ... } })
    }
    None => Err(...)
}
```

Add imports: `use ferro::session::{session, session_mut};`

This is Option A from RESEARCH.md §5 — minimal addition to existing handler.

---

### `app/src/routes.rs` (new OAuth routes)

**Analog:** Current `app/src/routes.rs` (exact pattern)

**New routes to add** (after the MCP routes at line 44-45):
```rust
use ferro_mcp_oauth::handlers::{
    protected_resource_metadata, authorization_server_metadata,
    register_client, authorize_get, authorize_post, token_exchange,
};

// In routes! block:
// OAuth discovery (public, no middleware)
get!("/.well-known/oauth-protected-resource", protected_resource_metadata),
get!("/.well-known/oauth-authorization-server", authorization_server_metadata),

// Dynamic Client Registration (public)
post!("/register", register_client),

// Authorization + consent (session middleware already global)
get!("/authorize", authorize_get),
post!("/authorize", authorize_post),

// Token exchange (public, no session needed)
post!("/token", token_exchange),
```

No middleware group needed — discovery, register, and token endpoints are public. Session middleware is already global (bootstrapped in `app/src/bootstrap.rs`).

**Route macro pattern** (`app/src/routes.rs` lines 1-2):
```rust
use ferro::{get, group, post, resource, routes};
```

---

### `app/src/migrations/m20260611_create_oauth_clients_table.rs` (migration, CRUD)

**Analog:** `app/src/migrations/m20260228_create_api_keys_table.rs` (exact role match, lines 1-81)

**Full file pattern** — copy `m20260228_create_api_keys_table.rs` structure exactly:
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
                    .table(OauthClients::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(OauthClients::Id)
                        .big_integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(OauthClients::ClientId).string().not_null())
                    .col(ColumnDef::new(OauthClients::ClientName).string().null())
                    .col(ColumnDef::new(OauthClients::RedirectUris).text().not_null())
                    .col(ColumnDef::new(OauthClients::CreatedAt)
                        .timestamp_with_time_zone().not_null()
                        .default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_oauth_clients_client_id")
                    .table(OauthClients::Table)
                    .col(OauthClients::ClientId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OauthClients::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OauthClients {
    Table, Id, ClientId, ClientName, RedirectUris, CreatedAt,
}
```

---

### `app/src/migrations/mod.rs` (registration)

**Current file:** `app/src/migrations/mod.rs` lines 1-18

**Add one module declaration and one `Box::new(...)` entry:**
```rust
mod m20260611_create_oauth_clients_table;  // new line

// In migrations() vec:
Box::new(m20260611_create_oauth_clients_table::Migration),
```

Order matters: append after the `api_keys` migration entry (line 15).

---

### `.github/workflows/publish.yml` (Wave 2 addition)

**Current line** (line 274):
```yaml
WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server"
```

**Updated line:**
```yaml
WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server ferro-mcp-oauth"
```

`ferro-mcp-oauth` depends on `ferro-rs` (Wave 2 first crate) so it must be appended after `ferro-mcp-server`. The 5-second sleep between crates (line 289) provides sequencing within the wave.

Also add `ferro-mcp-oauth` to the library-change gate path patterns (the `case` statement around lines 50-59) — it is a publishable crate, so any change under `ferro-mcp-oauth/*` should trigger publication. The current gate already catches any path not explicitly excluded, so this is automatic.

---

## Shared Patterns

### `#[handler]` macro
**Source:** All controller files (e.g., `app/src/controllers/auth_controller.rs` line 33, `app/src/controllers/mcp.rs` line 39)
**Apply to:** All six `ferro-mcp-oauth` handler functions (`discovery`, `register`, `authorize_get`, `authorize_post`, `token_exchange`)
```rust
use ferro::{handler, HttpResponse, Request, Response};

#[handler]
pub async fn my_handler(req: Request) -> Response {
    Ok(HttpResponse::json(json!({ ... })))
}
```

### Error responses
**Source:** `app/src/controllers/auth_controller.rs` lines 50-52
**Apply to:** All handler functions in `ferro-mcp-oauth`
```rust
// 4xx with JSON body:
return Err(HttpResponse::json(json!({
    "error": "invalid_grant",
    "error_description": "..."
})).status(400));

// 5xx fallback:
return Err(HttpResponse::new().status(500));
```

### `thiserror` Error enum per crate
**Source:** `ferro-mcp-server/src/error.rs` (inferred from `ferro-mcp-server/Cargo.toml` `thiserror = "1.0"`)
**Apply to:** `ferro-mcp-oauth` — one `OAuthError` enum covering `InvalidGrant`, `InvalidClient`, `InvalidClientMetadata`, `ServerError(String)`, `JwtError(jsonwebtoken::errors::Error)`
```rust
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("invalid_grant")]
    InvalidGrant,
    // ...
}
```

### Session data access (put/get/remove)
**Source:** `framework/src/session/middleware.rs` lines 56-69
**Apply to:** `authorize.rs` (store OAuth params before login redirect), `auth_controller.rs` (read and clear return-to after login)
```rust
session_mut(|s| { s.put("key", value); });
session().and_then(|s| s.get::<String>("key"))
```

### `sanitize_identity` for env-sourced URL/name values in HTTP context
**Source:** `ferro-mcp-server/src/config.rs` lines 25-27
**Apply to:** `ferro-mcp-oauth/src/config.rs` — apply to `app_name` and `app_url` before storing in `OAuthConfig`
```rust
fn sanitize_identity(raw: String) -> String {
    raw.chars().filter(|c| !c.is_ascii_control()).collect()
}
```

---

## No Analog Found

All files have analogs. The one partial gap:

| File | Role | Note |
|------|------|------|
| `ferro-mcp-oauth/src/jwt.rs` (decode half) | service | `ferro-wallet/src/google/jwt.rs` is RS256/encode-only. The HS256 decode + `Validation` setup has no existing codebase analog — use RESEARCH.md §3 code directly. |

---

## Metadata

**Analog search scope:** `ferro-mcp-server/`, `framework/src/`, `ferro-wallet/src/`, `ferro-audit/src/`, `app/src/`, `.github/workflows/`
**Files scanned:** 19
**Pattern extraction date:** 2026-06-10
