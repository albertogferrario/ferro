# Phase 148: HtmlEmbed component — Pattern Map

**Mapped:** 2026-04-24
**Files analyzed:** 6 modified (5 Rust + 1 docs)
**Analogs found:** 6 / 6 — every surface has an in-crate precedent with verified line numbers
**Precedent phase:** 147 (DetailForm). Same playbook, smaller footprint (no runtime JS; no container recursion; no action; no error slot).

> **Density note.** The canonical code excerpts for every analog already live at fixed line numbers in
> `148-RESEARCH.md` §Architecture Patterns (Patterns 1-10). This PATTERNS.md is a single-file map that
> classifies each touched file, names its analog, anchors the insertion point, and **points at
> 148-RESEARCH.md** for the full excerpt rather than copying the same block twice. The planner reads
> both files; duplication is noise.

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/component.rs` | types + enum variant + serde arms + factory + test fixture | transform (Rust ↔ JSON shape) | `SeparatorProps` (L524-530) for struct; `Component::Separator` arms in enum/Serialize/Deserialize; `ComponentNode::separator` (L1478-1486) for factory; `all_known_types_round_trip` (L3710-3750) for fixture append | exact |
| `ferro-json-ui/src/render.rs` | render fn + dispatch arm + plugin-walk leaf arm + tests | request-response (HTML emit, the **one** `html_escape` bypass) | `render_separator` (L2359-2365) for fn shape; `render_component` dispatch (L294-322) for arm; `collect_plugin_types_node` leaf OR-chain (L164-194) | exact (shape); **intentional divergence**: no `html_escape` call |
| `ferro-json-ui/src/resolve.rs` | three leaf-group OR-chain additions + no-op tests | batch walk (no-op for this variant) | `resolve_component_node` leaf chain (L135-160) ending in `Plugin(_)`; `collect_unresolved_node` leaf chain (L313-338); `resolve_errors_node` leaf chain (L462-488) | exact |
| `ferro-json-ui/src/lib.rs` | public re-export + `COMPONENT_CATALOG` string entry | public API surface | re-export block (L59-72, alphabetical); `### DetailForm` catalog entry (L120-122) as safety-density precedent | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | `CatalogComponent` entry + exhaustive-list bump 41 → 42 | catalog entry + test fixture | `CatalogComponent { name: "Separator", ... }` (L597-607) for minimal single-prop shape; exhaustive-list assertion (L1207-1264) | exact |
| `docs/src/json-ui/components.md` | new `### HtmlEmbed` section | documentation | `### Separator` (L233-258) for density; `### DetailForm` (L473+) for safety-callout precedent | role-match (safety callout is new authorial content) |

**Files NOT modified (deliberate, per CONTEXT D-24 / §Deferred):**
- `ferro-json-ui/src/runtime/*` — no JS; HtmlEmbed has no runtime behavior.
- `ferro-json-ui/src/visibility.rs`, `ferro-json-ui/src/layout.rs`, `ferro-json-ui/src/plugin.rs` — unrelated surfaces.
- `ferro-mcp/src/tools/json_ui_catalog.rs:1375` `no_required` array — `html` is required, so HtmlEmbed is NOT added (D-20).

---

## Pattern Assignments

### 1. `ferro-json-ui/src/component.rs`

Five discrete edits. All analogs verified by direct Read of the current file.

#### 1a. `HtmlEmbedProps` struct — new type

**Primary analog:** `SeparatorProps` at `component.rs:524-530` — canonical minimal single-ish-field props struct.

**Current source (verified):**
```rust
// component.rs:524-530
/// Props for Separator component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeparatorProps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
}
```

**Divergence from analog:**
1. Drop `Eq` (matches `ImageProps` / `TextProps`; `String` has no meaningful `Eq` beyond `PartialEq`).
2. Prepend safety-first rustdoc (D-15) covering: (a) verbatim emission, (b) caller-owned XSS safety, (c) intended for server-generated content not user input, (d) pointer to `Component::Text` for escaped output.
3. Add `impl HtmlEmbedProps { pub fn new(html: impl Into<String>) -> Self { ... } }` (D-03).
4. Single required field — NO `#[serde(default, skip_serializing_if = ...)]` attribute (html is required, not optional).

**Full proposed struct body:** see `148-RESEARCH.md` §Pattern 1 (L236-262). Do not duplicate here.

**Insertion point:** Near the other single-field leaf props structs. `SeparatorProps` sits at L524-530; `DescriptionListProps` at L540-546. Place `HtmlEmbedProps` anywhere in that neighborhood that keeps leaf-props grouped — exact line is a write-time cosmetic decision.

**Doctest note (Pitfall 6):** The `HtmlEmbedProps::new` rustdoc example uses `use ferro_json_ui::HtmlEmbedProps` which depends on the re-export in `lib.rs` (§4a below). Wave 1 plan MUST touch `component.rs` and `lib.rs` together — splitting across dependent plans breaks `cargo test --doc` between merges.

