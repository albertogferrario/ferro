# Phase 147: DetailForm component for inline edit — ferro-json-ui — Context

**Gathered:** 2026-04-23
**Status:** Ready for planning
**Mode:** `--auto` (single-pass, recommended defaults selected for all gray areas)
**Source:** gestiscilo Phase 111 design session (2026-04-22)

<domain>
## Phase Boundary

Add a `DetailForm` component to `ferro-json-ui`. The component renders the same structural container in two modes — View and Edit — driven by a server-side URL query param (`?mode=edit`). In View mode the component renders a description-list-style read-only block with a "Modifica" link; in Edit mode it renders the identical structural container wrapped in a `<form>`, where the leaf value cells are replaced with input components, and a "Salva"/"Annulla" action pair is rendered at the bottom.

The component owns the mode toggle: callers thread mode + fields + action in, the component renders the correct tree. This replaces a pattern where handlers branched on mode and assembled two different component trees — which delivered inline edit functionally but did not deliver structural coherence between the two states.

The `EditMode` enum (`View` | `Edit`) ships with the component in `ferro-json-ui`, exposing a `from_query()` constructor that parses the query param value.

**Primary files touched:**
- `ferro-json-ui/src/component.rs` — `DetailFormProps`, `DetailField`, `EditMode` types; `Component::DetailForm` variant; serde match arms
- `ferro-json-ui/src/render.rs` — `render_detail_form()` + dispatch arm
- `ferro-json-ui/src/lib.rs` — public re-exports; `COMPONENT_CATALOG` entry
- `ferro-json-ui/src/resolve.rs` — URL resolution pass for the inner `action` (mirrors `Component::Form`)

**Out of scope:**
- JS-based mode toggle (mode is a server-side query param — no client state)
- Conditional visibility per field based on mode (all fields render in both modes)
- Multi-step / tabbed edit flows
- Dirty-checking or unsaved-changes warning
- Optimistic UI updates
- Edit-only fields (e.g. password confirmation) — any such divergence is a caller decision on what to pass as `DetailField.input`
- Client-side validation
- Gestiscilo Phase 111 migration itself — that work happens downstream in gestiscilo after this phase ships

</domain>

<decisions>
## Implementation Decisions

### Core types

- **D-01:** `EditMode` lives in `ferro-json-ui` (same crate as the DetailForm component). Two variants: `View` (default) and `Edit`. `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]`, `#[serde(rename_all = "snake_case")]`. Default value is `EditMode::View`.
- **D-02:** `EditMode::from_query(raw: Option<&str>) -> Self` — returns `Edit` when raw equals `"edit"` (case-insensitive), `View` otherwise. Handlers call `EditMode::from_query(req.query("mode").as_deref())`.
- **D-03:** `DetailField` struct (public):
  - `label: String` — description term shown in both modes
  - `value: String` — display string shown in View mode (plain text, HTML-escaped at render)
  - `input: ComponentNode` — component rendered in Edit mode in place of `value`
  - `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` — `JsonSchema` skipped (contains `ComponentNode` which has custom serde; matches `Tab` / `FormProps` precedent)
- **D-04:** `DetailFormProps` struct (public):
  - `mode: EditMode` — which mode to render
  - `action: Action` — form submit target (required; used only in Edit mode)
  - `fields: Vec<DetailField>` — the rows
  - `edit_url: String` — href for "Modifica" button (used only in View mode); typically `"/resource/{id}?mode=edit"`
  - `cancel_url: String` — href for "Annulla" button (used only in Edit mode); typically `"/resource/{id}"`
  - `edit_label: Option<String>` — defaults to `"Modifica"` at render time
  - `save_label: Option<String>` — defaults to `"Salva"` at render time
  - `cancel_label: Option<String>` — defaults to `"Annulla"` at render time
  - `method: Option<HttpMethod>` — overrides `action.method` for the form tag (mirrors `FormProps.method`); unset means use `action.method`
  - `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` — `JsonSchema` skipped (contains `Action`, which has custom `method` handling, and `Vec<DetailField>` which contains `ComponentNode`)

### Structural coherence guarantee

- **D-05:** Both View and Edit modes render the **same outer HTML scaffold** — identical `<dl class="grid grid-cols-1 gap-4 ...">` container with one `<div><dt>…</dt><dd>…</dd></div>` per field. Only the contents of each `<dd>` differ:
  - View: `<dd>{html_escape(field.value)}</dd>`
  - Edit: `<dd>{render_node(&field.input, data)}</dd>`
