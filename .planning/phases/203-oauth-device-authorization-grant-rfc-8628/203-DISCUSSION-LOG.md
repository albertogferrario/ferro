# Phase 203: OAuth Device Authorization Grant (RFC 8628) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-11
**Phase:** 203-oauth-device-authorization-grant-rfc-8628
**Mode:** `--auto` (all gray areas auto-selected; recommended option chosen per decision)
**Areas discussed:** Storage model, User-code format, Verification page surface, Verification flow (auth+consent+binding), Token polling state machine, Endpoints/client-validation/discovery

---

## Device/user-code storage model (D-01)

| Option | Description | Selected |
|--------|-------------|----------|
| ferro-cache `DeviceGrant` record (2 keys: device_code + user_code→device_code) | Mirrors 199 D-03 OAuthCode; ephemeral, TTL-native, mutable status | ✓ |
| DB `device_grants` table | Durable, but needs a reaper job for a minutes-lived credential; contradicts cache precedent | |
| Stateless signed device_code | No way to flip Pending→Approved; polling needs mutable server state | |

**User's choice:** ferro-cache `DeviceGrant` (auto — recommended)
**Notes:** Crate-owned state in `ferro-mcp-oauth` (new `device.rs`), unlike 202's app-local magic-link token. TTL ~600s.

---

## User-code format & charset (D-02)

| Option | Description | Selected |
|--------|-------------|----------|
| RFC 8628 §6.1 charset `BCDFGHJKLMNPQRSTVWXZ`, 8 chars `XXXX-XXXX`, case-insensitive | The spec's own recommended cross-device-entry format | ✓ |
| Numeric-only code | Smaller keyspace, easier to brute-force the short-lived grant | |
| Raw base64 | Case-sensitive, ambiguous glyphs, poor to read aloud / type | |

**User's choice:** RFC 8628 recommended charset (auto — recommended)
**Notes:** Normalize (strip hyphen/whitespace, uppercase) before lookup. `device_code` is a separate high-entropy URL-safe string, never shown.

---

## Verification page rendering surface (D-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Raw HTML in crate (like consent.rs) | Crate stays JSON-UI-free; matches consent look/CSRF discipline | ✓ |
| JSON-UI views | Would add a JSON-UI dependency to `ferro-mcp-oauth` (deliberately has none) | |
| Redirect into app's own views | Couples the crate to app routes; breaks mountable-handler model | |

**User's choice:** Raw HTML in crate (auto — recommended)
**Notes:** Two states on `GET /device` — code-entry, and confirm+consent.

---

## Verification flow — auth reuse, consent, binding (D-04)

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse 202 resume helper for login + existing consent screen; bind tenant at approval | One login, one consent; tenant captured via `Auth::id()`+`current_tenant()` at approve | ✓ |
| Fresh login/consent inside the device page | Duplicates two systems; violates one-consent/one-login constraint | |
| Bind tenant at `device_authorization` time | Device is anonymous there — tenant unknown until user authenticates | |

**User's choice:** Reuse resume + consent, bind at approval (auto — recommended)
**Notes:** `verification_uri={app_url}/device`, `verification_uri_complete={app_url}/device?user_code=...`. Mounts under `SessionUserTenantResolver` group like `/authorize`.

---

## Token-endpoint device-code grant branch + polling state machine (D-05)

| Option | Description | Selected |
|--------|-------------|----------|
| Extend `token.rs` with a device-code `grant_type` arm; full state machine | RFC 8628 §3.4: same endpoint, two grants; pending/slow_down/expired/denied/issued | ✓ |
| Separate `/device_token` endpoint | RFC uses the same token endpoint; a second one fragments the issuer | |
| Skip `slow_down` | SC-5 explicitly requires a slow_down backoff test; cheap with one timestamp | |

**User's choice:** Extend `token.rs` with device-code arm (auto — recommended)
**Notes:** Default interval 5s, expires_in 600s. `slow_down` enforced via `last_polled_at`. Issued token minted via existing `jwt.rs` (identical claims), both cache keys forgotten on issuance.

---

## Endpoints, client validation, PKCE, discovery (D-06)

| Option | Description | Selected |
|--------|-------------|----------|
| `POST /device_authorization` public + validate client_id; no PKCE; discovery advertises endpoint + grant type | Consistent with code-flow client validation; PKCE inapplicable (no redirect) | ✓ |
| Require PKCE on device flow | Non-standard for RFC 8628; protects a redirect code that doesn't exist here | |
| Skip client validation | Inconsistent with code flow; lets unregistered clients start a grant | |

**User's choice:** Public endpoint + client validation, no PKCE, discovery advertises both (auto — recommended)
**Notes:** Add a discovery test asserting `device_authorization_endpoint` and `urn:ietf:params:oauth:grant-type:device_code` appear.

---

## Claude's Discretion

- Module split / handler names; `handlers` re-export shape.
- Exact `device_code` length/encoding, precise TTL/interval (within RFC guidance).
- Whether the two cache entries share one record or the user_code entry is a pointer.
- Verification-page and terminal-page copy.
- `slow_down` enforcement strictness (strict reject vs advisory) and timing tolerance.
- Test file layout (integration vs in-module).

## Deferred Ideas

- Rate-limiting `POST /device_authorization` beyond RFC interval/slow_down.
- Refresh tokens (either grant).
- Consumer (gestiscilo) device-grant adoption.
- Adaptive backoff beyond a single `slow_down`.
- QR-code rendering of `verification_uri_complete` (client-side concern).
