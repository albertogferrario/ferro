# Phase 203: OAuth Device Authorization Grant (RFC 8628) - Research

**Researched:** 2026-06-11
**Domain:** `ferro-mcp-oauth` — OAuth 2.0 Device Authorization Grant (RFC 8628)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `DeviceGrant` stored in `ferro-cache`, not a DB table. Two cache keys:
  `mcp:device:{device_code}` (full record) and `mcp:usercode:{user_code}` → `device_code`
  (pointer). Fields: `client_id`, `status` (Pending|Approved|Denied), `user_id: Option<i64>`,
  `tenant_id: Option<i64>`, `created_at: i64`, `last_polled_at: Option<i64>`. TTL ~600s (10 min).

- **D-02:** `user_code` from RFC 8628 §6.1 charset (`BCDFGHJKLMNPQRSTVWXZ`), 8 chars grouped
  `XXXX-XXXX`. Verification normalizes case and strips hyphen/whitespace. `device_code` is
  high-entropy URL-safe random (like auth code in `pkce::generate_auth_code`), never shown.

- **D-03:** Verification page is **raw HTML** inside `ferro-mcp-oauth`, mirroring `consent.rs`.
  Two states on `GET /device`: code-entry form (no valid `user_code`) and confirm+consent
  (valid `user_code`, authenticated user). No JSON-UI dependency.

- **D-04:** `verification_uri = {app_url}/device`, `verification_uri_complete =
  {app_url}/device?user_code={user_code}`. Unauthenticated → `store_oauth_return_to` + redirect
  to `/auth/login`. Consent + `Auth::id()` + `current_tenant()` capture on POST. Mounts with
  `SessionUserTenantResolver` TenantMiddleware (`TenantFailureMode::Allow`).

- **D-05:** Token endpoint branching on `grant_type`. New arm:
  `urn:ietf:params:oauth:grant-type:device_code`. Default interval = 5s. Expiry = 600s.
  State transitions: missing/expired→`expired_token`, Pending→`authorization_pending`,
  Pending+fast poll→`slow_down` (bump interval +5s), Denied→`access_denied`,
  Approved→mint JWT via existing `jwt.rs` then forget both cache keys.

- **D-06:** `POST /device_authorization` (public). Client validated via `find_by_client_id`.
  No PKCE. Discovery adds `device_authorization_endpoint` and
  `urn:ietf:params:oauth:grant-type:device_code` to `grant_types_supported`.

### Claude's Discretion

- Module split (`device.rs` vs separate files), handler names, `handlers` re-export shape.
- Exact `device_code` length/encoding, precise TTL/interval values (within RFC guidance).
- Whether `user_code` entry stores only a pointer or a copy of the record.
- Verification-page copy and terminal page wording.
- Whether `slow_down` is enforced strictly (reject) or advisory (return code, let client self-correct).
- Test file layout (integration vs in-module unit tests).

### Deferred Ideas (OUT OF SCOPE)

- Rate-limiting `POST /device_authorization` beyond RFC polling controls.
- Refresh tokens.
- Consumer adoption (gestiscilo making device grant its primary MCP auth path).
- Polling-throttle / exponential backoff beyond a single `slow_down`.
- QR-code rendering of `verification_uri_complete`.

</user_constraints>

---

## Summary

Phase 203 adds the OAuth 2.0 Device Authorization Grant (RFC 8628) to `ferro-mcp-oauth` as
an alternate front door to the existing token issuer. The grant enables passwordless,
cross-device, and headless/CLI MCP clients to authenticate without a same-device browser
callback. Three moving parts share all existing surfaces: (1) a public `POST /device_authorization`
endpoint that issues `device_code`/`user_code` stored in `ferro-cache`; (2) a verification page
at `GET /device` that reuses the Phase 202 login-resume contract and the existing consent screen;
(3) a new arm in `POST /token` that polls the cache state machine and mints the identical JWT via
`jwt.rs` when approved.

The implementation is a net extension of existing patterns. `store.rs` gets a `DeviceGrant`
struct alongside `OAuthCode`. `token.rs` gets a second `grant_type` branch. `discovery.rs` gets
two new metadata fields. A new `device.rs` module owns the three handler functions. Route wiring
in `app/src/routes.rs` follows the established public-vs-session-group pattern already in use
for `/token` vs `/authorize`.

**Primary recommendation:** Implement `device.rs` as a self-contained module holding `DeviceGrant`
struct + cache helpers + three handler functions. Extend `token.rs`, `discovery.rs`, and
`lib.rs::handlers` to wire it in. The entire implementation reuses existing `ferro-mcp-oauth`
building blocks without new dependencies.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `POST /device_authorization` | API / Backend (crate handler, public) | — | Stateless credential issuance; no session needed; mirrors `/register` mount |
| `GET /device` verification page | API / Backend (crate handler, session group) | — | Must read session (CSRF, `Auth::id()`, `current_tenant()`) |
| `POST /device` approve/deny | API / Backend (crate handler, session group) | — | Writes cache state; captures user/tenant from session at approve time |
| `POST /token` device-code arm | API / Backend (crate handler, public) | — | Polls cache; mints JWT; same endpoint as auth-code arm |
| `DeviceGrant` state machine | Database / Storage (`ferro-cache`, ephemeral) | — | Ephemeral credential pattern (D-01); not DB |
| Discovery metadata | API / Backend (crate handler, public) | — | Extends existing `discovery.rs` |
| JWT minting | API / Backend (`jwt.rs`, shared) | — | Single token issuer invariant; device arm calls identical path |

---

## RFC 8628 Wire Contract

