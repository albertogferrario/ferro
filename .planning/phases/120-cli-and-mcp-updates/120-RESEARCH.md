# Phase 120: CLI & MCP Updates - Research

**Researched:** 2026-04-21
**Domain:** Rust CLI AI generation, MCP tool structs, ferro-json-ui Catalog API
**Confidence:** HIGH

## Summary

Phase 120 is a targeted rewrite of six files across two crates (`ferro-cli` and `ferro-mcp`).
The upstream phases (115, 117, 119) have already frozen the data model, catalog API, and render
pipeline. This phase connects those APIs to the AI generation and introspection surfaces.

Every change is a substitution: old v1 code out, v2 code in. The risk surface is narrow — the
two-pass AI generation in `ai.rs` is the only net-new logic. The rest is field additions to
structs, a file-walk replacement for regex scanning, and string constant updates.

The CONTEXT.md decisions are fully specified: file paths, struct field names, fallback behavior,
and the exact Anthropic `tool_use` call shape are all decided. Research confirms those decisions
are consistent with the actual codebase state.

**Primary recommendation:** Implement in dependency order — catalog struct fields first (D-04),
then inspect rewrite (D-05), then code_templates (D-06), then json_ui_generate (D-07), then
the two-pass AI chain (D-02), then make_json_view output change (D-01). Tests can be updated
incrementally alongside each task.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Two-pass AI generation | ferro-cli (CLI) | ferro-json-ui (Catalog API) | Generation is a local CLI operation; catalog provides schema and prompt |
| JSON file scan (inspect) | ferro-mcp (MCP tool) | ferro-json-ui (Catalog API) | Inspect runs inside ferro-mcp process against project root |
| Catalog JSON Schema exposure | ferro-mcp (MCP tool) | ferro-json-ui (Catalog) | MCP tool adds fields; catalog provides the data |
| Code templates | ferro-mcp (MCP tool) | — | Template strings live entirely in code_templates.rs |
| View file generation | ferro-cli (CLI) | ferro-json-ui (Catalog API) | CLI writes files; catalog validates output |
| Spec validation | ferro-json-ui (Catalog) | ferro-cli (consumer) | Catalog owns the validator; CLI calls validate() |

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `make:json-view` generates `src/views/{name}.json` (v2 spec), not `.rs`. mod.rs update logic removed entirely.
- **D-02:** Two-pass AI generation: Pass 1 (describe, max 1024 tokens) + Pass 2 (structured output via `tool_use`, max 4096 tokens). Fallback to static JSON template on Pass 2 invalid spec.
- **D-03:** Validation after Pass 2: `Spec::from_json` then `global_catalog().validate()`. On failure: print yellow warning, write static template. One attempt only, no retry.
- **D-04:** `JsonUiCatalog` gets two new fields: `json_schema: serde_json::Value` and `component_schemas: HashMap<String, serde_json::Value>`. Existing fields preserved.
- **D-05:** `json_ui_inspect` rewritten to scan `src/views/*.json` files using directory walk. `BUILTIN_TYPES` const removed. `inspect_component_schema` preserved unchanged.
- **D-06:** All three `json_view_templates()` templates replaced with v2 JSON spec strings. New fourth template `json_view_handler` (Rust handler using `JsonUi::render_file`).
- **D-07:** `VIEW_EXAMPLE` and `ViewConventions` in `json_ui_generate.rs` updated to v2.
- **D-08:** Zero v1 references (`Spec::builder()`, `Element::new()`, `JsonUiView`, `Component::`) in CLI/MCP generation or template paths after phase.

### Claude's Discretion

- Whether `call_anthropic_structured` is a separate function or a flag on the existing `call_anthropic` — prefer separate function.
- Whether two-pass orchestration is in `ai.rs` (raw API) or `make_json_view.rs` (orchestration) — prefer orchestration in `make_json_view.rs`.
- Whether `component_schemas` includes plugin components — include them.
- JSON scan depth for `json_ui_inspect` — flat `src/views/*.json` is sufficient; recursive is Claude's call.
- Validator fallback verbosity — print validation errors to stderr (yellow warning), then write static template.

