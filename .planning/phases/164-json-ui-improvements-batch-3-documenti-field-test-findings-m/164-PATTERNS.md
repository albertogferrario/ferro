# Phase 164: JSON-UI improvements batch 3 — Pattern Map

**Mapped:** 2026-05-17
**Files analyzed (created/modified):** 18 implementation files + 3 doc files + 2 phase-artefact files = 23 entries
**Analogs found:** 21 / 23 (2 entries are pure-doc/pure-artefact with no Rust analog)

This map is keyed by CONTEXT decision ID (D-01..D-19). For each new/modified file the planner gets: (a) the closest existing analog with exact file:line, (b) verbatim code excerpts to copy or model, (c) any cross-file invariants (count assertions, dispatch arms, schema regen sites). Per CONTEXT D-12..D-19 every excerpt is taken from `v12.0/json-ui-v2` HEAD (commit `ce44ac77`).

---

## File Classification

| Decision | New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|----------|-------------------|------|-----------|----------------|---------------|
| D-12 | `ferro-json-ui/src/spec.rs` (`Spec.title` field) | spec-type | static-to-runtime binding | `EachDirective` / `Visibility` enum at `spec.rs:96-100` + `visibility.rs:45` (untagged enum pattern) | role-match (no exact precedent for top-level Spec field binding) |
| D-12 | `framework/src/json_ui/mod.rs:89` (title extraction) | renderer-glue | request-response | `spec.title.as_deref().unwrap_or("Ferro")` at `framework/src/json_ui/mod.rs:89` (current literal-only path) | exact (modify in place) |
| D-13a | `ferro-json-ui/src/component.rs` (`KanbanBoardProps.data_path`) | props-struct | CRUD with data_path override | `DataTableProps.data_path: String` at `component.rs:838` | exact (same component family — data-driven container) |
| D-13a | `ferro-json-ui/src/render/containers.rs:292` (`render_kanban_board`) | renderer fn | data-path resolution | `render_data_table` at `render/data.rs:133-145` (`resolve_path(data, &props.data_path)` + `as_array().cloned()`) | exact (mirror call shape) |
| D-13b | `docs/src/json-ui/expressions.md` (worked `$each` example) | doc | reference example | existing `$each` directive doc in `expressions.md` (Phase 163 D-08) | partial (extend existing section) |
| D-14 | `ferro-json-ui/src/spec.rs:37` (`MAX_NESTING_DEPTH`) | const | static value | `pub const MAX_NESTING_DEPTH: usize = 3;` at `spec.rs:37` | exact (one-line bump) |
| D-14 | `ferro-json-ui/src/spec.rs:1704` (`nested_builder_flattens_two_levels` test) | test | unit | existing test at `spec.rs:1703-1725` | exact (rewrite same test for depth 5, add depth-6-rejected) |
| D-14 | `docs/src/json-ui/spec-construction.md` (depth constraint doc) | doc | reference | existing depth-3 mention in `spec-construction.md` | partial (update existing constant reference) |
| D-15 | `ferro-json-ui/src/component.rs:516` (`ImageProps.data_path`) | props-struct | CRUD with data_path override | `RichTextEditorProps.data_path: Option<String>` at `component.rs:285-286`; also `InputProps.data_path` at `component.rs:255-256`; also `SelectProps.data_path` at `component.rs:311-312` | exact (3 precedents in same file with identical `Option<String>` + `skip_serializing_if` shape) |
| D-15 | `ferro-json-ui/src/component.rs:456` (`DescriptionListProps.data_path`) | props-struct | CRUD with data_path override | same as above | exact |
| D-15 | `ferro-json-ui/src/render/atoms.rs:365` (`render_image`) | renderer fn | data-path resolution | `render_data_table` at `render/data.rs:144` uses `resolve_path(data, &props.data_path)`; `render_image` already takes `data` parameter (currently unused: `_data: &Value`) | role-match (sibling renderer already in same file; need to drop `_` prefix on `data` and add resolution) |
| D-15 | `ferro-json-ui/src/render/atoms.rs:563` (`render_description_list`) | renderer fn | data-path resolution | same as above | role-match |
| D-16 | `ferro-json-ui/src/loader.rs:138-143` (validation pipeline) | pipeline-glue | startup-vs-request validation | `load_cached` at `loader.rs:118-155` — current single-stage path | exact (modify in place) |
| D-16 | `framework/src/json_ui/mod.rs:48-54` (`JsonUi::resolve` callsite) | pipeline-glue | per-request | `resolve` at `framework/src/json_ui/mod.rs:48-54` + `resolve_with_errors` at line 202-205 | exact (add `Catalog::validate` call after `expand_directives`) |
| D-16 | `framework/tests/pipeline_order.rs` (NEW integration test) | test | end-to-end | `framework/src/json_ui/mod.rs` test module at line 696+ (`render_*` integration tests with `render_file`) | role-match (new file, model on existing render_with_errors tests) |
| D-17a | `ferro-json-ui/src/component.rs` (NEW `RawHtmlProps` struct) | props-struct | server-injected HTML island | `SkeletonProps` at `component.rs:568-576` (smallest minimal-prop struct in the file); also `ImageProps.inline_svg` at `component.rs:527-535` (verbatim-emission pattern + trust-boundary docstring) | role-match (no exact precedent — Skeleton for struct shape, inline_svg for trust-boundary doc) |
| D-17a | `ferro-json-ui/src/render/atoms.rs` (NEW `render_raw_html`) | renderer fn | verbatim HTML emission | `render_image` inline-SVG branch at `render/atoms.rs:373-379` (verbatim emission with `aria-label` wrapper, NO sanitization, intentional `// verbatim` comment) | exact (5-line verbatim emission pattern is already in this file) |
| D-17a | `ferro-json-ui/src/render/mod.rs:41-86` (BUILTIN_TYPES entry) | dispatch table | enum-like | `"DataTable"` entry at `render/mod.rs:85` and dispatch arm at `render/mod.rs:196` | exact (add `"RawHtml"` to list + arm) |
| D-17a | `ferro-json-ui/src/catalog.rs:123-369` (BUILTIN_SPECS entry) | dispatch table | enum-like | `"Image"` entry at `catalog.rs:167-172` (leaf with no slots) | exact (add `"RawHtml"` 5-tuple) |
| D-17a | `ferro-mcp/src/tools/json_ui_catalog.rs:289-340` (count + expected names) | test assertion | unit | existing count assertion at `json_ui_catalog.rs:289-290` + expected-names list at lines 296-337 | exact (bump 40→41, add "RawHtml") |
| D-17a | `ferro-json-ui/src/render/mod.rs:530` (BUILTIN_TYPES count test) | test assertion | unit | `assert_eq!(BUILTIN_TYPES.len(), 40);` at `render/mod.rs:530` | exact (bump to 41) |
| D-17a | `ferro-json-ui/src/catalog.rs:1052` (BUILTIN_SPECS count test) | test assertion | unit | `assert_eq!(BUILTIN_SPECS.len(), 40);` at `catalog.rs:1052` | exact (bump to 41) |
| D-18 | `ferro-json-ui/src/component.rs:153` (`CardProps.variant` + `CardVariant` enum) | props-struct + variant enum | enum-tagged variant | `ActionCardVariant` at `component.rs:888-896` + `ActionCardProps.variant` at `component.rs:909` | exact (identical 3-variant enum pattern with `#[default]`, `#[serde(rename_all = "snake_case")]`, `#[serde(default)]` on the field) |
| D-18 | `ferro-json-ui/src/render/containers.rs:53-55` (`render_card` variant branch) | renderer fn | match arm | `render_action_card` variant match in `render/atoms.rs` (search for `ActionCardVariant::` matches); also the existing hard-coded class string at `containers.rs:53-54` | role-match (existing hard-coded class is the body to replace with a match) |
| D-19/F2 | `ferro-cli/src/commands/json_ui_migrate_v1.rs:521-528` (HTTP method uppercase emission) | codemod helper | string-rewrite | `parse_action_expr` at `json_ui_migrate_v1.rs:515-535` — already emits `"POST"` / `"GET"` etc. uppercase | exact (verification-only; add regression test) |
| D-19/F2 | `ferro-cli/tests/json_ui_migrate_v1.rs` (NEW regression test) | test | unit | existing integration tests at `ferro-cli/tests/json_ui_migrate_v1.rs:63-95` | exact (model new test on `codemod_one_handler_emits_spec_and_rewrites_controller` shape) |
| D-19/F5 | `ferro-json-ui/src/visibility.rs:43-50` (custom `Deserialize` impl) | de-serializer | error reporting | `Visibility` untagged enum at `visibility.rs:43-50` (current source of the bad error) | exact (replace `#[derive(Deserialize)]` with hand-rolled impl) |
| D-19/F6 | `ferro-json-ui/src/component.rs:803` (`PageHeaderProps.actions`) | props-struct | lax deserializer | current field at `component.rs:803`: `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub actions: Vec<String>` — already accepts missing field and empty array; need to also accept empty string | role-match (custom `deserialize_with` is new pattern in this crate — see Pitfall 6 / RESEARCH lines 415-427) |
| D-04 | `ferro-mcp/src/tools/json_ui_validate_spec.rs` (NEW MCP tool) | tool | request-response | `ferro-mcp/src/tools/json_ui_inspect.rs:55-99` (`inspect_component`) and `json_ui_catalog.rs::execute` | role-match (same tool shape — `pub fn execute(...)` returning a Serialize struct) |
| D-04 | `ferro-mcp/src/tools/mod.rs:26-29` (module registration) | registration | declaration | existing `pub mod json_ui_catalog;` etc. at `tools/mod.rs:26-29` | exact (add `pub mod json_ui_validate_spec;`) |
| D-05 | `ferro-json-ui/src/spec.rs:749` (`validate_directives` audit) | validator | unit | existing `validate_directives` function at `spec.rs:749+` (Phase 163) | exact (audit only — extend if gap found) |
| D-01..D-03 | `.planning/phases/164-.../V1-DELETION-AUDIT.md` | phase artefact | doc | RESEARCH `Example 4` at line 532-555 ships a 16-row sample table | exact (use sample table verbatim) |
| D-06..D-07 | `docs/src/json-ui/plugins.md` (audit findings; only if gap found) | doc | reference | existing `plugins.md` from Phase 162 D-19 | partial (only modify if D-06 paper exercise surfaces a gap) |
| D-08..D-09 | `docs/src/json-ui/migration-v1-to-v2.md` (cheat sheet table) | doc | reference | RESEARCH cites Phase 162 D-20 as creator of this file | partial (prepend new top-of-page table) |
| D-08 | `docs/src/json-ui/components.md` (new sections for D-12..D-18) | doc | reference | existing components.md sections per Phase 116 / 162 | partial (extend existing per-component sections) |
| D-10..D-11 | `.planning/phases/164-.../COMPLETED.md` | phase artefact | doc | CONTEXT D-10 enumerates the 5 required sections | exact (template-driven) |

