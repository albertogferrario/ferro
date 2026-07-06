# Phase 118: Server-Side Expressions - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-19
**Phase:** 118-server-side-expressions
**Mode:** `--auto` — every gray area auto-resolved with the recommended option. No interactive questions asked.
**Areas discussed:** Pipeline placement, Path syntax, Resolution shape, Substitution scope, Template placeholder syntax, $data type semantics, Missing path behavior, Recursion / single-pass, Validation order, Performance posture, Module layout, Error surface, Catalog schema interaction, Plugin handling

---

## Pipeline placement

| Option | Description | Selected |
|--------|-------------|----------|
| Pre-render pass invoked from `JsonUi::resolve` | New `resolve_expressions(&mut Spec)` runs after `resolve_actions`, before `render_spec_to_html_with_plugins`. Walker stays expression-unaware. | ✓ |
| Inline in walker | Each per-component renderer checks for `$data`/`$template` in its props and resolves on the fly. | |
| Resolver invoked by callers per request | Skip `JsonUi::resolve` integration; callers manually call `resolve_expressions` before `JsonUi::render`. | |

**Auto choice:** Option 1 (pre-render pass).
**Rationale:** Phase 116 D-29 explicitly keeps the renderer expression-unaware. Pre-render pass is testable in isolation and gives Phase 119's loader a clean hook. `JsonUi::resolve` already clones the spec for `resolve_actions`; folding expression resolution into the same hop avoids a second copy.

---

## Path syntax

| Option | Description | Selected |
|--------|-------------|----------|
| Slash-separated `/segment/segment` for both `$data` and `$template` | Reuse `data::resolve_path`. Roadmap example `{user.name}` becomes `{/user/name}`. Single mental model across the codebase. | ✓ |
| Slash for `$data`, dot for `$template` placeholders | Roadmap example as-written: `{"$data": "path/to/value"}` and `{"$template": "{user.name}"}`. Two parsers, two conventions. | |
| Dot-separated `user.name` for both | Drop the leading slash everywhere. Breaks every existing `data_path` field. | |

**Auto choice:** Option 1 (slash everywhere).
**Rationale:** Five `*Props` structs already use `/segment/segment` for `data_path`; introducing dot syntax inside `$template` only fragments authoring. PROJECT.md "small mental model" criterion makes consistency the right call. Roadmap examples are illustrative; the binding contract is criteria 1–6 ("data paths"), which are syntax-agnostic.

---

## Resolution shape

| Option | Description | Selected |
|--------|-------------|----------|
| Mutating clone (`fn resolve_expressions(&mut Spec)`) | Caller clones first, then runs the resolver in place. Mirrors `resolve_actions` signature. | ✓ |
| Owned in/out (`fn resolve_expressions(Spec) -> Spec`) | Functional flavor; forces a move. | |
| Method on Spec (`Spec::resolve_expressions`) | Object-oriented placement. | |

**Auto choice:** Option 1 (`&mut Spec`).
**Rationale:** Direct parallel with `resolve_actions(&mut spec, resolver)` and `resolve_errors(&mut spec, errors)`. Keeps `JsonUi::resolve` flow uniform: one clone, multiple in-place transformations.

---

## Substitution scope

| Option | Description | Selected |
|--------|-------------|----------|
| `Element.props` recursive only | Walk `spec.elements.values_mut()` → `el.props` → recurse through Object/Array. Skip Spec metadata, children, action, visible, type_name, Spec.data itself. | ✓ |
| Everything except IDs | Also walk Spec.title, Spec.layout, Element.action.url, etc. | |
| Props + visibility | Replace `Visibility` evaluator with `$data` lookups. | |

**Auto choice:** Option 1 (props-only).
**Rationale:** `Spec.data` is the substitution source and must stay literal (recursion hazards). Metadata fields (`title`, `layout`) are server-side configuration. `Element.children` are IDs validated by Phase 115; expression substitution there would lose cycle/dangling guarantees. `Element.action.url` is `resolve_actions`-managed. `Element.visible` has its own typed `Visibility` evaluator. Restricting to `props` keeps the resolver's scope obvious.