---

#### 1b. `Component::HtmlEmbed` variant

**Analog:** Every existing variant in `Component` enum at `component.rs:1055-1098`. Immediately adjacent precedent is `Component::DetailForm(DetailFormProps)` at L1096.

**Current source (verified L1094-1098):**
```rust
    Image(ImageProps),
    KeyValueEditor(KeyValueEditorProps),
    DetailForm(DetailFormProps),
    Plugin(PluginProps),
}
```

**Emission (D-02):**
```rust
    Image(ImageProps),
    KeyValueEditor(KeyValueEditorProps),
    DetailForm(DetailFormProps),
    HtmlEmbed(HtmlEmbedProps),   // NEW
    Plugin(PluginProps),
```

**Ordering rule:** Enum ordering is **recency-grouped, Plugin always last**. The rule holds across every `match` block in this file (Serialize, Deserialize, render dispatch, resolver passes) — all four lists follow the enum declaration order. Phase 147 inserted `DetailForm` after `KeyValueEditor`; phase 148 inserts `HtmlEmbed` after `DetailForm`. Preserve Plugin at the tail.

---

#### 1c. `Component` Serialize arm

**Analog:** Every arm in the `impl Serialize for Component` block (L1119-1167). Immediate precedent: `Component::DetailForm(p) => serialize_tagged(serializer, "DetailForm", p)` at L1164.

**Current source (verified L1162-1166):**
```rust
            Component::Image(p) => serialize_tagged(serializer, "Image", p),
            Component::KeyValueEditor(p) => serialize_tagged(serializer, "KeyValueEditor", p),
            Component::DetailForm(p) => serialize_tagged(serializer, "DetailForm", p),
            Component::Plugin(p) => p.serialize(serializer),
        }
```

**Emission (D-12):**
```rust
            Component::DetailForm(p) => serialize_tagged(serializer, "DetailForm", p),
            Component::HtmlEmbed(p) => serialize_tagged(serializer, "HtmlEmbed", p),   // NEW
            Component::Plugin(p) => p.serialize(serializer),
```

**Helper used:** `serialize_tagged` at `component.rs:1104-1117` — no change needed; reused as-is.

---

#### 1d. `Component` Deserialize arm

**Analog:** Every arm in the `impl<'de> Deserialize<'de> for Component` block (L1172-1314). Immediate precedent: `"DetailForm"` arm at L1301-1303.

**Current source (verified L1298-1305):**
```rust
            "KeyValueEditor" => serde_json::from_value::<KeyValueEditorProps>(value)
                .map(Component::KeyValueEditor)
                .map_err(de::Error::custom),
            "DetailForm" => serde_json::from_value::<DetailFormProps>(value)
                .map(Component::DetailForm)
                .map_err(de::Error::custom),
            _ => {
                // Unknown type: treat as a plugin component.
```

**Emission (D-13):**
```rust
            "DetailForm" => serde_json::from_value::<DetailFormProps>(value)
                .map(Component::DetailForm)
                .map_err(de::Error::custom),
            "HtmlEmbed" => serde_json::from_value::<HtmlEmbedProps>(value)   // NEW
                .map(Component::HtmlEmbed)
                .map_err(de::Error::custom),
            _ => { /* plugin fallback */ }
```

**Insertion point:** After the `"DetailForm"` arm (ending L1303), before the `_ =>` plugin fallback (starts L1304).

---

#### 1e. `ComponentNode::html_embed` factory

**Analog:** `ComponentNode::separator` at `component.rs:1478-1486`.

**Current source (verified L1478-1486):**
```rust
    /// Create a Separator component node.
    pub fn separator(key: impl Into<String>, props: SeparatorProps) -> Self {
        Self {
            key: key.into(),
            component: Component::Separator(props),
            action: None,
            visibility: None,
        }
    }
```

**Emission (D-04):**
```rust
    /// Create an HtmlEmbed component node.
    ///
    /// ⚠️ **Safety.** The HTML in `props.html` is emitted verbatim without
    /// escaping. Do not pass user input. See [`HtmlEmbedProps`] for the full
    /// safety contract.
    pub fn html_embed(key: impl Into<String>, props: HtmlEmbedProps) -> Self {
        Self {
            key: key.into(),
            component: Component::HtmlEmbed(props),
            action: None,
            visibility: None,
        }
    }
```

**Insertion point:** Near the other recent factories. `separator` at L1478-1486 is the shape peer; the recency group (where phase 147 added `detail_form`) is a few dozen lines further down. Write-time decision — both locations are correct.

**Do NOT collapse to `(key, html)` shorthand** (D-04). Uniformity with the rest of the factory surface wins over one-keystroke savings.

