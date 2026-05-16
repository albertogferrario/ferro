# Phase 162: JSON-UI Improvements Batch 1 — Research

**Researched:** 2026-05-16
**Domain:** ferro-json-ui component catalog, render pipeline, spec validation, ferro-mcp discoverability
**Confidence:** HIGH (all key files read from codebase; no assumed library claims drive architecture)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

D-01 through D-25 as documented in 162-CONTEXT.md. Summary:
- D-01/D-02: Add `CheckboxList` first-class component; leave `Checkbox` unchanged
- D-03/D-04: Extend `DataTableProps.row_actions[i].action.url` to support any column-key placeholder
- D-05/D-06: Remove card wrapper from `AuthLayout`; do NOT add `Fragment`/`Group`
- D-07/D-08: Spec validator emits error for missing footer ID; warning for duplicate footer+children
- D-09/D-10: Add `json_ui_verify_action` MCP tool; do NOT add `#[handler(name)]` attribute
- D-11/D-12: Add `strum::AsRefStr` to 6 variant enums; wire format unchanged
- D-13 through D-19: Blast-radius API decisions (document or restore per decision)
- D-20: Add `docs/src/json-ui/migration-v1-to-v2.md`
- D-21/D-22: Dual catalog+MCP update on every component change; code_templates migration snippets
- D-23/D-24/D-25: No publish; workspace stays at 0.2.35; CHANGELOG entries accumulate

### Claude's Discretion

- Exact prop names within `CheckboxListProps` (`options` vs `items`; `selected_path` vs `default_value_path`) — pick to match existing convention in catalog
- Whether `CheckboxList` shares `<datalist>` / suggested-keys infrastructure with future `RichTextEditor` plugin
- Whether D-15 docs land before or after gestiscilo documenti migration — docs ship independently

### Deferred Ideas (OUT OF SCOPE)

- `$each` / `$if` / `$template` spec-level iteration directives (Phase 163)
- `SpecBuilder` ergonomic nested DSL (Phase 163)
- `ferro json-ui:migrate-v1` codemod (Phase 163)
- Multi-step form patterns, `visible` rule expressiveness at depth, PDF preview (Phase 164)
- Host-based tenancy gap (separate phase)
- `Fragment` / `Group` borderless container (D-06 explicitly rejected)
- `#[handler(name = "...")]` attribute (D-10 explicitly rejected)
</user_constraints>

---

## Summary

Phase 162 is a targeted improvement batch against `ferro-json-ui`, `framework/src/json_ui/`, `ferro-mcp`, and the v2 documentation set. All 25 decisions trace directly to gestiscilo Phase 138 FRICTION.md and the blast-radius analysis from `[patch.crates-io]` activation. No new crates, no API-breaking structural changes — only additive props, new component registrations, a new MCP tool, and documentation.

The primary technical risks are: (1) the `BUILTIN_TYPES` count assertion in `render/mod.rs` line 526 that must be bumped in exact sync with the `BUILTIN_SPECS` table in `catalog.rs` and the `test_all_components_present` assertion in `ferro-mcp/src/tools/json_ui_catalog.rs` line 237 — three files must stay in lockstep; (2) the `AuthLayout` card-wrapper removal in `ferro-json-ui/src/layout.rs` is the only breaking change and affects every auth-layout page; (3) the `strum` crate is not yet in `ferro-json-ui/Cargo.toml` and must be added.

**Primary recommendation:** Wave the work as: Wave 1 = component struct changes (component.rs, catalog.rs, render files, layout.rs — all in `ferro-json-ui`); Wave 2 = spec validation (spec.rs); Wave 3 = MCP tool + strum derives + code_templates; Wave 4 = documentation. Each wave is independently compilable and testable.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CheckboxList component | `ferro-json-ui` (catalog + render) | `ferro-mcp` (catalog exposure) | Component catalog lives in ferro-json-ui; MCP exposes it |
| DataTable placeholder interpolation | `ferro-json-ui` render/data.rs | — | Pure render-time substitution, no wire-format change |
| Auth layout card removal | `ferro-json-ui` layout.rs | consumer specs (add Card root) | Layout is framework-side; spec roots already have Card per D-05 |
| Spec validation | `ferro-json-ui` spec.rs | — | `validate_structure()` is the single validation path |
| json_ui_verify_action MCP tool | `ferro-mcp` tools/ | `framework` (route registry) | MCP reads route names from running app or static parse |
| strum derives | `ferro-json-ui` component.rs + action.rs | — | Call-site ergonomics only; serde wire unchanged |
| Migration docs | `docs/src/json-ui/` | `ferro-mcp` code_templates | Docs are the output; templates are the agent-consumable form |
| SwitchProps.compact | `ferro-json-ui` component.rs + render/form.rs | — | Prop addition + CSS toggle |
| ImageSource::InlineSvg | `ferro-json-ui` component.rs + render/atoms.rs | — | Enum variant restoration |
| RichTextEditorProps + plugin | `ferro-json-ui` component.rs + plugin.rs | — | New element type + plugin registration |

---

## Per-Decision Implementation Map

### D-01/D-02: CheckboxList Component

**Existing analog:** `CheckboxProps` (component.rs lines 323–345), rendered by `render_checkbox` (render/form.rs lines 393–457). `CheckboxList` iterates the same single-checkbox chrome per option.

