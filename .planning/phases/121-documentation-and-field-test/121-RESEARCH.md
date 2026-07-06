# Phase 121: Documentation & Field Test - Research

**Researched:** 2026-04-21
**Domain:** Documentation rewrite (v1 → v2 JSON-UI) + ferro sample app field test
**Confidence:** HIGH

## Summary

Phase 121 is a documentation-and-validation phase that closes the v12.0 milestone. All upstream Rust work (Phases 115–120) is complete on the `v12.0/json-ui-v2` branch. The phase has two concrete deliverables:

1. **Docs rewrite:** 7 existing pages (6 in `docs/src/json-ui/` + the overview at `docs/src/features/json-ui.md`) all reference deleted v1 types (`JsonUiView`, `ComponentNode`, `Component::`). Every page must be rewritten for v2 format (flat JSON spec, `JsonUi::render_file`, `$data`/`$template`). Two new pages must be created: `expressions.md` and `json-schema.md`.

2. **Field test:** Add a `pagamenti` dashboard demo to `app/src/` — a `src/views/pagamenti.json` spec file and a data-only Rust handler. This proves the full v2 pipeline works end-to-end in the sample app.

**Critical finding:** `JsonUi::render_file` was planned in Phase 119 D-05 but was NOT implemented. Phase 119 shipped only `load_cached` and `Spec::merge_data` in `ferro-json-ui`. The framework method `render_file` is missing from `framework/src/json_ui/mod.rs`. The field test cannot use `JsonUi::render_file` without first adding it. The planner must address this — either add `render_file` as Plan 0 of this phase (a small Rust addition), or the field test handler calls `load_cached` + `JsonUi::render` directly.

**Primary recommendation:** Add `JsonUi::render_file` as a Wave 0 task (it is ~30 lines following the existing pattern), then proceed with docs and field test in parallel. The CONTEXT.md says "no Rust code changes" but this is a prerequisite gap from Phase 119, not new scope.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Rewrite all 6 existing `docs/src/json-ui/` pages and the `docs/src/features/json-ui.md` overview page. All v1 references (`JsonUiView`, `ComponentNode`, `Component::`, `JsonUi::render(&view, ...)`) replaced with v2 equivalents.
- **D-02:** Add two new pages: `docs/src/json-ui/expressions.md` and `docs/src/json-ui/json-schema.md`. Add both to SUMMARY.md.
- **D-03:** Keep `components.md` as a single file; rewrite props tables and code examples from v1 Rust syntax to v2 JSON props format.
- **D-04:** Field test adds `app/src/views/pagamenti.json` + `app/src/controllers/pagamenti.rs` + route in `app/src/routes.rs`.
- **D-05:** "Handler reduced to data-only" = zero component-building code in the Rust handler, only data assembly and `JsonUi::render_file`.
- **D-06:** SUMMARY.md JSON-UI section updated to include Expressions and JSON Schema pages in order.

### Claude's Discretion

- Whether `components.md` is split — keep as one file.
- Exact pagamenti data shape — use realistic Italian payment records (importo, data, stato, descrizione).
- Whether expressions doc explains escape syntax — include `\{` and `\}`.
- Whether `json-schema.md` shows full `ferro json-ui:schema` output — show trimmed example.

### Deferred Ideas (OUT OF SCOPE)

