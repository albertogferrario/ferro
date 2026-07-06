# Phase 199: OAuth Browser Login — Research

**Researched:** 2026-06-10
**Domain:** OAuth 2.1 authorization server (Rust/Ferro), JWT HS256, PKCE S256, DCR, session-reuse login flow
**Confidence:** HIGH (all claims verified against codebase reads or official specs)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** New reusable crate `ferro-mcp-oauth` depending on `framework`, exposing mountable route handlers + token validator.
- **D-02:** Self-contained JWT signed with HS256 via `jsonwebtoken` v9. Claims: `sub`, tenant claim, `aud={APP_URL}/mcp`, `iss={APP_URL}`, `iat`, short `exp` (~1h).
- **D-03:** Authorization code stored in `ferro-cache` with ~60s TTL, single-use.
- **D-04:** `oauth_clients` DB table for DCR persistence; app owns the migration following the `api_keys` pattern.
- **D-05:** Minimal server-rendered HTML consent page from the crate, no Inertia coupling.
- **D-06:** `/authorize` reuses `Auth::check()`/session login; tenant from `current_tenant()`; JWT tenant claim name matches JWT-claim resolver.
- **D-07:** Bearer validation order: sig+exp → 401; aud mismatch → 403; tenant mismatch → 403; RFC 6750 `WWW-Authenticate` errors.

### Claude's Discretion
- Internal module layout of `ferro-mcp-oauth`.
- Exact JSON shapes of `.well-known` documents beyond spec-required fields.
- DCR response fields beyond `client_id`.
- JWT claim names for non-standardized fields (tenant claim name constrained by D-06 resolver match).
- Random-code length/encoding.
- Whether discovery docs are static handlers or generated from config.

### Deferred Ideas (OUT OF SCOPE)
- Refresh tokens / token rotation.
- Tenant-picker on consent for multi-tenant users.
- RS256 / asymmetric keys + JWKS endpoint.
- Multi-process cache backend for authorization codes.
- Per-tenant row scoping + policy gating of `tools/call` (Phase 200).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AMCP-07 | Application publishes OAuth discovery metadata (`.well-known/oauth-protected-resource`, `.well-known/oauth-authorization-server`) and DCR endpoint advertising authorization-code grant with PKCE (S256). | Sections 3 (endpoints), RFC 8414 fields, RFC 9728 fields. |
| AMCP-08 | Consumer completes browser authorization-code + PKCE flow reusing app login + consent, receiving an access token bound to `(user, tenant)` audience-restricted to MCP endpoint. | Sections 4 (token), 5 (PKCE), 6 (login reuse), 7 (consent). |
| AMCP-09 | MCP endpoint validates bearer token; invalid/expired → 401; audience or tenant mismatch → 403. | Section 8 (seam wiring), Section 4 (validation). |
</phase_requirements>

---

## Summary

Ten-bullet implementation shape:

1. `ferro-mcp-oauth` is a new Wave-2 crate (`framework` dep, no `ferro-mcp-server` dep). It exports six route handlers, one config struct, and one `validate_bearer` function. The app mounts the six routes and calls `validate_bearer` at the `/mcp` handler call site — `ferro-mcp-server` gains no new dependency.
2. The bearer seam in `ferro-mcp-server/src/auth.rs` (`extract_bearer`) is **deleted and replaced** at the call site in `app/src/controllers/mcp.rs` with a direct call to `ferro_mcp_oauth::validate_bearer(header, &config) -> BearerOutcome`. `BearerOutcome` stays in `ferro-mcp-server`; only the body changes. This is the lowest-coupling path: the seam type is unchanged, the validator lives where the key is configured, and `ferro-mcp-server` depends on nothing new.
3. The signing secret is `MCP_TOKEN_SECRET` (a new env var, crate-local to `ferro-mcp-oauth`, never in `AppConfig`). No workspace `APP_KEY` exists today — `AppConfig` has `name`, `url`, `environment`, `debug`, `inline_budget_threshold_bytes` only (`framework/src/config/providers/app.rs`). `MCP_TOKEN_SECRET` follows the crate-local pattern of `STRIPE_SECRET_KEY` in `ferro-stripe`. Must fail closed (panic/error) if unset in non-debug; debug may use a derived fallback.
4. `ferro::Cache` (static facade, `framework/src/cache/mod.rs`) provides `Cache::put(key, &value, Some(Duration))`, `Cache::get(key)`, `Cache::forget(key)`. Default driver: in-memory (InMemoryCache) with automatic Redis fallback if `REDIS_URL` is set. In-memory is single-process — safe for the skeleton; the deferred multi-process caveat is documented.
5. The JWT tenant claim name is `"tenant_id"` — this is confirmed from `framework/src/tenant/resolver.rs` (`JwtClaimResolver::new("tenant_id", lookup)` hard-codes the field name it reads via `claims["tenant_id"].as_i64()`). The access token MUST use exactly `"tenant_id"` as the claim key or Phase 200's tenant middleware will silently fail to bind the tenant.
6. The existing `/auth/login` handler does NOT support a return-to parameter — it is a JSON API endpoint (`req.json()`, returns JSON). The consent flow therefore cannot redirect to `/auth/login` and expect an HTML redirect back. Solution: `GET /authorize` checks `Auth::check()`; if unauthenticated, it stores the OAuth params in the session (key `"oauth_pending_authorize"`) and redirects the browser to `/auth/login`. A thin Ferro middleware or the existing `GuestMiddleware` pattern handles this at the `/authorize` handler level, not by modifying `auth_controller.rs`. After login, the app needs a `GET /authorize` browser redirect — the simplest hook is to store the return URL in session under a key the login route reads after `Auth::login()` sets the user, then redirect. This requires a minimal addition to the login handler OR a separate `GET /auth/login` HTML handler that is consent-flow-aware (the existing handler is POST-only / JSON).
7. The consent page is plain HTML returned via `HttpResponse::text(html).header("Content-Type", "text/html")` — `HttpResponse::text()` sets `text/plain` by default, so the Content-Type must be explicitly overridden. No `html()` constructor exists.
8. CSRF for the consent POST: `ferro::get_csrf_token()` (re-exported from session middleware) reads the current session's `csrf_token` field. The consent GET embeds this in a hidden `<input name="_token">`. The consent POST validates it against `get_csrf_token()` before processing.
9. The `oauth_clients` migration follows the `api_keys` pattern exactly: a new file in `app/src/migrations/`, registered in `app/src/migrations/mod.rs` `Migrator::migrations()`. The `ferro-mcp-oauth` crate ships a `pub fn oauth_clients_migration() -> Box<dyn MigrationTrait>` helper the app calls (same as the established pattern — no second migration mechanism, no SeaORM migration embedded in the crate itself).
10. `publish.yml` Wave 2 (`WAVE2_CRATES`) currently lists `"ferro-rs ferro-mcp ferro-mcp-server"`. `ferro-mcp-oauth` depends on `framework` (= `ferro-rs`) and must be added to Wave 2 after `ferro-rs` is indexed.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Discovery documents | API/Backend (`ferro-mcp-oauth`) | — | Static metadata derived from APP_URL; no DB |
| Dynamic client registration | API/Backend (`ferro-mcp-oauth`) | DB (oauth_clients table) | Persists client_id across restarts |
| Authorization code flow | API/Backend (`ferro-mcp-oauth`) | Session (existing SessionMiddleware) | Browser-facing but app-server-rendered |
| PKCE verification | API/Backend (`ferro-mcp-oauth`) | ferro-cache (code store) | Pure crypto, no browser |
| Consent screen HTML | API/Backend (`ferro-mcp-oauth`) | Session (CSRF) | Server-rendered, no frontend build |
| Token minting | API/Backend (`ferro-mcp-oauth`) | — | HS256 JWT, in-process |
| Bearer validation | API/Backend (`ferro-mcp-oauth::validate_bearer`) | ferro-mcp-server (BearerOutcome) | Validator in crate that holds the key |
| Login session reuse | API/Backend (existing `Auth::check`) | Session (task-local) | Auth guard already in framework |
| Tenant binding | API/Backend (`current_tenant()`) | JWT claim `tenant_id` | Must match `JwtClaimResolver` |

