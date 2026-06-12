# Phase 205: Fix ferro-mcp-server tools/call result content blocks - Context

**Gathered:** 2026-06-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix the `ferro-mcp-server` `tools/call` **success-path result envelope** so a strict MCP
client parses it. Today `handle_tools_call` (`ferro-mcp-server/src/jsonrpc.rs:84-91`) places
`DispatchResult.rows` — a `Vec<serde_json::Value>` of bare projection records — directly into
the result `content` array. Each row has no `type` field, so it is not a valid MCP content
block; a strict client (Claude Code's SDK) Zod-rejects every item. The same result object
also carries `total`/`limit`/`offset` as non-standard top-level keys.

This phase: wrap rows as valid MCP content blocks (text content + `structuredContent`), add a
client-schema interop regression test that parses the emitted result with the MCP client's own
types, and re-run the live `:8090` browser-OAuth dogfood (`alice@acme.test` → `list_order`).

The defect is isolated to success-path result formatting. OAuth/login-resume/consent/token/
tenant-scoping are already verified working (v12.6 + Phase 202) and are **not** in scope. The
JSON-RPC error envelope (`-32601/-32602/-32603`) is already valid and **not** in scope.
</domain>

<decisions>
## Implementation Decisions

### Result envelope construction
- **D-01:** Build the result with `rmcp::model::CallToolResult::structured(value)` rather than
  hand-assembling JSON. `rmcp 0.12` is already a `ferro-mcp-server` dependency and its
  `structured()` constructor produces exactly the required shape: `content =
  vec![Content::text(value.to_string())]` (a valid `{"type":"text","text":...}` block),
  `structuredContent = Some(value)`, `isError = Some(false)`. Using the library type keeps the
  output schema-correct by construction and avoids re-deriving the content union by hand.

### structuredContent payload shape
- **D-02:** Nest the pagination metadata inside the structured value as a single object —
  `{ "rows": [...], "total": N, "limit": N, "offset": N }` — and pass that whole object to
  `CallToolResult::structured`. This fixes the secondary defect (non-standard `total`/`limit`/
  `offset` top-level result keys) in the same change: everything lives under `structuredContent`,
  and the text content block mirrors it.

### Text content block granularity
- **D-03:** Emit a **single** text content block containing the full structured JSON payload
  (the default behavior of `CallToolResult::structured`). Do not emit one text block per row —
  per-row blocks are noisy and the structured data is already available via `structuredContent`.

### Regression test strictness
- **D-04:** The interop regression test must parse the **emitted** `result` with the MCP
  client's own types — deserialize `result.content` into `Vec<rmcp::model::Content>` and assert
  every block parses, plus assert `structuredContent` is present and round-trips. The prior unit
  tests asserted the server's own output shape and so missed the bug; the new test must exercise
  the same strictness a real client applies. (`CallToolResult` itself derives `Serialize` only,
  not `Deserialize` — the test parses the content/structuredContent fields, not the whole
  envelope, into the client types. Planner/researcher confirm the exact deserialization target.)

### Error-path scope
- **D-05:** Leave the existing JSON-RPC error envelope unchanged. Invalid-filter (`-32602`) and
  internal (`-32603`) errors already serialize as valid JSON-RPC errors; converting tool-level
  failures into `CallToolResult { isError: true }` is a separate behavioral change, deferred.

### Live dogfood re-run
- **D-06:** After the fix, re-run the live `:8090` browser-OAuth dogfood end-to-end using the
  harness in the canonical refs: `alice@acme.test` → `list_order`, confirming (a) Claude Code's
  MCP client now parses the result without Zod errors and (b) tenant scoping still returns only
  Acme's 2 of 4 orders. This is the phase's acceptance gate, not just a unit pass.

### Claude's Discretion
- Exact naming of any helper introduced to assemble the structured value.
- Whether the `result` JSON-RPC wrapping (`"result": <CallToolResult>`) is produced inline in
  `handle_tools_call` or via a small serialize step — as long as the inner value is the
  serialized `CallToolResult`.
- Compact vs pretty JSON inside the text block (default `value.to_string()` is acceptable).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### The bug site and data source
- `ferro-mcp-server/src/jsonrpc.rs` §`handle_tools_call` (lines 48-100, defect at 84-91) —
  builds the result object; `"content": result.rows` is the malformed line.
- `ferro-mcp-server/src/dispatch.rs` §`DispatchResult` (lines 20-24) — the
  `rows: Vec<Value>` / `total` / `limit` / `offset` fields that become the structured payload.
- `ferro-mcp-server/src/renderer.rs` — `McpRenderer` (tool definition side; context only, the
  result-formatting fix is in `jsonrpc.rs`, not here).

### MCP result types (the canonical fix + test types)
- `rmcp 0.12` `CallToolResult` + `Content` —
  `~/.cargo/registry/src/index.crates.io-*/rmcp-0.12.0/src/model.rs:1534` for `CallToolResult`
  (fields `content`/`structured_content`/`is_error`/`_meta`, `#[serde(rename_all="camelCase")]`)
  and `structured()` at ~1582. `Content` is the content-block union the test deserializes into.
  Declared dep: `ferro-mcp-server/Cargo.toml:15` (`rmcp = "0.12"`, features `server,macros,base64`).

### Roadmap + live harness
- `.planning/ROADMAP.md` §"Phase 205" (line ~2580) — phase statement and scope fence.
- Live `:8090` browser-OAuth dogfood harness — recorded in operator memory
  (`project_ferro_mcp_toolcall_content_bug.md`): run `cd app && ../target/debug/app` (port 8090,
  `app/.env`, seeded `app/database.db` with alice@acme.test/tenant Acme); registered with Claude
  Code as MCP server `ferro-sample-app` → `http://127.0.0.1:8090/mcp`; drive OAuth via
  `chrome-devtools-3` (clear `/tmp/chrome-mcp-3/Singleton{Lock,Socket,Cookie}` first); grab the
  dev `/auth/verify?token=…` link from the DOM since the button href is `#`.

### External spec
- MCP `CallToolResult` / content-block schema (modelcontextprotocol.org spec) — the normative
  shape `rmcp::model::CallToolResult` mirrors; reference only, the rmcp type is authoritative
  in-tree.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `rmcp::model::CallToolResult::structured(Value)` — produces a schema-valid envelope (text
  block + structuredContent + isError:false) from a single structured value. Drop-in for the fix.
- `rmcp::model::Content` — the client-side content-block type; the regression test deserializes
  into it to replicate strict client parsing.
- `DispatchResult` already carries exactly `{rows, total, limit, offset}` — assemble these into
  one `serde_json::json!({...})` value and hand it to `CallToolResult::structured`.

### Established Patterns
- `handle_tools_call` returns a `serde_json::Value` (`{"result": ...}` or `{"error": ...}`); the
  outer JSON-RPC `id`/`jsonrpc` wrapping is added by the caller. The fix only changes the
  `result` value; the error branch and the `Method not found` branch stay as-is.
- Tests live inline (`#[cfg(test)] mod tests`) in each module (`renderer.rs`, `dispatch.rs`).
  The new interop test fits the same in-module convention in `jsonrpc.rs`.

### Integration Points
- Only `jsonrpc.rs::handle_tools_call`'s `Ok(result)` arm changes. No signature change, no
  change to `dispatch`, `renderer`, `schema`, or `auth`.
</code_context>

<specifics>
## Specific Ideas

- The bug was caught by a real client (Claude Code's MCP SDK) but missed by unit tests because
  the tests asserted the server's own emitted shape. The regression test's value is precisely
  that it parses with the *client's* type, not the server's expectation — D-04 is the load-
  bearing decision that prevents recurrence.
</specifics>

<deferred>
## Deferred Ideas

- **Tool-level error results.** Converting invalid-filter / internal failures from JSON-RPC
  errors into `CallToolResult { isError: true, content: [...] }` (per the MCP spec's distinction
  between protocol errors and tool-execution errors). Out of scope here — the current JSON-RPC
  error envelope is valid; revisit if a client needs tool-level error semantics.
- **`_meta` for pagination.** Placing `total`/`limit`/`offset` in `CallToolResult._meta` instead
  of inside `structuredContent`. Not chosen (structuredContent keeps one coherent payload); note
  only if a future client wants pagination separated from the structured data.

</deferred>

---

*Phase: 205-fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p*
*Context gathered: 2026-06-12*
