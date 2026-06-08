# Phase 169: StreamText Component - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 5 new/modified files
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/component.rs` | model | CRUD (props definition) | `component.rs:633-665` (`SkeletonProps` / `RawHtmlProps`) | exact |
| `ferro-json-ui/src/render/mod.rs` | controller | request-response (dispatch + init-script pipeline) | `render/mod.rs:43-131` (BUILTIN_TYPES + `render_spec_to_html_with_plugins`) | exact |
| `ferro-json-ui/src/render/atoms.rs` | service | request-response (leaf HTML emitter) | `atoms.rs:1374-1382` (`render_raw_html`) | exact |
| `ferro-json-ui/src/catalog.rs` | config | CRUD (component registration) | `catalog.rs:264-269` (`RawHtml` entry) | exact |
| `docs/src/json-ui/components.md` | docs | — | `docs/src/json-ui/components.md:1426-1447` (`### RawHtml`) | exact |

---

## Pattern Assignments

### `ferro-json-ui/src/component.rs` — add `StreamTextProps`

**Analog:** `component.rs:633-665` (`SkeletonProps` lines 633-642, `RawHtmlProps` lines 644-665)

**Derive + serde attribute pattern** (lines 633-642, `SkeletonProps` as cleanest all-optional example):
```rust
/// Props for Skeleton loading placeholder.
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

**Required-field pattern** (lines 660-665, `RawHtmlProps`):
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RawHtmlProps {
    /// Server-constructed HTML emitted verbatim. NOT sanitized.
    #[serde(default)]
    pub html: String,
}
```

**Key rules extracted from both analogs:**
- `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]` — exact set, no extras.
- `#[serde(rename_all = "snake_case")]` is **NOT** on props structs (it is only on enums like `ToastVariant` at line 671). Field names are already snake_case; no rename attribute needed.
- Required `String` fields: use `#[serde(default)]` to survive `null` prop values, as in `RawHtmlProps`.
- Optional fields: `#[serde(default, skip_serializing_if = "Option::is_none")]`, as in `SkeletonProps`.
- Placement: add `StreamTextProps` immediately after `RawHtmlProps` (after line 665), before the `ToastVariant` enum at line 667.

**Target struct shape:**
```rust
/// Props for the `StreamText` component — SSE token stream renderer.
///
/// Connects to `sse_url` via `EventSource` and appends arriving tokens as
/// plain text. The SSE endpoint MUST emit `event: done` on completion to
/// prevent `EventSource` auto-reconnect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StreamTextProps {
    /// URL of the server-sent-events endpoint that streams tokens.
    /// Must emit `event: done` on completion.
    #[serde(default)]
    pub sse_url: String,
    /// Text shown inside the content area before the first token arrives.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Status text shown while the stream is open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loading_text: Option<String>,
}
```

---

### `ferro-json-ui/src/render/mod.rs` — four edits

**Analog A: BUILTIN_TYPES array** (lines 43-92)

The full array ends with:
```rust
    "RawHtml",       // line 68 — last atom before Containers block
    // Containers (containers.rs)
    "Card",
    ...
    "MediaCardGrid", // line 91 — last entry, currently index 43 (len == 44)
];
```
Add `"StreamText"` immediately after `"RawHtml"` (line 68), before the `// Containers` comment. Update the comment at line 36 from `"44 built-in element type names"` to `"45 built-in element type names"`.

**Analog B: dispatch match arm** (lines 164-215)

Last atom arm before Containers block (line 189):
```rust
        "RawHtml" => atoms::render_raw_html(el, spec, data, depth),
        // Containers
        "Card" => containers::render_card(el, spec, data, depth),
```
Add immediately after line 189:
```rust
        "StreamText" => atoms::render_streamtext(el, spec, data, depth),
```

**Analog C: `render_spec_to_html_with_plugins` — early-return gap** (lines 114-131)

Current implementation (lines 114-131):
```rust
pub fn render_spec_to_html_with_plugins(spec: &Spec, data: &Value) -> RenderResult {
    let html = render_spec_to_html(spec, data);
    let plugin_types = collect_plugin_types(spec);
    if plugin_types.is_empty() {
        return RenderResult {
            html,
            css_head: String::new(),
            scripts: String::new(),
        };
    }
    let type_names: Vec<String> = plugin_types.into_iter().collect();
    let assets = collect_plugin_assets(&type_names);
    RenderResult {
        html,
        css_head: render_css_tags(&assets.css),
        scripts: render_js_tags(&assets.js, &assets.init_scripts),
    }
}
```
The `if plugin_types.is_empty()` early-return silently drops the StreamText init script when no plugins are present. Replace with the pattern below (adds `collect_builtin_init_scripts` call and merges its output with plugin init scripts before the early-return check).

**Analog D: `render_js_tags` signature** (lines 293-317)

```rust
pub(crate) fn render_js_tags(assets: &[Asset], init_scripts: &[String]) -> String {
    ...
    for init in init_scripts {
        out.push_str("<script>");
        out.push_str(init);
        out.push_str("</script>\n");
    }
    out
}
```
`init_scripts` is `&[String]`. The new `collect_builtin_init_scripts` returns `Vec<String>`. Merge with `assets.init_scripts` (also `Vec<String>`) via `.iter().chain(...).cloned().collect()` before passing to `render_js_tags`.

