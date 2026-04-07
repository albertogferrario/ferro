# ferro-api-mcp

Standalone MCP server that bridges OpenAPI specs to MCP tools.

Loads any OpenAPI v3 specification and exposes its operations as Model Context Protocol tools, allowing LLM clients to invoke arbitrary HTTP APIs through a uniform interface. Handles schema translation, request construction, and response forwarding over stdio transport.

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-api-mcp

License: MIT
