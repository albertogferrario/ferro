# Phase 162: Deferred Items

**Deferred:** 2026-05-16
**Source:** Extracted from `162-CONTEXT.md` `<deferred>` section.

Phase 162 deliberately stays narrow — only items justified by the four gestiscilo Phase 138 controllers AND the blast-radius API decisions. Items observed but explicitly deferred:

- `$each` / `$if` / `$template` spec-level iteration directives — deferred to Phase 163 (gestiscilo Phase 140 cassa, where heterogeneous iteration provides the forcing function).
- `SpecBuilder` ergonomic nested DSL — deferred to Phase 163.
- `ferro json-ui:migrate-v1` codemod — deferred to Phase 163.
- Multi-step form patterns, `visible` rule expressiveness at depth, PDF preview routing — deferred to Phase 164 (gestiscilo Phase 142 documenti).
- Host-based tenancy gap — out of JSON-UI scope; tracked in `.planning/backlog/host-based-tenancy.md` for a dedicated tenancy-layer phase. The `PreRouteMiddleware.rewrite` → `handle` rename surfaced in the blast radius is a one-line gestiscilo-side fix and does not bring the tenancy work forward.
- `Fragment` / `Group` borderless container (D-06) — explicitly rejected for Phase 162. Revisit only if a future phase finds a use case that D-05 + existing containers (Grid 1-col, FormSection without title) cannot express.
- `#[handler(name = "...")]` attribute (D-10) — explicitly rejected for Phase 162. Revisit only if a future use case justifies a second source of truth for route names.

## SRI Hash TODO (from Plan 162-04) — RESOLVED in Phase 162

`ferro-json-ui/src/plugins/rich_text_editor.rs` Quill 2.0.3 CDN assets now carry sha384 SRI integrity hashes pinned to the jsdelivr-served bytes:

- `quill.snow.css`: `sha384-ecIckRi4QlKYya/FQUbBUjS4qp65jF/J87Guw5uzTbO1C1Jfa/6kYmd6dXUF6D7i`
- `quill.js`: `sha384-utBUCeG4SYaCm4m7GQZYr8Hy8Fpy3V4KGjBZaf4WTKOcwhCYpt/0PfeEe3HNlwx8`

Pinned via `rich_text_editor_plugin_assets_carry_sri_hashes` unit test. T-162-04-02 closed.

## Detailed next-phase inputs

Tracked in `162-CONTEXT.md` `<followups>` section. Expected Phase 163 / Phase 164 inputs:

- `$each` directive — spec-level iteration over a data array for homogeneous element shapes.
- `$if` directive — conditional element emission.
- `$template` element with auto-suffixed IDs — closes products detail edit-mode case.
- `SpecBuilder` ergonomic nested DSL — reduces Rust-side spec-construction friction.
- `ferro json-ui:migrate-v1` codemod — auto-rewrites `make_node(id, Component::X(props))` call trees into stub JSON spec entries.
