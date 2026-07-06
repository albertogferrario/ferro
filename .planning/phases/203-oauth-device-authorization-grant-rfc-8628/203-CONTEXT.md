# Phase 203: OAuth Device Authorization Grant (RFC 8628) - Context

**Gathered:** 2026-06-11
**Status:** Ready for planning
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen and logged inline)

<domain>
## Phase Boundary

`ferro-mcp-oauth` gains the **OAuth 2.0 Device Authorization Grant (RFC 8628)** as an alternate
front door to the *same* token issuer it already runs for the authorization-code flow. This is the
auth path for clients that cannot complete a same-device browser callback: passwordless
(magic-link) users on a different device, headless/CLI MCP clients, and any cross-device login.

Three new moving parts, all reusing v12.6/202 surfaces:

1. **`POST /device_authorization`** — a public endpoint a device/CLI calls to start the flow. It
   returns `device_code`, `user_code`, `verification_uri`, `verification_uri_complete`,
   `expires_in`, and `interval` (RFC 8628 §3.2).
2. **A user-code verification page** at the `verification_uri` — the user opens it in *any* browser,
   enters/confirms the short `user_code`, authenticates (reusing the existing app login + the
   Phase 202 resume contract), goes through the **existing consent screen**, and on approval the
   `device_code` is bound to the authenticated `user_id` + `tenant_id`.
3. **Device-code token polling** — `POST /token` with
   `grant_type=urn:ietf:params:oauth:grant-type:device_code` returns `authorization_pending`,
   `slow_down`, `expired_token`, `access_denied`, or an `access_token` (RFC 8628 §3.5). Issued
   tokens are audience-bound and tenant-scoped **identically** to the authorization-code flow —
   they go through the same `jwt.rs` minting, not a parallel path.

**In scope:** the `device_authorization` endpoint + request/response shapes; the user-code
verification page (entry + confirm) bound to the existing consent + tenant scoping; the device-code
grant branch in the token endpoint with the full polling state machine (pending / slow_down /
expired / denied / issued); single-use + TTL on both `device_code` and `user_code`; discovery
metadata advertising `device_authorization_endpoint` and the device-code grant type; the RFC-8628
test matrix (SC-5); docs.

**Out of scope:**
- **A second token issuer or claims shape** — device tokens are minted by the existing `jwt.rs`
  with the same audience binding and tenant scoping (conceptual-coherence constraint: one token
  path).
- **A parallel consent / permission system** — the verification page reuses the existing consent
  screen and tenant resolution; it does not introduce new scopes or a new approval UI model.
- **Refresh tokens** — not part of this grant here (the authorization-code flow doesn't issue them
  either; out of scope unless a later phase adds refresh across both grants).
- **Consumer adoption** — gestiscilo making device grant its primary MCP auth path is a *consumer*
  phase; here we ship the server surface so adoption is configuration, not new ferro code.
- **Rate-limiting the device_authorization endpoint** beyond the RFC's `interval`/`slow_down`
  polling controls (note as a hardening deferral).

**Carrying forward from Phase 202 / v12.6 (the surfaces this phase reuses):**
- The **login + resume contract** (`store_oauth_return_to` / `take_oauth_return_to` /
  `oauth_resume_redirect`) — the verification page redirects an unauthenticated user to the app
  login and resumes back to the verification page after auth, exactly as `/authorize` does.
- The **consent screen** (`consent.rs` `authorize_post`) and its CSRF + tenant capture
  (`Auth::id()` + `current_tenant()`) — the device verification approval reuses this binding logic.
- The **`ferro-cache` single-use + short-TTL credential precedent** (199 D-03 `OAuthCode`,
  `mcp:code:{code}` key, get-then-forget) — the device/user codes follow it, not a DB table.
- The **`jwt.rs` token minting** + `validate_bearer` audience binding — device-code exchange mints
  the identical token.

</domain>

<decisions>
## Implementation Decisions

