# Phase 199: OAuth Browser Login - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-10
**Phase:** 199-oauth-browser-login
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen)
**Areas discussed:** Architectural home, Token format, Code/PKCE storage, DCR persistence, Consent screen, Login reuse + tenant binding, Bearer validation

---

## Architectural home of the OAuth server

| Option | Description | Selected |
|--------|-------------|----------|
| New `ferro-mcp-oauth` crate (→ framework) | Reusable mountable routes + validator; every consumer app inherits the MCP-OAuth endpoint | ✓ |
| `framework` submodule | OAuth lives inside core framework | |
| App-local (sample `app` only) | Bolt OAuth onto the sample app | |

**Choice:** New `ferro-mcp-oauth` crate. **Notes:** OAuth-for-MCP has no rmcp coupling, so Phase 198's "keep rmcp out of framework" reason does not apply. Dedicated crate keeps framework lean and matches one-concern-per-crate. App-local rejected — would re-bolt the killer feature onto every consumer. Two research flags raised: publish-wave/dependency-direction and the `extract_bearer` seam-shape reconciliation.

---

## Access-token format

| Option | Description | Selected |
|--------|-------------|----------|
| Self-contained JWT (HS256) | `(user, tenant)`, `aud`, `iss`, short `exp`; no DB hit per call | ✓ |
| Opaque DB-backed token | Random token, validated by DB lookup | |
| JWT RS256 | Asymmetric keypair | |

**Choice:** JWT HS256 via `jsonwebtoken` v9 (already a workspace dep). **Notes:** Self-validating, symmetric single-issuer. Research flag: `AppConfig` has no signing key today — introduce an env-driven secret, fail closed if unset; reconcile crate-local vs framework-wide `APP_KEY`.

---

## Authorization-code + PKCE-challenge storage

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro-cache`, ~60s TTL, single-use | Ephemeral code record with challenge | ✓ |
| Database table | Persistent codes + cleanup job | |

**Choice:** `ferro-cache` short TTL. **Notes:** Codes are ephemeral; cache is TTL-native, no cleanup job. Research flag: confirm cache driver persists across the authorize→token boundary; note multi-process caveat.

---

## Dynamic client registration persistence

| Option | Description | Selected |
|--------|-------------|----------|
| Database table `oauth_clients` | Survives restart; client_ids resolvable at authorize | ✓ |
| Cache, long TTL | Lost on restart | |

**Choice:** DB table (crate ships migration or app owns it). **Notes:** Clients long-lived vs codes; PKCE public clients store no secret. Research flag: crate-shipped vs app-owned migration — follow established pattern.

---

## Consent screen

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal server-rendered HTML (from crate) | Self-contained, no app frontend coupling | ✓ |
| Inertia/React page in app | Forces frontend wiring on every consumer | |
| JSON-UI page | Heavier than needed for one form | |

**Choice:** Server-rendered HTML from `ferro-mcp-oauth`. **Notes:** Research flag: consent POST reuses framework CSRF token.

---

## Login reuse + tenant binding

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse existing session login + `current_tenant()` | Redirect to `/auth/login` with return-to; tenant from middleware context; claim name matches JWT-claim resolver | ✓ |
| New OAuth-specific login/identity | Parallel auth system | |

**Choice:** Reuse existing login + tenant middleware. **Notes:** One tenant system, not two (claim matches `framework/src/tenant/resolver.rs`). Research flags: confirm `/auth/login` supports return-to redirect; handle `None`/multi-tenant-membership (tenant picker deferred to Phase 200 if it expands scope).

---

## Bearer validation filling the seam

| Option | Description | Selected |
|--------|-------------|----------|
| Ordered: sig+exp→401, aud→403, tenant→403 | Distinguishes authn-fail from authz-fail; RFC 6750 errors | ✓ |

**Choice:** Ordered validation returning `BearerOutcome::Authenticated((user, tenant))`. **Notes:** Matches SC-5 exactly (401 vs 403 split).

## Claude's Discretion

- Internal module layout of `ferro-mcp-oauth`; exact `.well-known`/DCR JSON shapes beyond spec
  fields; non-standard JWT claim names (tenant claim constrained by resolver match); random-code
  length/encoding; static vs config-generated discovery docs.

## Deferred Ideas

- Refresh tokens / rotation; tenant-picker for multi-tenant users; RS256 + JWKS; multi-process
  cache backend for codes; per-tenant scoping + policy gating (Phase 200).
