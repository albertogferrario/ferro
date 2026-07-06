# Phase 115: Spec v2 Data Structures - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-18
**Phase:** 115-spec-v2-data-structures
**Mode:** `--auto` — Alberto's directive: "make a well designed implementation, inspire from already existing tools, do as this is your product"
**Areas discussed:** Props struct treatment, Validation enforcement points, Plugin components in v2, Caller breakage scope

---

## Props Struct Treatment

| Option | Description | Selected |
|--------|-------------|----------|
| Keep typed Props structs | ~40 CardProps/TableProps/... survive as JsonSchema-bearing Rust types; Phase 117 Catalog reflects on them. Strip `Vec<ComponentNode>` children. | ✓ |
| Delete typed Props structs | Element.props is opaque `serde_json::Value`; schemas hand-written from scratch in Phase 117. | |
| Hybrid (keep for built-ins, erase for future plugins) | Built-ins keep typed props, plugin-like components are purely Value. | |

**Selected:** Keep typed Props structs (recommended default).
**Rationale:** Rust types as source of truth mirrors Vercel json-render (Zod) and rjsf (JSON Schema). Phase 117's per-component schema story needs these structs — deleting them creates 40 hand-written schemas as debt. Stripping `Vec<ComponentNode>` fields lets most structs derive `JsonSchema` cleanly; manual impls survive only for the irreducibly recursive ones.

---

## Validation Enforcement Points

| Option | Description | Selected |
|--------|-------------|----------|
| Parse-time structural + deferred semantic | `Spec::from_json()` fails fast on dangling refs / cycles / depth / ID format. Semantic (types, enum constraints) waits for Phase 117's Catalog. | ✓ |
| Explicit `Spec::validate()` only | Parsing succeeds for any syntactically-valid JSON; callers must call `validate()` themselves. | |
| Render-time only | Validation runs inside the renderer; no upfront checks. | |

**Selected:** Parse-time structural, deferred semantic (recommended default).
**Rationale:** Structural errors are programming bugs — discover them early. Semantic validation requires the Catalog which is Phase 117's deliverable; forcing it into Phase 115 means either a shadow Catalog or early validator wiring that will be rewritten. Matches Protobuf (structural-at-parse) and React (validate-at-reconcile) discipline.

**Scope of Phase 115 structural checks:**
1. Root element exists in `elements` map.
2. Every child ID resolves to an existing element (no dangling refs).
3. No cycles in the element graph (DFS with visited set).
4. Nesting depth ≤ 3 from root.
5. Element IDs match `^[A-Za-z_][A-Za-z0-9_-]{0,127}$`.
6. Duplicate IDs in raw JSON maps are detected and rejected (not silently overwritten).

---

## Plugin Components in v2

| Option | Description | Selected |
|--------|-------------|----------|
| Fully type-erased Element | `type_name: String` + `props: Value`. No built-in vs plugin distinction at the type level. Catalog/renderer decides at dispatch time. | ✓ |
| Typed enum with Plugin escape variant | `ElementKind::Builtin(BuiltinType) \| Plugin(String)`. Keeps compile-time guarantees for built-ins. | |
| String + explicit `is_plugin` bool | Redundant with Catalog knowledge — adds noise. | |

**Selected:** Fully type-erased Element (recommended default).
**Rationale:** Vercel json-render's flat model validates this at scale. Type-erasure kills the v1 `Component::Plugin` wart, removes the ~40-variant custom Serialize/Deserialize block (~200 lines), and makes built-ins and plugins symmetric. Phase 117 can reason about type_name uniformly by consulting the Catalog.

**Related Phase 117 strategy (documented, not implemented in 115):** plugins register a `JsonSchema` at registration time → Catalog dynamically assembles `oneOf` from built-ins + currently-registered plugins. Phase 115's only obligation is leaving `Element` open so this works later.

---

## Caller Breakage Scope in Phase 115

| Option | Description | Selected |
|--------|-------------|----------|
| Rewrite callers with a placeholder renderer | Phase 115 delivers v2 types + placeholder render + caller migration. Workspace stays green; real renderer arrives in Phase 116. | ✓ |
| Delete v1, stub callers with `todo!()` / feature gates | Workspace may fail to build sample app without feature flag gymnastics. | |
| Keep v1 alive in Phase 115, delete in Phase 116 | Contradicts roadmap success criterion #4 ("v1 types are deleted — clean break, no v1 types remain"). | |

**Selected:** Rewrite callers with a placeholder renderer (recommended default).
**Rationale:** "Clean break" is a project-level norm (per user memory and `.planning/PROJECT.md`). The placeholder — a pretty-printed spec JSON dump inside an HTML skeleton — is cheap to write and throwaway in Phase 116. Every commit lands green, matching the workspace's pre-push lint + test gate.

**Callers migrated in Phase 115:**
- `framework/src/json_ui/mod.rs` — `JsonUi::render` signature switches to `&Spec`.
- `framework/src/lib.rs` — re-exports updated.
- Sample `app/` crate — JsonUiView construction → `Spec::builder()`.
- `ferro-mcp/src/service.rs` and `tools/render_projection.rs` — output type follows `JsonUiRenderer::Output = Spec`.
- `ferro-mcp` `code_templates` / `json_ui_*` tools — type signatures updated; semantic rewrites wait for Phase 120.

---

## Claude's Discretion

- Internal module layout within `spec.rs` (single file vs. split).
- Exact error-variant serialization format inside `SpecError` (thiserror convention).
- Placeholder renderer body shape — raw JSON dump, `<pre>`, or minimal type-aware walk.
- Naming of internal constants (`MAX_NESTING_DEPTH = 3`).
- Whether `PluginProps` convenience struct is kept post-deletion of `Component::Plugin` variant.

## Deferred Ideas

- Catalog / JSON Schema assembly — Phase 117.
- Plugin schema registration API — Phase 117.
- `$data` / `$template` expression evaluation — Phase 118.
- Spec loader with hot reload — Phase 119.
- MCP two-tier AI strategy — Phase 120.
- gestiscilo field test conversion — Phase 121.
- IDE plugin / cross-spec composition — v13.0+ backlog.
