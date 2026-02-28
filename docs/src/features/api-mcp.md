# MCP Bridge (ferro-api-mcp)

ferro-api-mcp is a standalone binary that bridges any Ferro REST API to the Model Context Protocol (MCP). AI agents can discover and call your API endpoints as MCP tools without custom integration code.

## How It Works

1. Reads the OpenAPI spec from your Ferro app's `/api/docs/openapi.json` endpoint
2. Converts each API operation into an MCP tool with typed input schemas
3. Runs as a stdio MCP server that AI agents connect to
4. Supports `x-mcp` vendor extensions for customizing tool names, descriptions, hints, and visibility

## Prerequisites

- A Ferro app with `make:api` scaffold (see [REST API](api.md))
- The API running and accessible (e.g., `ferro serve` on localhost:8080)
- An API key generated via `ferro make:api` setup

## Setup

### Building

```bash
cargo build --release -p ferro-api-mcp
```

Binary location: `target/release/ferro-api-mcp`

### CLI Options

```
ferro-api-mcp [OPTIONS] --spec-url <URL>

Options:
  --spec-url <URL>    URL to fetch the OpenAPI spec from
  --api-key <KEY>     API key for Authorization header (optional)
  --base-url <URL>    Override the base URL for API calls
  --log-level <LEVEL> Log level: debug, info, warn, error [default: info]
  --dry-run           Validate spec and print tool summary without starting server
```

### Validating Setup

```bash
ferro-api-mcp --spec-url http://localhost:8080/api/docs/openapi.json \
  --api-key your-api-key \
  --dry-run
```

Expected output:

```
Fetched spec: 4521 bytes

ferro-api-mcp v0.1.0
API: My App
Base URL: http://localhost:8080/
Tools: 5 registered

Tools:
  - list_users: List all users with pagination.
  - create_user: Create a new user.
  - show_user: Retrieve a single user by ID.
  - update_user: Update an existing user.
  - delete_user: Delete a user by ID.

Dry run complete. 5 tools validated.
```

## MCP Host Configuration

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```json
{
  "mcpServers": {
    "my-app": {
      "command": "/path/to/ferro-api-mcp",
      "args": [
        "--spec-url", "http://localhost:8080/api/docs/openapi.json",
        "--api-key", "your-api-key"
      ]
    }
  }
}
```

### Claude Code

Add to `.claude.json` (project-level) or `~/.claude.json` (global):

```json
{
  "mcpServers": {
    "my-app": {
      "command": "/path/to/ferro-api-mcp",
      "args": [
        "--spec-url", "http://localhost:8080/api/docs/openapi.json",
        "--api-key", "your-api-key"
      ]
    }
  }
}
```

### Cursor

Add via Settings > MCP Servers:

```json
{
  "my-app": {
    "command": "/path/to/ferro-api-mcp",
    "args": [
      "--spec-url", "http://localhost:8080/api/docs/openapi.json",
      "--api-key", "your-api-key"
    ]
  }
}
```

## x-mcp Extensions

Ferro's `build_openapi_spec()` automatically emits `x-mcp` vendor extensions on each operation. ferro-api-mcp reads these at startup to customize tool behavior.

| Extension | Effect |
|-----------|--------|
| `x-mcp-tool-name` | AI-friendly snake_case tool name (e.g., `list_users`) |
| `x-mcp-description` | AI-optimized description for the tool |
| `x-mcp-hint` | Usage hint appended to tool description |
| `x-mcp-hidden` | Set to `true` to exclude the operation from MCP tools |

These are emitted automatically by the framework. No configuration is needed. ferro-api-mcp uses the extension values as overrides, falling back to auto-generated names and descriptions when extensions are absent.

## Troubleshooting

| Problem | Cause | Solution |
|---------|-------|----------|
| "Cannot connect to {url}" | API server not running | Start the server with `ferro serve` |
| "HTTP 401" on tool calls | Missing or invalid API key | Check `--api-key` matches a key in the database |
| "HTTP 404" on tool calls | Endpoint does not exist | Verify the API is running and the spec is current |
| "request timed out" | API slow or network issue | Check server logs, verify connectivity |
| "spec parsed but 0 operations" | Empty or malformed spec | Check `/api/docs/openapi.json` manually |
| "unsupported OpenAPI version" | Spec is not 3.0.x | ferro-api-mcp requires OpenAPI 3.0.x |
| Tool arguments rejected | Missing required fields | Check tool input schema for required params |

## Base URL Resolution

ferro-api-mcp resolves the API base URL in this order:

1. `--base-url` flag (explicit override)
2. `servers[0].url` from the OpenAPI spec
3. Origin of the `--spec-url` (scheme + host + port)

This means most setups need only `--spec-url`. Use `--base-url` when the API server is behind a reverse proxy or on a different host than the spec endpoint.