- `ferro json-ui:validate` CLI command
- Per-component doc pages (splitting components.md)
- Docs for ferro-projections v2 rendering
- Interactive schema explorer
- Dark mode screenshots
- Migration guide (v1 → v2)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DOC-01 | All JSON-UI docs rewritten for v2 spec format, no v1 references | All 7 pages confirmed to contain v1 content; v2 patterns fully documented below |
| DOC-02 | JSON Schema export documented with usage examples | `ferro json-ui:schema` CLI confirmed in `ferro-cli/src/commands/json_ui_schema.rs`; `Catalog::json_schema()` API confirmed |
| FIELD-01 | One gestiscilo-style page converted to JSON spec in ferro app/ | v2 pipeline confirmed working on branch; render_file gap identified and solution documented |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Docs (pages 1–7 rewrite) | `docs/src/` markdown | — | Pure documentation, no Rust changes |
| Expressions doc | `docs/src/json-ui/expressions.md` | `ferro-json-ui/src/expression.rs` | Docs surface the user API; expression.rs is the source of truth |
| JSON Schema doc | `docs/src/json-ui/json-schema.md` | `ferro-cli/src/commands/json_ui_schema.rs` | Docs surface CLI + catalog API |
| Field test — spec file | `app/src/views/pagamenti.json` | `ferro-json-ui/src/catalog.rs` (validates) | JSON spec file is data, not code |
| Field test — handler | `app/src/controllers/pagamenti.rs` | `framework/src/json_ui/mod.rs` | Handler calls render_file; framework owns HTTP pipeline |
| render_file (missing) | `framework/src/json_ui/mod.rs` | `ferro-json-ui/src/loader.rs` | Framework bridges loader.rs cache to HTTP response |

## Standard Stack

No new dependencies. All work is docs + Rust in the existing workspace.

### Verified Present on v12.0 Branch [VERIFIED: codebase]

| Module | Location | Purpose in Phase 121 |
|--------|----------|----------------------|
| `Spec`, `Element`, `SpecBuilder` | `ferro-json-ui/src/spec.rs` | v2 type model documented in getting-started.md |
| `resolve_expressions` | `ferro-json-ui/src/expression.rs` | Source of truth for expressions.md |
| `global_catalog()`, `Catalog::json_schema()` | `ferro-json-ui/src/catalog.rs` | Source of truth for json-schema.md |
| `load_cached` | `ferro-json-ui/src/loader.rs` | Called by render_file (to be added) |
| `Spec::merge_data` | `ferro-json-ui/src/spec.rs` | Called by render_file |
| `render_spec_to_html_with_plugins` | `ferro-json-ui/src/render/mod.rs` | Called by render_file |
| `ferro json-ui:schema` CLI | `ferro-cli/src/commands/json_ui_schema.rs` | Documented in json-schema.md |
| `JsonUi::render` | `framework/src/json_ui/mod.rs` | Already exists; render_file follows same pattern |

### What Is Missing [VERIFIED: codebase read of framework/src/json_ui/mod.rs on v12.0]

| Item | Status | Resolution |
|------|--------|------------|
| `JsonUi::render_file` | NOT in `framework/src/json_ui/mod.rs` | Add as ~30-line function (Wave 0) |

## Architecture Patterns

### System Architecture Diagram (Field Test Pipeline)

```
pagamenti.json (src/views/pagamenti.json)
      |
      v
ferro app server receives GET /pagamenti
      |
      v
pagamenti::index handler
  |-- assembles serde_json data (payment records + meta)
  |-- calls JsonUi::render_file("views/pagamenti.json", data)
         |
         v
     load_cached(path, reload_if_changed)
     -> reads file, Spec::from_json, global_catalog().validate()
         |
         v
     spec.merge_data(handler_data)   -- handler data overwrites spec.data keys
         |
         v
     JsonUi::resolve(spec)
     -> resolve_actions (handler names → URLs)
     -> resolve_expressions ($data/$template in props)
         |
         v
     render_spec_to_html_with_plugins(spec, data)
     -> DashboardLayout wraps rendered HTML
         |
         v
     HttpResponse::text(html).header("Content-Type", "text/html; charset=utf-8")
```

### v2 Spec File Format (source of truth for all docs rewrites)

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Pagamenti",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "Pagamenti" },
      "children": ["stats_row", "payments_table"]
    },
    "stats_row": {
      "type": "StatCard",
      "props": {
        "label": "Totale",
        "value": { "$data": "/meta/totale_formattato" }
      }
    },
    "payments_table": {
      "type": "DataTable",
      "props": {
        "columns": [
          { "key": "data", "label": "Data", "format": "date" },
          { "key": "descrizione", "label": "Descrizione" },
          { "key": "importo", "label": "Importo", "format": "currency" },
          { "key": "stato", "label": "Stato" }
        ],
        "data_path": "/pagamenti",
        "empty_message": "Nessun pagamento trovato."
      }
    }
  }
}
```
[VERIFIED: matches Spec struct in ferro-json-ui/src/spec.rs on v12.0 branch]

### render_file Implementation Pattern (missing from framework, must add)

```rust
// In framework/src/json_ui/mod.rs, add to impl JsonUi:

/// Load a v2 spec file, merge handler data, and render to HTML.
///
/// Uses the process-level spec cache (`ferro_json_ui::load_cached`).
/// In dev (`!Config::is_production()`), reloads on mtime change.
/// In production, the spec is loaded once and cached for the process lifetime.
pub fn render_file(
    path: impl AsRef<std::path::Path>,
    handler_data: serde_json::Value,
) -> Response {
    Self::render_file_with_config(path, handler_data, &JsonUiConfig::new())
}

pub fn render_file_with_config(
    path: impl AsRef<std::path::Path>,
    handler_data: serde_json::Value,
    config: &JsonUiConfig,
) -> Response {
    let reload = !crate::config::Config::is_production();
    let arc_spec = ferro_json_ui::load_cached(path.as_ref(), reload)
        .map_err(|e| HttpResponse::text(format!("spec load error: {e}")).status(500))?;
    let spec = (*arc_spec).clone().merge_data(handler_data);
    Self::render_with_config(&spec, &serde_json::Value::Null, config)
}
```

Note: `Config::is_production()` — verify the exact method name in `framework/src/config/mod.rs` before using. [ASSUMED — method exists based on 119-CONTEXT.md reference]

### Expression System (source of truth for expressions.md)

Two expression shapes only — no others. Hard cap enforced by codebase, not just policy.

```json
// $data — type-preserving field extraction
{ "$data": "/path/to/value" }

