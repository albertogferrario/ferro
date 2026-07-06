# ferro-mcp-server

MCP tool rendering target for Ferro projections.

`McpRenderer` implements the `Renderer` trait from `ferro-projections`, translating a `ServiceDef` projection into an MCP tool definition. The same `ServiceDef` that renders to JSON-UI (visual output via `ferro-json-ui`) also renders to an MCP tool schema and tool output via this crate. One projection source; multiple rendering targets.
