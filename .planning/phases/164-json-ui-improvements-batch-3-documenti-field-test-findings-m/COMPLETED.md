# v12.0 JSON-UI Friction Loop — Completion Summary

Phases 162, 163, 163.1, and 164 collectively absorbed the gestiscilo Phase 138–143 compile-time migration friction and the V7-RUNTIME-FRICTION.md runtime findings. This document is the authoritative record of what was shipped, what was explicitly excluded, and what is deferred to future milestones.

**Phase 160 (v1 deletion) is UNBLOCKED.** The v1 deletion audit in Section 5 contains zero BLOCKER rows.

**Inputs to downstream phases:**

- **Phase 160** (v1 deletion) — reads Section 5 (v1 → v2 surface migration table) to confirm deletion is safe.
- **Phase 161** (v12.0 merge + publish) — Section 1 is the basis of the v12.0 CHANGELOG entry.
- **Future v12.x phases** — Section 4 (Deferred) provides scope candidates.

---

## 1. Shipped across Phases 162–164

Organized by concern. Each row cites the phase, decision ID, the delivered artifact, and its commit or plan reference.

### New and restored components

| Phase / Decision | Component or prop | What changed | Ref |
|------------------|-------------------|--------------|-----|
| 162 D-01, D-02 | `CheckboxList` | New first-class component: data-driven multi-select (`field`, `options`, `options_path`, `selected_path`); XSS-safe HTML emission | Plan 162-01 |
| 162 D-16 | `SwitchProps.compact` | Restored `compact: Option<bool>` field (pure CSS scale-75 toggle) — 6 gestiscilo settings sites unblocked | Plan 162-03 |
| 162 D-17 | `ImageProps::inline_svg` | Restored `ImageSource::InlineSvg { svg: String }` variant — server-constructed SVG bar charts | Plan 162-03 |
| 162 D-18 | `RichTextEditor` (as plugin) | Re-implemented as v2 plugin element (`RichTextEditorProps`): `field`, `label`, `default_value`, `data_path`, `error`; Quill 2.0.3 asset injection via `register_built_in_plugins` | Plan 162-04 |
| 164 D-17a | `RawHtml` | New narrow primitive: single `html: String` field; verbatim emission in `<div data-ferro-raw-html>`; safety docstring; closes the `"type": "Plugin"` runtime block (F9) | Plan 164-03 |
| 164 D-18 | `CardVariant` | New enum (`Bordered` default / `Elevated`) on `CardProps.variant`; `Elevated` gives `shadow-md + p-8` without border for auth/error/marketing pages; closes F10 | Plan 164-05 |

### DataTable and per-row action ergonomics

| Phase / Decision | Change | Ref |
|------------------|--------|-----|
| 162 D-03 | `DataTable.row_actions[].action.url` — `{row_key}` placeholder interpolation at render time | Plan 162-02 |
| 162 D-04 | Generalized to all column keys (`{label}`, `{slug_path}`, `{status}`, …) — same render-time substitution logic | Plan 162-02 |

### Auth layout

| Phase / Decision | Change | Ref |
|------------------|--------|-----|
| 162 D-05 | Auth layout card wrapper removed. The template now provides structural centering only; each spec declares its own `Card` root. Breaking change; all auth-using specs already had `Card` roots. | Plan 162-06 |

### Spec validation

| Phase / Decision | Change | Ref |
|------------------|--------|-----|
| 162 D-07 | Spec validator emits an error when a footer-referenced element ID is absent from the `elements` map | Plan 162-07 |
| 162 D-08 | Spec validator emits a warning when the same element ID appears in both `props.footer` and `children` of the same parent | Plan 162-07 |
| 163 D-12 (case 1) | `$each.path` resolves-to-array check (`SpecError::EachPathNotArray`) | Plan 163-04 |
| 163 D-12 (case 2) | `$if.path` resolves-cleanly check (`SpecError::IfPathMissing`) | Plan 163-04 |
| 163 D-12 (case 3) | Circular-ref guard in templated elements (`SpecError::NestedEach`, `SpecError::MismatchedEach`) | Plan 163-04 |
| 164 D-05 (case 4) | Children references to `$if`-gated elements allowed at parse time (correct behavior confirmed + regression test added) | Plan 164-09 |
| 164 D-16 | Two-stage validation: structural hard-fail at load; catalog/enum-shape validation deferred to after `expand_directives` at render time (closes F8 — `Alert.variant="" + visible` no longer blocks startup) | Plan 164-07 |