// $template — string interpolation (result is always a string)
{ "$template": "Hello, {/user/name}!" }
```

Rules (from `ferro-json-ui/src/expression.rs`):
- Resolved in `Element.props` only — `spec.title`, `spec.layout`, `el.action`, `el.visible`, `el.children` are NOT walked
- Single-pass: expressions inside resolved `$data` output are NOT re-resolved (inner-platform firewall)
- Missing paths → `null` for `$data`, `""` for `{/path}` placeholders in `$template`
- Infallible: malformed expressions degrade to literal JSON, no panic

[VERIFIED: ferro-json-ui/src/expression.rs on v12.0 branch, read fully]

### v1 → v2 Pattern Mapping (for all docs rewrites)

| v1 Pattern | v2 Equivalent |
|-----------|---------------|
| `JsonUiView::new().title("X").component(...)` | JSON spec file with `"$schema"`, `"root"`, `"elements"` |
| `ComponentNode { key, component: Component::Card(CardProps {...}), ... }` | `"my_id": { "type": "Card", "props": { ... } }` |
| `Component::Text(TextProps { content, element })` | `"my_id": { "type": "Text", "props": { "content": "...", "element": "h1" } }` |
| `InputProps { data_path: Some("/data/user/name") }` | `"name_field": { "props": { "data_path": { "$data": "/user/name" } } }` |
| `JsonUi::render(&view, &data)` | `JsonUi::render_file("views/name.json", data)` |
| `ComponentNode.children = vec![ComponentNode {...}]` | `"children": ["child_id"]` — IDs, not nested objects |
| `visibility: Some(Visibility {...})` | `"visible": { "field": "...", "op": "...", "value": "..." }` |

[VERIFIED: spec.rs (Spec, Element structs), expression.rs, 121-CONTEXT.md decisions]

### Project Structure for Field Test

```
app/src/
├── controllers/
│   ├── mod.rs           -- add: pub mod pagamenti;
│   └── pagamenti.rs     -- new: data-only handler
├── views/
│   └── pagamenti.json   -- new: v2 spec file
└── routes.rs            -- add: get!("/pagamenti", ...).name("pagamenti.index")
```

[VERIFIED: app/src/controllers/mod.rs and app/src/routes.rs on v12.0 branch]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Spec validation in field test | Custom JSON validation | `global_catalog().validate()` inside `load_cached` | Already runs at load time in the cache pipeline |
| Expression resolution in handler | Manual string substitution | `JsonUi::render_file` → `resolve_expressions` | Runs automatically in render pipeline |
| HTML rendering from spec | Custom HTML builder | `render_spec_to_html_with_plugins` | Already handles all 39 component types + plugins |
| IDE schema validation | Custom schema generation | `ferro json-ui:schema --output schema.json` | Outputs the catalog's full JSON Schema document |

## Common Pitfalls

### Pitfall 1: `render_file` is missing — field test cannot compile
**What goes wrong:** Handler calls `JsonUi::render_file(...)` but the method does not exist on the v12.0 branch. Compilation fails.
**Why it happens:** Phase 119 shipped `load_cached` and `Spec::merge_data` but not the framework wrapper. Phase 121's CONTEXT says "no Rust code changes" — but this is a gap, not a choice.
**How to avoid:** Add `render_file` as Wave 0 before writing the field test handler. The implementation is ~30 lines following the existing `render_with_config` pattern. See Code Examples section.
**Warning signs:** `error[E0599]: no method named 'render_file' found for struct 'JsonUi'`

### Pitfall 2: pagamenti.json must use only catalog-valid component types
**What goes wrong:** Using a component type name that doesn't match the 39 built-ins exactly (case-sensitive) causes `CatalogError::UnknownType` at load time.
**Why it happens:** `load_cached` calls `global_catalog().validate()` — the 39 type names must match exactly.
**How to avoid:** Use component names from the COMPONENT_CATALOG constant: `DataTable` (not `Table`), `StatCard`, `Card`, `Text`, `PageHeader`. Check `ferro-json-ui/src/catalog.rs` BUILTIN_SPECS for the exact list.
**Warning signs:** `Error: spec failed catalog validation: unknown component type 'Table'`

### Pitfall 3: docs still on master branch, not v12.0
**What goes wrong:** Editing docs in the working tree (master) instead of on the `v12.0/json-ui-v2` branch. Phase 121 work belongs on the v12.0 branch.
**Why it happens:** The working directory is currently on master.
**How to avoid:** Confirm `git branch` shows `v12.0/json-ui-v2` before writing any files. All Phase 121 work must be committed to that branch.
**Warning signs:** `git status` shows changes to docs/ files from master (which has v1 docs).

### Pitfall 4: `children` in v2 are element IDs, not nested objects
**What goes wrong:** Docs example shows `"children": [{ "type": "Button", ... }]` (v1 ComponentNode style). Spec validation rejects this — children must be strings (element IDs).
**Why it happens:** v1 had `Vec<ComponentNode>` (nested objects); v2 is a flat map with ID references.
**How to avoid:** All `children` values in docs and pagamenti.json must be `["element_id"]` arrays.

### Pitfall 5: `$data` and `$template` expressions apply to props values, not keys
**What goes wrong:** Writing `{ "$data": "..." }` as a prop key name instead of as a prop value. E.g., `"props": { "$data": "/user/name" }` instead of `"props": { "content": { "$data": "/user/name" } }`.
**Why it happens:** The expression shape is an object substitution: the entire object `{"$data": "..."}` is replaced by the resolved value. The `$data` key must be the ONLY key in the object.
**How to avoid:** `expression.rs::is_data_expr` checks `obj.len() == 1` — any additional keys in the same object cause it to fall through to literal JSON.

### Pitfall 6: `DataTable` vs `Table` naming
**What goes wrong:** The old v1 docs reference `TableProps` for a `Table` component. The v2 catalog has `DataTable` (not `Table`) for data-bound tables. `Table` is a legacy type and may not exist in the v2 catalog.
**Why it happens:** Component naming changed between v1 and v2.
**How to avoid:** Verify the exact type name in the catalog before documenting. The COMPONENT_CATALOG constant in lib.rs and catalog.rs BUILTIN_SPECS are the source of truth.

### Pitfall 7: `ferro json-ui:schema` shells out to `cargo run`
**What goes wrong:** Assuming `ferro json-ui:schema` is a fast in-process call. It actually shells out to `cargo run --quiet -- json-ui:schema`, which can take 30+ seconds on first run (compilation).
**Why it happens:** `json_ui_schema.rs` uses the same shell-out pattern as `db_status`. The schema export requires running the user's app binary.
**How to avoid:** In docs, note that the command requires a compiled Ferro project. The schema is generated from the app's own catalog (including custom plugins). This is not a defect — document it honestly.

## Code Examples

### Handler Pattern (pagamenti.rs)

```rust
// Source: CONTEXT.md D-04, D-05; render_file pattern from 119-CONTEXT.md D-05
use ferro::{handler, Response, JsonUi};