---

## Pattern Assignments

### D-12 — `Spec.title` accepts `String | {$data: ...}` binding

#### Modify: `ferro-json-ui/src/spec.rs` (Spec.title field at line 59-60)

**Analog (closest existing untagged-enum binding):** `Visibility` at `ferro-json-ui/src/visibility.rs:43-50`

**Untagged enum pattern (lines 43-50 of `visibility.rs`):**
```rust
/// Visibility rule with logical composition support.
///
/// Uses `#[serde(untagged)]` to support clean JSON:
/// - Simple: `{"path": "/data/users", "operator": "not_empty"}`
/// - Compound: `{"and": [...]}`
/// - Nested: `{"not": {"path": ..., "operator": ...}}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Visibility {
    And { and: Vec<Visibility> },
    Or { or: Vec<Visibility> },
    Not { not: Box<Visibility> },
    Condition(VisibilityCondition),
}
```

**Existing `Spec.title` field to replace** (`spec.rs:58-60`):
```rust
/// Optional document title (used by layouts to populate `<title>`).
#[serde(default, skip_serializing_if = "Option::is_none")]
pub title: Option<String>,
```

**Pattern to apply (per RESEARCH Pattern 1 + Pitfall 5):**
```rust
/// Bindable string field — either a literal or a runtime `$data` reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum TitleBinding {
    Literal(String),
    Binding(DataRef),
}

/// `{"$data": "/path"}` shape — the only expression form supported on
/// top-level Spec fields. Mirrors `expression::EXPR_DATA_KEY` ("$data")
/// at `expression.rs:29`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DataRef {
    #[serde(rename = "$data")]
    pub data: String,
}

