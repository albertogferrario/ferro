# Phase 199: OAuth Browser Login - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen and logged inline)

<domain>
## Phase Boundary

The application becomes a spec-compliant OAuth 2.0 authorization server **for its own MCP
endpoint**. A standard MCP client can:

1. Discover the authorization server via `GET /.well-known/oauth-protected-resource` and
   `GET /.well-known/oauth-authorization-server` (the documents Phase 198's `WWW-Authenticate`
   header points at).
2. Dynamically register via `POST /register` (RFC 7591) and receive a `client_id`.
3. Run a browser **authorization-code + PKCE (S256)** flow at `GET /authorize` that **reuses
   the application's existing session login** (no second login system), shows a **consent**
   step, and redirects back with an authorization code.
4. Exchange the code at `POST /token` for an **access token bound to `(user, tenant)`**, with
   the MCP endpoint as **audience** and a short expiry.

Finally, the **bearer-validation seam** Phase 198 stubbed (`ferro-mcp-server::extract_bearer`,
always `Unauthenticated`) is filled with real validation: valid token → request proceeds;
invalid/expired → `401`; audience or tenant mismatch → `403`.

**In scope:** the four endpoint families (discovery, DCR, authorize+consent, token), token
minting + validation, PKCE verification, reuse of the existing login + session, the consent
screen, and filling the `/mcp` bearer seam. Tests prove the full flow without a live external
IdP (the app *is* the IdP).

**Out of scope (Phase 200):** per-tenant row scoping of `tools/call` results, and policy-layer
gating of tool calls. This phase only *binds* `(user, tenant)` into the token and *validates*
it; Phase 200 makes `dispatch` honor the tenant and run the policy layer (AMCP-10/11). The
token's tenant claim is the seam Phase 200 reads.

**Carrying forward:**
- Phase 198 `extract_bearer(authorization_header: Option<&str>) -> BearerOutcome` is the seam
  to fill (`ferro-mcp-server/src/auth.rs`); `BearerOutcome::Authenticated(serde_json::Value)`
  already exists for the principal. The `/mcp` handler (`app/src/controllers/mcp.rs`) already
  extracts the `Authorization` header before reading the body and branches on the outcome.
- Phase 198 chose to keep the `/mcp` handler **app-local** to avoid pulling `rmcp`+`schemars`
  into `framework`; the OAuth server has **no rmcp dependency**, so that constraint does not
  bind here.
- `serverInfo`/header URLs are sourced from `APP_NAME`/`APP_URL` via per-crate `from_env()`
  (project-agnostic rule) — the discovery documents and `aud`/`iss` values follow the same
  convention, never hardcoded.

</domain>

<decisions>
## Implementation Decisions

### Architectural home of the OAuth server (D-01) — AMCP-07/08/09, killer feature
- **D-01:** House the OAuth authorization server (discovery docs, DCR, `/authorize` + consent,
  `/token`, token mint + validate, PKCE) in a **new reusable crate `ferro-mcp-oauth`** that
  depends on `framework` (for `Request`/`HttpResponse`, `Auth`/session, `ferro-cache`, config)
  and exposes **mountable route handlers** plus a **token validator** the `/mcp` handler calls.
  Every ferro consumer app gets the MCP-OAuth endpoint by mounting these routes — this is the
  killer feature (per-tenant, projection-derived MCP toolset reachable over standard OAuth),
  so it must be framework-level infrastructure, **not** bolted into the sample `app`.
  - **[auto] recommended default** — chosen over (b) a `framework` submodule and (c) app-local.
    Rationale: OAuth-for-MCP has no `rmcp` coupling, so the Phase 198 "keep rmcp out of
    framework" reason does not apply; a dedicated crate keeps `framework` lean (jsonwebtoken,
    consent rendering, DCR storage all live in the crate) and matches the workspace's
    one-concern-per-crate shape (`ferro-stripe`, `ferro-audit`, …). App-local (c) is rejected:
    it would re-bolt the killer feature onto every consumer instead of shipping it once.
  - **RESEARCH FLAG (load-bearing):** confirm the dependency direction. `ferro-mcp-oauth →
    framework` makes it a **Wave 2+** publish crate (alongside `ferro-rs`/`ferro-mcp-server`);
    framework must NOT depend back on it (cycle). Add it to `.github/workflows/publish.yml`
    Wave 2. If a `framework` dependency is judged too heavy and a pure-logic crate + app-side
    glue is preferred, record that finding and fall back to a `framework` submodule.
  - **RESEARCH FLAG:** reconcile the **seam shape**. `extract_bearer(Option<&str>)` cannot
    validate a JWT without the signing key/config. Preferred: the `/mcp` handler calls
    `ferro-mcp-oauth`'s validator (which has config) and maps the result into `BearerOutcome`,
    keeping `ferro-mcp-server::extract_bearer` as a thin parser or replacing the call site.
    Decide whether to evolve the seam signature (Phase 198 intended "no signature change", but
    that intent predates needing the key) or wrap it — pick the lower-coupling option and
    record why. `ferro-mcp-server` must not gain a `ferro-mcp-oauth` dependency if avoidable.

