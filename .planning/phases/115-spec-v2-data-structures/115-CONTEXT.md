# Phase 115: Spec v2 Data Structures - Context

**Gathered:** 2026-04-18
**Status:** Ready for planning
**Mode:** `--auto` — decisions auto-selected for a well-designed implementation inspired by Vercel json-render, JSON Forms, rjsf, and Airbnb/DoorDash/Lyft SDUI patterns.

<domain>
## Phase Boundary

Introduce the v2 type foundation in `ferro-json-ui`:
- `Spec` struct: `{ $schema, root, elements, title?, layout?, data? }` — flat element map keyed by ID.
- `Element` struct: `{ type_name, props, children, action?, visible? }` — type-erased, props stored as `serde_json::Value`, children as `Vec<String>` referring into the map.
- `Spec::from_json()` with parse-time structural validation (root exists, child refs resolve, no cycles, depth ≤ 3, ID format).
- Manual `JsonSchema` scaffolding where derive is blocked by custom ser/de or recursion — just enough for Phase 117 to build on.
- Delete v1 types (`JsonUiView`, `ComponentNode` wrapper, `Component` enum, custom Serialize/Deserialize for Component).
- Rewrite callers (`framework/src/json_ui/mod.rs`, sample `app`, `ferro-mcp` templates) with a placeholder Phase 115 renderer so the workspace stays green. The real v2 render pipeline is Phase 116's job.

**What this phase does NOT do** (enforced by the roadmap, not re-discussed here): JSON Schema validation of semantics (Phase 117), `$data`/`$template` expression evaluation (Phase 118), page loader (Phase 119), AI tool updates (Phase 120), docs and field test (Phase 121).

</domain>

<decisions>
## Implementation Decisions

### Type Foundation

- **D-01: Element is fully type-erased.** `type_name: String` + `props: serde_json::Value`. No built-in vs plugin distinction at the type level; distinction is resolved only at catalog/render time (Phase 117). This kills the v1 `Component::Plugin` escape-hatch variant entirely and mirrors Vercel json-render's flat model.
- **D-02: `Component` enum and `ComponentNode` wrapper are deleted.** The 40-variant match and custom Serialize/Deserialize in `component.rs` go away. Elements are indistinguishable at the Spec level except by their string `type_name`.
- **D-03: Typed `*Props` structs survive as the validation source of truth.** CardProps, TableProps, FormProps, etc. remain as Rust types carrying `#[derive(JsonSchema)]` wherever derive works. Phase 117's Catalog reflects on these types to assemble per-component JSON schemas. Strip `children: Vec<ComponentNode>` and `fields: Vec<ComponentNode>` fields — children have moved to `Element.children: Vec<String>`.
- **D-04: Manual `JsonSchema` impls only where necessary.** Once `Vec<ComponentNode>` fields are gone, most Props structs derive `JsonSchema` cleanly. The remaining manual impls are for types that still carry unusual ser/de (keep the set minimal; log which ones need manual work in RESEARCH.md).
- **D-05: Schema version constant** `SCHEMA_VERSION = "ferro-json-ui/v2"`. Lives in `spec.rs` alongside `Spec`. The v1 `SCHEMA_VERSION` is removed.

### Spec + Element Shape

- **D-06: Spec struct fields (exact):**
  ```
  pub struct Spec {
      pub schema: String,             // "$schema" in JSON, = "ferro-json-ui/v2"
      pub root: String,               // ID of the root element
      pub elements: HashMap<String, Element>,
      pub title: Option<String>,
      pub layout: Option<String>,
      pub data: serde_json::Value,    // default null, skip_serializing_if is_null
  }
  ```
  Errors map (`errors: Option<HashMap<String, Vec<String>>>` in v1) is **not** on Spec v2. It flows through the rendering context in Phase 116+ rather than bleeding into the type.