---

#### 1f. `all_known_types_round_trip` fixture append

**Analog:** Every tuple in the `known_types` array at `component.rs:3711-3733`. `Separator` at L3730 is the closest shape (no required fields, minimal JSON).

**Current source (verified L3711-3733):**
```rust
        let known_types: &[(&str, &str)] = &[
            ("Alert", r#"{"type":"Alert","message":"m"}"#),
            ...
            ("Separator", r#"{"type":"Separator"}"#),
            ("Skeleton", r#"{"type":"Skeleton"}"#),
            ("Text", r#"{"type":"Text","content":"c"}"#),
        ];
```

**Emission:** Append one tuple (alphabetical slot between `Image` at L3719 and `Input` at L3720 — but appending at the tail before the closing bracket is equally acceptable; the array is consumed as an unordered set by the for-loop at L3734-3749):
```rust
            ("HtmlEmbed", r#"{"type":"HtmlEmbed","html":"<b>x</b>"}"#),
```

**Why this matters:** Round-trip test is a secondary safety net for serde arms 1c / 1d. Compile won't catch a typo in the type string `"HtmlEmbed"` (D-14 constrains it exactly) — this test will.

**Additional tests to add** in `#[cfg(test)] mod tests` (Wave 0 RED):
- `html_embed_serde_roundtrip` — full struct → JSON → struct with `PartialEq` equality. Mirrors the pattern in `key_value_editor_serde_roundtrip` at L3759+.
- `html_embed_factory_sets_key` — assert `ComponentNode::html_embed("k", HtmlEmbedProps::new("<p>x</p>")).key == "k"`.
- `html_embed_props_new_accepts_str_and_string` — exercise the `impl Into<String>` bound on both literal and owned inputs.

---

### 2. `ferro-json-ui/src/render.rs`

Three discrete edits (render fn + dispatch arm + leaf-walk arm) + load-bearing tests.

#### 2a. `render_html_embed` function — new (the `html_escape` exception)

**Analog:** `render_separator` at `render.rs:2359-2365` — minimal render function with no data param, no children, returns a single string.

**Current source (verified L2359-2365):**
```rust
fn render_separator(props: &SeparatorProps) -> String {
    let orientation = props.orientation.as_ref().cloned().unwrap_or_default();
    match orientation {
        Orientation::Horizontal => "<hr class=\"my-4 border-border\">".to_string(),
        Orientation::Vertical => "<div class=\"mx-4 h-full w-px bg-border\"></div>".to_string(),
    }
}
```

**Emission (D-05, D-06, D-09):** Even simpler — one `format!`, no variant matching, and **crucially** no `html_escape` call on the dynamic `props.html` field. This is the **only** render function in the file that does not escape its dynamic string input.

Full body excerpt available at `148-RESEARCH.md` §Pattern 4 (L358-371). Key requirements the planner must preserve verbatim:
1. Function signature `fn render_html_embed(props: &HtmlEmbedProps) -> String` — **no `data: &Value` parameter** (D-05). Dropping `data` is deliberate: HtmlEmbed has no data-binding surface in this phase (D-06).
2. Body literal: `format!("<div>{}</div>", props.html)` — no default class, id, attributes (D-06).
3. **Mandatory inline comment** flagging the deliberate `html_escape` omission. Without it, a future audit (human or agent) will "fix" the missing escape as a bug. The comment is load-bearing documentation, not cosmetic. See §Shared Patterns S-3 below.
4. Rustdoc on the function restating the bypass contract (agent-readable via rust-analyzer / cargo doc).
5. Function visibility: module-private (no `pub` modifier) — matches `render_separator`.

**Insertion point:** Near `render_separator` at L2359 in the simple-leaf render cluster, OR alphabetically — write-time cosmetic choice.

---

#### 2b. Dispatch arm in `render_component`

**Analog:** `Component::Separator(props) => render_separator(props)` at `render.rs:300`. Every leaf arm in this match block omits the `data` parameter.

**Current source (verified L294-322):**
```rust
fn render_component(component: &Component, data: &Value) -> String {
    match component {
        Component::Text(props) => render_text(props),
        Component::Button(props) => render_button(props),
        Component::Badge(props) => render_badge(props),
        Component::Alert(props) => render_alert(props),
        Component::Separator(props) => render_separator(props),
        Component::Progress(props) => render_progress(props),
        ...
        Component::DescriptionList(props) => render_description_list(props),

        // Container components.
        Component::Card(props) => render_card(props, data),
        ...
        Component::KeyValueEditor(props) => render_key_value_editor(props, data),
        ...
```

**Emission (D-07):**
```rust
        Component::Separator(props) => render_separator(props),
        Component::HtmlEmbed(props) => render_html_embed(props),   // NEW — simple leaf, no data arg
        Component::Progress(props) => render_progress(props),
```

