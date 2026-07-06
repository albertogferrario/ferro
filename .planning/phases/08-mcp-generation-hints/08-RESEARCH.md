# Phase 8 Research: MCP Generation Hints

## Research Question

How should cancer-mcp embed generation hints in introspection responses to help AI agents generate framework-appropriate code?

## Standard Stack

### 1. llms.txt Specification
- **What**: Markdown format for providing LLM context at `/llms.txt` endpoint
- **Adoption**: 600+ sites including Anthropic, Cloudflare, Stripe
- **Format**: H1 title, blockquote summary, markdown sections, optional file lists
- **Use case**: Provide curated documentation context for LLMs
- **Relevance**: Could expose cancer framework patterns as `/llms.txt`

### 2. AGENTS.md Specification
- **What**: Standard format for guiding coding agents (Linux Foundation stewardship)
- **Location**: `.github/AGENTS.md` or `AGENTS.md` in repo root
- **Content**: Coding conventions, testing requirements, PR guidelines
- **Use case**: Agent reads this before making changes
- **Relevance**: Cancer apps could include AGENTS.md with framework conventions

### 3. MCP Output Schemas
- **What**: MCP spec supports semantic type annotations in tool outputs
- **Types**: datetime, email, file_path, identifier, etc.
- **Hints**: Tool descriptions can include structured hints for models
- **Relevance**: Cancer-mcp tools already have descriptions; can add generation hints

## Architecture Patterns

### Laravel Boost Approach (Reference Implementation)
Laravel Boost provides MCP server for Laravel framework:

1. **15+ Specialized Tools**:
   - `search_laravel_docs` - Vectorized search across 17k doc sections
   - `get_migration_status` - Database state
   - `get_route_list` - Route introspection
   - `artisan` - Run any artisan command
   - `composer` - Manage dependencies

2. **Composable AI Guidelines**:
   - `/ai.json` endpoint returns framework-specific rules
   - Agent can fetch and apply guidelines before code generation
   - Includes naming conventions, patterns, anti-patterns

3. **Documentation API**:
   - Semantic search over Laravel docs
   - Returns relevant examples with code snippets
   - Context-aware based on current task

### Key Insight
Laravel Boost separates concerns:
- **Tools**: Return raw data (routes, models, migrations)
- **Guidelines**: Return conventions/patterns (naming, structure)
- **Docs**: Return examples/templates (code snippets)

Cancer-mcp could adopt similar separation.

## Don't Hand-Roll

1. **Use MCP's built-in hint mechanisms** - Don't invent custom formats
2. **Leverage existing tool descriptions** - Already structured (When/Returns/Combine)
3. **Don't duplicate documentation** - Reference existing docs, don't embed full copies
4. **Avoid complex template engines** - Simple string templates sufficient
5. **Don't over-structure** - Agents handle natural language well

## Common Pitfalls

### 1. Raw Data Without Context
**Problem**: Returning model fields without explaining how to use them
```rust
// Current: Just data
{"name": "User", "fields": ["id", "email", "password_hash"]}

// Better: Data + hints
{
  "name": "User",
  "fields": [...],
  "hints": {
    "create_migration": "sea-orm-cli migrate generate create_users_table",
    "never_expose": ["password_hash"],
    "validation": {"email": "email()", "password": "min(8)"}
  }
}
```

### 2. Missing Code Templates
**Problem**: Agent knows structure but not syntax
```rust
// Current: Model info only
{"name": "Post", "table": "posts", "relations": ["belongs_to: User"]}

// Better: Include template
{
  "name": "Post",
  ...,
  "templates": {
    "handler": "pub async fn show(req: Request) -> Response {\n    let post = Post::find_by_id(...).await?;\n    Ok(json!(post))\n}",
    "validation": "Validator::new(&data).rules(\"title\", rules![required()]).validate()"
  }
}
```

### 3. No Negative Examples
**Problem**: Agent doesn't know what to avoid
```rust
// Better: Include warnings
{
  "hints": {
    "avoid": [
      "Don't use Entity::find() without pagination for large tables",
      "Don't expose password_hash in JSON responses",
      "Don't skip validation for user input"
    ]
  }
}
```