- **D-07: Element struct fields (exact):**
  ```
  pub struct Element {
      #[serde(rename = "type")]
      pub type_name: String,
      #[serde(default)]
      pub props: serde_json::Value,
      #[serde(default, skip_serializing_if = "Vec::is_empty")]
      pub children: Vec<String>,
      #[serde(default, skip_serializing_if = "Option::is_none")]
      pub action: Option<Action>,
      #[serde(default, skip_serializing_if = "Option::is_none")]
      pub visible: Option<Visibility>,
  }
  ```
  `"type"` is the JSON key (keyword-collision-safe via rename). Action and Visibility keep their existing types unchanged from v1.

### Validation

- **D-08: Structural validation runs at parse time in `Spec::from_json(&str) -> Result<Spec, SpecError>`.** Fail-fast discipline: dangling child IDs, cycles, missing root, bad ID format, and depth overflow are all programming errors. They surface before any render attempt.
- **D-09: Four structural checks, in order:**
  1. `root` key exists in `elements`.
  2. Every ID appearing in any `Element.children` resolves to an existing `elements` entry (no dangling refs).
  3. No cycles in the element reference graph (DFS from root, reject back-edges).
  4. Nesting depth ≤ 3 from root (Screen > Section > Component), matching Airbnb/DoorDash/Lyft SDUI practice.
- **D-10: ID format regex:** `^[A-Za-z_][A-Za-z0-9_-]{0,127}$`. Rust-ident-ish + hyphens, bounded at 128 chars. Enforced during `from_json()` — every key in `elements` and every entry in any `children` must match.
- **D-11: `SpecError` enum** with concrete variants: `RootMissing(String)`, `DanglingChild { element: String, child: String }`, `Cycle { path: Vec<String> }`, `DepthExceeded { max: 3, found: usize, path: Vec<String> }`, `InvalidId(String)`, `Json(serde_json::Error)`. Error paths are structured, not formatted strings — callers can act on them.
- **D-12: Uniqueness of IDs is guaranteed by `HashMap` storage.** `serde_json::from_str` with `Map`-duplicate default behavior silently overwrites; Phase 115 detects this by preserving the raw `serde_json::Value` first and rejecting via a `DuplicateId(String)` variant before hydrating into the Spec.
- **D-13: No JSON Schema / semantic validation in Phase 115.** Catalog-driven semantic validation (unknown types, required props, enum constraints) is Phase 117. `from_json()` does not consult a Catalog; it only checks structure.

### Plugin Story

- **D-14: Plugin registry unchanged at runtime.** Existing `register_plugin(name, plugin)` and asset collection survive untouched. What disappears is the *type-level* Plugin escape hatch — because Elements are type-erased strings, a plugin named `"Map"` is just an Element with `type_name = "Map"`. No special variant.
- **D-15: Phase 117 will extend plugin registration** with optional `props_schema: schemars::Schema` so the Catalog can weave plugins into the full `oneOf`. Phase 115 deliberately keeps this out of scope but leaves `Element` open (type_name is an unrestricted string) so Phase 117 is unblocked.
- **D-16: Phase 115 does not validate plugin type names.** An Element with an unregistered `type_name` parses cleanly. Whether it renders is the renderer's and (later) Catalog's concern.

### Caller Migration (scope within Phase 115)

