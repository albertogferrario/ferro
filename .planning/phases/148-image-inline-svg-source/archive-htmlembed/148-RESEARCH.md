# Phase 148: HtmlEmbed component for ferro-json-ui — unescaped HTML injection — Research

**Researched:** 2026-04-24
**Domain:** ferro-json-ui component system — Rust HTML rendering, deliberate escape-bypass for server-generated content
**Confidence:** HIGH

## Summary

Phase 148 is the mechanically simplest component-addition in recent history: a single-field `HtmlEmbed` leaf
with no data binding, no children, no action, no runtime JS, and no default styling. The entire change is
six file edits following the established 42nd-component playbook (Separator / Image are the nearest shape
peers). One thing distinguishes this phase from its 41 predecessors: the renderer is the **only** function
in `render.rs` that does not call `html_escape` on a dynamic string. That asymmetry is the whole point of
the component — and it is the single load-bearing invariant every artifact in this phase must protect.

The research below supplements the locked decisions in `148-CONTEXT.md` with (1) concrete code-pattern
excerpts lifted from the current tree at verified line numbers, (2) exact substring-assertion test bodies
for the render tests, (3) a load-bearing XSS passthrough test that documents intent rather than suggesting
a bug, (4) resolver no-op tests that prove the leaf groups skip `HtmlEmbed` unchanged, and (5) an
8-dimension Nyquist validation plan mapped to the five requirements EMBED-01..EMBED-05. Nothing here
reopens locked decisions; everything here tells the planner HOW.

**Primary recommendation:** Follow the phase 146/147 wave structure exactly — Wave 0 RED tests in
`component.rs` (serde + factory), `render.rs` (verbatim emission, XSS passthrough, wrapping-div shape,
empty string), `resolve.rs` (three no-op assertions), and `ferro-mcp/src/tools/json_ui_catalog.rs`
(exhaustive-list bump 41 → 42, `"HtmlEmbed"` appended, `no_required` allowlist unchanged). Wave 1 splits
across the same five implementation axes (types / renderer / resolver / catalog+docs / CI gate) as phase
147. The planner MAY consolidate into 3 plans since there is no runtime JS wave and no resolver logic
beyond the OR-chain arm. A reasonable minimum is `148-01` Wave 0 RED, `148-02` Rust impl
(component.rs + render.rs + resolve.rs + lib.rs), `148-03` MCP catalog + docs + CI gate.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Core type shape:**
- **D-01** — `HtmlEmbedProps` is a named struct with derives `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema` and a single field `pub html: String`.
- **D-02** — `Component::HtmlEmbed(HtmlEmbedProps)` variant added to the `Component` enum. Inserted after `Component::KeyValueEditor(...)` and `Component::DetailForm(...)`, before `Plugin`.
- **D-03** — `HtmlEmbedProps::new(html: impl Into<String>) -> Self` convenience constructor.
- **D-04** — `ComponentNode::html_embed(key: impl Into<String>, props: HtmlEmbedProps) -> Self` factory. NO shorthand collapse to `(key, html)`.

**Rendering:**
- **D-05** — `fn render_html_embed(props: &HtmlEmbedProps) -> String` — takes `&HtmlEmbedProps` only (no `data: &Value`).
- **D-06** — Output: `<div>{props.html}</div>` — emitted VERBATIM without `html_escape`. No default class, id, or attributes.
- **D-07** — Dispatch arm in `render_component` in `ferro-json-ui/src/render.rs` near the simple leaf group.
- **D-08** — Leaf arm in `collect_plugin_types_node` (alphabetically placed among leaf components).
- **D-09** — Function is module-private (no `pub` modifier).

**Resolver participation:**
- **D-10** — `Component::HtmlEmbed(_)` joins the leaf OR-chain in all three passes: `resolve_component_node`, `collect_unresolved_node`, `resolve_errors_node`.
- **D-11** — No standalone arms — grouped OR-chain only.

**Serde:**
- **D-12** — `Component::Serialize`: add `Component::HtmlEmbed(p) => serialize_tagged(serializer, "HtmlEmbed", p)`.
- **D-13** — `Component::Deserialize`: add `"HtmlEmbed"` arm returning `HtmlEmbedProps` → `Component::HtmlEmbed`.
- **D-14** — JSON wire format: `{"type": "HtmlEmbed", "html": "<svg>...</svg>"}` — `key` belongs to `ComponentNode`, not props.

**Security framing — load-bearing:**
- **D-15** — Rustdoc on `HtmlEmbedProps` leads with a safety warning covering: (a) HTML emitted verbatim without escaping; (b) caller responsible for XSS safety; (c) intended for server-generated content, NOT user input; (d) one-line pointer to `Component::Text` for escaped output.
- **D-16** — `COMPONENT_CATALOG` string description foregrounds the safety contract: unescaped output, caller-owned safety, do-not-pass-user-input.
- **D-17** — MCP `CatalogComponent` description uses the same safety-first phrasing (agents must see the asymmetry).

**MCP catalog & exhaustive-list assertion:**
- **D-18** — Add `CatalogComponent { name: "HtmlEmbed", description: "...", props: vec![prop("html", "String", true, "...")], variants: None }` adjacent to DetailForm / KeyValueEditor entries.
- **D-19** — Bump `ferro-mcp/src/tools/json_ui_catalog.rs:1212` assertion `41` → `42` and update its comment; append `"HtmlEmbed"` to the `expected` array near `"DetailForm"` / `"KeyValueEditor"` entries.
- **D-20** — `HtmlEmbed` has one required prop — do NOT add to the `no_required` allowlist at line 1375.

**Docs:**
- **D-21** — New `### HtmlEmbed` section in `docs/src/json-ui/components.md` with: opening paragraph, safety callout (blockquote/warning block), props table, Rust example (`ComponentNode::html_embed(...)` + `HtmlEmbedProps::new(...)`), JSON example, use-case list, "for escaped text, use `Text`" pointer.

**TDD wave structure:**
- **D-22** — Wave 0 RED tests precede Wave 1 implementation.
- **D-23** — Plan split may follow phase 147's five-plan pattern or consolidate to 3 plans. No runtime wave.
- **D-24** — No runtime JS; no entry in `ferro-json-ui/src/runtime/`.

### Claude's Discretion

- Exact Rustdoc wording for safety warning (satisfies D-15 content; prose follows `InputProps`/`FormProps` style).
- Exact MCP catalog description text (satisfies D-16/D-17; phrasing is authorial).
- Alphabetical vs. recency-grouped ordering for new variants/factory/catalog entries — match whatever local ordering already exists.
- Emit an XSS passthrough test (`<script>alert('xss')</script>`) that documents intent — **recommended: yes** (the one case where such a test is a contract, not a smell).
- Exact wording/emoji on docs safety callout — follow existing docs site conventions.
- `HtmlEmbedProps` `Default` derive — **recommended: no** (empty `html` is a semantic foot-gun).

### Deferred Ideas (OUT OF SCOPE)

- `class` / `id` / `style` props on the wrapper `<div>`.
- Configurable `wrapper_tag` (`<span>`, `<section>`).
- `data_path: Option<String>` binding.
- Built-in sanitization opt-in (`sanitize: Option<bool>`).
- Plugin-style HtmlEmbed variants (`HtmlEmbedIframe`, etc.).
- Markdown-aware sibling component (`MarkdownEmbed`).
- Framework-level `#[warning("unescaped")]` marker / clippy lint.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| EMBED-01 | `Component::HtmlEmbed` variant in `Component` enum (implementation: `Component::HtmlEmbed(HtmlEmbedProps)` per D-01/D-02) | Canonical variant-insertion + serde arms pattern at `ferro-json-ui/src/component.rs:1055-1098` (enum), `:1119-1167` (Serialize), `:1172-1314` (Deserialize). Precedent: phase 147 DetailForm insertion. |
| EMBED-02 | Renderer bypasses `html_escape` | `render_separator` (`render.rs:2359-2365`) is the minimal render-function precedent; `render_html_embed` copies that shape but OMITS the `html_escape` call on `props.html`. Inline comment flags the deliberate omission. XSS passthrough test documents contract. |
| EMBED-03 | Resolver participates in all three passes (no-op) | `Component::HtmlEmbed(_)` joins the existing leaf OR-chains at `resolve.rs:135-160` / `:313-338` / `:462-488`. No standalone arms. Three no-op resolver tests prove the passthrough contract. |
| EMBED-04 | MCP catalog entry | `CatalogComponent` entry added adjacent to `DetailForm` (`json_ui_catalog.rs:252-312`) / `KeyValueEditor` (`:313-`). Exhaustive-list assertion (`json_ui_catalog.rs:1207-1263`) bumped 41 → 42 and `"HtmlEmbed"` appended to the `expected` array. |
| EMBED-05 | CI gate green | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` — same gate as every prior phase. Wave 1 gate hit after all implementation edits merge. |

</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `HtmlEmbedProps` struct + serde derives + safety rustdoc | Rust library (`ferro-json-ui/src/component.rs`) | — | All component schemas live in this file |
| `Component::HtmlEmbed` variant + Serialize/Deserialize arms | Rust library (`component.rs`) | — | Tagged-enum dispatch lives with the enum declaration |
| `ComponentNode::html_embed(...)` factory | Rust library (`component.rs`) | — | Parallel placement to `ComponentNode::separator`, `::image` |
| `render_html_embed(props)` renderer (the ONE html_escape bypass) | Rust library (`ferro-json-ui/src/render.rs`) | — | Server-side HTML generation, dispatched from `render_component` |
| Resolver leaf-group participation (3 passes, no-op) | Rust library (`ferro-json-ui/src/resolve.rs`) | — | Leaf components have no action, no children, no field error semantics — OR-chain pattern |
| Plugin-type collection walk (leaf arm) | Rust library (`render.rs`) | — | `collect_plugin_types_node` leaf group |
| Public API re-export + `COMPONENT_CATALOG` string entry | Rust library (`ferro-json-ui/src/lib.rs`) | — | User-facing types re-exported from crate root |
| MCP `CatalogComponent` entry + exhaustive-list assertion bump | `ferro-mcp/src/tools/json_ui_catalog.rs` | — | Agents introspect catalog via MCP; exhaustive test enforces coverage |
| User documentation (`### HtmlEmbed` section) | `docs/src/json-ui/components.md` | — | Every new component gets a section matching existing template |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.0 | Props serialization/deserialization | Used by every component in the crate [VERIFIED: existing `Serialize, Deserialize` derives on all *Props structs] |
| serde_json | 1.0 | JSON round-trip tests + `serialize_tagged` helper | Already a direct dependency |
| schemars | 1.x | `JsonSchema` derive on `HtmlEmbedProps` | Already used by `SeparatorProps`, `ImageProps`, all other simple props |