**Analog E: BUILTIN_TYPES count test** (lines 567-573)

```rust
#[test]
fn builtin_types_count_matches_dispatch() {
    // Defense-in-depth check: BUILTIN_TYPES must be 44 entries.
    // ...
    assert_eq!(BUILTIN_TYPES.len(), 44);
}
```
Update the comment and assertion: `44` → `45`.

**`collect_plugin_types` pattern for `collect_builtin_init_scripts`** (lines 242-250):
```rust
pub(crate) fn collect_plugin_types(spec: &Spec) -> HashSet<String> {
    let mut types = HashSet::new();
    for el in spec.elements.values() {
        if !BUILTIN_TYPES.contains(&el.type_name.as_str()) {
            types.insert(el.type_name.clone());
        }
    }
    types
}
```
Mirror this walk pattern for `collect_builtin_init_scripts`: iterate `spec.elements.values()`, check `el.type_name == "StreamText"`, return init script `Vec<String>` (one entry) or empty.

---

### `ferro-json-ui/src/render/atoms.rs` — add `render_streamtext`

**Analog:** `render_raw_html` (lines 1372-1382)

```rust
// ── RawHtml — server-injected HTML island ────────────────────────────────

pub(crate) fn render_raw_html(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: RawHtmlProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("RawHtml", e),
    };
    // Verbatim emission — intentionally NOT escaped (server-only trust).
    // See RawHtmlProps rustdoc for the trust boundary.
    format!("<div data-ferro-raw-html>{}</div>", props.html)
}
```

Key patterns to copy verbatim:
- Function signature: `pub(crate) fn render_*(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String`
- Props decode pattern: `let props: XProps = match decode_props(&el.props) { Ok(p) => p, Err(e) => return decode_diagnostic("X", e) };`
- `html_escape` for any prop value entering an HTML attribute: `use super::html_escape;` is already imported at line 22.
- `decode_diagnostic` and `decode_props` are already defined at lines 29-53 (same file).

**Import addition required in `atoms.rs` line 12-19 `use crate::component::{...}` block:**
Add `StreamTextProps` to the existing import list (currently ends with `ToastProps, ToastVariant`).

**Test patterns** (lines 2168-2184, `RawHtmlProps` round-trip tests):
```rust
// ── RawHtml (D-17a) ─────────────────────────────────────────────────

#[test]
fn raw_html_props_serde_roundtrip() {
    use crate::component::RawHtmlProps;
    let p = RawHtmlProps {
        html: "<span>x</span>".into(),
    };
    let j = serde_json::to_value(&p).unwrap();
    let back: RawHtmlProps = serde_json::from_value(j).unwrap();
    assert_eq!(p, back);
}

#[test]
fn render_raw_html_emits_verbatim() {
    let spec = spec_with_root(Element::new("RawHtml").prop("html", "<b>hi</b>"));
    let el = spec.elements.get("root").unwrap();
    let html = render_raw_html(el, &spec, &json!(null), 1);
    assert_eq!(html, "<div data-ferro-raw-html><b>hi</b></div>");
}
```

Test helper already in `atoms.rs` `#[cfg(test)]` module (line 1394):
```rust
fn spec_with_root(el: crate::spec::ElementBuilder) -> Spec {
    Spec::builder()
        .element("root", el)
        .build()
        .expect("trivial spec builds")
}
```
Use this helper for all `render_streamtext` tests — no new helper needed.

---

### `ferro-json-ui/src/catalog.rs` — register `StreamText`

**Analog:** `RawHtml` entry (lines 264-269)

```rust
(
    "RawHtml",
    "Server-injected HTML island. CONSUMER is responsible for sanitization — see docs/src/json-ui/plugins.md.",
    || to_value(schema_for!(RawHtmlProps)).unwrap(),
    &[],
),
```

**Import block** (lines 29-38, must add `StreamTextProps`):
```rust
use crate::component::{
    ActionCardProps, AlertProps, AvatarProps, BadgeProps, BreadcrumbProps, ButtonGroupProps,
    ButtonProps, CalendarCellProps, CardProps, CheckboxListProps, CheckboxProps, ChecklistProps,
    CollapsibleProps, DataTableProps, DescriptionListProps, DetailPageProps, DropdownMenuProps,
    EmptyStateProps, FormProps, FormSectionProps, GridProps, HeaderProps, ImageProps, InputProps,
    KanbanBoardProps, MediaCardGridProps, ModalProps, NotificationDropdownProps, PageHeaderProps,
    PaginationProps, ProductTileProps, ProgressProps, RawHtmlProps, SelectProps, SeparatorProps,
    SidebarProps, SkeletonProps, StatCardProps, SwitchProps, TableProps, TabsProps, TextProps,
    ToastProps,
};
```
Add `StreamTextProps` to this list (alphabetical order puts it after `StatCardProps`, before `SwitchProps`... or simply append before `ToastProps`).