- **D-17: `framework/src/json_ui/mod.rs`:** `JsonUi::render(&Spec, &Value) -> Response` signature replaces `JsonUi::render(&JsonUiView, &Value)`. Body becomes a placeholder: serialize the Spec to pretty JSON inside an HTML shell with a clearly marked `<!-- v2 render pipeline arrives in Phase 116 -->` comment. Existing resolve/errors wiring is preserved against the Spec shape where possible; anything requiring the real walker is TODO'd against Phase 116 with a panic-free fallback. No feature flags.
- **D-18: Sample `app` crate** rewrites any JsonUiView construction to `Spec::builder()`. The sample page stays visually broken until Phase 116, but compiles and responds 200 with the placeholder HTML. This is acceptable — "clean break" is a project norm (see memory: `project_ferro_publication.md`).
- **D-19: `ferro-mcp` `code_templates` tool:** template strings ending in JsonUiView syntax switch to v2 flat-spec syntax. `json_ui_inspect` and `json_ui_generate` MCP tools update their parsing to Spec in Phase 120 — Phase 115 only needs them to keep compiling (adjust type signatures, mark TODOs).
- **D-20: `ferro_json_ui::projection::JsonUiRenderer` (the `Renderer` impl that turns `ServiceDef` + intents into output):** output type switches from `JsonUiView` to `Spec`. Internal mapping logic (field_map, relationship_map) stays naive — emit flat elements with whatever type_name the current code picks. Phase 117.1 rewrites the mapping to be schema-driven. Phase 115 just changes the *output shape*.
- **D-21: No migration shims.** No `JsonUiView::to_spec()`, no `v1_compat` module, no `#[cfg(feature = "v1")]`. Clean break per roadmap.

### Builder API

- **D-22: `Spec::builder()` fluent API** replaces `JsonUiView::new().title().layout().component()`. Sketch:
  ```
  Spec::builder()
      .title("Users")
      .layout("dashboard")
      .data(json!({"users": []}))
      .element("root", Element::new("Card").child("header").child("table"))
      .element("header", Element::new("Text").prop("content", "User list"))
      .element("table", Element::new("DataTable").prop("data_path", "/data/users"))
      .build()  // runs structural validation, returns Result<Spec, SpecError>
  ```
  Element IDs are explicit (not auto-generated) — mirrors React keys, enforces author intent.
- **D-23: `Element::new(type_name)` + `.prop(key, value)` + `.child(id)` + `.action(Action)` + `.visible(Visibility)` builders.** Props accumulate into an internal `serde_json::Map`; `build()` packs them into `props: Value`.
- **D-24: `Spec::builder().build()` runs the same structural validation as `from_json()`.** Both paths produce Specs that are guaranteed structurally sound. Tests verify both gates reject the same inputs.

### File Layout

