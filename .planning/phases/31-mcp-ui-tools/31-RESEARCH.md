# Phase 31: MCP UI Tools - Research

**Researched:** 2026-02-09
**Domain:** Ferro MCP server extension for JSON-UI introspection and generation
**Confidence:** HIGH

<research_summary>
## Summary

Researched the existing ferro-mcp infrastructure, the ferro-json-ui crate's public API, the framework's render integration, and the CLI's `make:json-view` AI generation pipeline to determine what MCP tools Phase 31 needs.

The ferro-mcp server currently has zero JSON-UI awareness — no tools reference JSON-UI types, views, components, or the render pipeline. Phase 31 bridges this gap by adding tools that let AI agents inspect existing JSON-UI views, understand the component catalog, generate new views from models/routes, and validate view correctness.

The implementation is entirely internal to the Ferro codebase. No external libraries or ecosystem research is needed. The key challenge is designing tools that provide the right level of abstraction for agent consumption — not too low-level (raw types) and not too high-level (black-box generation).

**Primary recommendation:** Add 4-6 focused MCP tools following the existing `code_templates` / `generation_context` pattern. Reuse the CLI's AI context assembly (`scan_models`, `scan_routes`, `COMPONENT_CATALOG`) for the generation tool, and add introspection tools for existing views.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already in crate)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rmcp | 0.12 | MCP server SDK | Already used, provides `#[tool]` macros |
| serde/serde_json | 1.0 | JSON serialization | Already used everywhere |
| syn | 2.x | Rust source parsing | Already used for model introspection |
| regex | 1.x | Pattern matching | Already used for route/handler scanning |
| walkdir | 2.x | Directory traversal | Already used for file discovery |
| schemars | 1.x | JSON Schema for tool params | Already used for MCP param types |

### New Dependencies Needed
None. All required functionality is available through existing dependencies.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Regex model scanning | syn AST visiting | syn already used in models.rs; CLI uses regex for speed — keep both patterns |
| Hardcoded component catalog | Runtime type reflection | Reflection too complex; hardcoded catalog matches CLI approach |
| Per-tool view generation | Single omnibus tool | Multiple focused tools better for agent workflow flexibility |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Existing MCP Tool Structure

Every MCP tool in ferro-mcp follows this pattern:

```
ferro-mcp/src/tools/{tool_name}.rs  — Pure logic module
ferro-mcp/src/service.rs            — Tool registration via #[tool] macro
```

**Tool module pattern:**
```rust
// tools/{name}.rs
pub struct ToolOutput { /* serializable response fields */ }
pub fn execute(project_root: &Path, /* params */) -> Result<ToolOutput, Error> { ... }
```

**Service registration pattern:**
```rust
// service.rs
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ToolParams { pub field: Option<String> }

#[tool(name = "tool_name", description = "...")]
pub async fn tool_name(&self, params: Parameters<ToolParams>) -> String {
    match tools::tool_name::execute(&self.project_root, ...) {
        Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}
```

### Recommended Tool Architecture for Phase 31

```
ferro-mcp/src/tools/
├── json_ui_catalog.rs       # Component catalog reference
├── json_ui_inspect.rs       # Inspect existing JSON-UI views
├── json_ui_generate.rs      # Generate view from model/route context
├── json_ui_validate.rs      # Validate view correctness
└── mod.rs                   # Add pub mod declarations
```

### Pattern 1: Catalog Tool (Static Reference)
**What:** Return the component catalog with types, props, variants, and examples
**When to use:** Agent needs to understand available components before building a view
**Follows:** Same pattern as `code_templates.rs` — hardcoded content, no project scanning
```rust
pub fn execute(component: Option<&str>) -> JsonUiCatalog {
    // Return component details, optionally filtered by name
}
```

### Pattern 2: Inspection Tool (Source Scanning)
**What:** Scan `src/views/*.rs` files for existing JSON-UI view functions
**When to use:** Agent needs to understand existing views before modifying or adding new ones
**Follows:** Same pattern as `list_models.rs` — scan source files with regex/syn
```rust
pub fn execute(project_root: &Path, filter: Option<&str>) -> Result<Vec<ViewInfo>, Error> {
    // Walk src/views/, parse view functions, extract component structure
}
```

### Pattern 3: Generation Tool (Context Assembly + Output)
**What:** Assemble context from models/routes and produce a JSON-UI view
**When to use:** Agent wants to create a new view for a given model or page
**Follows:** Same approach as CLI's `ai.rs::build_view_context()` but returns structured data instead of calling Anthropic
```rust
pub fn execute(
    project_root: &Path,
    model: Option<&str>,
    description: Option<&str>,
) -> Result<GenerationContext, Error> {
    // Return: component catalog + model fields + routes + example code
    // Agent uses this context to write the view itself
}
```