// Spec.title becomes:
#[serde(default, skip_serializing_if = "Option::is_none")]
pub title: Option<TitleBinding>,
```

**Re-export to add** (`ferro-json-ui/src/lib.rs:83-86` — currently re-exports `spec::*` types):
- Add `TitleBinding` and `DataRef` to the `pub use spec::{...}` re-export block.

#### Modify: `framework/src/json_ui/mod.rs:89` (title extraction)

**Existing literal-only path (line 89):**
```rust
let title = spec.title.as_deref().unwrap_or("Ferro");
```

**Resolve via `resolve_path` (use the same `data::resolve_path` that powers `render_data_table` at `data.rs:19-40`):**
```rust
let title: String = match &spec.title {
    None => "Ferro".to_string(),
    Some(ferro_json_ui::TitleBinding::Literal(s)) => s.clone(),
    Some(ferro_json_ui::TitleBinding::Binding(r)) => {
        // Resolve at render time against spec.data. Missing path → empty title.
        // resolve_path is pub(crate) in ferro-json-ui; expose a thin
        // public helper or perform inline JSONPointer resolution here.
        spec.data.pointer(&r.data)
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "Ferro".to_string())
    }
};
let title = title.as_str();   // build_response signature expects &str — keep call site identical
```

**Note:** `resolve_path` is `pub(crate)` (see `data.rs:19`). The cross-crate caller in `framework/` must use `serde_json::Value::pointer` (already imported) which accepts JSON Pointer syntax `/foo/bar`. This avoids a new public surface and matches the JSONPath-style strings used everywhere else in the codebase.

---

### D-13a — `KanbanBoardProps.data_path` (data-driven columns)

#### Modify: `ferro-json-ui/src/component.rs:862-867` (`KanbanBoardProps`)

**Analog:** `DataTableProps` at `component.rs:835-848`:
```rust
pub struct DataTableProps {
    pub columns: Vec<Column>,
    pub data_path: String,                                   // REQUIRED — drives the entire table
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_actions: Option<Vec<DropdownMenuAction>>,
    // ...
}
```

**Existing KanbanBoardProps (lines 862-867):**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KanbanBoardProps {
    pub columns: Vec<KanbanColumnProps>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile_default_column: Option<String>,
}
```

