# Phase 121: Documentation & Field Test - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning
**Mode:** `--auto` — decisions auto-selected from codebase analysis and upstream phase context. Downstream agents may override any auto-choice by editing this file before `/gsd-plan-phase`.

<domain>
## Phase Boundary

Complete the v12.0 milestone with two deliverables:

1. **Documentation rewrite** — all JSON-UI docs updated from v1 (Rust builder pattern) to v2 (flat JSON spec format). Includes 6 existing pages + 1 overview page + 2 new pages.
2. **Field test** — one realistic dashboard page added to the ferro `app/` sample app as a JSON spec file with a data-only handler. Proves the full v2 pipeline (load → validate → render) works end-to-end in a real handler.

**What this phase does NOT do:**
- Change any Rust code in ferro-json-ui, ferro-mcp, ferro-cli, or framework (Phases 115–120 are complete)
- Add new components or change Spec/Element shape
- Work in the gestiscilo-it repo (gestiscilo-it/app is currently empty — field test uses ferro app/)
- Update non-JSON-UI docs

</domain>

<decisions>
## Implementation Decisions

### D-01: Doc pages to rewrite (v1 → v2)

**Decision:** Rewrite all 6 existing `docs/src/json-ui/` pages and the `docs/src/features/json-ui.md` overview page. Every v1 reference (`JsonUiView`, `ComponentNode`, `Component::`, `JsonUi::render(&view, ...)`) is replaced with v2 equivalents (JSON spec object, `JsonUi::render_file`, `JsonUi::render_json`).

Pages to rewrite:
- `docs/src/features/json-ui.md` — overview (currently describes `JsonUiView` builder)
- `docs/src/json-ui/getting-started.md` — primary tutorial (all code samples are v1)
- `docs/src/json-ui/components.md` — per-component reference (1427 lines, all v1 props API)
- `docs/src/json-ui/actions.md` — action binding examples (v1 ComponentNode syntax)
- `docs/src/json-ui/data-binding.md` — data paths and visibility (v1 data_path as Rust field)
- `docs/src/json-ui/layouts.md` — layout configuration (v1 JsonUiView.layout() call)
- `docs/src/json-ui/plugins.md` — plugin registration and rendering (v1 Component::Plugin)

**Why:** All 6 docs plus the overview page still use v1 Rust builder API. Phase 115 deleted those types; any developer following these docs will get compile errors.

**How to apply:** The new example pattern throughout is: create `src/views/{name}.json`, put the flat spec in it, handler calls `JsonUi::render_file("views/{name}.json", data)`. Data binding via `$data`/`$template` in props instead of `data_path` Rust field references.

### D-02: New doc pages to create

**Decision:** Add two new pages:
- `docs/src/json-ui/json-schema.md` — JSON Schema export, IDE validation, AI structured output, external validation
- `docs/src/json-ui/expressions.md` — `$data` and `$template` expression system with explicit hard cap rationale

Add both to `docs/src/SUMMARY.md` in the JSON-UI section.

**Why:** Success criteria 2 and 3 explicitly call for these. No expressions doc exists anywhere; the $data/$template system is entirely undocumented. JSON Schema export via `ferro json-ui:schema` is likewise undocumented.

**How to apply:**
- `expressions.md`: Covers `{"$data": "/path"}` (type-preserving), `{"$template": "Hello, {/user/name}!"}` (string interpolation), resolution scope (Element.props only), slash-path syntax. Includes explicit hard cap section: "No `$if`, `$for`, `$state`, `$bind`" with SDUI inner-platform risk explanation (Airbnb/DoorDash/Lyft precedent).
- `json-schema.md`: Covers `ferro json-ui:schema` CLI, per-component schema via `json_ui_catalog` MCP tool, IDE integration (VS Code `$schema` field), AI structured output (Pass 2 constrained generation), `jsonschema` crate for server-side validation.

### D-03: Components doc format

**Decision:** Keep `components.md` as a detailed per-component reference — rewrite all props tables and code examples from v1 Rust struct syntax to v2 JSON props objects. Each component section shows the JSON spec snippet (not Rust).

Example transformation:
```
// v1 (deleted)
Component::Card(CardProps { title: "...", ... })

// v2
{ "type": "Card", "props": { "title": "...", "description": null } }
```

**Why:** The 1427-line components.md is the primary reference. Developers and agents need to see exact prop names/types in JSON format, not Rust struct syntax.

