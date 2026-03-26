# Phase 112: Agent-First Philosophy - Research

**Researched:** 2026-03-26
**Domain:** mdBook documentation authoring — content restructuring and cross-linking
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Introduction rewrite:** Position as "An agent-first Rust web framework with Laravel-inspired conventions." Keep "Laravel-inspired" as heritage descriptor; lead with agent-first. Quick example stays as traditional handler code with an agent callout box: "AI agents can generate this automatically via MCP." Philosophy section restructured at Claude's discretion.
- **Working with Agents guide:** Full agent setup guide (MCP config, discovery loop, common workflows, troubleshooting). Lives in Getting Started section of SUMMARY.md, after Quick Start. Include copy-paste MCP config snippets for Claude Desktop, Claude Code (.claude.json), and generic stdio. Covers ferro-mcp (framework introspection) only — ferro-api-mcp already has its own page. Agent-to-CLI workflow documented as a section within this guide (not a separate page).
- **MCP tool references on feature pages:** Dedicated `## MCP Tools` section at the bottom of each feature page. Skip feature pages that have no relevant MCP tools (no empty sections). Standardize existing MCP sections (ai.md, whatsapp.md) to match the new pattern. api-mcp.md is already a dedicated MCP page — no changes needed there.

### Claude's Discretion

- Philosophy section restructuring approach
- Which agent workflows to document (suggested: discovery loop, error diagnosis, model exploration, code generation)
- Documentation format for workflow section (prose vs pseudo-conversation)
- Per-tool detail level on feature pages (one-liner vs example)
- Exact wording of agent callout box in introduction

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PHIL-01 | introduction.md rewritten to lead with agent-first value proposition | Current intro text analyzed; "agent-first" must appear in paragraph 1 and MCP before framework comparison |
| PHIL-02 | "Working with Agents" guide created documenting MCP workflow | ferro-mcp binary invoked as `ferro mcp`; config patterns documented in api-mcp.md and ~/.claude.json |
| PHIL-03 | MCP tool references added to each feature documentation page | All 25 feature pages audited; 21 lack MCP sections; tool-to-feature mapping confirmed from CONTEXT.md |
| PHIL-04 | Agent-to-CLI workflow documented (agent calls MCP → reads hints → uses CLI) | generation_context and code_templates tools return hints; get_handler + list_routes drive code gen |
</phase_requirements>

## Summary

Phase 112 is a docs-only rewrite. No Rust code changes. The work is surgical content authoring across `docs/src/`: one full rewrite (`introduction.md`), one new page (`getting-started/working-with-agents.md`), one SUMMARY.md edit, and MCP section additions to 21 of the 25 feature pages. The four pages that already have MCP sections (ai.md, api.md, api-mcp.md, whatsapp.md) require standardization to the new `## MCP Tools` heading pattern.

The core challenge is consistency of the section format across 21 files. The tool-to-feature mapping is fully known from CONTEXT.md and confirmed against the 57 MCP tools in `ferro-mcp/src/tools/`. Two features (static-files.md and testing.md) likely have no relevant ferro-mcp tools and should be skipped per the "no empty sections" rule. The themes.md and multi-tenancy.md pages also have no direct MCP tool coverage and should be skipped.

The ferro-mcp server is invoked as `ferro mcp` (the main ferro CLI binary handles the `mcp` subcommand). The binary path for config snippets follows the pattern `target/release/ferro` for production, with `target/debug/ferro` for development — consistent with how it is configured in `.claude.json`.

**Primary recommendation:** Write introduction.md first to lock the agent-first voice, then create the Working with Agents guide as the canonical MCP reference, then add `## MCP Tools` sections to feature pages using the confirmed tool mapping.

## Standard Stack

### Core

| Tool | Purpose | Notes |
|------|---------|-------|
| mdBook | Documentation framework | All docs are `.md` files under `docs/src/` |
| SUMMARY.md | Navigation registry | New page requires one line addition under Getting Started |
| fenced `rust` / `json` / `bash` blocks | Code examples | Existing convention — keep consistent |

No new libraries or dependencies. This is pure Markdown authoring.

## Architecture Patterns

### Existing Documentation Structure

```
docs/src/
├── introduction.md          # Full rewrite target
├── SUMMARY.md               # Add one entry after quickstart
├── getting-started/
│   ├── installation.md
│   ├── quickstart.md
│   └── working-with-agents.md  # New file (PHIL-02)
├── the-basics/
│   └── routing.md, middleware.md, controllers.md, request-response.md
├── features/                # 25 files — 21 need MCP sections added
│   └── *.md
├── json-ui/
└── reference/
```