### Device/user-code storage model (D-01)
- **D-01:** Store device-grant state in **`ferro-cache`**, mirroring the `OAuthCode` precedent
  (199 D-03) — **not a DB table** (these are ephemeral credentials living minutes, the same class
  as authorization codes and magic-link tokens). A single `DeviceGrant` record holds the full
  state machine:
  - keyed by `device_code` under `mcp:device:{device_code}` (the client polls with this),
  - a **second cache entry** `mcp:usercode:{user_code}` → `device_code` so the verification page
    can resolve a user-entered code back to the grant,
  - fields: `client_id`, `status` (`Pending` | `Approved` | `Denied`), `user_id: Option<i64>`,
    `tenant_id: Option<i64>` (both `None` until approval binds them), `created_at`, optional
    `last_polled_at` (for `slow_down` enforcement, D-05). TTL ~ the RFC `expires_in` (recommend
    600s / 10 min).
  - **[auto] recommended default** — chosen over (b) a DB `device_grants` table (heavier; needs a
    reaper job for a credential that lives minutes; contradicts the established 199 D-03 /
    202 D-02 "ephemeral credentials live in cache" pattern) and (c) a single stateless signed
    device_code (no way to flip Pending→Approved without a store; polling needs mutable state).
  - This is **new crate-owned state in `ferro-mcp-oauth`** (a `device.rs` store type alongside
    `store::OAuthCode`), not app-local — the device endpoints are crate handlers like
    `authorize`/`consent`/`token`.

### User-code format & charset (D-02)
- **D-02:** Generate the `user_code` from the **RFC 8628 §6.1 recommended charset**
  (`BCDFGHJKLMNPQRSTVWXZ` — 20 unambiguous uppercase consonants, no vowels/digits to avoid words
  and visual confusion), **8 characters grouped `XXXX-XXXX`** with a hyphen for readability.
  Verification accepts the code **case-insensitively** and **ignores the hyphen/whitespace**
  (normalize before lookup) so a user can type `wdjbmfxg`, `WDJB-MFXG`, etc. The `device_code` is a
  separate **high-entropy URL-safe random string** (like the auth code), never shown to the user.
  - **[auto] recommended default** — chosen over (b) a numeric-only code (smaller keyspace, easier
    to brute-force the short-lived grant) and (c) raw base64 (case-sensitive, ambiguous glyphs,
    poor to read aloud / type from another device). The RFC's own recommended charset is the
    standard and is built for cross-device manual entry.

### Verification page rendering surface (D-03)
- **D-03:** Render the verification page as **server-built HTML inside `ferro-mcp-oauth`**,
  consistent with the existing **`consent.rs` raw-HTML** approach — the crate stays free of a
  JSON-UI dependency (the sample app's JSON-UI login is app-local; crate handlers emit HTML
  directly with the same CSRF + escaping discipline as `consent.rs`). Two states on the same
  `verification_uri` (`GET /device`):
  - **code-entry** form when no/invalid `user_code` is present,
  - **confirm + consent** when a valid `user_code` is resolved and the user is authenticated
    (renders the existing consent approve/deny surface, scoped to the device grant's `client_id`).
  - **[auto] recommended default** — chosen over (b) JSON-UI views (would add a JSON-UI dep to
    `ferro-mcp-oauth`, which deliberately has none — `consent.rs` is raw HTML for exactly this
    reason) and (c) a redirect into the app's own views (couples the crate to app routes; breaks
    the "mountable crate handlers" model). Match `consent.rs` so the device approval and the
    code-flow consent look and behave the same.

### Verification flow — auth reuse, consent, and binding (D-04)
- **D-04:** `verification_uri` = `{app_url}/device`; `verification_uri_complete` =
  `{app_url}/device?user_code={user_code}` (pre-fills the code so a clickable link skips manual
  entry, RFC 8628 §3.3.1). Flow:
  1. `GET /device` (optionally with `?user_code=`): if **unauthenticated**, store the current
     device URL via the **Phase 202 `store_oauth_return_to` helper** and redirect to the app login;
     after login the resume helper returns the user here (same contract `/authorize` uses).
  2. Authenticated + valid `user_code` → render the **confirm + consent** page (CSRF token in
     session, as `consent.rs` does).
  3. `POST /device` (approve/deny): CSRF-validate; on **approve**, capture `user_id = Auth::id()`
     and `tenant_id = current_tenant().map(|t| t.id)` and flip the `DeviceGrant` to `Approved` with
     those bound (the **same tenant capture as `consent.rs` step 4c**); on **deny**, flip to
     `Denied`. Render a terminal "you may return to your device" page.
  - The verification page mounts under a route group with the **`SessionUserTenantResolver`
    TenantMiddleware** (same group semantics as `/authorize`, D-07 from 199) so the bound
    `tenant_id` is real.
  - **[auto] recommended default** — chosen over (b) building a fresh login/consent inside the
    device page (duplicates two systems; violates the one-consent / one-login constraint) and over
    (c) binding tenant at `device_authorization` time (the device is unauthenticated there — tenant
    is only known once the *user* authenticates at verification, exactly as the code flow binds at
    consent, not at `/authorize`).

