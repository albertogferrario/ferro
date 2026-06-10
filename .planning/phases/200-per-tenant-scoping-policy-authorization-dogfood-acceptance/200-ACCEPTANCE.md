---
phase: 200
slug: per-tenant-scoping-policy-authorization-dogfood-acceptance
acceptance_type: dogfood-run
requirement: SC-4 / AMCP-10 / AMCP-11
status: PENDING
verdict: ~   # Replace with GO or NO-GO after the run
recorded_by: ~
recorded_at: ~
---

# Phase 200 — Dogfood Acceptance Record

> SC-4: A real MCP client completes a browser login against a live consumer application,
> calls `tools/list` then `tools/call` for one exposed projection, and receives that
> tenant's rows. A run that fails end to end is NO-GO and the design is revised before
> the phase is marked complete.

---

## Prerequisite Checklist

Before running, verify every item below. A NO-GO caused by an unchecked prerequisite
is a setup miss, not a design defect — attribute correctly.

- [ ] `APP_URL` is set and the server is reachable at that address from the browser.
- [ ] `APP_URL` satisfies the Origin check in `ferro-mcp-oauth` (must be `http://` or `https://`
      scheme + host; no bare `localhost` without scheme).
- [ ] `MCP_TOKEN_SECRET` is set to at least 32 bytes of random data.
- [ ] Token audience `{APP_URL}/mcp` resolves to the running server.
- [ ] `app db:fresh` (or `cargo run -p app`) applied migrations and seeded the dogfood fixture:
      2 tenants (`acme`, `globex`), 2 users, 4 orders.
- [ ] `alice@acme.test` (password: `password123`) can log in at `/auth/login`.
- [ ] `bob@globex.test` (password: `password123`) can log in at `/auth/login`.
- [ ] `node --version` reports >= 18.

---

## Run Procedure

Follow `dogfood/README.md`. In brief:

1. Start the app: `APP_URL=... MCP_TOKEN_SECRET=... cargo run -p app`
2. In a second terminal: `node dogfood/run_dogfood.mjs`
3. When prompted, complete the browser login as the chosen tenant user.
4. Paste the redirect URL back into the script.
5. Record the result below.

---

## Observed Result

**Tenant logged in:** ~
(e.g. `alice@acme.test` → tenant `acme`, `tenant_id=1`)

**Script output summary:**
```
(paste the script's final lines here)
```

**tools/list showed:** ~
(list the tool names returned, confirm `list_order` is present)

**tools/call result:**
- Rows returned: ~
- All rows had `tenant_id` = ~
- Any cross-tenant rows visible: ~

**Claude Desktop confirmation (optional but recommended):**
- [ ] Added Claude Desktop MCP config from `dogfood/README.md`
- [ ] Claude Desktop prompted for browser login
- [ ] `list_order` tool appeared in tool list
- [ ] Calling `list_order` returned only the authenticated tenant's orders

---

## Verdict

**GO / NO-GO:** ~

*Replace `~` with `GO` or `NO-GO`.*

### If GO

Phase 200 acceptance gate passed. Tenant isolation is confirmed end to end via a
real MCP client with browser login.

### If NO-GO

**Failure point:** ~
(Describe exactly where the sequence broke: which step, what error.)

**Attribution:**
- [ ] Design defect — the system does not implement SC-1/SC-2/SC-4 correctly.
  (Requires a design revision before the phase is marked complete.)
- [ ] Setup issue — a prerequisite was not met (missing secret, seed not applied, wrong URL, etc.).
  (Fix the setup and re-run before attributing to design.)

**Notes for revision:**
~

---

*This record is the sole documented manual verification in Phase 200
(see 200-VALIDATION.md §Manual-Only). A NO-GO blocks phase completion.*