### Spec depth and data binding

| Phase / Decision | Change | Ref |
|------------------|--------|-----|
| 164 D-14 (F4) | `MAX_NESTING_DEPTH` raised from 3 to 5; depth-6 specs rejected with `SpecError::DepthExceeded`; documented in `spec-construction.md` | Plan 164-01 |
| 164 D-12 (F1) | `Spec.title` accepts literal `String` or `{"$data": "/path"}` binding via `TitleBinding` enum; renderer resolves bindings via JSON Pointer at response-build time | Plan 164-04 |
| 164 D-15 (F7) | `ImageProps.data_path: Option<String>` — resolves `src` dynamically; falls back to static `src` | Plan 164-03 |
| 164 D-15 (F7) | `DescriptionListProps.data_path: Option<String>` — resolves `items` dynamically; `items` made `#[serde(default)]` | Plan 164-03 |
| 164 D-13a (F3) | `KanbanBoardProps.data_path: Option<String>` — resolves columns from handler data; `data_path` wins over static `columns`; `columns` made `#[serde(default, skip_serializing_if)]` | Plan 164-06 |

### Variant enum ergonomics

| Phase / Decision | Change | Ref |
|------------------|--------|-----|
| 162 D-11 | `#[derive(strum::AsRefStr)]` added to `AlertVariant`, `BadgeVariant`, `ButtonVariant`, `ToastVariant`, `DialogVariant`, `NotifyVariant` — call-site ergonomics; wire format unchanged | Plan 162-08 |

### Iteration directives

| Phase / Decision | Directive | What was delivered | Ref |
|------------------|-----------|--------------------|-----|
| 163 D-01, D-02 | `$each` | Wire-format field on `Element`; instantiates one element per item in a data array at resolve time; auto-suffixed IDs (`element_id-0`, `element_id-1`, …) | Plans 163-01, 163-03 |
| 163 D-03 | `$if` | Conditional element removal before catalog validation; elements whose `$if` evaluates falsy are not rendered (no hidden DOM) | Plans 163-02, 163-03 |
| 163 D-04 | `expand_directives` resolve-time pass | Orchestrates `$each` expansion and `$if` removal before validation and render | Plan 163-03 |

### Spec construction ergonomics

| Phase / Decision | Change | Ref |
|------------------|--------|-----|
| 163 D-06, D-07 | `NestedElement` ergonomic nested-tree DSL on `SpecBuilder` — builds the flat `elements` map from nested Rust types | Plan 163-05 |

### Migration codemod (`ferro json-ui:migrate-v1`)

| Phase / Decision | Change | Ref |
|------------------|--------|-----|
| 163 D-09, D-10, D-11 | `ferro json-ui:migrate-v1 <FILE>` — AST-based (syn); rewrites `make_node` call trees into stub JSON spec + `JsonUi::render_file` controller; idempotent; `--dry-run` flag; TODO markers for runtime-branching handlers | Plan 163-07 |
| 163.1 WR-01 | Multi-root handler guard: `top_ids.len() != 1` returns `HandlerResult::Unsupported` before spec construction — closes silent data-loss bug where the first root was used and remaining elements were orphaned | Plan 163.1-01 |
| 164 D-19/F2 | Regression test locking uppercase HTTP method emission (POST/GET/PUT/PATCH/DELETE) — the codemod already emitted uppercase; the test prevents regression | Plan 164-02 |

### Deserialization ergonomics

| Phase / Decision | Change | Ref |
|------------------|--------|-----|
| 164 D-19/F5 | Hand-rolled `Visibility::Deserialize` impl — dispatches by key presence; on no match emits the offending JSON and all four accepted shapes (was: opaque `data did not match any variant of untagged enum Visibility`) | Plan 164-08 |
| 164 D-19/F6 | Lax deserializer on `PageHeaderProps.actions` — accepts `null`, `""`, `[]`, and `["a", "b", ...]`; Rust type stays `Vec<String>` | Plan 164-08 |

### MCP and tooling