### D-04: Field test — pagamenti demo in ferro app/

**Decision:** Add a payments dashboard demo to the ferro `app/` sample app:

- `app/src/views/pagamenti.json` — v2 flat spec showing a payments list with StatCard summary and DataTable. Uses `$data` expressions to bind to handler-provided data.
- `app/src/controllers/pagamenti.rs` — data-only handler: `JsonUi::render_file("views/pagamenti.json", data)` where `data` is a `serde_json::json!({...})` with pagamenti records.
- Register route in `app/src/routes.rs`.

The demo represents a realistic dashboard page (orders/payments list with total stats) — covers the full v2 pipeline: JSON spec file → `render_file` → expression resolution → catalog validation → HTML render.

**Why:** gestiscilo-it/app is empty, so no real Rust v1 page exists to convert. The ferro sample app provides the same proof: an existing app using JSON spec files with real handler data. Success criterion 5 (renders identically to Rust-built version) is satisfied by verifying the rendered HTML matches what a manually-built Spec would produce.

### D-05: Success criterion 4 interpretation

**Decision:** "Handler reduced to data-only" means the Rust handler has zero component-building code — only data assembly and `JsonUi::render_file`. The JSON spec file is the sole UI definition.

**Why:** This is the v12.0 architectural payoff: handlers become data providers, UI shape lives in versioned JSON files. The demo must show this distinction clearly.

### D-06: SUMMARY.md update

**Decision:** Update `docs/src/SUMMARY.md` JSON-UI section to:
```
# JSON-UI

- [Getting Started](json-ui/getting-started.md)
- [Components](json-ui/components.md)
- [Actions](json-ui/actions.md)
- [Data Binding & Visibility](json-ui/data-binding.md)
- [Expressions](json-ui/expressions.md)
- [Layouts](json-ui/layouts.md)
- [Plugins](json-ui/plugins.md)
- [JSON Schema](json-ui/json-schema.md)
```