- **D-06:** In Edit mode, the `<dl>` is wrapped by a `<form>` element (same form attributes as `render_form`: `action` URL, method, method spoofing for PUT/PATCH/DELETE). In View mode, no `<form>` wrapper.
- **D-07:** The action bar (Modifica in View, Salva+Annulla in Edit) renders **outside the `<dl>`** but **inside** the form element (Edit) or inside a parent wrapper div (View). Uses the same `flex gap-2` pattern as `render_form`'s trailing-button area.

### View-mode rendering

- **D-08:** `<dl>` structure mirrors `render_description_list`: `<dl class="grid grid-cols-1 gap-4">` with `<dt class="text-sm font-medium text-text-muted">` and `<dd class="mt-1 text-sm text-text">`. Reuses existing semantic tokens — no new styling vocabulary.
- **D-09:** "Modifica" button renders as an `<a>` link (not a button), targeting `edit_url`. Uses the existing `ButtonVariant::Outline` / secondary styling via `render_button`-style classes inline, so it matches the design system.
- **D-10:** `edit_url` is emitted verbatim (after `html_escape`) as the `href`. No URL resolution pass — callers build the URL with `?mode=edit` manually. This matches the spec and avoids over-abstracting simple query-param links.

### Edit-mode rendering

- **D-11:** Form element uses the same attributes and method-spoofing logic as `render_form` (see `ferro-json-ui/src/render.rs:971`):
  - `action` attr from `props.action.url` (resolver populates this); `"#"` fallback
  - `method` attr: GET for Get, POST for everything else
  - Hidden `<input type="hidden" name="_method" value="…">` for PUT/PATCH/DELETE
- **D-12:** Each `DetailField.input` is rendered via `render_node(&field.input, data)`, exactly as `render_form` renders its children. This means the full component surface is available — `Input`, `Select`, `Textarea`, `Switch`, `Checkbox`, plugins, etc.
- **D-13:** Input pre-fill is the caller's responsibility. Each input component uses its own `default_value` / `data_path`. `DetailField.value` is **not** threaded into the input's default — `value` is the View-mode display string only. This keeps rendering rules orthogonal to mode.
- **D-14:** "Salva" button renders as `<button type="submit">` inside the form, with primary variant styling. "Annulla" button renders as an `<a>` link next to Salva, targeting `cancel_url` — outline/secondary styling.

### Action resolution

- **D-15:** `Component::DetailForm(props)` participates in the resolver pass like `Component::Form`. The resolver populates `props.action.url` from `props.action.handler` (mirrors the arms in `ferro-json-ui/src/resolve.rs:46`, `:219`, `:399`).
- **D-16:** `edit_url` and `cancel_url` are **not** resolved — they are raw hrefs. If an app wants handler-based resolution for these, it can build them from its route registry and pass the resolved string.

### Serde integration

- **D-17:** `Component::DetailForm(DetailFormProps)` variant added to the `Component` enum. Serialized via `serialize_tagged(serializer, "DetailForm", p)`. Deserialized via a match arm in the custom `Deserialize` impl (`ferro-json-ui/src/component.rs:1079` pattern).
- **D-18:** `ComponentNode::detail_form(key, props)` constructor added (mirrors `ComponentNode::form`, `ComponentNode::input`, etc.) — `pub fn detail_form(key: impl Into<String>, props: DetailFormProps) -> Self`.
- **D-19:** `COMPONENT_CATALOG` entry added in `ferro-json-ui/src/lib.rs` — name `"DetailForm"`, description citing "split-mode detail page with inline edit".

### No runtime JS

- **D-20:** No entry added to `ferro-json-ui/src/runtime/`. Mode toggle is server-side (URL-driven). The component is pure HTML/CSS rendering.

### Claude's Discretion