#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "meta": {
            "totale_formattato": "€ 1.245,00"
        },
        "pagamenti": [
            {
                "data": "2026-04-20",
                "descrizione": "Abbonamento mensile",
                "importo": "€ 99,00",
                "stato": "Completato"
            },
            {
                "data": "2026-04-15",
                "descrizione": "Ordine #1042",
                "importo": "€ 246,00",
                "stato": "Completato"
            }
        ]
    });
    JsonUi::render_file("views/pagamenti.json", data)
}
```

### render_file Implementation

```rust
// Source: 119-CONTEXT.md D-05; pattern matches existing JsonUi::render_with_config
// Add to impl JsonUi in framework/src/json_ui/mod.rs

/// Load a v2 spec file by path, merge handler data, and render to HTML.
pub fn render_file(
    path: impl AsRef<std::path::Path>,
    handler_data: serde_json::Value,
) -> Response {
    Self::render_file_with_config(path, handler_data, &JsonUiConfig::new())
}

pub fn render_file_with_config(
    path: impl AsRef<std::path::Path>,
    handler_data: serde_json::Value,
    config: &JsonUiConfig,
) -> Response {
    let reload = !crate::Config::is_production();
    let arc_spec = ferro_json_ui::load_cached(path.as_ref(), reload)
        .map_err(|e| HttpResponse::text(format!("Failed to load spec: {e}")).status(500))?;
    let spec = (*arc_spec).clone().merge_data(handler_data);
    Self::render_with_config(&spec, &serde_json::Value::Null, config)
}
```

Note: Verify exact `Config::is_production()` method name. [ASSUMED — see Open Questions]

### Getting Started Handler (new v2 pattern for docs)

```rust
use ferro::{handler, JsonUi, Response};

#[handler]
pub async fn dashboard() -> Response {
    let data = serde_json::json!({
        "orders_today": 42
    });
    JsonUi::render_file("views/dashboard.json", data)
}
```

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Dashboard",
  "layout": "dashboard",
  "root": "welcome",
  "elements": {
    "welcome": {
      "type": "Card",
      "props": { "title": "Welcome" },
      "children": ["orders_stat"]
    },
    "orders_stat": {
      "type": "StatCard",
      "props": {
        "label": "Orders Today",
        "value": { "$data": "/orders_today" }
      }
    }
  }
}
```

### Expression Examples for expressions.md

```json
// $data: type-preserving field extraction
{ "$data": "/user/name" }           // resolves to "Alice" (string)
{ "$data": "/order/total" }          // resolves to 99.50 (number)
{ "$data": "/flags/verified" }       // resolves to true (boolean)
{ "$data": "/missing_key" }          // resolves to null

// $template: string interpolation (always a string result)
{ "$template": "Hello, {/user/name}!" }           // "Hello, Alice!"
{ "$template": "Order #{/order/id} — {/order/status}" }

// Where to use them — inside props values only:
"props": {
  "label": "Totale Ordini",
  "value": { "$template": "€ {/totale}" },
  "data_path": { "$data": "/source_path" }
}

// Hard cap — these do NOT exist:
// $if, $for, $state, $bind, $map, $reduce — not in ferro-json-ui
```

[VERIFIED: expression.rs const EXPR_DATA_KEY, EXPR_TEMPLATE_KEY]

### JSON Schema Usage Example for json-schema.md

```bash
# Export the full Spec JSON Schema for your app (includes custom plugins)
ferro json-ui:schema --output schema.json --pretty

# Export per-component schema
ferro json-ui:schema --component DataTable --pretty
```