**Ordering rule:** The dispatch table is grouped by family (leaf-simple cluster, Container components, Form field components, Layout components, Standalone components — separated by `// ── family name ──` comment banners). HtmlEmbed belongs in the **simple-leaf cluster** near `Separator` / `DescriptionList`. Not in Container (no children) and not in Form field (no action / no data binding).

---

#### 2c. Leaf arm in `collect_plugin_types_node`

**Analog:** The leaf OR-chain at `render.rs:164-194` ending in `Component::KeyValueEditor(_) => {}`.

**Current source (verified L164-194):**
```rust
        // Leaf components have no children to recurse into.
        Component::Table(_)
        | Component::Button(_)
        ...
        | Component::Image(_)
        | Component::KeyValueEditor(_) => {}
        Component::KanbanBoard(props) => { ... }
```

**Emission (D-08):** Append `| Component::HtmlEmbed(_)` to the existing OR-chain before the `=> {}`:
```rust
        | Component::Image(_)
        | Component::KeyValueEditor(_)
        | Component::HtmlEmbed(_) => {}   // NEW — appended to tail of existing leaf chain
```

**Critical gotcha:** `Component::HtmlEmbed(_)` goes in the **leaf** chain (no children). `DetailForm` (phase 147) is NOT in this chain because it IS a container — `HtmlEmbed` is genuinely a leaf. Do NOT accidentally add a recursive arm.

---

#### 2d. Render tests

**Analogs:**
- `render_separator_tests` region (verify by grep after read) — minimal render substring assertions.
- Phase 147's `render_detail_form_view_xss_escapes_strings` (inverted) for the XSS passthrough shape.
- `render_to_html(&view, &serde_json::Value::Null)` call idiom appears in every render test.

**Tests to add** (Wave 0 RED; §Code Examples in 148-RESEARCH.md L730-799 gives full bodies):
1. `render_html_embed_verbatim_svg` — construct `HtmlEmbedProps::new("<svg>...</svg>")`, assert rendered output contains `<div><svg>...</svg></div>` byte-for-byte.
2. `render_html_embed_wrapping_div_always_present` — assert `<div>` and `</div>` substrings flank the html field.
3. `render_html_embed_empty_string` — `HtmlEmbedProps::new("")` → `<div></div>` (Pitfall 7 documentation test).
4. `render_html_embed_html_entities_preserved` — assert `&amp;` and `&lt;` in the input pass through as `&amp;` and `&lt;` (verbatim, NOT double-escaped).
5. `render_html_embed_emits_html_verbatim_without_escaping` — **load-bearing** XSS passthrough test. Input `<script>alert('xss')</script>`; assert output CONTAINS `<script>alert('xss')</script>` literally. Include a doc-comment explaining this test documents intent (the bypass contract) and MUST NOT be "fixed" by expecting `&lt;script&gt;`.
6. `render_html_embed_dispatched_via_render_component` — integration via full `render_to_html`; verifies the dispatch arm (2b) is wired.

**Insertion point:** New numbered banner in `mod tests` (follow the `// ── N. Name ──` convention used by form/detail_form/key_value_editor test blocks).

---

### 3. `ferro-json-ui/src/resolve.rs`

Three leaf-group OR-chain additions + three no-op tests. Pure mechanical edit; Rust's exhaustive match enforces correctness at compile time (Pitfall 2).

#### 3a. `resolve_component_node` leaf arm

**Current source (verified L134-161):**
```rust
        // Leaf components with no children or actions to resolve.
        Component::Button(_)
        | Component::Input(_)
        ...
        | Component::Image(_)
        | Component::KeyValueEditor(_)
        | Component::Plugin(_) => {}
    }
}
```

**Emission (D-10):** Insert `| Component::HtmlEmbed(_)` immediately before `Component::Plugin(_)`:
```rust
        | Component::Image(_)
        | Component::KeyValueEditor(_)
        | Component::HtmlEmbed(_)   // NEW — no action, no children
        | Component::Plugin(_) => {}
```

---

#### 3b. `collect_unresolved_node` leaf arm

**Current source (verified L313-338):**
```rust
        Component::Button(_)
        | Component::Input(_)
        ...
        | Component::Image(_)
        | Component::KeyValueEditor(_)
        | Component::Plugin(_) => {}
    }
}
```

**Emission (D-10):** Same shape as 3a — insert before `Component::Plugin(_)`:
```rust
        | Component::KeyValueEditor(_)
        | Component::HtmlEmbed(_)   // NEW
        | Component::Plugin(_) => {}
```

---

#### 3c. `resolve_errors_node` leaf arm

