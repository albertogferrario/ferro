# Phase 118: Server-Side Expressions - Context

**Gathered:** 2026-04-19
**Status:** Ready for planning
**Mode:** `--auto` — decisions auto-selected to keep `$data` / `$template` resolution minimal, reuse the existing `data::resolve_path` primitive, and preserve the Phase 116 walker's catalog-unaware shape. Downstream agents may override any auto-choice by editing this file before `/gsd-plan-phase`.

<domain>
## Phase Boundary

Add a server-side pre-render pass that walks `Element.props` and substitutes two expression objects:

- `{"$data": "/path"}` → resolves to whatever JSON value sits at `spec.data` + `/path` (slash-separated, same convention as `data_path`). Substitution preserves the original JSON type.
- `{"$template": "literal {/path/in/data} more text"}` → produces a `String` with each `{slash-path}` placeholder replaced by `resolve_path_string(&spec.data, ...)`. Missing placeholders interpolate as empty string.

Concretely Phase 118 ships:

- `ferro-json-ui/src/expression.rs` (new) — `pub fn resolve_expressions(spec: &mut Spec)`, the recursive `resolve_value` walker, and inline tests.
- `ferro-json-ui/src/lib.rs` re-export of `resolve_expressions` alongside `resolve_actions` / `resolve_errors`.
- `framework/src/json_ui/mod.rs::JsonUi::resolve` calls `resolve_expressions(&mut resolved)` immediately after `resolve_actions` and before any render path. `JsonUi::render`, `JsonUi::render_with_config`, `JsonUi::render_json`, and the `_with_errors` variants inherit the new step transparently.
- Integration tests in `framework/src/json_ui/mod.rs` covering both expression types end-to-end through `JsonUi::render` and `JsonUi::render_json`.
- Per-request cycle: `Spec::from_json` (Phase 115 structural validation) → `resolve_actions` → `resolve_expressions` → `Catalog::validate` (when callers opt in; Phase 119 will hard-wire it) → `render_spec_to_html_with_plugins`.

**Hard cap (criterion 6, locked — do not re-open):** only `$data` and `$template`. No `$if`, `$for`, `$state`, `$bind`, `$ref`, `$concat`, `$let`. Inner-platform-effect risk is the named v12.0 risk and the cap is the architectural answer to it.

**What this phase does NOT do** (locked by ROADMAP / earlier phases — do not re-open):
- Change `Spec` / `Element` shape (Phase 115).
- Change the renderer walker, dispatch match, or per-component HTML emission (Phase 116).
- Modify `Catalog` shape, `Catalog::validate` body, or `global_catalog()` API (Phase 117).
- Touch `Spec::from_service_def` or projection helpers (Phase 117.1) — projector continues to emit `data_path` strings; expression resolution is orthogonal.
- Add a page loader or hot-reload (Phase 119 wires the load-time order).
- Update CLI / MCP generation tools to emit expressions (Phase 120 — generators may start using expressions later, but not in 118).
- Convert any gestiscilo page (Phase 121).
- Resolve expressions on the client. v12.0 is server-authoritative; expressions resolve before HTML leaves the server.
- Resolve expressions inside `Spec.data`, `Spec.title`, `Spec.layout`, `Element.children`, `Element.action`, or `Element.visible`. Expression substitution is restricted to `Element.props` recursive scan (D-04 below). Visibility conditions already have their own evaluator and stay untouched.
- Introduce a parallel path-syntax dialect. Slash-separated paths are the single mental model.

</domain>

<decisions>
## Implementation Decisions

### D-01: Single-pass pre-render resolver, mutating-clone pattern
**Decision:** Add `pub fn resolve_expressions(spec: &mut Spec)` in a new `ferro-json-ui/src/expression.rs` module. Callers clone the spec first (the same clone `JsonUi::resolve` already takes for `resolve_actions`), then call the resolver in place. The walker (Phase 116) stays expression-unaware and continues to receive concrete typed-by-Catalog props.

**Why:** Phase 116 D-29 explicitly states the renderer is catalog-unaware; making it expression-unaware preserves the same separation. A pre-render pass is testable in isolation, leaves `render_spec_to_html` unchanged, and matches the architectural shape Phase 119's loader will lean on (parse → resolve → validate → render).