### Anti-Patterns to Avoid
- **Calling Anthropic API from MCP tool:** The MCP tools should provide context, not call AI. The agent using the tools IS the AI.
- **Duplicating the CLI's generation logic:** The CLI generates complete files; MCP tools should provide building blocks and context.
- **Returning raw Rust source:** MCP tools return structured JSON. Source code snippets are ok as examples, but primary output should be typed data.
- **Monolithic "do everything" tool:** Prefer composable tools: catalog for reference, inspect for existing views, generate-context for creation.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Component catalog | Manual component list | Reuse `COMPONENT_CATALOG` from ferro-cli's `ai.rs` | Already maintained, includes all 20 components |
| Model scanning | New model scanner | Reuse `scan_models()` from ferro-cli or `introspection::models` | Already works, tested |
| Route scanning | New route scanner | Reuse `introspection::routes::scan_routes()` | Already works, tested |
| View file scanning | Complex AST parsing | Regex-based scanning (same as CLI) | Views follow predictable patterns |
| Component type enumeration | Hard-code component names | Derive from ferro-json-ui's `Component` enum docs | Stay in sync with crate |
| Tool param types | Custom validation | schemars `JsonSchema` derive | Framework convention |

**Key insight:** Phase 31 is mostly about exposing *existing* information through the MCP protocol. The data sources (component catalog, model fields, route definitions, view source files) already exist. The work is structuring them for agent consumption.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Duplicating CLI Context Assembly
**What goes wrong:** Building a parallel `scan_models()` in ferro-mcp that diverges from ferro-cli's version
**Why it happens:** Different crates, tempting to rewrite
**How to avoid:** Either extract shared scanning into a common crate, or keep scanning in ferro-mcp's existing `introspection` module which already has model/route scanning
**Warning signs:** Two different regex patterns for the same thing in different crates

### Pitfall 2: Tool Output Too Large
**What goes wrong:** A single MCP tool returns the entire component catalog + all models + all routes + all views
**Why it happens:** Trying to be "helpful" by returning everything at once
**How to avoid:** Keep tools focused. Catalog tool returns components. Inspect tool returns views. Generate-context returns what's needed for a specific view.
**Warning signs:** Tool response exceeding 50KB, agent context window pressure

### Pitfall 3: Inconsistent Component Catalog
**What goes wrong:** MCP catalog lists different components or props than ferro-json-ui actually has
**Why it happens:** Hardcoded catalog not updated when ferro-json-ui changes
**How to avoid:** The component catalog should be the single source of truth. Consider generating it from the actual types or referencing the same constant used by the CLI.
**Warning signs:** Agent generates view code that doesn't compile because it uses wrong prop names

### Pitfall 4: Missing JSON-UI in MCP Instructions
**What goes wrong:** MCP instructions string doesn't mention JSON-UI tools, so agents don't know they exist
**Why it happens:** Forgetting to update `FERRO_MCP_INSTRUCTIONS` const in service.rs
**How to avoid:** Update the instructions string with JSON-UI tool category, when-to-use guidance, and workflow documentation
**Warning signs:** Agents using code_templates for JSON-UI instead of the dedicated tools

### Pitfall 5: Not Updating Existing Tools
**What goes wrong:** `application_info` doesn't mention JSON-UI views, `code_templates` doesn't have JSON-UI category
**Why it happens:** Phase 31 focuses on new tools but forgets to update existing ones
**How to avoid:** Audit existing tools that should be JSON-UI aware: `application_info`, `code_templates`, `generation_context`, `dependency_graph`
**Warning signs:** Agent asks about JSON-UI but gets no information from standard discovery tools
</common_pitfalls>

<code_examples>
## Code Examples

### Existing Tool Pattern (from code_templates.rs)
```rust
// Source: ferro-mcp/src/tools/code_templates.rs
#[derive(Debug, Serialize)]
pub struct CodeTemplates {
    pub templates: Vec<CodeTemplate>,
}

pub fn execute(category: Option<&str>) -> CodeTemplates {
    let all_templates = build_templates();
    let templates = match category {
        Some(cat) => all_templates.into_iter().filter(|t| t.category == cat).collect(),
        None => all_templates,
    };
    CodeTemplates { templates }
}
```

### Service Registration (from service.rs)
```rust
// Source: ferro-mcp/src/service.rs
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CodeTemplatesParams {
    /// Filter by category: handler, model, migration, middleware, validation
    pub category: Option<String>,
}

#[tool(name = "code_templates", description = "Get copy-paste code templates...")]
pub async fn code_templates(&self, params: Parameters<CodeTemplatesParams>) -> String {
    let templates = tools::code_templates::execute(params.0.category.as_deref());
    serde_json::to_string_pretty(&templates).unwrap_or_else(|_| "{}".to_string())
}
```

### CLI Context Assembly (from ferro-cli/src/ai.rs)
```rust
// Source: ferro-cli/src/ai.rs — model scanning for view generation
fn scan_models(project_root: &Path) -> String {
    let models_dir = project_root.join("src/models");
    // Regex: pub struct (\w+) \{
    // Extract fields: pub (\w+) : ([^,\n]+)
    // Output: ### ModelName: field (type), field (type)
}

fn scan_routes(project_root: &Path) -> String {
    let routes_file = project_root.join("src/routes.rs");
    // Reuses generate_routes::parse_routes_file()
    // Output: METHOD path -> handler_module::handler_fn (name: "route.name")
}
```