### 4. Missing Import Context
**Problem**: Agent generates code but forgets imports
```rust
// Better: Include required imports
{
  "code_template": "...",
  "imports": [
    "use cancer::prelude::*;",
    "use crate::models::user::User;"
  ]
}
```

## Code Examples

### Enhanced Model Response
```rust
#[derive(Debug, Serialize)]
pub struct ModelDetailsWithHints {
    // Existing fields
    pub name: String,
    pub table: Option<String>,
    pub path: String,
    pub fields: Vec<FieldInfo>,

    // Generation hints
    pub hints: ModelHints,
}

#[derive(Debug, Serialize)]
pub struct ModelHints {
    pub handler_template: String,
    pub validation_rules: HashMap<String, String>,
    pub never_expose: Vec<String>,
    pub common_queries: Vec<QueryExample>,
    pub related_files: Vec<String>,
}
```

### Enhanced Route Response
```rust
#[derive(Debug, Serialize)]
pub struct RouteInfoWithHints {
    // Existing fields
    pub method: String,
    pub path: String,
    pub handler: String,

    // Generation hints
    pub hints: RouteHints,
}

#[derive(Debug, Serialize)]
pub struct RouteHints {
    pub request_format: Option<String>,  // Expected request body
    pub response_format: Option<String>, // Expected response
    pub similar_routes: Vec<String>,     // Related endpoints
    pub test_template: String,           // How to test this route
}
```

### New Tool: generation_context
```rust
// Returns comprehensive context for code generation
{
  "naming_conventions": {
    "models": "PascalCase singular (User, BlogPost)",
    "tables": "snake_case plural (users, blog_posts)",
    "handlers": "snake_case verb (show, create, update)",
    "routes": "RESTful (GET /users, POST /users, GET /users/:id)"
  },
  "file_structure": {
    "handlers": "src/handlers/{resource}.rs",
    "models": "src/models/{resource}.rs",
    "migrations": "migration/src/{timestamp}_{name}.rs"
  },
  "common_patterns": {
    "crud_handler": "...",
    "validation": "...",
    "error_handling": "..."
  },
  "avoid": [
    "Don't use unwrap() in handlers - return proper errors",
    "Don't skip validation for POST/PUT requests",
    "Don't hardcode configuration values"
  ]
}
```

## Recommendations

### Approach 1: Enhance Existing Tools (Recommended)
Add optional `include_hints` parameter to existing tools:
- `list_models(include_hints: true)` - Returns ModelDetailsWithHints
- `list_routes(include_hints: true)` - Returns RouteInfoWithHints
- Backwards compatible, opt-in enhancement

### Approach 2: New Dedicated Tools
Add specialized tools for generation context:
- `generation_context` - Framework conventions and patterns
- `code_templates` - Templates for common operations
- `validation_hints` - Field-specific validation rules

### Approach 3: Hybrid (Best)
Combine both approaches:
1. Add `include_hints` to existing tools for inline context
2. Add `generation_context` tool for comprehensive conventions
3. Add `code_templates` tool for copy-paste templates

## Sources

1. **llms.txt Specification**: https://llmstxt.org/
2. **AGENTS.md Standard**: https://agents-md.org/ (referenced in search results)
3. **Laravel Boost**: https://github.com/nicksarafa/laravel-boost-mcp
4. **MCP Specification**: https://modelcontextprotocol.io/
5. **Addy Osmani's AI Workflow**: Referenced in search results on LLM code generation
6. **Simon Willison's Patterns**: Referenced in search results on prompt engineering

## Next Steps

1. Create Phase 8 plan based on Approach 3 (Hybrid)
2. Prioritize high-impact enhancements:
   - `generation_context` tool (conventions + patterns)
   - `code_templates` tool (handler, model, migration templates)
   - `include_hints` parameter for `list_models` and `get_handler`
3. Update CANCER_MCP_INSTRUCTIONS with generation workflow
