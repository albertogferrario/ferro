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

## Code Review Findings Deferred (from 162-REVIEW.md)

Surfaced by `/gsd-code-review 162` at phase landing. Deferred for triage in a follow-up phase rather than landing inline at the close of Phase 162.

- **WR-01 — RichTextEditor Quill assets lack `.crossorigin("")`.** `ferro-json-ui/src/plugins/rich_text_editor.rs:96-100` calls `.integrity(...)` without `.crossorigin("")`. Browsers require crossorigin to send a CORS request before the SRI hash can be checked on cross-origin CDN resources, so the integrity attribute is silently non-functional in its current form. The Map plugin uses `.crossorigin("")` on its CDN assets as the established pattern. Fix: append `.crossorigin("")` to both Asset builders in `css_assets()` and `js_assets()` and extend `rich_text_editor_plugin_assets_carry_sri_hashes` to assert the crossorigin field.
- **WR-02 — json_ui_verify_action returns nearest candidate with no distance threshold.** `ferro-mcp/src/tools/json_ui_verify_action.rs::find_handler` always returns the nearest route name by Levenshtein distance even when that candidate is semantically unrelated to the query. In a large route table this actively misleads agent consumers. Fix: gate the candidate on `min(handler.len() / 2, 8)` (or similar) and return `candidate: None` when no near match exists.
- **WR-03 — Dead public export `register_built_in_plugins`.** `ferro-json-ui/src/plugins/mod.rs:16-19` exposes `register_built_in_plugins()` re-exported through `lib.rs:78`. `global_plugin_registry()` already registers built-ins via `OnceLock::get_or_init`, so the helper is uncalled and would double-register if invoked. Fix: remove the helper and its re-export, or annotate it `#[doc(hidden)]` with a note that it is safe-but-redundant.
- **IN-01 — CheckboxList `selected_path` silently drops non-string array elements.** No diagnostic when a `selected_path` resolves to an array containing numbers / objects. Consider an `eprintln!` warning or surface through `validate_structure`.
- **IN-02 — RichTextEditor `initial_esc` placed in `<div>` body is dead.** Quill always overwrites the container body from `input.value` in the IIFE, so the escaped initial value inside the div is unreachable. Harmless dead code; can be removed once the contract is documented.

## Detailed next-phase inputs

Tracked in `162-CONTEXT.md` `<followups>` section. Expected Phase 163 / Phase 164 inputs:

- `$each` directive — spec-level iteration over a data array for homogeneous element shapes.
- `$if` directive — conditional element emission.
- `$template` element with auto-suffixed IDs — closes products detail edit-mode case.
- `SpecBuilder` ergonomic nested DSL — reduces Rust-side spec-construction friction.
- `ferro json-ui:migrate-v1` codemod — auto-rewrites `make_node(id, Component::X(props))` call trees into stub JSON spec entries.
