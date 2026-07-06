# Phase 202: Login-resume contract + magic-link sample app - Context

**Gathered:** 2026-06-11
**Status:** Ready for planning
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen and logged inline)

<domain>
## Phase Boundary

Two coupled deliverables that make a **passwordless (magic-link) ferro app** complete the
v12.6 OAuth/MCP browser-login flow:

1. **A formalized login-resume contract in `ferro-mcp-oauth`.** Today the `/authorize` handler
   stores the in-flight authorize request as `oauth_return_to` in the session, and the sample
   app's login handler reads/clears that key inline with a duplicated string literal. This phase
   turns that into a **documented, crate-owned helper** that any login handler — synchronous
   password, asynchronous magic-link, future SSO — calls to obtain and consume the post-login
   redirect target. The contract is the seam that lets a login method that runs in a *separate
   request* (magic-link `verify`) resume the OAuth flow instead of dead-ending on a dashboard.

2. **The bundled sample app login converted from password to magic-link**, as the golden-path
   exemplar: a request-link handler issues a single-use, TTL-bounded token; a `verify` handler
   authenticates and redirects via the resume helper. In development (`APP_ENV=local`) the link
   is surfaced without a real email send. The login + magic-link views render through JSON-UI and
   are themed via `ThemeMiddleware`. An acceptance test drives the full async sequence end to end.

**In scope:** the resume-helper API in `ferro-mcp-oauth` (store + take + redirect, single
session-key owner); the sample app's magic-link request/verify handlers + token storage; the
JSON-UI login + confirmation views; the dev-mode link surfacing; the async-flow acceptance test;
clippy/test green and CWD-independent boot.

**Out of scope:**
- **Cross-device delivery** — Phase 203 (OAuth Device Authorization Grant, RFC 8628). Phase 202
  delivers only the **same-device** resume path.
- **Consumer adoption** — gestiscilo `verify_magic_link` calling the helper is a *consumer*
  phase; here we only shape the helper so that adoption is a one-line call.
- The **token issuer, consent screen, and tenant-scoping surfaces** — reused unchanged from
  v12.6 (no second token path, no parallel permission system).
- Refresh tokens, registration-flow changes (the password `register` handler is untouched).

**Carrying forward from Phase 199 (v12.6 OAuth browser login):**
- `oauth_return_to` is already written by `ferro-mcp-oauth/src/authorize.rs` (Step 3) as a raw
  session `put` of a literal key, and already read+forgotten inline by
  `app/src/controllers/auth_controller.rs` (`login` and `login_form`). Phase 202 **replaces both
  inline sites with the helper** so the key has one owner.
- D-06 (login reuse): `/authorize` redirects unauthenticated users to `/auth/login`. The magic-
  link conversion changes *what `/auth/login` does*, not the redirect contract.
- The login view is already JSON-UI (`src/views/login.json`, `layout: "auth"`); `ThemeMiddleware`
  is already mounted in `app/src/bootstrap.rs`.

</domain>

<decisions>
## Implementation Decisions

### Resume-helper API shape — the contract (D-01)
- **D-01:** Add a **symmetric helper set to `ferro-mcp-oauth`** that owns the `oauth_return_to`
  session key end to end:
  - a **store** helper the `/authorize` handler calls when redirecting an unauthenticated user
    (replacing the inline `session_mut(|s| s.put("oauth_return_to", url))` literal in
    `authorize.rs`),
  - a **take/consume** helper the login handler calls post-auth (reads the stored value **and
    clears it** from the session in one call),
  - an ergonomic **`oauth_resume_redirect(default)`**-style helper returning a `302`
    `HttpResponse` to the stored target, or to the caller-provided default when absent.
  The `"oauth_return_to"` string becomes a **single crate-owned constant** (or stays private
  behind the helpers) — no duplicated literal across `authorize.rs` and the app
  ([[feedback_no_duplicate_control_surface]]: one source of truth for the session key). Document
  that **any login method that calls the take/redirect helper participates in the OAuth flow**;
  one that doesn't, doesn't.
  - **[auto] recommended default** — chosen over (b) leaving the session logic inline in the app
    and only documenting the key string (two sources of truth; the magic-link `verify` handler
    would re-duplicate the read/forget), and (c) a middleware that auto-resumes after any login
    (hidden control flow, over-engineered for what is a single 302). A plain helper keeps the
    data flow explicit and is the minimal surface gestiscilo's `verify_magic_link` can adopt.

