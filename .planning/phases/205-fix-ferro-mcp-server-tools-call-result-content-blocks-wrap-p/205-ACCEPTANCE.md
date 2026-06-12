# Phase 205 — Live Dogfood Acceptance (D-06)

**Date:** 2026-06-12
**Verdict:** GO
**Harness:** live `:8090` sample app binary (rebuilt with the fix), browser-driven OAuth via chrome-devtools-3

## What was tested

End-to-end MCP `tools/call` against the live server, exercising the patched result envelope
(`handle_tools_call` → `CallToolResult::structured`). The full authorization path was driven in a
real browser: dynamic client registration → PKCE authorize → magic-link login (`alice@acme.test`) →
consent approval → authorization-code/token exchange → authenticated `POST /mcp` `tools/call list_order`.

## Observed result

`tools/call list_order` (alice@acme.test, tenant Acme) returned HTTP 200 with this envelope:

- `result.content[0]` = `{ "type": "text", "text": "<json>" }` — a valid MCP content block.
  The original defect (bare row objects with no `type`, which a strict client rejects) is gone.
- `result.structuredContent` = `{ "rows": [...], "total": 2, "limit": 25, "offset": 0 }` — present
  and equal to the row data.
- `result.isError` = `false`.
- The outer JSON-RPC envelope (`jsonrpc: "2.0"`, `id`) is preserved by the caller.

| Check | Expected | Observed | Status |
|-------|----------|----------|--------|
| Content block shape | `type:text` content block (parseable by a strict client) | `content[0].type == "text"` | PASS |
| structuredContent | present, mirrors rows + pagination | present, `total:2` | PASS |
| Parse errors | none | none (valid CallToolResult shape) | PASS |
| Tenant scoping | alice sees only Acme's 2 of 4 orders | 2 rows, both `tenant_id:1` | PASS |

Order rows observed: id 1 (Alice Acme, 120.0, submitted, tenant 1) and id 2 (Alice Acme, 85.5,
delivered, tenant 1). The two Globex-tenant orders were correctly excluded.

## Verification method note

The bearer token from the prior session had expired, so re-authorization was performed by driving
the OAuth flow directly in the browser and inspecting the `tools/call` response on the wire. The
wire envelope is a valid `CallToolResult` (content union + `structuredContent`), which is exactly
what a strict MCP client validates. Equivalent strict-type parsing is additionally covered by the
in-tree D-04 regression test (`tools_call_result_parses_as_valid_mcp_content`), which deserializes
the emitted result with `rmcp::model::CallToolResult`'s own deserializer.

## Conclusion

GO. The result-formatting defect is resolved end-to-end against a live MCP client path, and
per-tenant scoping is unaffected by the envelope change.