**No new dependencies needed.** The entire phase reuses existing infrastructure.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `<div>{html}</div>` output shape | `<span>{html}</span>` / `<section>{html}</section>` | Deferred per D-06 — callers wrap in parent components when inline is needed |
| `HtmlEmbedProps::new(...)` constructor | Only literal struct init | D-03 mirrors existing ergonomic helpers (`InputProps::new`, `ButtonProps::new`) — one-line compact call sites |
| Passing `data: &Value` to renderer | No data param | D-05 — `HtmlEmbed` has no data-binding surface in v1; skip the param |
| Default derive on `HtmlEmbedProps` | `#[derive(Default)]` | Claude's Discretion resolves to NO — empty `html` is a foot-gun; explicit construction via `new()` |

---

## Architecture Patterns

### System Architecture Diagram

```
Caller (handler / view builder):
    let chart_svg: String = generate_chart(&data);
    ComponentNode::html_embed("my-chart", HtmlEmbedProps::new(chart_svg))
              │
              ▼
Serde serialize (when dumping JSON spec):
    {"type": "HtmlEmbed", "key": "my-chart", "html": "<svg>...</svg>"}
              │
              ▼
resolve_actions(view, resolver):
    resolve_component_node      → Component::HtmlEmbed(_) => {}  (no-op leaf)
    collect_unresolved_node     → Component::HtmlEmbed(_) => {}  (no-op leaf)
    resolve_errors_node         → Component::HtmlEmbed(_) => {}  (no-op leaf)
              │
              ▼
render_to_html(view, data):
    render_component(Component::HtmlEmbed(props), data)
         └── render_html_embed(props)
              │
              ▼
         format!("<div>{}</div>", props.html)   ← NO html_escape call
              │
              ▼
    Raw HTML emitted verbatim inside the wrapping <div>.
    Caller's SVG / markdown / widget is rendered by the browser.
```

### Files Touched

```
ferro-json-ui/src/
├── component.rs     # +HtmlEmbedProps struct (with safety rustdoc + new()),
│                    # +Component::HtmlEmbed variant,
│                    # +Serialize arm (L1163-area),
│                    # +Deserialize arm (L1300-area, near KeyValueEditor/DetailForm),
│                    # +ComponentNode::html_embed factory,
│                    # +serde round-trip + factory tests in mod tests
├── render.rs        # +fn render_html_embed (minimal, explicit no-escape inline comment),
│                    # +dispatch arm in render_component (L294-322 region),
│                    # +Component::HtmlEmbed(_) appended to leaf OR-chain in
│                    #   collect_plugin_types_node (L163-193 region),
│                    # +render tests (verbatim emission, wrapping-div, empty, XSS passthrough)
├── lib.rs           # +HtmlEmbedProps in component re-export block (L59-72),
│                    # +### HtmlEmbed entry in COMPONENT_CATALOG string
└── resolve.rs       # +Component::HtmlEmbed(_) appended to leaf OR-chain in
                     #   resolve_component_node (L135-160),
                     # +same in collect_unresolved_node (L313-338),
                     # +same in resolve_errors_node (L462-488),
                     # +three no-op resolver tests

ferro-mcp/src/tools/
└── json_ui_catalog.rs  # +CatalogComponent entry (name/description/props),
                        # +bump assertion 41 → 42 at L1212 + updated comment,
                        # +"HtmlEmbed" appended to expected[] at L1218-1260

docs/src/json-ui/
└── components.md    # +### HtmlEmbed section with safety callout,
                     #   props table, Rust example, JSON example, use cases,
                     #   explicit "for escaped text, use Text" pointer
```

### Pattern 1: Minimal single-field Props struct

**Source:** `ferro-json-ui/src/component.rs:524-530` (`SeparatorProps`).

```rust
/// Props for Separator component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeparatorProps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
}
```

**For `HtmlEmbedProps`** — add safety-first rustdoc, drop `Eq` (String is not `Eq`-derivable usefully here; `PartialEq` suffices — matches `ImageProps`, `TextProps`, etc.):

```rust
/// Props for HtmlEmbed component.
///
/// ⚠️ **Safety contract.** `html` is emitted **verbatim** into the page without
/// HTML-escaping. The caller is responsible for ensuring the content is safe
/// against XSS. Intended for server-generated content — inline SVG, pre-rendered
/// markdown, static HTML widgets, third-party embed snippets. **Never pass user
/// input directly into this component.** For escaped text output, use
/// [`Component::Text`] instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HtmlEmbedProps {
    /// Raw HTML string. Emitted verbatim inside a wrapping `<div>`. NOT escaped.
    pub html: String,
}

impl HtmlEmbedProps {
    /// Create `HtmlEmbedProps` from any string-convertible value.
    ///
    /// ```
    /// # use ferro_json_ui::HtmlEmbedProps;
    /// let props = HtmlEmbedProps::new("<svg>...</svg>");
    /// ```
    pub fn new(html: impl Into<String>) -> Self {
        Self { html: html.into() }
    }
}
```

### Pattern 2: Enum variant + serde arm insertion

**Sources:**
- `ferro-json-ui/src/component.rs:1055-1098` — enum declaration (`DetailForm` is at L1096, `Plugin` at L1097).
- `ferro-json-ui/src/component.rs:1119-1167` — `Serialize` arms (DetailForm at L1164).
- `ferro-json-ui/src/component.rs:1172-1314` — `Deserialize` arms (DetailForm at L1301-1303).

**Enum insertion** (after `DetailForm`, before `Plugin`):

```rust
pub enum Component {
    // … existing variants …
    Image(ImageProps),
    KeyValueEditor(KeyValueEditorProps),
    DetailForm(DetailFormProps),
    HtmlEmbed(HtmlEmbedProps),   // NEW
    Plugin(PluginProps),
}
```

**Serialize arm** (after the `DetailForm` arm at L1164):

```rust
Component::DetailForm(p) => serialize_tagged(serializer, "DetailForm", p),
Component::HtmlEmbed(p) => serialize_tagged(serializer, "HtmlEmbed", p),   // NEW
Component::Plugin(p) => p.serialize(serializer),
```

**Deserialize arm** (after the `"DetailForm"` arm at L1301-1303, before the `_ => Plugin` catch-all at L1304):

```rust
"DetailForm" => serde_json::from_value::<DetailFormProps>(value)
    .map(Component::DetailForm)
    .map_err(de::Error::custom),
"HtmlEmbed" => serde_json::from_value::<HtmlEmbedProps>(value)
    .map(Component::HtmlEmbed)
    .map_err(de::Error::custom),
