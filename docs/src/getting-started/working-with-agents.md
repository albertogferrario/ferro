# Working with Agents

Ferro is built agent-first: agents get the same structural understanding of your application that developers do, via MCP (Model Context Protocol). Routes, models, handlers, validations, services, and code generation hints are all accessible as tool calls — no file-reading required.

## Setting Up ferro-mcp

Ferro's CLI binary doubles as an MCP server. The same `ferro` binary you use for migrations and scaffolding exposes a full suite of introspection tools when invoked with the `mcp` subcommand. There is no separate binary to install — build your project and the MCP server is ready.

```bash
cargo build
# ferro binary is now at target/debug/ferro (development) or target/release/ferro (production)
```

### MCP Configuration

#### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```json
{
  "mcpServers": {
    "ferro": {
      "command": "/path/to/your/app/target/debug/ferro",
      "args": ["mcp"],
      "type": "stdio"
    }
  }
}
```

Restart Claude Desktop after saving. Use `target/release/ferro` for production builds.

#### Claude Code (.claude.json)

Place at project root as `.claude.json` (project-scope, applies to this repo only) or `~/.claude.json` (user-scope, applies to all projects):

```json
{
  "mcpServers": {
    "ferro": {
      "command": "/path/to/your/app/target/debug/ferro",
      "args": ["mcp"],
      "type": "stdio"
    }
  }
}
```

#### Generic stdio (any MCP-compatible host)

Any MCP host that supports stdio transport (Cursor, Windsurf, etc.) uses this pattern:

```json
{
  "mcpServers": {
    "ferro": {
      "command": "/path/to/ferro",
      "args": ["mcp"]
    }
  }
}
```

The `type: "stdio"` field is optional — stdio is the default transport for most hosts.

## The Discovery Loop

The core agent workflow follows three steps:

1. **Orient** — Call `application_info` to understand the project: installed crates, configured features, database details, and overall application structure.

2. **Explore** — Use feature-specific tools to drill into the area of interest. Exploring routes: `list_routes` then `explain_route`. Understanding data: `list_models` then `explain_model` then `model_usages`. Diagnosing a problem: `last_error` or `diagnose_error`.

3. **Generate** — Call `code_templates` or `generation_context` to get scaffolding hints, then execute the appropriate CLI command to produce the actual code.

This loop repeats at each level of depth. An agent scaffolding a feature might call `application_info` once, `list_routes` to check for conflicts, `explain_model` to understand related data, and `code_templates` to identify the right CLI command — all before writing a single line of code.

## Common Workflows

### Exploring a new codebase

An agent joining an unfamiliar Ferro project starts with `application_info` to get the full picture: what crates are installed, what features are active, what database is configured. It follows with `list_routes` to map the API surface and `list_models` to understand the data layer. Within minutes the agent has the structural knowledge that a developer would need days to accumulate by reading source files.

### Diagnosing a build or runtime error

When a build fails or a request errors, the agent calls `last_error` to retrieve the most recent error Ferro captured, then `diagnose_error` with the error text to get a structured root-cause analysis with suggested fixes. The agent does not need to read log files or trace through stack frames manually.

### Understanding models and their relationships

For features that involve data, the agent calls `list_models` to get all model names, `explain_model` on the relevant models to understand their fields and constraints, and `model_usages` to see where a model is referenced across the codebase. `relation_map` gives a full graph of model relationships. This is enough context to generate correct queries, validations, and forms without reading individual source files.

### Generating code from templates

When an agent needs to scaffold a handler, model, or migration, it calls `code_templates` to get the list of available templates and their generation hints. Each template specifies the CLI command that produces it. The agent selects the appropriate template, reads the hint, and calls the CLI command. The scaffolded code follows Ferro conventions and integrates with the project's existing structure.

## Agent-to-CLI Workflow

The bridge between MCP introspection and code generation is explicit in Ferro: MCP tools return `generation_hints` that point directly to the CLI command that scaffolds the code.

**End-to-end example: scaffolding a new model**

1. The agent calls `code_templates` and receives a list of templates. The `model` template entry includes:
   ```
   generation_hint: "Use `ferro make:scaffold <ModelName>` to scaffold a new model with migration"
   ```

2. The agent reads the hint, identifies the CLI command: `ferro make:scaffold`.

3. The agent executes the CLI command:
   ```bash
   ferro make:scaffold Post
   ```

4. Ferro generates `src/models/post.rs` and a timestamped migration file with the correct SeaORM structure.

5. The agent reads the generated file using `get_handler` or directly, confirms the structure matches intent, and proceeds to add fields or relationships.

The key insight: the agent never guesses the CLI command syntax. `generation_context` and `code_templates` always provide the canonical invocation. The agent's job is to select the right template and supply the right arguments — the framework handles the rest.

**Other generation hints flow the same way:**

- `code_templates` → `ferro make:handler` — scaffold a request handler
- `code_templates` → `ferro make:migration` — scaffold a database migration
- `code_templates` → `ferro make:job` — scaffold a background job
- `generation_context` for the current route → `ferro make:controller` with the matching name

## Troubleshooting

**"command not found" when starting the MCP server**

The binary path in the MCP config must point to the `ferro` binary, not `ferro-mcp`. The binary is at `target/debug/ferro` (development) or `target/release/ferro` (production). There is no standalone `ferro-mcp` binary.

**"No tools available" after connecting**

Check the JSON syntax in your config file — a trailing comma or missing bracket will silently prevent the server from loading. Validate with a JSON linter. Also confirm the binary path is absolute, not relative.

**Tools return stale data after code changes**

The ferro MCP server reads your source code at startup. After changing models, routes, or handlers, rebuild the project (`cargo build`) and restart your MCP host so the server reloads with the updated source.

**`application_info` shows missing features**

Features are detected from `Cargo.toml` dependencies. If a crate is not in your workspace's `Cargo.toml`, it will not appear in `application_info`. Add the dependency and rebuild.

## See Also

- [MCP Bridge](../features/api-mcp.md) — `ferro-api-mcp`, the consumer-facing API MCP server that exposes your application's REST API to external agents (separate from the framework introspection covered here)
