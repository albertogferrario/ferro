# Phase 202: Login-resume contract + magic-link sample app - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-11
**Phase:** 202-login-resume-contract-magic-link-sample-app
**Mode:** `--auto` (all gray areas auto-selected; recommended default chosen per area)
**Areas discussed:** Resume-helper contract shape, Magic-link token storage, Dev-mode link surfacing, Verify handler flow, Login view conversion

> **Milestone activation note:** `init phase-op 202` initially reported `phase_found: false` —
> the v12.7 milestone pointer was not active (STATE.md `milestone: v12.6`, ROADMAP bullet `📋`).
> Per STATE.md's own instruction ("set milestone + 🚧 marker when beginning") and the known
> pointer-drift convention, the ROADMAP v12.7 bullet was flipped `📋 → 🚧` and STATE.md frontmatter
> updated to `milestone: v12.7` / `status: in_progress` before re-running init, which then found
> the phase. No scope change — bootstrap only.

---

## Resume-helper contract shape

| Option | Description | Selected |
|--------|-------------|----------|
| Crate-owned store+take+redirect helpers, single session-key owner | `ferro-mcp-oauth` owns `oauth_return_to` end to end; authorize.rs stores via helper, login handler takes/redirects via helper | ✓ |
| Document the key string only, keep inline session logic in app | App reads/writes the literal; crate just documents the convention | |
| Auto-resume middleware after any login | Middleware intercepts post-login and resumes the OAuth flow transparently | |

**User's choice:** [auto] Crate-owned store+take+redirect helpers (recommended default)
**Notes:** Two-sources-of-truth and hidden-control-flow rejected. The magic-link `verify` handler
runs in a separate request — without a shared helper it would re-duplicate the read/forget. The
helper is also the minimal surface gestiscilo's `verify_magic_link` adopts.
[[feedback_no_duplicate_control_surface]].

---

## Magic-link token storage

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro-cache`, single-use, ~15 min TTL | Token keyed in cache, value = user id; deleted on verify | ✓ |
| DB `login_tokens` table | Persisted row + cleanup reaper | |
| Stateless signed token | Self-contained, no store | |

**User's choice:** [auto] `ferro-cache` single-use TTL (recommended default)
**Notes:** Mirrors the v12.6 authorization-code precedent (199 D-03). DB table is overkill for a
minutes-lived credential and needs a reaper; stateless can't be single-use-revoked. App-local
exemplar infra — no new crate API.

---

## Dev-mode link surfacing vs real email

| Option | Description | Selected |
|--------|-------------|----------|
| Dev: surface on confirmation page + log; non-dev: `Channel::Mail` | `Environment::is_development()` gate; test follows the surfaced link | ✓ |
| Log-only in dev | Link only in stdout/tracing | |
| Always send real email | SMTP in every environment | |

**User's choice:** [auto] Surface on page + log in dev; `Channel::Mail` in non-dev (recommended default)
**Notes:** SC-2 requires "surfaced without a real email send"; SC-3's acceptance test must run
offline. Real-email path documented via the v11.9 `ferro-notifications` capability but not exercised
in CI.

---

## Verify handler flow & session establishment

| Option | Description | Selected |
|--------|-------------|----------|
| `GET /auth/verify?token=...`, delete-on-use, then resume redirect | Magic-link GET click → consume token → `Auth::login` → D-01 redirect | ✓ |
| POST verify | Form submission to verify | |
| Establish session without consuming token | Login without single-use deletion | |

**User's choice:** [auto] `GET /auth/verify`, delete-on-use, resume redirect (recommended default)
**Notes:** Email links are GET. Not consuming the token leaves it replayable. Invalid/expired →
re-render request-link page with error.

---

## Login view conversion (UX + theming)

| Option | Description | Selected |
|--------|-------------|----------|
| Email-only form → `POST /auth/login` (request-link) + confirmation state; delete password path | Single login entry; JSON-UI `layout:"auth"` + ThemeMiddleware | ✓ |
| Keep both password + magic-link | Two login systems | |
| Separate `/auth/request-link` route | Extra route alongside `/auth/login` | |

**User's choice:** [auto] Email-only `POST /auth/login` + confirmation; delete old password path (recommended default)
**Notes:** SC-2 says "converted from password to magic-link" — keeping both contradicts it. Reuse
`POST /auth/login` (the `/authorize` redirect already targets it) rather than a new route. `register`
password path untouched (calls `Auth::login` directly, not `authenticate()`).

---

## Claude's Discretion

- Exact helper names and session-key constant visibility.
- Token length/encoding and exact TTL.
- Confirmation-page copy; dev link as anchor vs plain text.
- Helper module layout (`resume.rs` vs extend `authorize.rs`).
- Acceptance test home (`app/src/tests/` vs `ferro-mcp-oauth/tests/`).

## Deferred Ideas

- Cross-device / headless magic-link auth — Phase 203 (RFC 8628 device grant).
- Consumer `verify_magic_link` adoption — gestiscilo consumer phase.
- Rate-limiting magic-link requests — hardening pass.
- Magic-link for registration — only login converted.
- Real-email path tested in CI.