- Exact Tailwind class lists for button styling and action-bar layout — reuse the idioms already present in `render_form` / `render_button`.
- Whether to emit a wrapping `<section>` or plain `<div>` around the whole component — pick whichever yields the clearest assertion surface for render tests.
- Whether the View-mode "Modifica" link sits above or below the `<dl>` — default to below, right-aligned, to match edit-mode's Salva/Annulla placement.
- Rust doc comments on the public types — follow `InputProps` / `FormProps` doc style.
- Tests: follow existing `render_*` test patterns in `render.rs` — assert on rendered HTML substrings; one test per mode at minimum, plus serde round-trip for `DetailFormProps` and `EditMode::from_query`.
- Whether `DetailField` ships a `DetailField::new(label, value, input)` convenience constructor — yes if it reduces call-site boilerplate to match the `ComponentNode::input(...)` ergonomic baseline.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Component patterns this phase mirrors
- `ferro-json-ui/src/component.rs:187-203` — `FormProps` struct: canonical shape for an action-bound form component (`action: Action`, `method: Option<HttpMethod>`)
- `ferro-json-ui/src/component.rs:434-440` — `DescriptionListProps` + `DescriptionItem`: canonical view-mode row shape (label/value pairs)
- `ferro-json-ui/src/component.rs:232-263` — `InputProps`: example of field-level form component with `default_value`, `data_path`, `error`
- `ferro-json-ui/src/component.rs:442-450` — `Tab` struct: precedent for "contains `Vec<ComponentNode>` so `JsonSchema` is skipped"
- `ferro-json-ui/src/component.rs:960-990` — `Component` enum: where `DetailForm(DetailFormProps)` variant is inserted
- `ferro-json-ui/src/component.rs:1017-1030` — `Component::Serialize` tagged-enum arms
- `ferro-json-ui/src/component.rs:1079-1115` — `Component::Deserialize` match arms
- `ferro-json-ui/src/component.rs:1240-1280` — `ComponentNode` factory constructors (`form`, `input`, `select`, etc.) — pattern for `detail_form(...)`

### Rendering patterns this phase mirrors
- `ferro-json-ui/src/render.rs:971-1031` — `render_form`: canonical form rendering with `action.url`, method spoofing, max-width wrapping
- `ferro-json-ui/src/render.rs:2427-2439` — `render_description_list`: canonical `<dl>/<dt>/<dd>` structure and Tailwind classes
- `ferro-json-ui/src/render.rs:288-310` — `render_component` / `render_node` dispatch arms

### Action type (required for form submit)
- `ferro-json-ui/src/action.rs:68-102` — `Action` struct + `Action::new`, `Action::get`, `Action::delete`
- `ferro-json-ui/src/action.rs` — `HttpMethod` enum for `method: Option<HttpMethod>` prop

### URL resolver integration (DetailForm participates here)
- `ferro-json-ui/src/resolve.rs:46`, `:219`, `:399` — `Component::Form(props)` resolver arms; DetailForm needs matching arms to resolve `props.action.url`

### Request integration
- `framework/src/http/request.rs:154` — `Request::query(&self, name: &str) -> Option<String>` — the caller pipeline for `EditMode::from_query(req.query("mode").as_deref())`

### Adjacent-phase precedent for "add component to ferro-json-ui"
- `.planning/phases/146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va/146-CONTEXT.md` — Phase 146 added KeyValueEditor; same shape of work (component.rs + render.rs + lib.rs re-export + catalog entry)
- `.planning/phases/146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va/146-PLAN.md` through `146-03-PLAN.md` — wave structure (RED tests → Rust impl → runtime) to emulate (skip the runtime wave for this phase)

### Project principles
- `.planning/PROJECT.md` §"Beauty as a design criterion" — conceptual coherence is a v1.0 gate; the DetailForm component exists to deliver this coherence at the view/edit boundary
- `/Users/alberto/.claude/CLAUDE.md` §"Form Field Rules" — every form field must have a proper `default_value` (D-13 defers this to caller-per-input, consistent with the rule)
- `/Users/alberto/.claude/CLAUDE.md` §"Architecture Principles" — "This is always a feature branch": no backwards compat layer required; add `DetailForm` directly

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `render_description_list` (render.rs:2427): the exact `<dl>/<dt>/<dd>` markup View mode should emit. Lift structure, do not call through — DetailForm has different contents per `<dd>`.
- `render_form` (render.rs:971): the exact `<form>` markup Edit mode should wrap around the `<dl>`, including method-spoofing for PUT/PATCH/DELETE. Method-spoofing block (render.rs:1001-1011) is copy-paste.
- `render_node(child, data)` (used throughout render.rs): canonical dispatch for rendering any `ComponentNode` — use it for rendering `DetailField.input` in Edit mode.
- `html_escape` (render.rs): must wrap every dynamic string emitted into HTML attrs and text nodes.
- `Action` + `Action::new(handler)` (action.rs:90): caller builds the submit action; resolver populates the URL.