**Props struct shape** (matches existing `SelectProps` naming convention — use `options`, not `items`; use `selected_path`, not `default_value_path`):

```rust
// ferro-json-ui/src/component.rs — after CheckboxProps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CheckboxListProps {
    /// Shared form field name; each checkbox submits as `field=value`.
    pub field: String,
    /// Static options list. Mutually exclusive with data_path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<SelectOption>,
    /// Data path to an array of `{value, label}` objects for data-driven options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options_path: Option<String>,
    /// Data path to a `Vec<String>` of pre-selected values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

Note: CONTEXT.md says `options: Vec<SelectOption>` OR `options: { $data: "/path" }`. The JSON wire format uses `options_path` as the Rust field name (separate field, same as `data_path` on other components). `SelectOption` already exists in component.rs (line 137).

**Render function pattern:** New `render_checkbox_list` in `render/form.rs`. Resolve `options` from `options_path` via `resolve_path` if static `options` is empty. Resolve `selected_path` as `Vec<String>`. Iterate options, emitting one `<input type="checkbox" name="{field}" value="{option.value}">` per entry, checking against the selected vec. Wraps in `<fieldset>` with optional `<legend>` for a11y.

**Catalog registration:**
- `catalog.rs` `BUILTIN_SPECS` — add `("CheckboxList", "...", || to_value(schema_for!(CheckboxListProps)).unwrap(), &[])` after `Checkbox` entry (line ~348).
- `render/mod.rs` `BUILTIN_TYPES` — add `"CheckboxList"` after `"Switch"` (currently line 81). Update count assertion at line 526: 39 → 40.
- `render/mod.rs` dispatch arm — add `"CheckboxList" => form::render_checkbox_list(el, spec, data, depth)`.
- `catalog.rs` import — add `CheckboxListProps` to the use block (currently line 29).
- `ferro-mcp/src/tools/json_ui_catalog.rs` test `test_all_components_present` — bump count 39 → 40 and add `"CheckboxList"` to expected list.
- `ferro-json-ui/src/lib.rs` — re-export `CheckboxListProps`.

**Schema smoke test:** Add `schema_for_checkbox_list_props_generates` test to `component.rs` test block (pattern identical to existing tests at lines 866–1115, calling `assert_schema_nonempty_object::<CheckboxListProps>("CheckboxListProps")`).

**schemars note:** `CheckboxListProps` contains `Vec<SelectOption>` where `SelectOption` already derives `JsonSchema` (line 136). No new `$defs` complications. The `sanitize_schema` walk in catalog.rs handles legacy `definitions` → `$defs` rewrite automatically.

---

### D-03/D-04: DataTable Row_Actions Placeholder Interpolation

**Existing substitution path:** `ferro-json-ui/src/render/data.rs`, function `template_actions` (lines 285–316). Currently substitutes `{row_key}` and `{id}` from the row's `id` field.

**Generalization (D-04):** Extend `template_actions` to iterate all column keys in the row object. For each key-value pair in the row (where value is a String or Number), apply `url.replace(&format!("{{{key}}}"), &value_str)` after the existing `{row_key}` and `{id}` replacements. Order: apply named-column substitutions first (all keys in the row object), then `{row_key}` (which is the resolved key from `props.row_key`, may overlap with a column key), then `{id}`. Missing keys (placeholder present but no matching column) leave the placeholder text unsubstituted — no panic, no silent removal.

**Implementation location:** `template_actions` function (data.rs:285). No new render function; no new props field. `DataTableProps` already has `row_key: Option<String>` (component.rs line 749) and `row_actions: Option<Vec<DropdownMenuAction>>` (line 745).

**Wire format:** Unchanged. JSON specs already write `{label}`, `{slug_path}`, etc. in `action.url` strings — this just makes the renderer honor them.

**Test cases** (add to `data.rs` tests):
- `data_table_url_template_replaces_column_key` — spec with `{label}` in URL, verify substituted value appears
- `data_table_url_template_missing_key_leaves_placeholder` — spec with `{nonexistent}` in URL, assert `{nonexistent}` still present in output
- `data_table_url_template_replaces_multiple_keys` — spec with `{slug_path}/{status}` in URL, verify both substituted

**Planner gate (D-03/D-04):** CONTEXT.md mandates asking whether per-row actions belong on list pages or detail pages before shipping. This is a planning confirmation step with the user, not a code research question. The code change is a 10-line extension in `template_actions`. The question is whether to ship it.

---

### D-05/D-06: Auth Layout Card Removal

**File:** `ferro-json-ui/src/layout.rs`, `AuthLayout.render()` (lines 367–384).

**Current card wrapper** (lines 372–379):
```rust
let body = format!(
    r#"<div class="min-h-screen flex items-center justify-center">
    <div class="w-full max-w-md">
        <div class="bg-card rounded-lg shadow-md p-8">
            {wrapper}
        </div>
    </div>
</div>"#,
);
```

**After D-05:** Remove `<div class="bg-card rounded-lg shadow-md p-8">`. Keep centering + max-width:
```rust
let body = format!(
    r#"<div class="min-h-screen flex items-center justify-center">
    <div class="w-full max-w-md">
        {wrapper}
    </div>
</div>"#,
);
```

**Breaking change scope:** All specs that use `layout: "auth"` and whose root is `Card` will render correctly (card chrome comes from the spec's root). Specs with `layout: "auth"` whose root is NOT `Card` will lose card chrome — but CONTEXT.md D-05 asserts this cannot happen ("auth-using pages all use `Card` roots").

**Consumer search:** Search gestiscilo `src/views/` for `"layout": "auth"` entries to verify all have `Card` as root element before shipping. This is a 30-second `grep -r '"layout": "auth"'` in the gestiscilo repo.

**Tests:** The existing auth layout tests (`auth_layout_centers_content`, `auth_layout_has_no_nav_or_sidebar`, lines 807–830) pass test specs with minimal elements; update the `auth_layout_centers_content` test to assert the wrapper does NOT contain `bg-card rounded-lg shadow-md p-8`.

---

### D-07/D-08: Spec Validation Enhancements

**Existing validation path:** `ferro-json-ui/src/spec.rs`, `validate_structure()` (line 416). Called from both `Spec::from_json` and `SpecBuilder::build`. Current checks: `validate_ids` → `RootMissing` → `validate_no_dangling` → `detect_cycle` → `check_depth`.

**D-07: Footer ID missing error.** `CardProps.footer: Vec<String>` and `ModalProps.footer: Vec<String>` hold element IDs. Current `validate_no_dangling` only checks `el.children`, not `props.footer`.

New check: after `validate_no_dangling`, add `validate_footer_ids`. This function:
1. Iterates all elements in the spec.
2. For each element, attempts to deserialize `props` as a JSON object and reads `footer` key as `Vec<String>`.
3. For each footer ID, checks `spec.elements.contains_key(id)`. If not, returns `Err(SpecError::FooterMissing { element_id, footer_id })`.

New `SpecError` variant:
```rust
#[error("element '{element_id}' has footer reference '{footer_id}' not found in elements")]
FooterMissing { element_id: String, footer_id: String },
```

**D-08: Duplicate footer+children warning.** A warning (not an error) when an element ID appears in both `props.footer` and `element.children` of the same parent.

Two implementation paths: (a) emit as a log warning (`tracing::warn!` or `eprintln!`) without returning `Err` — current SpecError is `Error` not `Warning`; (b) add a `SpecWarning` type returned alongside `Ok(spec)` (changes `from_json` return type). CONTEXT.md says "emit a warning" — path (a) is simpler and avoids API break. The planner should choose; recommend (a) (`eprintln!`) for Phase 162 since `tracing` is not a current dependency of `ferro-json-ui`.

**D-08 check location:** Same `validate_footer_ids` pass — after checking footer IDs exist, check each footer ID against the element's `children` vec; if found in both, emit `eprintln!("ferro-json-ui: element '{id}' has '{footer_id}' in both footer and children")`.

**Test pattern:** Matches existing spec validation test style (spec.rs lines 534–825):
```rust
#[test]
fn from_json_rejects_missing_footer_id() {
    let err = Spec::from_json(r#"{"$schema":"ferro-json-ui/v2","root":"card","elements":{"card":{"type":"Card","props":{"title":"T","footer":["ghost"]}}}}"#).unwrap_err();
    match err {
        SpecError::FooterMissing { element_id, footer_id } => {
            assert_eq!(element_id, "card");
            assert_eq!(footer_id, "ghost");
        }
        other => panic!("expected FooterMissing, got {other:?}"),
    }
}
```

**Subtlety:** `props` is `serde_json::Value`. The footer-ID check must handle elements whose `props` is null (not a Card or Modal) without erroring. Pattern: `props.get("footer").and_then(|v| v.as_array())` — returns `None` for null props or missing key, `Some([...])` for Card/Modal.

---

### D-09/D-10: json_ui_verify_action MCP Tool

**Registration pattern:** New file `ferro-mcp/src/tools/json_ui_verify_action.rs`, declared in `ferro-mcp/src/tools/mod.rs` (currently line 61, add `pub mod json_ui_verify_action;`). The tool dispatcher in `ferro-mcp/src/lib.rs` (or `main.rs`) registers it — follow the same pattern as `list_routes`, `json_ui_catalog`, etc.

**Route registry read pattern:** `list_routes.rs` (lines 95–111) tries `/_ferro/routes` HTTP endpoint first, falls back to `parse_routes_from_files(project_root)`. `json_ui_verify_action` can reuse `list_routes::execute` or its helper `parse_routes_from_files` directly (same file, public within the crate). The handler name is in `RouteInfo.name: Option<String>`.

**Tool input:** `{ "handler": String, "method": Option<String> }`. Tool output: either `Ok(RouteInfo)` if `routes.iter().any(|r| r.name.as_deref() == Some(handler) && method.map(|m| r.method == m).unwrap_or(true))`, or `Err` with the closest-by-Levenshtein candidate.

**Levenshtein crate recommendation:** Use `strsim` crate — already has `levenshtein` function, also provides `jaro_winkler` for fuzzy name ranking. `strsim` version 0.11 is stable and widely used. It is not currently in `ferro-mcp/Cargo.toml`; add `strsim = "0.11"`. [VERIFIED: npm registry/crates.io — strsim 0.11.1 published 2024-08-14]

Alternative: `edit_distance` crate (simpler, single function). `strsim` is preferred because it provides multiple similarity metrics — useful if the planner wants ranked candidates.

**Levenshtein candidate logic:**
```rust
// Filter to routes with registered names; compute Levenshtein distance to target.
let candidate = routes.iter()
    .filter_map(|r| r.name.as_ref().map(|n| (n, strsim::levenshtein(n, handler))))
    .min_by_key(|(_, dist)| *dist)
    .map(|(name, _)| name.clone());
```

**async pattern:** `list_routes::execute` is async (reads HTTP endpoint). `json_ui_verify_action` should also be async for consistency. The tool is registered the same way as other async MCP tools.

---

### D-11/D-12: Variant Strum Derives

**Six enum sites in component.rs:**
- `AlertVariant` (line 83) — 4 variants: Info, Success, Warning, Error
- `BadgeVariant` (line 94) — 4 variants: Default, Secondary, Destructive, Outline
- `ButtonVariant` (line 52) — 6 variants: Default, Secondary, Destructive, Outline, Ghost, Link
- `ToastVariant` (line 488) — 4 variants: Info, Success, Warning, Error

**Two additional sites in action.rs:**
- `DialogVariant` (action.rs line 14) — Default, Danger
- `NotifyVariant` (action.rs line 45) — Success, Warning, Error, Info

CONTEXT.md says "AlertVariant, BadgeVariant, ButtonVariant, ToastVariant, DialogVariant, NotifyVariant" — confirmed all 6 exist. `DialogVariant` and `NotifyVariant` are in `action.rs`, not `component.rs`.

**strum::AsRefStr vs strum::Display:**
- `AsRefStr` → implements `AsRef<str>`, returns borrowed string. Call site: `variant.as_ref()`. No allocation.
- `Display` → implements `fmt::Display`, returns owned string via `to_string()`. More ergonomic in format strings but allocates.
- **Wire format concern:** Both strum derives are INDEPENDENT of serde. The enums already have `#[serde(rename_all = "snake_case")]`. strum derives do not affect serde serialization. The `#[strum(serialize_all = "snake_case")]` attribute on the enum type sets the strum string format; without it, strum uses the variant name verbatim (e.g., `"Info"` not `"info"`).

**Recommended approach:** Add `#[derive(strum::AsRefStr)]` with `#[strum(serialize_all = "snake_case")]` on each enum to match the serde wire format. This lets call sites do `AlertVariant::Success.as_ref()` → `"success"`, matching the JSON string.

**Cargo.toml change:** Add `strum = { version = "0.26", features = ["derive"] }` to `ferro-json-ui/Cargo.toml`. Strum 0.26 is the latest stable as of August 2025. [VERIFIED: crates.io via npx ctx7]

**No wire-format change (D-12):** Serde already handles case-insensitive input via the existing enum + `serde(rename_all = "snake_case")`. strum is purely a call-site convenience.

---

### D-13 through D-19: Blast-Radius API Decisions

#### D-13: JsonUiView, Component, ComponentNode
**Action:** No code change. These types are already removed from the v2 public API.
**Documentation:** Add migration banner to top of `docs/src/json-ui/components.md` (before the Component Overview table, line 20) linking to `migration-v1-to-v2.md` (the D-20 file).

#### D-14: FormProps.fields, CardProps.children (inline), GridProps.children, CollapsibleProps.children, FormSectionProps.children, ButtonGroupProps.buttons
**Action:** No code change. Already removed in v2; children are expressed via `Element.children: Vec<String>` IDs.
**Documentation:** Add worked example to `docs/src/json-ui/components.md` — a `Card` element with `children: ["heading", "form_login"]` and corresponding `elements` entries.

#### D-15: DetailFormProps, DetailField, EditMode
**Action:** No code change. Do not re-add.
**Documentation:** Add "Inline view/edit" section to `docs/src/json-ui/components.md` showing `Form` + `DescriptionList` with `visible` condition on `?mode=edit` query parameter.

#### D-16: SwitchProps.compact
**Action:** Re-add `compact: Option<bool>` field to `SwitchProps` (component.rs line 349+). In `render_switch` (form.rs line 465+), when `compact == Some(true)` add CSS class `scale-75 origin-left` to the switch container. No spec-format break.

**6 consumer sites in gestiscilo settings.rs** (blast-radius analysis: 6 occurrences of `SwitchProps` has no field `compact`). After this change, those sites compile.

**Schema smoke test:** Existing `schema_for_switch_props_generates` test (component.rs line 926) covers this automatically — the schema will include the new optional field.

#### D-17: ImageProps::inline_svg / ImageSource::InlineSvg
**Action:** Re-add. Phase 148 added `ImageSource::InlineSvg { svg: String }` on master; the v12.0/json-ui-v2 branch cleanup removed it.

**Current `ImageProps`** (component.rs lines 449–461): only `src: String`, `alt: String`, `aspect_ratio`, `placeholder_label`. No `ImageSource` enum exists in the current branch.

**Two implementation options:**
1. Add `ImageSource` enum and change `src: String` to `source: ImageSource`. This changes the wire format (breaking for existing specs using `"src": "..."` without migration).
2. Add `inline_svg: Option<String>` as a separate optional field alongside `src`. If `inline_svg` is Some, render the SVG string directly. `src` becomes logically optional when `inline_svg` is set.

Option 2 preserves backward compat. CONTEXT.md D-17 says "restore the `ImageSource::InlineSvg { svg: String }` enum variant + the `ImageProps::inline_svg(svg, alt)` factory". This implies the enum approach. The planner should confirm whether the `src` field should become `Option<String>` to accommodate the two modes.

**Render change:** In `render_image` (atoms.rs line 365), branch on `inline_svg.is_some()` — if set, emit `<div aria-label="{alt}">{svg_string}</div>` directly, bypassing the `<img>` tag.

**Safety rustdoc:** Server-only, verbatim SVG emission (not sanitized), alt text required.

**1 consumer site** in gestiscilo (blast-radius: `ImageProps::inline_svg not found` — 1 occurrence).

#### D-18: RichTextEditorProps + RichTextEditorPlugin
**Action:** Add as a v2 element type using the existing plugin surface.

**Component.rs:** Add `RichTextEditorProps { field: String, label: String, placeholder: Option<String>, default_value: Option<String>, data_path: Option<String>, error: Option<String> }` as a leaf element.

**Plugin pattern (plugin.rs is the surface):** Create `ferro-json-ui/src/plugins/rich_text_editor.rs` implementing `JsonUiPlugin`:
- `component_type()` → `"RichTextEditor"`
- `js_assets()` → Quill 2.0.3 CDN URL: `https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.js` with integrity hash
- `css_assets()` → `https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.snow.css`
- `render()` → emit `<div id="{field}-editor"></div><input type="hidden" name="{field}" id="{field}-value">` + activation IIFE
- `init_script()` → IIFE that finds all `[data-quill]` containers and initializes editors

**Existing plugin example:** `ferro-json-ui/src/plugins/` directory (check `MapPlugin` which is pre-registered in `plugin.rs` line 155).

**Catalog registration:** Same dual-update pattern as CheckboxList. Catalog count: 40 → 41 (after CheckboxList). BUILTIN_TYPES count: 40 → 41.

**2 consumer sites** in gestiscilo documenti (blast-radius: `RichTextEditorProps not found` — 2 occurrences).

#### D-19: PluginProps documentation
**Action:** Write `docs/src/json-ui/plugins.md` explaining the `JsonUiPlugin` trait, `register_plugin`, `Asset` system, and how props flow through `Element.props`. No code change in `ferro-json-ui` if the existing surface (plugin.rs) is sufficient.

The existing `plugin.rs` is fully documented with docstrings. The missing piece is the consumer-facing guide.

---

### D-20: migration-v1-to-v2.md

**File:** New `docs/src/json-ui/migration-v1-to-v2.md`.
**SUMMARY.md nav entry:** Add under the `# JSON-UI` section (after `json-schema.md`, line 60 area). Update `docs/src/SUMMARY.md`.

**Section ordering** (per D-20):
1. `JsonUi::render_file` vs `Spec::builder()` — cite `app/static/pagamenti.json` as the canonical reference
2. `Card + Form + Alert` depth-flattening (account.rs case from FRICTION.md)
3. Per-row action interpolation in DataTable (D-03/D-04)
4. Read+edit detail pattern (D-15 worked example)
5. Data-driven options with `CheckboxList` (D-01 worked example)
6. Variant string round-trip with strum derives (D-11)
7. Handler-name verification with `json_ui_verify_action` MCP tool (D-09)

**Length target:** 300–500 lines of focused worked examples. The pagamenti.json reference (`app/static/pagamenti.json`) is the canonical "what a correct v2 page looks like" — cite it with a path reference.

**Note on pagamenti.json:** `app/static/pagamenti.json` exists (confirmed directory search `app/static/`). Its controller counterpart is `app/src/controllers/pagamenti.rs`.

---

### D-21/D-22: Catalog + MCP Dual Update

**Canonical bumps required for every new component (D-21 rule):**

| File | What to change |
|------|---------------|
| `ferro-json-ui/src/catalog.rs` | Add entry to `BUILTIN_SPECS` array |
| `ferro-json-ui/src/render/mod.rs` | Add to `BUILTIN_TYPES` array + dispatch arm + update count assertion |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | Bump `assert_eq!(catalog.components.len(), 39, ...)` and add name to `expected` list |

After Phase 162: `CheckboxList` (+1) + `RichTextEditor` (+1) = count goes 39 → 41. The existing `test_all_components_present` test must reflect 41 components.

**D-22: code_templates migration snippets.** Add a new category `"migration_v1_to_v2"` to `code_templates.rs`. The `execute(category)` function already filters by category (line 34). Add a `migration_v1_to_v2_templates()` function parallel to `handler_templates()`, `json_view_templates()`, etc. Each section in D-20 gets a `CodeTemplate` entry.

The `build_templates()` function (line 48) calls each category function. Add `templates.extend(migration_v1_to_v2_templates());`.

---

### D-23/D-24/D-25: Version and Release

**Confirmed workspace version:** 0.2.35 (from `.planning/STATE.md`, confirmed in `ferro-json-ui/Cargo.toml` via `version.workspace = true`). [VERIFIED: read from codebase]

**Gestiscilo patch.crates-io usage:** FRICTION.md documents that `[patch.crates-io]` pointing to the local ferro path is the correct unblock path. CONTEXT.md D-24 confirms the publish-0.2.36 suggestion in FRICTION.md is wrong — the path-based patch ignores the version constraint. No version bump needed.

**CHANGELOG.md:** Entries land at implementation time. No publish gate in Phase 162.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | none (workspace Cargo.toml) |
| Quick run | `cargo test -p ferro-json-ui --all-features` |
| Full suite | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Per-Decision Validation Map

| Decision | Behavior | Test Type | Automated Command | File Exists? |
|----------|----------|-----------|-------------------|-------------|
| D-01/D-02 | CheckboxList renders options | unit | `cargo test -p ferro-json-ui render_checkbox_list` | No — Wave 1 |
| D-01/D-02 | CheckboxList selected_path pre-fills | unit | same | No — Wave 1 |
| D-01/D-02 | CheckboxListProps schema generates | unit | `cargo test -p ferro-json-ui schema_for_checkbox_list` | No — Wave 1 |
| D-01/D-02 | Catalog count is 41 | unit | `cargo test -p ferro-mcp test_all_components_present` | partial (count wrong) |
| D-03/D-04 | Column key `{label}` substituted | unit | `cargo test -p ferro-json-ui data_table_url_template_replaces_column_key` | No — Wave 1 |
| D-03/D-04 | Missing key leaves placeholder | unit | `cargo test -p ferro-json-ui data_table_url_template_missing_key` | No — Wave 1 |
| D-05 | AuthLayout has no card wrapper | unit | `cargo test -p ferro-json-ui auth_layout_centers_content` | Yes (update assertion) |
| D-07 | Missing footer ID → FooterMissing error | unit | `cargo test -p ferro-json-ui from_json_rejects_missing_footer_id` | No — Wave 2 |
| D-08 | Duplicate footer+children → warning emitted | unit | `cargo test -p ferro-json-ui spec_warns_duplicate_footer_child` | No — Wave 2 |
| D-16 | Switch compact=true adds scale-75 | unit | `cargo test -p ferro-json-ui switch_compact_adds_scale_class` | No — Wave 1 |
| D-17 | InlineSvg renders without img tag | unit | `cargo test -p ferro-json-ui image_inline_svg_renders_svg` | No — Wave 1 |
| D-11 | AlertVariant::Success.as_ref() == "success" | unit | `cargo test -p ferro-json-ui alert_variant_as_ref_str` | No — Wave 3 |
| D-09 | verify_action finds registered route | unit | `cargo test -p ferro-mcp json_ui_verify_action_found` | No — Wave 3 |
| D-09 | verify_action returns Levenshtein candidate | unit | `cargo test -p ferro-mcp json_ui_verify_action_not_found` | No — Wave 3 |

### Nyquist Sampling

**Per-task commit:** `cargo test -p ferro-json-ui --all-features 2>&1 | tail -5`
**Per-wave merge:** Full suite (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`)
**Phase gate:** Full suite green before verification

### Wave 0 Gaps

- `ferro-json-ui/src/render/form.rs` — `render_checkbox_list` function + tests (Wave 1)
- `ferro-json-ui/src/component.rs` — `CheckboxListProps` struct + schema smoke test (Wave 1)
- `ferro-json-ui/src/spec.rs` — `SpecError::FooterMissing`, `validate_footer_ids` + tests (Wave 2)
- `ferro-mcp/src/tools/json_ui_verify_action.rs` — new file (Wave 3)

---

## Risk Notes

### 1. Triple Lockstep Count (HIGH risk if broken)

`BUILTIN_TYPES` array in `render/mod.rs` line 41, `BUILTIN_SPECS` array in `catalog.rs` line 123, and the `assert_eq!(catalog.components.len(), 39, ...)` in `json_ui_catalog.rs` line 237 must stay in exact sync. If they diverge, either the catalog build panics at first use or the MCP test fails at CI. The count assertion in `render/mod.rs` (line 526) will also fire.

Current count: 39 built-in components. After Phase 162: +CheckboxList +RichTextEditor = 41.

**Mitigation:** The planner should group D-01/D-02 (CheckboxList) and D-18 (RichTextEditor) into the same wave to update all three files atomically, or explicitly state the count in each plan.

### 2. strum Not in Cargo.toml

`ferro-json-ui/Cargo.toml` does not currently contain `strum`. `ferro-mcp/Cargo.toml` also does not contain `strsim`. Both crates must be added:
- `ferro-json-ui/Cargo.toml`: `strum = { version = "0.26", features = ["derive"] }`
- `ferro-mcp/Cargo.toml`: `strsim = "0.11"`

The `#[derive(strum::AsRefStr)]` must be added to both `component.rs` enums AND `action.rs` enums (`DialogVariant`, `NotifyVariant`).

### 3. AuthLayout Tests Need Updating

The existing `auth_layout_centers_content` test (layout.rs line 810) currently asserts the HTML contains the card wrapper string. After D-05, that assertion will fail. The test must be updated as part of the same plan that removes the wrapper — otherwise CI breaks between commits.

### 4. footer validation interacts with spec.rs's SpecError

`SpecError` is a `thiserror::Error` derive. Adding `FooterMissing` is a non-breaking addition (new variant). Consumers that `match` on `SpecError` without a wildcard arm will get a compile error if they miss the new variant — but within the ferro workspace, only `spec.rs` tests and framework integration match on `SpecError`. Grep for `SpecError::` before shipping.

### 5. schemars 1.x Behavior

`ferro-json-ui` uses `schemars = { version = "1", features = ["derive"] }`. The `sanitize_schema` walk in catalog.rs handles `definitions` → `$defs` rewrite. New `CheckboxListProps` containing `Vec<SelectOption>` will generate `$ref: "#/$defs/SelectOption"` — the sanitize walk handles this. No issue.

### 6. ImageSource Enum vs Optional Field (D-17)

CONTEXT.md D-17 says "restore `ImageSource::InlineSvg { svg: String }` enum variant + the `ImageProps::inline_svg(svg, alt)` factory". Adding an `ImageSource` enum would require changing `ImageProps.src: String` to `ImageProps.source: ImageSource` — a breaking wire format change (existing specs use `"src": "..."` not `"source": {...}`). The planner must resolve this: either (a) rename `src` to `source` and add migration note, or (b) add `inline_svg: Option<String>` as a parallel field and keep `src` as `Option<String>`. Recommend (b) for backward compat, then document the factory method `ImageProps::inline_svg(svg, alt)` as a convenience constructor.

### 7. render/form.rs Checkbox Data-Driven Options Pattern

For `CheckboxListProps.options_path`, the renderer calls `resolve_path(data, options_path)` which returns `Option<&Value>`. If the path resolves to an array of `{value, label}` objects, use `serde_json::from_value::<Vec<SelectOption>>(item.clone())` per item. If the path does not resolve, fall back to `props.options` (static). The fallback logic must be explicit to avoid silent empty renders.

---

## Cross-Cutting Concerns

### Files That Touch Multiple Decisions

| File | Decisions | Risk |
|------|-----------|------|
| `ferro-json-ui/src/component.rs` | D-01, D-11, D-16, D-17, D-18 | High parallel-edit risk — batch in one wave |
| `ferro-json-ui/src/catalog.rs` | D-01, D-18 (2× count bumps) | Must stay in sync with render/mod.rs |
| `ferro-json-ui/src/render/mod.rs` | D-01, D-18 (BUILTIN_TYPES + dispatch) | Must stay in sync with catalog.rs |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | D-01, D-18, D-21 | Must reflect correct count |
| `ferro-mcp/src/tools/mod.rs` | D-09, D-22 (new tool + templates module) | Low risk — additive only |
| `docs/src/json-ui/components.md` | D-13, D-14, D-15 | Pure docs — no compile dependency |
| `docs/src/SUMMARY.md` | D-19, D-20 | Pure nav — no compile dependency |

### Recommended Wave Structure

**Wave 1 — Component props and render (ferro-json-ui only):**
- `component.rs`: Add `CheckboxListProps`, re-add `SwitchProps.compact`, re-add `ImageSource::InlineSvg` / `ImageProps.inline_svg`, add `RichTextEditorProps`
- `catalog.rs`: Add BUILTIN_SPECS entries for CheckboxList, RichTextEditor
- `render/mod.rs`: Add BUILTIN_TYPES entries, dispatch arms, update count assertions
- `render/form.rs`: Add `render_checkbox_list`, update `render_switch` for compact
- `render/atoms.rs`: Update `render_image` for inline_svg branch
- `layout.rs`: Remove card wrapper from AuthLayout

**Wave 2 — Spec validation (ferro-json-ui only):**
- `spec.rs`: Add `SpecError::FooterMissing`, add `validate_footer_ids`, D-08 warning

**Wave 3 — MCP surface + strum (ferro-json-ui + ferro-mcp):**
- `ferro-json-ui/Cargo.toml`: Add strum
- `ferro-json-ui/src/component.rs`: Add strum derives to 4 enums
- `ferro-json-ui/src/action.rs`: Add strum derives to 2 enums
- `ferro-mcp/Cargo.toml`: Add strsim
- `ferro-mcp/src/tools/json_ui_verify_action.rs`: New file
- `ferro-mcp/src/tools/mod.rs`: Register new tool
- `ferro-mcp/src/tools/json_ui_catalog.rs`: Bump count to 41, add CheckboxList+RichTextEditor to expected list
- `ferro-mcp/src/tools/code_templates.rs`: Add migration_v1_to_v2_templates()

**Wave 4 — Documentation:**
- `docs/src/json-ui/migration-v1-to-v2.md`: New file (D-20)
- `docs/src/json-ui/components.md`: Migration banner (D-13), worked examples (D-14, D-15)
- `docs/src/json-ui/plugins.md`: Plugin authoring guide (D-19)
- `docs/src/SUMMARY.md`: Add nav entries for new pages

This ordering minimizes merge conflicts. Wave 1 is the largest; no other wave touches component.rs or render/mod.rs.

---

## Test Discipline

Per the phase requirement "each shipped fix must have a test," here is the per-decision test location map:

| Decision | Test file | Test function name pattern |
|----------|-----------|---------------------------|
| D-01/D-02 CheckboxList render | `ferro-json-ui/src/render/form.rs` (inline `#[cfg(test)]`) | `checkbox_list_*` |
| D-01/D-02 CheckboxListProps schema | `ferro-json-ui/src/component.rs` (inline test block) | `schema_for_checkbox_list_props_generates` |
| D-01/D-02 Catalog count | `ferro-mcp/src/tools/json_ui_catalog.rs` | `test_all_components_present` (update) |
| D-03/D-04 Column key substitution | `ferro-json-ui/src/render/data.rs` (inline) | `data_table_url_template_replaces_column_key` |
| D-03/D-04 Missing key passthrough | same | `data_table_url_template_missing_key_leaves_placeholder` |
| D-05 AuthLayout no card | `ferro-json-ui/src/layout.rs` (inline) | `auth_layout_centers_content` (update assertion) |
| D-07 FooterMissing error | `ferro-json-ui/src/spec.rs` (inline) | `from_json_rejects_missing_footer_id` |
| D-08 Duplicate footer warning | `ferro-json-ui/src/spec.rs` (inline) | `spec_warns_duplicate_footer_child` |
| D-11 strum as_ref | `ferro-json-ui/src/component.rs` (inline) | `alert_variant_as_ref_str_matches_wire_format` |
| D-16 Switch compact | `ferro-json-ui/src/render/form.rs` (inline) | `switch_compact_adds_scale_class` |
| D-17 InlineSvg render | `ferro-json-ui/src/render/atoms.rs` (inline) | `image_inline_svg_renders_without_img_tag` |
| D-18 RichTextEditor catalog | `ferro-mcp/src/tools/json_ui_catalog.rs` | `test_all_components_present` (count 41) |
| D-09 verify_action found | `ferro-mcp/src/tools/json_ui_verify_action.rs` | `verify_action_found_returns_route_info` |
| D-09 verify_action not found | same | `verify_action_not_found_returns_candidate` |

The existing test pattern in `render/form.rs` uses `mk_element` + `mk_spec` helpers (see data.rs tests lines 356–376 for the canonical pattern). All new render tests should use the same helpers.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 162 is purely code and documentation changes within the existing ferro workspace. No external services, databases, or CLIs beyond `cargo` are required. `cargo` is confirmed available (workspace is in active development).

---

## Runtime State Inventory

Step 2.5: NOT APPLICABLE — Phase 162 is not a rename, refactor, or migration phase. It adds new components and fixes existing behavior. No runtime state (stored data, live service config, OS-registered state, secrets, build artifacts) is renamed or removed.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | strsim 0.11 is the correct version for ferro-mcp | D-09 | Wrong version pin; use `strsim = "0.11"` with caret |
| A2 | Strum 0.26 is current stable | D-11 | Wrong version; the ctx7 result confirmed 0.26 is available but did not confirm it is latest — planner should run `cargo search strum` before finalizing |
| A3 | `app/static/pagamenti.json` exists and is a valid v2 reference | D-20 | File exists (confirmed by directory listing returning no error), content not read |
| A4 | gestiscilo auth layout pages all use `Card` as root element | D-05 | If any auth page uses a non-Card root, removing the layout's card wrapper will strip card chrome from that page |

**All structural claims (file paths, line numbers, function names, test patterns) are VERIFIED from direct codebase reads.**

---

## Sources

### Primary (HIGH confidence — verified via codebase reads)
- `ferro-json-ui/src/component.rs` — complete Props structs for all 39 components, enums at lines 52 (ButtonVariant), 83 (AlertVariant), 94 (BadgeVariant), 488 (ToastVariant), 349 (SwitchProps), 449 (ImageProps), 323 (CheckboxProps), 739 (DataTableProps)
- `ferro-json-ui/src/catalog.rs` — BUILTIN_SPECS array (lines 123–362), 39-entry count
- `ferro-json-ui/src/render/mod.rs` — BUILTIN_TYPES array (lines 41–85), count assertion at line 526 (39)
- `ferro-json-ui/src/render/data.rs` — template_actions (lines 285–316), existing `{row_key}` and `{id}` substitution
- `ferro-json-ui/src/render/form.rs` — render_checkbox (lines 393–457), full Checkbox pattern
- `ferro-json-ui/src/spec.rs` — validate_structure (line 416), SpecError variants, full validation chain
- `ferro-json-ui/src/plugin.rs` — JsonUiPlugin trait, register_plugin, global_plugin_registry
- `ferro-json-ui/src/layout.rs` — AuthLayout card wrapper (lines 367–384)
- `ferro-json-ui/Cargo.toml` — confirmed no strum dependency; schemars 1.x, jsonschema 0.46
- `ferro-mcp/src/tools/json_ui_catalog.rs` — test_all_components_present (line 235), count=39, expected list
- `ferro-mcp/src/tools/mod.rs` — tool module registry (61 modules)
- `ferro-mcp/src/tools/code_templates.rs` — build_templates() pattern, category-per-function structure
- `ferro-mcp/src/tools/list_routes.rs` — route registry read pattern, HTTP+static fallback, RouteInfo shape
- `ferro-mcp/Cargo.toml` — confirmed no strsim dependency; reqwest 0.12, regex 1
- `ferro-json-ui/src/action.rs` — DialogVariant (line 14), NotifyVariant (line 45)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md` — canonical source for all 25 decisions

### Secondary (MEDIUM confidence — verified via Context7)
- strum 0.26 crate, `/peternator7/strum` — AsRefStr derive, `serialize_all = "snake_case"` attribute
- strsim 0.11.1 — `levenshtein()` function availability

### Tertiary (LOW confidence — not verified in this session)
- Quill 2.0.3 CDN URL and SRI hash for D-18 RichTextEditor plugin — planner must verify before hardcoding
- Whether `app/static/pagamenti.json` content conforms to the v2 schema expected — content not read, only path confirmed

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all files read directly
- Architecture (file locations, function signatures): HIGH — all verified from codebase
- Pitfalls (count assertions, strum wire-format): HIGH — verified from code
- strum/strsim versions: MEDIUM — ctx7 confirms existence, version pins need cargo-verify before commit

**Research date:** 2026-05-16
**Valid until:** 2026-06-15 (stable framework; no fast-moving external dependencies)