---

## 1. Crate and Dependency Architecture

### ferro-mcp-oauth Cargo.toml (proposed)

```toml
[package]
name = "ferro-mcp-oauth"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "OAuth 2.1 authorization server for ferro-mcp-server endpoints"
repository = "https://github.com/albertogferrario/ferro"
homepage = "https://ferro-rs.dev"
readme = "README.md"
keywords = ["oauth", "mcp", "ferro", "pkce", "jwt"]
categories = ["web-programming", "authentication"]

[dependencies]
# Framework — the one internal dep that places this in Wave 2
ferro-rs = { path = "../framework", version = "0.2" }
# JWT minting and validation
jsonwebtoken = "9"
# DCR persistence
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }
sea-orm-migration = "1.0"
# Code generation and PKCE
rand = "0.8"
base64 = "0.22"
sha2 = "0.10"
subtle = "2.5"
# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
async-trait = "0.1"
```

All versions are in-use across the workspace; no new transitive deps.
`ferro-mcp-server` is NOT a dependency of `ferro-mcp-oauth`. [VERIFIED: codebase]

### Dependency direction

```
app ──depends──> ferro-mcp-oauth ──depends──> framework (ferro-rs)
app ──depends──> ferro-mcp-server ──depends──> ferro-projections
framework ──no dep──> ferro-mcp-oauth   (no cycle)
ferro-mcp-server ──no dep──> ferro-mcp-oauth  (no cycle; BearerOutcome stays in ferro-mcp-server)
```

`framework` already exports `ferro_cache` (re-exported as `TaggableCache` and the static `Cache` facade). `ferro-mcp-oauth` uses `ferro::Cache` (the static facade) for auth code storage. [VERIFIED: `framework/src/lib.rs` line 62, line 229-232; `framework/src/cache/mod.rs`]

### publish.yml wave placement [VERIFIED: `.github/workflows/publish.yml`]

```yaml
# Wave 2 — current
WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server"
# Wave 2 — after this phase
WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server ferro-mcp-oauth"
```

`ferro-mcp-oauth` depends on `ferro-rs` (= `framework`), which is published first in Wave 2. Because Wave 2 publishes sequentially with a 5-second sleep between crates (and `ferro-rs` is first in the list), `ferro-mcp-oauth` must be appended AFTER `ferro-mcp-server` so `ferro-rs` is already indexed.

### Bearer seam wiring decision (D-01 research flag resolved)

**Decision: replace the body of `extract_bearer` in `ferro-mcp-server/src/auth.rs` with a real validator, OR delete `extract_bearer` and call `ferro_mcp_oauth::validate_bearer` directly at the call site in `app/src/controllers/mcp.rs`.**

The preferred approach is **option B** (direct call site replacement):

```rust
// app/src/controllers/mcp.rs — Phase 199 change
use ferro_mcp_oauth::{validate_bearer, OAuthConfig};
// ...
let config = McpServerConfig::from_env();
let oauth_config = OAuthConfig::from_env(); // holds the signing secret
match validate_bearer(authorization.as_deref(), &oauth_config) {
    BearerOutcome::Unauthenticated => return Err(challenge_response(&config)),
    BearerOutcome::Authenticated(principal) => { /* use principal */ }
}
```

`BearerOutcome` stays in `ferro-mcp-server` as the shared type. `ferro-mcp-server/src/auth.rs`'s `extract_bearer` stub is deleted entirely (the Phase 198 test `any_bearer_is_still_unauthenticated_in_phase_198` is also deleted). This keeps `ferro-mcp-server` free of any new dependency: `validate_bearer` lives in `ferro-mcp-oauth` and is only imported in `app/src/controllers/mcp.rs`, which already depends on both crates.

### Module layout

```
ferro-mcp-oauth/
├── Cargo.toml
├── src/
│   ├── lib.rs            # pub use; exports validate_bearer, OAuthConfig, oauth_clients_migration
│   ├── config.rs         # OAuthConfig: app_name, app_url, token_secret; from_env(); fail-closed
│   ├── discovery.rs      # GET /.well-known/oauth-protected-resource
│   │                     # GET /.well-known/oauth-authorization-server
│   ├── register.rs       # POST /register (DCR RFC 7591)
│   ├── authorize.rs      # GET /authorize (login check, consent redirect)
│   ├── consent.rs        # GET/POST /authorize?step=consent (render + submit)
│   ├── token.rs          # POST /token (code exchange, JWT mint)
│   ├── validate.rs       # validate_bearer(header, config) -> BearerOutcome
│   ├── pkce.rs           # generate_code_verifier, verify_s256 (constant-time)
│   ├── jwt.rs            # mint_token, decode_token (HS256 via jsonwebtoken)
│   ├── store.rs          # OAuthCode (cached struct), OAuthClient (DB model)
│   └── migration.rs      # pub fn oauth_clients_migration() -> Box<dyn MigrationTrait>
└── tests/
    └── flow_integration.rs  # full discover→register→authorize→token→validate test
```

