# Plan 200-07 Summary — Dogfood Acceptance

**Status:** Complete (verdict GO)
**Requirements:** AMCP-10, AMCP-11 (SC-4)

## What was delivered

- `dogfood/run_dogfood.mjs` — checked-in human-in-the-loop scripted MCP client driving the
  full OAuth + MCP sequence (discovery → DCR → `/authorize` browser login → `/token` →
  `tools/list` → `tools/call`) against a live app. (commit `525051c7`)
- `dogfood/README.md` — prerequisites, run steps, and a Claude Desktop config path. (`525051c7`)
- `200-ACCEPTANCE.md` — the GO/NO-GO record, now filled with the **GO** verdict and evidence.

## Acceptance run (SC-4)

Executed against a live `cargo run` server at `http://127.0.0.1:8080` with the dogfood
fixture seeded (2 tenants, 2 users, 4 orders). The browser-login step was driven
programmatically (cookie jar + manual redirects), exercising the identical HTTP OAuth flow.

- **Run A** (`alice@acme.test`, token `tenant_id=1`): `tools/call list_order` → 2 rows, both
  `tenant_id=1`; no tenant-2 rows.
- **Run B** (`bob@globex.test`, token `tenant_id=2`): `tools/call list_order` → 2 rows, both
  `tenant_id=2`; no tenant-1 rows.
- `list_order` present in `tools/list`; `view-orders` Gate ability enforced before dispatch.

→ **SC-1** (bidirectional isolation), **SC-2** (policy gating; negative path covered by
automated tests), **SC-3** (single tenant system), **SC-4** (live dogfood) all satisfied.

## Deviation: NO-GO → design revision → GO

The first end-to-end run was a **NO-GO** — the `/authorize` browser-login flow could not
complete because the app never mounted `SessionMiddleware`, so no session cookie was issued
and login did not persist. Fixed in commit `ee8aed92` (sessions table migration + global
`SessionMiddleware` + `SESSION_SECURE=false` for local http). Re-run is GO. This is the
design revision the phase goal prescribes for a NO-GO; in-process tests had missed it
because they never performed an HTTP cookie round-trip.

## Key files
- created: `dogfood/run_dogfood.mjs`, `dogfood/README.md`, `200-ACCEPTANCE.md`
- fix (separate commit `ee8aed92`): `app/src/migrations/m20260611_create_sessions_table.rs`,
  `app/src/migrations/mod.rs`, `app/src/bootstrap.rs`

## Self-Check: PASSED