### Established Patterns
- Serde tagged enum: `{"type": "DetailForm", ...}` via `serialize_tagged` + custom `Deserialize` match arm.
- `Component` variant with children-containing-props: `JsonSchema` derive is skipped (matches `Tabs`, `Form`, `TabsProps`). Add the explanatory comment `// JsonSchema skipped: contains Vec<DetailField> which contains ComponentNode`.
- Resolver pass: every component holding an `Action` or nested `ComponentNode`s gets three arms (one per resolve phase: URL resolution, data resolution, action resolution — see resolve.rs:46/219/399). DetailForm needs matching arms.
- Rendering tests assert on substrings of the rendered HTML (no snapshot framework), following `render_form` tests and phase 146's `render_key_value_editor` tests.

### Integration Points
- **`Component` enum** (component.rs ~line 963): insert `DetailForm(DetailFormProps)` alphabetically or grouped with form-family variants.
- **`Component::Serialize`** (component.rs ~line 1017): add `Component::DetailForm(p) => serialize_tagged(serializer, "DetailForm", p)` arm.
- **`Component::Deserialize`** (component.rs ~line 1079): add `"DetailForm" => serde_json::from_value::<DetailFormProps>(value).map(Component::DetailForm)` arm.
- **`render_component` / `render_node` dispatch** (render.rs ~line 288): add `Component::DetailForm(props) => render_detail_form(props, data)` arm.
- **Resolver** (resolve.rs:46, :219, :399): add `Component::DetailForm(props) => { ... }` arms that walk into `props.action` for URL resolution and `props.fields[i].input` for child-node resolution (mirror `Component::Form`).
- **`lib.rs` re-exports** (lib.rs:64-65 region): add `DetailFormProps`, `DetailField`, `EditMode` to the public prelude.
- **`COMPONENT_CATALOG`** (lib.rs): add `"DetailForm"` entry with a one-sentence description so MCP introspection surfaces it.
- **Public `ComponentNode` factory** (component.rs ~line 1245 region): add `ComponentNode::detail_form(key, props)`.

</code_context>

<specifics>
## Specific Ideas

- The coherence win the caller sees: a single `Component::DetailForm(...)` call in both View and Edit branches of the handler — no controller-side branching on mode, no two parallel rendering trees. The handler's only mode-awareness is `let mode = EditMode::from_query(req.query("mode").as_deref());` and passing it into `DetailFormProps`.
- Because `DetailField.input` is a full `ComponentNode`, the component is open-ended: a field can be an `Input`, `Select`, `Textarea`, `Switch`, `KeyValueEditor` (Phase 146), or any future component. The surface does not constrain the edit vocabulary.
- Gestiscilo origin (retained for context): the failed Phase 111 attempt collocated branching inside one handler with `DescriptionList` (view) and `Form` (edit). Same branching, just moved. This phase fixes that by moving the branch into the component and preserving the outer structural container.
- Italian default labels match gestiscilo's target audience. Projects targeting other locales override via `edit_label` / `save_label` / `cancel_label`. A future phase could introduce `ferro-lang` binding for these defaults, but that is deliberately deferred — hardcoded strings ship first.

</specifics>

<deferred>
## Deferred Ideas

- **i18n binding for default button labels via `ferro-lang`** — currently Italian literals; a future phase can wire them to translation keys once the ferro-lang→json-ui pattern is established for any other component.
- **Handler-based resolution for `edit_url` / `cancel_url`** — today they are raw strings. If multiple components grow a need for `Action::get`-style route-name references for simple navigation, generalize the resolver. Not needed now.
- **Per-field mode override** — e.g. "this field is always read-only even in Edit mode." Solve when a real use case surfaces; today, callers swap the `DetailField.input` for a read-only rendering (or just drop the field from the Edit view) at their own discretion.
- **Conditional mode toggle visibility** — e.g. only show "Modifica" button if the user has edit permission. Today the caller decides whether to render `DetailForm` at all based on authorization. A `can_edit: bool` prop is a possible future addition.
- **Nested sections / groups** — multi-section DetailForm (e.g. "Dati anagrafici" / "Contatti" groups). Today callers stack multiple `Component::DetailForm` instances. Section grouping is a future extension if the pattern recurs.
- **Form guards** — `FormProps.guard` exists; `DetailFormProps` intentionally omits it for v1. Revisit if a real guard case appears for detail forms.
- **Gestiscilo Phase 111 migration** — after this phase ships, gestiscilo removes `EditableField` / `editable_section` helpers and replaces the two-branch `build_informazioni_tab` rendering with a single `Component::DetailForm(...)` call. That work happens in the gestiscilo repo, not here.

</deferred>

---

*Phase: 147-detailform-component-for-inline-edit-ferro-json-ui*
*Context gathered: 2026-04-23*