### Magic-link token storage — sample-app exemplar (D-02)
- **D-02:** Store the single-use, TTL-bounded login token in **`ferro-cache`**, keyed by the
  high-entropy token, value = the user identifier (id or email). TTL ~15 min; **deleted on first
  successful verify** (single-use). Token generated with `rand` (already a dep). This mirrors the
  v12.6 authorization-code storage precedent (199 D-03: `ferro-cache`, TTL-native, single-use)
  rather than introducing a DB table for an ephemeral credential. This is **app-local exemplar
  infrastructure, not crate API** — `ferro-mcp-oauth` gains no magic-link types.
  - **[auto] recommended default** — chosen over (b) a DB `login_tokens` table (heavier; needs a
    reaper job for a credential that lives minutes) and (c) a stateless signed token (no
    single-use revocation without a store; a clicked link would stay replayable until expiry).

### Dev-mode link surfacing vs real email (D-03)
- **D-03:** Gate on **`Environment::is_development()`** (true for `APP_ENV=local`, the default).
  In dev/test: **do not send a real email** — surface the magic link directly on the post-request
  **confirmation page** (JSON-UI) and also log it via `tracing`, so both the acceptance test and
  a human can follow it. In non-dev: dispatch via **`ferro-notifications` `Channel::Mail`** (the
  v11.9 capability already in the workspace) — documented, but **not exercised by the test** (no
  SMTP dependency in CI).
  - **[auto] recommended default** — chosen over (b) log-only (not visible on the page; a weaker,
    less testable exemplar) and (c) always sending real email (forces SMTP config and makes the
    acceptance test depend on a mail server). The dev path is what SC-2 ("surfaced without a real
    email send") and SC-3 (the async acceptance test) require.

### Verify handler flow & session establishment (D-04)
- **D-04:** **`GET /auth/verify?token=...`** (magic links are opened via GET from an email or the
  dev confirmation page). The handler: look up the token in cache → **absent/expired** →
  re-render the request-link page with an error; **valid** → delete the token (single-use),
  `Auth::login(user_id)`, then return the D-01 resume redirect — `302` to `oauth_return_to` when
  the login was initiated by `/authorize`, otherwise to `/`.
  - **[auto] recommended default** — chosen over POST-verify (breaks the email-link click) and
    over establishing the session without consuming the token (leaves the link replayable). GET +
    delete-on-use is the standard magic-link shape.

### Login view conversion — UX + theming (D-05)
- **D-05:** Replace the password login view (`src/views/login.json`) with an **email-only "send
  login link" form** (single `Input` + `Button`) posting to **`POST /auth/login`** (which becomes
  the request-link handler), plus a **confirmation state** ("Check your email" — and the dev link
  when `is_development()`). Both states render through **JSON-UI** with `layout: "auth"` and are
  themed via **`ThemeMiddleware`** (already mounted). **Delete the old password
  `login_form`/`authenticate` path entirely** (no deprecation, per architecture principles) — the
  `register` handler is untouched because it hashes and calls `Auth::login` directly without going
  through `authenticate()`.
  - **[auto] recommended default** — chosen over keeping both password and magic-link (two login
    systems; contradicts SC-2's "converted from password to magic-link") and over adding a
    separate `/auth/request-link` route (extra surface — reuse `POST /auth/login` as the single
    login entry the `/authorize` redirect already targets).

### Claude's Discretion
- Exact helper names (e.g. `take_oauth_return_to` / `oauth_resume_redirect` / `store_oauth_return_to`)
  and whether the session-key constant is `pub` or private behind the helpers.
- Token length/encoding (high-entropy, URL-safe) and the exact TTL value.
- Confirmation-page copy; whether the dev link renders as a clickable anchor or plain text.
- Module layout for the helper in `ferro-mcp-oauth` (new `resume.rs` vs extending `authorize.rs`).
- Whether the acceptance test lives in `app/src/tests/` (drives the app flow — natural home) or
  `ferro-mcp-oauth/tests/`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & forward seams
- `.planning/ROADMAP.md` §"Phase 202: Login-resume contract + magic-link sample app" — goal,
  SC-1…SC-5, consumer pairing.
- `.planning/ROADMAP.md` §"Phase 203: OAuth Device Authorization Grant (RFC 8628)" — the
  cross-device path; read so the resume helper does not foreclose the device-grant flow that
  shares the same login + consent + tenant-scoping surfaces.
- `.planning/ROADMAP.md` §"v12.7 Passwordless MCP Auth" — the two-gap field finding (login-resume
  continuation + cross-device delivery) and the conceptual-coherence constraint (one token issuer).
- `.planning/REQUIREMENTS.md` — v12.6/MCP scope context; design spec pointer below is the
  authoritative flow.
- `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md` — the v12.6
  browser-login design the resume contract formalizes.

### v12.6 carry-forward (the code this phase modifies)
- `.planning/phases/199-oauth-browser-login/199-CONTEXT.md` — D-06 (login reuse + `oauth_return_to`),
  D-03 (`ferro-cache` single-use TTL precedent the magic-link token mirrors), D-05 (consent reuse).
- `ferro-mcp-oauth/src/authorize.rs` — Step 3 writes `oauth_return_to` as an inline literal +
  `/auth/login` redirect; **the store helper replaces this**.
- `ferro-mcp-oauth/src/lib.rs` — public exports + `handlers` re-export module; the new resume
  helper exports here.
- `app/src/controllers/auth_controller.rs` — `login`, `login_form`, `authenticate` (the password
  path to convert), and the **inline `oauth_return_to` read/forget** in `login`/`login_form` that
  the take/redirect helper replaces.
- `app/src/views/login.json` — the JSON-UI login view (`layout: "auth"`) to convert to email-only
  + confirmation; its test in `auth_controller.rs` (`login_view_is_valid_and_posts_to_login`)
  locks the contract that must be updated.
- `app/src/routes.rs` — `/auth` guest group (`/login` page+post) and the `/authorize` group; the
  new `GET /auth/verify` route mounts in the guest group.

### Framework reuse points
- `framework/src/config/env.rs` — `Environment` (`Local`/`Development`/`Production`/`Testing`),
  `Environment::detect()` (reads `APP_ENV`), `is_development()` — the D-03 dev-mode gate.
- `framework/src/lib.rs` (line ~205) + `ferro-notifications/src/lib.rs` — `Channel::Mail`,
  `MailMessage`, `NotificationDispatcher` for the non-dev send path (D-03).
- `app/src/config/mail.rs` — existing `MailConfig::from_env()` (driver/host/from) for the non-dev
  send.
- `ferro-cache` (`Cache::put/get/forget`, TTL) — magic-link token store (D-02); see
  `ferro-mcp-oauth/src/lib.rs` `cache_test_helpers` for the in-test bootstrap pattern.
- `framework/src/session` — `session()`, `session_mut()`, `put`/`get`/`forget` (the resume helper
  wraps these); `Auth::login(user_id)` for session establishment (D-04).
- `app/src/bootstrap.rs` — `ThemeMiddleware` mount (D-05 theming) and provider/middleware setup;
  confirm magic-link request/verify run under the guest group's middleware.
- `rand` (dep of `ferro-mcp-oauth` and `framework`) — high-entropy token generation (D-02).

### External specs
- **RFC 6749** OAuth 2.0 Core — the authorize→token flow the resume contract reconnects.
- **RFC 8628** OAuth 2.0 Device Authorization Grant — Phase 203 (read forward only; not built here).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `oauth_return_to` session round-trip already exists (writer in `authorize.rs`, reader in
  `auth_controller.rs`) — Phase 202 **consolidates** it into one helper, it does not invent it.
- `ferro-cache` TTL store + the crate's `cache_test_helpers::bootstrap_test_cache()` — directly
  reusable for the magic-link token (D-02) and its test.
- `Environment::is_development()` / `detect()` — the dev-mode branch (D-03).
- `ferro-notifications` `Channel::Mail` + `MailMessage` (v11.9) + `app/src/config/mail.rs` — the
  non-dev send path, already in the workspace.
- JSON-UI `Card`/`Form`/`Input`/`Button` + `layout: "auth"` + `ThemeMiddleware` — the login and
  confirmation views (D-05) reuse the exact pattern already in `login.json`.
- `Auth::login(user_id)` — session establishment after token verify (D-04), same call `register`
  already uses.

### Established Patterns
- `ferro-mcp-oauth` exposes mountable handlers via `pub mod handlers` and helpers via top-level
  `pub use` — the resume helper follows this export shape.
- Single-use + short-TTL credentials live in `ferro-cache`, not the DB (199 D-03 authorization
  code) — the magic-link token follows it; DB tables are for long-lived records (199 D-04 clients).
- JSON-UI views with `layout: "auth"`, posting to a named `/auth/...` handler; content-negotiated
  handlers (`is_form` branch) already split browser-form vs JSON in `auth_controller.rs`.

### Integration Points
- `authorize.rs` Step 3 → calls the **store** helper instead of the inline `put`.
- `POST /auth/login` (auth_controller) → becomes the **request-link** handler (issue token, store
  in cache, dev-surface or mail).
- New `GET /auth/verify` (routes.rs guest group) → verify token, `Auth::login`, **take/redirect**
  helper.
- `src/views/login.json` → email-only request form + confirmation state.
- The async acceptance test drives: unauth `GET /authorize` → 302 `/auth/login` → request link →
  `verify` (with `oauth_return_to` in session) → 302 resume `/authorize` → consent rendered (SC-3).

</code_context>

<specifics>
## Specific Ideas

- The **load-bearing deliverable is the contract, not the magic-link UI.** The magic-link login is
  the *exemplar that proves* a separate-request login method can resume the OAuth flow; the reusable
  value is the `ferro-mcp-oauth` helper that any consumer login handler calls. Keep the helper
  minimal and the magic-link code app-local.
- The contract must stay compatible with **Phase 203's device grant**, which reuses the same login
  + consent + tenant-scoping surfaces and the same token issuer — the resume helper is about the
  *browser same-device redirect*; the device grant is an alternate front door to the same issuance.
  Do not bake authorization-code-redirect assumptions into the helper that would block 203.
- Honor the **CWD-independent boot** requirement (SC-5): the magic-link views load via the same
  `JsonUi::render_file("src/views/…")` path — verify the app still boots from any working directory
  (this has bitten the sample app before — see recent `from_path` → embedded-default fix).

</specifics>

<deferred>
## Deferred Ideas

- **Cross-device / headless magic-link auth** — Phase 203 (OAuth Device Authorization Grant,
  RFC 8628). The whole reason magic-link breaks the loopback callback is cross-device; 202 only
  fixes same-device resume.
- **Consumer `verify_magic_link` adoption** — gestiscilo same-device path; consumer phase, consumes
  the published helper.
- **Rate-limiting / throttling magic-link requests** — abuse protection for the request-link
  endpoint; note for a hardening pass, not required for the exemplar.
- **Magic-link for registration** (passwordless sign-up) — only login is converted; registration
  keeps its password path. Revisit if the app drops passwords entirely.
- **Real-email path in CI** — the non-dev `Channel::Mail` send is documented but untested here;
  an integration test against a mail sink could be added later.

None of these belong in Phase 202 — analysis stayed within scope.

</deferred>

---

*Phase: 202-login-resume-contract-magic-link-sample-app*
*Context gathered: 2026-06-11*