- **D-25: New files:**
  - `ferro-json-ui/src/spec.rs` — `Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, `SpecError`, `SCHEMA_VERSION`, parse-time validation.
- **D-26: Rewritten files:**
  - `ferro-json-ui/src/component.rs` — delete `Component` enum, delete `ComponentNode`, delete custom Serialize/Deserialize. Keep only the Props structs (with `Vec<ComponentNode>` fields stripped to `Vec<String>` where they referenced children, or removed entirely if the field isn't structural). Keep enums (Size, ButtonVariant, etc.).
  - `ferro-json-ui/src/render.rs` — placeholder `render_spec_to_html(&Spec, &Value) -> String` that emits JSON inside an HTML shell. Real implementation lands in Phase 116.
  - `ferro-json-ui/src/projection/mod.rs` — `JsonUiRenderer::Output` becomes `Spec`. Internal mapping stays functionally identical; only the output struct changes.
  - `ferro-json-ui/src/lib.rs` — re-export `Spec`, `Element`, `SpecBuilder`, `SpecError`, `SCHEMA_VERSION`. Remove `JsonUiView`, `ComponentNode`, `Component` re-exports. Update doc examples.
  - `ferro-json-ui/src/plugin.rs` — unchanged in Phase 115 (registration API grows in Phase 117).
- **D-27: Deleted files:**
  - `ferro-json-ui/src/view.rs` — `JsonUiView` lives here; gone in v2.
- **D-28: Unchanged files:** `action.rs`, `visibility.rs`, `config.rs`, `data.rs`, `layout.rs`, `resolve.rs`, `plugins/`, `runtime/`.

### Testing

- **D-29: Round-trip test corpus.** A directory of JSON fixtures in `ferro-json-ui/tests/fixtures/` covering: simple single-element spec, three-level nested (max allowed depth), spec with actions, spec with visibility, spec with plugin-named element. Each fixture is parsed → serialized → reparsed and the two parses compared.
- **D-30: Rejection test corpus.** Fixtures that MUST fail `from_json` with specific `SpecError` variants: missing root, dangling child, simple cycle (A→B→A), self-cycle (A→A), 4-level nesting (exceeds max), invalid ID (contains a space, empty string, starts with a digit), duplicate ID in raw JSON map.
- **D-31: Builder parity test.** For each valid fixture, constructing the equivalent Spec via `Spec::builder()` produces serialization-equal output.
- **D-32: JsonSchema generation smoke tests.** For every surviving `*Props` struct, `schema_for!(TProps)` succeeds and produces a valid JSON object. Matches the existing pattern in `view.rs` tests we're about to delete.

### Documentation (out of scope for Phase 115 beyond inline rustdoc)

- Full JSON-UI user-facing docs rewrite lands in Phase 121.
- Phase 115 updates: rustdoc on `Spec`, `Element`, `SpecBuilder`, `SpecError`, top-of-file example in `lib.rs`. No user-docs (`docs/src/`) updates required this phase — they'd be churn ahead of Phase 116's rendering story.

### Claude's Discretion

- Internal module organization within `spec.rs` (single file vs. `spec/mod.rs` + `spec/validate.rs` + `spec/builder.rs` split) — pick what reads best.
- Whether `SpecError` uses `thiserror` (convention elsewhere in the workspace; likely yes).
- Whether the placeholder renderer emits raw pretty JSON, a tiny `<pre>`-wrapped debug dump, or a minimal walk that renders only Card and Text (enough to smoke-test the wiring) — pick whatever keeps `framework/tests` green with the least churn.
- Internal naming of depth check constants (`MAX_NESTING_DEPTH = 3` is fine).
- Whether to keep `PluginProps` as a convenience struct for downstream code or delete it. (The v1 Component::Plugin variant is gone regardless.)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### v12.0 milestone scope + success criteria
- `.planning/ROADMAP.md` §"v12.0 JSON-UI v2 — Spec-Driven Rendering (Planned)" — milestone goal, domain research summary, risks, stay/change list.
- `.planning/ROADMAP.md` §"Phase 115: Spec v2 Data Structures" — goal, depends-on, requirements, caveats, success criteria (7 checks).
- `.planning/ROADMAP.md` §"Phase 116: Flat Element Renderer" — informs what the Phase 115 placeholder renderer hands off to.

### Current ferro-json-ui crate (v1 — most of this is being rewritten)
- `ferro-json-ui/src/lib.rs` — public API, re-exports, `COMPONENT_CATALOG` const (Phase 117 replaces).
- `ferro-json-ui/src/view.rs` — `JsonUiView` + `SCHEMA_VERSION = "ferro-json-ui/v1"`. **Delete in Phase 115.**
- `ferro-json-ui/src/component.rs` — `Component` enum (40 variants), `ComponentNode`, custom Serialize/Deserialize at lines 972–1160, ~40 Props structs. **Component + ComponentNode deleted; Props structs survive with `Vec<ComponentNode>` → `Vec<String>` updates.**
- `ferro-json-ui/src/visibility.rs` — `Visibility` enum (And/Or/Not/Condition). **Unchanged.**
- `ferro-json-ui/src/action.rs` — `Action`, `ActionOutcome`, etc. **Unchanged.**
- `ferro-json-ui/src/plugin.rs` — `register_plugin`, `PluginRegistry`. **Unchanged in Phase 115; extended with schema registration in Phase 117.**
- `ferro-json-ui/src/render.rs` — current tree walker. **Replaced by placeholder in Phase 115; rewritten in Phase 116.**
- `ferro-json-ui/src/projection/mod.rs` — `JsonUiRenderer`, `RenderMode`, `VisualContext`. **Output type switches `JsonUiView` → `Spec` in Phase 115.**

### Callers to migrate
- `framework/src/json_ui/mod.rs` — `JsonUi::render`, `resolve_with_errors`, form handling paths.
- `framework/src/lib.rs` — re-exports of JsonUiView etc.
- `ferro-mcp/src/service.rs` — MCP service wiring.
- `ferro-mcp/src/tools/render_projection.rs` — uses JsonUiView output from the projection renderer.
- `ferro-mcp/src/tools/` — `json_ui_inspect`, `json_ui_generate`, `code_templates` touch v1 types and catalog strings. Phase 115 keeps them compiling; Phase 120 rewrites semantics.
- `app/` (sample app) — any hand-built JsonUiView construction gets a Spec::builder port.

### v11.5 prior context (renderer trait shape + ServiceDef bridge)
- `.planning/phases/133-generalize-renderer-trait/133-CONTEXT.md` — Renderer trait is modality-agnostic, renderers live in their output crate, ferro-projections owns only the trait.
- `.planning/phases/134-relocate-renderers-to-output-crates/134-CONTEXT.md` — `JsonUiRenderer` now lives in `ferro-json-ui/src/projection/`.
- `.planning/phases/135-servicedef-derivation-bridge/135-CONTEXT.md` — `ServiceDef::from_model()` exists; Phase 117.1 will consume v2 specs from ServiceDef.

### Domain research references (informing design choices)
- Vercel json-render (13k★ on GitHub, Jan 2026) — flat element map, Zod-defined catalog, AI-constrained generation. **Pattern adopted:** flat `root + elements` shape; type-erased `type_name`.
- JSON Forms (jsonforms.io) — JSON Schema for data + UI Schema for layout hints, framework-agnostic core. **Pattern informing:** `spec.data` stays distinct from `spec.elements`.
- react-jsonschema-form (rjsf) — schema-as-source-of-truth for validation; catastrophic perf with large oneOf. **Pattern informing:** compile-once validator in Phase 117; per-component schemas for AI structured output.
- Airbnb / DoorDash / Lyft SDUI — 3-tier hierarchy (Screen > Section > Component), GraphQL unions / protobuf for component typing, version fragmentation is the operational pain point. **Pattern adopted:** depth cap = 3; schema version string in every spec.

### Workspace conventions
- `CLAUDE.md` (project root) — architecture principles, Testing & Linting command (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`), ferro-mcp update requirement when framework changes.
- `.planning/codebase/CONVENTIONS.md` — crate conventions, builder patterns, error type patterns.
- `ferro-projections/CLAUDE.md` — crate boundary rules (no runtime logic in ServiceDef, no rendering deps). Phase 115 does not touch ferro-projections, but Phase 117.1 will.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (survive v2 unchanged or near-unchanged)
- `Visibility` / `VisibilityCondition` / `VisibilityOperator` — full expression semantics (And/Or/Not/Condition). Slots directly onto `Element.visible`.
- `Action` / `ActionOutcome` / `ConfirmDialog` / `HttpMethod` — slot directly onto `Element.action`.
- ~40 `*Props` structs — minus `children: Vec<ComponentNode>` and `fields: Vec<ComponentNode>` fields, these survive as Phase 117's Catalog input.
- `PluginRegistry`, `register_plugin`, asset collection — unchanged in Phase 115.
- `LayoutContext`, `DashboardLayout`, etc. — unchanged; Spec still carries `layout: Option<String>`.