### Deferred Ideas (OUT OF SCOPE)

- Per-component structured output (one API call per component).
- Retry on validation failure with errors injected as context.
- `ferro json-ui:validate` CLI command.
- Watch mode for `make:json-view`.
- `generation_context` MCP tool v1 reference audit (grep first; include in scope only if hits found).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TOOL-01 | `ferro make:json-view` generates v2 flat specs using two-pass generation | D-01 + D-02: output path change in make_json_view.rs + two new functions in ai.rs |
| TOOL-02 | MCP `json_ui_generate` tool uses `catalog.prompt()` for concise context and `catalog.component_schema()` for per-component structured output | D-07: VIEW_EXAMPLE and ViewConventions string updates in json_ui_generate.rs |
| TOOL-03 | MCP `json_ui_catalog` exposes JSON Schema per component | D-04: two new fields added to JsonUiCatalog struct; populated from global_catalog() |
| TOOL-04 | MCP `json_ui_inspect` works with v2 format and reports validation errors; all code templates use v2 spec format | D-05 (inspect rewrite) + D-06 (templates rewrite) |
</phase_requirements>

## Standard Stack

No new external dependencies are introduced in this phase. All required crates are already in
the relevant `Cargo.toml` files.

### Confirmed present in ferro-cli/Cargo.toml [VERIFIED: codebase grep]
| Crate | Purpose |
|-------|---------|
| `ferro-json-ui` | `global_catalog()`, `Spec::from_json`, `CatalogError` |
| `serde_json` | JSON parsing, `serde_json::Value` for structured output body |
| `reqwest` (blocking) | `call_anthropic` HTTP client — already used for Pass 1; Pass 2 reuses same client |
| `console` | `style(...)` for warning output |

### Confirmed present in ferro-mcp/Cargo.toml [VERIFIED: codebase grep]
| Crate | Purpose |
|-------|---------|
| `ferro-json-ui` | `global_catalog()`, `ComponentSpec` |
| `serde_json` | `serde_json::Value` for new catalog fields |
| `serde` | `Serialize` on new fields |

### No New Dependencies
`std::fs::read_dir` (already used) handles the JSON file walk in `json_ui_inspect.rs`. No `walkdir` or glob crate is needed for flat `src/views/*.json` scanning. [VERIFIED: codebase — existing code already uses `fs::read_dir` in inspect.rs]

## Architecture Patterns

### System Architecture Diagram

```
User runs `ferro make:json-view dashboard --desc "..."` 
         |
         v
make_json_view.rs::run()
    |-- D-01: output path = src/views/{name}.json (not .rs)
    |-- mod.rs update logic REMOVED
    |-- D-02: if ANTHROPIC_API_KEY present:
    |       |
    |       v
    |   ai::build_json_view_pass1(name, desc)
    |   -> (system_p1, user_p1)
    |   ai::call_anthropic(system_p1, user_p1) → plain-text component plan
    |       |
    |       v
    |   ai::build_json_view_pass2(pass1_result, schema)
    |   -> (system_p2, user_p2, schema)
    |   ai::call_anthropic_structured(system_p2, user_p2, schema) → JSON string
    |       |
    |       v
    |   D-03: Spec::from_json(json_string)?
    |         global_catalog().validate(&spec)?
    |         On error: warn + fallback to json_view_template()
    |-- write src/views/{name}.json
    
Agent reads MCP tools:
    json_ui_catalog → D-04: struct gains json_schema + component_schemas
    json_ui_inspect → D-05: scan src/views/*.json (not *.rs regex)
    json_ui_generate → D-07: VIEW_EXAMPLE + ViewConventions = v2 JSON
    code_templates(json_view) → D-06: 3 replaced + 1 new handler template
```

