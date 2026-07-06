---
phase: 120-cli-and-mcp-updates
verified: 2026-04-21T14:00:00Z
status: passed
score: 7/7
overrides_applied: 0
---

# Phase 120: CLI & MCP Updates — Verification Report

**Phase Goal:** Update CLI and MCP tools for JSON-UI v2 spec format — zero v1 builder references in generation paths, two-pass AI in make:json-view, json_ui_catalog exposes v2 JSON Schema surface.
**Verified:** 2026-04-21
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `ferro make:json-view` generates v2 flat specs using two-pass generation (describe → structure) | VERIFIED | `make_json_view.rs:27` writes `.json`; `generate_with_ai()` at line 115 calls `call_anthropic_plain` (Pass 1) then `call_anthropic_structured` (Pass 2) |
| 2 | MCP `json_ui_generate` uses `catalog.prompt()` for concise context and `catalog.component_schema()` for per-component structured output | VERIFIED | `json_ui_generate.rs:114` — `component_catalog: global_catalog().prompt()`; `ViewConventions.file_location` = `"src/views/{name}.json"` |
| 3 | MCP `json_ui_catalog` exposes JSON Schema per component | VERIFIED | `json_ui_catalog.rs:18,20` — `pub json_schema: serde_json::Value` and `pub component_schemas: HashMap<String, serde_json::Value>` present; lines 82-99 populate from `cat.json_schema().clone()` and per-component iterator |
| 4 | MCP `json_ui_inspect` works with v2 format | VERIFIED | `json_ui_inspect.rs` module doc says "Scans `src/views/*.json` files (v2 flat spec format)"; `BUILTIN_TYPES` = 0 occurrences; `serde_json::from_str` = 1 match |
| 5 | All code templates in ferro-mcp use v2 spec format | VERIFIED | `code_templates.rs` has 3 occurrences of `ferro-json-ui/v2`; `json_view_handler` template present at line 1084; zero `Spec::builder`/`Element::new` in generation templates |
| 6 | No v1 type references remain in CLI or MCP code (generation paths) | VERIFIED | Zero `Spec::builder`/`Element::new` hits in `ferro-cli/src/ai.rs`, `make_json_view.rs`, `code_templates.rs`, `json_ui_generate.rs`, `generation_context.rs`. Remaining hits are: `module.rs` (out of scope, pre-existing), `json_ui_catalog.rs` `BUILDER_API`/`ACTION_API` documentation constants (intentional reference docs), `make.rs` test assertion strings, `JsonUiViewList` struct name in `json_ui_inspect.rs` (MCP output type, not v1 component) |
| 7 | Generated specs validated against `catalog.json_schema()` before being returned to user | VERIFIED | `make_json_view.rs:148` — `Spec::from_json(&json_str)` + `global_catalog().validate(&spec)` at line 158; fallback to static template on failure |