[CITED: https://www.rfc-editor.org/rfc/rfc8628]

### §3.1 Device Authorization Request

`POST /device_authorization` — `application/x-www-form-urlencoded`

| Parameter | Required | Value |
|-----------|----------|-------|
| `client_id` | REQUIRED | registered client id |
| `scope` | OPTIONAL | ignored in this phase (single implicit scope) |

Unknown `client_id` → respond `400 {"error": "invalid_client"}`.

### §3.2 Device Authorization Response — Exact Field Names

`200 application/json` response body. All fields listed here must appear with **exactly** these names (MCP clients branch on them):

| Field | Type | Required | Value / Semantics |
|-------|------|----------|-------------------|
| `device_code` | string | REQUIRED | High-entropy server-side credential; never shown to user |
| `user_code` | string | REQUIRED | Short human-typeable code, e.g. `"WDJB-MFXG"` |
| `verification_uri` | string | REQUIRED | `{app_url}/device` |
| `verification_uri_complete` | string | OPTIONAL (send it) | `{app_url}/device?user_code={user_code}` (clickable; pre-fills code) |
| `expires_in` | integer (seconds) | REQUIRED | 600 (D-01 TTL) |
| `interval` | integer (seconds) | OPTIONAL (send it) | 5 (D-05 default) |

### §3.4 Token Request — Device-Code Arm

`POST /token` — `application/x-www-form-urlencoded`

| Parameter | Required | Value |
|-----------|----------|-------|
| `grant_type` | REQUIRED | `urn:ietf:params:oauth:grant-type:device_code` (verbatim) |
| `device_code` | REQUIRED | opaque string returned by `POST /device_authorization` |
| `client_id` | REQUIRED | same `client_id` from step 1 |

### §3.5 Token Error Codes — Exact Strings

`400 application/json` with `{"error": "<code>"}`. These strings are the contract; grep targets for SC-5 tests:

| Error String | Condition | Client Behavior |
|---|---|---|
| `"authorization_pending"` | Grant is Pending; within polling interval | Client retries after `interval` seconds |
| `"slow_down"` | Grant is Pending; polled faster than `interval` | Client **MUST** add 5 seconds to its polling interval permanently for this session |
| `"access_denied"` | User denied consent (`Denied` state) | Terminal; client stops polling |
| `"expired_token"` | `device_code` TTL elapsed or grant not found in cache | Terminal; restart flow |

Successful response (grant is Approved):
```json
{"access_token": "<jwt>", "token_type": "Bearer", "expires_in": 3600}
```

### `slow_down` Rule (RFC §3.5 verbatim semantics)

When a client polls faster than `interval` seconds:
1. Return `{"error": "slow_down"}` immediately (do NOT advance to `authorization_pending`).
2. The client's *new* minimum interval = previous interval + 5 seconds, applied to **all subsequent requests in this session**.
3. The server-side enforcement: compare `last_polled_at` (Unix seconds) on the stored record against `SystemTime::now()`. If `now - last_polled_at < interval`, return `slow_down`. Always update `last_polled_at` on each poll regardless.

### §4 Discovery Metadata Key

Add to `authorization_server_metadata()` in `discovery.rs`:

```json
{
  "device_authorization_endpoint": "{app_url}/device_authorization",
  "grant_types_supported": ["authorization_code", "urn:ietf:params:oauth:grant-type:device_code"]
}
```

Field name is `device_authorization_endpoint` (verbatim, RFC §4).

### §6.1 User-Code Recommended Charset

Verbatim charset: **`BCDFGHJKLMNPQRSTVWXZ`** — 20 uppercase consonants, no vowels, no digits.
Rationale: avoids profanity (no vowels), avoids visual confusion (no `0`/`O`, `1`/`I`), works
across keyboard layouts.

Format: 8 characters, grouped `XXXX-XXXX` with a hyphen (e.g. `WDJB-MFXG`).
Normalization at verification: uppercase, strip hyphens and whitespace before cache lookup.

Generation: select 8 characters uniformly at random from the 20-char charset using
`rand::thread_rng()` (same dependency already in `ferro-mcp-oauth`'s `Cargo.toml`).

Keyspace: 20^8 = 25,600,000,000 combinations. With 600s TTL, brute-force is infeasible
assuming rate-limiting at the verification page (or even without it given single-use once
validated). [CITED: RFC 8628 §5 — "user code SHOULD have enough entropy that guessing it via
a brute-force attack becomes infeasible"]

### §5 Security Considerations

| Threat | RFC Guidance | Implementation |
|--------|-------------|----------------|
| `user_code` brute-force | High entropy + short TTL | 20-char charset, 8 chars → 2.56×10^10 space; 600s TTL; single-use after approval |
| `device_code` entropy | Very high entropy, never shown | `generate_auth_code()` returns 256 bits of entropy (32 random bytes, base64url-encoded) |
| Phishing via `verification_uri` | Must be short and easy to type | `{app_url}/device` — short path; user types at a browser, not a CLI |
| Device polling abuse | `slow_down` + `expired_token` | `last_polled_at` check; 600s TTL enforced by cache |

---

## Standard Stack

All dependencies are already present in `ferro-mcp-oauth/Cargo.toml`. **No new crate dependencies are required.** [VERIFIED: ferro-mcp-oauth/Cargo.toml]

| Dependency | Already Present | Usage in Phase 203 |
|---|---|---|
| `ferro` (ferro-rs) | yes | `Cache::put/get/forget`, `Auth::id()`, `current_tenant()`, `session::get_csrf_token()`, `ferro::handler` macro |
| `rand = "0.8"` | yes | `device_code` generation (`generate_auth_code()` reuse) and `user_code` charset sampling |
| `serde` / `serde_json` | yes | `DeviceGrant` serialization |
| `subtle = "2.5"` | yes | CSRF constant-time comparison |
| `thiserror` | yes | No new variants needed; existing `OAuthError` may be extended |
| `chrono` | yes | `created_at` timestamp |

**Installation:** No new `cargo add` commands required.

---

## Architecture Patterns

### System Architecture Diagram

```
CLI/Device                  User Browser              ferro-mcp-oauth crate
    │                            │                            │
    │ POST /device_authorization │                            │
    │───────────────────────────────────────────────────────>│
    │                            │          validate client_id (find_by_client_id)
    │                            │          generate device_code (generate_auth_code)
    │                            │          generate user_code (random from charset)
    │                            │          Cache::put(mcp:device:{device_code}, DeviceGrant{Pending})
    │                            │          Cache::put(mcp:usercode:{user_code}, device_code)
    │<──────────────────────────────────────────────────────(device_code, user_code,
    │  {device_code, user_code,  │                            verification_uri, expires_in, interval)
    │   verification_uri, ...}   │
    │                            │
    │ [poll loop]                │ GET /device?user_code=XXXX-XXXX
    │ POST /token                │───────────────────────────>│
    │  grant_type=device_code    │                  if !Auth::check():
    │  device_code=...           │                    store_oauth_return_to("/device?user_code=...")
    │───────────────────────────>│                    → 302 /auth/login
    │                            │<──────────────────(302 /auth/login)
    │                            │
    │ {error:                    │ [user logs in via existing login flow]
    │  authorization_pending}    │ GET /device?user_code=XXXX-XXXX (resume)
    │<───────────────────────────│───────────────────────────>│
    │                            │                  resolve user_code → device_code
    │ [wait interval seconds]    │                  look up DeviceGrant
    │                            │                  render confirm+consent HTML
    │                            │<──────────────────(consent HTML)
    │                            │
    │                            │ POST /device {action=approve, _token=...}
    │                            │───────────────────────────>│
    │                            │                  CSRF validate
    │                            │                  user_id = Auth::id()
    │                            │                  tenant_id = current_tenant().map(|t| t.id)
    │                            │                  Cache::put(mcp:device:{device_code},
    │                            │                    DeviceGrant{Approved, user_id, tenant_id})
    │                            │<──────────────────(terminal "return to device" page)
    │
    │ POST /token (next poll)
    │───────────────────────────────────────────────────────>│
    │                            │          Cache::get(mcp:device:{device_code})
    │                            │          → Approved
    │                            │          Cache::forget(mcp:device:{...})
    │                            │          Cache::forget(mcp:usercode:{...})
    │                            │          build_claims(user_id, tenant_id, app_url, 3600)
    │                            │          mint_token(&claims, &config.token_secret)
    │<──────────────────────────────────────────────────────(access_token, token_type, expires_in)
```

### Recommended Project Structure (new files only)

```
ferro-mcp-oauth/src/
├── device.rs           # DeviceGrant struct, cache helpers, 3 handler fns
└── (existing files unchanged except token.rs, discovery.rs, lib.rs)
```

`device.rs` is one module that owns both the store-type and the handlers (analogous to
`store.rs` housing `OAuthCode` alongside the SeaORM entity). Splitting into
`device_authorization.rs` / `device_verify.rs` is valid but introduces unnecessary file-count
overhead for what is a single cohesive feature.

### Pattern 1: DeviceGrant Record

```rust
// ferro-mcp-oauth/src/device.rs
// Source: mirrors OAuthCode in store.rs [VERIFIED: ferro-mcp-oauth/src/store.rs]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceGrantStatus {
    Pending,
    Approved,
    Denied,
}

/// Ephemeral device grant stored in `ferro-cache` with ~600s TTL.
///
/// Two cache keys:
/// - `mcp:device:{device_code}` → full `DeviceGrant` record
/// - `mcp:usercode:{user_code}` → `device_code` string (pointer)
///
/// Status transitions: Pending → Approved (on user consent) | Denied (on user deny).
/// `user_id` and `tenant_id` are `None` until the Approved transition.
/// `last_polled_at` is updated on every token poll for `slow_down` enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceGrant {
    pub client_id: String,
    pub status: DeviceGrantStatus,
    pub user_id: Option<i64>,
    pub tenant_id: Option<i64>,
    pub created_at: i64,          // Unix seconds
    pub last_polled_at: Option<i64>, // Unix seconds; None = never polled
}
```

### Pattern 2: Cache Put-Overwrite for State Transitions

`ferro-cache` has no in-place update. State transitions (Pending→Approved) are done via
get→modify→put with the same key and the same TTL:

```rust
// Source: mirrors token.rs Cache::get/forget pattern [VERIFIED: ferro-mcp-oauth/src/token.rs]
// and consent.rs Cache::put pattern [VERIFIED: ferro-mcp-oauth/src/consent.rs]

const DEVICE_CODE_TTL: Duration = Duration::from_secs(600);
const DEVICE_INTERVAL_SECS: i64 = 5;

pub async fn approve_device_grant(
    device_code: &str,
    user_id: i64,
    tenant_id: Option<i64>,
) -> Result<(), ferro::CacheError> {
    let key = format!("mcp:device:{device_code}");
    let mut grant: DeviceGrant = Cache::get(&key)
        .await?
        .ok_or(ferro::CacheError::Miss)?;   // or map to handler error
    grant.status = DeviceGrantStatus::Approved;
    grant.user_id = Some(user_id);
    grant.tenant_id = Some(tenant_id);
    // Re-put with the same key. TTL is refreshed from now.
    // Remaining TTL is not preserved — acceptable: approval happens well within 600s.
    Cache::put(&key, &grant, Some(DEVICE_CODE_TTL)).await
}
```

Key insight: `Cache::put` on an existing key replaces the record and resets the TTL. This is
correct here; an approved grant that gets replaced with a fresh 600s TTL window is fine —
the polling client will consume it within seconds. [VERIFIED: ferro-mcp-oauth/src/consent.rs lines 220-232 show the put pattern; token.rs lines 62-64 show get+forget]

### Pattern 3: Token Endpoint Grant-Type Branching

Current `token.rs` has a hard-reject on anything but `authorization_code` (line 50-55). The
device arm is added before the `return Err(...)`:

```rust
// Source: token.rs structure [VERIFIED: ferro-mcp-oauth/src/token.rs]

// ── Step 2: grant_type dispatch ───────────────────────────────────────────────
match form.grant_type.as_str() {
    "authorization_code" => {
        // existing arm — unchanged
        token_exchange_auth_code(form).await
    }
    "urn:ietf:params:oauth:grant-type:device_code" => {
        token_exchange_device_code(form, config).await
    }
    _ => Err(json_error(400, "unsupported_grant_type", "unsupported grant_type")),
}
```

`TokenRequest` needs two new optional fields (`device_code: Option<String>`) since the device
arm uses `device_code` instead of `code`/`redirect_uri`/`code_verifier`. Use `#[serde(default)]`
on optional fields.

### Pattern 4: JWT Minting — Identical Call Shape

The device arm calls `build_claims` + `mint_token` with the **same signature** as the
auth-code arm. A diff of the two arms must show the same call:

```rust
// Source: token.rs lines 99-101 [VERIFIED: ferro-mcp-oauth/src/token.rs]
// Auth-code arm:
let claims = build_claims(record.user_id, record.tenant_id, &config.app_url, 3600);
let access_token = mint_token(&claims, &config.token_secret)
    .map_err(|e| json_error(500, "server_error", &format!("token mint error: {e}")))?;

// Device arm (MUST be identical):
let claims = build_claims(grant.user_id.unwrap(), grant.tenant_id, &config.app_url, 3600);
let access_token = mint_token(&claims, &config.token_secret)
    .map_err(|e| json_error(500, "server_error", &format!("token mint error: {e}")))?;
```

`McpTokenClaims` fields: `sub` (user_id as string), `tenant_id: Option<i64>`, `aud: Vec<String>`
(`["{app_url}/mcp"]`), `iss` (`app_url`), `iat`, `exp`. Device tokens are claims-identical to
auth-code tokens. [VERIFIED: ferro-mcp-oauth/src/jwt.rs lines 22-61]

### Pattern 5: Verification Page — Unauth Redirect (mirrors authorize.rs)

```rust
// Source: authorize.rs lines 87-102 [VERIFIED: ferro-mcp-oauth/src/authorize.rs]

// In device_verification_get handler:
if !Auth::check() {
    let return_url = format!(
        "/device?user_code={}",
        urlencoding::encode(&user_code_param),
    );
    crate::resume::store_oauth_return_to(return_url);
    return Err(ferro::HttpResponse::new()
        .status(302)
        .header("Location", "/auth/login"));
}
```

After authentication, `auth_controller::verify_magic_link` calls `oauth_resume_redirect("/")`
which reads `oauth_return_to` from the session and returns `302 /device?user_code=...`, landing
back at the verification page. [VERIFIED: app/src/controllers/auth_controller.rs line 209]

### Pattern 6: CSRF on Verification POST (mirrors consent.rs)

```rust
// Source: consent.rs lines 124-141 [VERIFIED: ferro-mcp-oauth/src/consent.rs]

let session_csrf = get_csrf_token().ok_or_else(|| {
    ferro::HttpResponse::text("<h1>Error</h1>").status(400)
})?;
let csrf_ok: bool = form.token.as_bytes().ct_eq(session_csrf.as_bytes()).into();
if !csrf_ok {
    return Err(ferro::HttpResponse::text("<h1>CSRF error</h1>").status(400));
}
```

The verification form HTML must include `<input type="hidden" name="_token" value="{csrf_token}">`.
`device_code` must also be a hidden field so the POST handler knows which grant to approve,
without re-exposing it to URL parameters.

### Pattern 7: Discovery Extension (mirrors discovery.rs)

```rust
// Source: discovery.rs lines 26-37 [VERIFIED: ferro-mcp-oauth/src/discovery.rs]

pub(crate) fn authorization_server_metadata(app_url: &str) -> Value {
    json!({
        // ... existing fields ...
        "grant_types_supported": [
            "authorization_code",
            "urn:ietf:params:oauth:grant-type:device_code",
        ],
        "device_authorization_endpoint": format!("{}/device_authorization", app_url),
    })
}
```

### Anti-Patterns to Avoid

- **Storing `DeviceGrant` in DB:** Contradicts D-01 and the established ephemeral-credential
  pattern. TTL + single-use are cache primitives; a DB would need a reaper job.
- **Adding `user_code` data to the `device_code` cache key or vice versa redundantly:** The
  `mcp:usercode:{user_code}` entry stores only the `device_code` string (pointer), not a full
  `DeviceGrant` copy. This keeps state in one place (under `mcp:device:{device_code}`) and
  avoids split-brain on state transitions.
- **Minting a device-specific token or adding device-specific claims:** The device token is
  issued by `build_claims` + `mint_token` identically to the auth-code token. No new claims,
  no new `token_type`, no separate audience. One issuer.
- **Checking `grant_type` at the routing layer instead of inside `token_exchange`:** Keep the
  single `POST /token` endpoint; branch internally on `grant_type`.
- **Binding tenant at `POST /device_authorization` time:** The device is anonymous there.
  `tenant_id` is captured only at `POST /device` approval, when `current_tenant()` is
  meaningful. This mirrors the auth-code flow where tenant is captured at consent, not at
  `/authorize`.
- **Re-reading `user_code` from the form for the approve POST without a hidden `device_code`
  field:** The user_code entry in cache is a pointer; after approve, the flow needs the
  `device_code` to update the `mcp:device:{...}` entry. Embed `device_code` as a hidden field
  (not `user_code`) in the confirm+consent form.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| High-entropy random string | Custom base64 random | `pkce::generate_auth_code()` (already in crate) | 256 bits via `rand::thread_rng()`, URL-safe encoding — identical to auth codes |
| User-code random selection | Custom charset sampling | `rand::thread_rng().gen_range(0..20)` index into `CHARSET` const | Same `rand` dep; no new crate |
| CSRF constant-time compare | `==` string compare | `subtle::ConstantTimeEq` (already in crate) | Timing oracle prevention |
| HTML escaping | Custom regex | `authorize::html_escape()` (already in crate) | XSS prevention; already tested |
| JWT minting | New claims struct | `jwt::build_claims` + `jwt::mint_token` | One token issuer invariant |
| URL percent-encoding | Custom | `authorize::urlencoding::encode()` (already in crate) | Used for return_url construction |
| App URL construction | `env::var("APP_URL")` direct | `config::sanitized_app_url()` | Strips control chars (CR-01 analog) |

**Key insight:** The entire device grant reuses ~95% existing code. The three new handler
functions, the `DeviceGrant` struct, and two cache-key helpers are genuinely new; everything
else is a call into existing modules.

---

## slow_down Enforcement Mechanics

[CITED: RFC 8628 §3.5]

The `slow_down` response means: "you polled before the interval elapsed; increase your interval
by 5 seconds."

**Server-side implementation with cache-overwrite model:**

1. On each poll (`POST /token` device-code arm):
   a. `Cache::get("mcp:device:{device_code}")` → `DeviceGrant`.
   b. If grant is `Pending`:
      - Compute `elapsed = now_unix - last_polled_at.unwrap_or(created_at)`.
      - If `elapsed < DEVICE_INTERVAL_SECS (5)`:
        - Update `last_polled_at = now_unix` on the record.
        - `Cache::put("mcp:device:{device_code}", &updated_grant, Some(DEVICE_CODE_TTL))`.
        - Return `{"error": "slow_down"}`.
      - Else:
        - Update `last_polled_at = now_unix`.
        - `Cache::put(...)` (overwrite with updated timestamp).
        - Return `{"error": "authorization_pending"}`.
   c. If grant is `Approved`: proceed to mint (forget both keys).
   d. If grant is `Denied`: return `{"error": "access_denied"}`.
2. If `Cache::get` returns `None` (TTL elapsed or never existed): return `{"error": "expired_token"}`.

**Timing tolerance for tests:** The in-memory cache used by `bootstrap_test_cache()` does not
enforce TTL sub-second. Tests drive `slow_down` by setting `last_polled_at = now_unix` on the
record *before* calling the handler (simulate a recent poll), then calling the handler within
the same second. This is deterministic without `sleep`.

**Race condition note:** Concurrent polls from the same device could both read `Pending` and
both get past the `elapsed` check before either writes back `last_polled_at`. Given the
device-grant use case (single polling client), this is acceptable; true concurrent polling
from the same device is unusual and the RFC does not require atomic compare-and-swap. Flag as
a known limitation, not a blocking bug. [ASSUMED — the RFC does not specify atomic update
semantics for `slow_down`]

---

## Route Mounting + Middleware

[VERIFIED: app/src/routes.rs lines 70-98]

```rust
// In app/src/routes.rs — import new handlers
use ferro_mcp_oauth::handlers::{
    // ... existing ...
    device_authorization,          // new
    device_verification_get,       // new
    device_verification_post,      // new
};

routes! {
    // ... existing routes ...

    // Device Authorization — public (no session, like /register and /token)
    post!("/device_authorization", device_authorization),

    // Device verification page — session + tenant (like /authorize group)
    group!("/", {
        get!("/device", device_verification_get),
        post!("/device", device_verification_post),
    }).middleware(
        TenantMiddleware::new()
            .resolver(SessionUserTenantResolver::new())
            .on_failure(TenantFailureMode::Allow),
    ),

    // ... existing ...
}
```

Key points:
- `/device_authorization` mounts at the top level without middleware (public, like
  `/register`). D-06 says no session needed; `client_id` validated against DB.
- `/device` GET+POST mount in a `TenantMiddleware` group with `SessionUserTenantResolver` and
  `TenantFailureMode::Allow` — identical to the `/authorize` group (lines 71-78 of routes.rs).
  `Allow` is critical: an unauthenticated visit must reach the handler (not be rejected) so the
  handler can issue the login redirect.

`verification_uri` and `verification_uri_complete` are built from `sanitized_app_url()`:

```rust
let app_url = crate::config::sanitized_app_url();
let verification_uri = format!("{}/device", app_url);
let verification_uri_complete = format!("{}/device?user_code={}", app_url,
    urlencoding::encode(&display_user_code));
```

---

## Verification Page Flow — Detailed

### GET /device Handler Logic

```
device_code = lookup_from_cache(user_code_param)
case auth_state:
  !Auth::check() →
    store_oauth_return_to(format!("/device?user_code={}", urlencoding::encode(&user_code_param)))
    302 /auth/login
  Auth::check() + valid user_code →
    lookup DeviceGrant by device_code (from usercode pointer)
    if grant is None/Expired → render error page ("Code expired or invalid")
    if grant is Denied/Approved → render terminal page ("This code has already been used")
    if grant is Pending →
      lookup client from DB (find_by_client_id(grant.client_id))
      csrf_token = get_csrf_token()
      render confirm+consent HTML (client_name, device_code as hidden field, CSRF)
  Auth::check() + no/invalid user_code param →
    render code-entry form (input for user_code, POST to /device)
```

### POST /device Handler Logic (approve/deny)

```
parse form: {_token, action, device_code}
CSRF validate (ct_eq, same as consent.rs)
if action == "deny":
  lookup DeviceGrant by device_code
  set status = Denied
  Cache::put(mcp:device:{device_code}, updated_grant, DEVICE_CODE_TTL)
  // usercode pointer can stay or be forgotten; doesn't matter after terminal state
  render terminal "Access denied" page
if action == "approve":
  user_id = Auth::id() → 401 if absent (session expired)
  tenant_id = current_tenant().map(|t| t.id)
  lookup DeviceGrant by device_code
  set status = Approved, user_id, tenant_id
  Cache::put(mcp:device:{device_code}, updated_grant, DEVICE_CODE_TTL)
  render terminal "You may return to your device" page
```

**Hidden fields in confirm+consent form:**
- `_token` (CSRF)
- `action` (set by submit button value: "approve" / "deny")
- `device_code` (opaque; needed by POST handler to update the right grant)

Note: the `device_code` is an internal credential, never shown in the UI. It is safe to embed
as a hidden form field because it is already returned to the polling device in plaintext
over TLS in the `/device_authorization` response.

---

## Common Pitfalls

### Pitfall 1: TTL Reset on Put-Overwrite Causes Premature Expiry

**What goes wrong:** Updating `last_polled_at` via put-overwrite resets the TTL to a fresh
600s from now. If polls happen repeatedly near the end of the 10-minute window, the TTL keeps
extending, allowing grants to live longer than expected.

**Why it happens:** `Cache::put` with `Some(Duration)` always sets absolute TTL from the call
time; there is no "preserve remaining TTL" API.

**How to avoid:** For `last_polled_at` updates, this is acceptable — the polling extension is
bounded by the client eventually receiving `expired_token` or `access_token`. For the approve
step (state transition), the same trade-off applies; it is noted as a minor deviation from
strict 600s behavior. If strict TTL enforcement is needed, store `created_at` on the record
and check `now - created_at > 600` before returning `Approved` (manual expiry check).

**Warning signs:** Tests that store a grant and immediately check expiry by calling the handler
many times will see the TTL extend.

### Pitfall 2: Concurrent Poll vs Approve — Cache Overwrite Race

**What goes wrong:** If the polling client calls `POST /token` at the exact same instant the
user approves at `POST /device`, both handlers read the `Pending` record, and the token handler
may get the old `Pending` record while the approval writes `Approved`.

**Why it happens:** `ferro-cache` `InMemoryCache` uses an `RwLock`; get+put is not atomic.

**How to avoid:** The scenario is benign — the polling client gets `authorization_pending`, waits
`interval` seconds, and polls again to find `Approved`. No data loss. Document as known
non-atomic behavior. [ASSUMED — acceptable per D-05 which does not require atomic CAS]

### Pitfall 3: `user_code` Entry Survives After Token Issue

**What goes wrong:** After `Approved` → token issued (both keys forgotten via
`Cache::forget`), the `mcp:usercode:{user_code}` entry may still exist if `forget` is called
only on `mcp:device:{device_code}`.

**How to avoid:** On token issue, forget BOTH keys in order:
1. `Cache::forget("mcp:device:{device_code}")` (T-199-02 get-then-forget discipline)
2. `Cache::forget("mcp:usercode:{user_code}")` — the device code hash was stored on the record
   at grant creation; retrieve it from the approved grant before forgetting.

Store the `user_code_normalized` (without hyphen, uppercase) on the `DeviceGrant` record so the
token handler can reconstruct the `mcp:usercode:` key to forget.

### Pitfall 4: User-Code Normalization Mismatch

**What goes wrong:** User types `wdjb-mfxg`; cache lookup uses `WDJB-MFXG`; miss.

**How to avoid:** Normalize user input on the way in at the verification page handler:
```rust
let normalized = user_code.to_uppercase().replace(['-', ' '], "");
// lookup mcp:usercode:{normalized}
```
Store the `mcp:usercode:` key as the normalized (no-hyphen uppercase) form at grant creation.

### Pitfall 5: Token Endpoint Reads device_code from Wrong Form Field

**What goes wrong:** Current `TokenRequest` struct has `code: String` (for auth-code arm).
The device arm uses `device_code: String`. If `TokenRequest` is extended naively, a device
request missing `code` fails at deserialization before the `grant_type` branch.

**How to avoid:** Make `code`, `redirect_uri`, `code_verifier` optional (`Option<String>` with
`#[serde(default)]`); add `device_code: Option<String>`. Each arm validates the presence of
its own required fields after the branch.

### Pitfall 6: Discovery Test Misses the New grant_types_supported Entry

**What goes wrong:** Existing test asserts `grant_types[0] == "authorization_code"` (by index).
Adding `urn:ietf:params:oauth:grant-type:device_code` changes index positions.

**How to avoid:** Use `.contains()` or `.iter().any()` assertion style in the updated test:
```rust
assert!(grant_types.iter().any(|v| v.as_str() == Some("authorization_code")));
assert!(grant_types.iter().any(|v| v.as_str() == Some("urn:ietf:params:oauth:grant-type:device_code")));
```

### Pitfall 7: CWD-Relative View Paths Do Not Apply Here

No view files are loaded from disk by `ferro-mcp-oauth` handlers (raw HTML is rendered inline,
as in `consent.rs`). No `JsonUi::render_file` calls in this crate. No CWD concern.

---

## Code Examples

### DeviceGrant Generation (POST /device_authorization handler)

```rust
// Source: mirrors pkce::generate_auth_code [VERIFIED: ferro-mcp-oauth/src/pkce.rs]

const DEVICE_CODE_TTL: Duration = Duration::from_secs(600);
const DEVICE_INTERVAL_SECS: i64 = 5;
const USER_CODE_CHARSET: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ"; // RFC 8628 §6.1

pub fn generate_device_code() -> String {
    // Reuse existing function — 256-bit URL-safe random, identical to auth codes
    crate::pkce::generate_auth_code()
}

pub fn generate_user_code() -> String {
    let mut rng = rand::thread_rng();
    let chars: String = (0..8)
        .map(|_| USER_CODE_CHARSET[rng.gen_range(0..USER_CODE_CHARSET.len())] as char)
        .collect();
    // Format as XXXX-XXXX
    format!("{}-{}", &chars[..4], &chars[4..])
}

/// Normalize user input for lookup: uppercase, strip hyphens and spaces.
pub fn normalize_user_code(input: &str) -> String {
    input
        .to_uppercase()
        .chars()
        .filter(|c| *c != '-' && *c != ' ')
        .collect()
}
```

### Cache Storage at device_authorization

```rust
// Source: mirrors consent.rs Cache::put [VERIFIED: ferro-mcp-oauth/src/consent.rs lines 219-231]

let device_code = generate_device_code();
let display_user_code = generate_user_code();          // "BCDF-GHJK"
let normalized_user_code = normalize_user_code(&display_user_code); // "BCDFGHJK"

let now_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
let grant = DeviceGrant {
    client_id: form.client_id.clone(),
    status: DeviceGrantStatus::Pending,
    user_id: None,
    tenant_id: None,
    created_at: now_unix,
    last_polled_at: None,
    normalized_user_code: normalized_user_code.clone(), // needed by token handler to forget
};

Cache::put(
    &format!("mcp:device:{device_code}"),
    &grant,
    Some(DEVICE_CODE_TTL),
).await?;

Cache::put(
    &format!("mcp:usercode:{normalized_user_code}"),
    &device_code,     // pointer: just the device_code string
    Some(DEVICE_CODE_TTL),
).await?;
```

### Token Handler — Single-Use + State Machine

```rust
// Source: mirrors token.rs get-then-forget [VERIFIED: ferro-mcp-oauth/src/token.rs lines 61-64]

// Device-code arm:
let device_key = format!("mcp:device:{}", form.device_code.as_deref().unwrap_or(""));
let grant: Option<DeviceGrant> = Cache::get(&device_key).await.ok().flatten();

// Do NOT forget yet — we only forget on Approved (single-use on token issue)

let grant = match grant {
    None => return Err(json_error(400, "expired_token", "device_code expired or not found")),
    Some(g) => g,
};

let now_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

match grant.status {
    DeviceGrantStatus::Pending => {
        let last_poll = grant.last_polled_at.unwrap_or(grant.created_at);
        let elapsed = now_unix - last_poll;

        let updated = DeviceGrant { last_polled_at: Some(now_unix), ..grant.clone() };
        let _ = Cache::put(&device_key, &updated, Some(DEVICE_CODE_TTL)).await;

        if elapsed < DEVICE_INTERVAL_SECS {
            return Err(json_error(400, "slow_down",
                "polling too fast; increase interval by 5 seconds"));
        }
        return Err(json_error(400, "authorization_pending",
            "authorization request is still pending"));
    }
    DeviceGrantStatus::Denied => {
        return Err(json_error(400, "access_denied", "authorization request was denied"));
    }
    DeviceGrantStatus::Approved => {
        // Single-use: forget both keys before minting (T-199-02 discipline)
        let _ = Cache::forget(&device_key).await;
        let _ = Cache::forget(&format!("mcp:usercode:{}", grant.normalized_user_code)).await;

        // Identical mint call to auth-code arm [VERIFIED: token.rs lines 99-101]
        let claims = build_claims(
            grant.user_id.expect("Approved grant must have user_id"),
            grant.tenant_id,
            &config.app_url,
            3600,
        );
        let access_token = mint_token(&claims, &config.token_secret)
            .map_err(|e| json_error(500, "server_error", &format!("token mint error: {e}")))?;

        return Ok(ferro::HttpResponse::json(json!({
            "access_token": access_token,
            "token_type": "Bearer",
            "expires_in": 3600,
        })));
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Password login only | Magic-link + OAuth Device Grant | v12.7 (202-203) | Headless/CLI clients no longer blocked on browser callback |
| `grant_types_supported: ["authorization_code"]` | Adds `urn:ietf:params:oauth:grant-type:device_code` | Phase 203 | Discovery advertises both grants |
| Single `grant_type` in `token_exchange` | Two-arm dispatch on `grant_type` | Phase 203 | RFC-compliant single token endpoint |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `ferro-cache` `InMemoryCache` uses `RwLock`; get+put is not atomic | Pitfall 2 | If cache provides atomic CAS, the race concern is moot; implementation is still correct |
| A2 | TTL reset on put-overwrite is acceptable for `last_polled_at` updates | Pitfall 1 | If strict 600s enforcement is required, add manual `created_at` expiry check |
| A3 | Concurrent poll vs approve race is benign (poll gets another `authorization_pending`, tries again) | Pitfall 2 | Benign in practice; not a security concern |
| A4 | `slow_down` enforcement is advisory (return the code, let client self-correct) rather than strictly atomic | slow_down section | RFC requires returning the code; no atomicity requirement stated in RFC |

**If this table is empty for a project decision:** All implementation choices are derived from
verified code and CONTEXT.md locked decisions.

---

## Open Questions (RESOLVED)

1. **Where to store `normalized_user_code` on `DeviceGrant`**
   - What we know: the token handler needs to forget `mcp:usercode:{...}` on approval; it has
     the `device_code` from the form but not the `user_code`.
   - What's unclear: whether to store the normalized user_code on `DeviceGrant` or derive it
     differently.
   - Recommendation: add `normalized_user_code: String` field to `DeviceGrant`. Small cost;
     eliminates the derivation problem. This is Claude's Discretion per CONTEXT.md.
   - **RESOLVED:** Plan 203-01 adds `normalized_user_code: String` to the `DeviceGrant` struct;
     Plan 203-04 forgets `usercode_cache_key(&grant.normalized_user_code)` on approval. Decided.

2. **Manual expiry check for strict 600s TTL**
   - What we know: cache put-overwrite resets TTL; approved grants can linger slightly longer.
   - What's unclear: whether the product requires strict TTL or "best-effort" is fine.
   - Recommendation: add `if now_unix - grant.created_at > 600 { return expired_token }` before
     the state-machine match. Cheap and explicit. Default to this.
   - **RESOLVED:** Plan 203-04 adds the `now_unix - grant.created_at > 600` guard before the
     state-machine match, returning `expired_token`. Strict TTL enforced independent of cache TTL.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies beyond the existing Rust toolchain and `cargo`).
All dependencies are already declared in `ferro-mcp-oauth/Cargo.toml`. No new crates, no new
system tools, no new services.

---

## Validation Architecture

`workflow.nyquist_validation` key is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `#[tokio::test]` + inline `#[test]` (`tokio = { version = "1", features = ["full"] }`) |
| Config file | `ferro-mcp-oauth/Cargo.toml` `[dev-dependencies]` |
| Quick run command | `cargo test -p ferro-mcp-oauth` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| SC | Behavior | Test Type | Automated Command | File Exists? |
|----|----------|-----------|-------------------|-------------|
| SC-1 | `POST /device_authorization` returns exact RFC §3.2 fields | unit | `cargo test -p ferro-mcp-oauth device_authorization` | ❌ Wave 0 |
| SC-2 | Verification page: code-entry + confirm+consent states; unauth→login→resume | unit | `cargo test -p ferro-mcp-oauth device_verification` | ❌ Wave 0 |
| SC-3 | `POST /token` device-code arm returns correct error/success per §3.5 | unit | `cargo test -p ferro-mcp-oauth token_exchange` | partial (existing file needs new cases) |
| SC-4 | Discovery advertises `device_authorization_endpoint` + device-code grant type | unit | `cargo test -p ferro-mcp-oauth discovery` | partial (existing file; add assertions) |
| SC-5 | pending→approved, expiry, slow_down backoff, denied, tenant binding | unit | `cargo test -p ferro-mcp-oauth device_polling` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp-oauth`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Concrete Test List (SC-5 Matrix)

These are the specific test functions the planner must include in Wave-0 or Task tasks:

| Test Name | Scenario | Key Assertion |
|-----------|----------|---------------|
| `device_grant_pending_returns_authorization_pending` | Poll grant in Pending state, within interval | `error == "authorization_pending"` |
| `device_grant_approved_returns_access_token` | Grant transitions Pending→Approved, poll | `access_token` present, `token_type == "Bearer"` |
| `device_grant_expired_returns_expired_token` | `Cache::get` returns `None` (TTL elapsed simulation) | `error == "expired_token"` |
| `device_grant_slow_down_on_fast_poll` | Set `last_polled_at = now_unix`, poll immediately | `error == "slow_down"` |
| `device_grant_denied_returns_access_denied` | Grant in Denied state, poll | `error == "access_denied"` |
| `device_grant_tenant_binding` | Approved grant has `tenant_id = Some(7)`, poll | Minted JWT has `tenant_id = 7` claim |
| `device_grant_token_claims_identical_to_auth_code` | Compare `McpTokenClaims` from device vs auth-code arm | Same `sub`, `aud`, `iss`, `tenant_id` structure |
| `device_authorization_response_fields` | Call device_authorization handler logic | All 6 fields present (`device_code`, `user_code`, `verification_uri`, `verification_uri_complete`, `expires_in`, `interval`) |
| `user_code_normalization_strips_hyphen_and_case` | `normalize_user_code("wdjb-mfxg")` | Returns `"WDJBMFXG"` |
| `user_code_format_is_xxxx_hyphen_xxxx` | `generate_user_code()` | Length 9 (`XXXX-XXXX`), char 4 is `-`, all other chars in `BCDFGHJKLMNPQRSTVWXZ` |
| `discovery_advertises_device_authorization_endpoint` | `authorization_server_metadata(app_url)` | Key `device_authorization_endpoint` == `{app_url}/device_authorization` |
| `discovery_advertises_device_grant_type` | `authorization_server_metadata(app_url)` | `grant_types_supported` contains `"urn:ietf:params:oauth:grant-type:device_code"` |

### Wave 0 Gaps

- [ ] `ferro-mcp-oauth/src/device.rs` — covers SC-1, SC-2, SC-5 (new module)
- [ ] New test cases in `ferro-mcp-oauth/src/token.rs` — covers SC-3 (device-code arm)
- [ ] New assertions in `ferro-mcp-oauth/src/discovery.rs` tests — covers SC-4

---

## Security Domain

`security_enforcement` is not set to `false` in config — included.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Session-based auth via `Auth::check()` / `Auth::id()` (inherited from framework) |
| V3 Session Management | yes | CSRF via `get_csrf_token()` + `subtle::ConstantTimeEq` (matches consent.rs) |
| V4 Access Control | yes | Tenant binding at approval; `TenantFailureMode::Allow` on verification group |
| V5 Input Validation | yes | `client_id` validated against DB; `user_code` normalized before lookup |
| V6 Cryptography | yes | `device_code`: `generate_auth_code()` (256-bit random); `user_code`: uniform sampling from 20-char charset; JWT: HS256 with existing `jwt.rs` |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| `user_code` brute-force during 600s window | Spoofing | 20^8 keyspace; single-use once approved; TTL |
| `device_code` theft (network intercept) | Information Disclosure | Returned over TLS only; high entropy (256-bit); single-use |
| CSRF on `POST /device` approval | Tampering | `get_csrf_token()` + `ct_eq` (identical to consent.rs) |
| Open redirect via `store_oauth_return_to` | Spoofing | Stored URL constructed by handler itself from validated `/device?user_code=...`; never from user input directly |
| `device_code` in hidden form field | Information Disclosure | Acceptable: already transmitted in cleartext to polling device over TLS; embedding in server-side HTML for same-origin POST is not a new disclosure |
| Tenant binding bypass | Elevation of Privilege | Tenant captured from `current_tenant()` at approve time (TenantMiddleware active on the group); not from form input |
| Token claims divergence from auth-code path | Spoofing / Confusion | `build_claims` + `mint_token` called with identical arguments; single JWT minting path |

---

## Sources

### Primary (HIGH confidence)

- `ferro-mcp-oauth/src/store.rs` — `OAuthCode` record shape and cache key pattern [VERIFIED]
- `ferro-mcp-oauth/src/consent.rs` — CSRF pattern, HTML render, `Auth::id()` + `current_tenant()` capture [VERIFIED]
- `ferro-mcp-oauth/src/token.rs` — `token_exchange` structure, get-then-forget discipline (T-199-02), JWT mint call [VERIFIED]
- `ferro-mcp-oauth/src/jwt.rs` — `build_claims`, `mint_token`, `McpTokenClaims` fields [VERIFIED]
- `ferro-mcp-oauth/src/authorize.rs` — `store_oauth_return_to` + unauth redirect pattern [VERIFIED]
- `ferro-mcp-oauth/src/resume.rs` — `oauth_resume_redirect`, `take_oauth_return_to` contract [VERIFIED]
- `ferro-mcp-oauth/src/discovery.rs` — `authorization_server_metadata` structure and test pattern [VERIFIED]
- `ferro-mcp-oauth/src/pkce.rs` — `generate_auth_code()` implementation [VERIFIED]
- `ferro-mcp-oauth/src/config.rs` — `sanitized_app_url()`, `OAuthConfig::from_env()` [VERIFIED]
- `ferro-mcp-oauth/src/lib.rs` — `handlers` re-export shape, `cache_test_helpers` [VERIFIED]
- `app/src/routes.rs` — route mounting pattern; `/authorize` group as template for `/device` group [VERIFIED]
- `app/src/controllers/auth_controller.rs` — `verify_magic_link` → `oauth_resume_redirect("/")` (Phase 202 resume contract) [VERIFIED]
- `ferro-mcp-oauth/Cargo.toml` — existing dependencies confirming no new crates needed [VERIFIED]

### Secondary (MEDIUM confidence)

- RFC 8628 full text — §3.1, §3.2, §3.4, §3.5 error strings, §4 discovery key, §6.1 charset [CITED: https://www.rfc-editor.org/rfc/rfc8628]

### Tertiary (LOW confidence)

- None.

---

## Metadata

**Confidence breakdown:**
- RFC wire contract: HIGH — fetched from authoritative source
- Standard stack: HIGH — verified against `Cargo.toml`; no new dependencies
- Architecture/patterns: HIGH — derived from reading all relevant source files
- Pitfalls: MEDIUM — code-reading + RFC-reading; race condition analysis is ASSUMED
- Test architecture: HIGH — mirrors existing test patterns in `token.rs`, `discovery.rs`

**Research date:** 2026-06-11
**Valid until:** 2026-07-11 (stable domain; `ferro-mcp-oauth` is internal)