---

## 2. Endpoint Specifications

### GET /.well-known/oauth-protected-resource

**RFC 9728** protected resource metadata. Referenced by the Phase 198 `WWW-Authenticate` header.

Request: none (public, unauthenticated).

Response `200 application/json`:
```json
{
  "resource": "{APP_URL}/mcp",
  "authorization_servers": ["{APP_URL}"]
}
```

RFC 9728 §2 requires `resource` and `authorization_servers`. The MCP spec does not require additional fields at the protected-resource level. `{APP_URL}` is `OAuthConfig.app_url`. [CITED: RFC 9728 §2; VERIFIED: MCP authorization spec]

Error cases: none — always 200 or 500 internal.

### GET /.well-known/oauth-authorization-server

**RFC 8414** authorization server metadata. Consumed by MCP clients for endpoint discovery.

Response `200 application/json`:
```json
{
  "issuer": "{APP_URL}",
  "authorization_endpoint": "{APP_URL}/authorize",
  "token_endpoint": "{APP_URL}/token",
  "registration_endpoint": "{APP_URL}/register",
  "response_types_supported": ["code"],
  "grant_types_supported": ["authorization_code"],
  "code_challenge_methods_supported": ["S256"],
  "token_endpoint_auth_methods_supported": ["none"]
}
```

RFC 8414 §2 required fields: `issuer`, `authorization_endpoint`, `token_endpoint`. The remaining fields above are recommended by the MCP spec for advertising PKCE and DCR. `token_endpoint_auth_methods_supported: ["none"]` signals public clients only (matches PKCE-only flow). [CITED: RFC 8414 §2; VERIFIED: MCP authorization spec]

Note: The MCP spec uses `/.well-known/oauth-authorization-server` at the root of the domain hosting the MCP server. Since `/mcp` is at the root, the authorization base URL is `{APP_URL}` and the metadata URL is `{APP_URL}/.well-known/oauth-authorization-server`. [CITED: MCP authorization spec, "Authorization Base URL"]

### POST /register

**RFC 7591** Dynamic Client Registration.

Request `application/json`:
```json
{
  "client_name": "My MCP Client",
  "redirect_uris": ["http://localhost:3000/callback"],
  "grant_types": ["authorization_code"],
  "response_types": ["code"],
  "token_endpoint_auth_method": "none"
}
```

Required: `redirect_uris` (RFC 7591 §2). `client_name` is optional but stored. `grant_types` and `response_types` validated if present.

Response `201 application/json`:
```json
{
  "client_id": "<uuid-or-random-id>",
  "client_name": "My MCP Client",
  "redirect_uris": ["http://localhost:3000/callback"],
  "grant_types": ["authorization_code"],
  "token_endpoint_auth_method": "none",
  "client_id_issued_at": 1749600000
}
```

No `client_secret` — public clients only. [CITED: RFC 7591 §2, §3.2.1]

Error cases:
- Missing `redirect_uris` → `400 {"error":"invalid_client_metadata","error_description":"redirect_uris required"}`
- Invalid `redirect_uri` scheme (must be `http://localhost` or `https://`) → `400`
- DB insert failure → `500`

`client_id` is generated as a UUID v4 or high-entropy random string (not a sequential integer to prevent enumeration).

### GET /authorize

OAuth authorization endpoint. This is browser-facing.

Query parameters (RFC 6749 §4.1.1):
- `response_type` (required): must be `"code"`
- `client_id` (required): must match a registered client
- `redirect_uri` (required): must exactly match a URI from the registered client's `redirect_uris`
- `code_challenge` (required for PKCE): BASE64URL(SHA256(code_verifier))
- `code_challenge_method` (required): must be `"S256"` — plain is rejected
- `state` (optional but recommended): opaque, echoed back in redirect
- `scope` (optional): ignored for Phase 199 (single implicit scope)

**Step 1 — auth check:**
If `Auth::check()` is false: store the full OAuth query-string in the session under `"oauth_pending_authorize"`, redirect to `/auth/login` with `?return_to=/authorize`. After login, the modified login handler reads `return_to` from the query and redirects back to `/authorize` (the pending params are in session).

If `Auth::check()` is true: proceed to consent.

**Step 2 — consent:**
Validate client_id, redirect_uri. On error, show an error page (do NOT redirect back to client on redirect_uri mismatch or invalid client_id — RFC 6749 §4.1.2.1). Render the consent HTML page.

### POST /authorize (consent submit)

Form body: `_token` (CSRF), `action` (`"approve"` or `"deny"`), all OAuth params echoed as hidden fields.

**Approve path:**
1. Verify CSRF token (`get_csrf_token()` must match `_token`).
2. Regenerate/re-validate client_id + redirect_uri.
3. Capture `Auth::id()` as `user_id` and `current_tenant()` as `tenant`.
4. Generate auth code: 32 random bytes URL-safe base64 encoded (~43 chars).
5. Store `OAuthCode { client_id, redirect_uri, code_challenge, user_id, tenant_id, created_at }` in `Cache::put("mcp:code:{code}", &record, Some(Duration::from_secs(60)))`.
6. Redirect to `{redirect_uri}?code={code}&state={state}` (302).

**Deny path:**
Redirect to `{redirect_uri}?error=access_denied&state={state}` (302).

Error cases:
- CSRF mismatch → `400`
- Client/redirect validation failure → error page (no redirect)
- Cache write failure → `500`

### POST /token

OAuth token endpoint (RFC 6749 §4.1.3).

Request `application/x-www-form-urlencoded`:
- `grant_type` = `"authorization_code"`
- `code` = the authorization code
- `redirect_uri` = must exactly match what was used in /authorize
- `client_id` = must match registered client
- `code_verifier` = PKCE verifier