### Patterns to Replicate
- **Builder pattern:** consuming `mut self` → `Self` (per CLAUDE.md user memory). `SpecBuilder` and `ElementBuilder` follow this.
- **`thiserror`-derived error enum per crate:** `SpecError` joins the existing error types (`ActionError` etc. if any).
- **`#[serde(rename = "$schema")]`** on Spec's schema field — preserves the existing JSON shape convention from v1.
- **Custom Serialize for complex enums with typed discriminators** — v2 Element does NOT need this; being type-erased removes the whole ~200-line custom Ser/De block.

### Integration Points
- `framework/src/json_ui/mod.rs::JsonUi` — framework entry point for rendering. Signature change from `&JsonUiView` → `&Spec` ripples out to every handler that renders JSON-UI.
- `framework/src/lib.rs` — re-exports. Add `Spec`, `Element`, `SpecBuilder`, `SpecError`, drop `JsonUiView`, `ComponentNode`, `Component`.
- `ferro-json-ui::projection::JsonUiRenderer` — output type switches to `Spec`. The `Renderer` trait's associated `Output` type follows suit.
- `ferro-mcp::tools::render_projection` — consumes projection output; swap to `Spec`.

### Non-obvious v1 behaviors to preserve in v2
- `#[serde(skip_serializing_if = "serde_json::Value::is_null")]` on `data` — keep; v2 `data` is still `serde_json::Value` and the empty-omit behavior is friendly to hand-authored JSON.
- `#[serde(default)]` fallbacks on most optional fields — keep; same argument.
- The v1 round-trip tests in `view.rs` (7 tests) and per-Props tests — port equivalents to `spec.rs` before deleting the originals.