### Token-endpoint device-code grant branch + polling state machine (D-05)
- **D-05:** Extend **`token.rs`** to branch on `grant_type`. Today it hard-rejects anything but
  `authorization_code`; add a `urn:ietf:params:oauth:grant-type:device_code` arm (keep the existing
  code arm unchanged — one endpoint, two grants, RFC 8628 §3.4). The device arm reads the
  `DeviceGrant` by `device_code` and returns (RFC 8628 §3.5):
  - missing/expired grant → `expired_token`,
  - `Pending` → `authorization_pending`,
  - `Pending` **and polled faster than `interval`** → `slow_down` (and the client must add 5s to
    its interval); enforce via `last_polled_at` on the record,
  - `Denied` → `access_denied`,
  - `Approved` → mint the JWT via the **existing `jwt.rs` path** (audience-bound, tenant-scoped
    from the bound `user_id`/`tenant_id`) and **forget both cache keys** (single-use, T-199-02
    get-then-forget discipline), return `access_token`.
  - Default `interval` = **5s** (RFC default); `expires_in` = the grant TTL (600s).
  - **[auto] recommended default** — chosen over (b) a separate `/device_token` endpoint (RFC says
    the device grant uses the **same** token endpoint with a distinct `grant_type`; a second
    endpoint would fragment the issuer) and over (c) skipping `slow_down` (SC-5 explicitly requires
    a `slow_down` backoff test; cheap to implement with one timestamp field).

### Endpoints, client validation, PKCE, and discovery (D-06)
- **D-06:** Paths and metadata:
  - `POST /device_authorization` (public, like `/register` and `/token` — no session) takes
    `client_id` (form/POST per RFC 8628 §3.1) and **validates it against the `oauth_clients`
    table** (same `find_by_client_id` check the code flow uses); unknown client → `invalid_client`.
  - **No PKCE** on the device flow — PKCE protects an authorization *code in a redirect*, and the
    device grant has no redirect; the `device_code` is a bearer secret returned directly over TLS
    and bound server-side to the polling client. (Do not bolt PKCE onto the device flow; it is not
    in RFC 8628 and adds no security here.)
  - **Discovery** (`discovery.rs` `authorization_server_metadata`) advertises
    `device_authorization_endpoint = {app_url}/device_authorization` and adds
    `urn:ietf:params:oauth:grant-type:device_code` to `grant_types_supported` (RFC 8628 §4) — and
    add a discovery test asserting both appear.
  - **[auto] recommended default** — chosen over requiring PKCE (non-standard for this grant) and
    over skipping client validation (the code flow validates `client_id`; the device flow should be
    consistent — an unregistered client should not be able to start a grant).

### Claude's Discretion
- Exact module split (`device.rs` for the store type + endpoint handlers, vs separate
  `device_authorization.rs` / `device_verify.rs`) and handler names; the `handlers` re-export shape
  in `lib.rs`.
- Exact `device_code` length/encoding and the precise TTL/interval values (within RFC guidance:
  interval 5s, expires_in ~600s).
- Whether the two cache entries share one record or the `user_code` entry stores only a pointer.
- Verification-page copy and the terminal success/denied page wording.
- Whether the `slow_down` interval bump is enforced strictly (reject) or advisory (return code,
  let the client self-correct) — RFC requires returning `slow_down`; the exact server-side timing
  tolerance is discretionary.
- Test file layout (`ferro-mcp-oauth/tests/` integration vs in-module unit tests), reusing
  `cache_test_helpers::bootstrap_test_cache()`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & milestone framing
- `.planning/ROADMAP.md` §"Phase 203: OAuth Device Authorization Grant (RFC 8628)" — goal,
  SC-1…SC-5, consumer pairing (gestiscilo cross-device adoption).
- `.planning/ROADMAP.md` §"v12.7 Passwordless MCP Auth (Phases 202–203)" — the two-gap field
  finding (login-resume + cross-device delivery) and the conceptual-coherence constraint: **one
  token issuer, no parallel permission system**.
- `.planning/phases/202-login-resume-contract-magic-link-sample-app/202-CONTEXT.md` — the resume
  contract this phase reuses (same-device path); 203 is the cross-device path that shares the same
  login + consent + tenant-scoping surfaces.

### v12.6 carry-forward (the code this phase extends)
- `ferro-mcp-oauth/src/consent.rs` — `authorize_post`: CSRF validation (constant-time `ct_eq`),
  client_id + redirect_uri re-validation, **tenant/user capture (`Auth::id()` + `current_tenant()`)
  and code minting** — the device approval reuses this binding logic and the raw-HTML render style.