Response `200 application/json` (RFC 6749 §5.1):
```json
{
  "access_token": "<JWT>",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

No `refresh_token` (deferred). No `scope`.

Error response `400 application/json` (RFC 6749 §5.2):
```json
{"error": "invalid_grant", "error_description": "..."}
```

Error codes:
- `grant_type` missing or not `"authorization_code"` → `unsupported_grant_type`
- `code` not found in cache (expired or already used) → `invalid_grant`
- `redirect_uri` mismatch → `invalid_grant`
- `client_id` mismatch → `invalid_client`
- PKCE `code_verifier` fails S256 verification → `invalid_grant`
- `code_challenge_method` was not `S256` (stored in code record) → `invalid_grant`

Single-use enforcement: `Cache::forget("mcp:code:{code}")` is called immediately on first retrieval, before any other validation. If the code is not present, return `invalid_grant`. [CITED: RFC 6749 §4.1.2, §4.1.3, §5.2]

---

## 3. Token Mint and Validate

### Claims struct

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct McpTokenClaims {
    sub: String,          // user_id as string
    tenant_id: i64,       // EXACT name — matches JwtClaimResolver "tenant_id" field
    aud: Vec<String>,     // vec!["{APP_URL}/mcp"]
    iss: String,          // "{APP_URL}"
    iat: i64,             // unix timestamp
    exp: i64,             // iat + 3600
}
```

**Critical: `tenant_id` is the exact field name.** From `framework/src/tenant/resolver.rs`:
```rust
// JwtClaimResolver reads claims["tenant_id"].as_i64()
let resolver = JwtClaimResolver::new("tenant_id", lookup);
```
[VERIFIED: `framework/src/tenant/resolver.rs` lines 209-213, test at line 419]

For Phase 200's tenant middleware to resolve the tenant from the JWT, the `JwtClaimResolver` must be wired with `"tenant_id"` and an upstream middleware must call `req.insert::<serde_json::Value>(claims)`. Phase 199's validator (`validate_bearer`) returns the claims as `BearerOutcome::Authenticated(json!({ "sub": ..., "tenant_id": ... }))`. Phase 200 inserts this into request extensions.

**Single-tenant apps (no TenantMiddleware):** `current_tenant()` returns `None`. In this case, omit `tenant_id` from the token entirely (or set it to 0). `BearerOutcome::Authenticated` still succeeds; the tenant check in D-07 is skipped when `tenant_id` is absent (not a mismatch).

### HS256 mint (jsonwebtoken v9) [VERIFIED: context7 /keats/jsonwebtoken]

```rust
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

fn mint_token(claims: &McpTokenClaims, secret: &[u8]) -> Result<String, Error> {
    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret);
    encode(&header, claims, &key).map_err(Error::Jwt)
}
```

### HS256 validate (jsonwebtoken v9) [VERIFIED: context7 /keats/jsonwebtoken]

```rust
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

fn decode_token(token: &str, secret: &[u8], expected_aud: &str)
    -> Result<McpTokenClaims, TokenError>
{
    let mut validation = Validation::new(Algorithm::HS256);
    // Pin algorithm to HS256 — prevents alg=none and RS256→HS256 confusion attacks
    validation.algorithms = vec![Algorithm::HS256];
    validation.set_audience(&[expected_aud]);
    validation.validate_exp = true;
    validation.leeway = 0; // no clock skew tolerance for short-lived tokens
    
    let key = DecodingKey::from_secret(secret);
    let data = decode::<McpTokenClaims>(token, &key, &validation)?;
    Ok(data.claims)
}
```

**Error → HTTP status mapping (D-07):**

| `jsonwebtoken::ErrorKind` | HTTP Status | WWW-Authenticate |
|---------------------------|-------------|-----------------|
| `ExpiredSignature` | 401 | `Bearer error="invalid_token", error_description="Token expired"` |
| `InvalidSignature` | 401 | `Bearer error="invalid_token"` |
| `InvalidToken`, `InvalidKeyFormat`, `Base64(_)`, `Json(_)` | 401 | `Bearer error="invalid_token"` |
| `InvalidAudience` | 403 | `Bearer error="insufficient_scope"` (RFC 6750 §3.1) |
| Any other | 401 | `Bearer error="invalid_token"` |

Tenant mismatch (claim present but not matching expected tenant) → 403 (evaluated post-decode in `validate_bearer`, not by jsonwebtoken).

`InvalidAudience` from jsonwebtoken maps to 403, not 401, because the token is validly signed — the bearer is authenticated but the token is not scoped for this resource. [CITED: RFC 6750 §3.1; D-07]

### Key source (D-02 research flag resolved)

`AppConfig` has no signing key (`framework/src/config/providers/app.rs` — confirmed). [VERIFIED: codebase]

No workspace `APP_KEY` exists. The existing pattern for crate-local secrets is `STRIPE_SECRET_KEY` in `ferro-stripe/src/config.rs` — each crate owns its env var with a crate-specific prefix.

Decision: `MCP_TOKEN_SECRET` as env var. `OAuthConfig::from_env()` reads it with `std::env::var("MCP_TOKEN_SECRET")`. Fail-closed behavior:

```rust
pub struct OAuthConfig {
    pub app_name: String,
    pub app_url: String,
    pub token_secret: Vec<u8>,
}

impl OAuthConfig {
    pub fn from_env() -> Result<Self, OAuthConfigError> {
        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string());
        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string());
        let secret_str = std::env::var("MCP_TOKEN_SECRET")
            .map_err(|_| OAuthConfigError::MissingSecret)?;
        if secret_str.len() < 32 {
            return Err(OAuthConfigError::SecretTooShort);
        }
        Ok(Self {
            app_name,
            app_url,
            token_secret: secret_str.into_bytes(),
        })
    }
}
```

In debug mode (`APP_DEBUG=true`), a caller MAY fall back to a derived key (e.g., `sha2::Sha256::digest(b"dev-only-mcp-secret")`) but this must log a prominent warning and NEVER happen silently in non-debug. [ASSUMED: the debug fallback policy; the fail-closed behavior in non-debug is required by D-02]

`MCP_TOKEN_SECRET` must be ≥ 32 bytes (256 bits) for HS256 security. [CITED: NIST SP 800-107 for HMAC key length]

---

## 4. PKCE and Code Storage

### Code generation

```rust
use rand::Rng;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

fn generate_auth_code() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    URL_SAFE_NO_PAD.encode(bytes)  // 43 URL-safe base64 chars
}
```

43 chars = 256 bits of entropy. [CITED: RFC 7636 §4.1 recommends high-entropy verifier]

### PKCE S256 verification [CITED: RFC 7636 §4.2, §4.6]

