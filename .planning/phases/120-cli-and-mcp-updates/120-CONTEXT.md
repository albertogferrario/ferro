# Phase 120: CLI & MCP Updates - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning
**Mode:** `--auto` — decisions auto-selected from codebase analysis and upstream phase context. Downstream agents may override any auto-choice by editing this file before `/gsd-plan-phase`.

<domain>
## Phase Boundary

Update all AI-facing tools to generate and inspect **v2 flat JSON specs** using a two-pass AI strategy.

This phase ships:

- `ferro make:json-view` generates a `src/views/{name}.json` file (v2 flat spec JSON), not a `.rs` file. Removes `mod.rs` update logic — JSON files are not Rust modules.
- CLI two-pass AI generation: Pass 1 describes the page layout in text; Pass 2 generates the full JSON spec constrained to `catalog.json_schema()`.
- `ferro-mcp` `json_ui_generate` provides generation context in v2 JSON format — example views, conventions, and spec template all updated.
- `ferro-mcp` `json_ui_catalog` exposes `json_schema` (full spec schema) and `component_schemas` (per-component props schema map) — sourced from `global_catalog()`.
- `ferro-mcp` `json_ui_inspect` scans `src/views/*.json` files instead of Rust builder patterns — the TODO(Phase 120) in the current source.
- `ferro-mcp` `code_templates` `json_view` category updated to emit v2 JSON spec format plus a paired Rust handler template using `JsonUi::render_file`.
- Generated specs validated against `global_catalog().validate(&spec)` before being returned.
- No v1 references (`Spec::builder()`, `Element::new()`, `JsonUiView`, `Component::`) remain in CLI/MCP generation or template code.

**What this phase does NOT do:**
- Change `Spec`/`Element` struct shape (Phase 115, frozen)
- Change the renderer or catalog (Phases 116/117, shipped)
- Change the expression resolution pipeline (Phase 118, shipped)
- Change `JsonUi::render_file` or the page loader (Phase 119, implementing)
- Convert gestiscilo pages (Phase 121)
- Add new JSON-UI components

</domain>

<decisions>
## Implementation Decisions

### D-01: `make:json-view` output format

**Decision:** The command generates `src/views/{name}.json` containing a v2 flat JSON spec. The `.rs` file path (`{name}.rs`) and `mod.rs` update logic are removed. No new Rust module is created — handlers call `JsonUi::render_file("views/{name}.json", data)` independently.

The JSON output template (static fallback, no AI):
```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "{{Title}}",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "{{Title}}" },
      "children": ["heading"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "{{Title}}", "element": "h1" }
    }
  }
}
```

**Why:** v2 views are JSON files (`ferro-json-ui/src/loader.rs` D-07 from Phase 119). Generated Rust code that uses the v1 builder pattern is v1; v2 generation means generating the spec file itself.

**How to apply:** Change `view_file` from `views_dir.join(format!("{name}.rs"))` to `views_dir.join(format!("{name}.json"))`. Remove the `mod_file` / `mod.rs` path entirely. Update the CLI's usage instructions to say "use in a handler with `JsonUi::render_file`."

### D-02: Two-pass AI generation in `ferro-cli/src/ai.rs`

**Decision:** Two separate `call_anthropic` calls:

- **Pass 1 (describe):** System prompt = minimal role + `global_catalog().prompt()` (cacheable). User message = page name + description. Ask the model to return a plain-text JSON-UI component plan: which components to use, what data each displays, what actions are present. No structured output constraint. Max tokens: 1024.

- **Pass 2 (structure):** System prompt = the Pass 1 output as context + `catalog.json_schema()` as the schema constraint (via Anthropic `tool_use` or JSON mode). User message = "Generate the complete v2 JSON spec for this view." Structured output schema = `global_catalog().json_schema()` (ensures the output is a valid spec). Max tokens: 4096.

If Pass 2 returns an invalid spec (fails `global_catalog().validate()`), log the errors and fall back to the static JSON template (no panic, no silent garbage).

**Why:** The ROADMAP caveats state that two-pass reduces hallucination and that per-component schema keeps token overhead manageable. Pass 1 is cheap and produces a natural-language scaffold that grounds Pass 2. Validating the output after Pass 2 catches cases where the model hallucinated invalid component types or prop shapes.

**How to apply:** Rename / replace `build_view_context` (which builds a Rust-code generation prompt) with `build_json_view_context` that returns a `(system_pass1, user_pass1)` tuple. Add `build_json_view_pass2(pass1_result, schema)` → `(system_pass2, user_pass2)`. The `call_anthropic` function can be reused for both passes with different bodies; the structured output call may need a separate `call_anthropic_structured(system, user, schema)` variant that sets `tool_use` / JSON mode.

### D-03: Validation of generated specs