**Current source (verified L462-488):**
```rust
        // Leaf components with no form field semantics.
        Component::Table(_)
        | Component::Button(_)
        ...
        | Component::Image(_)
        | Component::Plugin(_) => {}
        Component::KeyValueEditor(props) => {
            set_field_error(&mut props.error, &props.field, errors, all);
        }
    }
}
```

**Emission (D-10):** Insert `| Component::HtmlEmbed(_)` before `| Component::Plugin(_)`:
```rust
        | Component::Image(_)
        | Component::HtmlEmbed(_)   // NEW — no error field
        | Component::Plugin(_) => {}
        Component::KeyValueEditor(props) => { ... }
```

**Critical gotcha:** The third pass's leaf chain does NOT contain `KeyValueEditor` — KeyValueEditor has its own standalone arm at L489-491 because it DOES have an `error` field. `HtmlEmbed` has no `error` field and belongs in the leaf chain alongside `Image`, `DataTable`, `Plugin`. Do NOT give `HtmlEmbed` a standalone arm (D-11).

---

#### 3d. Resolver tests

**Analogs:** Any existing no-op resolver test pattern (`resolve_form_action` at `resolve.rs:592-633` is the shape for positive-resolution tests; for a no-op leaf, the shape is simpler — build a view, call the resolver, assert the `html` field is unchanged).

**Tests to add** (Wave 0 RED):
1. `resolve_component_node_leaves_html_embed_untouched` — build view with `Component::HtmlEmbed`, call `resolve_actions(&mut view, test_resolver)`, assert `html` field byte-for-byte unchanged.
2. `collect_unresolved_node_ignores_html_embed` — build view with only HtmlEmbed, assert `resolve_actions_strict` returns `Ok(())` (no unresolved actions).
3. `resolve_errors_node_skips_html_embed` — build view with HtmlEmbed + field errors supplied to `resolve_errors`, assert HtmlEmbed's props are unchanged.

**Insertion point:** Inside existing `#[cfg(test)] mod tests` block. Place after existing resolver-test region; following numbered-banner convention if present.

---

### 4. `ferro-json-ui/src/lib.rs`

Two edits: public re-export + `COMPONENT_CATALOG` entry.

#### 4a. Public re-export

**Analog:** The `pub use component::{…}` block at `lib.rs:59-72` — **strictly alphabetical** ordering confirmed by Read.

**Current source (verified L59-72):**
```rust
pub use component::{
    ActionCardProps, ActionCardVariant, AlertProps, AlertVariant, AvatarProps, BadgeProps,
    BadgeVariant, BreadcrumbItem, BreadcrumbProps, ButtonGroupProps, ButtonProps, ButtonType,
    ButtonVariant, CardProps, CheckboxProps, ChecklistItem, ChecklistProps, CollapsibleProps,
    Column, ColumnFormat, Component, ComponentNode, DataTableProps, DescriptionItem,
    DescriptionListProps, DetailField, DetailFormProps, DropdownMenuAction, DropdownMenuProps,
    EditMode, EmptyStateProps, FormMaxWidth, FormProps, FormSectionProps, GapSize, GridProps,
    HeaderProps, IconPosition, ImageProps, InputProps, InputType, KanbanBoardProps,
    ...
};
```

**Emission (alphabetical, between `HeaderProps` and `IconPosition` at L66):**
```
-    HeaderProps, IconPosition, ImageProps, InputProps, KanbanBoardProps,
+    HeaderProps, HtmlEmbedProps, IconPosition, ImageProps, InputProps, KanbanBoardProps,
```

**Why it must ship in the same plan as the struct:** doctest in `HtmlEmbedProps::new` uses `use ferro_json_ui::HtmlEmbedProps` — a missing re-export fails `cargo test --doc` (Pitfall 6).

---

#### 4b. `COMPONENT_CATALOG` entry

**Analog:** `### DetailForm` entry at `lib.rs:120-122` — safety-foregrounded, authoring-rule-inclusive.

**Current source (verified L120-122):**
```
### DetailForm
Props: mode (EditMode: view|edit), action (Action), fields (Vec<DetailField {label, value, input}>), edit_url (String), cancel_url (String), edit_label (Option<String>, default "Modifica"), save_label (Option<String>, default "Salva"), cancel_label (Option<String>, default "Annulla"), method (Option<HttpMethod>)
Split-mode detail page with inline edit: ... Authoring rule (Option A): ...
```

**Emission (D-16):** Safety-first description; foreground the `html_escape` bypass and the "do not pass user input" rule. Full proposed text at `148-RESEARCH.md` §Pattern 8 (L494-499).

Essential content:
1. `Props: html (String)` — one required field.
2. One-sentence function: "Injects pre-rendered HTML verbatim into a wrapping `<div>`."
3. Safety contract: "Renderer does NOT html-escape the html prop — callers are responsible for content safety (XSS)."
4. Intended targets: "server-generated content: inline SVG charts, pre-rendered markdown, static HTML widgets, third-party embed snippets."
5. Explicit non-target: "Do NOT pass user input. For escaped text output, use `Component::Text`."
6. JSON example line.