**Pattern to apply (data_path as an OPTIONAL override of static `columns`):**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KanbanBoardProps {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<KanbanColumnProps>,           // CHANGED: now skippable when data_path set
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,                 // NEW: runtime override
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile_default_column: Option<String>,
}
```

#### Modify: `ferro-json-ui/src/render/containers.rs:292-302` (`render_kanban_board`)

**Existing render fn entry (lines 292-305):**
```rust
pub(crate) fn render_kanban_board(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: KanbanBoardProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode KanbanBoard props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    if props.columns.is_empty() {
        return String::new();
    }
```

**Pattern to apply (mirror `render_data_table` at `render/data.rs:144-145`):**
```rust
// After the from_value decode, before the empty check:
use crate::data::resolve_path;
let columns: Vec<KanbanColumnProps> = if let Some(path) = props.data_path.as_deref() {
    // Runtime override: deserialize column array from data at path.
    let raw = resolve_path(data, path)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    raw.into_iter()
        .filter_map(|v| serde_json::from_value::<KanbanColumnProps>(v).ok())
        .collect()
} else {
    props.columns.clone()
};

if columns.is_empty() {
    return String::new();
}
// ...rest of fn uses local `columns` shadow instead of props.columns
```

**Visibility note:** `resolve_path` is `pub(crate)` in `ferro-json-ui::data` — same crate as `render::containers`, so the import works (already used by `render::data::render_data_table`).

---

### D-13b — Document `$each` for kanban columns

**File:** `docs/src/json-ui/expressions.md`
**No code analog** — pure doc work. Extend the existing `$each` section (added in Phase 163 D-08) with a worked example that builds a kanban board from a data array. Reference D-13a above as the preferred path for "one element type per column"; reserve `$each` for "one element type per card row inside a column" (per RESEARCH Assumption A7).

---

### D-14 — Raise `MAX_NESTING_DEPTH` 3 → 5

#### Modify: `ferro-json-ui/src/spec.rs:37`

**Existing line:**
```rust
/// Maximum allowed nesting depth from the root element.
///
/// Matches the Screen > Section > Component hierarchy documented by the
/// SDUI research in `115-CONTEXT.md` (D-09). Paths exceeding this depth
/// surface as [`SpecError::DepthExceeded`].
pub const MAX_NESTING_DEPTH: usize = 3;
```

**Replace with:**
```rust
pub const MAX_NESTING_DEPTH: usize = 5;
```

Update the doc comment to reflect the new constraint (root → grid → card → row → atom = depth 5).

#### Modify: `ferro-json-ui/src/spec.rs:1703-1725` (existing test)

**Existing test (`nested_builder_flattens_two_levels`):**
```rust
#[test]
fn nested_builder_flattens_two_levels() {
    // root > section > text — three-deep, exactly at MAX_NESTING_DEPTH=3.
    let spec = Spec::builder()
        .element_nested(
            "root",
            NestedElement::new("Screen").child(
                NestedElement::new("Section")
                    .child(NestedElement::new("Text").prop("content", "leaf")),
            ),
        )
        .build()
        .expect("three levels at depth limit must be valid");
    // ...
}
```

**Add two new tests using the same builder shape (one for depth 5 valid, one for depth 6 rejected as `SpecError::DepthExceeded`).** The existing test pattern is the template — just chain more `NestedElement::new(...).child(...)` calls.

#### Modify: `docs/src/json-ui/spec-construction.md`

Update any `MAX_NESTING_DEPTH = 3` reference to `5`. Add the rationale (depth-4 dashboard hit ceiling; 5 covers it with one level of headroom).

---

### D-15 — `data_path` on `ImageProps` and `DescriptionListProps`

#### Modify: `ferro-json-ui/src/component.rs:514-536` (`ImageProps`)

**Strongest analog (THREE existing `data_path: Option<String>` precedents in the same file):**

`InputProps.data_path` (`component.rs:255-256`):
```rust
/// Data path for pre-filling from handler data (e.g., "/data/user/name").
#[serde(default, skip_serializing_if = "Option::is_none")]
pub data_path: Option<String>,
```

`SelectProps.data_path` (`component.rs:311-312`):
```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub data_path: Option<String>,
```

`RichTextEditorProps.data_path` (`component.rs:285-286`):
```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub data_path: Option<String>,
```

**Pattern to apply to `ImageProps`:**
```rust
pub struct ImageProps {
    #[serde(default)]                            // CHANGED: was required
    pub src: String,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_svg: Option<String>,
    /// Optional data-path override of `src`. When set, the renderer resolves
    /// the value at this path against handler data and uses it as the
    /// `<img src>`. Falls back to `src` when missing or non-string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,               // NEW
}
```

**Note `inline_svg` constructor (`component.rs:544-552`) sets `src: String::new()`.** That constructor must be updated to also initialize `data_path: None`.

#### Modify: `ferro-json-ui/src/component.rs:454-460` (`DescriptionListProps`)

**Existing:**
```rust
pub struct DescriptionListProps {
    pub items: Vec<DescriptionItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u8>,
}
```

**Pattern to apply:**
```rust
pub struct DescriptionListProps {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<DescriptionItem>,             // CHANGED: was required
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u8>,
    /// Optional data-path override of `items`. When set, the renderer
    /// resolves the array at this path and decodes each entry as a
    /// `DescriptionItem`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,               // NEW
}
```

#### Modify: `ferro-json-ui/src/render/atoms.rs:365-405` (`render_image`)

**Existing signature (note `_data: &Value` — currently unused):**
```rust
pub(crate) fn render_image(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
```

**Drop the underscore on `data` and add resolution before line 395 (`format!("<div class=\"relative w-full\"...")`):**
```rust
pub(crate) fn render_image(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: ImageProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Image", e),
    };

    // inline_svg branch unchanged at lines 373-379.

    // D-15: data_path takes precedence over static src.
    let resolved_src = props
        .data_path
        .as_deref()
        .and_then(|p| crate::data::resolve_path(data, p))
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| props.src.clone());

    // ...rest of fn, replacing `props.src` with `resolved_src` in the format! at line 402-403
```

#### Modify: `ferro-json-ui/src/render/atoms.rs:563-584` (`render_description_list`)

**Same pattern — drop underscore on `_data`, resolve `data_path` → array, fall back to `props.items`:**
```rust
pub(crate) fn render_description_list(
    el: &Element,
    _spec: &Spec,
    data: &Value,
    _depth: usize,
) -> String {
    let props: DescriptionListProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("DescriptionList", e),
    };

    let items: Vec<DescriptionItem> = props.data_path
        .as_deref()
        .and_then(|p| crate::data::resolve_path(data, p))
        .and_then(|v| v.as_array().cloned())
        .map(|arr| arr.into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect())
        .unwrap_or_else(|| props.items.clone());

    // ...rest of fn uses `items` shadow instead of props.items
```

---

### D-16 — Validate after `expand_directives` (pipeline reorder)

#### Modify: `ferro-json-ui/src/loader.rs:138-143` (load-time path)

**Existing (single-stage, hard-fail at startup):**
```rust
let content = fs::read_to_string(&canonical)?;
let spec = Spec::from_json(&content).map_err(LoadError::Parse)?;
global_catalog()
    .validate(&spec)
    .map_err(LoadError::Catalog)?;
```

**Pattern (RESEARCH Pitfall 1 Option A — two-stage with warning):** Keep `Catalog::validate` at load time but DOWNGRADE catalog errors to a `tracing::warn!` (or similar) rather than failing. Structural errors from `Spec::from_json` still fail-loud. The hard-enforcement moves to per-request validation post `expand_directives`.

```rust
let content = fs::read_to_string(&canonical)?;
let spec = Spec::from_json(&content).map_err(LoadError::Parse)?;
// Catalog validation at load time becomes a WARNING (D-16 Option A).
// Hard enforcement moves to JsonUi::resolve, AFTER expand_directives,
// so $if-gated elements with shape-invalid props no longer fail at load.
if let Err(errs) = global_catalog().validate(&spec) {
    // structural errors (footer IDs, element references, depth) already caught
    // by Spec::from_json above; remaining errors are enum-shape problems that
    // may be gated by $if at render time.
    for e in &errs {
        tracing::warn!(target: "ferro_json_ui::catalog", "load-time validation warning: {e}");
    }
}
```

#### Modify: `framework/src/json_ui/mod.rs:48-54` (`JsonUi::resolve`)

**Existing pipeline (3 steps):**
```rust
fn resolve(spec: &Spec) -> Spec {
    let mut resolved = spec.clone();
    expand_directives(&mut resolved);
    resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
    resolve_expressions(&mut resolved);
    resolved
}
```

**Pattern to apply (insert catalog validation AFTER `expand_directives`):**
```rust
fn resolve(spec: &Spec) -> Spec {
    let mut resolved = spec.clone();
    expand_directives(&mut resolved);
    // D-16: validate after $if removal so gated bad-variant specs pass.
    // Per-request enforcement (load-time is warning-only — see loader.rs).
    if let Err(errs) = ferro_json_ui::global_catalog().validate(&resolved) {
        // Surface as render-time error. Same surface as Spec::from_json failure.
        // Build_response converts Err into a 500 HTML response.
        // Choose pattern: either return a poison-spec, or thread an Option<Vec<CatalogError>>
        // back to render_with_config so it can short-circuit.
        // RECOMMENDATION: panic-free; convert to a synthetic Spec with a single
        // error-display element. Planner picks final shape.
        tracing::error!(target: "ferro_json_ui::catalog", "render-time validation failed: {errs:?}");
    }
    resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
    resolve_expressions(&mut resolved);
    resolved
}
```

**Note:** `resolve_with_errors` at `framework/src/json_ui/mod.rs:202-205` follows the same shape and must receive the identical insert.

#### NEW: `framework/tests/pipeline_order.rs`

**Analog:** the existing render integration tests at `framework/src/json_ui/mod.rs:696-1113` (specifically `render_with_errors_*` tests). The new test should construct a spec with `Alert { variant: "", visible: { exists: /flash } }` and verify that with `data = {}` (flash absent) the spec renders cleanly (the alert is removed by the `$if`/visibility mechanism before catalog validation runs).

---

### D-17a — `Component::RawHtml` (D-17a default per CONTEXT)

#### NEW: `RawHtmlProps` in `ferro-json-ui/src/component.rs`

**Analogs (two complementary precedents):**

1. **Struct shape — smallest props struct in the file:** `SkeletonProps` at `component.rs:568-576`:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SkeletonProps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rounded: Option<bool>,
}
```

2. **Trust-boundary doc — verbatim emission with `# Safety` rustdoc:** `ImageProps::inline_svg` at `component.rs:527-535`:
```rust
/// Server-rendered inline SVG string. When set, the SVG is emitted verbatim
/// inside a `<div aria-label="{alt}">` wrapper; no `<img>` tag is produced.
///
/// # Safety
/// Content is NOT sanitized. The SVG string is emitted into the response
/// verbatim. Pass only server-constructed SVG (e.g. bar charts, QR codes).
/// Do NOT pass untrusted input. `alt` is required and is HTML-escaped.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub inline_svg: Option<String>,
```

Also relevant: `RichTextEditorProps` rustdoc at `component.rs:267-276` ("Sanitization on submit is the consumer's responsibility — handle this in the form handler before persisting (e.g. via `ammonia`).") — this is the **discipline mandate** per RESEARCH `Don't Hand-Roll` row 3 and `Anti-Patterns` row 4.

**Pattern to apply:**
```rust
/// Props for the RawHtml component — server-injected HTML island.
///
/// # Safety
/// `html` is emitted into the response VERBATIM with NO sanitization. The
/// component exists to bridge server-rendered HTML fragments (e.g. a Stripe
/// Connect status pill, a WhatsApp link badge) into a v2 spec where a
/// first-class component would be over-engineering.
///
/// Sanitization is the CONSUMER's responsibility — pass only server-constructed
/// HTML, or run untrusted input through `ammonia` in the handler before
/// embedding. This mirrors `RichTextEditorProps` discipline (component.rs:273).
///
/// For richer widgets (interactive forms, charts, OAuth flows), use the
/// first-class plugin system (`JsonUiPlugin`) instead — see docs/src/json-ui/plugins.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RawHtmlProps {
    /// Server-constructed HTML emitted verbatim. NOT sanitized.
    #[serde(default)]
    pub html: String,
}
```

#### NEW: `render_raw_html` in `ferro-json-ui/src/render/atoms.rs`

**Analog (verbatim-emission body):** `render_image` inline-SVG branch at `render/atoms.rs:373-379`:
```rust
// D-17: inline SVG branch — emit verbatim, no <img> tag.
// Server-only; content is NOT sanitized; alt is HTML-escaped for the aria-label.
if let Some(ref svg) = props.inline_svg {
    return format!(
        "<div aria-label=\"{}\">{}</div>",
        html_escape(&props.alt),
        svg // verbatim — intentionally not escaped (server-only trust)
    );
}
```

**Pattern to apply (5-line render fn, single-purpose):**
```rust
// ── N. RawHtml — server-injected HTML island (D-17a) ────────────────────

pub(crate) fn render_raw_html(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: RawHtmlProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("RawHtml", e),
    };
    // Verbatim emission — intentionally not escaped (server-only trust).
    // See RawHtmlProps rustdoc for the trust boundary.
    format!("<div data-ferro-raw-html>{}</div>", props.html)
}
```

Import to add at `render/atoms.rs:14-21`: append `RawHtmlProps` to the existing `use crate::component::{...}` block.

#### Modify: `ferro-json-ui/src/render/mod.rs:41-86` (`BUILTIN_TYPES`)

**Pattern:** append `"RawHtml"` to the leaves block (after `"ProductTile"` at line 65) and add a dispatch arm at line 176 (after the existing leaf-dispatch lines):
```rust
"ProductTile" => atoms::render_product_tile(el, spec, data, depth),
"RawHtml" => atoms::render_raw_html(el, spec, data, depth),      // NEW
// Containers
"Card" => containers::render_card(el, spec, data, depth),
```

#### Modify: `ferro-json-ui/src/catalog.rs:123-369` (`BUILTIN_SPECS`)

**Analog (smallest leaf entry):** `Image` at `catalog.rs:167-172`:
```rust
(
    "Image",
    "Image with optional aspect ratio and skeleton fallback on load error.",
    || to_value(schema_for!(ImageProps)).unwrap(),
    &[],
),
```

**Pattern to apply (append after `ProductTile` at line 257-262):**
```rust
(
    "RawHtml",
    "Server-injected HTML island. CONSUMER is responsible for sanitization — see docs.",
    || to_value(schema_for!(RawHtmlProps)).unwrap(),
    &[],
),
```

**Critical:** order in `BUILTIN_SPECS` MUST match order in `BUILTIN_TYPES` (drift guard at `catalog.rs:531`). If the planner inserts `RawHtml` after `ProductTile` in `BUILTIN_TYPES`, it must do the same in `BUILTIN_SPECS`.

#### Modify: three count-assertion sites (Pitfall 3 — RESEARCH lines 362-372)

1. **`ferro-json-ui/src/render/mod.rs:530`** — `assert_eq!(BUILTIN_TYPES.len(), 40);` → `41`
2. **`ferro-json-ui/src/catalog.rs:1052`** — `assert_eq!(BUILTIN_SPECS.len(), 40);` → `41`
3. **`ferro-mcp/src/tools/json_ui_catalog.rs:289-291`** — bump literal `40` to `41` and `"all 40 built-in components"` → `"all 41 built-in components"`; append `"RawHtml"` to the expected-names array at `json_ui_catalog.rs:296-337`.

---

### D-18 — `CardVariant` enum on `CardProps`

#### Modify: `ferro-json-ui/src/component.rs:151-162` (`CardProps`)

**Analog (exact precedent — same crate, same file, same shape):** `ActionCardVariant` at `component.rs:888-896` + its use on `ActionCardProps.variant` at `component.rs:902-913`:
```rust
/// Visual variant for action cards.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionCardVariant {
    #[default]
    Default,
    Setup,
    Danger,
}

// ...

pub struct ActionCardProps {
    pub title: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub variant: ActionCardVariant,
    // ...
}
```

**Other matching enum precedents in the same file:** `AlertVariant` (line 89), `BadgeVariant` (line 100), `ButtonVariant` (line 55), `ToastVariant` (line 584). All use `#[serde(rename_all = "snake_case")]` (RESEARCH Pitfall 2 — DO NOT use `"lowercase"` from the friction file even though it's identical for single-word variants).

**Pattern to apply (per RESEARCH Pitfall 2 — use snake_case not lowercase):**
```rust
/// Visual variant for Card chrome. `Bordered` is the dashboard default
/// (border + subtle shadow, compact padding). `Elevated` is for auth /
/// error / marketing pages (no border, larger shadow, generous padding).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CardVariant {
    #[default]
    Bordered,
    Elevated,
}

// CardProps gets the variant field:
pub struct CardProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FormMaxWidth>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<String>,
    #[serde(default)]                                // NEW
    pub variant: CardVariant,                        // NEW
}
```

#### Modify: `ferro-json-ui/src/render/containers.rs:27-98` (`render_card`)

**Existing hard-coded class string (lines 53-55):**
```rust
let mut html = String::from(
    "<div class=\"rounded-lg border border-border bg-card shadow-sm overflow-visible\"><div class=\"p-4\">",
);
```

**Pattern to apply (per CONTEXT D-18 verbatim spec + RESEARCH `Code Examples` §Example 1):**
```rust
let (outer_class, inner_pad) = match props.variant {
    CardVariant::Bordered => (
        "rounded-lg border border-border bg-card shadow-sm overflow-visible",
        "p-4",
    ),
    CardVariant::Elevated => (
        "rounded-lg bg-card shadow-md overflow-visible",
        "p-8",
    ),
};
let mut html = format!(
    "<div class=\"{outer_class}\"><div class=\"{inner_pad}\">"
);
// ...rest of fn (h3, description, body wrapper, footer wrapper, max_width wrap) unchanged.
```

Import update at `containers.rs:14-17`: add `CardVariant` to the existing `use crate::component::{...}` block.

---

### D-19/F2 — Codemod uppercase HTTP methods (verification + regression test)

#### Verify: `ferro-cli/src/commands/json_ui_migrate_v1.rs:520-528`

**Existing code already emits uppercase:**
```rust
let method_name = path_ident_tail(&call.func)?;
let http_method = match method_name.as_str() {
    "post" => "POST",
    "get" => "GET",
    "put" => "PUT",
    "patch" => "PATCH",
    "delete" => "DELETE",
    _ => return None,
};
```

This matches `HttpMethod` at `action.rs:25-26`:
```rust
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod { ... }
```

**Action:** No code change. Add a regression test asserting `method: "POST"` shows up in the emitted JSON for `Action::post("foo")`.

#### NEW test in `ferro-cli/tests/json_ui_migrate_v1.rs`

**Analog:** existing `codemod_one_handler_emits_spec_and_rewrites_controller` at `ferro-cli/tests/json_ui_migrate_v1.rs:63-95`:
```rust
#[test]
fn codemod_one_handler_emits_spec_and_rewrites_controller() {
    let dir = TempDir::new().unwrap();
    let src_path = write_fixture(&dir, "in_auth.rs", "src/controllers/in_auth.rs");
    let _cwd_guard = ChangeCwd::new(dir.path());

    json_ui_migrate_v1::run(src_path.to_string_lossy().to_string(), false).expect("codemod runs");
    // ...assertions on emitted file
}
```

**Pattern to apply (mirror this exactly):**

1. Create new fixture `ferro-cli/tests/fixtures/migrate_v1/in_post_action.rs` with a controller that builds a v1 view containing `Action::post("users.store")`.
2. Add test `codemod_emits_uppercase_http_method` that runs the codemod on the fixture and asserts the output JSON contains `"method": "POST"` (uppercase) and does NOT contain `"method": "post"` (lowercase).

---

### D-19/F5 — Visibility error message names the bad variant

#### Modify: `ferro-json-ui/src/visibility.rs:43-50` (`Visibility` enum)

**Existing untagged-derive shape:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Visibility {
    And { and: Vec<Visibility> },
    Or { or: Vec<Visibility> },
    Not { not: Box<Visibility> },
    Condition(VisibilityCondition),
}
```

**No exact analog inside ferro-json-ui** — this would be the first hand-rolled `Deserialize` impl in the crate. RESEARCH `Pitfall 4` and `Don't Hand-Roll` row 2 provide the recipe:

**Pattern to apply (dispatch-by-shape, listing all four accepted forms):**
```rust
// Keep #[derive(Serialize, JsonSchema)] — only Deserialize is hand-rolled.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Visibility {
    And { and: Vec<Visibility> },
    Or { or: Vec<Visibility> },
    Not { not: Box<Visibility> },
    Condition(VisibilityCondition),
}

impl<'de> serde::Deserialize<'de> for Visibility {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let v = serde_json::Value::deserialize(d)?;
        // Dispatch by key presence.
        if let Some(obj) = v.as_object() {
            if obj.contains_key("and") {
                #[derive(serde::Deserialize)]
                struct AndShape { and: Vec<Visibility> }
                let shape: AndShape = serde_json::from_value(v.clone()).map_err(D::Error::custom)?;
                return Ok(Visibility::And { and: shape.and });
            }
            if obj.contains_key("or") {
                #[derive(serde::Deserialize)]
                struct OrShape { or: Vec<Visibility> }
                let shape: OrShape = serde_json::from_value(v.clone()).map_err(D::Error::custom)?;
                return Ok(Visibility::Or { or: shape.or });
            }
            if obj.contains_key("not") {
                #[derive(serde::Deserialize)]
                struct NotShape { not: Box<Visibility> }
                let shape: NotShape = serde_json::from_value(v.clone()).map_err(D::Error::custom)?;
                return Ok(Visibility::Not { not: shape.not });
            }
            if obj.contains_key("path") && obj.contains_key("operator") {
                let cond: VisibilityCondition = serde_json::from_value(v).map_err(D::Error::custom)?;
                return Ok(Visibility::Condition(cond));
            }
        }
        Err(D::Error::custom(format!(
            "invalid Visibility shape: {v}. \
             Accepted: \
             {{\"and\": [...]}}, \
             {{\"or\": [...]}}, \
             {{\"not\": {{...}}}}, \
             {{\"path\": \"/p\", \"operator\": \"...\", \"value\": ...}}"
        )))
    }
}
```

**Regression-test budget:** the round-trip tests in `visibility.rs::tests` (Phase 116 baseline) must continue to pass. Add a new test asserting the error message text contains all four shape names for input `{"expr": "foo"}`.

---

### D-19/F6 — `PageHeader.actions` accepts empty string (lax deserializer)

#### Modify: `ferro-json-ui/src/component.rs:795-804` (`PageHeaderProps`)

**Existing (lines 795-804):**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PageHeaderProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breadcrumb: Vec<BreadcrumbItem>,
    /// IDs of action button elements rendered to the right of the title.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
}
```

**No precedent for `deserialize_with = "..."` in `ferro-json-ui`** (RESEARCH Pitfall 6 confirms). Pattern is novel. Per RESEARCH Pitfall 6 the recipe avoids breaking the Rust API:

**Pattern to apply (RESEARCH lines 415-427 verbatim):**
```rust
#[serde(default, deserialize_with = "deserialize_actions_lax", skip_serializing_if = "Vec::is_empty")]
pub actions: Vec<String>,

// Helper at bottom of component.rs (or in a new module):
fn deserialize_actions_lax<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Vec<String>, D::Error> {
    use serde::de::Error;
    let v = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::String(s) if s.is_empty() => Ok(Vec::new()),
        serde_json::Value::Array(arr) => arr
            .into_iter()
            .map(|v| {
                v.as_str()
                    .map(String::from)
                    .ok_or_else(|| D::Error::custom("expected string"))
            })
            .collect(),
        other => Err(D::Error::custom(format!(
            "PageHeader.actions: expected array or empty string, got {other:?}"
        ))),
    }
}
```

**Trade-off:** This loosens the wire-format contract for `actions` only. Other `Vec<String>` ID-slot fields (e.g. `CardProps.footer`) remain strict. If the planner wants the laxness applied uniformly, the helper can be promoted to a crate-level utility and reused — but per CONTEXT D-19, F6 is lower-priority and may be deferred entirely.

---

### D-04 — MCP `json_ui_validate_spec` tool

#### NEW: `ferro-mcp/src/tools/json_ui_validate_spec.rs`

**Analog (tool shape):** `ferro-mcp/src/tools/json_ui_inspect.rs:1-99`:
```rust
//! JSON-UI inspect tool — ...
use serde::Serialize;
// ...

