---
name: ferro:info
description: Show Ferro project information via MCP introspection
allowed-tools:
  - Bash
  - Read
  - Glob
---

<objective>
Display comprehensive project information using ferro-mcp introspection tools.

This command gathers and presents:
- Project metadata (name, version)
- Installed Ferro crates
- Database connection status
- Registered models, routes, and services
</objective>

<process>

<step name="gather_info">

Use ferro-mcp tools to gather project information. Call the following MCP tools:

1. **application_info** - Get project overview, installed crates, configuration
2. **list_models** - Get all registered models
3. **list_routes** - Get route count summary
4. **list_services** - Get registered services

If ferro-mcp is not available, fall back to reading Cargo.toml and scanning the codebase.

</step>

<step name="check_database">

Check database configuration:

```bash
if [ -f .env ]; then
    grep -E "^DATABASE_" .env | sed 's/PASSWORD=.*/PASSWORD=***/'
fi
```

</step>

<step name="present_info">

Present information in a clean format:

```
# Ferro Project: {name}

## Overview
- Version: {version}
- Rust Edition: {edition}

## Ferro Crates
{list of ferro-* crates with versions}

## Database
- Driver: {driver}
- Database: {name}
- Status: {connected/not configured}

## Statistics
- Models: {count}
- Routes: {count}
- Services: {count}
- Middleware: {count}

## Quick Actions
- /ferro:routes - View all routes
- /ferro:models - View all models
- /ferro:db:schema - View database schema
```

</step>

</process>

<fallback>
If ferro-mcp tools are unavailable, scan the project directly:

1. Read Cargo.toml for project info
2. Glob `src/models/*.rs` for models
3. Glob `src/controllers/*.rs` for controllers
4. Read `src/routes.rs` for route definitions
</fallback>