**Ordering rule:** `COMPONENT_CATALOG` follows a loose family/recency grouping. `### DetailForm` is at L120-122 (insertion order: after `### Form` at L117-118). Place `### HtmlEmbed` either:
- Alphabetically (between `### Header` at L?? and `### Image` at L??) — planner confirms at write time by reading the current file.
- OR adjacent to `### DetailForm` / `### KeyValueEditor` as "recent additions" block — matches CONTEXT.md §specifics.

Either placement satisfies D-16; pick one and be consistent.

---

### 5. `ferro-mcp/src/tools/json_ui_catalog.rs`

Two edits: add `CatalogComponent` to the builder + bump the exhaustive-list assertion 41 → 42 and append `"HtmlEmbed"` to the expected array.

#### 5a. `CatalogComponent` entry

**Analog:** `CatalogComponent { name: "Separator", ... }` at `json_ui_catalog.rs:597-607` — canonical minimal single-prop entry. `prop(...)` helper at L1194-1201.

**Current source (verified L597-607):**
```rust
        CatalogComponent {
            name: "Separator".to_string(),
            description: "Visual divider between content sections.".to_string(),
            props: vec![prop(
                "orientation",
                "Option<Orientation>",
                false,
                "Direction: horizontal (default) or vertical",
            )],
            variants: None,
        },
```

**Emission (D-18):** Same structure — single prop entry with `required: true` (not `false` — `html` is mandatory) and safety-first description. Full proposed entry at `148-RESEARCH.md` §Pattern 10 (L549-560).

Essential shape:
```rust
        CatalogComponent {
            name: "HtmlEmbed".to_string(),
            description: "Injects pre-rendered HTML verbatim into a wrapping <div>. \
                          The renderer does NOT html-escape the `html` prop — this is \
                          the only component in ferro-json-ui that bypasses escaping. \
                          Caller is responsible for content safety (XSS). Intended for \
                          server-generated content: inline SVG charts, pre-rendered \
                          markdown, static HTML widgets. Do NOT pass user input. For \
                          escaped text output, use Component::Text.".to_string(),
            props: vec![prop(
                "html",
                "String",
                true,   // REQUIRED — D-20 applies: do NOT add to no_required allowlist
                "Raw HTML emitted unescaped inside a wrapping <div>. Caller is \
                 responsible for content safety.",
            )],
            variants: None,
        },
```

**Insertion point:** Adjacent to the other recent additions (DetailForm / KeyValueEditor entries). The catalog builder function is NOT alphabetical — loose family grouping per CONTEXT §code_context.

---

#### 5b. Exhaustive-list assertion bump

**Analog:** The `test_all_components_present` function at `json_ui_catalog.rs:1207-1264`.

**Current source (verified L1207-1263):**
```rust
    #[test]
    fn test_all_components_present() {
        let catalog = execute(None);
        assert_eq!(
            catalog.components.len(),
            41,
            "Catalog should contain all 41 built-in components (including DetailForm + KeyValueEditor backfill), got {}",
            catalog.components.len()
        );

        let names: Vec<&str> = catalog.components.iter().map(|c| c.name.as_str()).collect();
        let expected = [
            "Text", "Button", "Card", "Table", "Form", "Input", ...
            "DetailForm",
            "KeyValueEditor",
        ];
        for name in &expected {
            assert!(names.contains(name), "Missing component: {name}");
        }
    }
```

**Three atomic edits in one function (D-19):**
1. Change `41` to `42` at the assertion (currently L1212).
2. Update the comment string to reflect the HtmlEmbed addition — suggested: `"Catalog should contain all 42 built-in components (including HtmlEmbed), got {}"`.
3. Append `"HtmlEmbed",` to the `expected` array (currently ends at L1260 with `"KeyValueEditor",`). Alphabetical or append-to-tail — the array is consumed as an unordered set, so either is correct.

**Do NOT** modify the `no_required` allowlist at L1375 (D-20, Pitfall 4). `html` is required; HtmlEmbed does not join that allowlist.

---

### 6. `docs/src/json-ui/components.md`

New `### HtmlEmbed` section with safety callout.

**Analog (density):** `### Separator` at `docs/src/json-ui/components.md:233-258` — minimal single-prop docs section with props table, Rust example, JSON output example.

**Current source (verified L233-258):**
```markdown
### Separator

Visual divider between content sections.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `orientation` | `Option<Orientation>` | No | `Horizontal` | Direction: `horizontal` or `vertical` |

```rust
use ferro::{ComponentNode, Component, SeparatorProps};