**How to apply:** New file. Public function signature mirrors `resolve_actions`. Single source of truth for the walk; no second copy in the renderer.

### D-02: Path syntax is slash-separated for both `$data` and `$template`
**Decision:** Reuse `crate::data::resolve_path` (`/segment/segment`) verbatim for both expression types. The roadmap example `{"$data": "path/to/value"}` is interpreted as the same slash convention with the leading slash optional (existing `resolve_path` already accepts both). The roadmap example `{"$template": "Hello, {user.name}!"}` becomes `{"$template": "Hello, {/user/name}!"}` — placeholders carry slash paths, never dot paths.

**Why:** Every `data_path` field across the v2 surface (Input, Select, Checkbox, Switch, Table, DataTable, the projector's `/data/{name}` convention) already uses slash-separated paths. Introducing a dot-path dialect inside `$template` only fragments the mental model and forces a parallel parser. PROJECT.md "small mental model" criterion makes this the right call. The roadmap text is illustrative; the binding contract is success criteria 1–6 ("data paths") which are syntax-agnostic.

**How to apply:** `data::resolve_path` becomes `pub(crate)` if it isn't already, or stays as-is and `expression.rs` lives in the same crate. The `$template` placeholder regex is `\{[^{}\\]*\}` — match a brace-delimited path with no nested braces or backslashes inside. Whitespace inside placeholders is trimmed before resolution (`{ /user/name }` ≡ `{/user/name}`). Literal braces in template text use `\{` and `\}` escapes; literal backslash is `\\`.

### D-03: Type-preserving `$data`, missing → `Value::Null`
**Decision:** `{"$data": "/path"}` resolves to `data::resolve_path(&spec.data, "/path").cloned().unwrap_or(Value::Null)`. The substituted JSON value preserves the original type — number, bool, string, object, array, or null. There is no auto-stringification.

**Why:** Success criterion 3 says expressions work in all positions (string, number, boolean values). Forcing `$data` to always return a string would break `ProgressProps.value: f64`, `CheckboxProps.checked: bool`, `DataTableProps.columns` etc. Type preservation lets `Catalog::validate` (which runs after resolution per D-08) check the resolved spec against the typed `*Props` schemas without false positives.

**How to apply:** When the resolver encounters a JSON object with exactly one key `"$data"` whose value is a JSON string, replace the whole object with the resolved `Value` (or `Value::Null` if path misses). Objects with `"$data"` plus additional keys are treated as literal data — see D-06.

### D-04: Resolution scope is `Element.props` recursive only
**Decision:** The walker descends into every `Element.props` value in `spec.elements.values_mut()` and recurses through `serde_json::Value::Object` and `serde_json::Value::Array` nodes. It does NOT touch `Spec.data` (the resolver source must stay literal), `Spec.title`, `Spec.layout`, `Element.type_name`, `Element.children`, `Element.action`, or `Element.visible`. Visibility expressions already have their own typed `Visibility` evaluator and stay outside the `$data`/`$template` surface.

**Why:** `Spec.data` is the substitution source — resolving expressions inside it produces ambiguity (recursive expansion, ordering hazards). Metadata fields (`title`, `layout`) are server-side configuration, not user-visible content with dynamic substitution semantics. `Element.children` are IDs validated at parse time and must stay literal IDs (Phase 115 D-09 cycle/dangling check would lose its guarantee). `Element.action.url` is already populated by `resolve_actions`. `Element.visible` has its own typed expression language scoped to the `Visibility` enum.

**How to apply:** The walker takes `&mut Value` and mutates in place. It treats every `Object` of the shape `{"$data": <string>}` or `{"$template": <string>}` as an expression node and replaces it. Otherwise it recurses into children (`Object` values, `Array` items). Strings, numbers, booleans, and nulls are leaves.

### D-05: `$template` placeholder behavior
**Decision:** A `{"$template": "<literal>"}` object replaces itself with a `Value::String` produced by scanning `<literal>` left-to-right and substituting every `{slash-path}` placeholder via `data::resolve_path_string(&spec.data, path)`. Missing placeholders interpolate as empty string `""`. Non-string/`null` resolutions use the existing `resolve_path_string` rules (numbers and bools stringify, objects/arrays JSON-serialize, null → empty).

**Why:** Mirrors HTML/UI norms — missing data shouldn't show debug brackets to end users. Reusing `resolve_path_string` keeps stringification policy in one place and matches what `data_path` form pre-fill already does. The roadmap example `"Hello, {user.name}!"` resolves consistently with what authors expect from a template literal.

**How to apply:** Inline regex (or hand-rolled scanner) inside `expression.rs`. Escaping: `\{` → literal `{`, `\}` → literal `}`, `\\` → literal `\`. Any other backslash sequence is preserved verbatim. A `$template` whose value is not a string is treated as a malformed expression per D-06.

### D-06: Malformed expressions degrade to literal JSON, never panic, never log
**Decision:** Three malformed shapes are silently passed through as literal JSON values:
1. `{"$data": <non-string>}` — e.g., `{"$data": 42}` or `{"$data": null}`.
2. `{"$template": <non-string>}` — same idea.
3. `{"$data": "/x", "extra_key": "y"}` or `{"$template": "...", "x": 1}` — expression markers with sibling keys.

For all three, the resolver leaves the JSON object untouched and continues walking. Catalog validation (Phase 117) is the layer that surfaces shape errors to the author, not the resolver.

**Why:** Phase 116 D-09 set the precedent: render-time helpers are infallible. The expression resolver inherits the same posture. Author errors flow through `Catalog::validate` as `PropsInvalid` errors with structured paths; duplicating that diagnostic surface inside the resolver buys nothing and risks contradictory error reports.

**How to apply:** `is_data_expr(obj)` returns true iff the object has exactly one key `"$data"` of string type. Same for `is_template_expr`. Anything else recurses normally.

### D-07: Single-pass resolution, no recursive expansion
**Decision:** `$data` resolves to the literal JSON value at the path. If that value itself contains `$data` or `$template` markers, they are NOT re-resolved. The resolver makes exactly one pass over the spec.

**Why:** Recursive resolution opens a billion-laughs / expansion-attack surface and adds inner-platform-effect (an author who plants `$data` in `spec.data` is one step away from data-driven templating). `spec.data` should hold ground-truth values; expressions live in `props`. Single-pass keeps the cost trivially bounded by spec size.

**How to apply:** When `$data` returns a `Value::Object` containing further expression markers, those markers stay as literal data — they are NOT walked again. Document this in the module rustdoc.

### D-08: Pipeline ordering — resolve before validate
**Decision:** The canonical request lifecycle is:
1. `Spec::from_json` (or `Spec::builder().build()`) — Phase 115 structural validation.
2. `resolve_actions(&mut spec, resolver)` — handler → URL.
3. `resolve_expressions(&mut spec)` — `$data` / `$template` substitution (this phase).
4. `Catalog::validate(&spec)` — schema validation against the now-concrete props (callers opt in today; Phase 119 hard-wires it at load time).
5. `render_spec_to_html_with_plugins(&spec, &data)` — Phase 116 walker emits HTML.

`JsonUi::resolve` in `framework/src/json_ui/mod.rs` is updated to perform steps 2 and 3. Step 4 is left to the caller in Phase 118 because today's `JsonUi::render` does not yet validate.

**Why:** Validation must see the resolved props or it would reject every spec that uses an expression in a typed slot. Resolving before validating is the only ordering that lets the typed `*Props` schemas remain authoritative. Phase 119's page loader will codify this order at the file-load entry point.

**How to apply:** Update `JsonUi::resolve` (in `framework/src/json_ui/mod.rs`) to call `resolve_expressions(&mut resolved)` immediately after `resolve_actions(...)`. No other framework changes required for Phase 118.

### D-09: Resolver is infallible; no `Result`, no diagnostic emission
**Decision:** `resolve_expressions(&mut Spec)` returns `()`. There is no `Result`, no error variant, no log, no HTML comment. Missing paths null/empty per D-03/D-05; malformed expressions degrade to literal JSON per D-06.

**Why:** Pure-function symmetry with `resolve_actions` (which is also infallible). Diagnostics belong to the validator (Phase 117) and the renderer (Phase 116 D-10 HTML comments). The resolver's only job is faithful substitution.

**How to apply:** Function signature `pub fn resolve_expressions(spec: &mut Spec)`. Internal helpers may return Option/bool but the public surface is action-only.

### D-10: Performance — always walk, no fast-path detection
**Decision:** `resolve_expressions` always walks every `Element.props` even when the spec contains no expressions. No "expressions present?" pre-scan, no `Cow<Spec>`-style optimization.

**Why:** The walk is O(total JSON nodes in props), which for a typical Phase 116 page (tens of elements, modest props) is sub-millisecond. Pre-scan adds a second walk for the same cost. `Cow` adds a second clone path that complicates the public API. Phase 116 D-28 already deferred render-cache to post-v1.0 for the same cost/benefit reason — expression resolution follows.

**How to apply:** The walker descends unconditionally. If profiling later shows expression resolution dominates render time on real gestiscilo pages, revisit (Phase 121's field test is the natural opportunity to measure).

### D-11: Module layout
**Decision:** Phase 118 introduces exactly one new file:

- `ferro-json-ui/src/expression.rs` — `resolve_expressions(&mut Spec)`, `resolve_value(&mut Value, &Value)`, `resolve_data_expr(&Value, &Value) -> Option<Value>`, `resolve_template_expr(&str, &Value) -> String`, `is_data_expr(&Map) -> Option<&str>`, `is_template_expr(&Map) -> Option<&str>`, `EXPR_DATA_KEY = "$data"`, `EXPR_TEMPLATE_KEY = "$template"`, inline tests.

Existing files modified:

- `ferro-json-ui/src/lib.rs` — `pub mod expression;` declaration; `pub use expression::resolve_expressions;` re-export grouped with the existing `resolve::*` re-exports.
- `ferro-json-ui/src/data.rs` — visibility bump if `resolve_path` / `resolve_path_string` need to be `pub(crate)` rather than the current `pub(crate)` (likely no change — they're already `pub(crate)` and `expression.rs` lives in the same crate).
- `framework/src/json_ui/mod.rs` — `JsonUi::resolve` calls `resolve_expressions(&mut resolved)` after `resolve_actions(...)`; new integration tests for the round-trip.

No other files touch.

**Why:** Single-file Phase 118 keeps the change auditable and easy to revert if the design needs to evolve. Mirrors the file-per-feature convention already used for `action.rs`, `visibility.rs`, `resolve.rs`.

**How to apply:** Planner picks final inline organization; functions can be flat or grouped under a `mod data_expr` / `mod template_expr` split if the file approaches ~600 LOC.

### D-12: Testing surface
**Decision:** Phase 118 ships:

- Unit tests in `expression.rs` covering: `$data` with simple/nested/array paths; `$data` with missing path → null; `$data` preserving number, bool, object, array types; `$data` with non-string value → literal passthrough; `$data` with sibling keys → literal passthrough; `$template` with single placeholder, multiple placeholders, no placeholder; `$template` with missing placeholder → empty; `$template` with escaped braces; `$template` whose value is non-string → literal; nested expressions inside arrays and inside object values inside `Element.props`; expressions adjacent to `Spec.data` (resolver does not touch the source); single-pass guarantee (a `$data` whose target value contains another `$data` marker is NOT re-resolved); `Element.children`, `Element.action`, `Element.visible` are NOT walked.
- Integration tests in `framework/src/json_ui/mod.rs` covering: `JsonUi::render` resolves `$data` before HTML emission (assert resolved value appears in markup); `JsonUi::render_json` returns the resolved spec (no expression markers in output JSON); `JsonUi::render_with_config` honors expression resolution; `JsonUi::render_with_errors` resolves expressions and applies errors against the resolved props (order: actions → expressions → errors).

**Why:** Inline tests prove the resolver in isolation; integration tests prove the pipeline ordering documented in D-08 actually holds. Together they make D-08 a structural guarantee, not a documentation hope.

**How to apply:** Standard `#[cfg(test)] mod tests { … }` blocks. No new test fixtures required; tests construct Spec via `Spec::builder()` inline.

### D-13: Catalog schema is unchanged in Phase 118
**Decision:** Phase 117's catalog schema treats every prop as its typed shape (e.g. `CardProps.title: String`). It does NOT add an `oneOf: [String, ExpressionObject]` for every slot. Authors who use `$data` in a typed slot rely on D-08 ordering (resolution happens before validation).

**Why:** Adding expression-object branches to every prop schema would balloon the catalog schema (already a Phase 117 caveat — full catalog is 40-80 KB) and force AI generation tools to emit the union. Keeping the schema typed and ordering resolution upstream of validation is the cleaner separation.

**How to apply:** No code changes in `ferro-json-ui/src/catalog.rs`. Documentation note in `expression.rs` rustdoc that callers running validation must call `resolve_expressions` first. Phase 119's loader will encode this; today (Phase 118) the framework wiring does the same in `JsonUi::resolve`.

### D-14: Plugin props pass through unchanged
**Decision:** Plugin components (those whose `type_name` is not in `BUILTIN_TYPES`) have untyped `serde_json::Value` props. The resolver treats them identically to built-in props — recurses into the props value and substitutes any `$data`/`$template` markers it finds.

**Why:** Plugins benefit from `$data` for the same reasons built-ins do: their authors should not hand-resolve `data_path`-style references. The resolver is type-agnostic, so this falls out for free.

**How to apply:** No special-case branch needed. `resolve_value` on `el.props` works whether the props came from a typed `*Props` serialization or a plugin.

### Claude's Discretion
- Whether `is_data_expr` / `is_template_expr` are free functions, methods on a private `Expr` enum, or pattern-matched inline — pick whichever reads cleanest after the implementation lands.
- Whether the template parser is a hand-rolled scanner or uses a tiny `regex` / `winnow` dependency — prefer hand-rolled (zero new deps; the grammar is trivial).
- Whether `expression.rs` exports any helper types beyond `resolve_expressions` (e.g., a `pub fn substitute_template(template: &str, data: &Value) -> String` for downstream re-use) — only export what `framework`/tests actually consume; do not pre-export speculative helpers.
- Whether nested `$data` markers inside `$template` resolution paths are special-cased (they are not — `$template` interpolates `data_path` strings, not nested expression objects). The hard cap (D-07) makes this a non-question.
- Whether the integration tests live alongside the existing `framework/src/json_ui/mod.rs` test block or in a sibling `#[cfg(test)]` file — match whichever is consistent with the existing layout.
- Whether `data::resolve_path` and `data::resolve_path_string` get renamed or kept (`pub(crate)`) — keep as-is; no API surface change is in scope for this phase.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and success criteria
- `.planning/ROADMAP.md` §"Phase 118: Server-Side Expressions" — goal, depends-on (Phase 116), Requirements (EXPR-01, EXPR-02, EXPR-03), 2 caveats (inner-platform effect risk, binding-expression alternative), 6 success criteria.
- `.planning/ROADMAP.md` §"v12.0 JSON-UI v2 — Spec-Driven Rendering" milestone preamble — overall context, key risks, what stays vs. what changes, hard cap rationale.
- `.planning/PROJECT.md` "Out of Scope" — `$state`, `$bindState`, JS-powered interactivity (server-authoritative model is correct); "Expression language beyond `$data` and `$template` — Inner platform effect is the #1 strategic risk in SDUI."
- `.planning/PROJECT.md` "Key Decisions" — `Server-side expressions only` (Planned); `Hard cap on expression language` (Planned); `Max nesting depth: 3 levels` (Planned).

### Locked upstream decisions (do not re-open)
- `.planning/phases/115-spec-v2-data-structures/115-CONTEXT.md` — Spec/Element shape, type-erasure model, parse-time validation, `SCHEMA_VERSION = "ferro-json-ui/v2"`, builder pattern. Particularly D-09 (depth ≤ 3), D-21 (no migration shims).
- `.planning/phases/116-flat-element-renderer/116-CONTEXT.md` — `BUILTIN_TYPES` list, walker shape, infallible-renderer posture (D-09), HTML-comment diagnostics (D-10), and the explicit out-of-scope reminder D-29 (`$data`/`$template` not evaluated in Phase 116; expression resolver runs as a pre-render pass).
- `.planning/phases/117-catalog-and-json-schema/117-CONTEXT.md` — `Catalog::validate` shape, `CatalogError` variants, the architectural note that the walker stays catalog-unaware (D-33). Phase 118's resolver follows the same separation.
- `.planning/phases/117.1-schema-driven-projections/117.1-CONTEXT.md` — projector emits `data_path` strings (D-15) and explicitly defers `$data` to Phase 118; the projector must NOT generate expression markers in this milestone.

### Downstream constraints (read to avoid painting into a corner)
- `.planning/ROADMAP.md` §"Phase 119: Page Loader" — loader will encode the parse → resolve → validate → render order; Phase 118 must expose the resolver in the shape the loader can call (D-08).
- `.planning/ROADMAP.md` §"Phase 120: CLI & MCP Updates" — AI generation may begin emitting expressions later; the resolver must accept whatever shapes the catalog schema allows.
- `.planning/ROADMAP.md` §"Phase 121: Documentation & Field Test" — gestiscilo conversion will exercise expressions on real pages; performance budget validated there.

### ferro-json-ui source (what Phase 118 adds and what it consumes)
- `ferro-json-ui/src/spec.rs` — `Spec`, `Element`, builder. **Consumed read-only; no shape changes.**
- `ferro-json-ui/src/data.rs` — `resolve_path` (line 19), `resolve_path_string` (line 55). **Consumed verbatim by the new resolver. Both functions stay `pub(crate)`; `expression.rs` lives in the same crate.**
- `ferro-json-ui/src/resolve.rs` — `resolve_actions` (line 35), `resolve_actions_strict`, `resolve_errors`, `resolve_errors_all`. **Pattern reference for the new `resolve_expressions` signature; not modified.**
- `ferro-json-ui/src/render/mod.rs` — Phase 116 walker. **Unchanged. Resolver runs upstream.**
- `ferro-json-ui/src/catalog.rs` — `Catalog::validate`. **Unchanged. Validation runs after resolution; Phase 118 documents the ordering.**
- `ferro-json-ui/src/visibility.rs` — `Visibility::evaluate`. **Unchanged. Visibility expressions stay outside the `$data`/`$template` surface.**
- `ferro-json-ui/src/component.rs` — typed `*Props` structs with their `data_path` fields (lines 159, 247, 279, 338, 359, 743). **Read-only reference; no struct changes. The `data_path` convention coexists with `$data` — they are complementary, not competing.**
- `ferro-json-ui/src/lib.rs` — re-exports. **Add `pub mod expression;` and `pub use expression::resolve_expressions;`.**
- `ferro-json-ui/Cargo.toml` — **No new dependencies.** Hand-rolled template scanner, no `regex` crate.

### Framework integration
- `framework/src/json_ui/mod.rs::JsonUi::resolve` (lines 39-43) — clones spec and calls `resolve_actions`. **Phase 118 adds `resolve_expressions(&mut resolved)` immediately after.**
- `framework/src/json_ui/mod.rs::JsonUi::render`, `render_with_config`, `render_json`, `render_with_errors`, `render_with_errors_config`, `render_validation_error`, `render_json_with_errors` — all converge on `JsonUi::resolve`, so they inherit expression resolution transparently.
- `framework/src/json_ui/mod.rs` test block — Phase 118 adds integration tests asserting end-to-end expression resolution through every public render path.

### Workspace conventions
- `CLAUDE.md` (project root) — testing gate (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`), no co-author lines in commits, builder pattern (consuming `mut self → Self`), `thiserror` per crate, ferro-mcp update requirement when framework behavior changes (Phase 118 doesn't change MCP surface; no MCP edit required).
- `.planning/codebase/CONVENTIONS.md` — crate conventions, error patterns, file layout norms.

### Domain research references (informing the hard cap)
- Airbnb / DoorDash / Lyft SDUI retrospectives — every production SDUI system warns about schemas evolving into programming languages. `$data` + `$template` is the deliberately-narrow answer; resisting `$if`/`$for`/`$state`/`$bind` is the load-bearing v12.0 architectural choice.
- Appsmith / ToolJet / Retool binding expression model (`{{query.data}}`) — more flexible at runtime, harder to validate at compile time, easy to leak into client-side state. Ferro deliberately picks the simpler, server-authoritative path.
- JSON Pointer (RFC 6901) — the slash-segmented path syntax `data::resolve_path` already implements is JSON-Pointer-shaped without the `~0`/`~1` escapes; Phase 118 inherits this without further commitment to RFC 6901 conformance (a future phase can tighten if a real use case appears).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (Phase 118 consumes as-is)
- `ferro_json_ui::data::resolve_path(&Value, &str) -> Option<&Value>` — slash-separated path resolver. Used directly by `resolve_data_expr` to look up `{"$data": "/path"}` targets. Already handles leading-slash optionality, empty path → root, array-index segments, and missing-key returns `None`.
- `ferro_json_ui::data::resolve_path_string(&Value, &str) -> Option<String>` — the same with stringification rules (numbers/bools `to_string`, objects/arrays `serde_json::to_string`, null → None). Used directly by `resolve_template_expr` for placeholder substitution.
- `ferro_json_ui::resolve::resolve_actions(&mut Spec, impl Fn(&str) -> Option<String>)` — the signature template Phase 118's `resolve_expressions(&mut Spec)` mirrors.
- `ferro_json_ui::Spec::elements: HashMap<String, Element>` — direct iteration target for the resolver.
- `serde_json::Value::as_object_mut()`, `as_array_mut()`, `as_str()` — standard mutation helpers; no new dep.

### Patterns to Replicate
- **Infallible pre-render helper** (`resolve_actions`, `resolve_errors`) — Phase 118's `resolve_expressions` returns `()`. Errors flow through Catalog/render diagnostics, not the resolver.
- **Single source of truth in `pub(crate)` helpers** — `data::resolve_path`/`resolve_path_string` are already the one-true path resolver; the expression module re-uses them rather than re-implementing the slash walk.
- **Inline `#[cfg(test)] mod tests`** — every `resolve.rs` / `data.rs` / `visibility.rs` ships its own tests in the same file. Phase 118 follows.

### Integration Points
- `framework::JsonUi::resolve` — single entry point that every public `JsonUi::render*` method routes through. Adding one line here covers the entire render surface.
- `Spec::from_json` callers (Phase 119 will exercise this; Phase 118 has none directly) — the resolver expects a structurally-valid Spec. It does not re-validate the spec.
- `Catalog::validate` — runs against the resolved props per D-08. Phase 118 does NOT alter `Catalog`'s body; it only documents the ordering.
- `JsonUiRenderer::render` (the projection bridge) — emits `data_path` strings, not expressions. The resolver is a no-op on projector output today, which is the desired baseline for v12.0.

### Non-obvious behaviors to preserve
- `Element.action` is `resolve_actions`-managed and stays untouched by `resolve_expressions`.
- `Element.visible` uses the typed `Visibility` enum and is NOT a `$data`/`$template` substitution surface.
- `Element.children` are element IDs validated by Phase 115 — leaving them literal is non-negotiable (cycle/dangling guarantees rest on this).
- Plugin props are `serde_json::Value` and benefit from expression substitution exactly like built-in props (D-14).
- The order of clone → `resolve_actions` → `resolve_expressions` in `JsonUi::resolve` matches the order Phase 119's loader will hard-wire.

### Non-obvious behaviors to drop
None — Phase 118 is purely additive. No deletions, no signature changes outside `JsonUi::resolve`'s body.

</code_context>

<specifics>
## Specific Ideas

- **Slash paths everywhere is the conceptual win.** The roadmap example (`{user.name}` with dot syntax inside `$template`) was illustrative shorthand. Real-world authoring sees `data_path: "/users/0/name"` in five different prop slots; making `$template` honor a different convention would be a needle-stick that confuses every author and every AI. Single mental model > literal roadmap example.
- **Resolver-before-validator is the load-bearing ordering.** It is the only sequence that lets `Catalog`'s typed props schema stay narrow. If validation ran first, every `String` prop would need `oneOf: [String, ExpressionObject]` and the schema bloats to 80+ KB — exactly the size Phase 117's caveats warned against. Phase 118 fixes the order; Phase 119 enshrines it at the load entry point.
- **Single-pass with no recursion is the inner-platform-effect firewall.** Allowing `$data` to return values that themselves get re-resolved is half a step from `$let` and a full step from `$concat`. The hard cap (D-07) is what keeps `$data` from drifting into a binding language.
- **Type preservation matters more than convenience.** It would be tempting to stringify everything `$data` returns ("expressions are content") — but `ProgressProps.value: f64` would break, `CheckboxProps.checked: bool` would break, the projector's `Vec<Column>` patterns would break. Preserving JSON type is the cheap right answer.
- **`Element.props` is the only substitution surface.** Spec metadata (`title`, `layout`), structural fields (`children`, IDs), and the action/visibility surfaces all have their own resolution stories. Restricting `$data`/`$template` to props keeps the resolver's scope obvious and keeps every other field's invariants intact.
- **Hand-rolled template scanner over a regex dep.** The grammar is tiny (`{path}` with `\{`/`\}` escapes); pulling in `regex` for ten lines of code is the wrong dependency cost. Future phases that need richer template syntax (they shouldn't) can reach for a parser then.
- **Catalog stays schema-blind to expressions.** Phase 117 D-33 already says the walker is catalog-unaware; Phase 118 takes the symmetric stance for the validator. Resolution is the bridge that lets both ends of the pipeline ignore expressions.
- **The killer feature is what Phase 118 does NOT add.** The hard cap is the architectural deliverable. Every SDUI retrospective in the domain research warns about this exact slippery slope; honoring the cap is what makes Ferro's spec-driven rendering claim defensible against the "but Retool has bindings" objection.

</specifics>

<deferred>
## Deferred Ideas

- **`$if` / `$for` / `$switch` conditionals and loops** — explicitly out of scope per criterion 6 and PROJECT.md "Out of Scope" §"Expression language beyond `$data` and `$template`". Inner-platform-effect risk is named.
- **`$state` / `$bindState` client-side state hooks** — out of scope per PROJECT.md "Out of Scope" §"Client-side state management".
- **`$ref` cross-spec references** — would invite cycle / expansion attack surface; out of scope per Phase 115 "Deferred Ideas" §"Cross-spec composition / include directives".
- **Expression markers inside `Spec.data`** — D-04 explicitly excludes; would require recursion semantics (D-07 banned).
- **Expression markers inside `Element.children`** — D-04 explicitly excludes; would break Phase 115 cycle/dangling guarantees.
- **JSON Pointer `~0`/`~1` escape compliance** — `data::resolve_path` does not escape; Phase 118 inherits the slash-only convention. If a future use case needs literal slashes inside keys, revisit then.
- **Recursive (multi-pass) expression resolution** — D-07 banned; revisit only if a concrete use case appears that doesn't decompose into a single-pass equivalent.
- **Per-element error reporting from the resolver** — D-09 banned; render-time HTML comments and Catalog validation errors are the diagnostic surface.
- **Path-cache for hot props that resolve the same `$data` repeatedly** — D-10 deferred; Phase 121 field test is the natural moment to measure.
- **Cow-based zero-clone fast path when no expressions exist** — D-10 deferred for the same reason.
- **Schema-level expression markers** (`oneOf: [String, ExpressionObject]` per typed slot) — D-13 banned; would balloon the catalog schema.
- **`$template` placeholders that read non-`spec.data` sources** (e.g., locale, request, env) — out of scope; `spec.data` is the substitution source and stays the single truth.
- **AI generation tools emitting expressions** — Phase 120 may start using expressions in generated specs, but Phase 118 ships the resolver only. The roadmap explicitly defers generator updates.
- **gestiscilo migration to expressions** — Phase 121.
- **MCP introspection tool for expressions** (e.g., `mcp__ferro__inspect_expression`) — not in scope; the resolver is internal pipeline plumbing, not an authoring surface that needs an MCP tool.
- **Reviewed Todos** — none. The `gsd-tools todo match-phase 118` query returned `todo_count: 0`, so there are no outstanding ideas to fold or defer at this phase.

</deferred>

---

*Phase: 118-server-side-expressions*
*Context gathered: 2026-04-19*
*Mode: --auto*
