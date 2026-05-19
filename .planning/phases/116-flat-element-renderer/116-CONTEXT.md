# Phase 116: Flat Element Renderer - Context

**Gathered:** 2026-04-18
**Status:** Ready for planning
**Mode:** `--auto` — decisions auto-selected for a well-designed implementation inspired by Vercel json-render (flat-map dispatch), the ported v1 renderer (8057 LOC, 50 per-component functions), and Airbnb/DoorDash/Lyft SDUI slot-map conventions.

<domain>
## Phase Boundary

Replace the Phase 115 placeholder renderer in `ferro-json-ui/src/render.rs` with the real flat-element walker. `render_spec_to_html(&Spec, &Value)` and `render_spec_to_html_with_plugins(&Spec, &Value)` must:

- Walk `spec.elements` by ID starting at `spec.root` (no tree recursion across `Vec<ComponentNode>`).
- Dispatch per-element to a typed renderer by `type_name`, with a plugin-registry fallback.
- Render every component type the v1 renderer supported (~30 built-ins + plugins) against the v2 type-erased Element shape.
- Honor `Element.action.url` (pre-resolved) and `Element.visible` (evaluated inline).
- Handle missing children gracefully (skip + observable warning, never panic).
- Collect plugin assets via a parallel pass over `spec.elements`.
- Produce output HTML byte-for-byte compatible with v1 for all test-covered component shapes (so existing framework/gestiscilo golden tests stay green once ported).

**What this phase does NOT do** (locked by roadmap, do not re-open): JSON Schema / catalog-driven semantic validation (Phase 117), `$data` / `$template` expression evaluation (Phase 118), page loader and spec file caching (Phase 119), CLI/MCP tool updates (Phase 120), docs rewrite and gestiscilo field test (Phase 121), changes to `Spec` / `Element` struct shape (locked by Phase 115).

</domain>

<decisions>
## Implementation Decisions

### Dispatch architecture

- **D-01: Single `match el.type_name.as_str()` dispatch.** ~30 arms for built-ins plus a default arm that consults the plugin registry. Matches Vercel json-render's switch-by-type pattern and keeps each per-component renderer's body nearly verbatim from v1.
- **D-02: Reject the trait-object dispatch alternative.** A `HashMap<&str, Box<dyn Renderer>>` or plugin-style registry for built-ins would lose niche optimization, add indirection, and impose a props-agnostic function signature that forces a `serde_json::Value` deserialize inside each renderer anyway. Symmetry with plugins is not a runtime goal — the plugin system is for external extension, built-ins are compile-time-known.
- **D-03: Dispatch default arm is `with_plugin(type_name, |p| p.render(&el.props, data))`.** If neither built-in nor plugin matches, emit an HTML comment diagnostic (see D-10) and render nothing.
- **D-04: Per-element pipeline, in order:**
  1. Evaluate `el.visible` — if false, return `""` without walking children.
  2. Match `el.type_name` → dispatch to typed renderer.
  3. Typed renderer: deserialize `el.props` into the component's `*Props` struct, then emit HTML. Children are resolved by ID lookup inside the renderer.

### Element graph and slot binding

- **D-05: `Element.children` is the graph-canonical primary slot.** Single-slot container components (`Form`, `FormSection`, `Grid`, `Collapsible`, `ButtonGroup`, `PageHeader` breadcrumb area) render `Element.children` in order. Phase 115's parse-time validator guarantees every ID resolves, no cycles, depth ≤ 3.
- **D-06: Multi-slot components carry slot-specific `Vec<String>` ID lists in their Props.** Phase 116 re-adds these fields (lost during the Phase 115 v1-type strip):
  - `CardProps.footer: Vec<String>` — body comes from `Element.children`, footer from props.
  - `ModalProps.footer: Vec<String>` — body from `Element.children`, footer from props.
  - `Tab { value, label, children: Vec<String> }` — per-tab children inside `TabsProps.tabs[i]`.
  - `KanbanColumnProps.children: Vec<String>` — per-column children inside each column.
  - `PageHeaderProps.actions: Vec<String>` — action buttons rendered to the right of title.
  IDs in slot lists reference `spec.elements` directly (same lookup as `Element.children`). They do NOT need to be duplicated into `Element.children` — multi-slot elements may keep `Element.children` empty.