### Component Catalog Structure (from ferro-cli/src/ai.rs)
```rust
// Source: ferro-cli/src/ai.rs — COMPONENT_CATALOG const
// 20 components defined with full prop signatures:
// Text, Button, Card, Table, Form, Input, Select, Alert, Badge,
// Modal, Checkbox, Switch, Separator, DescriptionList, Tabs,
// Breadcrumb, Pagination, Progress, Avatar, Skeleton
// Plus: Action, ComponentNode, JsonUiView builder API
```

### JSON-UI View Pattern (typical output)
```rust
// Source: typical generated view
use ferro::{
    ComponentNode, Component, CardProps, JsonUiView, TextElement, TextProps,
    TableProps, Column, Action, ButtonProps, ButtonVariant, Size,
};

pub fn view() -> JsonUiView {
    JsonUiView::new()
        .title("Users")
        .layout("app")
        .component(ComponentNode {
            key: "users-table".to_string(),
            component: Component::Table(TableProps {
                columns: vec![
                    Column { key: "id".to_string(), label: "ID".to_string(), format: None },
                    Column { key: "name".to_string(), label: "Name".to_string(), format: None },
                ],
                data_path: "/data/users".to_string(),
                row_actions: Some(vec![Action::get("users.show")]),
                empty_message: Some("No users found".to_string()),
                sortable: None, sort_column: None, sort_direction: None,
            }),
            action: None,
            visibility: None,
        })
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| N/A — no JSON-UI MCP tools exist | Adding them in Phase 31 | 2026-02-09 | Enables agent-driven JSON-UI development |
| CLI-only view generation | MCP tools + CLI generation | Phase 31 | Agents can introspect and generate views without CLI |

**Key context:**
- ferro-json-ui is stable (Phases 23-30 complete, schema v1)
- 20 components fully implemented with HTML rendering
- CLI's `make:json-view` proves the AI generation pattern works
- MCP server has 35+ existing tools as reference for conventions

**No external dependencies or ecosystem changes needed.** This is purely internal tooling extension.
</sota_updates>

<open_questions>
## Open Questions

1. **Should the generate tool call Anthropic or just provide context?**
   - What we know: The CLI calls Anthropic directly for view generation. MCP tools are consumed by AI agents.
   - What's unclear: Does it make sense for an MCP tool to call an LLM when the consumer IS an LLM?
   - Recommendation: Provide structured context (models, routes, catalog, examples). Let the consuming agent generate the view code. This avoids double-LLM calls and gives the agent more control.

2. **Should we extract shared scanning utilities into a common crate?**
   - What we know: Both ferro-cli and ferro-mcp scan models/routes with similar but different code
   - What's unclear: Whether the maintenance burden justifies a shared crate
   - Recommendation: For Phase 31, reuse ferro-mcp's existing `introspection` module. Consider shared crate in a future cleanup phase if patterns diverge.

3. **How many tools vs one comprehensive tool?**
   - What we know: Existing MCP tools are focused (one tool = one concern). Agent workflows compose multiple tools.
   - What's unclear: Whether 4-6 tools or 2-3 would serve agents better
   - Recommendation: Start with focused tools matching the existing pattern. The MCP instructions workflow guidance handles composition.

4. **Should `code_templates` get a json-ui category?**
   - What we know: `code_templates` currently covers handler, model, migration, middleware, validation
   - What's unclear: Whether json-ui templates belong there or in a dedicated tool
   - Recommendation: Add a `json_view` category to `code_templates` for consistency, AND have a dedicated catalog tool for the full component reference.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- ferro-mcp/src/service.rs — All 35 existing tool registrations, pattern reference
- ferro-mcp/src/tools/ — All tool implementation modules
- ferro-mcp/src/introspection/ — Model, route, event, job scanning
- ferro-json-ui/src/ — Complete crate: 20 components, actions, visibility, render, layout
- ferro-cli/src/ai.rs — COMPONENT_CATALOG, context assembly, Anthropic API integration
- ferro-cli/src/commands/make_json_view.rs — View generation command
- framework/src/json_ui.rs — Framework integration (render_json, resolve helpers)

### Secondary (MEDIUM confidence)
- rmcp 0.12 Context7 docs — Tool macro patterns, Parameters type, ToolRouter
- CLAUDE.md instructions — ferro-mcp update requirements when features change

### Tertiary (LOW confidence - needs validation)
- None — all findings from direct source code analysis
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: ferro-mcp (rmcp 0.12 Rust MCP server)
- Ecosystem: ferro-json-ui, ferro-cli, framework integration
- Patterns: Existing MCP tool patterns, CLI context assembly
- Pitfalls: Catalog consistency, tool output size, instruction updates

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies needed
- Architecture: HIGH — follows well-established patterns from 35 existing tools
- Pitfalls: HIGH — based on concrete code analysis and known maintenance patterns
- Code examples: HIGH — all from actual codebase

**Research date:** 2026-02-09
**Valid until:** 2026-03-09 (30 days — internal patterns, stable codebase)
</metadata>

---

*Phase: 31-mcp-ui-tools*
*Research completed: 2026-02-09*
*Ready for planning: yes*
