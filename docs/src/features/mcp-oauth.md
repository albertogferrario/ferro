# MCP OAuth Authorization Server

`ferro-mcp-oauth` is a mountable crate that turns any Ferro application into an OAuth 2.1
authorization server for its MCP endpoint. It implements the authorization-code flow (RFC 6749
with PKCE, RFC 7636) and the device authorization grant (RFC 8628), backed by the same JWT
issuer so both flows produce tokens with identical audience binding and tenant scoping.

## Quick Start

Add `ferro-mcp-oauth` to your `Cargo.toml` and mount the handlers in `app/src/routes.rs`:

```rust
use ferro_mcp_oauth::handlers::{
    authorization_server_handler, authorize_get, authorize_post,
    device_authorization, device_verification_get, device_verification_post,
    protected_resource_handler, register_client, token_exchange,
};

routes! {
    // OAuth discovery (public)
    get!("/.well-known/oauth-protected-resource", protected_resource_handler),
    get!("/.well-known/oauth-authorization-server", authorization_server_handler),

    // Authorization-code flow (session + tenant)
    group!("/", {
        get!("/authorize", authorize_get),
        post!("/authorize", authorize_post),
    }).middleware(
        TenantMiddleware::new()
            .resolver(SessionUserTenantResolver::new())
            .on_failure(TenantFailureMode::Allow),
    ),

    // Dynamic Client Registration + token exchange (public)
    post!("/register", register_client),
    post!("/token", token_exchange),

    // Device Authorization Grant (public — no session)
    post!("/device_authorization", device_authorization),

    // Device verification page (session + tenant, Allow so unauthenticated visitors reach login)
    group!("/", {
        get!("/device", device_verification_get),
        post!("/device", device_verification_post),
    }).middleware(
        TenantMiddleware::new()
            .resolver(SessionUserTenantResolver::new())
            .on_failure(TenantFailureMode::Allow),
    ),
}
```

Run the database migration once:

```rust
use ferro_mcp_oauth::CreateOauthClientsTable;

// In your bootstrap or migration runner
CreateOauthClientsTable::up(&db).await?;
```

## Configuration

| Variable | Description |
|----------|-------------|
| `MCP_TOKEN_SECRET` | HMAC-SHA256 secret for signing JWTs. Required. |
| `APP_URL` | Base URL used in discovery metadata and token audience. Required. |

## Authorization-Code Flow

The standard browser-based OAuth 2.1 flow with PKCE:

1. Client registers via `POST /register` (Dynamic Client Registration, RFC 7591).
2. Client redirects the browser to `GET /authorize` with `response_type=code`,
   `client_id`, `redirect_uri`, `code_challenge` (S256), and `state`.
3. If the user is unauthenticated, the handler stores the in-flight URL and redirects to
   the app login page. After authentication the login handler calls `oauth_resume_redirect`
   to return to the authorize endpoint.
4. Authenticated user sees a consent page; on approval an authorization code is issued
   and the browser is redirected to `redirect_uri` with `code` and `state`.
5. Client exchanges the code at `POST /token` with `grant_type=authorization_code` and
   `code_verifier` (PKCE proof).

## Device Authorization Grant (RFC 8628)

The device grant is the MCP auth path for clients that cannot complete a same-device browser
redirect: headless CLI tools, cross-device logins, and passwordless flows.

### Step 1 — Request a device code

The device (CLI, daemon, or headless client) sends its `client_id` to the public endpoint:

```
POST /device_authorization
Content-Type: application/x-www-form-urlencoded

client_id=<registered_client_id>
```

Response (RFC 8628 §3.2):

```json
{
  "device_code": "<opaque high-entropy string>",
  "user_code": "BCDF-GHJK",
  "verification_uri": "https://your-app.example.com/device",
  "verification_uri_complete": "https://your-app.example.com/device?user_code=BCDF-GHJK",
  "expires_in": 600,
  "interval": 5
}
```

| Field | Description |
|-------|-------------|
| `device_code` | Opaque polling credential. Never shown to the user. |
| `user_code` | Short human-typeable code (`XXXX-XXXX`, RFC 8628 §6.1 charset). |
| `verification_uri` | URL the user opens on any browser to complete authorization. |
| `verification_uri_complete` | Same URL with `user_code` pre-filled (suitable for QR codes). |
| `expires_in` | Grant TTL in seconds (600 s / 10 min). |
| `interval` | Minimum polling interval in seconds (5 s). |

