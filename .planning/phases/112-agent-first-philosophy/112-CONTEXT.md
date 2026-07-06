# Phase 112: Agent-First Philosophy - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Rewrite docs to lead with and consistently reinforce Ferro's agent-first identity. Every feature page makes MCP tools discoverable. Covers: introduction.md rewrite, "Working with Agents" guide, MCP tool references on feature pages, agent-to-CLI workflow documentation. No new framework code — docs-only phase.

</domain>

<decisions>
## Implementation Decisions

### Introduction rewrite
- Position as agent-first with dev appeal: "An agent-first Rust web framework with Laravel-inspired conventions"
- Supplement "Laravel of Rust" tagline — keep "Laravel-inspired" as descriptor, lead with agent-first
- Quick example stays as traditional handler code with an agent callout box: "AI agents can generate this automatically via MCP"
- Philosophy section restructured at Claude's discretion to match agent-first-with-dev-appeal tone

### Working with Agents guide
- Full agent setup guide: MCP config, discovery loop, common workflows, troubleshooting
- Lives in Getting Started section of SUMMARY.md, after Quick Start
- Include copy-paste MCP config snippets for Claude Desktop, Claude Code (.claude.json), and generic stdio
- Covers ferro-mcp (framework introspection) only — ferro-api-mcp already has its own page
- Agent-to-CLI workflow documented as a section within this guide (not a separate page)

### Agent-to-CLI workflow section
- Workflows selected at Claude's discretion — pick the ones that best demonstrate agent-first value
- Documentation format (prose vs pseudo-conversation) at Claude's discretion
- No visual diagrams — text descriptions are sufficient

### MCP tool references on feature pages
- Dedicated "## MCP Tools" section at the bottom of each feature page
- Detail level at Claude's discretion — simple tools get one-liners, complex tools get examples
- Skip feature pages that have no relevant MCP tools (no empty sections)
- Standardize existing MCP sections (ai.md, whatsapp.md) to match the new pattern
- api-mcp.md is already a dedicated MCP page — no changes needed there

### Claude's Discretion
- Philosophy section restructuring approach
- Which agent workflows to document (suggested: discovery loop, error diagnosis, model exploration, code generation)
- Documentation format for workflow section (prose vs pseudo-conversation)
- Per-tool detail level on feature pages (one-liner vs example)
- Exact wording of agent callout box in introduction

</decisions>

<specifics>
## Specific Ideas

- Keep "Laravel-inspired" as heritage/descriptor, but "agent-first" leads every positioning statement
- Agent callout on the introduction quick example should feel natural, not bolted on
- MCP config snippets should be copy-paste ready — minimize friction for first-time setup

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `docs/src/features/ai.md` and `docs/src/features/whatsapp.md` already have MCP sections — use as pattern reference (then standardize)
- `docs/src/features/api-mcp.md` has extensive MCP config examples for Claude Desktop and Claude Code
- 57 MCP tools across `ferro-mcp/src/tools/` — tool names map directly to feature areas

### Established Patterns
- mdBook format in `docs/src/` with `SUMMARY.md` navigation
- Feature pages follow: intro → usage examples → configuration → (optional MCP section)
- Code examples use fenced `rust` blocks

### Integration Points
- `docs/src/SUMMARY.md` — needs new entry under Getting Started for "Working with Agents"
- `docs/src/introduction.md` — full rewrite
- 25 feature pages in `docs/src/features/` — MCP sections added where relevant
- Only 4/25 feature pages currently mention MCP (ai.md, api-mcp.md, api.md, whatsapp.md)

### Tool-to-Feature Mapping (approximate)
- Events: `list_events`
- Queues: `list_jobs`, `job_history`, `queue_status`
- Broadcasting: `list_broadcast_channels`
- Caching: `cache_inspect`
- Authentication: `list_policies`, `session_inspect`
- Database: `database_schema`, `database_query`, `list_migrations`, `list_models`, `explain_model`, `model_usages`, `relation_map`
- Validation: (covered by `code_templates`)
- API Resources: `list_resources`
- Rate Limiting: `list_rate_limiters`
- Routing: `list_routes`, `explain_route`, `get_handler`, `route_dependencies`, `test_route`
- Middleware: `list_middleware`, `get_middleware`
- Services: `list_services`
- Projections: `list_projections`, `inspect_projection`, `render_projection`, `validate_projection`, `projection_coverage`
- JSON-UI: `json_ui_catalog`, `json_ui_inspect`, `json_ui_generate`
- Localization: `list_lang_files`
- General: `application_info`, `diagnose_error`, `last_error`, `code_templates`, `generation_context`, `get_config`, `dependency_graph`, `search_docs`

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 112-agent-first-philosophy*
*Context gathered: 2026-03-26*
