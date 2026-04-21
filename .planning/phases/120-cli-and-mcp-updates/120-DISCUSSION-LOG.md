# Phase 120: CLI & MCP Updates - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-21
**Phase:** 120-cli-and-mcp-updates
**Mode:** `--auto` — all areas auto-selected and auto-resolved from codebase analysis
**Areas discussed:** Output Format, Two-Pass AI Generation, Validation, Catalog Schema Exposure, Inspect v2 Scan, Code Templates, V1 Removal

---

## Output Format (make:json-view)

| Option | Description | Selected |
|--------|-------------|----------|
| `.rs` file (Spec::builder) | Keep generating Rust files with builder pattern (v1) | |
| `.json` file (v2 flat spec) | Generate JSON spec files directly | ✓ |

**User's choice:** `.json` file — v2 views are spec files, not Rust modules
**Notes:** mod.rs update logic removed; handlers call `JsonUi::render_file` separately

---

## Two-Pass AI Generation

| Option | Description | Selected |
|--------|-------------|----------|
| Single pass with schema | One call, full spec schema as output constraint | |
| Two-pass: describe → structure | Pass 1 = text description, Pass 2 = JSON spec with schema constraint | ✓ |
| Per-component calls | Separate API call per component type | |

**User's choice:** Two-pass — matches ROADMAP caveat and v0.dev/Lovable pattern
**Notes:** Pass 2 uses Anthropic `tool_use` with `catalog.json_schema()` as input_schema

---

## Validation

| Option | Description | Selected |
|--------|-------------|----------|
| No validation | Return whatever the AI generates | |
| Validate + retry | Validate, retry once on failure | |
| Validate + fallback | Validate, fall back to static template on failure | ✓ |

**User's choice:** Validate + fallback — transparent, no silent garbage
**Notes:** Print validation errors as yellow warnings before writing static fallback

---

## Catalog Schema Exposure (json_ui_catalog)

| Option | Description | Selected |
|--------|-------------|----------|
| New MCP tool | Separate `json_ui_schema` tool | |
| Extend existing struct | Add `json_schema` + `component_schemas` to `JsonUiCatalog` | ✓ |

**User's choice:** Extend existing struct — one MCP call gets everything
**Notes:** Preserves existing field shape (CONTEXT 117 D-24); adds two fields

---

## Inspect v2 Scan

| Option | Description | Selected |
|--------|-------------|----------|
| Keep v1 regex scan | Continue scanning Rust files for JsonUiView patterns | |
| Rewrite for v2 JSON | Scan src/views/*.json, parse flat specs | ✓ |

**User's choice:** Rewrite for v2 JSON — explicit TODO(Phase 120) in source
**Notes:** Remove BUILTIN_TYPES const; extract components from elements[*].type

---

## Code Templates

| Option | Description | Selected |
|--------|-------------|----------|
| Keep v1 Rust templates | Preserve Spec::builder() templates | |
| Replace with v2 JSON templates | Rewrite json_view templates as JSON spec strings | ✓ |
| Both | Keep v1, add v2 alongside | |

**User's choice:** Replace with v2 JSON templates + add json_view_handler template
**Notes:** Three existing templates replaced; one new Rust handler template added

---

## Claude's Discretion

- Whether `call_anthropic_structured` is a flag or a separate function — prefer separate function
- Whether `component_schemas` includes plugin components — include them
- JSON scan depth (flat vs recursive) — flat (`src/views/*.json`) sufficient for v12.0
- Fallback verbosity — print errors to stderr, then write static template

## Deferred Ideas

- Per-component structured output (N API calls) — deferred, full-spec schema is sufficient
- Retry on validation failure — deferred, one-shot + fallback is the v12.0 policy
- `ferro json-ui:validate` CLI command — Phase 121+
- Watch mode for make:json-view — deferred