### Access-token format (D-02) — AMCP-08, SC-4
- **D-02:** Mint a **self-contained JWT signed with HS256**. Claims: `sub` = user id,
  a **tenant claim** (D-06), `aud` = the MCP endpoint URL (`{APP_URL}/mcp`), `iss` =
  `{APP_URL}`, `iat`, and a **short `exp`** (default 1 hour). Self-validating → no DB round-trip
  on each `/mcp` call. Use `jsonwebtoken` v9 (already a workspace dep via `ferro-wallet`).
  - **[auto] recommended default** — chosen over an opaque DB-backed token (extra lookup per
    call, no payoff for a single-server symmetric setup). HS256 chosen over RS256: symmetric,
    single issuer = single validator, no keypair management.
  - **RESEARCH FLAG (load-bearing):** `AppConfig` today has **no signing secret** (only
    `name`/`url`/`debug`/`environment`). Decide the key source — introduce an env-driven secret
    (e.g. `MCP_TOKEN_SECRET` / `APP_KEY`) consumed by `ferro-mcp-oauth::from_env()`, mirroring
    `InertiaConfig::app_name`. Must fail closed (refuse to mint/validate) if unset in
    non-debug. Confirm whether a framework-wide `APP_KEY` should be introduced instead of a
    crate-local secret (avoid duplicate key vocabulary — see [[feedback_no_duplicate_control_surface]]).

### Authorization-code + PKCE-challenge storage (D-03) — AMCP-08, SC-3
- **D-03:** Store the short-lived authorization code server-side in **`ferro-cache` with a
  ~60s TTL**, single-use (deleted on first redemption). The stored record holds `client_id`,
  `redirect_uri`, `code_challenge` (+ method `S256`), `user_id`, `tenant`, and expiry. `/token`
  recomputes `S256(code_verifier)` and constant-time-compares it to the stored challenge.
  - **[auto] recommended default** — codes are ephemeral; `ferro-cache` (already in the
    workspace, TTL-native) fits better than a DB table that would need a cleanup job. The code
    value itself is generated with `rand` (already a `framework` dep) as a high-entropy opaque
    string.
  - **RESEARCH FLAG:** confirm `ferro-cache`'s default driver in the sample app persists across
    the authorize→token request boundary (in-memory single-process is fine for one server;
    note the multi-process caveat for production).