### Step 2 — User authorizes on any device

The user opens `verification_uri` (or scans the QR code for `verification_uri_complete`) on
any browser — a phone, tablet, or desktop — and follows the flow:

1. If unauthenticated: redirected to the app login page via the Phase 202 resume contract;
   returns to `/device` automatically after authentication.
2. Enters or confirms the `user_code`.
3. Sees the consent page (same approve/deny UI as the authorization-code flow).
4. On approval: `user_id` and `tenant_id` are captured from the session and bound to the
   grant. The device receives an `access_token` on the next poll.

### Step 3 — Poll for the token

While the user completes Step 2, the device polls `POST /token` at intervals ≥ `interval`
seconds:

```
POST /token
Content-Type: application/x-www-form-urlencoded

grant_type=urn:ietf:params:oauth:grant-type:device_code&device_code=<device_code>&client_id=<client_id>
```

Response codes (RFC 8628 §3.5):

| HTTP | `error` | Meaning |
|------|---------|---------|
| 200 | — | Token issued; `access_token`, `token_type`, `expires_in` present. |
| 400 | `authorization_pending` | User has not yet approved. Continue polling. |
| 400 | `slow_down` | Poll interval too short; add 5 s to the current interval. |
| 400 | `access_denied` | User denied the request. Stop polling. |
| 400 | `expired_token` | Grant TTL elapsed or `device_code` unknown. Restart the flow. |

### Token identity

Tokens issued by the device grant are minted by the same `jwt.rs` path as the
authorization-code grant, with identical `sub`, `aud`, `iss`, and `tenant_id` claims.
There is one token issuer; the device grant is an alternate entry point, not a parallel path.

## Discovery Metadata

Both flows are advertised in the RFC 8414 authorization-server metadata:

```
GET /.well-known/oauth-authorization-server
```

```json
{
  "issuer": "https://your-app.example.com",
  "authorization_endpoint": "https://your-app.example.com/authorize",
  "token_endpoint": "https://your-app.example.com/token",
  "registration_endpoint": "https://your-app.example.com/register",
  "device_authorization_endpoint": "https://your-app.example.com/device_authorization",
  "response_types_supported": ["code"],
  "grant_types_supported": [
    "authorization_code",
    "urn:ietf:params:oauth:grant-type:device_code"
  ],
  "code_challenge_methods_supported": ["S256"],
  "token_endpoint_auth_methods_supported": ["none"]
}
```

## Validating Bearer Tokens

At the MCP endpoint, call `validate_bearer` to verify and decode incoming tokens:

```rust
use ferro_mcp_oauth::{validate_bearer, BearerCheck};

match validate_bearer(&req) {
    BearerCheck::Valid(claims) => {
        // claims.sub  — user id
        // claims.tenant_id — tenant id (None for single-tenant apps)
    }
    BearerCheck::Missing => { /* 401 */ }
    BearerCheck::Invalid(reason) => { /* 401 with reason */ }
}
```

## Security Notes

- **CSRF**: The consent and device-verification POST handlers use constant-time comparison
  (`subtle::ConstantTimeEq`) on session CSRF tokens.
- **Tenant binding**: `tenant_id` is captured from the session at consent/approval time, not
  from form input. The `TenantFailureMode::Allow` mode ensures unauthenticated visitors reach
  the login redirect rather than a 403.
- **Single-use codes**: Authorization codes and device codes are forgotten from cache
  immediately after a successful token exchange (get-then-forget discipline).
- **Device code entropy**: `device_code` is a 256-bit URL-safe random string (same entropy as
  PKCE authorization codes). `user_code` uses the RFC 8628 §6.1 unambiguous consonant charset
  (`BCDFGHJKLMNPQRSTVWXZ`) and is accepted case-insensitively with or without the hyphen.
- **Rate limiting**: The polling `slow_down` response enforces a minimum interval between polls.
  Additional rate limiting on `POST /device_authorization` is not implemented; apply an upstream
  reverse-proxy limit if needed.
