# Phase 150: ferro-json-ui RichTextEditor component — Context

**Gathered:** 2026-04-29
**Status:** Ready for planning
**Mode:** `--auto` (single-pass, recommended defaults selected for all gray areas)
**Source:** gestiscilo-it v6.4 — used by Phase 125 (document template editor); REQUIREMENTS.md DOC-02, DOC-04

<domain>
## Phase Boundary

Add a `RichTextEditor` component to `ferro-json-ui`. The component renders a host `<div data-rich-text-editor>` plus a per-page bootstrap that loads Quill 2.0.3 (Snow theme) from jsDelivr, SRI-pinned. Authors get rich-text input in dashboard forms with no JS build step.

The component is a **first-class `Component::RichTextEditor` variant** (mirrors KeyValueEditor / Phase 146) but, unlike KeyValueEditor, also requires **external CDN assets** (mirrors the Map plugin / `JsonUiPlugin` asset pattern). This is the first first-class component variant in ferro-json-ui that requires CDN-loaded JS/CSS — the plugin asset pipeline (`Asset` + dedup) is reused, not duplicated.

Output is dual-format on submit: two hidden inputs `{name}_delta` (canonical Delta JSON, lossless) and `{name}_html` (sanitized HTML, rendering input). Consumer controllers read both.

Toolbar `formats` whitelist is the single source of truth: it constrains both Quill's toolbar config (at init) and the HTML post-process (at submit). Image/video/HTML-paste paths are not reachable through the prop surface.

**Primary files touched:**
- `ferro-json-ui/src/component.rs` — `RichTextEditorProps` struct + `Component::RichTextEditor` variant + serde match arms + `ComponentNode::rich_text_editor(...)` factory
- `ferro-json-ui/src/render.rs` — `render_rich_text_editor()` + dispatch arm
- `ferro-json-ui/src/runtime/rich_text_editor.rs` — new IIFE module (`setupRichTextEditor`)
- `ferro-json-ui/src/runtime/mod.rs` — wire new module into bundle + dispatcher
- `ferro-json-ui/src/lib.rs` — public re-exports + `COMPONENT_CATALOG` entry
- `ferro-json-ui/src/render.rs` (asset emission) OR `ferro-json-ui/src/plugin.rs` — Quill CDN/SRI asset injection hook for first-class components (see D-07)
- `ferro-mcp` `CatalogComponent` registry — auto-derived from JsonSchema, else manual entry (SC-7)
- `docs/src/json-ui/components.md` — `### RichTextEditor` section with props table + Rust + JSON example (SC-5)

**Out of scope:**
- Image / video / file-upload toolbar buttons (and their handlers)
- HTML paste path (Quill clipboard module configured to strip on paste)
- Markdown-to-Delta or Delta-to-Markdown converters
- Inertia / SPA fetch-based submission (works with full-page form POST only)
- Multiple Quill themes beyond Snow (Bubble, custom themes)
- Quill modules beyond toolbar (mention, autoformat, syntax highlighting, etc.)
- Real-time collaboration (Yjs / shared state)
- Per-instance keyboard binding overrides
- Accessibility audit beyond the defaults Quill 2.x ships with (deferred to a focused a11y phase if a real gap surfaces)
- Bundling Quill into `ferro-base.css` / a vendored asset path — CDN with SRI is the contract for v1
- Server-side HTML sanitization in Rust — sanitization happens in the IIFE before submit; server trusts the `_html` payload format only insofar as the IIFE produced it (callers already store `_delta` as the canonical record)

</domain>

<decisions>
## Implementation Decisions

### Architectural placement

- **D-01:** `Component::RichTextEditor(RichTextEditorProps)` is added as a **first-class component variant** in `ferro-json-ui/src/component.rs` — not a plugin via `JsonUiPlugin`. Locked by SC-1 ("exists in the ferro-json-ui component catalog with ... compile-enforced via the existing component derive").
- **D-02:** The component reuses the **existing plugin asset infrastructure** (`Asset` struct with SRI/crossorigin, page-level dedup) rather than introducing a parallel CDN-emission code path. This is the cross-cutting refactor this phase introduces: first-class components can declare CDN assets through the same pipeline plugins use. Mechanism choice (synthetic registered-at-startup plugin under the hood vs explicit asset hook in `render_rich_text_editor`) is **Claude's discretion** — the **constraint** is "one Quill load per page, idempotent, deduplicated by URL across all editor instances, identical SRI emission shape as `MapPlugin`." See `<code_context>` for the Map plugin precedent.