ComponentNode {
    key: "divider".to_string(),
    component: Component::Separator(SeparatorProps { orientation: None }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "divider", "type": "Separator" }
```
```

**Analog (safety-callout precedent):** `### DetailForm` at `docs/src/json-ui/components.md:473+` — uses a **When to use** paragraph and structured tables for more-involved components. Phase 148 reuses its density but substitutes a **Safety** blockquote callout at the top (the first surface a reader sees).

**Required content (D-21):** Match Separator density, adding one extra block — the safety callout. Ordered:

1. Section heading `### HtmlEmbed`.
2. Opening paragraph (1-2 sentences): "Injects pre-rendered HTML verbatim into a wrapping `<div>`. Intended for server-generated content like inline SVG charts or pre-rendered markdown."
3. **Safety callout** (blockquote style `> ⚠️ Warning`, following whichever emoji/wording convention the docs already use — planner greps for existing `⚠️` or `> **Warning`** blocks to match). Content: unescaped emission; caller XSS responsibility; not for user input; pointer to `Component::Text`.
4. Props table — single row: `html` / `String` / Yes / - / Raw HTML emitted unescaped.
5. Rust example using `ComponentNode::html_embed(...)` with `HtmlEmbedProps::new(...)` (shape from `148-RESEARCH.md` §Example 1, L736-755).
6. JSON output example: `{"key": "chart", "type": "HtmlEmbed", "html": "<svg>...</svg>"}` (§Example 2 L759-765).
7. Use-case list (bullet): inline SVG, pre-rendered markdown, static HTML widgets, third-party embed snippets.
8. Closing pointer: "For escaped text output, use [`Text`](#text)."

**Insertion point:** Adjacent to `### DetailForm` / `### KeyValueEditor` (the other recent component additions). Follow whichever ordering convention sibling sections already use.

---

## Shared Patterns

### S-1: Tagged-enum add (three-edit discipline)

**Source:** `component.rs:1104-1117` (`serialize_tagged` helper); `:1119-1167` (Serialize impl); `:1172-1304` (Deserialize impl); enum declaration `:1055-1098`.

**Apply to:** `Component::HtmlEmbed`.

**Rule:** Adding a new `Component` variant requires **three edits in `component.rs`**, all using the same string literal for the type tag:
1. Add variant to the enum declaration, Plugin-always-last (§1b).
2. Add `Component::{Name}(p) => serialize_tagged(serializer, "{Name}", p),` Serialize arm (§1c).
3. Add `"{Name}" => serde_json::from_value::<{Name}Props>(value).map(Component::{Name}).map_err(de::Error::custom),` Deserialize arm before the `_ =>` plugin fallback (§1d).

All three use the **same** string `"HtmlEmbed"` — a typo in one fails serde round-trip silently, caught by the `all_known_types_round_trip` fixture (§1f).

---

### S-2: Three-pass resolver participation (leaf-group OR-chain)

**Source:** `resolve.rs:134-161`, `:313-338`, `:462-488`.

**Apply to:** `Component::HtmlEmbed(_)` in all three passes.

**Rule:** Leaf variants (no action, no children, no error slot) join the existing OR-chain ending in `=> {}` — **never** a standalone arm (D-11). The three matches are exhaustive — missing an arm is a compile error, not a silent pass-through (Pitfall 2). This is a feature: the compiler enforces the three-edit discipline.

Note the third pass's leaf chain is narrower than the first two — it excludes `Input`/`Select`/`Checkbox`/`Switch`/`KeyValueEditor` (which have field-error arms). `HtmlEmbed` joins alongside `Image` / `DataTable` / `Plugin` in all three chains.

---

### S-3: The `html_escape` exception (load-bearing)

**Source:** `render.rs:2917` — `pub(crate) fn html_escape`, applied by every existing `render_*` to dynamic content.

**Apply to:** **Only** `render_html_embed` — the **one** function in the file that does NOT call it.

**Rule:** The asymmetry is the entire purpose of the component. Protect it via four concurrent signals:
1. **Rustdoc on `HtmlEmbedProps`** (§1a) — contract stated at type-definition site.
2. **Rustdoc on `render_html_embed`** (§2a) — repeats the contract at the rendering site.
3. **Inline `// SAFETY CONTRACT` comment inside the function body** (§2a) — prevents a drive-by "fix" that adds `html_escape` without reading the docs.
4. **Load-bearing XSS passthrough test** `render_html_embed_emits_html_verbatim_without_escaping` (§2d, test #5) — asserts `<script>alert('xss')</script>` round-trips literally, with a test-body comment explaining it documents intent (not a smell).

Additionally, every other user-touchable surface (`COMPONENT_CATALOG` string, MCP catalog description, docs chapter) foregrounds the bypass in its description. Five concurrent surfaces → impossible to miss.

---

### S-4: Naming / casing / style conventions

**Source:** all existing `*Props` structs.

**Apply to:** `HtmlEmbedProps`, `Component::HtmlEmbed`, `ComponentNode::html_embed`, serde tag `"HtmlEmbed"`.

**Rule:** PascalCase for types, snake_case for factory functions, exact-case-preserving string tags for serde. The name `HtmlEmbed` (two words, PascalCased with internal capital `E`) is fixed by ROADMAP; everything flows from it.

---

## No Analog Found

**None.** Every file modified and every edit has a verified in-crate precedent.

The **one** deliberate divergence from analogs is the omission of `html_escape` in `render_html_embed` (§2a). That divergence is:
- The component's entire purpose (no analog possible by construction).
- Documented in five concurrent surfaces (S-3).
- Guarded by a dedicated load-bearing test (§2d test #5).
- Compile-enforced by Rust's exhaustive match on `Component` for all three resolver arms (Pitfall 2).

The phase has no greenfield design decisions. It is copy-paste with one explicit, well-documented omission.

---

## Insertion-Point Ordering Summary

| File | Section | Ordering rule | HtmlEmbed insertion |
|------|---------|---------------|---------------------|
| `component.rs` | `Component` enum (L1055-1098) | recency-grouped, `Plugin` last | after `DetailForm` (L1096), before `Plugin` (L1097) |
| `component.rs` | `Serialize` impl (L1119-1167) | matches enum order | after `DetailForm` arm (L1164), before `Plugin` (L1165) |
| `component.rs` | `Deserialize` impl (L1172-1304) | matches enum order | after `"DetailForm"` (L1301-1303), before `_ =>` (L1304) |
| `component.rs` | struct definitions | leaf-props neighborhood | near `SeparatorProps` (L524-530) — write-time cosmetic |
| `component.rs` | `ComponentNode` factories | family grouping, recency at tail | near `::separator` (L1478-1486) OR near recent additions — write-time cosmetic |
| `component.rs` | `all_known_types_round_trip` fixture (L3710-3750) | unordered set (consumed in a for-loop) | append tuple; alphabetical or tail position |
| `render.rs` | `render_component` dispatch (L294-322) | family comment-banner groups | simple-leaf cluster, after `Separator` (L300) or near it |
| `render.rs` | `collect_plugin_types_node` leaf chain (L164-194) | unordered OR-chain ending in `=> {}` | append before `=> {}` |
| `render.rs` | function definitions | recency | `fn render_html_embed` near `render_separator` (L2359) |
| `render.rs` | `mod tests` | numbered banner sections | new numbered banner |
| `resolve.rs` | `resolve_component_node` leaf chain (L134-161) | OR-chain ending in `Plugin(_) => {}` | insert `HtmlEmbed(_)` before `Plugin(_)` |
| `resolve.rs` | `collect_unresolved_node` leaf chain (L313-338) | same | same |
| `resolve.rs` | `resolve_errors_node` leaf chain (L462-488) | same (narrower chain) | insert before `Plugin(_)`; NOT in `KeyValueEditor` arm |
| `lib.rs` | `pub use component::{…}` (L59-72) | **strictly alphabetical** | `HtmlEmbedProps` between `HeaderProps` and `IconPosition` |
| `lib.rs` | `COMPONENT_CATALOG` | loose family/recency grouping | near `### DetailForm` (L120-122) OR alphabetical — match existing |
| `ferro-mcp/.../json_ui_catalog.rs` | `execute(None)` builder | loose family grouping | near DetailForm / KeyValueEditor entries |
| `ferro-mcp/.../json_ui_catalog.rs` | `test_all_components_present` (L1207-1264) | three atomic sub-edits | bump `41 → 42` (L1212); update comment (L1213); append `"HtmlEmbed",` to expected (L1218-1260) |
| `docs/src/json-ui/components.md` | `###` sections | family/recency grouping | near `### DetailForm` (L473+) OR alphabetical |

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/`, `ferro-mcp/src/tools/`, `docs/src/json-ui/`
**Files read (verified line numbers):** `component.rs` (~3900 lines), `render.rs` (dispatch / leaf-walk / render_separator regions), `resolve.rs` (three leaf chains), `lib.rs` (re-export + COMPONENT_CATALOG prefix), `json_ui_catalog.rs` (Separator entry + test_all_components_present + no_required allowlist), `components.md` (Separator + DetailForm sections)
**Source-of-truth code excerpts:** See `148-RESEARCH.md` §Architecture Patterns (Patterns 1-10) for full code blocks. This PATTERNS.md points to them rather than duplicating.
**Pattern extraction date:** 2026-04-24
**Precedent:** Phase 147 DetailForm. Phase 148 is structurally simpler: no container recursion, no action, no error slot, no runtime JS, no field-error arm. The one axis where phase 148 is more exacting is **safety messaging**, which must appear in five concurrent surfaces (S-3).