- `ferro-mcp-oauth/src/token.rs` — `token_exchange`: the `grant_type` gate (currently
  `authorization_code`-only), the **get-then-forget single-use discipline** (T-199-02), and the
  JWT mint call — the device-code grant arm is added here.
- `ferro-mcp-oauth/src/store.rs` — `OAuthCode` cache record (`mcp:code:{code}`, ~60s TTL) the
  `DeviceGrant` record mirrors; `find_by_client_id` for client validation.
- `ferro-mcp-oauth/src/jwt.rs` + `McpTokenClaims` + `validate_bearer` (`validate.rs`) — the
  audience-bound, tenant-scoped token the device exchange must mint **identically**.
- `ferro-mcp-oauth/src/discovery.rs` — `authorization_server_metadata`: add
  `device_authorization_endpoint` + the device-code grant type (and a test).
- `ferro-mcp-oauth/src/authorize.rs` — Step 3 unauthenticated redirect + `store_oauth_return_to`
  usage; the verification page's unauth path mirrors it.
- `ferro-mcp-oauth/src/resume.rs` + `lib.rs` exports — `store_oauth_return_to`,
  `take_oauth_return_to`, `oauth_resume_redirect` (Phase 202) the verification page calls.
- `ferro-mcp-oauth/src/lib.rs` — `pub mod handlers` re-export shape; the new device handlers export
  here for mounting in `app/src/routes.rs`.

### App wiring
- `app/src/routes.rs` lines ~70–88 — the `/authorize` group with the
  **`SessionUserTenantResolver` TenantMiddleware (`TenantFailureMode::Allow`)**, and the public
  `/register` / `/token` mounts. The `device_authorization` endpoint mounts public (like
  `/register`); the `/device` verification page mounts in a session/tenant group like `/authorize`.
- `app/src/controllers/auth_controller.rs` — `login_page` / `login` / `verify_magic_link` (the
  login the verification page redirects to via the resume helper).

### Framework reuse points
- `ferro-cache` (`Cache::put/get/forget`, TTL) + `cache_test_helpers::bootstrap_test_cache()` in
  `ferro-mcp-oauth/src/lib.rs` — `DeviceGrant` storage (D-01) and its tests.
- `framework/src/session` — CSRF token in session (as `consent.rs` does) and `Auth::id()`.
- Tenant resolution: `current_tenant()` + `SessionUserTenantResolver` (199 D-07) — the bound
  `tenant_id` (D-04).
- `rand` (dep of `ferro-mcp-oauth`) — `device_code` and `user_code` generation (D-02).
- `framework/src/config` — `sanitized_app_url()` (used by `discovery.rs`) for
  `verification_uri` / `verification_uri_complete` interpolation (no hardcoded host).

### External specs
- **RFC 8628** OAuth 2.0 Device Authorization Grant — the authoritative spec:
  - §3.1 device authorization request (`client_id`),
  - §3.2 device authorization response (`device_code`, `user_code`, `verification_uri`,
    `verification_uri_complete`, `expires_in`, `interval`) — **SC-1**,
  - §3.3 user interaction, §3.3.1 `verification_uri_complete` — **SC-2**,
  - §3.4 device-grant token request (`grant_type=urn:ietf:params:oauth:grant-type:device_code`),
  - §3.5 token response + error codes (`authorization_pending`, `slow_down`, `access_denied`,
    `expired_token`) — **SC-3, SC-5**,
  - §4 discovery metadata (`device_authorization_endpoint`) — **SC-4**,
  - §6.1 user-code recommended charset — **D-02**.
- **RFC 6749** OAuth 2.0 Core — base error/response semantics the token endpoint already follows.
- **RFC 8414** Authorization Server Metadata — the discovery doc `discovery.rs` extends (SC-4).
- `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md` — the v12.6
  browser-login design context (consent + tenant scoping the device flow reuses).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`consent.rs` `authorize_post`** — CSRF (`ct_eq`), client/redirect validation, and the exact
  **`Auth::id()` + `current_tenant()` capture** the device approval binds with. The device confirm
  page is a near-sibling of the consent page.
- **`token.rs` `token_exchange`** — already structured as parse → grant_type gate → cache
  get-then-forget → mint; adding a device-code arm is an extension, not a rewrite.
