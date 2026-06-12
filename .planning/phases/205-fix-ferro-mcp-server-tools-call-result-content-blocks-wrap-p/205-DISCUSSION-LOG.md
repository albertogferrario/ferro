# Phase 205: Fix ferro-mcp-server tools/call result content blocks - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-12
**Phase:** 205-fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p
**Areas discussed:** Envelope construction, structuredContent shape, Text block granularity, Regression test strictness, Error-path scope
**Mode:** `--auto` (recommended defaults auto-selected)

---

## Envelope construction

| Option | Description | Selected |
|--------|-------------|----------|
| `CallToolResult::structured()` | Use the rmcp 0.12 constructor (already a dep) — emits text block + structuredContent + isError:false | ✓ |
| Hand-build JSON | Manually assemble `{"content":[{"type":"text",...}],"structuredContent":...}` | |
| Text-only, no structuredContent | One text block, drop structured data | |

**User's choice:** `CallToolResult::structured()` (auto / recommended)
**Notes:** Library type keeps output schema-correct by construction; rmcp 0.12 already declared at Cargo.toml:15.

---

## structuredContent shape

| Option | Description | Selected |
|--------|-------------|----------|
| Nest `{rows,total,limit,offset}` | One structured object passed to `structured()`; text block mirrors it | ✓ |
| Keep top-level keys | Leave total/limit/offset alongside content (still non-standard) | |
| Move to `_meta` | Put pagination in CallToolResult._meta | |

**User's choice:** Nest inside the structured value (auto / recommended)
**Notes:** Fixes the secondary non-standard top-level-keys defect in the same change.

---

## Text block granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Single text block | Whole structured JSON in one block (default of `structured()`) | ✓ |
| One block per row | Each projection record as its own text block | |

**User's choice:** Single text block (auto / recommended)
**Notes:** Per-row blocks noisy; structured data already available via structuredContent.

---

## Regression test strictness

| Option | Description | Selected |
|--------|-------------|----------|
| Parse with client types | Deserialize emitted `content[]` into `Vec<rmcp::model::Content>`; assert structuredContent present | ✓ |
| Hand-rolled validator | Custom JSON-schema check | |
| String contains check | Assert output string contains `"type":"text"` | |

**User's choice:** Parse with the client's own types (auto / recommended)
**Notes:** Prior unit tests asserted server output shape and missed the bug; the test must replicate strict client parsing. Load-bearing recurrence guard.

---

## Error-path scope

| Option | Description | Selected |
|--------|-------------|----------|
| Keep JSON-RPC errors | Fix success path only; leave -32601/-32602/-32603 envelope | ✓ |
| Convert to isError result | Turn tool failures into CallToolResult{isError:true} | |

**User's choice:** Keep JSON-RPC error envelope (auto / recommended)
**Notes:** Phase scope is success-path result content blocks; JSON-RPC errors are already valid. Conversion deferred.

---

## Claude's Discretion

- Helper naming for assembling the structured value.
- Inline vs small-step serialization of the `"result"` JSON-RPC value.
- Compact vs pretty JSON inside the text block.

## Deferred Ideas

- Tool-level error results (`CallToolResult { isError: true }`) per MCP protocol-vs-tool error distinction.
- `_meta`-based pagination placement instead of structuredContent.
