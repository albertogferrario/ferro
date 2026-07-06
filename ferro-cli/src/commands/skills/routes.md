---
name: ferro:routes
description: List all registered routes
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
---

<objective>
Display all registered routes in the Ferro application.

Uses ferro-mcp list_routes tool for accurate runtime information, with fallback to static analysis.
</objective>

<process>

<step name="get_routes">

**Primary method:** Use ferro-mcp `list_routes` tool to get all registered routes.

The tool returns:
- HTTP method (GET, POST, PUT, DELETE, etc.)
- Path pattern
- Handler name
- Middleware stack

**Fallback method:** If MCP unavailable, parse route definitions:

```bash
# Find route files
find src -name "routes.rs" -o -name "*.rs" | xargs grep -l "Route::\|\.route(" 2>/dev/null
```

Then read and parse route definitions.

</step>

<step name="format_output">

Present routes in a table format:

```
┌────────┬─────────────────────┬─────────────────────┬─────────────────┐
│ Method │ Path                │ Handler             │ Middleware      │
├────────┼─────────────────────┼─────────────────────┼─────────────────┤
│ GET    │ /                   │ home::index         │ web             │
│ GET    │ /api/users          │ users::index        │ api, auth       │
│ POST   │ /api/users          │ users::store        │ api, auth       │
│ GET    │ /api/users/{id}     │ users::show         │ api, auth       │
│ PUT    │ /api/users/{id}     │ users::update       │ api, auth       │
│ DELETE │ /api/users/{id}     │ users::destroy      │ api, auth       │
└────────┴─────────────────────┴─────────────────────┴─────────────────┘

Total: 6 routes
```

</step>

<step name="offer_details">

After listing, offer to explain specific routes:

"Use `/ferro:route:explain <path>` for detailed information about a specific route."

</step>

</process>

<arguments>
- `[filter]` - Optional filter to match routes (e.g., `/ferro:routes api` shows only /api/* routes)
- `--json` - Output as JSON for programmatic use
</arguments>