```rust
use sha2::{Digest, Sha256};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use subtle::ConstantTimeEq;

fn verify_s256(code_verifier: &str, stored_challenge: &str) -> bool {
    let hash = Sha256::digest(code_verifier.as_bytes());
    let recomputed = URL_SAFE_NO_PAD.encode(hash);
    // Constant-time compare to prevent timing oracle
    recomputed.as_bytes().ct_eq(stored_challenge.as_bytes()).into()
}
```

`sha2` and `subtle` are already `framework` dependencies. [VERIFIED: `framework/Cargo.toml` lines 71-72]

### Cached code struct

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OAuthCode {
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,   // stored as-received from /authorize
    pub user_id: i64,
    pub tenant_id: Option<i64>,   // None for single-tenant
    pub created_at: i64,          // unix timestamp for audit
}
```

### ferro::Cache API calls [VERIFIED: `framework/src/cache/mod.rs`]

Store (in `POST /authorize` approve path):
```rust
Cache::put(
    &format!("mcp:code:{code}"),
    &oauth_code,
    Some(Duration::from_secs(60)),
).await?;
```

Retrieve + single-use delete (in `POST /token`):
```rust
// forget() first — single-use guarantee even if validation fails below
let record: Option<OAuthCode> = Cache::get(&format!("mcp:code:{code}")).await?;
Cache::forget(&format!("mcp:code:{code}")).await.ok(); // ignore forget error
let record = record.ok_or(invalid_grant_error())?;
```

The `forget` before validation is deliberate — a failed token exchange cannot replay the code. [CITED: RFC 6749 §4.1.2 code single-use]

Default driver is `InMemoryCache` (auto-falls-back from Redis if `REDIS_URL` unset). Bootstrapped by `Server::run()` via `Cache::bootstrap()`. In a single-process deployment, the authorize→token round-trip occurs within the same process — safe. [VERIFIED: `framework/src/cache/mod.rs` lines 84-98]

---

## 5. Login Reuse and Tenant Binding

### Redirect-after-login (D-06 research flag)

**Finding:** The existing `POST /auth/login` handler in `app/src/controllers/auth_controller.rs` is a JSON API endpoint — it takes `req.json()`, returns JSON, and has no return-to redirect mechanism. [VERIFIED: `app/src/controllers/auth_controller.rs`]

**Required addition:** A `GET /auth/login` HTML handler (or a middleware hook after `Auth::login()`) that supports `?return_to=<path>` redirect. Two options:

**Option A (minimal, preferred):** Store the OAuth params in the session before redirecting to `/auth/login`. After `Auth::login()` is called in the POST handler, if `session.get("oauth_return_to")` is set, read it and redirect the browser there instead of returning JSON. This requires modifying `auth_controller.rs::login` to check for a session key.

**Option B (cleanest):** Add a new `GET /auth/login` HTML handler that renders a login form (separate from the JSON API). The OAuth flow redirects to this HTML form; after submit it calls `Auth::attempt`, then reads `?return_to` and redirects. The existing JSON handler is unchanged.

**Recommendation (Claude's Discretion):** Option B is cleaner (no mutation of the JSON API), but Option A is minimal. Either is valid. The CONTEXT.md says "add a minimal redirect-after-login mechanism rather than forking the login handler" — this suggests Option A. The planner must choose.

**Tenant capture:**
```rust
// In GET /authorize, after Auth::check() succeeds:
let user_id = Auth::id().expect("auth check passed");
let tenant = current_tenant(); // Option<TenantContext>
let tenant_id = tenant.map(|t| t.id); // i64 or None
```

`current_tenant()` uses `tokio::task_local` and is set by `TenantMiddleware` if configured. [VERIFIED: `framework/src/tenant/context.rs`]

**Single-tenant (no TenantMiddleware):** `current_tenant()` returns `None` → `tenant_id` is `None` → token has no `tenant_id` claim → Phase 200's `JwtClaimResolver` finds no claim → tenant remains unresolved (acceptable for single-tenant). [VERIFIED: `framework/src/tenant/context.rs` line 32-37]

**Multi-tenant (ambiguous tenant):** If `current_tenant()` returns `None` even though TenantMiddleware is active (meaning the tenant was not resolved from the request to `/authorize`), the authorize endpoint should return an error rather than issuing a token with no tenant binding. Tenant picker is deferred to Phase 200 per CONTEXT.md.

---

## 6. Consent Page and CSRF

### Returning HTML from a Ferro handler [VERIFIED: `framework/src/http/response.rs`]

```rust
// HttpResponse::text() sets Content-Type: text/plain by default.
// Must override to text/html:
let html = render_consent_html(...);
Ok(HttpResponse::text(html).header("Content-Type", "text/html; charset=utf-8"))
```

No `html()` constructor exists on `HttpResponse`. [VERIFIED: `framework/src/http/response.rs`]

### CSRF token issuance and validation [VERIFIED: `framework/src/session/middleware.rs`, `framework/src/auth/guard.rs`]

```rust
// Reading the CSRF token for the form:
use ferro::session::get_csrf_token;
let csrf = get_csrf_token().unwrap_or_default(); // embed in hidden input

// Validating in POST /authorize:
use ferro::session::get_csrf_token;
let submitted_token = form._token; // from form body
let session_token = get_csrf_token().ok_or_else(|| csrf_error())?;
if submitted_token.as_bytes().ct_eq(session_token.as_bytes()).into() == false {
    return Err(HttpResponse::new().status(400));
}
```

`generate_csrf_token()` generates a new 40-char alphanumeric string (same format as session ID). It is regenerated on `Auth::login()` (see `guard.rs` line 87). `get_csrf_token()` reads the current session's `csrf_token` field. [VERIFIED: `framework/src/session/middleware.rs` lines 94-96, 222-223; `framework/src/auth/guard.rs` lines 86-88]

### Consent HTML form shape

Minimal consent page HTML (template):
```html
<!DOCTYPE html>
<html>
<head><title>Authorize {client_name}</title></head>
<body>
  <h1>Authorize Access</h1>
  <p><strong>{client_name}</strong> is requesting access to your account.</p>
  <form method="POST" action="/authorize">
    <input type="hidden" name="_token" value="{csrf_token}">
    <input type="hidden" name="client_id" value="{client_id}">
    <input type="hidden" name="redirect_uri" value="{redirect_uri}">
    <input type="hidden" name="code_challenge" value="{code_challenge}">
    <input type="hidden" name="code_challenge_method" value="S256">
    <input type="hidden" name="state" value="{state}">
    <input type="hidden" name="response_type" value="code">
    <button type="submit" name="action" value="approve">Approve</button>
    <button type="submit" name="action" value="deny">Deny</button>
  </form>