- **D-07: Slot IDs are NOT covered by Phase 115's structural validator.** Known Phase 116 limitation: `Spec::from_json` validates cycles/depth/dangling only on `Element.children`. Slot-borne dangling references surface at render time via D-10 graceful handling. Full slot-ID graph validation is a Phase 117 catalog concern (catalog knows props shape, can walk slots).
- **D-08: No Element struct changes.** Phase 115 shipped `Element { type_name, props, children, action, visible }` and it is frozen. Everything multi-slot lives inside typed Props (or the plugin's untyped `props: Value`).

### Graceful failure surface

- **D-09: Renderer is infallible — returns `String`, never `Result`.** SDUI production practice: authoring mistakes must degrade gracefully because specs may be AI-generated or loaded from files at runtime. A broken element never takes down the page.
- **D-10: Diagnostic surface is HTML comments.** Zero new dependencies (no `tracing` / `log`), observable in browser devtools, and preserves the v1 render-returns-String contract. Emitted cases:
  - Missing slot/children ID: `<!-- ferro-json-ui: element 'parent' references missing child 'ghost' -->`
  - Unknown `type_name` (not a built-in, not in plugin registry): `<!-- ferro-json-ui: unknown component type 'Foo' -->`
  - Props deserialization failure: `<!-- ferro-json-ui: failed to decode Card props on element 'hero': missing field 'title' -->`
  - Render-time cycle tripwire (defensive): `<!-- ferro-json-ui: cycle guard tripped at depth N — spec should have been rejected at parse time -->`
- **D-11: Defense-in-depth cycle guard.** Each `render_element(id, …)` call increments a depth counter passed through the walker; if depth exceeds `MAX_NESTING_DEPTH + 1` (= 4) it emits D-10 comment and returns `""`. Phase 115 guarantees acyclic + ≤3 deep at parse time, so this tripwire fires only for specs that bypassed `from_json` (hand-built Specs from callers that skipped validation). Cheap safety net.
- **D-12: `serde_json::from_value::<TProps>(el.props.clone())` is the standard decode step.** On `Err`, emit D-10 comment and return `""`. On `Ok`, proceed with typed render. No `.unwrap_or_default()` or silent degradation — authors get a visible diagnostic.

### Visibility

- **D-13: Inline per-element evaluation.** `render_element` consults `el.visible` before dispatching. Visibility is evaluated against `data` using the existing `Visibility::evaluate(&Value) -> bool` semantics from `visibility.rs`. No separate pre-pass that mutates the spec.
- **D-14: Invisible elements are skipped entirely — their children are not rendered.** Matches React conditional render semantics. If an invisible element is the root, the output is empty with a single HTML comment `<!-- ferro-json-ui: root hidden -->`.

### Action resolution

- **D-15: Renderer assumes actions are pre-resolved.** `Element.action.url` must be `Some(...)` by the time `render_spec_to_html` is called. Callers invoke `resolve_actions(&mut spec, resolver)` (already in `resolve.rs`) before render. This is the existing contract — `framework/src/json_ui/mod.rs::JsonUi::render` already does the resolve + render sequence.
- **D-16: Actions with `url = None` at render time degrade to `href="#"` + diagnostic comment.** Matches v1 behavior; avoids broken forms. Strict callers use `resolve_actions_strict` upstream.

### Plugin rendering and asset collection

- **D-17: Plugin fallback in the dispatch default arm.** Reads `with_plugin(type_name, |p| p.render(&el.props, data))`. Plugins operate on the raw `serde_json::Value` props (no typed layer) — their contract is unchanged from v1.
- **D-18: Plugin asset collection is a separate pass over `spec.elements`.** Replaces v1's `collect_plugin_types(view)` recursive tree walk with `collect_plugin_types(spec)` flat-map walk. Returns `HashSet<String>` of plugin type names present in the spec (after subtracting the built-in type list). `collect_plugin_assets` is reused as-is.
- **D-19: Plugins are identified by "type_name not in BUILTINS AND present in plugin registry."** The built-in list is maintained as `const BUILTIN_TYPES: &[&str]` in `render/mod.rs` alongside the dispatch match — a single source of truth. If `type_name` matches a built-in, it is never treated as a plugin (even if a plugin registers the same name — plugins cannot shadow built-ins).

### Module layout

- **D-20: Split `render.rs` into a `render/` directory.** v1 was 8057 LOC in a single file — unwieldy. Phase 116 lands:
  - `render/mod.rs` — public API (`render_spec_to_html`, `render_spec_to_html_with_plugins`, `RenderResult`), `BUILTIN_TYPES` constant, dispatch match, `render_element(id, spec, data, depth) -> String`, plugin asset collection, HTML helpers (`html_escape`), tag emitters (css/js asset rendering).
  - `render/containers.rs` — Card, Modal, Tabs, KanbanBoard, PageHeader, FormSection, Grid, Collapsible, ButtonGroup.
  - `render/form.rs` — Form, Input, Select, Checkbox, Switch.
  - `render/data.rs` — Table, DataTable, DescriptionList, Pagination, CalendarCell.
  - `render/atoms.rs` — Text, Button, Badge, Alert, Separator, Progress, Avatar, Image, Skeleton, Breadcrumb, EmptyState, StatCard, Checklist, Toast, NotificationDropdown, Sidebar, Header, DropdownMenu, ActionCard, ProductTile.
  Each file <2000 LOC. All per-component renderers take `(props: &TProps, el: &Element, spec: &Spec, data: &Value, depth: usize) -> String` so every container can recurse via `render_element(child_id, spec, data, depth + 1)`.
- **D-21: Per-component renderer bodies port verbatim from v1 where possible.** The v1 renderer is the canonical HTML emission — it produced UI that gestiscilo depended on in production. Phase 116's value-add is the walker and dispatch surface, not redesigning each component's HTML. Git ref for the v1 code is `40385f32^:ferro-json-ui/src/render.rs` (8057 LOC, all ~50 render functions intact).
- **D-22: `html_escape` stays in `render/mod.rs`** (already there in the placeholder; D-20 keeps it). Exported `pub(crate)` — every renderer uses it.

### Testing

- **D-23: Port every v1 renderer test.** The Phase 115-02 commit (`c88745a4`) deleted v1's inline render tests alongside the v1 types. Their golden assertions against HTML output are still the right contract — port each one to v2 by rewriting construction from `JsonUiView::new()...` to `Spec::builder()...`. Target: one `#[test]` per component type minimum.
- **D-24: New tests specifically for Phase 116's flat-walker behavior:**
  - Missing slot child (e.g., `Card.footer` references ID not in spec) emits diagnostic comment, does not panic.
  - Unknown `type_name` (no built-in, no plugin) emits diagnostic comment, does not panic.
  - `Element.visible = Some(condition that evaluates false)` skips the element and its children.
  - `Element.visible` on the root renders empty with root-hidden comment.
  - Plugin dispatch: register a test plugin, construct a spec using its type name, assert render output includes the plugin's HTML and asset collection returns its CSS/JS URLs.
  - Action URL inlining: spec with `Action { handler, url: Some("/x") }` renders a `<form action="/x">` or `<a href="/x">` as appropriate.
  - Action not-yet-resolved (`url: None`): falls back to `href="#"` with diagnostic comment.
  - Cycle tripwire: construct a Spec bypassing `from_json` validation with a 5-level chain; render terminates with tripwire comment (depth guard works).
- **D-25: Integration tests at framework level.** Port the existing `framework/src/json_ui/mod.rs` tests that currently assert on the placeholder (`assert!(html.contains("v2 render pipeline arrives in Phase 116"))`) to assert the real HTML output — at minimum, confirm 200 status, non-empty HTML, expected component markers, plugin asset block presence when applicable.
- **D-26: Snapshot-style fixture tests welcome but not required.** If the v1 test suite had JSON-input → HTML-output fixtures, port them. Otherwise, inline construction per test is fine — this phase is about surface correctness, not snapshot-coverage.

### Performance

- **D-27: No optimization work beyond "O(n) walk."** `HashMap::get(&id)` is O(1); `render_element` visits each element at most once (the DAG shape allows shared children, but for Phase 116 we call each child render every time it is referenced — duplicate rendering is fine and matches v1 semantics).
- **D-28: Defer memoization / render-cache to post-v1.0.** Production gestiscilo page load (tens of elements) is not a bottleneck; any caching adds invalidation complexity for marginal gain.

### Out-of-scope reminders

- **D-29: `$data` / `$template` expressions are NOT evaluated in Phase 116.** If props contain `{"$data": "/some/path"}` as a value, the renderer treats it as a literal JSON object. Phase 118 adds the expression resolver as a pre-render pass. Do not preemptively add expression handling — inner-platform-effect risk per Phase 115 research.
- **D-30: Catalog validation is NOT called from the renderer.** Malformed specs are not rejected before render — they surface via D-10 diagnostics. Phase 117 adds `Catalog::validate(&spec)` as an explicit pre-flight step callers can opt into; the renderer itself stays schema-unaware.
- **D-31: `framework/src/json_ui/mod.rs::JsonUi::render` signature is unchanged.** Phase 115 already switched it to `&Spec`. Phase 116 only changes what `render_spec_to_html_with_plugins` does internally.

### Claude's Discretion

- Tracing/log dep addition is deliberately rejected (D-10) — if operational experience later shows HTML-comment diagnostics are insufficient, add `tracing = "0.1"` in a follow-up phase.
- Exact split point between `render/data.rs` and `render/containers.rs` for borderline components (e.g., DescriptionList sits between data and layout) — pick whatever keeps file sizes balanced.
- Whether per-component render functions live as `pub(crate) fn render_card(...)` in their section file or as private `fn render_card(...)` with a module-level re-export — pick the Rust-idiomatic choice (likely `pub(crate)` so the dispatch match can call them from `render/mod.rs`).
- Whether to introduce a small `Walker` struct carrying `(spec: &Spec, data: &Value, depth: usize)` through every call vs. passing three args explicitly — pick whichever reads cleaner after the port.
- Whether `render_spec_to_html` returns an HTML-comment diagnostic when `spec.root` is somehow missing from `spec.elements` (shouldn't happen post-from_json) or just returns empty — defensive skip is fine.
- Diagnostic comment exact wording — consistency matters more than the specific phrase.
- Whether to keep `render_spec_to_html_with_plugins` signature returning `RenderResult` or add a new `render_to_bytes` / `render_to_writer` variant — NOT for Phase 116.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and success criteria
- `.planning/ROADMAP.md` §"Phase 116: Flat Element Renderer" — goal, depends-on, requirements (RENDER-01/02/03), 6 success criteria.
- `.planning/ROADMAP.md` §"v12.0 JSON-UI v2" milestone preamble — overall milestone context.

### Upstream Phase 115 decisions (locked — do not re-open)
- `.planning/phases/115-spec-v2-data-structures/115-CONTEXT.md` — Spec/Element shape, type-erasure model, parse-time validation contract, SCHEMA_VERSION.
- `.planning/phases/115-spec-v2-data-structures/115-VERIFICATION.md` — what Phase 115 actually shipped (7 success criteria confirmed).
- `.planning/phases/115-spec-v2-data-structures/115-RESEARCH.md` — SDUI domain research (Vercel json-render, JSON Forms, rjsf, Airbnb/DoorDash/Lyft patterns) that informs Phase 116's dispatch and slot choices.

### Downstream constraints (read to avoid painting into a corner)
- `.planning/ROADMAP.md` §"Phase 117: Catalog & JSON Schema" — catalog validation comes AFTER render; Phase 116 must not couple to catalog.
- `.planning/ROADMAP.md` §"Phase 118: Server-Side Expressions" — `$data`/`$template` resolve BEFORE render; Phase 116 does not handle them.
- `.planning/ROADMAP.md` §"Phase 119: Page Loader" — render is called AFTER load+merge; signature stays `(&Spec, &Value) -> String`.

### ferro-json-ui source (what Phase 116 rewrites and what it preserves)
- `ferro-json-ui/src/render.rs` — current Phase 115 placeholder (~95 LOC). **Replaced by `render/` directory in Phase 116.**
- `ferro-json-ui/src/spec.rs` — `Spec`, `Element`, `SpecError`, `MAX_NESTING_DEPTH`, `SCHEMA_VERSION`. **Unchanged; render reads these.**
- `ferro-json-ui/src/component.rs` — ~30 `*Props` structs. **Phase 116 re-adds slot `Vec<String>` fields to CardProps, ModalProps, Tab, KanbanColumnProps, PageHeaderProps per D-06.** Other structs unchanged.
- `ferro-json-ui/src/resolve.rs` — `resolve_actions`, `resolve_errors`. **Unchanged; callers run these before render.**
- `ferro-json-ui/src/visibility.rs` — `Visibility` enum, `VisibilityCondition`, `VisibilityOperator`. **Unchanged; `visible.evaluate(&data)` called inline per D-13.**
- `ferro-json-ui/src/action.rs` — `Action`, `ActionOutcome`, `ConfirmDialog`, `HttpMethod`. **Unchanged; renderer reads `el.action.url`.**
- `ferro-json-ui/src/data.rs` — `resolve_path`, `resolve_path_string` (currently `#[allow(dead_code)]`). **Phase 116 drops the `allow(dead_code)` and uses these for data_path / value pre-fill in Input/Select/Table/DataTable renderers.**
- `ferro-json-ui/src/plugin.rs` — `JsonUiPlugin` trait, `PluginRegistry`, `with_plugin`, `collect_plugin_assets`, `Asset`, `CollectedAssets`. **Unchanged; dispatch default arm uses `with_plugin`.**
- `ferro-json-ui/src/layout.rs` — layout chrome, `LayoutContext`. **Unchanged; layout wraps render output at framework level.**
- `ferro-json-ui/src/lib.rs` — re-exports. **Phase 116 keeps `render_spec_to_html`, `render_spec_to_html_with_plugins`, `RenderResult` public; renames/removes nothing.**
- `ferro-json-ui/src/runtime/` — client-side IIFE bundle for tabs, forms, dialogs. **Unchanged; rendered HTML includes the existing `data-*` attributes the runtime consumes.**

### Framework integration
- `framework/src/json_ui/mod.rs` — `JsonUi::render`, `render_with_config`, `render_json` entry points. Already wired to `render_spec_to_html_with_plugins(&Spec, &Value)` (Phase 115-03). **Phase 116 changes are transparent here — signatures stay the same.**
- `framework/src/json_ui/mod.rs` tests — currently assert on the Phase 115 placeholder marker string. **Phase 116 rewrites these to assert real component markers.**

### v1 renderer source (the canonical HTML reference to port from)
- Git ref `40385f32^:ferro-json-ui/src/render.rs` — 8057 LOC, ~50 `render_*` functions. Retrieve with `git show 40385f32^:ferro-json-ui/src/render.rs`. **Each per-component renderer body is the port target for Phase 116.** Do not redesign component HTML — the gestiscilo app depends on byte-level stability for a handful of components (grid breakpoints, card hover states, form ARIA wiring).
- Git ref `40385f32^:ferro-json-ui/src/component.rs` — v1 Props shape including the slot fields (CardProps.children/footer, Tab.children, KanbanColumnProps.children) that Phase 116 re-introduces as `Vec<String>` per D-06.

### Domain research (informed Phase 116 architectural choices)
- Vercel json-render (13k★ GitHub, Jan 2026) — flat `elements` map + root pointer + switch-by-type dispatch. **Phase 116's D-01 dispatch and D-05/D-06 slot model mirror this.**
- Airbnb Ghost / DoorDash Mosaic / Lyft component kit — slot-map-on-element convention. **Phase 116 chooses typed-props-slot over generic slot-map because Phase 115 already froze `Element` and props are where typed renderers already look.**
- rjsf (react-jsonschema-form) perf notes — large `oneOf` is slow. **Informs Phase 117 more than Phase 116; mentioned here because slot validation defers to Phase 117.**

### Workspace conventions
- `CLAUDE.md` (project root) — Testing & Linting invocation (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`), no co-author lines in commits, update ferro-mcp when framework behavior changes.
- `ferro-json-ui/CLAUDE.md` if present (check during planning; otherwise rely on project root).
- `.planning/codebase/CONVENTIONS.md` — crate conventions, builder patterns, error types.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (Phase 116 consumes as-is)
- `spec::Spec`, `spec::Element`, `spec::MAX_NESTING_DEPTH` — the input contract.
- `action::Action` with `url: Option<String>`, `method: HttpMethod`, `confirm: Option<ConfirmDialog>`, `on_success`/`on_error: Option<ActionOutcome>`, `target: Option<String>` — all already shaped for the renderer's needs.
- `visibility::Visibility` — has an `evaluate(&Value) -> bool` method (or equivalent). Verify during planning and port the call.
- `data::resolve_path`, `data::resolve_path_string` — drop the `#[allow(dead_code)]` in Phase 116.
- `plugin::with_plugin`, `plugin::collect_plugin_assets`, `plugin::Asset`, `plugin::CollectedAssets` — no change needed; dispatch uses them directly.
- `resolve::resolve_actions`, `resolve::resolve_errors` — callers invoke before render; renderer relies on the post-resolve state.
- Every `*Props` struct in `component.rs` — renderer deserializes `el.props` into these. Phase 116 adds slot `Vec<String>` fields (D-06) to five of them.
- `render::RenderResult` struct (`html`, `css_head`, `scripts`) — public API, unchanged.
- `render::html_escape` helper — relocated into `render/mod.rs`, same semantics.

### Patterns to Replicate
- **v1 renderer's per-component function signature** — `fn render_card(props: &CardProps, data: &Value) -> String`. Phase 116 shifts to `fn render_card(props: &CardProps, el: &Element, spec: &Spec, data: &Value, depth: usize) -> String` so children lookup is possible. The extra params add to the call site but are free at construction (plain references).
- **v1 class-name output** — Tailwind utility classes copied from `40385f32^:ferro-json-ui/src/render.rs` verbatim. Do not reinvent styling.
- **Plugin CSS/JS injection pattern** — `render_css_tags` and `render_js_tags` helpers in v1 render.rs (lines ~200–250) — port verbatim to `render/mod.rs`.

### Integration Points
- `framework/src/json_ui/mod.rs::JsonUi::render` calls `render_spec_to_html_with_plugins` and wraps the returned `RenderResult` in the layout's head/body/scripts slots. This wiring is already in place from Phase 115-03.
- `ferro-mcp` tools that exercise rendering (`render_projection`, some view-introspection tools) are independent of Phase 116's internals — they consume `Spec` and call through the same `JsonUi::render` path. No MCP changes expected for Phase 116 (those are Phase 120).
- gestiscilo application at `app/` — any handler that renders a real view will exercise the new walker. Sample `app/` pages should render identically to v1 by the end of Phase 116; visual regressions are the acceptance bar.

### Non-obvious v1 behaviors to preserve
- **Card's horizontal wrap wrapper:** v1 wraps top-level components in `<div class="flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto">`. Phase 116's top-level `render_spec_to_html` must emit the same wrapper around the rendered root.
- **Tabs auto-hide when only one tab** — single-tab panes skip the tab bar and render children directly (v1 render_tabs early-return). Port the same behavior.
- **Tabs server-driven fallback** — empty-children tabs render as `<a href="?tab=...">` links for full-page reload; tabs with children render as client-side `<button data-tab="...">` triggers. Port the logic.
- **Form/Switch auto-submit forms** — Switch with an `action` prop wraps itself in a `<form>` that submits on change. Port faithfully.
- **Input/Select/Checkbox data_path pre-fill** — `data_path` resolves against `&Value data` to produce the `value=""` / `checked` attribute. Uses `data::resolve_path_string`.
- **DataTable URL templating for row_actions** — v1 renderer rewrites `{id}` placeholders in action URLs using each row's data. Port verbatim.

### Non-obvious v1 behaviors that MUST change
- **Tree-recursive child access** — v1 reads `props.children: Vec<ComponentNode>` directly. Phase 116 reads `Element.children: Vec<String>` (for single-slot containers) or `TProps.slot_field: Vec<String>` (for multi-slot) and looks up IDs via `spec.elements.get(id)`. Missing lookups emit D-10 diagnostic.
- **Panic-on-Component::Plugin** vs. type-erased dispatch — v1 had `Component::Plugin(PluginProps { plugin_type, props })` as an enum variant. v2 has no enum variant — plugins are ordinary Elements whose `type_name` misses the built-in match and resolves in the plugin registry.
- **Asset collection tree walk** — v1's `collect_plugin_types_node` recursed through every typed children collection (Card.children, Form.fields, Modal.footer, Tab.children, KanbanColumn.children, etc.). v2's collection is a flat pass over `spec.elements.values()` checking each `type_name` against built-ins — O(n) and much simpler.

</code_context>

<specifics>
## Specific Ideas

- **The v1 renderer is the spec.** ~8000 lines of HTML emission that gestiscilo production depends on. Phase 116's job is to wrap that logic in a flat walker, not redesign it. When in doubt during implementation, port verbatim from `git show 40385f32^:ferro-json-ui/src/render.rs` and change only the signature / child-lookup site.
- **Slot re-addition is the one place component.rs grows.** Every other Phase 116 change is in the renderer. The five slot fields (CardProps.footer, ModalProps.footer, Tab.children, KanbanColumnProps.children, PageHeaderProps.actions) are the sharp edge that makes the flat model expressive enough to replace v1 feature-for-feature.
- **HTML-comment diagnostics are load-bearing.** They're the difference between "Spec is broken and the page is blank with no clue why" and "Spec is broken and the author can see exactly which ID missed in devtools." This is a deliberate design choice informed by Vercel json-render's inline error surface. No logging infra, no error callbacks — just a comment in the DOM.
- **Depth-guard cycle tripwire is defense-in-depth, not primary validation.** Phase 115 already rejects cycles and ≥4-deep specs at parse time. The render-time guard exists for hand-built Specs that bypass `from_json` (e.g., constructed via mutation after `builder().build()`). It should fire <0.01% of the time in production — its value is preventing a stack overflow if it ever does.
- **Match statement size is a feature.** ~30 arms in one match block looks long but is the clearest dispatch in Rust — every built-in type is one search away, and adding a new built-in is one arm. Refactoring it into a macro or a derive would obscure the most-edited code in the crate.
- **`Element.visible` semantics mirror React's `{condition && <X/>}`, not CSS `display:none`.** Invisible elements do not emit HTML at all (no placeholder, no skeleton). This matters for SEO and accessibility — screen readers don't see hidden-but-rendered content that way.
- **Action URL inlining reuses the existing `resolve::resolve_actions` pipeline** — Phase 116 intentionally does not reimplement URL generation; it is a strict downstream consumer of `el.action.url`.
- **`render_element(id, spec, data, depth)` is the one recursive function in the entire renderer.** Every container delegates child rendering back to it. This is the architectural center — all dispatch, diagnostic, visibility, and cycle-guard logic lives there.

</specifics>

<deferred>
## Deferred Ideas

- **Render-cache / memoization** — Phase 116 renders each Element once per appearance; shared children are re-rendered per reference. Caching adds invalidation complexity; defer to post-v1.0 perf pass if profiling shows it matters.
- **Streaming renderer** (write-into-`&mut String` vs. allocate-per-call) — v1 uses String allocations throughout; Phase 116 preserves this. A `Write`-based variant is a post-v1.0 optimization.
- **`tracing` / `log` integration** — HTML-comment diagnostics are the Phase 116 observability surface. If ops experience shows they are insufficient, add `tracing = "0.1"` in a follow-up phase.
- **Catalog-driven validation before render** — Phase 117.
- **`$data` / `$template` expression resolution** — Phase 118.
- **Spec hot-reload / file-watcher** — Phase 119.
- **CLI + MCP updates for v2** — Phase 120.
- **Docs rewrite** — Phase 121.
- **Full slot-ID graph validation** (validating IDs referenced in props slot lists) — Phase 117 catalog concern; Phase 116 accepts the known gap and surfaces violations via D-10 diagnostics at render time.
- **`render_to_writer<W: Write>` API** — would enable zero-alloc streaming; not a v12.0 goal.
- **Client-side React/Vue/Svelte runtime that consumes the same Spec JSON** — the server-authoritative HTML-rendering stance is a deliberate Phase 115 decision; client runtime lives outside the v12.0 scope.
- **Per-element render instrumentation / profiling hooks** — post-v1.0 DX work.
- **Sandboxing untrusted Specs** (Spec from external sources) — not a current threat model; if it becomes one, the HTML-escape pass already on content fields is the starting point.

</deferred>

---

*Phase: 116-flat-element-renderer*
*Context gathered: 2026-04-18*
*Mode: --auto*