**Score:** 7/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp/src/tools/json_ui_catalog.rs` | `json_schema: serde_json::Value` and `component_schemas: HashMap` | VERIFIED | Both fields present; populated from `global_catalog()` in `execute()` |
| `ferro-mcp/src/tools/generation_context.rs` | `json_ui_view` pattern uses v2 JSON spec | VERIFIED | `$schema` at line 118, `JsonUi::render_file` at line 134; zero `Spec::builder` hits |
| `ferro-mcp/src/tools/code_templates.rs` | 4 `json_view` templates, all v2 | VERIFIED | 3 JSON spec templates + `json_view_handler` Rust template; zero v1 references |
| `ferro-mcp/src/tools/json_ui_generate.rs` | `VIEW_EXAMPLE` is v2 JSON, `ViewConventions` uses `.json` | VERIFIED | `file_location: "src/views/{name}.json"`; zero v1 references |
| `ferro-mcp/src/tools/json_ui_inspect.rs` | Scans `*.json`, no `BUILTIN_TYPES` | VERIFIED | `BUILTIN_TYPES` = 0; `serde_json::from_str` present; module doc confirms v2 scan |
| `ferro-cli/src/ai.rs` | `call_anthropic_plain`, `call_anthropic_structured`, `build_json_view_pass1`, `build_json_view_pass2` | VERIFIED | All four functions present at lines 27, 99, 174, 199 |
| `ferro-cli/src/commands/make_json_view.rs` | Writes `.json`, two-pass AI, validation fallback | VERIFIED | `.json` at line 27; `generate_with_ai` at line 115; `Spec::from_json` + `validate` at lines 148/158 |
| `ferro-cli/src/templates/make.rs` | `json_view_template` returns v2 JSON spec | VERIFIED | `$schema: "ferro-json-ui/v2"` at line 110; zero v1 markers in template body |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `json_ui_catalog::execute` | `global_catalog().json_schema()` | `cat.json_schema().clone()` | VERIFIED | Line 98 of json_ui_catalog.rs |
| `json_ui_catalog::execute` | `component_schemas` map | `components_sorted` + `plugin_components_sorted` + `component_schema` | VERIFIED | Lines 82-99 |
| `generation_context.rs` `json_ui_view` | v2 JSON spec | `$schema` + `render_file` | VERIFIED | Lines 118, 134 |
| `make_json_view::run` | `ai::build_json_view_pass1` + `call_anthropic_plain` | Pass 1 orchestration | VERIFIED | Lines 115-128 |
| `make_json_view::run` | `ai::build_json_view_pass2` + `call_anthropic_structured` | Pass 2 orchestration | VERIFIED | Lines 132-145 |
| `make_json_view::run` | `Spec::from_json` + `global_catalog().validate` | Post-Pass-2 validation | VERIFIED | Lines 148, 158 |
| `build_json_view_pass1` | `ferro_json_ui::global_catalog().prompt()` | String interpolation in system prompt | VERIFIED | `ai.rs:174` — function exists and documented as using `catalog_prompt` |
| `json_ui_generate::execute` | `global_catalog().prompt()` | `component_catalog` field | VERIFIED | `json_ui_generate.rs:114` |

### Anti-Patterns Found

| File | Pattern | Severity | Notes |
|------|---------|----------|-------|
| `ferro-cli/src/templates/module.rs` | `Spec::builder()`, `Element::new` at lines 83/88/98 | Info | Pre-existing; out of scope for Phase 120 per SUMMARY documentation. Not in a generation path affected by this phase. |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | `Spec::builder()` / `Element::new` in `BUILDER_API` constant | Info | Intentional documentation strings that describe the v1 API. The `builder_api` field is a reference doc preserved by D-24 invariant. Tests at lines 385-390 assert these strings exist in the documentation constant — correct behavior. |
| `ferro-mcp/src/tools/json_ui_inspect.rs` | `JsonUiViewList` struct name | Info | This is the MCP output type name for the list of views, not a v1 component type. No issue. |

No blockers found.

### Human Verification Required

None. All success criteria are verifiable programmatically for this phase.

### Gaps Summary

No gaps. All 7 roadmap success criteria are satisfied:

1. `ferro make:json-view` outputs `.json` with two-pass AI pipeline wired end-to-end in `generate_with_ai()`.
2. `json_ui_generate` uses `global_catalog().prompt()` for component catalog context.
3. `json_ui_catalog` exposes `json_schema` and `component_schemas` fields populated from the Phase 117 catalog.
4. `json_ui_inspect` scans `src/views/*.json`, parses JSON, removes `BUILTIN_TYPES` static list.
5. All `json_view` code templates use v2 JSON spec format (3 JSON + 1 Rust handler).
6. Zero v1 builder references in generation paths. Remaining occurrences are intentional documentation constants, test assertion strings, or pre-existing out-of-scope code (`module.rs`).
7. Validation via `Spec::from_json` + `catalog.validate()` gates AI output before file write; fallback to static template on failure.

---

_Verified: 2026-04-21T14:00:00Z_
_Verifier: Claude (gsd-verifier)_