### Pattern 1: introduction.md Structure (Rewrite Target)

**Current state:** Leads with "A Laravel-inspired web framework for Rust." MCP and agents not mentioned.

**Target structure:**

```markdown
# Ferro Framework

[Agent-first thesis paragraph — "agent-first" in sentence 1, MCP mentioned before comparisons]

[Laravel-inspired paragraph — heritage as secondary descriptor]

## Quick Example

[Existing handler code — unchanged]

> **AI agents can [do X] via MCP**
> [callout box — exact wording at Claude's discretion]

## Philosophy

[Restructured to lead with agent-first, then convention-over-config, DX, type safety, performance]

## Getting Started

[Existing link to installation]
```

### Pattern 2: MCP Tools Section (Feature Pages)

The new standard format, derived from inspecting existing sections in ai.md and whatsapp.md and the decision to standardize them:

```markdown
## MCP Tools

[One-sentence context for why these tools exist for this feature]

### `tool_name`

[One-liner or short description. Complex tools get an example.]

- **When to use:** [situation]
- **Returns:** [output description]

### `tool_name_2`

[One-liner description]
```

The ai.md section uses subsections with bullet-point details. The whatsapp.md section uses a flat list. The new standard should match the ai.md pattern (subsections with bullets) for complex tools and single-line entries for trivial tools — this is at Claude's discretion per detail level.

### Pattern 3: Working with Agents Guide Structure

```markdown
# Working with Agents

[Why Ferro is agent-first — 1-2 sentences]

## Setting Up ferro-mcp

### MCP Configuration

#### Claude Desktop
[copy-paste JSON snippet]

#### Claude Code (.claude.json)
[copy-paste JSON snippet]

#### Generic stdio (any MCP-compatible host)
[copy-paste JSON snippet]

## The Discovery Loop

[Core workflow: application_info → feature-specific tools → code generation]

## Common Workflows

### [Workflow 1: e.g., scaffold a feature with agent guidance]
### [Workflow 2: e.g., diagnose an error]
### [Workflow 3: e.g., explore models and generate CRUD]
### [Workflow 4: e.g., generate code from templates]

## Agent-to-CLI Workflow

[How agent reads MCP hints → selects CLI command → scaffolds code]

## Troubleshooting

[Common setup issues]
```

### Anti-Patterns to Avoid

- **Empty MCP sections:** The decision is explicit — skip pages where no ferro-mcp tools apply. Do not add placeholder text.
- **Inconsistent heading level:** All MCP sections use `## MCP Tools` (H2), not `## MCP Introspection` or other variants. Standardize ai.md and whatsapp.md.
- **Mentioning ferro-api-mcp in Working with Agents:** That binary gets its own page (api-mcp.md). The Working with Agents guide covers ferro-mcp only.
- **Duplicating api-mcp.md config examples:** The Working with Agents guide is for framework introspection (ferro-mcp). Link to api-mcp.md for the API bridge.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| MCP config format | Custom config format | Follow stdio pattern already in api-mcp.md and ~/.claude.json |
| Tool descriptions | Rewrite tool behavior | Read actual tool source in `ferro-mcp/src/tools/` |
| Navigation structure | Invent new SUMMARY section | Add to existing Getting Started section |

## Common Pitfalls

### Pitfall 1: Wrong Binary Name in MCP Config

**What goes wrong:** Docs show `ferro-mcp` as the command, but the actual binary is `ferro` with a `mcp` subcommand argument.

**Why it happens:** The crate is named `ferro-mcp` but builds into the main `ferro` CLI binary.

**How to avoid:** Config snippets must use `"command": "/path/to/ferro"` with `"args": ["mcp"]`, not `"command": "/path/to/ferro-mcp"`. This is confirmed from `~/.claude.json`.

**Warning signs:** Users getting "command not found" errors when setting up MCP.

### Pitfall 2: Standardizing Existing Sections Without Checking Them

**What goes wrong:** ai.md and whatsapp.md have existing `## MCP` sections with different names — ai.md uses `## MCP Tools`, whatsapp.md uses `## MCP Introspection`. Treating them as already standardized would leave whatsapp.md with a non-standard heading.

**Why it happens:** The sections were written independently before a standard existed.

**How to avoid:** Explicitly rename the whatsapp.md section heading to `## MCP Tools` to match the target standard. The ai.md heading is already correct.

### Pitfall 3: Skipping Pages That Should Have Sections