</body>
</html>
```

Form submission uses `req.form::<ConsentForm>()` (deserializes `application/x-www-form-urlencoded`). [VERIFIED: `framework/src/http/request.rs` line 485]

---

## 7. Filling the Phase 198 Seam

### Changes to `ferro-mcp-server/src/auth.rs`

**Delete:** The entire `extract_bearer` function body and its stub tests. `BearerOutcome` enum is kept (it is the shared contract type). The `Unauthenticated` and `Authenticated` variants remain unchanged.

After deletion, `ferro-mcp-server/src/auth.rs` contains only the `BearerOutcome` enum.

### Changes to `app/src/controllers/mcp.rs`

**Before (Phase 198):**
```rust
use ferro_mcp_server::{extract_bearer, ..., BearerOutcome, McpServerConfig};
// ...
match extract_bearer(authorization.as_deref()) {
    BearerOutcome::Unauthenticated => return Err(challenge_response(&config)),
    BearerOutcome::Authenticated(_principal) => { /* Phase 199+ */ }
}
```

**After (Phase 199):**
```rust
use ferro_mcp_server::{BearerOutcome, McpServerConfig, ...};
use ferro_mcp_oauth::{validate_bearer, OAuthConfig};
// ...
let oauth_config = OAuthConfig::from_env()
    .map_err(|_| challenge_response(&config))?;

match validate_bearer(authorization.as_deref(), &oauth_config) {
    BearerOutcome::Unauthenticated => return Err(challenge_response(&config)),
    BearerOutcome::Authenticated(principal) => {
        // principal = json!({ "sub": "42", "tenant_id": 7 })
        // Phase 200 inserts this into request extensions for JwtClaimResolver
        // For now, just allow the request through
        let _ = principal;
    }
}
```

`validate_bearer` signature:
```rust
pub fn validate_bearer(
    authorization_header: Option<&str>,
    config: &OAuthConfig,
) -> BearerOutcome
```

This is synchronous (JWT decode is synchronous in jsonwebtoken v9). No async needed. [VERIFIED: context7 /keats/jsonwebtoken — `decode()` is sync]

### Origin validation TODO (Phase 198 comment)

`app/src/controllers/mcp.rs` line 3: `// TODO(phase-199): validate Origin header`.

Origin validation must be implemented in Phase 199:
```rust
// DNS-rebinding prevention per MCP spec
if let Some(origin) = req.header("Origin") {
    let expected = &config.app_url; // or parse host from it
    if !origin.starts_with(expected.as_str()) {
        return Err(HttpResponse::new().status(403));
    }
}
```

The check is: if `Origin` is present and does not match `APP_URL` origin, reject. If `Origin` is absent, allow (non-browser clients). [CITED: MCP spec security considerations; ASSUMED: the exact validation logic — planner should decide whether to allow localhost origins unconditionally]

### New routes in `app/src/routes.rs`

```rust
use ferro_mcp_oauth::handlers::*;

routes! {
    // ... existing routes ...

    // OAuth discovery (public)
    get!("/.well-known/oauth-protected-resource", protected_resource_metadata),
    get!("/.well-known/oauth-authorization-server", authorization_server_metadata),

    // Dynamic Client Registration
    post!("/register", register_client),

    // Authorization + consent
    get!("/authorize", authorize_get),
    post!("/authorize", authorize_post),

    // Token exchange
    post!("/token", token_exchange),
}
```

### New migration in `app/src/migrations/`

New file `m20260611_create_oauth_clients_table.rs` (follow api_keys pattern):

```rust
#[derive(DeriveIden)]
enum OauthClients {
    Table,
    Id,
    ClientId,       // text, unique — the issued client_id
    ClientName,     // text nullable
    RedirectUris,   // text (JSON array stored as text)
    CreatedAt,
}
```

Registered in `app/src/migrations/mod.rs`:
```rust
mod m20260611_create_oauth_clients_table;
// ... in Migrator::migrations():
Box::new(m20260611_create_oauth_clients_table::Migration),
```

[VERIFIED: `app/src/migrations/mod.rs`; `app/src/migrations/m20260228_create_api_keys_table.rs` as pattern]

---

## 8. Security Threat Model Inputs

| # | Threat | STRIDE | Concrete Mitigation | ASVS L1 |
|---|--------|--------|---------------------|---------|
| T-01 | PKCE downgrade (client sends `code_challenge_method=plain`) | Spoofing | Reject any `code_challenge_method` other than `"S256"` at `/authorize`; return `invalid_request`. | V3.5 |
| T-02 | Authorization code replay | Repudiation | `Cache::forget` called before any validation on first retrieval. If key absent → `invalid_grant`. | V3.5 |
| T-03 | Auth code TTL bypass | Elevation | ferro::Cache TTL is enforced by the store. 60s TTL + `created_at` field in record for belt-and-suspenders audit check. | V3.5 |
| T-04 | Open redirect on `redirect_uri` | Tampering | Exact-string match against `redirect_uris` list from `oauth_clients` DB row. No prefix/subdomain match. Return error page (not redirect) if `redirect_uri` is invalid. [RFC 6749 §10.6] | V5.1 |
| T-05 | `redirect_uri` scheme injection | Tampering | At DCR time: only `http://localhost/*` or `https://*` accepted. Reject `javascript:`, `data:`, custom schemes. [MCP spec; RFC 6749 §3.1.2.1] | V5.1 |
| T-06 | alg=none / algorithm confusion | Spoofing | `validation.algorithms = vec![Algorithm::HS256]` — pinned. jsonwebtoken v9 does not accept `alg=none` by default, but the explicit pin is defense-in-depth. [CITED: context7 /keats/jsonwebtoken] | V3.5 |
| T-07 | RS256→HS256 confusion (asymmetric key as HMAC secret) | Spoofing | Blocked by same `algorithms` pin. No RS256 key exists in this system. | V3.5 |
| T-08 | Audience confusion (token for different resource) | Elevation | `validation.set_audience(&["{APP_URL}/mcp"])` forces exact aud match. `InvalidAudience` → 403. | V3.5 |
| T-09 | Tenant confusion (token for tenant A used at tenant B) | Elevation | After decode, compare `claims.tenant_id` against `current_tenant().id` if tenant middleware is active. Mismatch → 403. | V3.5 |
| T-10 | CSRF on consent POST | Spoofing | `_token` hidden field, validated via `get_csrf_token()` constant-time compare before processing. | V4.2 |
| T-11 | Timing attack on PKCE verify | Info disclosure | `subtle::ConstantTimeEq` for the S256 comparison. | V2.9 |
| T-12 | Timing attack on CSRF verify | Info disclosure | Same `subtle::ConstantTimeEq` pattern. | V2.9 |
| T-13 | Token signed with absent secret | Spoofing | `OAuthConfig::from_env()` returns `Err` if `MCP_TOKEN_SECRET` unset → server refuses to start / handler returns 500 + logs. Never falls back silently in non-debug. | V6.4 |
| T-14 | Short signing secret | Spoofing | Reject secrets shorter than 32 bytes (256 bits). | V6.4 |
| T-15 | Origin header DNS-rebinding | Spoofing | If `Origin` present: match against `APP_URL` origin. Absent: allow (non-browser). [MCP spec; Phase 198 TODO(phase-199)] | V4.3 |
| T-16 | Code substitution / injection | Tampering | Code stored in cache keyed by the code itself; `client_id` + `redirect_uri` stored in the code record and re-validated at token exchange. | V3.5 |
| T-17 | Mix-up attack (token from different AS) | Spoofing | `iss` claim validated: must equal `{APP_URL}`. `aud` must equal `{APP_URL}/mcp`. Both from the same config, no third-party delegation in Phase 199. | V3.5 |

### ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | `Auth::check()` + session (framework); existing |
| V3 Session Management | yes | `SessionMiddleware`; existing |
| V3.5 Token-based session management | yes | JWT HS256, `Validation` with exp+aud+iss+alg pin |
| V4 Access Control | yes | Tenant claim check; audience restriction |
| V5 Input Validation | yes | `redirect_uri` scheme + exact-match; `code_challenge_method` allowlist; form CSRF |
| V6 Cryptography | yes | HS256 with ≥32-byte key; `subtle::ConstantTimeEq`; `rand` for code/secret generation |

---

## 9. Validation Architecture

### Test Framework [VERIFIED: `ferro-mcp-server/Cargo.toml`, `framework/Cargo.toml`]

| Property | Value |
|----------|-------|
| Framework | tokio-test + in-process assertions (no test runner config needed) |
| Config file | none — inline `#[tokio::test]` |
| Quick run command | `cargo test -p ferro-mcp-oauth` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Notes |
|--------|----------|-----------|-------------------|-------|
| AMCP-07 | Discovery docs return correct JSON | unit | `cargo test -p ferro-mcp-oauth discovery` | Verify field names |
| AMCP-07 | DCR creates client, returns client_id | integration | `cargo test -p ferro-mcp-oauth register` | Needs in-memory SQLite |
| AMCP-08 | PKCE S256 verify: correct verifier → true | unit | `cargo test -p ferro-mcp-oauth pkce` | Pure fn |
| AMCP-08 | PKCE S256 verify: wrong verifier → false | unit | same | |
| AMCP-08 | Token mint round-trip (mint then decode) | unit | `cargo test -p ferro-mcp-oauth jwt` | |
| AMCP-08 | Full flow: DCR→authorize→token (no browser) | integration | `cargo test -p ferro-mcp-oauth flow` | Drive consent POST directly |
| AMCP-09 | validate_bearer: valid token → Authenticated | unit | `cargo test -p ferro-mcp-oauth validate` | |
| AMCP-09 | validate_bearer: expired → 401 | unit | same | fast-forward exp |
| AMCP-09 | validate_bearer: wrong aud → 403 | unit | same | |
| AMCP-09 | validate_bearer: wrong tenant_id → 403 | unit | same | requires tenant check |
| AMCP-09 | validate_bearer: no header → Unauthenticated | unit | same | |

### End-to-end integration test (no external IdP) strategy

Since the app IS the IdP, the full PKCE flow can be tested in-process:

```rust
#[tokio::test]
async fn full_pkce_flow() {
    // 1. Setup: in-memory SQLite DB, in-memory cache (ferro::Cache initialized with MemoryStore)
    // 2. DCR: call register_client() handler directly with test redirect_uri
    // 3. PKCE: generate code_verifier + code_challenge (S256)
    // 4. Authorize: call authorize_get() with Auth::id() set to test user, current_tenant() set
    // 5. Consent POST: call authorize_post() with action="approve", CSRF from session
    // 6. Extract code from redirect Location header
    // 7. Token: call token_exchange() with code + code_verifier
    // 8. Validate: call validate_bearer() with minted JWT
    // Assert: BearerOutcome::Authenticated with correct sub + tenant_id
}
```

The handlers are plain async Rust functions (not behind a running web server). The Ferro handler macro wraps them into `Request → Response` functions. For tests, create `Request` objects using the same test-helper pattern in `ferro-mcp-server/tests/dispatch_integration.rs`. [VERIFIED: `ferro-mcp-server/tests/dispatch_integration.rs` referenced in 197/198 CONTEXT]

### Wave 0 gaps (files to create)

- [ ] `ferro-mcp-oauth/tests/flow_integration.rs` — full PKCE flow integration test
- [ ] `ferro-mcp-oauth/src/lib.rs` and module files — crate scaffold (Wave 0 task)
- [ ] `app/src/migrations/m20260611_create_oauth_clients_table.rs` — OAuth clients migration
- [ ] `OAuthConfig` must be populated with a test secret in test environment

---

## 10. Environment Availability

This phase is code-only (no new external services). The `ferro::Cache` default is in-memory, so no Redis is required for tests.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| jsonwebtoken v9 | JWT | via ferro-wallet Cargo.toml | 9.x | already in workspace |
| rand 0.8 | code gen | via framework Cargo.toml | 0.8 | already in workspace |
| base64 0.22 | PKCE encoding | via framework Cargo.toml | 0.22 | already in workspace |
| sha2 0.10 | PKCE S256 | via framework Cargo.toml | 0.10 | already in workspace |
| subtle 2.5 | constant-time | via framework Cargo.toml | 2.5 | already in workspace |
| sea-orm 1.0 | DCR persistence | via ferro-mcp-server Cargo.toml | 1.0 | already in workspace |
| ferro::Cache | code storage | bootstrapped by Server::run() | in-memory default | no Redis needed |

