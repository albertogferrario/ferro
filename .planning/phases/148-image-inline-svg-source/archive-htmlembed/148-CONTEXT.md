# Phase 148: HtmlEmbed component for ferro-json-ui — unescaped HTML injection — Context

**Gathered:** 2026-04-24
**Status:** Ready for planning
**Mode:** `--auto` (single-pass, recommended defaults selected for all gray areas)

<domain>
## Phase Boundary

Add a single-field `HtmlEmbed` component to `ferro-json-ui` whose renderer emits
the provided `html` string **verbatim**, without passing it through
`html_escape`, inside a wrapping `<div>`. The resolver participates in all
three passes (`resolve_component_node`, `collect_unresolved_node`,
`resolve_errors_node`) as a no-op — the component has no action, no children,
and no form-field error semantics. The MCP catalog (`json_ui_catalog`) and the
runtime `COMPONENT_CATALOG` string in `ferro-json-ui/src/lib.rs` both gain an
entry whose description foregrounds the safety contract (unescaped output →
caller is responsible for content safety).

Intended callers: server-side code that pre-renders HTML fragments — inline
SVG charts, pre-rendered markdown, static HTML widgets — and wants to compose
them inside a JSON-UI view without defeating the structural model. This is the
**only** component in the crate that deliberately bypasses `html_escape`; that
asymmetry is load-bearing and must be visible in every surface where a human
or agent encounters the type (docstring, COMPONENT_CATALOG, MCP catalog, docs
chapter).

**Primary files touched:**
- `ferro-json-ui/src/component.rs` — `HtmlEmbedProps` struct; `Component::HtmlEmbed` variant; serde Serialize/Deserialize match arms; `ComponentNode::html_embed` factory
- `ferro-json-ui/src/render.rs` — `render_html_embed` function; dispatch arm in `render_component`; leaf-list arm in `collect_plugin_types_node`
- `ferro-json-ui/src/resolve.rs` — leaf-list arm in all three resolver pass groups (`resolve_component_node` ~line 135, `collect_unresolved_node` ~line 320, `resolve_errors_node` ~line 462)
- `ferro-json-ui/src/lib.rs` — public re-export of `HtmlEmbedProps`; `### HtmlEmbed` section in `COMPONENT_CATALOG` string
- `ferro-mcp/src/tools/json_ui_catalog.rs` — `CatalogComponent { name: "HtmlEmbed", ... }` entry; exhaustive-list assertion bumped from 41 → 42; `"HtmlEmbed"` added to the `expected` array
- `docs/src/json-ui/components.md` — `### HtmlEmbed` section with safety callout, props table, Rust + JSON examples, use cases