### Recommended File Structure (unchanged from current)
```
ferro-cli/src/
├── ai.rs                   # add build_json_view_pass1, build_json_view_pass2, call_anthropic_structured
├── commands/
│   └── make_json_view.rs   # change output path, remove mod.rs logic, wire two-pass
└── templates/
    └── make.rs             # update json_view_template() to return JSON string

ferro-mcp/src/tools/
├── json_ui_catalog.rs      # add json_schema + component_schemas to JsonUiCatalog
├── json_ui_inspect.rs      # rewrite execute() to scan *.json; remove BUILTIN_TYPES
├── json_ui_generate.rs     # update VIEW_EXAMPLE const + ViewConventions struct fields
└── code_templates.rs       # rewrite json_view_templates()
```

### Pattern 1: Anthropic tool_use for structured output

The structured output call (Pass 2) uses Anthropic's `tool_use` mechanism with `tool_choice`
forced. This is the production-tested path for constrained JSON from the Anthropic API.
[VERIFIED: codebase — existing `call_anthropic` already uses the messages API; this extends it]

```rust
// Source: CONTEXT.md D-02 + Anthropic Messages API (ASSUMED: tool_use schema)
let body = serde_json::json!({
    "model": model,
    "max_tokens": 4096,
    "temperature": 0.2,
    "system": [{ "type": "text", "text": system, "cache_control": {"type": "ephemeral"} }],
    "tools": [{
        "name": "emit_spec",
        "description": "Emit the complete v2 JSON-UI spec for the requested view.",
        "input_schema": catalog.json_schema()
    }],
    "tool_choice": { "type": "tool", "name": "emit_spec" },
    "messages": [{ "role": "user", "content": user_prompt }]
});

// Response parsing: extract tools[0].input (the structured JSON object)
let spec_value = response_json["content"]
    .as_array()
    .and_then(|arr| arr.iter().find(|item| item["type"] == "tool_use"))
    .and_then(|item| item.get("input"))
    .cloned()
    .ok_or_else(|| "no tool_use block in response".to_string())?;

let spec_str = serde_json::to_string(&spec_value)
    .map_err(|e| format!("serializing spec: {e}"))?;
```

Note on Pass 1 assistant prefill: the current `call_anthropic` hardcodes `"//!"` as the
assistant prefill. This MUST NOT be used for Pass 1 of the JSON-UI generation — Pass 1 returns
plain text, so the prefill should be empty or a neutral starter. The simplest approach is
`call_anthropic_structured` omits the prefill, and `call_anthropic` variant for Pass 1 either
drops the prefill or sets it to an empty string. [VERIFIED: codebase — ai.rs line 44-46]

### Pattern 2: JSON view file walk

```rust
// Source: CONTEXT.md D-05 — mirroring existing fs::read_dir pattern in inspect.rs
fn scan_json_views(project_root: &Path) -> Vec<ViewInfo> {
    let views_dir = project_root.join("src/views");
    let entries = match fs::read_dir(&views_dir) {
        Ok(e) => e.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => return Vec::new(),
    };
    let mut views = Vec::new();
    for entry in entries {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") { continue; }
        let content = match fs::read_to_string(&path) { Ok(c) => c, Err(_) => continue };
        let json: serde_json::Value = match serde_json::from_str(&content) { Ok(v) => v, Err(_) => continue };
        let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let title = json["title"].as_str().map(String::from);
        let layout = json["layout"].as_str().map(String::from);
        let components_used: Vec<String> = json["elements"]
            .as_object()
            .map(|m| {
                let mut types: Vec<String> = m.values()
                    .filter_map(|el| el["type"].as_str().map(String::from))
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                types.sort();
                types
            })
            .unwrap_or_default();
        let actions: Vec<String> = json["elements"]
            .as_object()
            .map(|m| m.values()
                .filter_map(|el| el["action"].as_object()
                    .and_then(|a| a.get("handler"))
                    .and_then(|h| h.as_str())
                    .map(String::from))
                .collect())
            .unwrap_or_default();
        let relative = path.strip_prefix(project_root).unwrap_or(&path)
            .to_string_lossy().to_string();
        views.push(ViewInfo { name, file: relative, title, layout, components_used, actions });
    }
    views
}
```