| Phase / Decision | Tool | What was delivered | Ref |
|------------------|------|--------------------|-----|
| 162 D-09 | `json_ui_verify_action` | Confirms a handler name is registered as a named route; returns closest Levenshtein candidate on miss | Plan 162-09 |
| 162 D-21 | `json_ui_catalog` (extended) | Built-in count maintained in lockstep across `BUILTIN_TYPES`, `BUILTIN_SPECS`, and MCP catalog assertion; count bumped with each new component | Plan 162-05 |
| 163 D-13 | `json_ui_catalog` (directives) | `$each` and `$if` directives reflected in MCP catalog tool output | Plan 163-06 |
| 164 D-04 | `json_ui_validate_spec` | New MCP tool — two-stage pipeline: structural errors from `Spec::from_json` in `structural_errors`; catalog errors from `Catalog::validate` in `catalog_errors`; always returns a `ValidateResponse` regardless of error set | Plan 164-09 |

### Documentation

| Phase / Decision | Document | What was delivered | Ref |
|------------------|----------|--------------------|-----|
| 162 D-20 | `docs/src/json-ui/migration-v1-to-v2.md` | New migration guide: `JsonUi::render_file` vs `Spec::builder`, depth-flattening pattern, per-row action interpolation, inline view/edit, data-driven options with CheckboxList, variant string round-trip, handler-name verification via MCP | Plan 162-10 |
| 162 D-19 | `docs/src/json-ui/plugins.md` | v2 plugin author guide: `JsonUiPlugin` trait, `register_plugin`, `Asset` system, consumer-facing examples | Plan 162-10 |
| 162 D-22 | `ferro-mcp` `code_templates` | v1→v2 migration patterns surfaced as code-snippet templates for agent authoring | Plan 162-10 |
| 163 D-08 | `docs/src/json-ui/spec-construction.md` | Decision rubric (JSON file / `$each` / `$if` / Rust `SpecBuilder` — when to use each); directive worked examples | Plan 163-09 |
| 163 D-13 | `docs/src/json-ui/expressions.md` | `$each` / `$if` reference sections | Plan 163-09 |
| 164 D-08 | `docs/src/json-ui/components.md` | New sections: `CardVariant` (Bordered/Elevated), `Image.data_path`, `DescriptionList.data_path`, `KanbanBoard.data_path`, `PageHeader.actions` lax acceptance, `RawHtml` with trust-boundary call-out, `CalendarCell` (was missing), `CheckboxList` (full section); Component Overview table updated | Plan 164-10 |
| 164 D-08 | `docs/src/json-ui/spec-construction.md` | Added `Spec.title` binding section: literal vs `{"$data": "/path"}`, fallback behavior, examples | Plan 164-10 |
| 164 D-09 | `docs/src/json-ui/migration-v1-to-v2.md` | Added 10-row cheat sheet at top of file; corrected stale "Depth limited to 3 levels" to 5 | Plan 164-10 |
| 164 D-13b | `docs/src/json-ui/expressions.md` | Added `$each`-for-kanban worked example (JSON spec + handler data + comparison table) | Plan 164-10 |
| 164 D-06 | `docs/src/json-ui/plugins.md` | D-06 paper-audit gaps fixed: `render(props, data)` second argument documented; `init_script()` per-page-once semantics documented; `When to use RawHtml instead` section added | Plan 164-10 |

---

## 2. Runtime frictions resolved (V7-RUNTIME-FRICTION.md F1–F10)