**Out of scope:**
- Client-side sanitization (DOMPurify-style)
- HTML validation / well-formedness checks at render time
- Wrapping-element choice beyond `<div>` (no `<span>`, `<section>`, etc.)
- Default Tailwind classes on the wrapping div (callers style via surrounding components)
- Data-binding (`data_path`) — `html` is a static `String` only; binding is a potential future extension, not this phase
- Per-caller escaping opt-in (the component's whole point is to skip escaping — if you need escaping, use `Text`)
- Runtime JS (no entry in `ferro-json-ui/src/runtime/`)
- Plugin variants of HtmlEmbed — this is a built-in component, not a plugin

</domain>

<decisions>
## Implementation Decisions

### Core type shape

- **D-01:** `HtmlEmbedProps` is a named struct, not a tuple variant or struct-enum variant:
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
  pub struct HtmlEmbedProps {
      pub html: String,
  }
  ```
  Rationale: every one of the 42 `Component` variants (after this phase lands) uses a named `*Props` struct. Introducing a bare `Component::HtmlEmbed { html: String }` struct-style variant would break that uniformity for zero gain and would make future extensions (class, wrapper_tag, data_path) a breaking change. The ROADMAP wording `Component::HtmlEmbed { html: String }` is conceptual — the implementation mirrors the crate's established pattern.
- **D-02:** `Component::HtmlEmbed(HtmlEmbedProps)` variant added to the `Component` enum. Inserted after `Component::KeyValueEditor(KeyValueEditorProps)` and before `Component::DetailForm(DetailFormProps)` — i.e. adjacent to the other "component added via recent phase" entries, before the `Plugin` catch-all.
- **D-03:** `HtmlEmbedProps::new(html: impl Into<String>) -> Self` convenience constructor. Keeps call-sites compact: `HtmlEmbedProps::new("<svg>...</svg>")` vs. `HtmlEmbedProps { html: "<svg>...</svg>".to_string() }`. Mirrors the `InputProps::new`-style ergonomic helpers already present elsewhere.
- **D-04:** `ComponentNode::html_embed(key: impl Into<String>, props: HtmlEmbedProps) -> Self` factory. Follows the `(key, props)` pattern used by every existing factory (`ComponentNode::separator`, `ComponentNode::image`, `ComponentNode::detail_form`, etc.). Do **not** collapse the factory to `(key, html)` shorthand — uniformity with the rest of the constructor surface wins over a one-keystroke savings for a single-field props struct.

### Rendering

- **D-05:** `fn render_html_embed(props: &HtmlEmbedProps) -> String` takes `&HtmlEmbedProps` only (no `data: &Value`). The component has no data-binding surface in this phase.
- **D-06:** Output shape: `<div>{props.html}</div>` — `props.html` is emitted **verbatim**, without `html_escape`. No default class, no default id, no attributes. If callers need styling, they wrap this component in a parent (`Card`, `Grid`, `FormSection`). This matches the ROADMAP goal ("inside a wrapping `<div>`") literally.
- **D-07:** Dispatch arm added to `render_component` in `ferro-json-ui/src/render.rs` (~line 294-322 region), grouped with other simple leaf components (near `Component::Separator`, `Component::DescriptionList`): `Component::HtmlEmbed(props) => render_html_embed(props)`.
- **D-08:** Leaf arm added to `collect_plugin_types_node` in `ferro-json-ui/src/render.rs` (~line 165-193 region) — `HtmlEmbed` has no children, so it joins the `| Component::Separator(_) | Component::DescriptionList(_) | ... => {}` leaf group alphabetically grouped with Image (both image-like leaf media).
- **D-09:** The function is public-within-crate (no `pub` modifier needed for `render_*` helpers — they're private module functions called via the `render_component` dispatch).

### Resolver participation

- **D-10:** `Component::HtmlEmbed(_)` is added to the **leaf group** in all three resolver passes in `ferro-json-ui/src/resolve.rs`, mirroring the treatment of `Component::Image(_)` / `Component::Separator(_)`:
  - `resolve_component_node` leaf arm (~line 135-160): add `| Component::HtmlEmbed(_)` to the OR-chain ending in `=> {}`
  - `collect_unresolved_node` leaf arm (~line 313-338): add `| Component::HtmlEmbed(_)` to the OR-chain ending in `=> {}`
  - `resolve_errors_node` leaf arm (~line 462-488): add `| Component::HtmlEmbed(_)` to the OR-chain ending in `=> {}`
- **D-11:** No standalone arms — the grouped OR-chain pattern is the convention for leaves with no action/children/error surface. A standalone `Component::HtmlEmbed(_) => {}` would be wrong noise.

### Serde

- **D-12:** `Component::Serialize` arm — add `Component::HtmlEmbed(p) => serialize_tagged(serializer, "HtmlEmbed", p)` in the match (component.rs ~line 1164, adjacent to `DetailForm`).
- **D-13:** `Component::Deserialize` arm — add in the `match type_str` block (component.rs ~line 1180+):
  ```rust
  "HtmlEmbed" => serde_json::from_value::<HtmlEmbedProps>(value)
      .map(Component::HtmlEmbed)
      .map_err(de::Error::custom),
  ```
- **D-14:** JSON wire format: `{"type": "HtmlEmbed", "html": "<svg>...</svg>"}` (optionally carrying a `key` at the outer `ComponentNode` level per crate convention, but `key` lives on `ComponentNode`, not on `HtmlEmbedProps`).

### Security framing — load-bearing

- **D-15:** Rustdoc on `HtmlEmbedProps` must lead with a prominent safety warning. Exact text TBD during implementation, but must include:
  - "HTML is emitted verbatim without escaping."
  - "Callers are responsible for ensuring the content is safe (XSS)."
  - "Intended for server-generated content (inline SVG, pre-rendered markdown), not user input."
  - A one-line note pointing at `Component::Text` for escaped-string output.
- **D-16:** `COMPONENT_CATALOG` entry in `ferro-json-ui/src/lib.rs` foregrounds the safety contract. Example description: `"Injects pre-rendered HTML (e.g., SVG charts, server-rendered markdown) verbatim without escaping. Caller is responsible for content safety — do NOT pass user input."` Placement: after the `### KeyValueEditor` entry, before `### Separator`, or in alphabetical order — whichever the existing catalog uses (implementation resolves at write time by matching current ordering).
- **D-17:** MCP `CatalogComponent` entry in `ferro-mcp/src/tools/json_ui_catalog.rs` uses the same safety-first description (agents reading this catalog must see the asymmetry before they suggest this component for user-input flows).

### MCP catalog & exhaustive-list assertion

- **D-18:** Add `CatalogComponent { name: "HtmlEmbed", description: "...", props: vec![prop("html", "String", true, "Raw HTML emitted unescaped into a wrapping <div>. Caller is responsible for content safety.")], variants: None }` to the catalog builder in `ferro-mcp/src/tools/json_ui_catalog.rs`. Placement adjacent to `DetailForm` / `KeyValueEditor` entries.
- **D-19:** Bump the exhaustive-list assertion:
  - `ferro-mcp/src/tools/json_ui_catalog.rs:1212` — change `41` to `42` and update the comment (`"... 41 built-in components (including DetailForm + KeyValueEditor backfill)"` → `"... 42 built-in components (including HtmlEmbed)"`).
  - `ferro-mcp/src/tools/json_ui_catalog.rs:1218-1260` — add `"HtmlEmbed",` to the `expected` array (alphabetically near `"Header"` or adjacent to `"DetailForm"` — match the existing order).
- **D-20:** Respect the `no_required` test at `ferro-mcp/src/tools/json_ui_catalog.rs:1375` — `HtmlEmbed` has one required prop (`html`), so it is NOT added to the `no_required` allowlist.

### Docs

- **D-21:** Add `### HtmlEmbed` section to `docs/src/json-ui/components.md`, placed adjacent to `### DetailForm` / `### KeyValueEditor` (match existing order). Section includes:
  - Opening paragraph describing purpose (server-generated HTML injection for SVG charts, pre-rendered markdown).
  - **Safety callout** styled as a blockquote or `> ⚠️ Warning` block, foregrounding the unescaped behavior and XSS caller-responsibility.
  - Props table (single row: `html` / `String` / Yes / - / Raw HTML to inject).
  - Rust example using `ComponentNode::html_embed(...)` with `HtmlEmbedProps::new(...)`.
  - JSON output example (`{"key": "chart", "type": "HtmlEmbed", "html": "<svg>...</svg>"}`).
  - Use-case list: inline SVG charts, pre-rendered markdown, static HTML widgets, third-party embed snippets.
  - Explicit "for escaped text, use `Text`" pointer.

### TDD wave structure (follow Phase 147 shape)

- **D-22:** Plans decompose into waves:
  - **Wave 0 RED tests** (no deps): serde round-trip for `HtmlEmbedProps` + `Component::HtmlEmbed` in `component.rs` tests; `render_html_embed` tests in `render.rs` (verbatim emission, wrapping `<div>`, empty string, HTML entity preservation, script tag preservation — to prove the escape bypass); resolver no-op tests in `resolve.rs` (HtmlEmbed skipped by all three passes); MCP catalog assertion bumps in `ferro-mcp/src/tools/json_ui_catalog.rs` tests (length → 42, `"HtmlEmbed"` in expected array).
  - **Wave 1 impl** (depends on Wave 0): `HtmlEmbedProps` struct + `Component::HtmlEmbed` variant + serde arms + `ComponentNode::html_embed` factory in `component.rs`; `render_html_embed` function + dispatch + leaf arm in `render.rs`; three leaf arms in `resolve.rs`; `COMPONENT_CATALOG` entry + public re-export in `lib.rs`; `CatalogComponent` entry + exhaustive-list bump in `ferro-mcp/src/tools/json_ui_catalog.rs`; `### HtmlEmbed` section in `docs/src/json-ui/components.md`; CI gate (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`).
- **D-23:** Follow phase 147's plan-splitting convention: multiple plans if wave 1 grows large enough to benefit from parallel execution. A reasonable split: `148-01` Wave 0 RED; `148-02` Rust types (component.rs); `148-03` Renderer (render.rs); `148-04` Resolver (resolve.rs); `148-05` MCP catalog + docs + CI gate. Planner may consolidate if wave 1 comfortably fits in 2-3 plans.
- **D-24:** No runtime wave (unlike phase 146 KeyValueEditor). `HtmlEmbed` has zero JS. Confirmed by the `render_html_embed` function returning a plain string with no plugin hooks.

### Claude's Discretion

- Exact Rustdoc wording for the safety warning on `HtmlEmbedProps` — must satisfy D-15's content requirements but prose style follows existing `InputProps` / `FormProps` doc patterns.
- Exact description text in the MCP catalog and COMPONENT_CATALOG — must satisfy D-16 / D-17 (safety-first) but phrasing is authorial.
- Alphabetical vs. recency-grouped ordering for the new variants / factory / catalog entries — match whatever local ordering already exists; do not invent a new convention.
- Whether to emit a test that asserts `<script>alert('xss')</script>` passes through unescaped (to **prove** the bypass contract). Recommended: yes — this is the one case where such a test documents intent rather than suggesting a bug.
- Exact wording and emoji (if any) on the docs safety callout — follow whatever convention the docs site's existing warnings use (scan `docs/src/json-ui/components.md` and sibling files).
- Whether `HtmlEmbedProps` gets a `Default` derive (recommended: no — a default empty `html` string is a semantic foot-gun; prefer explicit construction via `HtmlEmbedProps::new`).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Component patterns this phase mirrors (single-field / leaf variants)
- `ferro-json-ui/src/component.rs:524-530` — `SeparatorProps`: canonical minimal single-field-ish props struct (has one `Option<Orientation>`)
- `ferro-json-ui/src/component.rs:966-981` — `ActionCardProps`: example of a Props struct with derives + rustdoc pattern to mirror
- `ferro-json-ui/src/component.rs:1055-1098` — `Component` enum declaration; `HtmlEmbed(HtmlEmbedProps)` variant is inserted here (before `Component::Plugin`, adjacent to `DetailForm`/`KeyValueEditor`)
- `ferro-json-ui/src/component.rs:1119-1167` — `Component::Serialize` match arms; the `HtmlEmbed` serialize arm goes near `DetailForm` / `KeyValueEditor`
- `ferro-json-ui/src/component.rs:1172-1280` — `Component::Deserialize` match arms; the `"HtmlEmbed"` arm goes in this block
- `ferro-json-ui/src/component.rs:1478-1484` — `ComponentNode::separator` factory: exact shape for `ComponentNode::html_embed(key, props)`
- `ferro-json-ui/src/component.rs:4043-4047` — `ComponentNode::detail_form` factory test: same test shape to follow for the HtmlEmbed factory test

### Rendering patterns this phase mirrors
- `ferro-json-ui/src/render.rs:294-322` — `render_component` dispatch table; `Component::HtmlEmbed(props) => render_html_embed(props)` arm inserted near `Component::Separator` / `Component::DescriptionList`
- `ferro-json-ui/src/render.rs:2359-2365` — `render_separator`: canonical pattern for a minimal render function that returns a single string with no data-binding
- `ferro-json-ui/src/render.rs:163-193` — `collect_plugin_types_node`: leaf-component group where `HtmlEmbed(_)` joins the `=> {}` OR-chain
- `ferro-json-ui/src/render.rs` (grep `html_escape`) — every other render function calls `html_escape` on dynamic strings; `render_html_embed` is the **explicit exception** and must carry an inline comment documenting this

### Resolver patterns this phase mirrors
- `ferro-json-ui/src/resolve.rs:30-162` — `resolve_component_node`: HtmlEmbed joins the leaf OR-chain at lines 135-160
- `ferro-json-ui/src/resolve.rs:211-339` — `collect_unresolved_node`: HtmlEmbed joins the leaf OR-chain at lines 313-338
- `ferro-json-ui/src/resolve.rs:389-493` — `resolve_errors_node`: HtmlEmbed joins the leaf OR-chain at lines 462-488

### MCP catalog
- `ferro-mcp/src/tools/json_ui_catalog.rs:597-607` — `CatalogComponent` entry for `Separator`: canonical shape for a minimal single-prop catalog entry
- `ferro-mcp/src/tools/json_ui_catalog.rs:1193-1201` — `prop(...)` helper used to build `PropInfo` entries
- `ferro-mcp/src/tools/json_ui_catalog.rs:1207-1263` — `test_all_components_present`: the exhaustive-list assertion that must bump from 41 → 42 and gain `"HtmlEmbed"` in the expected array
- `ferro-mcp/src/tools/json_ui_catalog.rs:1364-1384` — `test_components_have_props`: assertion chain that `HtmlEmbed` must satisfy (has at least one required prop — `html`)

### COMPONENT_CATALOG runtime string
- `ferro-json-ui/src/lib.rs:103+` — `pub const COMPONENT_CATALOG: &str`: shared LLM context string; new `### HtmlEmbed` section goes here, matching the density and safety-tone of the `### DetailForm` / `### KeyValueEditor` entries at lines 120-147
- `ferro-json-ui/src/lib.rs:64-69` — public re-export block where `HtmlEmbedProps` is added

### Documentation
- `docs/src/json-ui/components.md:233-258` — `### Separator` section: shape and density to mirror for `### HtmlEmbed`
- `docs/src/json-ui/components.md:473+` — `### DetailForm` section: richer-example shape for more-involved components (safety callout style TBD here)

### Adjacent-phase precedent for "add component to ferro-json-ui"
- `.planning/phases/147-detailform-component-for-inline-edit-ferro-json-ui/147-CONTEXT.md` — immediate predecessor, same shape of work (component.rs + render.rs + resolve.rs + lib.rs + MCP catalog + docs); wave structure to emulate
- `.planning/phases/147-detailform-component-for-inline-edit-ferro-json-ui/147-01-PLAN.md` through `147-05-PLAN.md` — wave decomposition pattern (Wave 0 RED, Wave 1 impl split by file/area)
- `.planning/phases/146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va/146-CONTEXT.md` — two-phase-back precedent for adding a component; its runtime wave is NOT inherited (HtmlEmbed has no JS)

### Project principles
- `.planning/VISION.md` — "agent-first" philosophy; every surface a ferro-mcp consumer touches must communicate intent and constraints clearly — this is why the safety framing in the MCP catalog description is load-bearing, not cosmetic
- `.planning/PROJECT.md` §"Beauty as a design criterion" — conceptual coherence is a v1.0 gate; `HtmlEmbed` exists to close a hole in the composition model (pre-rendered HTML) without sacrificing the rest of the crate's escape-by-default discipline
- `/Users/alberto/.claude/CLAUDE.md` §"Architecture Principles" — "This is always a feature branch": no backwards-compat layer; `HtmlEmbed` is added directly and the `expected` catalog list is bumped to 42 in a single atomic commit alongside the variant
- `/Users/alberto/.claude/CLAUDE.md` §"Testing & Linting" — the CI gate (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`) is the completion criterion for wave 1; any plan must run it before claiming done

### Requirements
- `.planning/ROADMAP.md:1321-1326` — Phase 148 spec:
  - EMBED-01: `Component::HtmlEmbed { html: String }` variant in `Component` enum (implemented as `HtmlEmbed(HtmlEmbedProps)` per D-01)
  - EMBED-02: renderer bypasses `html_escape`
  - EMBED-03: resolver participates in all three passes (no-op)
  - EMBED-04: MCP catalog entry
  - EMBED-05: CI gate green

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `render_separator` (`ferro-json-ui/src/render.rs:2359-2365`) — canonical minimal render function; `render_html_embed` mirrors this shape but takes only `&HtmlEmbedProps` and concatenates a `<div>` wrapper around `props.html` **without** calling `html_escape`.
- `ComponentNode::separator` factory (`ferro-json-ui/src/component.rs:1478-1484`) — exact factory shape to clone for `ComponentNode::html_embed`.
- `serialize_tagged` helper (`ferro-json-ui/src/component.rs:1104-1117`) — used for `{"type": "HtmlEmbed", "html": "..."}` wire format via a one-line match arm.
- `prop(name, type, required, description)` helper (`ferro-mcp/src/tools/json_ui_catalog.rs:1193-1201`) — used to build the single `PropInfo` for the MCP catalog entry.

### Established Patterns
- **Leaf components with no action/children/errors** participate in the three resolver passes via a single OR-chain arm ending in `=> {}`. `HtmlEmbed` follows this — adding a standalone empty arm would be wrong-style.
- **`html_escape`-by-default** is enforced across `render.rs` by convention, not by a type-system mechanism. `render_html_embed` is the explicit, documented exception. Add an inline comment in the function body flagging this so future audits don't "fix" it.
- **Component exhaustiveness in the MCP catalog** is enforced by `test_all_components_present` (`ferro-mcp/src/tools/json_ui_catalog.rs:1207-1263`). Every component-addition phase bumps the magic number and appends to the `expected` array — phase 148 bumps 41 → 42 and adds `"HtmlEmbed"`.
- **COMPONENT_CATALOG string ordering** in `ferro-json-ui/src/lib.rs` roughly tracks insertion order (Text, Button, Card, Table, Form, DetailForm, Input…); new entries slot in where semantically adjacent components live (media-ish/content: near `DescriptionList`, `Separator`; or near the other recent additions `KeyValueEditor` / `DetailForm`).

### Integration Points
- `Component` enum (`component.rs:1055-1098`) — insert `HtmlEmbed(HtmlEmbedProps)` variant
- `Component::Serialize` (`component.rs:1119-1167`) — add `serialize_tagged` arm
- `Component::Deserialize` (`component.rs:1172-1280`) — add `"HtmlEmbed" => ...` arm
- `ComponentNode` factories (`component.rs:1478+`) — add `html_embed` factory
- `render_component` dispatch (`render.rs:294-322`) — add `HtmlEmbed(props) => render_html_embed(props)` arm
- `collect_plugin_types_node` leaf group (`render.rs:163-193`) — add `Component::HtmlEmbed(_)` to the OR-chain
- Three resolver passes in `resolve.rs` (lines 135-160, 313-338, 462-488) — add `Component::HtmlEmbed(_)` to each leaf OR-chain
- `COMPONENT_CATALOG` string + public re-export in `lib.rs` (lines 64-69 for re-export, ~103+ for string)
- `CatalogComponent` list + exhaustive-list assertion in `ferro-mcp/src/tools/json_ui_catalog.rs`
- `### HtmlEmbed` section in `docs/src/json-ui/components.md`

### Creative Options
- A `HtmlEmbed` test that passes through `<script>alert('xss')</script>` unescaped serves as both a regression guard AND executable documentation of the intended-bypass contract. Include it.

</code_context>

<specifics>
## Specific Ideas

- The coherence win: callers with server-generated HTML fragments (typically SVG charts from `plotters` / `charming`, or pre-rendered markdown via `pulldown-cmark`) no longer need workarounds like emitting a whole parallel HTML page or constructing custom plugin components. A single `Component::HtmlEmbed(HtmlEmbedProps::new(svg_string))` slots into any JSON-UI view tree.
- The asymmetry matters: every other component in ferro-json-ui HTML-escapes dynamic content. `HtmlEmbed` is the **only** exception. That asymmetry is the whole point — but it means the safety messaging must appear in every surface an author touches:
  1. `HtmlEmbedProps` rustdoc (D-15)
  2. `COMPONENT_CATALOG` string description (D-16)
  3. MCP `CatalogComponent` description (D-17)
  4. `### HtmlEmbed` docs chapter safety callout (D-21)
  5. Inline comment in `render_html_embed` function body flagging the deliberate `html_escape` omission
- Target content types explicitly named in rustdoc and docs: **inline SVG**, **pre-rendered markdown**, **static HTML widgets**, **third-party embed snippets** (e.g. a tweet embed). Explicitly non-targets: **user input**, **anything not controlled by the server**.
- The component's minimalism is a feature. Resist pressure to add `class`, `id`, `wrapper_tag`, `data_path`, or XSS sanitization in this phase — all are future extensions and listed in `<deferred>` below.

</specifics>

<deferred>
## Deferred Ideas

- **`class` / `id` / `style` props on the wrapper `<div>`** — today callers style via surrounding components (`Card`, `Grid`). If a real call-site needs a class on the wrapper itself, add it as an optional prop in a follow-up phase; don't preempt.
- **Configurable wrapper element** — e.g. `wrapper_tag: Option<String>` to emit `<span>...</span>` inline instead of `<div>...</div>`. Today, inline usage is solved by wrapping in a parent that enforces inline display. Revisit if a concrete need appears.
- **`data_path: Option<String>`** — binding raw HTML from the data payload (e.g. `render_html_embed` looks up `data.chart_svg` at render time). A clean extension, but today all intended callers have the HTML string at component-construction time. Deferred until a call-site actually needs the binding.
- **Built-in sanitization opt-in** — `sanitize: Option<bool>` that runs content through an HTML sanitizer. Out of scope — the whole point of this component is the caller assuming safety responsibility. If sanitization is needed, the caller runs it before constructing `HtmlEmbedProps`.
- **Plugin-style HtmlEmbed variants** — e.g. an `HtmlEmbedIframe` that sandboxes via `<iframe sandbox>`. Possible future extension as a separate component (not a variant); out of scope for Phase 148.
- **Markdown-aware sibling component** (`MarkdownEmbed { source: String }` that renders markdown server-side) — a natural next step but orthogonal; would belong in its own phase and could be layered on top of `HtmlEmbed` internally.
- **Framework-level `#[warning("unescaped")]` marker** — a compile-time lint that flags `HtmlEmbedProps` construction with a user-input-derived string. Ambitious and likely out-of-reach without a dedicated Clippy lint; not planned.

</deferred>

---

*Phase: 148-htmlembed-component-ferro-json-ui*
*Context gathered: 2026-04-24*