---

## Template placeholder syntax

| Option | Description | Selected |
|--------|-------------|----------|
| Single braces with slash paths: `"Hi {/user/name}!"` | One placeholder convention. Trim whitespace inside braces. Escapes: `\{`, `\}`, `\\`. | ✓ |
| Mustache double braces: `"Hi {{/user/name}}!"` | Less ambiguous but distinct from JSON braces, more typing. | |
| Dot paths inside single braces: `"Hi {user.name}!"` | Matches roadmap example literally. Fragments the path-syntax story. | |

**Auto choice:** Option 1 (single braces, slash paths).
**Rationale:** Single placeholder convention is clearer; reusing slash paths keeps a single mental model. Single braces are common (sprintf, Python format) and fit the roadmap's example shape. Hand-rolled scanner; no `regex` dep.

---

## $data type semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Type-preserving | `{"$data": "/x"}` substitutes whatever JSON type is at `/x` — number, bool, object, array, null. | ✓ |
| Always stringify | Force every `$data` result through `to_string()`-style coercion. | |
| String for content slots, typed for everything else | Special-case based on prop slot type. | |

**Auto choice:** Option 1 (type-preserving).
**Rationale:** Criterion 3 says expressions work in all positions (string, number, boolean values). Forcing strings would break `ProgressProps.value: f64`, `CheckboxProps.checked: bool`, `DataTableProps.columns: Vec<Column>`. Type preservation lets `Catalog::validate` see the resolved spec without false positives.

---

## Missing path behavior

| Option | Description | Selected |
|--------|-------------|----------|
| `$data` → `Value::Null`; `$template` placeholder → empty string | Mirrors HTML/UI norms — missing data shouldn't leak debug brackets. | ✓ |
| Leave expression markers verbatim when resolution fails | Render-time HTML comment surfaces the miss. | |
| Diagnostic comment in resolved spec | Insert `<!-- missing path -->` strings in resolved values. | |

**Auto choice:** Option 1 (null/empty).
**Rationale:** Matches criterion 4 ("missing data paths resolve to null/empty — never panic"). Catalog/render diagnostics handle author errors; the resolver stays a faithful substitution layer.

---

## Recursion / single-pass

| Option | Description | Selected |
|--------|-------------|----------|
| Single-pass | `$data` substitutes once; if the substituted value contains another marker, it stays literal. | ✓ |
| Multi-pass to fixed point | Re-resolve until no markers remain. | |
| Bounded depth (e.g., 3 passes) | Compromise. | |

**Auto choice:** Option 1 (single-pass).
**Rationale:** Recursive resolution opens billion-laughs / expansion-attack surface and adds inner-platform-effect risk. `spec.data` should hold ground-truth values; expressions live in `props`. Single-pass keeps cost bounded and the cap clean.

---

## Validation order

| Option | Description | Selected |
|--------|-------------|----------|
| Resolve before validate (Phase 118 wires `JsonUi::resolve`; Phase 119 wires the loader) | `Catalog::validate` sees the resolved typed-shape props. Schema stays narrow. | ✓ |
| Validate before resolve | Forces `oneOf: [String, ExpressionObject]` on every prop slot — catalog balloons to 80+ KB. | |
| Validate at both points | Double cost; no extra safety. | |

**Auto choice:** Option 1 (resolve → validate).
**Rationale:** Only ordering that lets typed `*Props` schemas remain authoritative. Phase 117 caveats already flagged the catalog-size risk this avoids.

---

## Performance posture

| Option | Description | Selected |
|--------|-------------|----------|
| Always walk every props value | Simple, predictable, sub-millisecond on typical pages. | ✓ |
| Pre-scan for expression markers, walk only if found | Adds a second pass for the same cost. | |
| `Cow<Spec>` to avoid clone when no expressions present | Complicates public API; second clone path. | |