**Decision:** After Pass 2 generation, parse the response as `serde_json::Value`, attempt `Spec::from_json`, then call `global_catalog().validate(&spec)`. On validation error: log the errors with `style("Warning:").yellow()`, fall back to static JSON template. Do NOT retry silently — one attempt, then fallback.

**Why:** The ROADMAP explicitly states "Generated specs are validated against `catalog.json_schema()` before being returned to the user." Failing silently would return garbage; retrying on every failure would make the command slow. One-shot validation + graceful fallback is the right balance for a CLI tool.

**How to apply:** Validation step lives in `ferro-cli/src/commands/make_json_view.rs` after the AI call chain completes. `ferro-cli` already depends on `ferro-json-ui`; use `ferro_json_ui::global_catalog()`.

### D-04: `json_ui_catalog` adds JSON Schema fields

**Decision:** Add two fields to `JsonUiCatalog`:

```rust
pub struct JsonUiCatalog {
    // existing fields preserved (CONTEXT 117 D-24)
    pub components: Vec<CatalogComponent>,
    pub plugin_components: Vec<CatalogComponent>,
    pub builder_api: String,
    pub action_api: String,
    // new in Phase 120:
    pub json_schema: serde_json::Value,
    pub component_schemas: std::collections::HashMap<String, serde_json::Value>,
}
```

`json_schema` = `global_catalog().json_schema()` (full spec schema).
`component_schemas` = one entry per built-in and plugin component, keyed by type name, value = `global_catalog().component_schema(name).cloned()`.

**Why:** Success criterion 3: "MCP `json_ui_catalog` tool exposes JSON Schema per component (replaces text-only catalog inspection)." Downstream agents using `json_ui_catalog` get the full schema in one MCP call without needing a second tool.

**How to apply:** In `ferro-mcp/src/tools/json_ui_catalog.rs::execute`, populate these fields after building the component lists. `component_schemas` can be built with a single `.map(|spec| (spec.name.clone(), spec.props_schema.clone())).collect()` pass over the catalog.

### D-05: `json_ui_inspect` v2 scanner

**Decision:** Rewrite `json_ui_inspect.rs` to scan `src/views/*.json` files (recursive glob of `src/views/**/*.json`). For each file:

1. Parse as `serde_json::Value`
2. Extract `title` from `spec["title"]`, `layout` from `spec["layout"]`
3. Collect `components_used` from `spec["elements"].*.type` values (deduplicated, sorted)
4. Collect `actions` from any `spec["elements"].*.action` fields

Return the existing `ViewInfo` struct shape — `name` = file stem, `file` = relative path. Remove the v1 `JsonUiView` / `Component::` regex patterns entirely.

The static `BUILTIN_TYPES` array in the current `json_ui_inspect.rs` is removed — component type validation is not `inspect`'s job; it belongs in `validate_projection` / `catalog.validate`.

**Why:** The existing code has an explicit `TODO(Phase 120): rewrite regexes to scan for → Spec and parse flat specs.` This is that rewrite. v2 codebases have no `.rs` view files to scan.

**How to apply:** Delete the existing regex-based scan logic. Add `scan_json_views(root: &Path) -> Vec<ViewInfo>` using a simple directory walk. Keep `inspect_component_schema(name: &str) -> ComponentSchemaInfo` — it already delegates to `global_catalog()` and doesn't need changes.

### D-06: `code_templates` v2 format for `json_view` category

**Decision:** Replace all three existing `json_view` templates (`basic_view`, `list_view`, `form_view`) with v2 equivalents. Each template is now a JSON spec string (not Rust builder code). Add a fourth template: `json_view_handler` (category `json_view`) that shows the paired Rust handler using `JsonUi::render_file`.

Template format for `basic_view`:
```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "{{title}}",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "{{title}}" },
      "children": ["heading"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "{{title}}", "element": "h1" }
    }
  }
}
```

`json_view_handler` Rust template:
```rust
#[handler]
pub async fn {{view_name}}(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/{{view_name}}.json", data)
}
```

**Why:** All code templates must use v2 format (success criterion 5). The paired handler template helps agents understand the full flow — spec file + Rust handler.

**How to apply:** Rewrite `fn json_view_templates()` in `ferro-mcp/src/tools/code_templates.rs`. The existing `imports` field for JSON templates can be empty (no Rust imports needed for spec files) or contain `use ferro::{JsonUi, Response};` for the handler template.

### D-07: `json_ui_generate` MCP example and conventions update

**Decision:** Update `VIEW_EXAMPLE` in `ferro-mcp/src/tools/json_ui_generate.rs` to show a v2 JSON spec instead of Rust builder code. Update `ViewConventions` to reflect:
- `file_location`: `"src/views/{name}.json"` (not `.rs`)
- `function_signature`: removed (not applicable to JSON files)
- `import_pattern`: `"use ferro::{JsonUi, Response};"` in the handler
- `layout_default`: unchanged (`"dashboard"`)