_ => {
    // Unknown type: treat as a plugin component.
    …
}
```

### Pattern 3: `ComponentNode::<factory>` constructor

**Source:** `ferro-json-ui/src/component.rs:1478-1486` (`ComponentNode::separator`).

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

**For `ComponentNode::html_embed`** — insert near the other recent additions (after `detail_form` at L1379-1386), with safety-restating rustdoc:

```rust
/// Create an HtmlEmbed component node.
///
/// ⚠️ **Safety.** The HTML in `props.html` is emitted verbatim. Do not pass
/// user input. See [`HtmlEmbedProps`] for the full safety contract.
pub fn html_embed(key: impl Into<String>, props: HtmlEmbedProps) -> Self {
    Self {
        key: key.into(),
        component: Component::HtmlEmbed(props),
        action: None,
        visibility: None,
    }
}
```

### Pattern 4: Minimal render function (the html_escape exception)

**Source pattern:** `ferro-json-ui/src/render.rs:2359-2365` (`render_separator`).

```rust
fn render_separator(props: &SeparatorProps) -> String {
    let orientation = props.orientation.as_ref().cloned().unwrap_or_default();
    match orientation {
        Orientation::Horizontal => "<hr class=\"my-4 border-border\">".to_string(),
        Orientation::Vertical => "<div class=\"mx-4 h-full w-px bg-border\"></div>".to_string(),
    }
}
```

**For `render_html_embed`** — even simpler; one format! with a prominent inline comment flagging
the deliberate `html_escape` omission so a future audit does not "fix" it:

```rust
/// Render an `HtmlEmbed` component.
///
/// Emits `<div>{props.html}</div>` with the `html` string written **verbatim**.
/// This is the only function in `render.rs` that does NOT pass dynamic content
/// through [`html_escape`]. The bypass is the component's entire purpose;
/// callers of [`HtmlEmbedProps`] accept responsibility for content safety.
fn render_html_embed(props: &HtmlEmbedProps) -> String {
    // SAFETY CONTRACT (do not "fix" by adding html_escape here):
    // HtmlEmbed exists precisely to inject caller-provided HTML without
    // escaping (e.g. server-generated SVG, pre-rendered markdown). Escaping
    // would defeat the component. See HtmlEmbedProps rustdoc.
    format!("<div>{}</div>", props.html)
}
```

Place near `render_separator` (L2359) in the simple-leaf render-function cluster, or alphabetically —
match whatever ordering discipline the file's maintainer prefers on the day of the edit. Placement
is cosmetic; correctness is not affected.

### Pattern 5: Dispatch arm in `render_component`

**Source:** `ferro-json-ui/src/render.rs:294-322`.

```rust
fn render_component(component: &Component, data: &Value) -> String {
    match component {
        Component::Text(props) => render_text(props),
        Component::Button(props) => render_button(props),
        // … existing leaf arms …
        Component::Separator(props) => render_separator(props),
        // … existing …
        Component::DescriptionList(props) => render_description_list(props),

        // Container components.
        Component::Card(props) => render_card(props, data),
        Component::Form(props) => render_form(props, data),
        Component::DetailForm(props) => render_detail_form(props, data),
        // …
```

**Insertion** — DetailForm is at L311 per current source; `HtmlEmbed` is a leaf (no `data` param),
so it belongs in the simple-leaf arm region near `Separator` / `DescriptionList`:

```rust
Component::Separator(props) => render_separator(props),
// … existing …
Component::HtmlEmbed(props) => render_html_embed(props),   // NEW — simple leaf, no data arg
// … existing …
```

Alphabetical placement within the leaf cluster is fine. Do NOT add `data` — the renderer signature
is `fn render_html_embed(props: &HtmlEmbedProps) -> String` (no `data: &Value`).

### Pattern 6: Leaf arm in `collect_plugin_types_node`

**Source:** `ferro-json-ui/src/render.rs:163-194`.

```rust
// Leaf components have no children to recurse into.
Component::Table(_)
| Component::Button(_)
| Component::Input(_)
// … existing …
| Component::Image(_)
| Component::KeyValueEditor(_) => {}
```

**Insertion** — append `HtmlEmbed` to the existing OR-chain. Alphabetically near `Image` or appended at
the end; convention at `L193-194` groups phase 146/147 additions at the tail:

```rust
| Component::Image(_)
| Component::KeyValueEditor(_)
| Component::HtmlEmbed(_) => {}   // NEW
```

Note: `DetailForm` is NOT in this leaf list — it is a container with its own arm at L119-123.
`HtmlEmbed` is genuinely a leaf (no children).

### Pattern 7: Resolver leaf-group participation (three passes)

**Sources:**
- `ferro-json-ui/src/resolve.rs:134-161` (`resolve_component_node` leaf OR-chain).
- `ferro-json-ui/src/resolve.rs:313-338` (`collect_unresolved_node` leaf OR-chain).
- `ferro-json-ui/src/resolve.rs:461-488` (`resolve_errors_node` leaf OR-chain).

Current shape at `resolve_component_node` (L134-160):

```rust
// Leaf components with no children or actions to resolve.
Component::Button(_)
| Component::Input(_)
| Component::Select(_)
// … existing …
| Component::Image(_)
| Component::KeyValueEditor(_)
| Component::Plugin(_) => {}
```

**Insertion** — append `Component::HtmlEmbed(_)` immediately before `Component::Plugin(_)` in all three
passes (DetailForm is handled by a dedicated non-leaf arm earlier in each match, so it does not appear
in these leaf chains). The third pass (`resolve_errors_node`) at L461-488 does not include `Input` /
`Select` / `Checkbox` / `Switch` in its leaf list (those have their own arms for field-error mapping);
`HtmlEmbed` joins that chain alongside `Image`, `DataTable`, `Plugin`:

```rust
// resolve_component_node — L158-161 (before Plugin):
| Component::Image(_)
| Component::KeyValueEditor(_)
| Component::HtmlEmbed(_)   // NEW
| Component::Plugin(_) => {}

// collect_unresolved_node — L336-338 (before Plugin):
| Component::Image(_)
| Component::KeyValueEditor(_)
| Component::HtmlEmbed(_)   // NEW
| Component::Plugin(_) => {}

// resolve_errors_node — L486-488 (before Plugin):
| Component::Image(_)
| Component::HtmlEmbed(_)   // NEW — no field error surface (no `field` / `error` props)
| Component::Plugin(_) => {}
```

**Gotcha:** the third pass (`resolve_errors_node`) does NOT contain `KeyValueEditor` in its leaf
chain — KeyValueEditor has a dedicated arm at L489-491 because it DOES have an `error` field.
`HtmlEmbed` has no `error` field, so the leaf chain is the right home.

### Pattern 8: COMPONENT_CATALOG entry (lib.rs)

**Source:** `ferro-json-ui/src/lib.rs:120-122` (DetailForm), `:145-147` (KeyValueEditor), `:149-150` (Separator).

DetailForm/KeyValueEditor entries are 2-4 lines with safety / authoring notes. Separator is 2 lines
minimal. `HtmlEmbed` follows the safety-foregrounded shape:

```
### HtmlEmbed
Props: html (String)
Injects pre-rendered HTML verbatim into a wrapping <div>. Renderer does NOT html-escape the html prop — callers are responsible for content safety (XSS). Intended for server-generated content: inline SVG charts, pre-rendered markdown, static HTML widgets, third-party embed snippets. Do NOT pass user input. For escaped text output, use Component::Text.
Example JSON: {"type": "HtmlEmbed", "html": "<svg width=\"100\" height=\"100\"><circle cx=\"50\" cy=\"50\" r=\"40\" fill=\"red\"/></svg>"}
```

Placement: insert in alphabetical order or adjacent to `DetailForm` / `KeyValueEditor` (the other
recent additions); reconcile at write time by matching the existing local ordering discipline.

### Pattern 9: Public re-export in lib.rs

**Source:** `ferro-json-ui/src/lib.rs:59-72`.

```rust
pub use component::{
    ActionCardProps, …,
    DescriptionListProps, DetailField, DetailFormProps, …,
    HeaderProps, IconPosition, ImageProps, …,
    KeyValueEditorProps, …,
    SeparatorProps, …,
};
```

**Insertion** — add `HtmlEmbedProps` in alphabetical position between `HeaderProps` and `IconPosition`:

```rust
pub use component::{
    …,
    HeaderProps, HtmlEmbedProps, IconPosition, ImageProps, InputProps, …,
};
```

### Pattern 10: MCP CatalogComponent entry

**Source:** `ferro-mcp/src/tools/json_ui_catalog.rs:597-607` (Separator — minimal single-prop entry).

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

**For `HtmlEmbed`** — placement adjacent to `DetailForm` (L252-312) and `KeyValueEditor` (L313+)
entries; description is safety-first prose; `variants: None`:

```rust
CatalogComponent {
    name: "HtmlEmbed".to_string(),
    description: "Injects pre-rendered HTML verbatim into a wrapping <div>. The renderer does NOT html-escape the `html` prop — this is the only component in ferro-json-ui that bypasses escaping. Caller is responsible for content safety (XSS). Intended for server-generated content: inline SVG charts, pre-rendered markdown, static HTML widgets, third-party embed snippets. Do NOT pass user input. For escaped text output, use Component::Text.".to_string(),
    props: vec![prop(
        "html",
        "String",
        true,
        "Raw HTML emitted unescaped inside a wrapping <div>. Caller is responsible for content safety.",
    )],
    variants: None,
},
```

### Anti-Patterns to Avoid

- **Do not** call `html_escape(&props.html)` inside `render_html_embed`. The entire phase exists to
  bypass escaping. The inline comment in `render_html_embed` must make this unmistakable.
- **Do not** add a default wrapping-div class (e.g. `<div class="html-embed">`). D-06 is explicit:
  no default class, id, or attribute. Callers style via surrounding components (Card, Grid, FormSection).
- **Do not** introduce a `data_path` / data-binding parameter in `render_html_embed`'s signature.
  D-05 locks the signature to `fn render_html_embed(props: &HtmlEmbedProps) -> String` (no `data: &Value`).
- **Do not** place `Component::HtmlEmbed(_)` as a standalone arm anywhere. D-11 requires the grouped
  OR-chain pattern for leaves. A standalone `=> {}` would be wrong-style.
- **Do not** add `HtmlEmbed` to the `no_required` allowlist at `json_ui_catalog.rs:1375`. `HtmlEmbed`
  has one required prop (`html`); the allowlist is for components with zero required props.
- **Do not** skip the inline "deliberate bypass" comment in `render_html_embed`'s body. Without it,
  a future reader (human or agent) will flag the missing `html_escape` as a bug and "fix" it.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| String construction for `<div>{html}</div>` | `String::with_capacity` + push_str chain | `format!("<div>{}</div>", props.html)` | Same performance in release (LLVM constant-fold), readable, matches `render_separator` |
| Testing substring presence in rendered HTML | Parse HTML, traverse DOM | `html.contains("<div>...</div>")` (substring assertion) | Existing `render.rs` test convention; fast; catches the invariant cheaply |
| Round-tripping `Component::HtmlEmbed` | Hand-crafted JSON string comparison | `serde_json::to_value` + `from_value` + `PartialEq` equality on the Component | Exact pattern at `component.rs:2510-2529` (Separator) and phase 146/147 tests |
| Verifying resolver does not touch `HtmlEmbed` | Read resolver source | Build a view containing `Component::HtmlEmbed`, call `resolve_actions(...)`, assert the `html` field is unchanged byte-for-byte | One test per pass; three tests total; zero new infrastructure |

**Key insight:** Every pattern in this phase already exists in the crate. Nothing new needs to be
invented. The planner's job is copy-paste with the one deliberate omission (`html_escape`) clearly
documented.

---

## Runtime State Inventory

> Not applicable — Phase 148 is a greenfield component addition with no rename, migration, or refactor
> of existing state. No runtime system stores "HtmlEmbed" as a key, id, or configuration entry prior
> to this phase.

---

## Environment Availability

> Skipped — Phase 148 is code-only. No external tools, services, runtimes, or CLI utilities beyond
> the existing Rust workspace toolchain are required.

---

## Common Pitfalls

### Pitfall 1: Adding `html_escape` by reflex

**What goes wrong:** A future edit "hardens" `render_html_embed` by wrapping `props.html` in
`html_escape(...)`. The visible symptom: inline SVG embeds render as literal text
(`&lt;svg&gt;...&lt;/svg&gt;`) instead of graphics. The test suite catches this instantly
via the XSS passthrough test — but only if that test exists.

**Root cause:** Every other `render_*` function in the file applies `html_escape` to dynamic strings;
the pattern is so consistent that omission looks like a bug.

**Prevention:**
1. The inline comment in `render_html_embed` explicitly flagging the deliberate omission.
2. The XSS passthrough test that asserts `<script>alert('xss')</script>` appears verbatim in the
   rendered output (NOT escaped to `&lt;script&gt;`).
3. The rustdoc on `HtmlEmbedProps` states the contract.

**Warning signs:** PR diff adds `html_escape` in `render_html_embed`; the XSS passthrough test fails.

### Pitfall 2: Forgetting one of the three resolver passes

**What goes wrong:** Plan covers `resolve_component_node` arm but misses `collect_unresolved_node` or
`resolve_errors_node`. Compile succeeds (all three matches are exhaustive over `Component`), but
either strict-mode fails to report `HtmlEmbed` as "resolved" (false negatives on unresolved-action
detection — minor since HtmlEmbed has no action) or the `resolve_errors` pass panics on unreachable.

Actually — Rust's non-exhaustive match compiles WITHOUT `HtmlEmbed` in each leaf chain because
`_ => { … }` catch-alls don't exist in these three match statements. The match is exhaustive over
the `Component` enum via explicit arms per variant, so adding `Component::HtmlEmbed(HtmlEmbedProps)`
without touching `resolve.rs` at all **causes a compile error**. This is a GOOD thing: the compiler
enforces the three-arm discipline.

**Root cause:** None — the three matches are exhaustive. Missing an arm is a compile error.

**Prevention:** The compile error is the prevention. Still, Wave 0 RED tests make the contract explicit
(three no-op resolver tests, one per pass).

**Warning signs:** `cargo build -p ferro-json-ui` fails with "non-exhaustive patterns: `&HtmlEmbed(_)`
not covered" — reported at three sites in `resolve.rs`.

### Pitfall 3: Forgetting the MCP catalog exhaustive-list bump

**What goes wrong:** `CatalogComponent` entry for `HtmlEmbed` is added to `execute(None)` but the
exhaustive-list assertion at `json_ui_catalog.rs:1212` still expects `41`. Test fails with
"Catalog should contain all 41 built-in components, got 42".

Or the count is bumped but `"HtmlEmbed"` is not appended to the `expected` array at L1218-1260 —
test passes the length check but fails the "Missing component: HtmlEmbed" `assert!` inside the
loop at L1261-1263.

**Root cause:** Three separate edits are required in one test block: (a) bump the number in the
`assert_eq!(...)` at L1212; (b) update the comment inside the same assertion; (c) append
`"HtmlEmbed"` to the `expected` array.

**Prevention:** Phase 147 precedent — plan must enumerate all three sub-edits as separate task
bullets. The Wave 0 RED test that bumps these assertions COMPILES without the implementation,
so the test starts FAILING at Wave 0 and turns GREEN when Wave 1 adds the CatalogComponent entry.

**Warning signs:** `cargo test -p ferro-mcp json_ui_catalog::tests::test_all_components_present`
fails with either a length mismatch OR a "Missing component: HtmlEmbed" panic.

### Pitfall 4: Accidentally adding `HtmlEmbed` to the `no_required` allowlist

**What goes wrong:** Someone extends the `no_required: [&str; N]` array at
`json_ui_catalog.rs:1375` to include `"HtmlEmbed"`, believing (incorrectly) that HtmlEmbed has
"no required props". The `html` prop IS required.

**Root cause:** Pattern-matching from the `Separator` / `Skeleton` precedent (those genuinely have
no required props).

**Prevention:** D-20 explicit instruction. The `test_components_have_props` test at L1365-1384
catches this: if `HtmlEmbed` is in `no_required`, the "at least one required prop" check skips —
but the CatalogComponent entry genuinely HAS a required `html` prop, so the skip is benign. The
concern is conceptual correctness, not test failure. Still, code review should flag.

**Warning signs:** PR diff modifies the `no_required` array. Reject if it includes `"HtmlEmbed"`.

### Pitfall 5: Docs drift

**What goes wrong:** `docs/src/json-ui/components.md` lacks a `### HtmlEmbed` section. The MCP
catalog advertises the component; the Rust API exposes it; but the human-facing docs say nothing.
This is the exact pitfall phase 147 explicitly called out (Pitfall 7 in 147-RESEARCH).

**Root cause:** CLAUDE.md rule at project level requires docs/src/ updates with framework changes.
Easy to skip in a component-add PR under time pressure.

**Prevention:** Include docs section in the plan explicitly (D-21). Plan 148-03 (or whichever plan
owns the MCP/docs/CI-gate cluster) must include the docs edit as an independent task.

### Pitfall 6: Rustdoc examples in `HtmlEmbedProps::new` fail doctest

**What goes wrong:** The rustdoc example `let props = HtmlEmbedProps::new("<svg>...</svg>");`
might not compile if the module's public re-exports haven't been updated. `cargo test --all-features`
runs doctests; a broken example fails the CI gate.

**Root cause:** `HtmlEmbedProps` is re-exported from `ferro_json_ui` at lib.rs; the doctest at the
struct definition uses `use ferro_json_ui::HtmlEmbedProps;` which depends on that re-export working.

**Prevention:** Wave 1 tasks MUST touch both `component.rs` (struct + rustdoc) and `lib.rs`
(re-export) in the same plan — splitting them across plans with a dependency means the doctest
is broken between merges. Phase 147 plan 02 did both together; emulate.

**Warning signs:** `cargo test --doc -p ferro-json-ui` fails with "unresolved import".

### Pitfall 7: Empty-string `html` edge case

**What goes wrong:** `HtmlEmbedProps::new("")` produces `<div></div>` — an empty div. Serde
round-trips it cleanly; rendered output is valid HTML. But if a consumer conditionally fills the
`html` field and forgets to guard for empty, they ship invisible empty divs.

**Root cause:** No foot-gun protection on empty string (deliberate — a `Default` derive is
specifically ruled out per Claude's Discretion).

**Prevention:** A dedicated Wave 0 test asserts `HtmlEmbedProps::new("")` → `<div></div>` renders
cleanly. This test documents the behavior rather than preventing the scenario, and prevents
regression if someone later adds empty-string handling without thinking about it.

**Warning signs:** None in code — this is a consumer-side concern documented in the test.

---

## Code Examples

All examples verified against the current codebase tree at 2026-04-24.

### Example 1: Building an `HtmlEmbed` component tree (Rust)

```rust
use ferro_json_ui::{ComponentNode, HtmlEmbedProps};

// Server-generated SVG chart (e.g. from plotters, charming, or a custom renderer).
fn build_chart_view(chart_svg: String) -> ComponentNode {
    ComponentNode::html_embed(
        "revenue-chart",
        HtmlEmbedProps::new(chart_svg),
    )
}

// Pre-rendered markdown (e.g. via pulldown_cmark::html::push_html).
fn build_article(rendered_markdown: &str) -> ComponentNode {
    ComponentNode::html_embed(
        "article-body",
        HtmlEmbedProps::new(rendered_markdown),
    )
}
```

### Example 2: Serialized JSON (round-trip contract)

```json
{
  "key": "revenue-chart",
  "type": "HtmlEmbed",
  "html": "<svg viewBox=\"0 0 200 100\"><polyline points=\"0,80 50,40 100,60 150,20 200,50\" stroke=\"blue\" fill=\"none\"/></svg>"
}
```

### Example 3: Rendered HTML (expected output)

For `HtmlEmbedProps { html: "<svg width=\"10\" height=\"10\"><circle cx=\"5\" cy=\"5\" r=\"4\"/></svg>".to_string() }`:

```html
<div><svg width="10" height="10"><circle cx="5" cy="5" r="4"/></svg></div>
```

Note the verbatim SVG inside `<div>...</div>` — the browser parses the inner SVG as SVG, NOT as escaped
text content. That is the entire contract of this component.

### Example 4: Load-bearing XSS passthrough test

**Source shape:** follows `render.rs:9005-9024` (`render_detail_form_view_xss_escapes_strings`) but
INVERTS the assertion direction — HtmlEmbed MUST emit the string unescaped:

```rust
#[test]
fn render_html_embed_emits_html_verbatim_without_escaping() {
    // LOAD-BEARING TEST. This asserts the deliberate escape bypass contract.
    // If this test ever starts expecting &lt;script&gt; instead of <script>,
    // someone has "fixed" render_html_embed by adding html_escape — DO NOT DO
    // THAT. The component's entire purpose is to bypass escaping.
    let view = JsonUiView::new().component(ComponentNode::html_embed(
        "danger",
        HtmlEmbedProps::new("<script>alert('xss')</script>"),
    ));
    let html = render_to_html(&view, &serde_json::Value::Null);
    assert!(
        html.contains("<script>alert('xss')</script>"),
        "HtmlEmbed MUST emit html verbatim; escaping would defeat the component. \
         Got: {html}"
    );
    assert!(
        !html.contains("&lt;script&gt;"),
        "HtmlEmbed MUST NOT html-escape its contents: {html}"
    );
}
```

This is the canonical example of a test that documents intent rather than suggesting a bug. Without it,
a future reader sees `render_html_embed` lacking `html_escape` and is likely to "fix" it.

### Example 5: Baseline render tests (full set)

**Source shape:** follows the `df_props_minimal` + `render_df` helper convention at `render.rs:8795-8840`
(phase 147). HtmlEmbed is simpler — no helper needed:

```rust
#[cfg(test)]
mod html_embed_tests {
    use super::*;
    use crate::component::*;
    use serde_json::Value;

    fn render_embed(html: &str) -> String {
        let view = JsonUiView::new().component(
            ComponentNode::html_embed("e", HtmlEmbedProps::new(html)),
        );
        render_to_html(&view, &Value::Null)
    }

    #[test]
    fn render_html_embed_wraps_in_div() {
        let out = render_embed("<svg/>");
        assert!(
            out.contains("<div><svg/></div>"),
            "expected wrapping div around verbatim html; got: {out}"
        );
    }

    #[test]
    fn render_html_embed_empty_string_produces_empty_div() {
        let out = render_embed("");
        assert!(
            out.contains("<div></div>"),
            "empty html should render empty div; got: {out}"
        );
    }

    #[test]
    fn render_html_embed_preserves_entities_verbatim() {
        // "&amp;" stays as "&amp;" — not re-escaped to "&amp;amp;".
        let out = render_embed("A &amp; B");
        assert!(
            out.contains("A &amp; B"),
            "existing entities must pass through verbatim; got: {out}"
        );
        assert!(
            !out.contains("&amp;amp;"),
            "must NOT double-escape: {out}"
        );
    }

    #[test]
    fn render_html_embed_preserves_angle_brackets_verbatim() {
        let out = render_embed("<span class=\"x\">hello</span>");
        assert!(
            out.contains("<span class=\"x\">hello</span>"),
            "angle brackets must pass through verbatim; got: {out}"
        );
        assert!(
            !out.contains("&lt;span"),
            "must NOT escape < to &lt;: {out}"
        );
    }

    #[test]
    fn render_html_embed_emits_html_verbatim_without_escaping() {
        // See Example 4 above — the load-bearing XSS passthrough test.
        let view = JsonUiView::new().component(ComponentNode::html_embed(
            "danger",
            HtmlEmbedProps::new("<script>alert('xss')</script>"),
        ));
        let html = render_to_html(&view, &Value::Null);
        assert!(
            html.contains("<script>alert('xss')</script>"),
            "HtmlEmbed MUST emit html verbatim; escaping would defeat the component: {html}"
        );
        assert!(
            !html.contains("&lt;script&gt;"),
            "HtmlEmbed MUST NOT html-escape its contents: {html}"
        );
    }
}
```

Five tests total. Every one is a substring assertion — the established convention in this file.

### Example 6: Serde round-trip tests (component.rs tests module)

**Source shape:** follows `component.rs:3696-3707` (`image_round_trips`) and `:2510-2529`
(`separator_defaults_to_horizontal`):

```rust
#[test]
fn html_embed_round_trips() {
    let json = r#"{"type":"HtmlEmbed","html":"<svg/>"}"#;
    let component: Component = serde_json::from_str(json).unwrap();
    match component {
        Component::HtmlEmbed(props) => {
            assert_eq!(props.html, "<svg/>");
        }
        _ => panic!("expected Component::HtmlEmbed"),
    }

    // Reverse direction.
    let original = Component::HtmlEmbed(HtmlEmbedProps::new("<svg/>"));
    let v = serde_json::to_value(&original).unwrap();
    assert_eq!(v["type"], "HtmlEmbed");
    assert_eq!(v["html"], "<svg/>");
    let reparsed: Component = serde_json::from_value(v).unwrap();
    assert_eq!(reparsed, original);
}

#[test]
fn html_embed_props_new_constructor() {
    let a = HtmlEmbedProps::new("<svg/>");
    let b = HtmlEmbedProps::new(String::from("<svg/>"));
    let c = HtmlEmbedProps::new("<svg/>".to_string());
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(a.html, "<svg/>");
}

#[test]
fn component_node_html_embed_factory_shape() {
    let node = ComponentNode::html_embed("chart", HtmlEmbedProps::new("<svg/>"));
    assert_eq!(node.key, "chart");
    assert!(node.action.is_none());
    assert!(node.visibility.is_none());
    assert!(
        matches!(node.component, Component::HtmlEmbed(_)),
        "expected Component::HtmlEmbed variant"
    );
}
```

Also MUST append an entry to `all_known_types_round_trip` at `component.rs:3710-...`:

```rust
("HtmlEmbed", r#"{"type":"HtmlEmbed","html":"<svg/>"}"#),
```

(This test iterates the full component surface; omitting HtmlEmbed creates a latent gap.)

### Example 7: Resolver no-op tests (resolve.rs tests module)

**Source shape:** follows the new phase 147 tests at `resolve.rs:1179-1309`. HtmlEmbed tests are
simpler because the assertion is "the node's `html` field is unchanged byte-for-byte":

```rust
#[test]
fn resolve_component_skips_html_embed() {
    let mut view = JsonUiView::new().component(ComponentNode::html_embed(
        "e",
        HtmlEmbedProps::new("<svg/>"),
    ));
    // Any resolver — HtmlEmbed has no handlers to resolve.
    resolve_actions(&mut view, test_resolver);
    match &view.components[0].component {
        Component::HtmlEmbed(props) => {
            assert_eq!(props.html, "<svg/>", "resolver must not mutate html field");
        }
        _ => panic!("expected Component::HtmlEmbed"),
    }
}

#[test]
fn collect_unresolved_skips_html_embed() {
    // A view with ONLY an HtmlEmbed must report zero unresolved actions.
    let view = JsonUiView::new().component(ComponentNode::html_embed(
        "e",
        HtmlEmbedProps::new("<svg/>"),
    ));
    // Use a resolver that returns None for everything — any action that WERE
    // collected would surface here.
    let result = resolve_actions_strict(view.clone(), |_: &str| None);
    assert!(
        result.is_ok(),
        "HtmlEmbed must not contribute to unresolved-action set; got: {result:?}"
    );
}

#[test]
fn resolve_errors_skips_html_embed() {
    let mut view = JsonUiView::new().component(ComponentNode::html_embed(
        "e",
        HtmlEmbedProps::new("<svg/>"),
    ));
    let mut errors = HashMap::new();
    errors.insert("any_field".to_string(), vec!["boom".to_string()]);
    resolve_errors(&mut view, &errors);
    match &view.components[0].component {
        Component::HtmlEmbed(props) => {
            // No field, no error prop — resolver must not have touched the node.
            assert_eq!(
                props.html, "<svg/>",
                "resolve_errors must not mutate html field"
            );
        }
        _ => panic!("expected Component::HtmlEmbed"),
    }
}
```

### Example 8: Docs section shape (`docs/src/json-ui/components.md`)

**Source shape:** density matches Separator (`components.md:233-258`); safety callout borrows the
warning-block convention from the wider docs site. Place adjacent to DetailForm / KeyValueEditor:

~~~markdown
### HtmlEmbed

Injects pre-rendered HTML verbatim into a wrapping `<div>`. Use for server-generated
content (inline SVG charts, pre-rendered markdown, static HTML widgets) when you
need to compose raw HTML into a JSON-UI view tree without defeating the structural
model.

> ⚠️ **Safety contract.** The `html` prop is emitted **verbatim** into the page
> without HTML-escaping. **`HtmlEmbed` is the only component in ferro-json-ui that
> bypasses escaping.** The caller is responsible for ensuring the content is safe
> against XSS. **Never pass user input into this component.** For escaped text
> output, use [`Component::Text`](#text).

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `html` | `String` | Yes | - | Raw HTML emitted verbatim inside a wrapping `<div>`. NOT escaped. |

```rust
use ferro::{ComponentNode, HtmlEmbedProps};

let chart_svg: String = generate_chart_svg(&data);
let node = ComponentNode::html_embed("chart", HtmlEmbedProps::new(chart_svg));
```

JSON output:

```json
{
  "key": "chart",
  "type": "HtmlEmbed",
  "html": "<svg>...</svg>"
}
```

**When to use:**
- Inline SVG charts generated server-side (e.g. plotters, charming, custom renderers).
- Pre-rendered markdown (e.g. pulldown-cmark → HTML string at request time).
- Static HTML widgets with no data binding.
- Third-party embed snippets (tweet embed HTML, etc.) that you trust.

**When NOT to use:**
- Any content derived from user input — even "sanitized" user input, unless you are certain your sanitizer is sufficient for your threat model.
- Content where the author wants escape-by-default safety. Use [`Component::Text`](#text) instead.
~~~

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Caller constructs a whole parallel HTML page outside JSON-UI for pre-rendered SVG / markdown | Single `Component::HtmlEmbed(HtmlEmbedProps::new(svg_string))` slots into any JSON-UI view tree | Phase 148 (this phase) | Closes a coherence hole: pre-rendered HTML is now a first-class citizen of the composition model without sacrificing escape-by-default discipline elsewhere |
| Plugin variant to wrap every kind of pre-rendered HTML (one plugin per use case) | One built-in `HtmlEmbed` for the general case; plugin components reserved for interactive widgets (Map, etc.) | Phase 148 design (D-01..D-06) | Fewer plugin variants for trivial cases; keeps the plugin surface for genuine behaviour differences |

**Deprecated / outdated:**
- Callers previously emitted raw HTML by writing custom plugin components whose only JS was a
  no-op. That workaround retires on this phase's landing. The plugin system remains for components
  with actual runtime behaviour.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The three `match` statements in `resolve.rs` (at L30+, L211+, L389+) have no `_` catch-all arm, so adding `Component::HtmlEmbed(HtmlEmbedProps)` to the enum without touching `resolve.rs` causes a compile error at three sites. | Pitfall 2, Pattern 7 | [VERIFIED in this research pass — read the three match statements at the line ranges above; all three enumerate leaf variants explicitly without `_`.] If wrong, plan would need an additional explicit check for missing arms. Low risk — verified. |
| A2 | `ferro-json-ui/src/lib.rs` re-export block at L59-72 is the canonical location for new public types; adding `HtmlEmbedProps` in alphabetical position is consistent with the existing file. | Pattern 9 | [VERIFIED in this research pass.] Low risk. |
| A3 | The `docs/src/json-ui/components.md` safety-callout blockquote style (`> ⚠️ **Safety contract.** …`) matches conventions elsewhere on the docs site. | Pattern in Example 8 | Not rigorously verified across the full docs corpus; the phrasing matches the general mdBook/markdown convention. Low risk (cosmetic). If wrong, docs reviewer adjusts blockquote style at write time. |
| A4 | The exhaustive-list assertion at `json_ui_catalog.rs:1212` currently reads `41` (post-phase-147). | Pitfall 3, D-19 | [VERIFIED in this research pass — `assert_eq!(catalog.components.len(), 41, ...)` at L1211-1214.] Low risk. |
| A5 | `HtmlEmbedProps` does NOT need to derive `Eq` — `String` is `Eq` but the crate convention for simple-field props is `PartialEq` only (matches `ImageProps`, `TextProps`). | Pattern 1 | [VERIFIED — SeparatorProps at L525 derives `Eq`; ImageProps does NOT. Either is acceptable; `PartialEq` is the safer default since a future `Option<f64>` prop extension would break `Eq`.] Low risk. |
| A6 | `Component::HtmlEmbed(HtmlEmbedProps)` adjacent to `DetailForm` in the enum (L1096) is the right ordering — the enum groups by "recent additions" at the tail, not alphabetically. | Pattern 2 | [VERIFIED — current enum ordering is recency-grouped, with `DetailForm` at L1096 and `Plugin` catch-all at L1097.] Low risk. |

---

## Open Questions

None blocking. Every decision is either locked by CONTEXT.md or resolved at write-time by matching
existing local conventions (ordering of catalog entries, precise Rustdoc wording within the D-15
content constraints, docs callout emoji/prefix).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` (no external framework) |
| Config file | `ferro-json-ui/Cargo.toml` and `ferro-mcp/Cargo.toml` (workspace; no test-specific config) |
| Quick run command | `cargo test -p ferro-json-ui --lib` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| Estimated runtime | ~30-60 seconds quick; ~3-5 minutes full (incremental) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EMBED-01 (D-01) | `HtmlEmbedProps` derives Debug/Clone/PartialEq/Serialize/Deserialize/JsonSchema | unit (compile-time + serde round-trip) | `cargo test -p ferro-json-ui html_embed_round_trips` | ❌ Wave 0 |
| EMBED-01 (D-02) | `Component::HtmlEmbed(HtmlEmbedProps)` variant present; all three resolver matches compile (non-exhaustive error if missing any of three arms) | unit (compile-time) | `cargo build -p ferro-json-ui` | ❌ Wave 0 |
| EMBED-01 (D-03) | `HtmlEmbedProps::new(impl Into<String>)` compiles and accepts `&str`, `String`, `&String` | unit | `cargo test -p ferro-json-ui html_embed_props_new_constructor` | ❌ Wave 0 |
| EMBED-01 (D-04) | `ComponentNode::html_embed(key, props)` returns node with correct `key`, variant, defaults | unit | `cargo test -p ferro-json-ui component_node_html_embed_factory_shape` | ❌ Wave 0 |
| EMBED-02 (D-05) | `render_html_embed` signature is `fn(&HtmlEmbedProps) -> String` — no `data: &Value` | unit (compile-time via dispatch arm) | `cargo build -p ferro-json-ui` | ❌ Wave 0 |
| EMBED-02 (D-06) | Output shape `<div>{html}</div>` with `html` verbatim; no class/id/attrs | unit | `cargo test -p ferro-json-ui render_html_embed_wraps_in_div` | ❌ Wave 0 |
| EMBED-02 (D-06, load-bearing) | `<script>alert('xss')</script>` emitted unescaped — XSS passthrough contract | unit | `cargo test -p ferro-json-ui render_html_embed_emits_html_verbatim_without_escaping` | ❌ Wave 0 |
| EMBED-02 (D-06) | HTML entities (`&amp;`) pass through verbatim without double-escape | unit | `cargo test -p ferro-json-ui render_html_embed_preserves_entities_verbatim` | ❌ Wave 0 |
| EMBED-02 (D-06) | Angle brackets pass through verbatim | unit | `cargo test -p ferro-json-ui render_html_embed_preserves_angle_brackets_verbatim` | ❌ Wave 0 |
| EMBED-02 (D-06, edge case) | Empty string renders as `<div></div>` | unit | `cargo test -p ferro-json-ui render_html_embed_empty_string_produces_empty_div` | ❌ Wave 0 |
| EMBED-02 (D-07) | Dispatch arm in `render_component` routes `Component::HtmlEmbed(p)` to `render_html_embed(p)` | unit (integration via render_to_html) | Exercised by every render test above | ❌ Wave 0 |
| EMBED-02 (D-08) | `Component::HtmlEmbed(_)` appears in `collect_plugin_types_node` leaf OR-chain (compile-time exhaustiveness) | unit (compile-time) | `cargo build -p ferro-json-ui` | ❌ Wave 0 |
| EMBED-03 (D-10, pass 1) | `resolve_component_node` does not mutate an `HtmlEmbed` node | unit | `cargo test -p ferro-json-ui resolve_component_skips_html_embed` | ❌ Wave 0 |
| EMBED-03 (D-10, pass 2) | `collect_unresolved_node` does not add `HtmlEmbed` to the unresolved set | unit | `cargo test -p ferro-json-ui collect_unresolved_skips_html_embed` | ❌ Wave 0 |
| EMBED-03 (D-10, pass 3) | `resolve_errors_node` does not mutate an `HtmlEmbed` node | unit | `cargo test -p ferro-json-ui resolve_errors_skips_html_embed` | ❌ Wave 0 |
| EMBED-04 (D-12, D-13) | Tagged-enum serialize emits `"type": "HtmlEmbed"`; deserialize arm parses it back | unit | `cargo test -p ferro-json-ui html_embed_round_trips` | ❌ Wave 0 |
| EMBED-04 (D-17, D-18) | MCP `CatalogComponent` entry present for `HtmlEmbed` with correct props shape | unit | `cargo test -p ferro-mcp json_ui_catalog::tests::test_all_components_present` | ❌ Wave 0 (bumped in Wave 0) |
| EMBED-04 (D-19) | Exhaustive-list assertion at `json_ui_catalog.rs:1212` bumped from 41 to 42; `"HtmlEmbed"` appended to `expected` array | unit | `cargo test -p ferro-mcp json_ui_catalog::tests::test_all_components_present` | ❌ Wave 0 |
| EMBED-04 (D-20) | `test_components_have_props` passes — `HtmlEmbed` has at least one required prop (not added to `no_required` allowlist) | unit | `cargo test -p ferro-mcp json_ui_catalog::tests::test_components_have_props` | ❌ Wave 0 (implicit — existing test) |
| EMBED-04 (D-16) | `COMPONENT_CATALOG` const string contains `### HtmlEmbed` header | unit | `cargo test -p ferro-json-ui component_catalog_lists_html_embed` (new assertion, trivial) | ❌ Wave 0 |
| EMBED-04 (D-21) | `docs/src/json-ui/components.md` contains `### HtmlEmbed` section with safety callout | manual/grep | `grep -q '^### HtmlEmbed' docs/src/json-ui/components.md` (visual review in code review) | ❌ Wave 1 (no automated doc-test infrastructure) |
| EMBED-04 (all_known_types_round_trip) | `component.rs` `all_known_types_round_trip` suite includes HtmlEmbed fixture | unit | `cargo test -p ferro-json-ui all_known_types_round_trip` | ❌ Wave 0 (append fixture) |
| EMBED-05 (CI gate) | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` all green | integration | same command | N/A (gate, not a test file) |

### 8-Dimension Validation Plan

The Nyquist validation discipline demands explicit coverage across eight orthogonal dimensions. For
phase 148 the coverage map is:

| Dimension | Coverage | Tests / Checks |
|-----------|----------|----------------|
| **1. Structural correctness** (syntax, types) | Compile-time enforcement | `cargo build -p ferro-json-ui` must pass — three `match` statements in resolve.rs become non-exhaustive if `HtmlEmbed` variant is added to enum without leaf-chain updates, yielding compile errors at three sites. Compiler is the first test suite. |
| **2. Behavioral correctness** (render output) | 5 render tests | `render_html_embed_wraps_in_div`, `..._empty_string_produces_empty_div`, `..._preserves_entities_verbatim`, `..._preserves_angle_brackets_verbatim`, plus the XSS passthrough test. Substring-assertion style matches existing `render.rs` convention. |
| **3. Semantic correctness** (intent: unescaped emission) | 1 load-bearing XSS passthrough test + inline comment in `render_html_embed` body + rustdoc on `HtmlEmbedProps` | `render_html_embed_emits_html_verbatim_without_escaping` — asserts `<script>alert('xss')</script>` appears verbatim AND `&lt;script&gt;` does NOT appear. The test body contains a prominent comment: "LOAD-BEARING TEST. ... DO NOT DO THAT." |
| **4. Integration correctness** (catalog/docs/serde) | serde round-trips + MCP catalog exhaustive list + `all_known_types_round_trip` | `html_embed_round_trips`, `component_node_html_embed_factory_shape`, `all_known_types_round_trip` (with HtmlEmbed fixture appended), `test_all_components_present` (bumped to 42), `test_components_have_props` (existing assertion, should pass unchanged). |
| **5. Security correctness** (explicit XSS intent documentation) | Same test as dimension 3, but framed as a security-contract artifact | The XSS passthrough test doubles as security documentation. `HtmlEmbedProps` rustdoc, `COMPONENT_CATALOG` entry, and MCP `CatalogComponent` description all foreground the caller-owned-safety contract. The `docs/src/json-ui/components.md` safety callout restates it for humans. |
| **6. Performance correctness** | N/A — non-issue | `render_html_embed` is a single `format!` macro call — O(n) in string length, zero allocations beyond the `String` the format! produces. No benchmarks needed. |
| **7. Observability correctness** | N/A — non-issue | Pure function, no logging, no side effects. No tracing spans to verify. |
| **8. Maintainability correctness** (future-proofing against reflexive "fix") | 3 reinforcing artifacts | (a) Inline comment in `render_html_embed` body flagging deliberate `html_escape` omission (see Pattern 4). (b) Rustdoc on `HtmlEmbedProps` stating the safety contract. (c) The XSS passthrough test (dimension 3 test) whose assertion reverses direction vs. every other render test in the file — making a future "fix" impossible without also editing the test. |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui --lib` (~2-3 seconds; exercises every HtmlEmbed test plus the existing ferro-json-ui suite)
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green + ferro-mcp crate tests green (catalog exhaustiveness enforced there) before `/gsd-verify-work`
- **Feedback latency:** < 60s (quick); < 300s (full)

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/component.rs` — add `html_embed_round_trips`, `html_embed_props_new_constructor`, `component_node_html_embed_factory_shape` tests under the existing `mod tests`; append `("HtmlEmbed", r#"{"type":"HtmlEmbed","html":"<svg/>"}"#)` fixture to `all_known_types_round_trip`.
- [ ] `ferro-json-ui/src/render.rs` — add `mod html_embed_tests` with 5 tests (`wraps_in_div`, `empty_string_produces_empty_div`, `preserves_entities_verbatim`, `preserves_angle_brackets_verbatim`, `emits_html_verbatim_without_escaping`).
- [ ] `ferro-json-ui/src/resolve.rs` — add `resolve_component_skips_html_embed`, `collect_unresolved_skips_html_embed`, `resolve_errors_skips_html_embed` tests under the existing `mod tests`.
- [ ] `ferro-json-ui/src/lib.rs` — no dedicated test needed for the re-export (covered by compile + tests above); may add a `component_catalog_lists_html_embed` assertion in an existing `mod tests` if one exists, or skip.
- [ ] `ferro-mcp/src/tools/json_ui_catalog.rs` — bump exhaustive-list assertion (L1212: `41` → `42`; update comment); append `"HtmlEmbed"` to `expected` array at L1218-1260. These edits land in Wave 0 as RED (tests begin failing); Wave 1 CatalogComponent entry makes them GREEN.

No framework installation needed; `cargo test` already present.

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Not in scope — HtmlEmbed is pure rendering |
| V3 Session Management | no | Not in scope |
| V4 Access Control | no | Caller decides whether to render HtmlEmbed at all |
| V5 Input Validation | **yes — inverted** | `HtmlEmbed` is the ONE ferro-json-ui component that explicitly does NOT validate / escape its input. The security model is "caller assumes responsibility". Every surface an author touches (rustdoc, catalog, MCP description, docs) foregrounds this contract. |
| V6 Cryptography | no | Not applicable |
| V7 Error handling | yes | `render_html_embed` has no error path — pure function; no panic possible on any `String` input |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via user input flowing into `HtmlEmbedProps.html` | Tampering | **Caller-side validation.** DetailForm, Form, etc. use `html_escape` at render time; `HtmlEmbed` deliberately does not. The safety contract is documented in four places (rustdoc, COMPONENT_CATALOG, MCP catalog, docs) to make misuse visible. |
| Agent auto-suggesting `HtmlEmbed` for user-input-derived content | Tampering | MCP `CatalogComponent` description foregrounds "Do NOT pass user input". Agents reading the catalog see the constraint before picking the component. Not a compile-time check, but the friction is visible. |
| Reflexive "fix" adding `html_escape` to `render_html_embed` in a future refactor | Regression | Inline comment in function body, rustdoc on props, load-bearing XSS passthrough test (assertion direction reversed vs. every other render test). A "fix" must edit all three sites. |
| Script injection via malformed SVG attributes | Tampering | Caller responsibility. `render_html_embed` does not parse the HTML; invalid or malicious SVG flows through unchanged. Caller should run their own sanitizer if the source is not trusted. |
| CSRF / form submission | Spoofing | Out of scope — `HtmlEmbed` renders static content with no form semantics. |
| Content-Security-Policy interactions | Tampering | `HtmlEmbed` emits raw HTML that may include `<script>` tags. Strict CSP environments (`script-src 'self'`) will block inline scripts inside `HtmlEmbed` at runtime. This is a deployment concern, not a ferro-json-ui bug. Document in the docs "When NOT to use" section. |

---

## Project Constraints (from CLAUDE.md)

Actionable directives extracted for the planner:

- **Run before every commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` — EMBED-05 enforces this as the phase gate.
- **No co-author attribution in commits** — applies to every plan.
- **Prefer editing existing files** over creating new ones — this phase is 100% edits to 4 existing files in ferro-json-ui + 1 in ferro-mcp + 1 in docs/src; no new files.
- **`docs/src/` must reflect framework changes** — `### HtmlEmbed` section in components.md is required (D-21).
- **Update ferro-mcp when introspection surface changes** — `json_ui_catalog.rs` gets a new `CatalogComponent` entry + exhaustive-list bump (D-18, D-19).
- **"This is always a feature branch"** — add `Component::HtmlEmbed` directly; no deprecation layer.
- **Small functions** — `render_html_embed` is a single `format!` call; `HtmlEmbedProps::new` is a one-liner. No comment-sectioning needed.
- **Concrete types, not `interface{}` / `any`** — `HtmlEmbedProps { html: String }` is maximally concrete.
- **Scientific, minimalistic comments** — the inline "deliberate bypass" comment in `render_html_embed` is load-bearing, not marketing.
- **Architectural principle — beauty as a design criterion** — this phase closes a coherence hole (pre-rendered HTML is now expressible in JSON-UI) without sacrificing the escape-by-default discipline elsewhere. The asymmetry is one component, clearly labeled.

---

## Sources

### Primary (HIGH confidence)

- `ferro-json-ui/src/component.rs:524-530` — `SeparatorProps`: canonical minimal single-field-ish props struct
- `ferro-json-ui/src/component.rs:1055-1098` — `Component` enum declaration (current with DetailForm at L1096)
- `ferro-json-ui/src/component.rs:1102-1167` — `serialize_tagged` helper + Serialize arms (DetailForm at L1164)
- `ferro-json-ui/src/component.rs:1172-1314` — `Deserialize` match arms (DetailForm at L1301-1303)
- `ferro-json-ui/src/component.rs:1365-1386` — `ComponentNode::detail_form` factory (shape to mirror for `html_embed`)
- `ferro-json-ui/src/component.rs:1478-1486` — `ComponentNode::separator` factory (minimal shape)
- `ferro-json-ui/src/component.rs:2510-2529` — `separator_defaults_to_horizontal` test (serde round-trip style)
- `ferro-json-ui/src/component.rs:3696-3707` — `image_round_trips` test (concise round-trip pattern)
- `ferro-json-ui/src/component.rs:3710+` — `all_known_types_round_trip` (MUST append `HtmlEmbed` fixture)
- `ferro-json-ui/src/component.rs:4046-4055` — `component_node_detail_form_factory_shape` test (factory test pattern)
- `ferro-json-ui/src/render.rs:101-197` — `collect_plugin_types_node` including DetailForm container arm (L119-123) and leaf group (L164-194)
- `ferro-json-ui/src/render.rs:294-322` — `render_component` dispatch (DetailForm at L311, Separator at L300)
- `ferro-json-ui/src/render.rs:2359-2365` — `render_separator` (minimal render-function pattern)
- `ferro-json-ui/src/render.rs:3081-3094` — `html_escape` function (what `render_html_embed` deliberately does NOT call)
- `ferro-json-ui/src/render.rs:3599-3637` — Separator render tests (minimal substring-assertion style)
- `ferro-json-ui/src/render.rs:8793-9062` — Phase 147 DetailForm render tests (richer substring-assertion style + `render_df` helper convention)
- `ferro-json-ui/src/resolve.rs:52-57` — `Component::DetailForm` arm in `resolve_component_node` (container, NOT leaf)
- `ferro-json-ui/src/resolve.rs:134-161` — `resolve_component_node` leaf OR-chain (insertion point for `HtmlEmbed(_)`)
- `ferro-json-ui/src/resolve.rs:313-338` — `collect_unresolved_node` leaf OR-chain
- `ferro-json-ui/src/resolve.rs:461-488` — `resolve_errors_node` leaf OR-chain
- `ferro-json-ui/src/resolve.rs:1179-1309` — Phase 147 DetailForm resolver tests (pattern for no-op assertions)
- `ferro-json-ui/src/lib.rs:59-72` — public re-export block (insertion point for `HtmlEmbedProps`)
- `ferro-json-ui/src/lib.rs:103-191` — `COMPONENT_CATALOG` literal (insertion point for `### HtmlEmbed` entry)
- `ferro-mcp/src/tools/json_ui_catalog.rs:252-312` — DetailForm CatalogComponent entry (rich description + many props)
- `ferro-mcp/src/tools/json_ui_catalog.rs:313-` — KeyValueEditor CatalogComponent entry
- `ferro-mcp/src/tools/json_ui_catalog.rs:597-607` — Separator CatalogComponent entry (minimal single-prop shape)
- `ferro-mcp/src/tools/json_ui_catalog.rs:1194-1201` — `prop(name, type, required, description)` helper
- `ferro-mcp/src/tools/json_ui_catalog.rs:1207-1264` — `test_all_components_present` (bump 41 → 42 + append `"HtmlEmbed"`)
- `ferro-mcp/src/tools/json_ui_catalog.rs:1364-1384` — `test_components_have_props` (HtmlEmbed NOT added to `no_required`)
- `docs/src/json-ui/components.md:233-258` — Separator docs section (minimal density model)
- `docs/src/json-ui/components.md:473-593` — DetailForm docs section (richer density with safety-ish rule)

### Secondary (MEDIUM confidence)

- `.planning/phases/147-detailform-component-for-inline-edit-ferro-json-ui/147-RESEARCH.md` — immediate precedent; pattern template for this research's structure
- `.planning/phases/147-detailform-component-for-inline-edit-ferro-json-ui/147-VALIDATION.md` — validation strategy template for VALIDATION.md (derivable by the planner from this file's Validation Architecture section)
- `.planning/phases/146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va/` — two-phases-back precedent for component-addition; its runtime wave is NOT inherited (HtmlEmbed has no JS)
- `.planning/ROADMAP.md:1321-1326` — Phase 148 spec with EMBED-01..EMBED-05
- `.planning/VISION.md` — agent-first philosophy; every surface a ferro-mcp consumer touches must communicate intent — motivates the four-location safety framing (rustdoc, COMPONENT_CATALOG, MCP catalog, docs)
- `./CLAUDE.md` §"Testing & Linting" — EMBED-05 CI gate
- `./CLAUDE.md` §"Architecture Principles" — "This is always a feature branch"

### Tertiary (LOW confidence)

None. All claims are anchored to current source (verified via Read/Grep in this research pass) or to
project reference documents.

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new dependencies; every pattern established in the crate
- Architecture: HIGH — all insertion points verified against current source with exact line numbers
- Pitfalls: HIGH — derived from observed patterns (the three-resolver-arms compile discipline, the `html_escape`-by-default convention, the MCP exhaustive-list assertion) and from phase 147's immediate precedent
- Tests: HIGH — substring-assertion bodies copied verbatim from existing `render.rs` / `component.rs` / `resolve.rs` test conventions; adjusted only for the `html_embed` subject and the (load-bearing) inverted XSS assertion direction
- Security framing: HIGH — the four-location safety messaging (rustdoc + COMPONENT_CATALOG + MCP description + docs callout) is directly required by D-15..D-17 and D-21 in CONTEXT.md

**Research date:** 2026-04-24
**Valid until:** ~30 days (stable crate; the v12.0 JSON-UI v2 milestone will overhaul spec format but is a separate track; phase 148 lands on the current v1-era component system)

## RESEARCH COMPLETE