### Pattern 3: component_schemas field population

The `to_catalog_component` closure in `json_ui_catalog.rs` iterates `ComponentSpec` once.
`component_schemas` can be built in a second pass over the same catalog — no new catalog
traversal needed. [VERIFIED: codebase — json_ui_catalog.rs lines 58-84]

```rust
// In execute() after building components + plugin_components:
let json_schema = cat.json_schema().clone();
let component_schemas: std::collections::HashMap<String, serde_json::Value> = cat
    .components_sorted()
    .chain(cat.plugin_components_sorted())
    .filter_map(|spec| {
        cat.component_schema(&spec.name)
            .map(|s| (spec.name.clone(), s.clone()))
    })
    .collect();
```

### Anti-Patterns to Avoid

- **Removing the `//!` prefill from the existing `call_anthropic` signature:** The existing
  `call_anthropic` is used by other code that may depend on the `//!` prefill. Introduce
  `call_anthropic_structured` as an additive change; do not modify `call_anthropic` except to
  remove the prefill for the plain-text Pass 1 variant (or create `call_anthropic_plain`).
  [VERIFIED: codebase — `build_view_context` in ai.rs is the only caller of `call_anthropic`
  and it relies on the `//!` prefill being prepended; Phase 120 replaces this entirely]

- **Importing `Spec` into ferro-mcp:** `json_ui_inspect.rs` does NOT need to parse into `Spec`
  — it extracts metadata from raw `serde_json::Value`. Keeping it as raw JSON avoids adding a
  `Spec::from_json` call path that returns `SpecError`, which would need error handling.
  [ASSUMED — the CONTEXT.md D-05 says "parse as serde_json::Value", not Spec]

- **Keeping `BUILTIN_TYPES` in `inspect.rs`:** The constant is a stale v1 list. Phase 120
  removes it per D-05. Component type validation is `catalog.validate`'s job, not `inspect`'s.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON Schema validation of AI output | Custom validator | `global_catalog().validate(&spec)` | Three-stage pipeline (type whitelist + per-element props + envelope check) already implemented and tested |
| Structured JSON from Anthropic | Custom parsing of free-form JSON | `tool_use` with `tool_choice: { type: "tool" }` | Forces model to emit structured output; avoids wrapping JSON in markdown fences |
| Component type extraction from spec | Regex on JSON text | `json["elements"].as_object().map(|m| m.values()...)` | Parse the JSON properly; regex on JSON is fragile |
| Catalog prompt text | Custom string builder | `global_catalog().prompt()` | Already concise (≤ 8 KB), alphabetically ordered, and tested |

## Common Pitfalls

### Pitfall 1: `call_anthropic` assistant prefill on JSON passes
**What goes wrong:** The existing `call_anthropic` appends `"//!"` as assistant prefill AND
prepends it to the response. If Pass 1 uses the same function without modification, the response
starts with `"//!"` which is invalid as plain text description. Pass 2 gets a `"{"` prefix via
`tool_use` (the model returns the structured object directly in `input`), so this function should
NOT be used for Pass 2 at all.
**Why it happens:** The function was designed for Rust code generation.
**How to avoid:** `call_anthropic_structured` is a separate function with no assistant prefill
and different response-extraction logic (reads `content[].type=="tool_use" → input`).
For Pass 1 (plain text), either create `call_anthropic_plain` or adjust Pass 1 to strip the
`//!` prefix if using the existing function.
**Warning signs:** AI output for Pass 1 starts with `//!`; Pass 2 response extraction fails
because there is no `tool_use` block.

### Pitfall 2: `json_schema()` returns a `&Value` (zero-copy), not owned
**What goes wrong:** `Catalog::json_schema()` returns `&Value` (lifetime tied to the catalog
singleton). The `tool_use` body requires an owned value for JSON serialization.
**Why it happens:** The catalog is designed for zero-copy reads.
**How to avoid:** Call `.clone()` when embedding in the request body:
`"input_schema": cat.json_schema().clone()`. This is a single clone at call time; not on every
request since `call_anthropic_structured` is called at most once per `make:json-view` run.

