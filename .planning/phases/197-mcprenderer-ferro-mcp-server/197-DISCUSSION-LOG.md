# Phase 197: McpRenderer & ferro-mcp-server - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-10
**Mode:** --auto (recommended defaults auto-selected)
**Areas discussed:** Crate placement, Opt-in marker, MCP protocol types, inputSchema derivation, Dispatch/read path, Publish wave

---

## Opt-in exposure marker

| Option | Description | Selected |
|--------|-------------|----------|
| `mcp_exposed: bool` on `ServiceDef` + builder | Plain metadata, no renderer dep; matches SC-1 wording | ✓ |
| Registration list in ferro-mcp-server | Exposure decided in the renderer crate, not the projection | |
| `intent_hint` variant | Overloads the intent-hint channel | |

**Choice:** [auto] bool on ServiceDef (recommended). Keeps exposure co-located with the projection; SC-4 holds (a bool is not a renderer dependency).

---

## MCP protocol types

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse workspace `rmcp` 0.12 | Already used by ferro-mcp/ferro-api-mcp; aligns with Phase 198 transport | ✓ |
| Hand-roll minimal MCP tool types | Lighter renderer crate, but reinvents protocol | |

**Choice:** [auto] reuse rmcp (recommended), emit serde_json::Value where rmcp's type is awkward. RESEARCH FLAG: confirm rmcp Tool ergonomics for pure emission; fall back to a minimal local struct if it couples to the server runtime.

---

## inputSchema derivation

| Option | Description | Selected |
|--------|-------------|----------|
| Pagination + conservative equality filters from ServiceDef fields | limit/offset + Identifier/FK readable non-sensitive fields; DataType→JSON-Schema | ✓ |
| Pagination only (no filters) | Even thinner, but under-delivers AMCP-02's "filter fields" | |
| All readable fields as filters | Too broad a surface for a skeleton | |

**Choice:** [auto] pagination + conservative filters (recommended). RESEARCH FLAG: exact filter-field predicate against FieldMeaning.

---

## Dispatch / read path

| Option | Description | Selected |
|--------|-------------|----------|
| Dispatch fn (separate from render) reusing an existing read mechanism | Resolves source model + read-only list; conn/context param for later tenant scoping | ✓ |
| Reimplement query logic in the renderer | Duplicates existing read code | |

**Choice:** [auto] reuse existing read path (recommended). RESEARCH FLAG (load-bearing): identify the exact reusable read fn (crud_list / render_projection data path / source-model resolution) and whether a live DB is needed for the in-process test.

---

## Crate placement & publish wave

| Option | Description | Selected |
|--------|-------------|----------|
| New ferro-mcp-server output crate, mirror ferro-json-ui, publish Wave 2 | Renderer-in-output-crate rule; depends on ferro-projections (1B) | ✓ |
| Put McpRenderer in framework/ferro-projections | Violates renderers-in-output-crate; SC-4 breach | |

**Choice:** [auto] new crate, Wave 2 (recommended). New crate needs one-time manual bootstrap publish (CI token is publish-update only).

---

## Claude's Discretion

- Module layout; McpRenderer Output/Context associated types; in-process exercise naming; limit cap value.

## Deferred Ideas

- HTTP transport (198), OAuth (199), per-tenant scoping (200), write intents / auto-exposure / MCP App UI (later).