```jsonc
// Add to .vscode/settings.json for IDE validation:
{
  "json.schemas": [
    {
      "fileMatch": ["src/views/*.json"],
      "url": "./schema.json"
    }
  ]
}
```

[VERIFIED: json_ui_schema.rs CLI implementation; VS Code json.schemas workspace setting is standard]

## Runtime State Inventory

Phase 121 is greenfield additions (docs + new files in app/) with no renames or data migrations. Not applicable.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `JsonUiView::new().component(ComponentNode { ... })` | Flat JSON spec file with `"elements"` map | Phase 115 | Views are data files, not Rust code |
| `data_path: Some("/data/user/name")` Rust field | `"props": { "data_path": { "$data": "/user/name" } }` | Phase 118 | Expressions in JSON props replace Rust field references |
| `JsonUi::render(&view, &data)` | `JsonUi::render_file("views/name.json", data)` | Phase 119 (partial) | One-stop file-backed render entry |
| Component children as `Vec<ComponentNode>` (nested) | `"children": ["id1", "id2"]` (flat ID references) | Phase 115 | Flat map eliminates nesting depth issues |
| `"$schema": "ferro-json-ui/v1"` | `"$schema": "ferro-json-ui/v2"` | Phase 115 | Version tag enables schema-based validation |

**Deprecated/outdated (v1 types deleted in Phase 115):**
- `JsonUiView` — deleted
- `ComponentNode` — deleted
- `Component::*` enum variants — deleted
- `JsonUiView::new().layout(...)` builder — deleted
- `data_path` as a Rust struct field — replaced by `$data` expression

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Config::is_production()` method exists in `framework/src/config/mod.rs` | Code Examples, render_file | render_file implementation uses wrong method; needs adjustment |
| A2 | DataTable is the v2 name for tabular data component (not Table) | Pitfall 6, pagamenti.json | Field test spec fails catalog validation; rename needed |
| A3 | The 39 built-in components from Phase 117 include all types used in pagamenti demo (Card, StatCard, DataTable, Text) | pagamenti.json spec | Field test spec fails catalog validation if any type is unknown |

## Open Questions

1. **Does `Config::is_production()` exist with that exact signature?**
   - What we know: 119-CONTEXT.md D-05 says to use `!Config::is_production()` for dev mode detection.
   - What's unclear: Exact method name and module path in the framework crate.
   - Recommendation: Planner should include a task to `grep -rn "is_production" framework/src/` before writing render_file.

2. **Is `DataTable` or `Table` the v2 catalog name for data-bound tables?**
   - What we know: The current lib.rs (master branch) exports `DataTableProps`; the COMPONENT_CATALOG constant says `DataTable`. v1 docs use `Table`.
   - What's unclear: Whether Phase 116 preserved or renamed the component.
   - Recommendation: Planner should read `ferro-json-ui/src/catalog.rs::BUILTIN_SPECS` on v12.0 branch to confirm exact type names before writing pagamenti.json.

3. **Should `render_file` be added as Phase 121 Wave 0, or is a workaround acceptable?**
   - What we know: render_file is missing; CONTEXT.md says "no Rust code changes."
   - What's unclear: Whether the user accepts this as a Phase 119 gap that must be filled, or wants the field test to use `load_cached` + `JsonUi::render` directly.
   - Recommendation: Add render_file as Wave 0 of Phase 121. It is a small (~30 line) additive function that was planned in Phase 119 and is needed to fulfill DOC-01 (docs must document the actual API). If the user vetoes Rust changes, the handler can use: `let spec = ferro_json_ui::load_cached(...)?; JsonUi::render(&spec, &data)` — but this pattern is less ergonomic and inconsistent with Phase 120's generated templates.

## Environment Availability

All work is in-repo. No external services needed.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| git branch v12.0/json-ui-v2 | All phase 121 work | ✓ | current | — |
| cargo (Rust toolchain) | render_file + field test | ✓ | (workspace) | — |
| ferro app/ | Field test | ✓ | v12.0 branch | — |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p ferro-json-ui -p framework --all-features 2>&1 \| tail -20` |
| Full suite command | `cargo test --all-features 2>&1 \| tail -30` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DOC-01 | No v1 references in any docs/src/json-ui/ file | grep check | `grep -rn "JsonUiView\|ComponentNode\|Component::" docs/src/json-ui/ docs/src/features/json-ui.md` | manual |
| DOC-02 | json-schema.md and expressions.md exist | file check | `ls docs/src/json-ui/json-schema.md docs/src/json-ui/expressions.md` | ❌ Wave 0 |
| FIELD-01 | pagamenti.json passes catalog validation | unit | `cargo test -p ferro-json-ui pagamenti` | ❌ Wave 0 |
| FIELD-01 | render_file compiles and renders HTML | unit | `cargo test -p framework render_file` | ❌ Wave 0 |

