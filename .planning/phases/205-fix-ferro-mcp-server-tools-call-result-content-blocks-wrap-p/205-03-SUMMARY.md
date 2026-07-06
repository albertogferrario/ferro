---
phase: 205-fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p
plan: 03
status: complete
completed: 2026-06-12
---

# 205-03 Summary — Live dogfood acceptance (D-06)

## Outcome

GO. The fix was verified end-to-end against the live `:8090` sample-app binary through a real
browser-driven OAuth flow, and per-tenant scoping is intact.

## Tasks

- **Task 1 (rebuild):** `cargo build --bin app` clean (exit 0); `target/debug/app` rebuilt with the
  Plan 01 fix. The stale pre-fix instance on :8090 was replaced with the rebuilt binary.
- **Task 2 (human-verify checkpoint):** Live dogfood executed. The MCP bearer token from the prior
  session had expired; re-authorization was driven directly in chrome-devtools-3 (DCR → PKCE
  authorize → magic-link login `alice@acme.test` → consent → token exchange) and the authenticated
  `tools/call list_order` response was inspected on the wire.
- **Task 3 (record verdict):** `205-ACCEPTANCE.md` written with the GO verdict, observed envelope,
  and tenant-scoping result.

## Observed (acceptance evidence)

`POST /mcp tools/call list_order` (alice@acme.test) → HTTP 200:
- `result.content[0]` = `{"type":"text","text":"<json>"}` — valid MCP content block (original
  bare-object defect gone).
- `result.structuredContent` = `{rows:[…], total:2, limit:25, offset:0}`.
- `result.isError` = false; outer `jsonrpc`/`id` envelope preserved.
- Exactly 2 rows, both `tenant_id:1` (Acme) — Globex's 2 orders correctly excluded.

## Key files

- created: `.planning/phases/205-…/205-ACCEPTANCE.md`

## Notes

- The live verification used the raw wire response (the strict client's token had expired and
  interactive `/mcp` re-auth was unavailable to the agent). The wire envelope is a valid
  `CallToolResult`; strict-type parse equivalence is covered by the D-04 unit test
  (`tools_call_result_parses_as_valid_mcp_content`) deserializing with `rmcp::model::CallToolResult`.
- A separate, out-of-scope framework defect was found during the dogfood: data-bound **absolute**
  URLs in a JSON-UI GET action resolve to `href="#"` (the sample app's dev magic-link button).
  Tracked and fixed independently of Phase 205 (see the dev-login-button fix commit).

## Self-Check: PASSED