**What goes wrong:** Feature pages without obvious tool names get skipped when they do have relevant general tools (e.g., `application_info`, `get_config`, `code_templates`).

**Why it happens:** The tool-to-feature mapping in CONTEXT.md doesn't list general tools for specific pages.

**How to avoid:** Pages for features that are configured via environment (database, queues, caching, etc.) can reference `get_config` and `application_info` as general health-check tools. Use judgment — if the connection is weak, skip.

### Pitfall 4: Losing the Developer Appeal

**What goes wrong:** Rewriting introduction.md to be so agent-focused that human developers feel it's not for them.

**Why it happens:** "Agent-first" can read as "human-second" if handled poorly.

**How to avoid:** The decided framing is "An agent-first Rust web framework with Laravel-inspired conventions" — agents are the headline differentiator, but Laravel-style DX is the substance. The introduction should appeal to developers who want both excellent tooling AND agent integration.

## Code Examples

### MCP Config — Claude Desktop

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

File location: `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)

### MCP Config — Claude Code (.claude.json)

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

Place at project root as `.claude.json` (project-scope) or `~/.claude.json` (user-scope).

### MCP Config — Generic stdio

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

Any MCP host that supports stdio transport (Cursor, Windsurf, etc.) uses this pattern.

### Agent Callout Box (Introduction)

mdBook supports blockquotes with `> `. The callout box should be:

```markdown
> **Agents can generate this automatically.** Connect `ferro-mcp` to your AI agent and use `code_templates` to scaffold handlers, `list_routes` to explore your API, and `get_handler` to read implementation details.
```

This is a starting point — exact wording is at Claude's discretion.

## Feature Page MCP Tool Mapping

This is the authoritative mapping for Plan 112-02. Confirmed against the actual tool files in `ferro-mcp/src/tools/`.

| Feature Page | MCP Tools | Action |
|---|---|---|
| events.md | `list_events` | Add `## MCP Tools` section |
| queues.md | `list_jobs`, `job_history`, `queue_status` | Add `## MCP Tools` section |
| notifications.md | `code_templates` (notifications category) | Add `## MCP Tools` section (one-liner only) |
| broadcasting.md | `list_broadcast_channels` | Add `## MCP Tools` section |
| storage.md | `code_templates` (storage category) | Add `## MCP Tools` section (one-liner only) |
| caching.md | `cache_inspect` | Add `## MCP Tools` section |
| authentication.md | `list_policies`, `session_inspect` | Add `## MCP Tools` section |
| multi-tenancy.md | none directly | Skip |
| api-resources.md | `list_resources` | Add `## MCP Tools` section |
| api.md | `crud_create`, `crud_list`, `crud_update`, `crud_delete` | Standardize existing section (already has MCP, but heading is `## MCP Integration` — rename to `## MCP Tools`) |
| api-mcp.md | entire page is MCP | No changes needed |
| rate-limiting.md | `list_rate_limiters` | Add `## MCP Tools` section |
| database.md | `database_schema`, `database_query`, `list_migrations`, `list_models`, `explain_model`, `model_usages`, `relation_map` | Add `## MCP Tools` section (rich section) |
| derive-macros.md | `explain_model` (overlaps with database) | Add `## MCP Tools` section (one-liner) |
| validation.md | `code_templates` (validation category) | Add `## MCP Tools` section (one-liner) |
| localization.md | `list_lang_files` | Add `## MCP Tools` section |
| testing.md | none directly | Skip |
| static-files.md | none directly | Skip |
| inertia.md | `inspect_props`, `list_props`, `generate_types` (overlaps) | Add `## MCP Tools` section |
| json-ui.md | `json_ui_catalog`, `json_ui_inspect`, `json_ui_generate` | Add `## MCP Tools` section (rich section) |
| stripe.md | `stripe` tool if it exists | Add `## MCP Tools` section |
| whatsapp.md | `whatsapp_config_status`, `whatsapp_webhook_events` | Standardize existing section (rename `## MCP Introspection` → `## MCP Tools`) |
| themes.md | none directly | Skip |
| projections.md | `list_projections`, `inspect_projection`, `render_projection`, `validate_projection`, `projection_coverage` | Add `## MCP Tools` section (rich section) |
| ai.md | `test_classifier`, `list_pending_confirmations` | Standardize existing section (heading already `## MCP Tools` — verify format matches standard) |

**Pages to skip (no ferro-mcp tools apply):** multi-tenancy.md, testing.md, static-files.md, themes.md