| F# | Friction (gestiscilo-side symptom) | Resolution | Shipped in |
|----|-------------------------------------|------------|------------|
| F1 | `Spec.title` rejects `{"$data": "/path"}` bindings; 23 specs had to strip bindings via `sed` | ferro fix: `TitleBinding` enum (D-12) | Plan 164-04 |
| F2 | HTTP method values emitted lowercase by codemod; 26 gestiscilo specs required `sed` uppercase correction | gestiscilo workaround already applied; ferro codemod verified-uppercase via regression test (D-19/F2) | Plan 164-02 |
| F3 | `KanbanBoard` has no `data_path` prop; dashboard kanban blocked | ferro fix: `KanbanBoardProps.data_path` (D-13a) | Plan 164-06 |
| F4 | `MAX_NESTING_DEPTH = 3` blocks depth-4 dashboard specs (root → grid → card → badge) | ferro fix: raised to 5 (D-14) | Plan 164-01 |
| F5 | `Visibility` enum parse fails with opaque error; debugging requires trial and error | ferro fix: hand-rolled `Visibility::Deserialize` naming offending shape and all accepted forms (D-19/F5) | Plan 164-08 |
| F6 | `PageHeader.actions` rejects `""` (empty string); controllers that pass `""` for no-actions state fail | ferro fix: lax deserializer accepting `null`, `""`, `[]`, and `[string…]` (D-19/F6) | Plan 164-08 |
| F7 | `Image.src` and `DescriptionList.items` are static-only; dynamic content blocked | ferro fix: `data_path` on both (D-15) | Plan 164-03 |
| F8 | Catalog validation runs against raw spec, blocking startup when an `Alert.variant=""` is inside a `visible`-gated element | ferro fix: two-stage validation — load-time warn, render-time enforce post-`expand_directives` (D-16) | Plan 164-07 |
| F9 | `"type": "Plugin"` unrecognized by v2 catalog; gestiscilo settings pages blocked | ferro fix: `RawHtml` primitive (D-17a) for HTML-island use cases; v2 plugin registry path documented as the recommended alternative for richer widgets (D-17b) | Plan 164-03 |
| F10 | Auth/error/marketing pages render with dashboard Card chrome (`border + shadow-sm + p-4`) after Phase 162 D-05 removed the layout-level card wrapper | ferro fix: `CardVariant::Elevated` (`shadow-md + p-8`, no border) on `CardProps.variant` (D-18) | Plan 164-05 |

---

## 3. Intentional gaps

Features explicitly not shipped in the v12.0 friction loop, with rationale.

**`Fragment` / `Group` borderless container (Phase 162 D-06 rejected)** — The underlying double-card problem was resolved by removing the layout-level card wrapper (D-05). A borderless container element with no semantic role and no visual chrome does not earn catalog surface. Consumers wanting no wrapper use the existing `children` slot on the parent directly, or a single-column `Grid`.

**`#[handler(name = "...")]` attribute (Phase 162 D-10 rejected)** — Route names are already registered at `route!`/`get!`/`post!` macro call sites via `.name("…")`. A second naming attribute would create two name sources that can drift. The MCP `json_ui_verify_action` tool (D-09) is the verification path; it reads the single existing source of truth.

**`$template` separate element type (Phase 163 D-05 rejected)** — `$each` covers the templated-element-tree case (one element per array item, auto-suffixed IDs). A parallel `$template` mechanism would fragment the directive surface without adding expressibility.

**Codemod directory-recursive mode (Phase 163 D-10 rejected)** — File-at-a-time keeps the codemod's failure mode explicit and auditable. A directory-recursive mode invites silent partial migration where a handler the codemod cannot translate produces a TODO marker that goes unreviewed.

**Modal chrome variant sweep** — `Modal` already uses `shadow-lg` (closer to `Elevated` than to `Bordered`). No gestiscilo friction surfaced from this. Accepted as no-action for v12.0.

**Granular `padding` / `elevation` props on `CardProps`** — Two variants (`Bordered`, `Elevated`) cover the gestiscilo use cases. A 2×2 prop matrix would be more surface area for less benefit; deferred pending real demand.

**`Component::Plugin` generic dispatch reintroduction** — Phase 115 D-01 removed this. Phase 164 D-17a (`RawHtml`) is a narrow, intentionally limited replacement for HTML-island cases (single `html` field, verbatim emission). Plugin widgets with structured props use named types registered via `JsonUiPlugin`. The generic dispatch is not reinstated.

**`ammonia` HTML sanitizer as a workspace dependency** — `RawHtml` and `RichTextEditor` emit verbatim HTML. Sanitization discipline is enforced at the consumer's handler, not at the renderer. Pushing sanitization to the consumer keeps the renderer simple and the trust boundary explicit.

---

## 4. Deferred to future milestones

Items surfaced during the friction loop but pushed past v12.0.