### Pitfall 3: `JsonUiCatalog` struct must remain serializable after adding `serde_json::Value` fields
**What goes wrong:** `json_schema` and `component_schemas` are `serde_json::Value` and
`HashMap<String, serde_json::Value>`. These serialize correctly with serde but the existing
tests in `json_ui_catalog.rs` assert on the serialized JSON keys. New tests must verify the
new fields appear.
**Why it happens:** The test `test_serialization` at line 323 checks that specific keys exist
but does not check for unexpected keys — adding fields won't break it. However, `test_all_components_present` checks `catalog.components.len() == 39` which remains unaffected.
**How to avoid:** Add assertions for `json_schema` and `component_schemas` in a new test.

### Pitfall 4: `make_json_view.rs` usage message still references Rust patterns
**What goes wrong:** Lines 150-156 of `make_json_view.rs` print usage instructions that say
`"use crate::views::{file_name};"` and show a `JsonUi::render` Rust snippet. These are v1
usage patterns that will confuse users after the output changes to JSON.
**Why it happens:** The usage block was written for `.rs` output.
**How to avoid:** Replace the usage message with v2 guidance: reference `JsonUi::render_file`
and show the correct handler pattern per CONTEXT.md D-06 `json_view_handler` template.

### Pitfall 5: `inspect.rs` `inspect_component` still uses `BUILTIN_TYPES` for is_builtin
**What goes wrong:** `inspect_component` (line 86 of `json_ui_inspect.rs`) uses the stale
`BUILTIN_TYPES` constant to determine if a component is built-in. After removing `BUILTIN_TYPES`,
this function needs a replacement check.
**Why it happens:** `inspect_component` is preserved per D-05, but its internal `is_builtin`
check relies on the removed constant.
**How to avoid:** Replace the `BUILTIN_TYPES.iter().any(...)` check with
`global_catalog().component_schema(component_type).is_some()` — if it's in the catalog, it
exists. Plugin detection then becomes: if it's not in the built-in components but IS in
plugin_components. Use `ferro_json_ui::global_catalog()` directly.

### Pitfall 6: `json_view_templates()` test expects 3 templates; after D-06 there are 4
**What goes wrong:** The test `test_all_categories_present` only checks that the `json_view`
category is non-empty — it passes. But if there is a count-based test elsewhere, it would fail.
**Why it happens:** No count-based test exists for `json_view_templates` currently
(confirmed by reading code_templates.rs tests).
**How to avoid:** No action needed beyond being aware. Verify grep: `grep -n "json_view" ferro-mcp/src/tools/code_templates.rs` confirms three template names today; after Phase 120 there are four.

## Code Examples

### v2 JSON template (static fallback and --no-ai path)
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
Source: CONTEXT.md D-01 [VERIFIED: matches v2 spec shape from Phase 115]

### json_view_handler template (new fourth template in code_templates.rs)
```rust
#[handler]
pub async fn {{view_name}}(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/{{view_name}}.json", data)
}
```
Source: CONTEXT.md D-06 [VERIFIED: consistent with JsonUi::render_file API from Phase 119]

### Pass 1 system prompt structure
```
"You are a Ferro JSON-UI view planner. Given a view name and description, \
 produce a plain-text component plan: which components to use, what data \
 each displays, what actions are present. Be concise.\n\n\
 {catalog.prompt()}"
```
Source: CONTEXT.md D-02 [ASSUMED: prompt text style]

### Pass 2 system prompt structure  
```
"You are a Ferro JSON-UI view generator. Use the tool to emit the complete \
 v2 JSON spec for the view described in the component plan below.\n\n\
 {pass1_result}"
```
Source: CONTEXT.md D-02 [ASSUMED: prompt text style]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Spec builder pattern in `.rs` files | Flat JSON spec in `.json` files | Phase 115 (data structures) | Views are now data files, not Rust modules |
| Rust code generation via AI | JSON spec generation via AI | Phase 120 (this phase) | No Rust compilation needed for view changes |
| Regex scan for `JsonUiView` in `.rs` files | JSON parse of `src/views/*.json` | Phase 120 (this phase) | Deterministic, structured extraction |
| Text-only catalog description | Full JSON Schema per component | Phase 117 (catalog) | Agents get machine-readable constraints |