### Wave 0 Gaps
- [ ] `docs/src/json-ui/expressions.md` — new page (DOC-02)
- [ ] `docs/src/json-ui/json-schema.md` — new page (DOC-02)
- [ ] `app/src/views/pagamenti.json` — spec file (FIELD-01)
- [ ] `app/src/controllers/pagamenti.rs` — handler (FIELD-01)
- [ ] `JsonUi::render_file` in `framework/src/json_ui/mod.rs` — missing from Phase 119 (FIELD-01 blocker)

## Security Domain

Phase 121 is documentation and a demo addition. No new attack surface.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | No | Docs only; render_file inherits existing load_cached + validate pipeline |
| V2 Authentication | No | — |

## Sources

### Primary (HIGH confidence)
- `v12.0/json-ui-v2:ferro-json-ui/src/spec.rs` — Spec, Element struct shapes; SCHEMA_VERSION = "ferro-json-ui/v2"
- `v12.0/json-ui-v2:ferro-json-ui/src/expression.rs` — EXPR_DATA_KEY = "$data", EXPR_TEMPLATE_KEY = "$template"; scope, single-pass rule, infallible semantics
- `v12.0/json-ui-v2:ferro-json-ui/src/loader.rs` — load_cached signature, LoadError variants; confirms render_file is NOT in loader
- `v12.0/json-ui-v2:framework/src/json_ui/mod.rs` — confirmed render_file absent; render, render_with_config, render_json present
- `v12.0/json-ui-v2:ferro-cli/src/commands/json_ui_schema.rs` — ferro json-ui:schema CLI: shells out to `cargo run -- json-ui:schema`
- `v12.0/json-ui-v2:docs/src/json-ui/getting-started.md` — confirmed v1 content (JsonUiView, ComponentNode)
- `v12.0/json-ui-v2:docs/src/json-ui/components.md` — confirmed v1 content (1427 lines)
- `v12.0/json-ui-v2:.planning/phases/121-documentation-and-field-test/121-CONTEXT.md` — locked decisions D-01..D-06
- `v12.0/json-ui-v2:.planning/phases/119-page-loader/119-CONTEXT.md` — D-05 specifying render_file; confirms it was planned but not implemented

### Secondary (MEDIUM confidence)
- `v12.0/json-ui-v2:ferro-json-ui/src/catalog.rs` (API shape via grep) — global_catalog(), json_schema(), validate()
- `v12.0/json-ui-v2:.planning/phases/120-cli-and-mcp-updates/120-PATTERNS.md` — confirmed v2 code templates using render_file

### Tertiary (LOW confidence)
- `Config::is_production()` method — cited in 119-CONTEXT D-05 but not read directly [ASSUMED]

## Metadata

**Confidence breakdown:**
- v2 spec format: HIGH — read spec.rs directly
- Expression semantics: HIGH — read expression.rs directly
- render_file missing: HIGH — read framework/src/json_ui/mod.rs, confirmed absent
- Docs state (all v1): HIGH — read getting-started.md and components.md on v12.0 branch
- CLI schema command: HIGH — read json_ui_schema.rs directly
- Config::is_production() name: LOW — assumed from 119-CONTEXT.md reference

**Research date:** 2026-04-21
**Valid until:** 2026-05-21 (stable branch; valid while v12.0/json-ui-v2 is the active branch)