#[derive(Debug, Serialize)]
pub struct ComponentSchemaInfo {
    pub name: String,
    pub is_plugin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props_schema: Option<serde_json::Value>,
    // ...
}

pub fn inspect_component(component_type: &str) -> ComponentSchemaInfo {
    use ferro_json_ui::global_catalog;
    let cat = global_catalog();
    // ...
}
```

**Pattern to apply (per RESEARCH `Code Examples` §Example 3):**
```rust
//! JSON-UI validate-spec tool — surfaces every parse/catalog error from
//! `Spec::from_json` + `global_catalog().validate()` via MCP. Agents authoring
//! specs get the same diagnostics they would see at server startup.
use serde::Serialize;
use ferro_json_ui::{global_catalog, Spec};

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub structural_errors: Vec<String>,
    pub catalog_errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn execute(spec_json: &str) -> ValidateResponse {
    let mut response = ValidateResponse {
        valid: true,
        structural_errors: Vec::new(),
        catalog_errors: Vec::new(),
        warnings: Vec::new(),
    };
    let spec = match Spec::from_json(spec_json) {
        Ok(s) => s,
        Err(e) => {
            response.valid = false;
            response.structural_errors.push(e.to_string());
            return response;
        }
    };
    if let Err(errs) = global_catalog().validate(&spec) {
        response.valid = false;
        response.catalog_errors = errs.into_iter().map(|e| e.to_string()).collect();
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_valid_spec() { /* ... */ }
    #[test]
    fn reports_catalog_errors() { /* ... */ }
}
```

#### Modify: `ferro-mcp/src/tools/mod.rs:26-29` (registration)

**Existing block:**
```rust
pub mod json_ui_catalog;
pub mod json_ui_generate;
pub mod json_ui_inspect;
pub mod json_ui_verify_action;
```

**Add:**
```rust
pub mod json_ui_validate_spec;
```

Plus the corresponding MCP-tool dispatch wiring (search the `ferro-mcp` MCP server entrypoint for the existing `json_ui_inspect` registration and mirror it).

---

### D-05 — Validator coverage audit for Phase 163 directives

**File:** `ferro-json-ui/src/spec.rs:749+` (`validate_directives`)

**No new pattern** — this is an audit task. RESEARCH §D-05 (line 48) confirms `SpecError::EachPathNotArray` and `SpecError::IfPathMissing` already exist (`spec.rs:185-188`). The audit checks for the four CONTEXT-required behaviours:

1. `$each.path` resolves to a JSON array — **EXISTING** (`SpecError::EachPathNotArray` at `spec.rs:186`, validated at `spec.rs:771`).
2. `$if.path` resolves cleanly — **EXISTING** (`SpecError::IfPathMissing` at `spec.rs:188`, validated at `spec.rs:856`).
3. No circular references in templated elements — **EXISTING** (`SpecError::NestedEach` at `spec.rs:191-192`, `SpecError::MismatchedEach` at `spec.rs:193-198`).
4. No `children` references to absent elements unless gated by `$if` — **AUDIT NEEDED**: confirm `validate_no_dangling` skips children whose target has `if_ = Some(_)`. If it doesn't, the planner ships the fix here.

---

### D-01..D-03 — V1-Deletion-Readiness Audit

**File:** `.planning/phases/164-.../V1-DELETION-AUDIT.md`

**No Rust analog.** Pure phase artefact. RESEARCH `Example 4` (lines 532-555) ships the canonical 16-row table verbatim. Planner reuses that table as the starting point and verifies each row against current source.

---

### D-06..D-07 — Plugin Surface Audit

**File:** `docs/src/json-ui/plugins.md` (only modified if D-06 surfaces a gap)

**No Rust analog.** Pure paper exercise. RESEARCH `Open Questions` §4 makes this a conditional deliverable: written artefact `PLUGIN-SURFACE-AUDIT.md` if any of (Stripe widget / WhatsApp flow / chart renderer) paper-implementations surface a doc gap; verbal checkpoint otherwise.

---

### D-08..D-09 — Documentation Pass

**Files:** `docs/src/json-ui/{components,migration-v1-to-v2,expressions,spec-construction,plugins}.md`

**No Rust analog.** Each new field / variant / behaviour from D-12..D-19 needs a corresponding doc section. The pattern is "add a sub-heading + example JSON + one paragraph of rationale" — match the existing `components.md` per-component sections.

---

### D-10..D-11 — COMPLETED.md

**File:** `.planning/phases/164-.../COMPLETED.md`

**No Rust analog.** Pure phase artefact. CONTEXT D-10 enumerates the 5 required sections verbatim: Shipped across Phases 162-164 / Runtime frictions resolved (F1-F10 table) / Intentional gaps / Deferred to future milestones / v1 → v2 surface migration table. RESEARCH §D-10 confirms input feed: input to Phase 160's gate, basis of CHANGELOG entry at Phase 161.

---

## Shared Patterns

### S-1 — Props decode + diagnostic fallback (every renderer)

**Source:** `decode_props` + `decode_diagnostic` at `ferro-json-ui/src/render/atoms.rs:31-55`
**Apply to:** Every new renderer in D-13a, D-15, D-17a, D-18

```rust
fn decode_diagnostic(type_name: &str, err: impl std::fmt::Display) -> String {
    format!(
        "<!-- ferro-json-ui: failed to decode {} props: {} -->",
        type_name,
        html_escape(&err.to_string())
    )
}

fn decode_props<TProps: serde::de::DeserializeOwned>(
    props: &Value,
) -> Result<TProps, serde_json::Error> {
    if props.is_null() {
        serde_json::from_value(Value::Object(serde_json::Map::new()))
    } else {
        serde_json::from_value(props.clone())
    }
}
```

Every renderer starts with:
```rust
let props: XxxProps = match decode_props(&el.props) {
    Ok(p) => p,
    Err(e) => return decode_diagnostic("Xxx", e),
};
```

**Note:** `render/containers.rs` uses an inline `serde_json::from_value` + manual `format!` diagnostic (lines 27-36 of `render_card`) — the `decode_props`/`decode_diagnostic` helpers live only in `atoms.rs`. New container renderers may either inline the call (as `render_card` does) or import the helpers from `atoms`. Consistency is desirable; planner picks.

### S-2 — Data-path resolution

**Source:** `crate::data::resolve_path(data, path)` — defined `pub(crate)` at `ferro-json-ui/src/data.rs:19`
**Apply to:** D-13a (KanbanBoard), D-15 (Image, DescriptionList)

Idiomatic usage (from `render/data.rs:144-145`):
```rust
let rows = resolve_path(data, &props.data_path);
let items: Vec<Value> = rows.and_then(|v| v.as_array().cloned()).unwrap_or_default();
```

For optional `data_path: Option<String>` overriding a static field:
```rust
let resolved = props.data_path.as_deref()
    .and_then(|p| crate::data::resolve_path(data, p))
    .and_then(|v| v.as_str().map(String::from))
    .unwrap_or_else(|| props.static_field.clone());
```

### S-3 — Variant enum on a Props struct

**Source:** `ActionCardVariant` at `component.rs:888-896` + `AlertVariant` at `component.rs:84-95` + `BadgeVariant` at `component.rs:98-100` + `ButtonVariant` at `component.rs:50-63` + `ToastVariant` at `component.rs:580-590`
**Apply to:** D-18 (`CardVariant`)

Verbatim convention:
```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]    // NEVER "lowercase" — see RESEARCH Pitfall 2
pub enum FooVariant {
    #[default]
    Default,
    Other,
    Whatever,
}