**Deprecated/outdated patterns being removed:**
- `Spec::builder()` / `Element::new()` in CLI/MCP generation paths: replaced by JSON spec strings
- `BUILTIN_TYPES` const in `json_ui_inspect.rs`: replaced by `global_catalog()` lookup
- `mod.rs` update logic in `make_json_view.rs`: JSON files are not Rust modules
- `build_view_context` in `ferro-cli/src/ai.rs`: replaced by `build_json_view_pass1` / `build_json_view_pass2`
- `//!` assistant prefill for JSON generation: replaced by empty prefill (Pass 1) or tool_use (Pass 2)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Anthropic tool_use response shape: `content[].type=="tool_use" → input` field | Code Examples, Pitfall 1 | Pass 2 response parsing fails; need to adjust field path |
| A2 | Pass 1 prompt text style (role + instructions) | Code Examples | Minor: only affects generation quality, not correctness |
| A3 | Pass 2 prompt text style | Code Examples | Minor: only affects generation quality |
| A4 | `json_ui_inspect.rs::inspect_component` should use `global_catalog()` instead of `BUILTIN_TYPES` | Pitfall 5 | Function breaks at runtime if component type check is wrong |
| A5 | `templates/make.rs::json_view_template` is the only static template source (no other templates file) | Architecture | If another templates file exists, it also needs updating |

A5 is LOW risk — confirmed by Glob: only `ferro-cli/src/templates/make.rs` exists under `ferro-cli/src/templates/`. [VERIFIED: codebase grep]

## Open Questions

1. **Does `call_anthropic` need a plain-text variant, or is Pass 1 using tool_use too?**
   - What we know: CONTEXT D-02 says "No structured output constraint" for Pass 1. Pass 1 is plain text.
   - What's unclear: Should `call_anthropic` be modified to make the prefill configurable, or should a new `call_anthropic_plain` function be added?
   - Recommendation: Add `call_anthropic_plain(system, user) -> Result<String, String>` that omits the assistant prefill block entirely. Both Pass 1 and any future non-code AI calls can use it. `call_anthropic` (with `//!` prefill) remains for backward compat with any other code using it.

2. **Does `generation_context` MCP tool reference v1 builder patterns?**
   - What we know: CONTEXT.md defers this to Phase 121+, but notes: "grep and include in scope if matches found."
   - What's unclear: Whether hits exist in `generation_context.rs`.
   - Recommendation: Planner should include a grep task as Wave 0: `grep -n "Spec::builder\|Element::new\|JsonUiView" ferro-mcp/src/tools/generation_context.rs`. If hits found, add update to scope; if none, confirm deferred.

## Environment Availability