**Pages that need standardization (existing sections to rename/reformat):** api.md (`## MCP Integration` → `## MCP Tools`), whatsapp.md (`## MCP Introspection` → `## MCP Tools`)

**Stripe note:** `ferro-mcp/src/tools/stripe.rs` exists. Confirm what tools it exposes before writing the stripe.md section.

## State of the Art

| Old Approach | Current Approach | Impact |
|---|---|---|
| "Laravel of Rust" as primary positioning | "Agent-first Rust framework with Laravel-inspired conventions" | Leads with differentiator; Laravel heritage becomes heritage, not pitch |
| MCP mentioned only on ai.md, api.md, api-mcp.md, whatsapp.md | `## MCP Tools` section on every relevant feature page | Makes MCP discoverable for each feature, not just AI-specific ones |
| No agent setup guide | "Working with Agents" in Getting Started | Agents and agent-using developers have a clear onboarding path |

## Open Questions

1. **Stripe MCP tool content**
   - What we know: `ferro-mcp/src/tools/stripe.rs` exists
   - What's unclear: What tools it exposes and what they do
   - Recommendation: Read `stripe.rs` at plan execution time before writing stripe.md section

2. **code_templates categories**
   - What we know: `code_templates` is a general-purpose tool
   - What's unclear: Exact categories available for notifications/storage/validation
   - Recommendation: Read `code_templates.rs` tool source at plan execution time to list accurate categories

3. **inertia.md MCP tools**
   - What we know: `inspect_props`, `list_props` and `generate_types` tools exist
   - What's unclear: Whether `generate_types` is a ferro-mcp tool or CLI-only
   - Recommendation: Check tool files at execution time; if it's CLI-only, omit from MCP section

## Validation Architecture

The `workflow.nyquist_validation` key is absent from `.planning/config.json`, so this section applies.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | mdBook link checker (manual) + file existence checks |
| Config file | `docs/book.toml` |
| Quick run command | `mdbook build docs/ 2>&1 | grep -i error` |
| Full suite command | `mdbook build docs/` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PHIL-01 | "agent-first" appears in introduction.md paragraph 1 | smoke | `grep -c "agent-first" docs/src/introduction.md` | ✅ (file rewritten) |
| PHIL-02 | Working with Agents guide exists and is linked in SUMMARY.md | smoke | `grep -c "working-with-agents" docs/src/SUMMARY.md` | ❌ Wave 0 |
| PHIL-03 | Feature pages have `## MCP Tools` sections where applicable | smoke | `grep -rl "## MCP Tools" docs/src/features/ | wc -l` | Partially (4 pages exist) |
| PHIL-04 | Agent-to-CLI workflow present in working-with-agents.md | smoke | `grep -c "CLI" docs/src/getting-started/working-with-agents.md` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `mdbook build docs/ 2>&1 | tail -5`
- **Per wave merge:** `mdbook build docs/`
- **Phase gate:** mdbook build succeeds with no broken links before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `docs/src/getting-started/working-with-agents.md` — covers PHIL-02, PHIL-04 (new file)

*(All other required files exist; they are being edited, not created.)*

## Sources

### Primary (HIGH confidence)

- Direct file inspection: `docs/src/introduction.md` — current content and structure analyzed
- Direct file inspection: `docs/src/SUMMARY.md` — navigation structure confirmed
- Direct file inspection: `ferro-mcp/src/tools/mod.rs` — confirmed 57 tool modules
- Direct file inspection: `docs/src/features/ai.md` and `docs/src/features/whatsapp.md` — existing MCP section patterns
- Direct file inspection: `docs/src/features/api-mcp.md` — MCP config snippet format and conventions
- Direct file inspection: `~/.claude.json` — confirmed ferro MCP binary invocation: `ferro mcp`
- Direct file inspection: `112-CONTEXT.md` — all user decisions sourced from here

### Secondary (MEDIUM confidence)

- File inspection of feature pages (all 25) to verify MCP section presence/absence
- Confirmed tool-to-feature mapping against CONTEXT.md approximate mapping plus actual tool file names

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — this is mdBook Markdown; no uncertainty
- Introduction rewrite approach: HIGH — content and decisions fully specified in CONTEXT.md
- Working with Agents structure: HIGH — decisions specify sections; MCP binary invocation confirmed from ~/.claude.json
- Feature page MCP mapping: HIGH — tool files confirmed to exist; skipped pages reasoned from absence of relevant tools
- MCP config format: HIGH — verified from ~/.claude.json and api-mcp.md

**Research date:** 2026-03-26
**Valid until:** Stable indefinitely — documentation only, no external dependencies