- **Host-based tenancy gap** — A separate tenancy-layer phase; tracked in `.planning/backlog/host-based-tenancy.md`. The `PreRouteMiddleware.rewrite` → `handle` rename (Phase 162 blast radius) is a consumer-side one-liner, not a ferro change.
- **Codemod directory-recursive mode** — Re-evaluation deferred. The file-at-a-time constraint is intentional for v12.0; revisit if batch-migration demand surfaces.
- **Advanced expression operators (arithmetic, string concatenation, ternary)** — Only `$data` reference bindings ship in v12.0. Richer expression syntax is a v12.1+ topic contingent on consumer demand.
- **Granular `Card` props (`padding`, `elevation`)** — Only `CardVariant` (two-value enum) ships in v12.0. Reopen if a use case emerges that neither `Bordered` nor `Elevated` can express.
- **Modal chrome variant** — `Modal` left unchanged; the shadow-class already matches the Elevated aesthetic. A dedicated `ModalVariant` field is deferred pending demand.
- **`LoadError::Catalog` variant cleanup** — The enum variant still exists (produced by the `load_builtins` test helper). It could be marked deprecated in a follow-up phase once all producers of the hard-fail catalog path are confirmed migrated.
- **`$if` evaluate-at-render-time visibility parity** — The existing `visible` field is evaluated at render time; `$if` is evaluated at resolve time (before catalog validation). A unified "remove element vs hide element" directive is a possible v12.1 simplification.

---

## 5. v1 → v2 surface migration table

Embedded from `V1-DELETION-AUDIT.md` (Plan 164-11). This is the table Phase 160 reads to confirm deletion can proceed.