### Dynamic client registration persistence (D-04) — AMCP-07, SC-2
- **D-04:** Persist registered clients in a **database table** (`oauth_clients`: `client_id`,
  `redirect_uris`, `client_name`, `created_at`). `ferro-mcp-oauth` ships the **migration** (or
  a migration helper the app registers), so registration survives restarts and `client_id`s
  remain resolvable at `/authorize` time.
  - **[auto] recommended default** — chosen over cache-with-long-TTL (clients are long-lived
    relative to codes; losing them on restart breaks in-flight clients). MCP public clients use
    PKCE and have no client secret, so the table stores no secret.
  - **RESEARCH FLAG:** confirm whether the crate ships its own SeaORM migration vs. the app
    owning the migration (mirror how other `ferro-*` crates that need tables handle this);
    follow the established pattern, do not invent a second migration mechanism.

### Consent screen (D-05) — AMCP-08, SC-3
- **D-05:** Render a **minimal server-rendered HTML consent page** directly from
  `ferro-mcp-oauth` (the browser opens `/authorize`; the page shows the requesting client name,
  the granted scope, and approve/deny). Self-contained HTML keeps the crate reusable with **no
  coupling to the consumer app's Inertia/React build**.
  - **[auto] recommended default** — chosen over an Inertia page (would force every consumer to
    wire a frontend component) and over JSON-UI (heavier than needed for one static form).
  - **RESEARCH FLAG:** confirm CSRF protection on the consent POST reuses framework's existing
    CSRF token (`generate_csrf_token` in `framework/src/session`) so the consent form is not a
    new, unprotected surface.

### Login reuse + tenant binding (D-06) — AMCP-08, SC-3
- **D-06:** `/authorize` checks `Auth::check()`. If unauthenticated, **redirect to the
  application's existing login** (`/auth/login`) with a return-to back to `/authorize`
  (preserving the OAuth query params); after the existing session login completes, `/authorize`
  resumes → consent → code. The **tenant** bound into the code/token is taken from the
  **existing tenant resolution context** (`framework::tenant::current_tenant()`, set by
  `TenantMiddleware`) at authorize time. The tenant claim name in the JWT **matches what the
  existing JWT-claim tenant resolver expects** (`framework/src/tenant/resolver.rs`), so Phase
  200's tenant middleware reads it with no new code path — one tenant system, not two.
  - **[auto] recommended default** — reuses session login + tenant middleware rather than
    building a parallel identity/tenancy system (continuous-coherence; no duplicate control
    surface).
  - **RESEARCH FLAG:** confirm the existing `/auth/login` supports a post-login redirect
    (return-to) parameter; if not, add a minimal redirect-after-login mechanism rather than
    forking the login handler.
  - **RESEARCH FLAG:** handle the **multi-tenant-membership** case — if `current_tenant()` is
    `None` (single-tenant app) omit/neutralize the tenant claim; if a user belongs to multiple
    tenants and the context is ambiguous, decide whether the consent screen offers a tenant
    picker (defer the picker to Phase 200 if it expands scope — note as deferred).