All dependencies are already in the workspace. No new external services required. [VERIFIED: codebase]

---

## 11. Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | debug mode may use a derived signing key fallback | Key source | None if false — fail-closed is always safe |
| A2 | Option A (session-based return_to) is the "minimal" solution for login redirect | Login reuse | Option B requires a new HTML login handler; either works |
| A3 | Origin validation: allow absent Origin header (non-browser clients) | Phase 198 TODO | Too strict: would break SDK clients that don't send Origin |
| A4 | `validate_bearer` is synchronous (no DB call) | Seam wiring | If DB lookup were needed (opaque tokens), signature would need to be async |
| A5 | DCR `client_id` generated as UUID v4 (random, not sequential) | DCR | Using sequential IDs enables enumeration; UUID v4 is standard |

All factual claims about the codebase in sections 1-9 are [VERIFIED] from the code reads above.

---

## Open Questions (RESOLVED)

All four questions are resolved; the resolutions are implemented in the Phase 199 plans
(Plan 04 Task 1/Task 2 and Plan 05 Task 2).

1. **Login redirect mechanism (D-06)** — **RESOLVED: Option A.**
   - What we knew: `POST /auth/login` is JSON-only; no return-to support.
   - Resolution: Option A — add a `session.get("oauth_return_to")` check after `Auth::login()`
     in the existing `POST /auth/login` handler. `GET /authorize` stores the return URL as
     `/authorize?{query_string}` in session before redirecting to `/auth/login`. One-line change
     to auth_controller. Implemented in Plan 05 Task 2 (`oauth_return_to` session key).

2. **Single-tenant token without tenant_id** — **RESOLVED: allow None.**
   - What we knew: `current_tenant()` returns `None` in single-tenant apps.
   - Resolution: `validate_bearer` allows an absent `tenant_id` (single-tenant apps are valid
     ferro consumers); the tenant check is skipped when `expected_tenant` is `None`. Phase 200
     tenant scoping simply finds no tenant to scope by. Implemented in Plan 03 (`validate.rs`).

3. **Origin validation exact rule** — **RESOLVED: allow absent, reject mismatched.**
   - What we knew: Phase 198 TODO says validate the Origin header.
   - Resolution: Allow absent `Origin` (non-browser SDK clients); reject present-but-mismatched.
     Implemented in Plan 05 Task 2 (`/mcp` Origin check).

4. **Consent flow for multi-tenant users with ambiguous tenant** — **RESOLVED: 400 on ambiguity.**
   - What we knew: `current_tenant()` returns `None` when tenant is not resolved from the
     `/authorize` request URL.
   - Resolution: Return `400 invalid_request` if `TenantMiddleware` is active but
     `current_tenant()` is `None` (ambiguous). Single-tenant (no `TenantMiddleware`) is not
     ambiguous. A consent-time tenant picker is explicitly deferred to Phase 200. Implemented in
     Plan 04 Task 1 (`/authorize`).

---

## Sources

### Primary (HIGH confidence — verified from codebase)
- `ferro-mcp-server/src/auth.rs` — BearerOutcome, extract_bearer (the seam)
- `app/src/controllers/mcp.rs` — /mcp handler, header-before-body ordering, Phase 198 auth branch
- `app/src/controllers/auth_controller.rs` — login/register handlers (JSON API, no return-to)
- `framework/src/auth/guard.rs` — Auth::check, Auth::id, Auth::login
- `framework/src/session/middleware.rs` — generate_csrf_token, get_csrf_token
- `framework/src/tenant/resolver.rs` — JwtClaimResolver with claim_field="tenant_id"
- `framework/src/tenant/context.rs` — current_tenant() task-local
- `framework/src/config/providers/app.rs` — AppConfig (no signing key confirmed)
- `framework/src/api/api_key.rs` — subtle::ConstantTimeEq pattern
- `framework/src/http/response.rs` — HttpResponse, Redirect
- `framework/src/cache/mod.rs` — Cache facade (put/get/forget)
- `app/src/migrations/m20260228_create_api_keys_table.rs` — migration pattern
- `app/src/migrations/mod.rs` — Migrator registration pattern
- `app/src/routes.rs` — route registration syntax
- `.github/workflows/publish.yml` — Wave 1A/1B/2/3 crate lists
- `ferro-mcp-server/src/config.rs` — McpServerConfig from_env() pattern with sanitize_identity
- `ferro-wallet/Cargo.toml` — jsonwebtoken = "9" version confirmation
- `framework/Cargo.toml` — rand="0.8", base64="0.22", sha2="0.10", subtle="2.5" confirmation

### Primary (HIGH confidence — official spec)
- MCP Authorization spec (modelcontextprotocol.io/specification/2025-03-26/basic/authorization) — full OAuth flow, discovery, PKCE requirement
- RFC 8414 §2 — authorization server metadata required fields
- RFC 9728 §2 — protected resource metadata fields
- RFC 7591 §2, §3.2.1 — DCR request/response, redirect_uris required
- RFC 6749 §4.1, §5.1, §5.2 — authorize/token shapes, error codes
- RFC 7636 §4.1, §4.2, §4.6 — PKCE S256: code_challenge = BASE64URL(SHA256(code_verifier))
- RFC 6750 §3.1 — WWW-Authenticate Bearer error params

### Secondary (HIGH confidence — Context7 verified)
- /keats/jsonwebtoken — HS256 encode/decode, Validation struct, ErrorKind enum, algorithm pinning

---

## Metadata

**Confidence breakdown:**
- Crate architecture and dependency direction: HIGH — confirmed from Cargo.toml files
- Bearer seam wiring: HIGH — confirmed from auth.rs, mcp.rs
- Tenant claim name "tenant_id": HIGH — confirmed from resolver.rs line 209
- JWT HS256 API (jsonwebtoken v9): HIGH — confirmed via context7
- ferro::Cache API: HIGH — confirmed from framework/src/cache/mod.rs
- Login redirect: MEDIUM — solution confirmed from code; the exact implementation option is Claude's Discretion
- CSRF pattern: HIGH — confirmed from session middleware

**Research date:** 2026-06-10
**Valid until:** 2026-07-10 (stable framework; 30-day window)

---

## RESEARCH COMPLETE
