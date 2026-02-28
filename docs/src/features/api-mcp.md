# MCP Bridge (ferro-api-mcp)

ferro-api-mcp is a standalone binary that bridges any Ferro REST API to the Model Context Protocol (MCP). AI agents can discover and call your API endpoints as MCP tools without custom integration code.

## Quick Start Workflow

From scaffold to working MCP integration in seven steps:

1. **Scaffold the API:**
   ```bash
   ferro make:api --all
   ```

2. **Wire routes** in `src/main.rs`:
   ```rust
   mod api;
   // In route registration:
   api::routes::api_routes()
   api::docs::docs_routes()
   ```

3. **Run the migration:**
   ```bash
   ferro db:migrate
   ```

4. **Generate an API key:**
   ```bash
   ferro make:api-key "My Key"
   ```
   Save the raw key -- it is shown only once.

5. **Start the server:**
   ```bash
   cargo run
   ```

6. **Verify the setup:**
   ```bash
   ferro api:check --api-key fe_live_...
   ```

7. **Add MCP config** to your AI agent (see [MCP Host Configuration](#mcp-host-configuration) below).

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

## Route Customization

Override the auto-generated MCP metadata on individual routes using builder methods:

```rust
use ferro::routing::*;

group!("/api/v1")
    .middleware(ApiKeyMiddleware::new())
    .routes([
        get!("/users", user_api::index)
            .mcp_tool_name("search_users")
            .mcp_description("Search users by name or email with pagination")
            .mcp_hint("Use page and per_page params for large result sets"),

        post!("/users", user_api::store)
            .mcp_description("Create a new user account"),

        delete!("/users/:id/sessions", user_api::clear_sessions)
            .mcp_hidden(),  // Exclude from MCP tools
    ])
```

### Available Methods

| Method | Effect | When to Use |
|--------|--------|-------------|
| `.mcp_tool_name("name")` | Override auto-generated tool name | When the default name is unclear (e.g., `store_user` -> `create_user_account`) |
| `.mcp_description("desc")` | Override auto-generated description | When the default summary needs more context for AI agents |
| `.mcp_hint("hint")` | Append hint text to description | To guide AI agents on parameter usage or expected behavior |
| `.mcp_hidden()` | Exclude route from MCP tools | For internal/admin endpoints that agents should not call |

### Group-Level Defaults

Set MCP defaults at the group level. Route-level overrides take precedence:

```rust
group!("/api/v1/internal")
    .mcp_hidden()  // Hide all routes in this group
    .routes([
        get!("/health", internal_api::health),
        get!("/metrics", internal_api::metrics),
    ])
```

### How It Works

Customizations are stored in the route registry and emitted as `x-mcp` vendor extensions in the OpenAPI spec. ferro-api-mcp reads these extensions at startup, using them as overrides over auto-generated values.

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