### Bearer validation filling the seam (D-07) — AMCP-09, SC-5
- **D-07:** `/mcp` bearer validation (via `ferro-mcp-oauth`'s validator, per D-01) verifies, in
  order: (1) JWT signature + `exp` → fail = **401** `invalid_token`; (2) `aud` == this MCP
  endpoint URL → mismatch = **403**; (3) tenant claim present/consistent → mismatch = **403**.
  On success, the principal `(user, tenant)` is returned via
  `BearerOutcome::Authenticated(json!({ "user": …, "tenant": … }))` and the existing dispatch
  path (already wired in Phase 198) runs. Error responses follow RFC 6750
  (`WWW-Authenticate: Bearer error="invalid_token"`).
  - **[auto] recommended default** — distinguishes 401 (authentication failed) from 403
    (authenticated but not authorized for this audience/tenant) exactly as SC-5 requires.

### Claude's Discretion
- Internal module layout of `ferro-mcp-oauth` (e.g. `discovery.rs`, `register.rs`,
  `authorize.rs`, `token.rs`, `validate.rs`, `consent.rs`).
- Exact JSON shapes of the two `.well-known` documents beyond the spec-required fields, and the
  DCR response fields beyond `client_id`.
- JWT claim names for non-standardized fields (tenant claim name constrained by D-06's resolver
  match).
- Random-code length/encoding (high entropy, URL-safe).
- Whether discovery docs are static handlers or generated from config.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & forward-looking seams
- `.planning/ROADMAP.md` §"Phase 199" — goal, success criteria SC-1…SC-5.
- `.planning/ROADMAP.md` §"Phase 200" — tenant-scoping + policy seams the token's tenant claim
  must feed (read so the claim shape does not block 200).
- `.planning/REQUIREMENTS.md` — AMCP-07, AMCP-08, AMCP-09 (this phase); AMCP-10/11 (Phase 200).

### Phase 198 carry-forward (the seam this phase fills)
- `.planning/phases/198-streamable-http-endpoint-unauthenticated-challenge/198-CONTEXT.md` —
  D-05 (bearer seam), D-06 (`WWW-Authenticate` format), D-02 (handler placement, dependency
  weight finding).
- `ferro-mcp-server/src/auth.rs` — `extract_bearer`, `BearerOutcome` (the seam to fill).
- `app/src/controllers/mcp.rs` — the `/mcp` handler: header-before-body ordering, the
  authenticated-dispatch branch wired-but-unreachable, `// TODO(phase-199): validate Origin`.
- `ferro-mcp-server/src/dispatch.rs`, `ferro-mcp-server/src/renderer.rs` — the read path the
  validated request reaches; `dispatch` still leaves the tenant filter to Phase 200.
- `.planning/phases/198-streamable-http-endpoint-unauthenticated-challenge/198-02-SUMMARY.md`,
  `SECURITY.md` — handler wiring + the Origin-validation TODO.

### Framework reuse points (the "reuse existing login/tenant" mandate)
- `app/src/controllers/auth_controller.rs` — existing `login`/`register`; `Auth::login`,
  `Auth::attempt`.
- `framework/src/auth/guard.rs` — `Auth::check`/`Auth::id` and the session glue `/authorize`
  reuses.
- `framework/src/session/mod.rs` — `SessionStore`, `generate_csrf_token`, `set_auth_user`
  (consent CSRF + session reuse).
- `framework/src/tenant/context.rs` — `current_tenant()` (tenant captured at authorize time).
- `framework/src/tenant/resolver.rs` — the **JWT-claim tenant resolver**; the access-token
  tenant claim name MUST match what it reads (D-06).
- `framework/src/api/api_key.rs` — closest existing token-validation + constant-time-compare
  pattern (`subtle::ConstantTimeEq`, `sha2`) to mirror for bearer validation.
- `framework/src/config/providers/app.rs` — `AppConfig` (`name`/`url`); **no signing key today**
  (D-02 key-source flag).
- `framework/src/http/response.rs` — `HttpResponse::status` + `header` for redirects, 401/403.
- `app/src/routes.rs` — where the new OAuth routes + `.well-known` mount; existing `/auth`
  group as the login analog.
- `app/src/migrations/m20260228_create_api_keys_table.rs` — migration pattern for `oauth_clients`
  (D-04).
- `ferro-wallet/src/google/jwt.rs` + `ferro-wallet/Cargo.toml` — existing `jsonwebtoken` v9
  usage to mirror for HS256 minting/validation (D-02).
- `.github/workflows/publish.yml` (Wave 1A / Wave 2 lists) — add `ferro-mcp-oauth` to the
  correct wave (D-01).

### External specs (no repo file — read upstream)
- **MCP Authorization** spec (modelcontextprotocol.io) — the canonical flow MCP clients run:
  protected-resource metadata → auth-server metadata → DCR → authorize+PKCE → token → bearer.
- **RFC 8414** OAuth 2.0 Authorization Server Metadata — `/.well-known/oauth-authorization-server`.
- **RFC 9728** OAuth 2.0 Protected Resource Metadata — `/.well-known/oauth-protected-resource`
  (the doc Phase 198's header points at).
- **RFC 7591** OAuth 2.0 Dynamic Client Registration — `POST /register`.
- **RFC 6749** OAuth 2.0 Core — authorization-code grant, `/authorize` + `/token` shapes/errors.
- **RFC 7636** PKCE — `code_challenge`/`code_verifier`, S256 method.
- **RFC 8707** Resource Indicators — audience-restricting the token to the MCP endpoint.
- **RFC 6750** Bearer Token Usage — `401`/`WWW-Authenticate: Bearer error="invalid_token"`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Auth::check`/`Auth::login`/`Auth::attempt` + session store — the existing login `/authorize`
  reuses; no new identity system.
- `framework::tenant::current_tenant()` + the JWT-claim tenant resolver — token tenant binding
  + Phase 200 read path, one tenant system.
- `jsonwebtoken` v9 (workspace dep via `ferro-wallet`) — JWT mint/validate.
- `rand`, `base64`, `sha2`, `subtle::ConstantTimeEq` (all in `framework`) — code generation,
  PKCE S256, constant-time challenge compare.
- `ferro-cache` — TTL-native store for the ~60s authorization code (D-03).
- `framework::session::generate_csrf_token` — consent-form CSRF (D-05).
- `api_key.rs` — the existing prefixed-token + hashed-compare + middleware pattern to mirror.

### Established Patterns
- `ferro-*` crates read app identity from `APP_NAME`/`APP_URL` via `from_env()`
  (mirror `InertiaConfig::app_name`) — discovery docs, `aud`, `iss` follow this.
- Routes register via `routes!`/`group!`/`post!`/`get!`; the OAuth + `.well-known` routes slot
  into the same surface as `/mcp` and `/auth`.
- Token-validation middleware precedent: `ApiKeyMiddleware` gates routes on a credential.

### Integration Points
- New mountable routes from `ferro-mcp-oauth`: `GET /.well-known/oauth-protected-resource`,
  `GET /.well-known/oauth-authorization-server`, `POST /register`, `GET /authorize`,
  `POST /authorize` (consent), `POST /token`.
- The `/mcp` handler's bearer branch (Phase 198) calls the new validator (D-01/D-07).
- `oauth_clients` migration registered in the app's migration list (D-04).
- A signing secret env var consumed at `from_env()` (D-02).

</code_context>

<specifics>
## Specific Ideas

- This phase delivers the **killer feature** of the milestone: a standard MCP client (Claude
  Desktop, MCP SDK script) logs in through the consumer app's own browser login and gets a
  `(user, tenant)`-scoped, audience-bound token for the projection-derived toolset. The app is
  its **own** authorization server — no external IdP. Keep every piece reusable so *any* ferro
  app inherits the MCP-OAuth endpoint by mounting routes, not by reimplementing OAuth.
- The token's tenant claim is the single thread tying authorize-time tenancy → Phase 200's
  per-tenant scoping. Getting the claim name to match the existing JWT-claim resolver is the
  difference between "one tenant system" and "two" — treat it as load-bearing.

</specifics>

<deferred>
## Deferred Ideas

- **Refresh tokens / token rotation** — short-lived access tokens only for v1; add refresh if a
  client session needs to outlive the access-token expiry without re-consent.
- **Tenant-picker on consent for multi-tenant users** — only if `current_tenant()` is
  insufficient; revisit with Phase 200's scoping work.
- **RS256 / asymmetric keys + JWKS endpoint** — only when a separate resource server must
  validate without the signing secret (multi-service deployment).
- **Multi-process cache backend for authorization codes** — only when the consumer runs more
  than one app process behind a load balancer (D-03 caveat).
- **Per-tenant row scoping + policy gating of `tools/call`** — Phase 200 (AMCP-10/11), reads
  this phase's tenant claim.

None of these belong in Phase 199 — analysis stayed within scope.

</deferred>

---

*Phase: 199-oauth-browser-login*
*Context gathered: 2026-06-10*