// Field usage:
#[serde(default)]
pub variant: FooVariant,
```

For variant enums that participate in catalog visual generation (Button/Alert/Badge/Toast/ActionCard), `strum::AsRefStr` and `#[strum(serialize_all = "snake_case")]` are added — D-18's `CardVariant` doesn't currently need this since `CardProps` is not exposed through a catalog factory. Add only if a future need surfaces.

### S-4 — Cross-crate "BUILTIN count = N" invariant

**Source:** Three coordinated assertions
**Apply to:** D-17a (RawHtml component addition)

| File | Line | Current | New |
|------|------|---------|-----|
| `ferro-json-ui/src/render/mod.rs` | 530 | `assert_eq!(BUILTIN_TYPES.len(), 40);` | `41` |
| `ferro-json-ui/src/catalog.rs` | 1052 | `assert_eq!(BUILTIN_SPECS.len(), 40);` | `41` |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | 289-291 | `40` + `"all 40 built-in components"` | `41` + matching string |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | 296-337 | `expected` array of 40 names | append `"RawHtml"` |

Plus the drift guard at `catalog.rs:531-540` (`BUILTIN_SPECS.len() != BUILTIN_TYPES.len()`) — keeps the two parallel arrays in sync; no code change needed but the planner must add both entries in the SAME ORDER.

