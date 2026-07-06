# ferro-mcp

MCP (Model Context Protocol) server for AI-assisted Ferro framework development.

Similar to Laravel Boost, this crate provides introspection tools that AI agents can use to understand and work with Ferro applications. It is launched in-process by the `ferro mcp` subcommand and exposes 80+ tools grouped by category.

## Tool categories

### Domain understanding
`domain_glossary`, `explain_route`, `explain_model`

### Application introspection
`application_info`, `list_routes`, `list_models`, `list_resources`, `list_policies`, `list_middleware`, `list_services`, `list_events`, `list_jobs`, `list_migrations`, `list_commands`, `list_rate_limiters`, `list_broadcast_channels`, `list_lang_files`, `list_projections`, `list_props`, `get_handler`, `get_middleware`, `get_config`

### Database
`db_schema`, `db_query`, `relation_map`, `session_inspect`

### Debugging
`last_error`, `diagnose_error`, `error_patterns`, `read_logs`, `browser_logs`, `test_route`

### Background jobs and cache
`job_history`, `queue_status`, `cache_inspect`

### Relationship analysis
`route_dependencies`, `model_usages`, `dependency_graph`, `model_usages`

### Code generation
`generation_context`, `code_templates`, `create_project`, `generate_types`, `crud_create`, `crud_list`, `crud_update`, `crud_delete`

### JSON-UI
`json_ui_catalog`, `json_ui_inspect`, `json_ui_generate`

### Projections
`inspect_projection`, `render_projection`, `validate_projection`, `projection_coverage`

### Contract validation
`validate_contracts`, `inspect_props`

### Integrations
`stripe_config_status`, `stripe_subscription_info`, `stripe_webhook_events`, `whatsapp_config_status`, `whatsapp_webhook_events`, `test_classifier`, `list_pending_confirmations`

### Metrics and docs
`request_metrics`, `search_docs`, `tinker`

See the [mdBook documentation](https://docs.ferro-rs.dev/) for the complete tool reference with schemas.

## Usage

### Start the MCP server

```bash
ferro mcp
```

### Install for an AI editor

```bash
ferro boost:install
```

This configures the MCP server for your editor (Cursor, VS Code, Claude).

### Manual configuration

Add to your MCP configuration:

```json
{
  "mcpServers": {
    "ferro": {
      "command": "cargo",
      "args": ["run", "--package", "ferro-mcp"],
      "cwd": "/path/to/your/project"
    }
  }
}
```

## Example tool output

### `application_info`

```json
{
  "framework_version": "0.2.0",
  "rust_version": "rustc 1.75.0",
  "database_engine": "sqlite",
  "environment": "local",
  "installed_crates": [
    {"name": "ferro-rs", "version": "0.2.0"},
    {"name": "ferro-events", "version": "0.2.0"}
  ],
  "models": [
    {"name": "User", "table": "users", "path": "src/models/users.rs"}
  ]
}
```

### `list_routes`

```json
{
  "routes": [
    {"method": "GET", "path": "/users", "handler": "users_controller::index"},
    {"method": "POST", "path": "/users", "handler": "users_controller::store"}
  ]
}
```

## License

MIT