### Props structure

- **D-03:** `RichTextEditorProps` (public, in `ferro-json-ui/src/component.rs`):
  - `name: String` — base form field name; emits two hidden inputs `{name}_delta` and `{name}_html` on submit (SC-3)
  - `value: Option<String>` — initial editor content. Auto-detected at runtime: if it parses as JSON, treated as Delta; otherwise loaded as HTML via `clipboard.dangerouslyPasteHTML` filtered through the formats allowlist. See D-12.
  - `formats: Vec<String>` — toolbar whitelist; drives both Quill toolbar config (at init) AND HTML allowlist (at submit-time post-process). Default: `vec!["bold", "italic", "underline", "list", "header", "link"]` (mapped from the SC-1 list — Quill represents bullet+ordered as one `list` format with an `ordered` attribute, headings as `header`)
  - `placeholder: Option<String>` — passed verbatim to Quill `placeholder` option
  - `theme: String` — defaults to `"snow"` at deserialization (`#[serde(default = "default_theme")]`); SC-1 says default Snow. Only `"snow"` is supported in v1; other values render as `snow` (no error) — bubble/custom is deferred (see `<deferred>`)
  - `label: Option<String>` — visible label rendered above the editor host (mirrors `InputProps.label` shape but optional like `KeyValueEditorProps.label`)
  - `error: Option<String>` — validation error rendered below the editor with destructive token styling (mirrors `InputProps.error`)
  - `data_path: Option<String>` — JSON pointer for pre-fill; resolves to a string passed as the initial value (overridden by explicit `value` if both set; `value` wins)
  - `required: Option<bool>` — emitted as `required` on a backing hidden input the IIFE flips when the editor is empty (so HTML5 form-validation surfaces)
- **D-04:** `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]` on `RichTextEditorProps` — `JsonSchema` IS derivable (props are flat: String / Option<String> / Vec<String> / Option<bool> — no `Action` or `ComponentNode` children, unlike `DetailFormProps` / `FormProps`).

### Hidden field emission