- **`store::OAuthCode` + `mcp:code:{code}` cache pattern** — the `DeviceGrant` record and its two
  cache keys (`mcp:device:{device_code}`, `mcp:usercode:{user_code}`) follow it directly.
- **`resume.rs` helpers (202)** — `store_oauth_return_to` / `oauth_resume_redirect` make the
  verification page's unauthenticated→login→resume path a few lines, identical to `/authorize`.
- **`discovery.rs`** — single function to extend for `device_authorization_endpoint` + grant type,
  with an existing test harness pattern to copy.
- **`jwt.rs` minting + `validate_bearer`** — the device token is the *same* token; no new claims.
- **`cache_test_helpers::bootstrap_test_cache()`** — in-test cache bootstrap for the polling/expiry
  tests (SC-5).

### Established Patterns
- Crate handlers emit **raw HTML** (`consent.rs`) — the verification page follows this; the crate
  has no JSON-UI dependency by design.
- **Ephemeral credentials → `ferro-cache`, long-lived records → DB** (199 D-03 vs D-04). Device and
  user codes are ephemeral → cache.
- **Single-use via get-then-forget BEFORE validation** (T-199-02) — applied to `device_code` on
  successful issuance.
- **Public OAuth endpoints** (`/register`, `/token`, discovery) mount with no session middleware;
  **session-bearing endpoints** (`/authorize`) mount under `SessionUserTenantResolver`
  TenantMiddleware (`Allow` failure mode so unauthenticated visitors reach the login redirect).
- Discovery URLs interpolate `sanitized_app_url()` — **no hardcoded host** (enforced by an existing
  discovery test).

### Integration Points
- New crate handlers in `ferro-mcp-oauth` (e.g. `device.rs`): `device_authorization` (POST),
  `device_verification_get` (GET `/device`), `device_verification_post` (POST `/device`); re-export
  via `lib.rs` `handlers`.
- `token.rs` → add the `urn:ietf:params:oauth:grant-type:device_code` branch.
- `discovery.rs` → advertise the new endpoint + grant type.
- `app/src/routes.rs` → mount `POST /device_authorization` public; mount `/device` (GET+POST) in a
  `SessionUserTenantResolver` group like `/authorize`.
- `store.rs` (or new `device.rs`) → `DeviceGrant` cache record + helpers.

</code_context>

<specifics>
## Specific Ideas

- **One token issuer is the load-bearing invariant.** The device-code exchange must mint via the
  exact same `jwt.rs` path with the same audience binding and tenant scoping as the code flow — the
  whole v12.7 conceptual-coherence claim is "an alternate front door to the same issuance." A
  reviewer should be able to diff the device-arm mint call against the code-arm and see the same
  claims construction.
- **Tenant is bound at verification (when the user authenticates), never at `device_authorization`**
  — the device is anonymous when it requests a code; `current_tenant()` is only meaningful once the
  human logs in at the verification page, exactly as the code flow binds tenant at consent, not at
  `/authorize`.
- **Reuse the consent surface, do not fork it.** The device confirm page should render the same
  approve/deny consent the code flow uses (scoped to the device grant's client), so there is one
  consent UX and one CSRF discipline.
- **The RFC error strings are the contract** — `authorization_pending`, `slow_down`,
  `expired_token`, `access_denied` must match RFC 8628 §3.5 verbatim; MCP/OAuth clients branch on
  them. SC-5's test matrix asserts each.
- Keep the device store crate-owned (`ferro-mcp-oauth`), not app-local — unlike the magic-link
  token (which was a sample-app exemplar in 202), the device grant **is** framework surface: every
  ferro app that mounts the OAuth handlers gets it.

</specifics>

<deferred>
## Deferred Ideas

- **Rate-limiting / abuse protection on `POST /device_authorization`** beyond the RFC
  `interval`/`slow_down` polling controls — a hardening pass; the same deferral noted for the
  magic-link request endpoint in 202.
- **Refresh tokens** for either grant — not issued today; a cross-grant refresh story is its own
  phase.
- **Consumer adoption** — gestiscilo making device grant its primary MCP auth path (consumer
  pairing), consumes the published surface.
- **A polling-throttle / exponential backoff beyond a single `slow_down`** — RFC's minimum is one
  `slow_down` response; richer adaptive backoff is not required for SC-5.
- **QR-code rendering of `verification_uri_complete`** on the device side — a client-side
  convenience, not a server endpoint concern.

None of these belong in Phase 203 — analysis stayed within scope.

</deferred>

---

*Phase: 203-oauth-device-authorization-grant-rfc-8628*
*Context gathered: 2026-06-11*