The `example` field in `JsonUiGenerationContext` becomes a JSON spec string.

**Why:** The `json_ui_generate` tool provides context for agents generating views. If the example shows builder pattern, agents will generate builder pattern — wrong for v2.

**How to apply:** Update `VIEW_EXAMPLE` const and `ViewConventions` fields in `json_ui_generate.rs`.

### D-08: No v1 references remain

**Decision:** After Phase 120:
- `ferro-cli/src/ai.rs` — old `build_view_context` Rust-code builder prompt removed or renamed; only `build_json_view_context` (v2 JSON) remains.
- `ferro-cli/src/templates.rs` (if it exists) — any `json_view_template` returning Rust builder code is replaced with a JSON template.
- `ferro-mcp/src/tools/code_templates.rs` — no `Spec::builder()` / `Element::new()` in `json_view` templates.
- `ferro-mcp/src/tools/json_ui_generate.rs` — no builder pattern in `VIEW_EXAMPLE`.
- `ferro-mcp/src/tools/json_ui_inspect.rs` — no `JsonUiView`, `Component::`, or v1 regex patterns.

Grep confirmation: `grep -rn "Spec::builder\|Element::new\|JsonUiView\|Component::" ferro-cli/src ferro-mcp/src --include="*.rs"` should return zero hits in generation/template paths after Phase 120.

**Why:** Success criterion 6: "No references to v1 types remain in CLI or MCP code."

### Claude's Discretion
- Whether `call_anthropic_structured` is a separate function or a flag on the existing `call_anthropic` — prefer a separate function for clarity since the request body shape differs (adds `tools` / JSON mode field).
- Whether the two-pass generation lives entirely in `ai.rs` or is split between `ai.rs` (raw API calls) and `make_json_view.rs` (orchestration) — prefer orchestration in `make_json_view.rs` with helpers in `ai.rs` (matches existing split).
- Whether `component_schemas` in `JsonUiCatalog` includes plugin components — include them (completeness; agents don't know at call time which components they'll need).
- The exact JSON scan depth for `json_ui_inspect` — `src/views/*.json` (flat) is sufficient for v12.0; recursive glob is a "Claude decides" extension.
- Whether the validator in `make:json-view` falls back silently or prints a detailed error — print the validation errors to stderr (yellow warning), then write the static template. Do not suppress.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and success criteria
- `.planning/ROADMAP.md` §"Phase 120: CLI & MCP Updates" — goal, depends-on (Phase 117, Phase 119), Requirements (TOOL-01–04), 7 success criteria, 3 caveats

### Upstream locked decisions (do not re-open)
- `.planning/phases/115-spec-v2-data-structures/115-CONTEXT.md` — v2 JSON spec shape: `$schema`, `root`, `elements` flat map, `Element.type`/`props`/`children`/`action`/`visible`
- `.planning/phases/117-catalog-and-json-schema/117-CONTEXT.md` — `global_catalog()`, `Catalog::validate`, `catalog.prompt()`, `catalog.component_schema(name)`, `catalog.json_schema()` — all APIs Phase 120 consumes
- `.planning/phases/119-page-loader/119-CONTEXT.md` — `JsonUi::render_file` API (handlers call this, not the builder pattern); `load_cached` / `LoadError`

### Source files to modify
- `ferro-cli/src/commands/make_json_view.rs` — change output from `.rs` to `.json`; remove mod.rs logic; wire two-pass AI
- `ferro-cli/src/ai.rs` — replace/extend `build_view_context` with two-pass JSON generation functions
- `ferro-mcp/src/tools/json_ui_generate.rs` — update `VIEW_EXAMPLE` and `ViewConventions` to v2
- `ferro-mcp/src/tools/json_ui_catalog.rs` — add `json_schema` + `component_schemas` fields
- `ferro-mcp/src/tools/json_ui_inspect.rs` — rewrite scan from Rust regex to JSON file walk (TODO(Phase 120) already in source)
- `ferro-mcp/src/tools/code_templates.rs` — rewrite `json_view_templates()` for v2 JSON format

### Framework entry points
- `ferro_json_ui::global_catalog()` — `Catalog::validate`, `catalog.prompt()`, `catalog.json_schema()`, `catalog.component_schema(name)`, `catalog.components_sorted()`, `catalog.plugin_components_sorted()`
- `ferro_json_ui::Spec::from_json(json: &str)` — parse JSON string into `Spec` (structural validation)
- `framework/src/json_ui/mod.rs` — `JsonUi::render_file` (Phase 119, the v2 handler entry point)