**BUILTIN_SPECS entry** (add immediately after `RawHtml` entry at line 269):
```rust
(
    "StreamText",
    "Connects to a server-sent-events endpoint and renders token-by-token output as plain text. The SSE endpoint must emit `event: done` on completion to prevent auto-reconnect.",
    || to_value(schema_for!(StreamTextProps)).unwrap(),
    &[],
),
```

**Important:** `BUILTIN_SPECS` order must match `BUILTIN_TYPES` order exactly (enforced by a drift guard in `Catalog::build`). `StreamText` goes immediately after `RawHtml` in both arrays.

---

### `docs/src/json-ui/components.md` — add `### StreamText`

**Analog:** `### RawHtml` section (lines 1426-1447)

```markdown
### RawHtml

Server-injected HTML island for narrow HTML-fragment use cases: ...

| Prop | Type | Description |
|------|------|-------------|
| `html` | `string` | Server-constructed HTML emitted verbatim into the response |

\`\`\`json
"status_pill": {
  "type": "RawHtml",
  "props": {
    "html": "<span class=\"pill pill-green\">Active</span>"
  }
}
\`\`\`

**Trust boundary.** ...
```

Add `### StreamText` immediately after the `### RawHtml` section (after line 1447), before the `---` separator. Follow the exact structure: description prose, props table, JSON usage example, contract note block.

---

## Shared Patterns

### `decode_props` + `decode_diagnostic`
**Source:** `ferro-json-ui/src/render/atoms.rs:29-53`
**Apply to:** `render_streamtext` in `atoms.rs`
```rust
fn decode_props<TProps: serde::de::DeserializeOwned>(
    props: &Value,
) -> Result<TProps, serde_json::Error> {
    if props.is_null() {
        serde_json::from_value(Value::Object(serde_json::Map::new()))
    } else {
        serde_json::from_value(props.clone())
    }
}

fn decode_diagnostic(type_name: &str, err: impl std::fmt::Display) -> String {
    format!(
        "<!-- ferro-json-ui: failed to decode {} props: {} -->",
        type_name,
        html_escape(&err.to_string())
    )
}
```

### `html_escape`
**Source:** `ferro-json-ui/src/render/mod.rs:256-262`
**Apply to:** `sse_url`, `placeholder`, `loading_text` in `render_streamtext`; already `pub(crate)` and imported as `use super::html_escape` in `atoms.rs:22`
```rust
pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
```

### `render_js_tags` init_scripts slot
**Source:** `ferro-json-ui/src/render/mod.rs:293-317`
**Apply to:** `render_spec_to_html_with_plugins` fix in `render/mod.rs`
```rust
pub(crate) fn render_js_tags(assets: &[Asset], init_scripts: &[String]) -> String {
    ...
    for init in init_scripts {
        out.push_str("<script>");
        out.push_str(init);
        out.push_str("</script>\n");
    }
    out
}
```
The `init_scripts` parameter is `&[String]`. Build a merged `Vec<String>` from `assets.init_scripts` + `builtin_scripts`, then pass `.as_slice()`.

### `schema_for!` catalog pattern
**Source:** `ferro-json-ui/src/catalog.rs:126-131` (any `BUILTIN_SPECS` entry)
**Apply to:** `StreamText` catalog entry
```rust
(
    "Text",
    "Semantic text element (p / h1 / h2 / h3 / span / div / section).",
    || to_value(schema_for!(TextProps)).unwrap(),
    &[],
),
```
The `schema_for!(XProps)` macro derives the JSON Schema from the `#[derive(JsonSchema)]` on the struct; the `|| ... .unwrap()` is a `SchemaFn` closure called lazily.

---

## No Analog Found

None — all five files have exact analogs in the codebase.

---

## Critical Implementation Notes (traps the planner must include as explicit tasks)

| Trap | Where | What Must Happen |
|------|-------|-----------------|
| Early-return drops init script | `render/mod.rs:117-123` | Condition must become `plugin_types.is_empty() && builtin_scripts.is_empty()` |
| BUILTIN_TYPES count test | `render/mod.rs:572` | `assert_eq!(BUILTIN_TYPES.len(), 44)` → `45`; same commit as array edit |
| BUILTIN_SPECS order drift | `catalog.rs` | `StreamText` must follow `RawHtml` in both `BUILTIN_TYPES` and `BUILTIN_SPECS` (drift guard enforces this at runtime) |
| `serde(rename_all)` confusion | `component.rs` | Do NOT add `#[serde(rename_all = "snake_case")]` to `StreamTextProps` (enum-only attribute; `RawHtmlProps` and `SkeletonProps` confirm omission) |
| XSS via placeholder/loading_text | `atoms.rs` | All three props (`sse_url`, `placeholder`, `loading_text`) must pass through `html_escape` before HTML emission |
| `innerHTML` prohibition | init script JS | Tokens appended via `document.createTextNode(e.data)`, never `innerHTML` |

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/` (component.rs, render/mod.rs, render/atoms.rs, catalog.rs), `docs/src/json-ui/components.md`
**Files scanned:** 5
**Pattern extraction date:** 2026-06-08
