# V1-Deletion Readiness Audit (Phase 164 D-01..D-03)

**Audited:** 2026-05-17
**Status:** draft (pending user review — see Sign-off section)
**Input to:** Phase 160 (v1 JSON-UI API deletion), Phase 161 (v12.0 merge + publish)

## Purpose

Phase 160 deletes the v1 JSON-UI API. Phase 164 ensures that deletion is safe by walking every v1 public surface element and confirming a v2 equivalent exists, OR documenting the gap as intentional, OR escalating to a BLOCKER (which must be closed before Phase 160 can run).

## Methodology

1. Enumerate v1 surface elements: types previously re-exported from `ferro-json-ui` and `framework` crates; consult Phase 115's deletion commit (`dbe5adaf`) and Phase 162 CONTEXT for the full list.
2. For each element: check v2 equivalent (via ferro-json-ui catalog, plugin guide, or docs), gestiscilo migration status (per Phase 138 + V7-RUNTIME-FRICTION.md), and Resolution.
3. Resolution column values:
   - **MIGRATED** — v2 has a direct equivalent; all consumers ported successfully.
   - **INTENTIONAL_DROP** — no v2 equivalent; documented in COMPLETED.md as a future-milestone gap; no current consumer blocked.
   - **BLOCKER** — no v2 equivalent; real consumer still needs it; Phase 164 ships a fix.

## Grep Evidence (v1 Surface Absence)

All commands run from worktree root on 2026-05-17.

| Check | Command | Result |
|-------|---------|--------|
| `JsonUiView`, `ComponentNode`, `PluginProps` in production source | `grep -rE '\b(JsonUiView\|ComponentNode\|PluginProps)\b' ferro-json-ui/src framework/src` | **2 matches — both `///` doc-comments** in `containers.rs` and `form.rs` (historical migration notes; not live code). Zero production code matches. |
| `Component::` enum-variant patterns | `grep -rE 'Component::(Card\|Plugin\|Text\|Form\|Button\|Grid)' ferro-json-ui/src framework/src` | **0 matches** |
| `view.rs` file existence | `ls ferro-json-ui/src/view.rs` | **No such file or directory** — deleted in commit `dbe5adaf` |
| `framework/src/lib.rs` v1 re-exports | `grep -E 'JsonUiView\|ComponentNode\|PluginProps\|Component::' framework/src/lib.rs` | **0 matches** |
| `ferro-json-ui/src/lib.rs` v1 re-exports | `grep -E 'JsonUiView\|ComponentNode\|PluginProps' ferro-json-ui/src/lib.rs` | **0 matches** |

**Interpretation:** All v1 type names are absent from production source. The two doc-comment references are historical notes (`/// Note: v1 iterated props.buttons: Vec<ComponentNode>`) that are present to assist future readers; they do not constitute live API surface.

## Surface Audit

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

## BLOCKER Summary

Total BLOCKER rows: **0**

Phase 160 (v1 deletion) is **UNBLOCKED**.

Every v1 surface element is classified as either MIGRATED (v2 equivalent verified in source) or INTENTIONAL_DROP (gap documented in COMPLETED.md; no consumer blocked). The two INTENTIONAL_DROP rows (`DetailFormProps` and `make_node` helpers) were never part of ferro's public API in the case of `make_node`, and have a documented v2-native design pattern in the case of `DetailFormProps`.

## Plugin Surface Audit Cross-Reference

See `PLUGIN-SURFACE-AUDIT.md` for the D-06 paper-exercise outcome.

**Spoiler:** Plan 10 conducted the D-06 audit during its plugins.md pass. Two minor gaps were found and fixed inline (undocumented `data` parameter in `render()`; undocumented `init_script()` per-page-once semantics). **Outcome B — minor gaps, both fixed. No escalation to BLOCKER.**

## Spot-Check Commands

For each MIGRATED row, the following grep commands confirm the v2 equivalent:

```bash
# RawHtmlProps (closes PluginProps gap)
grep -c "pub struct RawHtmlProps" ferro-json-ui/src/component.rs
# → 1

# CardVariant (closes auth-chrome gap)
grep -c "pub enum CardVariant" ferro-json-ui/src/component.rs
# → 1

# TitleBinding (closes Spec.title literal-only gap)
grep -c "pub enum TitleBinding" ferro-json-ui/src/spec.rs
# → 1

# MAX_NESTING_DEPTH = 5 (was 3)
grep "pub const MAX_NESTING_DEPTH" ferro-json-ui/src/spec.rs
# → pub const MAX_NESTING_DEPTH: usize = 5;

# ImageProps.data_path
grep -A 3 "pub struct ImageProps" ferro-json-ui/src/component.rs | grep data_path
# → (data_path field present, seen in struct body below src/alt/aspect_ratio)

# BUILTIN_TYPES count (41 including RawHtml)
grep "assert_eq!(BUILTIN_TYPES.len()" ferro-json-ui/src/render/mod.rs
# → assert_eq!(BUILTIN_TYPES.len(), 41);
```

## Sign-off

- [ ] Audit reviewed by user
- [ ] BLOCKER count confirmed = 0
- [ ] INTENTIONAL_DROP rows (DetailFormProps/DetailField/EditMode; make_node helpers) accepted as intentional gaps for v12.0
- [ ] Phase 160 (v1 deletion) cleared to proceed
- [ ] Phase 161 (v12.0 merge + publish) cleared to proceed after Phase 160