### S-5 — Schema regen is automatic via `schema_for!`

**Source:** Every entry in `BUILTIN_SPECS` at `catalog.rs:123-369` uses the closure `|| to_value(schema_for!(XxxProps)).unwrap()`.
**Apply to:** D-12 (TitleBinding/DataRef — schemars derives picked up by every Spec consumer), D-13a (KanbanBoardProps), D-15 (Image/DescriptionListProps), D-17a (new RawHtmlProps entry), D-18 (CardProps with new variant field)

No manual JSON Schema authoring. The `#[derive(JsonSchema)]` on each props struct drives generation; the MCP `json_ui_catalog` tool re-runs `schema_for!` on every call (`json_ui_catalog.rs::execute`). Adding new fields with proper `JsonSchema` derives propagates automatically.

### S-6 — `pub use` re-exports in `lib.rs`

**Source:** `ferro-json-ui/src/lib.rs:47-86`
**Apply to:** D-12 (`TitleBinding`, `DataRef`), D-17a (`RawHtmlProps`), D-18 (`CardVariant`)

Every public-API type added to `component.rs` or `spec.rs` must be appended to the corresponding `pub use component::{...}` (line 49) or `pub use spec::{...}` (line 83) block. The framework re-exports these via `framework/src/lib.rs`'s `pub use ferro_json_ui::*` cascade — no change needed downstream as long as the ferro-json-ui re-export is added.