**Auto choice:** Option 1 (always walk).
**Rationale:** Total JSON-node count in props is small for typical Phase 116 pages. Phase 116 D-28 deferred render-cache for the same cost/benefit reason — expression resolution follows the same posture. Phase 121 field test is the natural moment to measure.

---

## Module layout

| Option | Description | Selected |
|--------|-------------|----------|
| Single new file `ferro-json-ui/src/expression.rs` | Mirrors `action.rs`, `visibility.rs`, `resolve.rs` convention. Inline tests. | ✓ |
| New subdirectory `ferro-json-ui/src/expression/` | Premature; current file size estimate ≤ 600 LOC. | |
| Extend `resolve.rs` | Mixes structurally-different concerns (action URLs vs. value substitution). | |

**Auto choice:** Option 1 (single file).
**Rationale:** Single-file Phase 118 keeps the change auditable and reverts cleanly if the design needs to evolve. Matches the file-per-feature convention.

---

## Error surface

| Option | Description | Selected |
|--------|-------------|----------|
| Infallible (`fn resolve_expressions(&mut Spec)` returns `()`) | No Result, no log. Malformed expressions degrade to literal JSON; missing paths null/empty. Catalog/render diagnostics surface real problems. | ✓ |
| `Result<(), ResolveError>` | Resolver fails on malformed shapes. Adds a new error type. | |
| HTML comment side channel | Resolver writes diagnostics into the resolved JSON. | |

**Auto choice:** Option 1 (infallible).
**Rationale:** Symmetry with `resolve_actions` and the Phase 116 D-09 infallible-renderer posture. Diagnostics belong to the validator and renderer, not the substitution helper.

---

## Catalog schema interaction

| Option | Description | Selected |
|--------|-------------|----------|
| Catalog stays typed; resolver runs upstream of validator | Schema unchanged. Phase 118 documents the ordering. Phase 119 enforces it. | ✓ |
| Catalog gains `oneOf: [TypedShape, ExpressionObject]` per prop | Expression-aware schema. Adds 2-3× to schema size; Phase 117 caveats already warned about size growth. | |

**Auto choice:** Option 1 (no schema change).
**Rationale:** Keeps the catalog schema narrow (Phase 117 D-33 stance). Resolution is the bridge that lets validator and renderer stay expression-unaware on opposite ends of the pipeline.

---

## Plugin handling

| Option | Description | Selected |
|--------|-------------|----------|
| Plugins get expression substitution for free | Resolver is type-agnostic; recurses into `el.props` whether typed or untyped. | ✓ |
| Plugins must opt in | Add a flag on `JsonUiPlugin` trait. | |
| Plugins are excluded | Skip the resolver for non-`BUILTIN_TYPES`. | |

**Auto choice:** Option 1 (free for plugins).
**Rationale:** Plugin authors benefit from `$data` for the same reasons built-ins do. The resolver is type-agnostic by design, so this falls out without special-casing.

---

## Claude's Discretion

Areas left to the planner / implementer to choose without further direction:

- Hand-rolled scanner vs tiny dep for `$template` parsing (prefer hand-rolled — grammar is trivial, zero new deps is the cheap right answer).
- Whether `is_data_expr` / `is_template_expr` are free functions or methods on a private enum.
- Exact split between unit and integration tests when both cover the same end-to-end behavior.
- Whether `expression.rs` exports any helper types beyond `resolve_expressions` (only export what consumers actually need).

## Deferred Ideas

Tracked in CONTEXT.md `<deferred>` section. Includes: `$if`/`$for`/`$switch` (banned by criterion 6), `$state`/`$bindState` (PROJECT-level out-of-scope), `$ref` (Phase 115 deferred), markers inside `Spec.data` (D-04 banned), markers inside `Element.children` (D-04 banned), JSON Pointer escape compliance, multi-pass resolution (D-07 banned), schema-level expression markers (D-13 banned), template placeholders that read non-`spec.data` sources, AI generation emitting expressions (Phase 120), gestiscilo migration (Phase 121).

---

*Discussion auto-completed 2026-04-19. No interactive questions asked. Decisions captured in 118-CONTEXT.md.*
