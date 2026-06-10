---
phase: 200
slug: per-tenant-scoping-policy-authorization-dogfood-acceptance
acceptance_type: dogfood-run
requirement: SC-4 / AMCP-10 / AMCP-11
status: COMPLETE
verdict: GO
recorded_by: dogfood run against live app (127.0.0.1:8080), both tenants
recorded_at: 2026-06-11
---

# Phase 200 — Dogfood Acceptance Record

> SC-4: A real MCP client completes a browser login against a live consumer application,
> calls `tools/list` then `tools/call` for one exposed projection, and receives that
> tenant's rows. A run that fails end to end is NO-GO and the design is revised before
> the phase is marked complete.

---

## Prerequisite Checklist

- [x] `APP_URL=http://127.0.0.1:8080` set; server reachable.
- [x] `APP_URL` satisfies the Origin/scheme check (`http://` + host).
- [x] `MCP_TOKEN_SECRET` set to 64 hex chars (32 bytes) in `app/.env`.
- [x] Token audience `http://127.0.0.1:8080/mcp` resolves to the running server (confirmed in token claims).
- [x] Migrations applied + dogfood fixture seeded: 2 tenants (`acme`, `globex`), 2 users, 4 orders.
- [x] `alice@acme.test` / `password123` logs in (tenant 1).
- [x] `bob@globex.test` / `password123` logs in (tenant 2).
- [x] Node v22.12.0 (>= 18).

---

## How the run was driven

The acceptance was executed against a **live, separately-run** `cargo run` server
(`http://127.0.0.1:8080`). The browser-login step was driven **programmatically** (cookie
jar + manual redirects) rather than through a GUI browser, exercising the identical HTTP
OAuth sequence: discovery → dynamic client registration → `GET /authorize` → session login
(`POST /auth/login`) → consent (`POST /authorize`, CSRF-validated) → `POST /token`
(authorization-code + PKCE S256) → `POST /mcp` `initialize`/`tools/list`/`tools/call`.
The checked-in `dogfood/run_dogfood.mjs` remains the human-in-the-loop GUI path; a Claude
Desktop GUI run is the recommended optional human confirmation (not yet performed).

---

## NO-GO on first run → design revision → GO

**First run: NO-GO.** The flow broke at `GET /authorize`, which redirected to `/auth/login`
indefinitely even after a successful `POST /auth/login`. Root cause: the app never mounted
the framework's `SessionMiddleware`, so **no session cookie was ever issued** and the login
session did not persist across requests — `Auth::check()` at `/authorize` always saw an
unauthenticated request. The in-process Phase 200 tests passed because they drove
middleware directly and never performed an HTTP cookie round-trip. This is precisely the
gap the dogfood gate exists to catch.

**Design revision (commit `ee8aed92`):**
- Added the `sessions` table migration (`m20260611_create_sessions_table`, mirrors
  `framework::session::driver::database::sessions::Model`).
- Mounted `SessionMiddleware::new(SessionConfig::from_env())` as the first global middleware
  in `bootstrap.rs` so the session context + cookie wraps every downstream middleware,
  including the OAuth `/authorize` group.
- `SESSION_SECURE=false` for local `http://` so the cookie is issued/sent in dev.

**Re-run after the fix: GO (both tenant directions).**

---

## Observed Result

**Run A — `alice@acme.test` → tenant `acme` (`tenant_id=1`):**
```
6. token OK — claims sub=901 tenant_id=1 aud=http://127.0.0.1:8080/mcp
8. tools/list OK — tools: list_order
10. rows returned: 2; expected tenant_id=1
    order id=1 customer=Alice Acme tenant_id=1
    order id=2 customer=Alice Acme tenant_id=1
GO: PASS — 2 row(s), all tenant_id=1; list_order present; full OAuth+MCP flow completed.
```

**Run B — `bob@globex.test` → tenant `globex` (`tenant_id=2`):**
```
6. token OK — claims sub=... tenant_id=2 aud=http://127.0.0.1:8080/mcp
8. tools/list OK — tools: list_order
10. rows returned: 2; expected tenant_id=2
    order id=3 customer=Bob Globex tenant_id=2
    order id=4 customer=Bob Globex tenant_id=2
GO: PASS — 2 row(s), all tenant_id=2; list_order present; full OAuth+MCP flow completed.
```

**tools/list showed:** `list_order` (present, as required).

**tools/call result:**
- Rows returned: 2 (Run A), 2 (Run B).
- All rows had `tenant_id` = the authenticated tenant (1 for A, 2 for B).
- Cross-tenant rows visible: **none** — token A never saw tenant 2's orders and vice-versa (SC-1, both directions).

**Policy gating (AMCP-11):** the `order` projection requires the `view-orders` Gate ability,
checked via `Gate::authorize_for` before dispatch; both authenticated users hold it, so the
call proceeded. (Negative path — missing ability → `isError` tool error, no disclosure — is
covered by the automated tests `policy_deny_no_ability` / `policy_deny_tool_error_shape`.)

**Claude Desktop confirmation (optional):**
- [ ] Not yet performed — recommended as a follow-up human-facing confirmation; the scripted
      run already exercises the full OAuth + MCP contract a GUI client uses.

---

## Verdict

**GO / NO-GO:** **GO**

### If GO

Phase 200 acceptance gate passed. Per-tenant isolation is confirmed end to end via the full
OAuth browser-login + MCP `tools/call` sequence against a live server, in both tenant
directions, with the policy layer in effect. The one end-to-end gap found (missing session
middleware) was fixed as the design revision the gate requires, and the re-run is GO.

---

*This record is the sole documented manual verification in Phase 200
(see 200-VALIDATION.md §Manual-Only).*