### Non-obvious v1 behaviors to drop
- `errors: Option<HashMap<String, Vec<String>>>` on `JsonUiView` — gone in v2. Errors belong to the render context, not the Spec. (If Phase 116 needs them, they arrive as a side channel — not baked into Spec.)
- `Component::Plugin(PluginProps { plugin_type, props })` fallback for unknown types — replaced by "type_name is just a string, catalog decides if it's known".

</code_context>

<specifics>
## Specific Ideas

- **Design lineage.** The Spec shape is deliberately close to Vercel json-render (flat map + root pointer) because that library validated the "AI → JSON → UI" thesis at 13k★ and ferro is making the same bet with server-authoritative semantics. Where json-render takes Zod as its schema source, ferro takes `schemars::JsonSchema` — same idea, Rust-native.
- **Depth cap is sacred.** 3 levels is the empirical sweet spot from Airbnb/DoorDash/Lyft SDUI retrospectives — enough for Screen > Section > Component, not enough for schemas to become programming languages. Phase 118's expression system is the other half of this guardrail.
- **Type-erasure as symmetry.** v1's `Component::Plugin` was a wart — a "some types are special" backdoor. Making Element's type_name a plain string makes built-ins and plugins indistinguishable at the Spec level, which is the same stance Vercel json-render takes.
- **Placeholder renderer is load-bearing.** Every commit must land green (per user memory: commits + CLAUDE.md). The Phase 115 placeholder is the mechanism that buys Phase 116 a full sprint without a red workspace in between. The placeholder's implementation should be boring — a `<pre>` dump of the spec JSON inside a minimal HTML skeleton is fine.
- **Error enum variants are named for what the author did wrong, not what the parser saw.** `DanglingChild`, `Cycle`, `DepthExceeded` — each one should be actionable by a human reading a stack trace. Paths are structured `Vec<String>` so tooling (future IDE plugin, Phase 120's MCP inspect) can highlight the offending element by ID.

</specifics>

<deferred>
## Deferred Ideas

- **Catalog / JSON Schema assembly** — Phase 117.
- **Plugin schema registration API** — Phase 117.
- **`$data` / `$template` expression evaluation** — Phase 118.
- **Spec loader with hot reload** — Phase 119.
- **MCP `json_ui_generate` two-tier AI strategy** — Phase 120.
- **gestiscilo field test conversion** — Phase 121.
- **IDE plugin that consumes exported JSON Schema** — future backlog.
- **Cross-spec composition / include directives** — explicitly out of scope for v12.0 (would be an "inner platform effect" risk per domain research).
- **Client-side interactivity (beyond the existing IIFE runtime)** — PROJECT-level deferred; revisit post-v12.0 only if validation surfaces a concrete gap.

</deferred>

---

*Phase: 115-spec-v2-data-structures*
*Context gathered: 2026-04-18*
*Mode: --auto*