### S-7 — `#[serde(rename_all = "snake_case")]` is workspace canonical

**Source:** Every enum in `component.rs`, `visibility.rs`, `action.rs` (action.rs:15 for variants; `HttpMethod` at action.rs:25 uses `"UPPERCASE"` — the one documented exception for HTTP semantic alignment).
**Apply to:** D-18 (`CardVariant`) — see Pitfall 2.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `.planning/phases/164-.../V1-DELETION-AUDIT.md` | phase artefact | doc | RESEARCH Example 4 ships the table; no Rust analog needed |
| `.planning/phases/164-.../COMPLETED.md` | phase artefact | doc | CONTEXT D-10 defines structure; no Rust analog needed |

---

## Metadata

**Analog search scope:**
- `ferro-json-ui/src/` (component.rs, spec.rs, visibility.rs, catalog.rs, action.rs, data.rs, expression.rs, loader.rs, resolve.rs, render/*.rs)
- `ferro-mcp/src/tools/` (json_ui_catalog.rs, json_ui_inspect.rs, mod.rs)
- `ferro-cli/src/commands/json_ui_migrate_v1.rs` + `ferro-cli/tests/json_ui_migrate_v1.rs` + fixtures
- `framework/src/json_ui/mod.rs` (resolve pipeline, build_response title extraction)

**Files scanned:** 14 source files + 1 test file + 3 fixture files

**Pattern extraction date:** 2026-05-17 (against `v12.0/json-ui-v2` HEAD = `ce44ac77`)

**Confidence:**
- D-12 binding type pattern — **MEDIUM-HIGH**: no exact precedent for top-level Spec field binding; `Visibility` is the closest untagged-enum analog; `DataRef` is new.
- D-13a, D-15 data_path pattern — **HIGH**: three existing `data_path: Option<String>` precedents in the same file.
- D-14 const bump — **HIGH**: trivial change.
- D-16 pipeline reorder — **MEDIUM**: requires architectural decision (where to enforce catalog errors at render time — synthetic error spec vs. propagated error). Planner should resolve in PLAN.
- D-17a RawHtml — **HIGH**: `render_image` inline-SVG branch is an exact precedent for verbatim emission; `SkeletonProps` is the smallest props-struct shape; `RichTextEditorProps` docstring is the trust-boundary template.
- D-18 CardVariant — **HIGH**: exact precedent (`ActionCardVariant` + 4 other variant enums) in the same file.
- D-19/F2 codemod — **HIGH**: code is already shipped; verification + regression test only.
- D-19/F5 Visibility error — **MEDIUM**: no existing hand-rolled `Deserialize` in this crate; pattern is novel but well-understood (RESEARCH Pitfall 4 + Don't Hand-Roll row 2).
- D-19/F6 PageHeader lax — **MEDIUM**: no existing `deserialize_with` in this crate; pattern is novel but mechanically straightforward.
- D-04 MCP tool — **HIGH**: `json_ui_inspect.rs` is the exact tool-shape template.
- D-01..D-11 (audit + docs + COMPLETED) — **HIGH**: defined as artefacts in CONTEXT/RESEARCH; no Rust analog needed.