### Claude's Discretion
- Whether `components.md` is rewritten inline or split into multiple files — keep as one file (1427 lines is manageable; splitting adds navigation complexity)
- The exact pagamenti data shape (number of records, field names) — use realistic Italian payment records (importo, data, stato, descrizione) matching a typical gestiscilo-style app
- Whether the expressions doc explains the `\{` and `\}` escape syntax — include it (it's part of the spec)
- Whether `json-schema.md` shows a full example `ferro json-ui:schema` output — show a trimmed example (enough to understand the structure, not the full 40KB schema)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and success criteria
- `.planning/ROADMAP.md` §"Phase 121: Documentation & Field Test" — goal, 5 success criteria, depends-on (Phase 120)

### Upstream locked decisions (locked — do not re-open)
- `.planning/phases/115-spec-v2-data-structures/115-CONTEXT.md` — v2 spec shape: `$schema`, `root`, `elements` flat map, `Element.type`/`props`/`children`/`action`/`visible`; ID format; depth limit 3
- `.planning/phases/116-flat-element-renderer/116-CONTEXT.md` — renderer API: `render_spec_to_html`, `JsonUi::render_json`, component dispatch shape
- `.planning/phases/117-catalog-and-json-schema/117-CONTEXT.md` — `global_catalog()`, `Catalog::validate`, `catalog.prompt()`, `catalog.json_schema()`, `catalog.component_schema(name)`, 39 built-in components, `ferro json-ui:schema` CLI export
- `.planning/phases/118-server-side-expressions/118-CONTEXT.md` — `$data`/`$template` semantics: type-preserving substitution, slash-path syntax, scope (Element.props only), hard cap (no $if/$for/$state/$bind)
- `.planning/phases/119-page-loader/119-CONTEXT.md` — `JsonUi::render_file` API, file location convention (`src/views/*.json`), `LoadError` variants
- `.planning/phases/120-cli-and-mcp-updates/120-CONTEXT.md` — `ferro make:json-view` generates `.json` files; `json_ui_generate` MCP context; `json_ui_catalog` JSON Schema fields; `json_ui_inspect` scans `src/views/*.json`

### Docs structure
- `docs/src/SUMMARY.md` — current JSON-UI section (6 pages to rewrite + 2 new pages to add)
- `docs/src/json-ui/getting-started.md` — primary tutorial (328 lines, all v1)
- `docs/src/json-ui/components.md` — component reference (1427 lines, all v1)
- `docs/src/json-ui/data-binding.md` — data binding (309 lines, all v1)
- `docs/src/json-ui/actions.md` — actions (232 lines, v1)
- `docs/src/json-ui/layouts.md` — layouts (316 lines, v1)
- `docs/src/json-ui/plugins.md` — plugins (334 lines, v1)
- `docs/src/features/json-ui.md` — overview page (v1 quick example, how-it-works)

### Framework entry points to document
- `framework/src/json_ui/mod.rs` — `JsonUi::render_file`, `JsonUi::render_json`, `JsonUi::render_with_config`
- `ferro-json-ui/src/expression.rs` — `resolve_expressions` (internal; document the user-facing `$data`/`$template` syntax, not the Rust API)
- `ferro-json-ui/src/catalog.rs` — `global_catalog()`, `Catalog::json_schema()`, `Catalog::component_schema(name)`, `Catalog::validate()`
- `ferro-cli/src/commands/make_json_view.rs` — `ferro make:json-view` command (generates `.json` file)

### App sample to add
- `app/src/views/` — empty directory; add `pagamenti.json` here
- `app/src/controllers/` — add `pagamenti.rs` data-only handler
- `app/src/routes.rs` — register GET `/pagamenti` route

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 120 completed `make:json-view`, `json_ui_catalog`, `json_ui_inspect`, `code_templates` — all generating v2 format. These are the authoritative examples of v2 usage patterns for docs.
- `ferro-json-ui/src/expression.rs` — source of truth for `$data`/`$template` semantics. Read before writing expressions.md.
- `ferro-json-ui/src/catalog.rs` — source of truth for `json_schema()` and `component_schema()` output shape. Read before writing json-schema.md.
- Existing `docs/src/json-ui/*.md` — structure (headings, table layouts, code fence style) should be preserved even as content is rewritten.

### Established Patterns
- Docs use fenced Rust code blocks with `use ferro::*` import style
- JSON code examples use `json` fence type
- Component reference tables: `| Field | Type | Default | Description |`
- All doc examples are self-contained (don't reference app-specific setup)

### Integration Points
- `app/src/routes.rs` — uses `get!()` macro for route registration; pagamenti handler follows the same pattern as existing routes
- `app/src/controllers/mod.rs` — add `pub mod pagamenti;` declaration
- The pagamenti.json spec file must pass `global_catalog().validate()` — check component names against the 39 built-in types in Phase 117

</code_context>

<specifics>
## Specific Ideas

- `expressions.md` should prominently feature the hard cap with the "inner platform effect" explanation. The Airbnb/DoorDash/Lyft precedent (all converged to max depth 3, all eventually limited expressions) is the concrete rationale that makes the cap credible, not arbitrary.
- `json-schema.md` should show the `$schema: "ferro-json-ui/v2"` field and explain its role as the IDE hook (VS Code resolves this to the exported schema via `json.schemas` workspace setting).
- The pagamenti.json field test should use `$data` expressions (not static strings) to prove expression resolution works in the real pipeline. Example: `{"type": "Text", "props": {"content": {"$data": "/meta/totale_formattato"}}}`.
- The `getting-started.md` rewrite should lead with the JSON-first mental model: "define the page as a JSON file, handler provides the data." The Rust code appears only for the handler, not for view construction.
- `components.md` rewrite: document `"children"` as an array of element IDs (flat references, not nested objects) — this is the biggest conceptual shift from v1 where children were `Vec<ComponentNode>`.

</specifics>

<deferred>
## Deferred Ideas

- `ferro json-ui:validate` CLI command (validate a spec file on disk) — mentioned in Phase 120 deferred ideas; still out of scope
- Per-component doc pages (splitting components.md) — deferred, single file is sufficient for v12.0
- Docs for `ferro-projections` v2 rendering (Spec::from_service_def flow) — separate concern, not JSON-UI v2 docs
- Interactive schema explorer in docs — requires frontend work, v13.0+ concern
- Dark mode screenshots in docs — out of scope for v12.0
- Migration guide (v1 → v2) — not needed; ferro is pre-1.0, no external consumers. If added, belongs in `docs/src/upgrading/` not json-ui section.

</deferred>

---

*Phase: 121-documentation-and-field-test*
*Context gathered: 2026-04-21*