- **D-05:** Renderer emits, inside a wrapper `<div data-rich-text-editor>` block, exactly two hidden inputs: `<input type="hidden" name="{name}_delta" data-rte-hidden="delta">` and `<input type="hidden" name="{name}_html" data-rte-hidden="html">`. Both initialized to empty / the converted form of `value` at render time so JS-disabled clients still POST something coherent (they'll POST whatever the renderer wrote — the initial value, unchanged).
- **D-06:** Hidden field naming is `{name}_delta` / `{name}_html` (underscore-separated, single source of `name` from props). Locked by SC-3. Consumer controllers read both fields off `req.input()`.

### Quill loading

- **D-07:** Quill JS and CSS are loaded once per page from jsDelivr with SRI hashes pinned at compile time as `const` strings in `ferro-json-ui/src/render.rs` (or a new `ferro-json-ui/src/assets/quill.rs` if the planner prefers to scope them — Claude's discretion). URLs and SRI shape mirror `ferro-json-ui/src/plugins/map.rs:78-84`:
  ```rust
  const QUILL_JS_URL: &str  = "https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.js";
  const QUILL_CSS_URL: &str = "https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.snow.css";
  const QUILL_JS_SRI: &str  = "sha384-..."; // computed at planning time from the actual jsDelivr file
  const QUILL_CSS_SRI: &str = "sha384-..."; // computed at planning time from the actual jsDelivr file
  ```
- **D-08:** SRI hash computation is the **planner's responsibility** during plan creation. The plan must `curl -I` the two URLs and compute the SHA-384 hash; if the hash cannot be computed offline, plan a wave that bootstraps the constants and a separate audit that verifies them before the SRI lands in committed code. **Do not invent hashes**.
- **D-09:** `crossorigin="anonymous"` is set on both `<script>` and `<link>` tags (matches `MapPlugin` Asset emission shape).
- **D-10:** Quill version pin is `2.0.3` exactly. Bumping is a future phase that re-runs SRI computation.

### Per-editor wiring

- **D-11:** Editor host element: `<div data-rich-text-editor data-rte-formats="{json-encoded formats array}" data-rte-theme="snow" data-rte-placeholder="...">{initial HTML}</div>`. Each editor's config travels via `data-*` attributes; the IIFE reads them per-instance. Multiple editors on the same page work without ID collisions.
- **D-12:** Initial value handling: if `value` parses as JSON (try `JSON.parse` in IIFE; on success and the parsed value is an object with `ops`, treat as Delta), call `quill.setContents(parsed)`. Otherwise, pass through Quill's `clipboard.dangerouslyPasteHTML` which Quill itself filters by the configured formats. The filter at this stage is automatic — Quill drops any tag/attribute outside the formats whitelist.
- **D-13:** Submit interception: IIFE finds the editor's enclosing `<form>` (`element.closest('form')`) and attaches a `submit` listener (capture phase). On submit, before submission proceeds: serialize `quill.getContents()` → `{name}_delta` hidden input; serialize `quill.root.innerHTML` → run through the format-allowlist post-process → `{name}_html` hidden input.
- **D-14:** Multiple RichTextEditors in the same form: each editor's IIFE-installed listener is independent; submit serializes all of them. No ordering guarantees needed (they each write disjoint hidden inputs).

### Formats allowlist enforcement

- **D-15:** `formats: Vec<String>` is the **single source of truth**, enforced at TWO points:
  1. **Init**: passed to Quill's options `formats: [...]` (Quill drops any input — keystroke or paste — that's outside this list)
  2. **Submit**: IIFE post-process strips any tag whose tagname maps to a format outside `formats` from `quill.root.innerHTML` before writing `{name}_html`
- **D-16:** Tag→format map embedded in the IIFE: `b/strong→bold`, `i/em→italic`, `u→underline`, `s→strike`, `ul/ol/li→list`, `h1..h6→header`, `a→link`, `blockquote→blockquote`, `pre/code→code-block`. Tags not in the map are stripped (their text content is kept). Disallowed attributes (`style`, `class` not from Quill's own `ql-*`, event handlers, `src` on anything but `<a>` href in `link` mode) are stripped.
- **D-17:** No external HTML sanitizer (no DOMPurify, no Rust `ammonia` dep). The allowlist post-process is a hand-rolled walker in the IIFE — small, scoped, ES5-compatible. Image/video/HTML-paste paths are simply not in the formats map, so they cannot survive the post-process even if Quill emitted them.

### Default formats

- **D-18:** Default `formats` (when caller omits the prop) is `vec!["bold", "italic", "underline", "list", "header", "link"]`. Six entries. Maps SC-1's "bold/italic/underline/lists/headings/links" to Quill's actual format names (lists is one format with `bullet`/`ordered` attribute; headings is `header` with level attribute).
- **D-19:** Default toolbar configuration derives from `formats` automatically: a deterministic mapping from format name → toolbar group/button entry. e.g. `["bold", "italic", "underline"]` → `[['bold', 'italic', 'underline']]`; `["list"]` → `[[{'list': 'ordered'}, {'list': 'bullet'}]]`; `["header"]` → `[[{'header': [1, 2, 3, false]}]]`. The mapping table lives in the IIFE; it's not configurable for v1 (deferred).

### Runtime module

- **D-20:** New file `ferro-json-ui/src/runtime/rich_text_editor.rs` exporting `pub(super) const SOURCE: &str = r#"..."#;` containing `setupRichTextEditor()`. Vanilla ES5 (`var`, named function declarations, no arrow functions, no `let`/`const` outside template strings) — same dialect as `runtime/key_value_editor.rs:17`.
- **D-21:** `runtime/mod.rs` adds three lines:
  - `mod rich_text_editor;` near the existing `mod key_value_editor;`
  - `s.push_str(rich_text_editor::SOURCE);` in the bundle assembler (mirror `runtime/mod.rs:40`)
  - `setupRichTextEditor();` in the dispatcher (mirror `runtime/mod.rs:49`)
  - All three runtime-bundle tests (the ones that grep for `setupKeyValueEditor`) extend to also grep for `setupRichTextEditor`.

### MCP catalog & docs

- **D-22:** `COMPONENT_CATALOG` const in `ferro-json-ui/src/lib.rs` adds a `### RichTextEditor` section, format and prose mirroring `### KeyValueEditor` (`lib.rs:145-147`). Description: "Rich-text editor backed by Quill 2.0.3 (Snow theme, jsDelivr CDN, SRI-pinned). Emits two hidden inputs on submit: `{name}_delta` (Delta JSON, canonical) and `{name}_html` (sanitized HTML). The `formats` whitelist constrains both the toolbar and the HTML allowlist — image/video/HTML-paste paths are not reachable through the prop surface."
- **D-23:** ferro-mcp `CatalogComponent` for `RichTextEditor`:
  - If ferro-mcp's catalog auto-derives from `JsonSchema` on the props struct, no manual change needed — the `JsonSchema` derive on `RichTextEditorProps` (D-04) drives it
  - If ferro-mcp maintains a manual catalog (parallel to `COMPONENT_CATALOG` const), add the same description + props rows
  - Component count assertion in MCP tests increments by 1 (SC-6)
- **D-24:** Docs page `docs/src/json-ui/components.md` adds `### RichTextEditor` heading with: props table, Rust example showing `ComponentNode::rich_text_editor("body", RichTextEditorProps { name: "body".into(), formats: vec!["bold", "italic", "underline", "link"].into_iter().map(String::from).collect(), ..Default::default() })`, and a JSON example. Format mirrors the existing KeyValueEditor docs section.

### Public surface

- **D-25:** `lib.rs` re-exports `RichTextEditorProps` from the public prelude (`pub use component::RichTextEditorProps;` near the other prop re-exports).
- **D-26:** `ComponentNode::rich_text_editor(name, props)` factory constructor added in `component.rs` near the other factories (e.g. `ComponentNode::input`, `ComponentNode::key_value_editor` if it exists, else next to `form`/`input`). Signature: `pub fn rich_text_editor(name: impl Into<String>, props: RichTextEditorProps) -> Self` — and ensures `props.name` matches the `name` arg if both are provided (caller convenience: passing `name` to the factory sets `props.name` if unset).

### Validation & a11y

- **D-27:** `error: Option<String>` rendered below the editor host with `border-destructive` token styling on the host wrapper, matching the destructive-error pattern from `render_input` (KeyValueEditor's error rendering is the closest precedent — same Tailwind tokens). Empty `Option::None` → no error region rendered.
- **D-28:** `aria-label` on the editor host element derives from `label` if present, else from `name`. The host element is given `role="textbox"` and `contenteditable="true"` semantics by Quill itself at init — no need to pre-set them.
- **D-29:** `required: Option<bool>` (D-03): when true, the IIFE installs a guard that, on form submit, checks `quill.getText().trim().length > 0`. If empty, calls `event.preventDefault()` and surfaces the same destructive-token error region with a localized "Required" message (English literal in v1; i18n via ferro-lang is deferred — see Phase 147 / D-13 precedent).

### Claude's Discretion

- Exact Tailwind class lists for the editor wrapper, label, and error block — follow `render_input` / `render_key_value_editor` semantic-token conventions (no new tokens introduced).
- Whether to scope Quill assets in `ferro-json-ui/src/render.rs` next to the renderer or in a new `ferro-json-ui/src/assets/quill.rs` module — pick whichever matches the existing convention (the Map plugin keeps its asset constants inline next to the plugin struct, so inline next to `render_rich_text_editor` is the natural mirror).
- Whether the IIFE allowlist post-process uses `DOMParser` + tree-walk (cleaner) or regex (smaller) — DOMParser is preferred since the page already requires a browser DOM; regex sanitizers are bug factories.
- Test surface: follow the `render_key_value_editor` / `render_form` test pattern — assert on rendered HTML substrings; one render test for default formats, one for custom formats, one for `value` pre-fill, one for `error` rendering. Plus runtime-bundle tests asserting `setupRichTextEditor` is present (mirroring the existing `setupKeyValueEditor` assertions in `runtime/mod.rs:130, :162`).
- Whether to add an integration test that loads the rendered HTML through a headless DOM (`scraper` / `tl`) and asserts hidden-input naming — nice-to-have; not required if substring tests cover the same ground.
- Doc comment style on `RichTextEditorProps` and helpers — follow `InputProps` and `KeyValueEditorProps` doc tone (one-paragraph summary + per-field doc lines).

### Folded Todos

None — pending todos in STATE.md (workspace push, ferro-doctor multi-bin auto-resolve) are unrelated to this phase's scope.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap & cross-repo source
- `.planning/ROADMAP.md:1372-1389` — Phase 150 goal, source, dependencies, and 7 success criteria (the contract for this phase)
- `gestiscilo-it/app/.planning/REQUIREMENTS.md` (consumer requirements) — DOC-02, DOC-04 — note: this is the consumer-side spec referenced by the roadmap; the path is in the consumer repo and should be read if available locally during planning

### Component variant pattern this phase mirrors (KeyValueEditor / Phase 146)
- `ferro-json-ui/src/component.rs` — `KeyValueEditorProps` struct, `Component::KeyValueEditor` variant, serde tagged-enum match arms (Serialize ~line 1017, Deserialize ~line 1079)
- `ferro-json-ui/src/component.rs` — `ComponentNode::key_value_editor(...)` factory (search for `key_value_editor` in factories block, ~line 1240+)
- `ferro-json-ui/src/render.rs` — `render_key_value_editor()` and dispatch arm in `render_component` (~line 288)
- `ferro-json-ui/src/runtime/key_value_editor.rs` — canonical IIFE module shape (`pub(super) const SOURCE: &str`, vanilla ES5, `data-*` attribute selectors, event delegation)
- `ferro-json-ui/src/runtime/mod.rs:40` — bundle assembly (`s.push_str(key_value_editor::SOURCE);`)
- `ferro-json-ui/src/runtime/mod.rs:49` — dispatcher (`setupKeyValueEditor();`)
- `ferro-json-ui/src/runtime/mod.rs:130, :162` — runtime-bundle tests asserting setup-function presence
- `ferro-json-ui/src/lib.rs:103-153` — `COMPONENT_CATALOG` const; `### KeyValueEditor` section at `:145-147` is the format precedent
- `.planning/phases/146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va/146-CONTEXT.md` — full Phase 146 context (this phase's nearest sibling; read end-to-end)
- `.planning/phases/146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va/146-PLAN.md` through `146-03-PLAN.md` — wave structure (RED tests → Rust impl → runtime + serde) to emulate

### CDN/SRI asset pattern (Map plugin / v6.1 plugin pattern)
- `ferro-json-ui/src/plugins/map.rs:78-84` — `LEAFLET_CSS_URL` / `LEAFLET_JS_URL` / `LEAFLET_CSS_SRI` / `LEAFLET_JS_SRI` constants — exact shape Quill constants must follow
- `ferro-json-ui/src/plugins/map.rs:224-230` — `Asset::new(url).integrity(SRI).crossorigin("anonymous")` builder usage
- `ferro-json-ui/src/plugins/map.rs:432-454` — SRI assertion tests (CSS+JS both have integrity, both start with `sha256-` or `sha384-`)
- `ferro-json-ui/src/plugin.rs:1-100` — `Asset` struct + `JsonUiPlugin` trait (the existing CDN-asset injection pipeline this phase reuses)
- `ferro-json-ui/src/plugin.rs:108+` — `PluginRegistry` + page-level dedup mechanism

### DetailForm / Phase 147 precedent (component without runtime JS)
- `.planning/phases/147-detailform-component-for-inline-edit-ferro-json-ui/147-CONTEXT.md` — same shape of "add Component variant" work, but with no runtime module; useful for the component.rs / render.rs / lib.rs surface (skip its no-runtime decisions for this phase)

### Form integration (RichTextEditor lives inside `<form>`)
- `ferro-json-ui/src/render.rs` — `render_form` (~line 971-1031) — form action/method/method-spoofing pattern; relevant for IIFE submit interception (D-13)
- `ferro-json-ui/src/component.rs:232-263` — `InputProps`: example of a form field with `default_value`, `data_path`, `error`, `required` — the shape `RichTextEditorProps` mirrors

### Data-binding & escaping
- `ferro-json-ui/src/data.rs` — `resolve_path_string(data, path)` for `data_path` resolution
- `ferro-json-ui/src/render.rs` — `html_escape(&str)` — must be called on every dynamic value emitted into HTML attrs and text nodes (including `data-rte-*` attributes)

### MCP catalog
- `ferro-json-ui/src/lib.rs:103-153` — `COMPONENT_CATALOG` const (insert `### RichTextEditor` section)
- `ferro-mcp/src/tools/` — search for `json_ui_catalog` / `CatalogComponent` to find where ferro-mcp surfaces components; SC-6 / SC-7 require the new entry to appear here (planner: grep this directory and decide auto-derive vs manual entry)

### Project principles
- `.planning/PROJECT.md` §"Beauty as a design criterion" — conceptual coherence is a v1.0 gate; this phase's cross-cutting decision is **D-02** (first-class components reuse the plugin asset pipeline rather than introducing a parallel CDN path)
- `.planning/PROJECT.md` §"Continuous conceptual coherence" — every new feature asks whether it fits the existing surface or whether the surface needs to evolve. This phase evolves the asset-injection surface to support both plugins and first-class components (the surface needed to evolve; see D-02)
- `/Users/alberto/.claude/CLAUDE.md` §"Form Field Rules" — every form field needs a proper `default_value`. RichTextEditor's `value: Option<String>` is the equivalent (D-03); error paths use ferro's old-input restoration via the resolved hidden inputs
- `/Users/alberto/.claude/CLAUDE.md` §"Architecture Principles" — pre-1.0 / "always a feature branch": no backwards-compat shim required; add `RichTextEditor` directly. No deprecation cycle for the (non-existent) old rich-text mechanism

### Workspace conventions
- `framework/CLAUDE.md` §"Testing & Linting" — `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` MUST be green before commit (SC-6 explicit)
- `~/.claude/projects/-Users-alberto-repositories-albertogferrario-ferro/memory/feedback_ci_clippy_command_match.md` — match CI's exact clippy command, not just a local convenience version
- `.github/workflows/publish.yml` — when ferro-json-ui's version bumps for this phase, ensure the workflow's wave structure is correct (no new crate added — ferro-json-ui already exists in the right wave; planner verifies)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`Asset` struct + builder** (`ferro-json-ui/src/plugin.rs:23-53`): `Asset::new(url).integrity(SRI).crossorigin("anonymous")` — exact API for Quill JS+CSS assets. Reusing this is what makes D-02 work.
- **Plugin asset deduplication** (`ferro-json-ui/src/plugin.rs`): the existing per-page dedup-by-URL pipeline. Whatever mechanism the planner picks for D-07, it should plug into this rather than reinvent it.
- **`html_escape`** (`render.rs`): MUST wrap every dynamic attribute and text emission, including `data-rte-formats` (which holds JSON), `data-rte-placeholder`, etc.
- **`resolve_path_string`** (`data.rs`): for `data_path` resolution at render time (D-03's `data_path: Option<String>` field).
- **Tagged-enum serde helpers** (`component.rs`): `serialize_tagged(serializer, "RichTextEditor", p)` for Serialize; match arm `"RichTextEditor" => serde_json::from_value::<RichTextEditorProps>(value).map(Component::RichTextEditor)` for Deserialize.

### Established Patterns
- **Component variant with `JsonSchema`-able props**: `KeyValueEditorProps` derives `JsonSchema`. RichTextEditorProps follows (D-04) — props are flat, no `Action` / `ComponentNode` children, so the derive is clean.
- **`pub(super) const SOURCE: &str = r#"..."#;`**: the runtime-module shape — vanilla ES5, `var`-only, named function declarations, `data-*` attribute selectors, event delegation. `runtime/key_value_editor.rs` is the closest precedent.
- **CDN constants colocated with renderer**: `MapPlugin` keeps `LEAFLET_*_URL` / `LEAFLET_*_SRI` constants in `plugins/map.rs:78-84` — adjacent to the consumer code. Mirror for Quill in `render.rs` next to `render_rich_text_editor` (or in a focused `assets/quill.rs` if the planner prefers).
- **Test surface for renderer functions**: `render.rs` has substring-assertion tests for each `render_*` function. RichTextEditor follows: render-default, render-with-formats, render-with-value, render-with-error, plus the runtime-bundle assertions extending the existing `setupKeyValueEditor` pattern (`runtime/mod.rs:130, :162`).

### Integration Points
- **`Component` enum** (`component.rs` ~line 963): insert `RichTextEditor(RichTextEditorProps)` next to `KeyValueEditor` / form-family variants
- **`Component::Serialize`** (`component.rs` ~line 1017): add `Component::RichTextEditor(p) => serialize_tagged(serializer, "RichTextEditor", p)`
- **`Component::Deserialize`** (`component.rs` ~line 1079): add `"RichTextEditor"` match arm
- **`render_component` / `render_node` dispatch** (`render.rs` ~line 288): add `Component::RichTextEditor(p) => render_rich_text_editor(p, data)`
- **`runtime/mod.rs`**: push `rich_text_editor::SOURCE`, add `setupRichTextEditor();` to dispatcher; update bundle tests
- **`lib.rs` re-exports**: add `pub use component::RichTextEditorProps;`
- **`COMPONENT_CATALOG`** (`lib.rs:103-153`): add `### RichTextEditor` entry following the KeyValueEditor format (`lib.rs:145-147`)
- **`ComponentNode` factory** (`component.rs` ~line 1245): add `pub fn rich_text_editor(name: impl Into<String>, props: RichTextEditorProps) -> Self`
- **Page asset emission**: the cross-cutting integration point — wherever the existing renderer flushes plugin CSS/JS to `<head>` and pre-`</body>`, ensure the Quill assets get into the same flush. The planner's task is to find this site and decide D-02's mechanism.
- **ferro-mcp `CatalogComponent`** (in `ferro-mcp/`): planner greps for `CatalogComponent` / `json_ui_catalog` to determine whether to add a manual entry or rely on auto-derive; tests assert component count increments by 1.
- **`docs/src/json-ui/components.md`**: append `### RichTextEditor` section near `### KeyValueEditor`.
- **`framework/src/lib.rs`** re-exports (if user-facing): if the framework crate re-exports ferro-json-ui prop types, add `RichTextEditorProps` there too.

</code_context>

<specifics>
## Specific Ideas

- **The architectural novelty**: Phase 150 is the first first-class `Component::*` variant that requires CDN-loaded JS/CSS. KeyValueEditor (Phase 146) used inline runtime JS only; Map (the plugin precedent for CDN+SRI) lives in the parallel `JsonUiPlugin` system. Phase 150 unifies these — first-class components reuse the plugin asset pipeline. This is the v1-blocker conceptual-coherence move (PROJECT.md §"Continuous conceptual coherence"): the surface evolves to absorb the requirement rather than growing a parallel path.
- **Why two hidden inputs, not one**: `_delta` is the canonical, lossless record (round-trips through Quill perfectly). `_html` is the rendering input (cheap to display in lists, search-indexable, browser-trivially-renderable). Storing both removes the runtime cost of converting Delta → HTML on every read in the consumer app. Locked by SC-3.
- **Why `formats` is the single source of truth**: Quill maintains two parallel concepts — toolbar buttons and allowed formats. Letting them diverge means a button visible in the toolbar might produce content the post-process strips, or vice versa. `formats: Vec<String>` driving both at component-prop level (D-15) collapses these into one decision the caller makes once. SC-4's "consumer cannot bypass by mutating the DOM" is satisfied because the post-process runs server-trusted: even if the DOM had a `<img>` injected via devtools, the `_html` payload that reaches the server has it stripped.
- **Why no Rust-side HTML sanitizer**: Adding `ammonia` or similar to ferro-json-ui pulls in a sanitization library + html5ever for one component. The IIFE allowlist post-process is small and runs client-side; the server already trusts the IIFE output format (it's the same trust model as accepting form-encoded POSTs at all). If a future phase adds a server-side defense-in-depth pass (say, via a feature flag on `RichTextEditorProps`), it can layer on without breaking this contract.
- **Why CDN, not vendored**: SC-2 is explicit ("loaded from cdn.jsdelivr.net"). Vendoring Quill (~50 KB minified) into `ferro-base.css`-style static serving is a future optimization; the v1 contract is jsDelivr + SRI. The `ferro-base.css` pipeline (Phase 143 / v11.7) is for ferro's own emitted CSS, not third-party libraries.
- **Why Snow theme only**: Bubble theme has different DOM semantics (inline floating toolbar) that would diverge the IIFE wiring. Locking to Snow keeps the runtime module narrow; theme variation is deferred.
- **Why Quill 2.0.3 specifically**: 2.0.3 is the latest stable as of phase scoping; SRI-pinning means a version bump is a deliberate phase, which is correct (SRI hashes are part of the security contract). Quill 1.x is EOL.
- **Italian default labels are NOT used here**: unlike DetailForm (Phase 147) which had Italian-default button labels, RichTextEditor has no user-visible labels of its own — Quill's toolbar tooltips are English by default and customizing them is deferred (see `<deferred>`).

</specifics>

<deferred>
## Deferred Ideas

- **Image / video / file-upload toolbar buttons** — explicitly excluded by SC-4 (consumer apps cannot enable). A future "RichTextEditor with assets" phase could integrate `ferro-storage` for upload + return-URL flow; this is a substantial scope and belongs in its own phase.
- **Markdown ↔ Delta converters** — useful for syncing rich-text content with markdown-based knowledge bases. Defer until a real use case in gestiscilo or another consumer surfaces.
- **Inertia / SPA fetch-based submission** — current contract is full-page POST. A future Inertia-aware variant would intercept Inertia's submit pipeline. Not needed for gestiscilo Phase 125.
- **Multiple Quill themes (Bubble, custom)** — Snow only for v1. Bubble would require a different IIFE wiring; custom theme support means exposing CSS-token mappings.
- **Custom keyboard bindings per editor** — Quill's `keyboard` module is configurable but exposing it through props would expand the API surface significantly. Defer.
- **Mention / autocomplete module** — Quill 2.x has community modules for `@mention`-style triggers. Worth it only if a consumer asks; ferro keeps the v1 surface minimal.
- **Real-time collaboration (Yjs / shared state)** — fundamentally different runtime. Out of scope for v1; would be its own milestone.
- **Localized toolbar tooltips** — Quill 2.x supports a `tooltips` configuration; wire to `ferro-lang` once a similar precedent exists for any other component (DetailForm's deferred i18n is the same shape).
- **Server-side HTML sanitizer pass** — defense-in-depth on top of the IIFE allowlist. Add only if a real attack surface emerges (e.g., consumer apps that don't validate `_html` themselves). The `_delta` field is already the canonical record; a server-side pass on `_html` is purely a defense layer.
- **Vendoring Quill into static-served assets** — instead of jsDelivr CDN. Reduces external dep + offline-dev story; defer until the CDN dependency is actually a problem in practice.
- **Toolbar configuration override (beyond `formats`)** — exposing Quill's full `toolbar` config (groups, custom buttons, dropdown options) through props. Today the formats→toolbar mapping is internal (D-19). Open up only if a consumer needs it.
- **Per-instance theme override at component level** — today `theme: String` is a prop but only `"snow"` is supported. Promote when bubble lands.
- **Quill version bump** — `2.0.3` → newer. Each bump is its own phase (re-compute SRI, update tests).
- **Accessibility audit beyond Quill defaults** — a focused a11y phase if WCAG audits flag specific gaps; deferred.

### Reviewed Todos (not folded)

(STATE.md pending todos are unrelated to this phase: workspace push of v0.2.0, ferro-doctor multi-bin auto-resolve. Both belong elsewhere.)

</deferred>

---

*Phase: 150-ferro-json-ui-richtexteditor-component*
*Context gathered: 2026-04-29*