### Workspace conventions
- `CLAUDE.md` — fmt + clippy + test before every commit; no co-author lines; update ferro-mcp when framework behavior changes
- Anthropic API: model `claude-sonnet-4-5`, structured output via `tool_use`, system prompt with `cache_control: ephemeral`, temperature 0.2

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-cli/src/ai.rs::call_anthropic(system, user_prompt) -> Result<String, String>` — blocking HTTP call to Anthropic API; reuse for both passes; extend or clone for structured output variant
- `ferro_json_ui::global_catalog()` — already imported in both `ferro-cli/src/ai.rs` and `ferro-mcp/src/tools/json_ui_generate.rs`; `.prompt()` already used as system context
- `ferro-mcp/src/tools/json_ui_catalog.rs::to_catalog_component` closure — already iterates `ComponentSpec`; `component_schemas` can be built in the same loop
- `ferro-mcp/src/tools/json_ui_inspect.rs::inspect_component_schema` — already delegates to `global_catalog()`; no changes needed to this function

### Established Patterns
- CLI AI generation: one `call_anthropic(system, user)` call; assistant prefill `"//!"` removed for JSON output (use `"{"` instead)
- MCP tools: `execute(args) -> Struct` pattern; all tools return `Serialize` structs, no raw strings
- Error handling: `eprintln!` with `console::style` for warnings; `std::process::exit(1)` for fatal CLI errors
- Template placeholders: `{{name}}` double-brace style (already used in `code_templates.rs`)

### Integration Points
- `ferro-cli/Cargo.toml` — already has `ferro-json-ui` and `serde_json` deps; no new deps needed for JSON generation
- `ferro-mcp/Cargo.toml` — already has `ferro-json-ui`; `serde_json::Value` already used in `json_ui_catalog.rs`
- The `json_ui_inspect.rs` TODO comment explicitly calls out Phase 120 as the rewrite trigger — zero ambiguity about intent

### Non-obvious behaviors to preserve
- `json_ui_catalog.rs` public struct shape (`components`, `plugin_components`, `builder_api`, `action_api`) must be preserved — adding fields is fine, removing is not (CONTEXT 117 D-24)
- `code_templates.rs::execute` filter by `category` must still work; the renamed/replaced templates keep `category: "json_view"`
- `make:json-view` `--no-ai` flag falls back to the static JSON template (not a Rust template)

### Non-obvious behaviors to remove
- `make:json-view` `mod.rs` update logic — JSON files are not Rust modules, no module declaration needed
- Rust builder-pattern assistant prefill (`"//!"`) in `ai.rs` — v2 generation starts with `"{"` (JSON object)
- `json_ui_inspect.rs` `BUILTIN_TYPES` const (the stale v1 list of 20 types) — replaced by walking `elements[*].type` from parsed specs

</code_context>

<specifics>
## Specific Ideas

- The `TODO(Phase 120)` comment in `ferro-mcp/src/tools/json_ui_inspect.rs` line 11–14 is the exact directive for the v2 scan rewrite. The planner should cite it explicitly in the plan task.
- Two-pass generation makes the CLI slightly slower (two API round-trips). The first pass is bounded at 1024 tokens; total latency is ~2–3× a single call. No spinner / progress indicator changes are needed — the existing "Generating view with AI..." message covers both passes.
- The structured output call in Pass 2 should use Anthropic's `tool_use` mechanism (not a separate schema field) since that's the production-tested path for constrained JSON output. `tools: [{ name: "emit_spec", input_schema: catalog.json_schema() }]` + `tool_choice: { type: "tool", name: "emit_spec" }`.
- The `ViewConventions.function_signature` field in `JsonUiGenerationContext` was designed for Rust functions; in v2 it becomes a description of the Rust handler side: `"#[handler] pub async fn name(req: Request) -> Response { JsonUi::render_file(...) }"`.

</specifics>

<deferred>
## Deferred Ideas

- Per-component structured output (one API call per component): described in ROADMAP caveats as the most hallucination-resistant approach. Deferred because token overhead (N API calls) is not justified for v12.0; the full-spec JSON schema constraint in Pass 2 is sufficient.
- Retry on validation failure: if the generated spec is invalid, retry Pass 2 once with the validation errors injected as context. Deferred — fallback to static template is the simpler v12.0 policy.
- `ferro json-ui:validate` CLI command (validate an existing spec file on disk). Out of scope for Phase 120 — this is a standalone quality-of-life tool for Phase 121+.
- Watch mode for `make:json-view` (generate + hot-reload on description change). Deferred.
- `generation_context` MCP tool update: if it contains json-ui code examples, those may also reference v1 builder pattern. Planner should grep and include in scope if matches found — out of Phase 120 scope if no hits.

</deferred>

---

*Phase: 120-cli-and-mcp-updates*
*Context gathered: 2026-04-21*