This phase is purely code changes within the workspace. No external services, databases, or
CLI tools beyond the standard Rust toolchain are required.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo (rust toolchain) | All | ✓ | (workspace) | — |
| ANTHROPIC_API_KEY | AI generation tests | Runtime env | — | `--no-ai` flag path always works |
| Anthropic API (claude-sonnet-4-5) | Pass 1 + Pass 2 | External | — | Static template fallback |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p ferro-cli -p ferro-mcp --all-features 2>&1 \| tail -20` |
| Full suite command | `cargo test --all-features 2>&1 \| tail -30` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TOOL-01 | `make:json-view` writes `.json` not `.rs` | unit | `cargo test -p ferro-cli make_json_view` | ❌ Wave 0 |
| TOOL-01 | Two-pass AI chain falls back to static on invalid spec | unit | `cargo test -p ferro-cli ai_pass2_fallback` | ❌ Wave 0 |
| TOOL-02 | `json_ui_generate` VIEW_EXAMPLE contains no `Spec::builder()` | unit | `cargo test -p ferro-mcp json_ui_generate::test_example_not_empty` | ✅ (modify) |
| TOOL-03 | `json_ui_catalog` result has `json_schema` and `component_schemas` fields | unit | `cargo test -p ferro-mcp json_ui_catalog` | ✅ (extend) |
| TOOL-04 | `json_ui_inspect` scans `.json` files, not `.rs` | unit | `cargo test -p ferro-mcp json_ui_inspect` | ✅ (modify) |
| TOOL-04 | `code_templates(json_view)` returns 4 templates with no `Spec::builder()` | unit | `cargo test -p ferro-mcp code_templates` | ✅ (extend) |
| D-08 | No v1 references in CLI/MCP paths | grep check | `grep -rn "Spec::builder\|Element::new\|JsonUiView\|Component::" ferro-cli/src ferro-mcp/src --include="*.rs"` | manual |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-cli -p ferro-mcp --all-features 2>&1 | tail -20`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `ferro-cli/src/commands/make_json_view.rs` test: confirm `.json` output path (not `.rs`)
- [ ] `ferro-cli/src/ai.rs` test: `call_anthropic_structured` response parsing
- [ ] `ferro-cli/src/ai.rs` test: fallback to static template when `validate()` fails

*(Existing tests in `json_ui_catalog`, `json_ui_inspect`, `json_ui_generate`, `code_templates` modules need extension, not creation.)*

## Security Domain

This phase modifies AI generation and file-writing logic in the CLI. The security surface is
limited to:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Yes — AI output | `Spec::from_json` + `global_catalog().validate()` already performs structural and schema validation |
| V5 File path traversal | Low risk | `views_dir.join(format!("{name}.json"))` — `is_valid_identifier` check on name already present; JSON extension is hardcoded |
| V6 Cryptography | No | — |
| V2 Authentication | No | — |

No new attack surface beyond what Phase 117/119 already addressed in the validation pipeline.

## Sources

### Primary (HIGH confidence)
- Codebase: `ferro-cli/src/commands/make_json_view.rs` — full file read, confirmed current state
- Codebase: `ferro-cli/src/ai.rs` — full file read, confirmed `call_anthropic` signature and `//!` prefill behavior
- Codebase: `ferro-mcp/src/tools/json_ui_catalog.rs` — full file read, confirmed struct shape and component count (39 built-in + 1 plugin)
- Codebase: `ferro-mcp/src/tools/json_ui_inspect.rs` — full file read, confirmed TODO(Phase 120) comment and `BUILTIN_TYPES` const
- Codebase: `ferro-mcp/src/tools/code_templates.rs` — full file read, confirmed 3 v1 `json_view_templates()`
- Codebase: `ferro-mcp/src/tools/json_ui_generate.rs` — full file read, confirmed `VIEW_EXAMPLE` and `ViewConventions` v1 content
- Codebase: `ferro-json-ui/src/catalog.rs` — read through line 850, confirmed `json_schema()`, `component_schema()`, `validate()`, `prompt()`, `components_sorted()`, `plugin_components_sorted()` APIs
- Codebase: `ferro-cli/src/templates/make.rs` — confirmed `json_view_template()` returns v1 Rust builder code
- CONTEXT.md: full read, all decisions verified against source files

### Secondary (MEDIUM confidence)
- Anthropic Messages API tool_use pattern: CONTEXT.md D-02 cites this as "production-tested path" [CITED: 120-CONTEXT.md]

### Tertiary (LOW confidence)
- Anthropic tool_use response JSON shape (`content[].type=="tool_use" → input`) [ASSUMED — training knowledge]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps; all existing deps confirmed in source
- Architecture: HIGH — all six target files read; API signatures confirmed
- Pitfalls: HIGH — identified from direct source inspection; Pitfall 1 and 5 are non-obvious

**Research date:** 2026-04-21
**Valid until:** 2026-05-21 (stable internal codebase; only valid while Phase 119 is still the active upstream)