| v1 surface | v2 equivalent | gestiscilo usage | Resolution | Notes |
|------------|---------------|------------------|------------|-------|
| `JsonUiView` | `Spec { schema_version, root, elements }` builder + `JsonUi::render_file` | Migrated in all controllers; codemod available | MIGRATED | Deleted in commit `dbe5adaf`. `view.rs` file is absent. |
| `Component` enum | `Element.type_name: String` + catalog dispatch | Every element since Phase 115 | MIGRATED | Type-erased dispatch via 41-entry built-in catalog; no `Component::` enum variants remain. |
| `ComponentNode` | `Element` in flat `Spec.elements` HashMap (children are ID refs) | All controllers using `render_file` or `Spec::builder` | MIGRATED | v2 is flat; nesting expressed by `children: Vec<String>` ID refs. |
| `PluginProps { plugin_type, props }` | First-class plugin type names (e.g. `"StripeConnectStatus"`) via `JsonUiPlugin`; one-off HTML islands via `RawHtml` (Phase 164 D-17a Plan 03) | gestiscilo settings pages used `"type": "Plugin"` — closed by D-17a `RawHtml` | MIGRATED via D-17a | Phase 115 D-01 killed the generic dispatch; D-17a (`RawHtml`) + registered plugin surface is the migration path. `PluginProps` struct is absent from source. |
| `CardProps.children` (typed nested) | `Element.children: Vec<String>` (ID refs into flat map) | All Card uses | MIGRATED | Same pattern for FormProps.fields, GridProps.children, CollapsibleProps.children, FormSectionProps.children, ButtonGroupProps.buttons — all use ID-ref children. |
| `FormProps.fields` | `Element.children: Vec<String>` (IDs of Form child elements) | All Form uses | MIGRATED | |
| `GridProps.children` | `Element.children: Vec<String>` | All Grid uses | MIGRATED | |
| `CollapsibleProps.children` | `Element.children: Vec<String>` | All Collapsible uses | MIGRATED | |
| `FormSectionProps.children` | `Element.children: Vec<String>` | All FormSection uses | MIGRATED | |
| `ButtonGroupProps.buttons` | `Element.children: Vec<String>` | All ButtonGroup uses | MIGRATED | |
| `SwitchProps.compact` | Re-added in Phase 162 D-16 (`compact: Option<bool>`) | 6 gestiscilo settings sites | MIGRATED | In `component.rs`. |
| `ImageProps::inline_svg` | Re-added in Phase 162 D-17 (`inline_svg: Option<String>`) | gestiscilo statistiche bar charts | MIGRATED | In `component.rs`. |
| `RichTextEditorProps` | Re-implemented as plugin element via Phase 162 D-18 (`RichTextEditorPlugin`); props: `field`, `label`, `default_value`, `data_path`, `error` | 2 gestiscilo documenti templates | MIGRATED | Plugin type `"RichTextEditor"` registered via `register_built_in_plugins`. |
| `DetailFormProps` / `DetailField` / `EditMode` | Documented v2 pattern (Phase 162 D-15) — `Form` element with `Input` children pre-populated via `data_path`, `visible` condition on `?mode=edit` | gestiscilo documenti edit flows | INTENTIONAL_DROP | Pattern documented in `docs/src/json-ui/components.md` (Inline view/edit section) and migration guide. No consumer blocked. |
| `make_node` / `make_node_with_action` builder helpers | `JsonUi::render_file` + JSON spec files; `Spec::builder()` for runtime-constructed specs; codemod for legacy controllers | Phase 138 controllers all migrated; codemod available for stragglers | INTENTIONAL_DROP | Consumer-side helpers; never part of ferro public API. Documented in migration guide. |
| `view.rs` / `JsonUiView::new` builder chain | `Spec::builder()` / `Spec::from_json` / `JsonUi::render_file` | All controllers | MIGRATED | File deleted in commit `dbe5adaf`. |
| `Spec.title` literal-only (`Option<String>`) | `Option<TitleBinding>` accepting literal or `{"$data": "/path"}` binding — Phase 164 D-12 Plan 04 | 23 gestiscilo specs unblocked (were forced to strip bindings via sed) | MIGRATED via D-12 | `TitleBinding` and `DataRef` re-exported from `ferro-json-ui`. Renderer resolves bindings at response-build time. |
| `KanbanBoard` static-columns-only | `KanbanBoardProps.data_path: Option<String>` runtime column resolution — Phase 164 D-13a Plan 06 | gestiscilo dashboard kanban views | MIGRATED via D-13a | `columns` is now `#[serde(default, skip_serializing_if)]`; `data_path` wins when both are set. |
| `MAX_NESTING_DEPTH = 3` depth ceiling | `MAX_NESTING_DEPTH = 5` — Phase 164 D-14 Plan 01 | gestiscilo dashboard pages with depth-4 structures (root → grid → card → badge) | MIGRATED via D-14 | Constant at `ferro-json-ui/src/spec.rs`. |
| `Image.src` static-only | `ImageProps.data_path: Option<String>` — Phase 164 D-15 Plan 03 | gestiscilo statistiche dynamic image src | MIGRATED via D-15 | `data_path` resolves against `spec.data` at render time; falls back to `src`. |
| `DescriptionList.items` static-only | `DescriptionListProps.data_path: Option<String>` — Phase 164 D-15 Plan 03 | gestiscilo statistiche dynamic description lists | MIGRATED via D-15 | `items` is now `#[serde(default)]`; `data_path` resolves array at render time. |
| Parse-time enum validation against raw spec (Alert.variant="" blocks startup) | Validation after `expand_directives` — Phase 164 D-16 Plan 07 | 2 gestiscilo pages with `$if`-gated bad-variant Alert elements | MIGRATED via D-16 | Load-time catalog validation downgraded to `tracing::warn`; per-request enforcement runs post-`expand_directives`. |
| Card chrome hard-coded for dashboard | `CardVariant::Bordered` (default) / `CardVariant::Elevated` — Phase 164 D-18 Plan 05 | gestiscilo auth/login + error pages | MIGRATED via D-18 | `CardProps.variant` field with `#[serde(default)]`; Elevated gives `shadow-md + p-8` without border. |
| `Visibility` enum parse error opaque | Hand-rolled `Deserialize` impl names all four accepted shapes — Phase 164 D-19/F5 Plan 08 | gestiscilo clienti/list + flotta/list debugging | MIGRATED via D-19/F5 | Error now includes offending JSON and all four accepted shapes. |
| `PageHeader.actions` rejects empty string | Lax deserializer accepting `null`, `""`, `[]`, `[string...]` — Phase 164 D-19/F6 Plan 08 | gestiscilo pages where controller passes `""` when no actions | MIGRATED via D-19/F6 | Rust type stays `Vec<String>`; laxness is scoped to this field only. |

### BLOCKER summary

**Total BLOCKER rows: 0**

**Phase 160 (v1 deletion) is UNBLOCKED.** Every v1 surface element is classified as either MIGRATED (v2 equivalent verified in source) or INTENTIONAL_DROP (gap documented above; no consumer blocked). The two INTENTIONAL_DROP rows (`DetailFormProps` and `make_node` helpers) have documented v2-native design patterns in the migration guide.

---

## Handoff

- **Phase 160** reads this document (specifically Section 5) and proceeds with v1 deletion.
- **Phase 161** uses Section 1 as the basis for the v12.0 CHANGELOG entry.
- **Future v12.x phases** consult Section 4 (Deferred) when scoping follow-on work.
