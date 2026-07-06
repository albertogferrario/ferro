# Phase 162: JSON-UI improvements batch 1 — components, expressions, and spec ergonomics — Context

**Gathered:** 2026-05-16
**Status:** Ready for planning
**Source:** `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md`

## Planning Note — Bidirectional Adaptation

The friction loop is **not** "make v2 capable of expressing every v1 UI verbatim." It is a two-way conversation: ferro evolves to express what gestiscilo needs, AND gestiscilo's UI is allowed to be redesigned to fit naturally into the v2 model.

Before adding ferro complexity to satisfy a friction entry, the planner MUST ask:

1. **Is the v1 UI pattern actually right for the user?** Sometimes the friction surfaces that v1's design was already wrong — depth-4 nesting, dropdown dumps, modal-in-modal, etc. v2's structural constraints (depth-3 limit, flat element map, declarative iteration) are not arbitrary — they encode opinions about what makes a clean UI. If a friction entry is "v2 can't express my depth-5 form," the answer is often "your form shouldn't be depth-5."

2. **Is there a v2-native pattern that delivers the same user value?** Per-row action dropdowns on list pages are a common v1 pattern; v2's natural equivalent is often "navigate to detail page, take actions there" — which most users actually prefer. Before adding `{row_key}` interpolation in DataTable (D-03), the planner should explicitly ask whether the gestiscilo redesign is "move per-row actions to detail pages." If yes, the friction is solved by gestiscilo, not ferro.

3. **Does adding the ferro feature serve more than one consumer's idiosyncrasy?** Single-consumer needs go in the consumer's `Spec::builder()` escape hatch. Cross-consumer patterns become first-class catalog components or directives.

Phase 162 decisions below are biased toward the **smallest ferro change that lets gestiscilo express the right UI**, not the largest ferro change that lets gestiscilo port v1 verbatim. Where a decision could be solved by gestiscilo redesign, that is the preferred resolution.

<domain>
## Phase Boundary

Phase 162 absorbs the friction surfaced by gestiscilo Phase 138 (v1→v2 migration of `auth.rs`, `account.rs`, `onboarding.rs`, `pages.rs`). It is the **first** of three batched improvement phases consuming gestiscilo field-test friction files. Items in this phase are the component, expression, validation, and API-surface changes whose justification comes from those four migrated controllers and from the blast-radius analysis activated when `[patch.crates-io]` started compiling the not-yet-migrated codebase against v2.

Out of scope (deferred to later phases):

- Phase 163 (gestiscilo Phase 140, cassa/calendario): `$each` / `$if` / `$template` spec-level iteration directives, `SpecBuilder` ergonomic nested DSL, and the `ferro json-ui:migrate-v1` codemod. Their justification comes from heterogeneous-iteration sites in `cassa/orders.rs` and `cassa/products.rs` and will arrive in Phase 140's friction file.
- Phase 164 (gestiscilo Phase 142, documenti): multi-step form patterns, `visible` rule expressiveness at depth, PDF preview routing. Documenti's friction file is the canonical input.
- Host-based tenancy proposal: `.planning/backlog/host-based-tenancy.md`. Not consumed by Phase 162 because it is a tenancy-layer concern, not a JSON-UI concern. The `PreRouteMiddleware.rewrite` → `handle` rename surfaced in the blast radius is one trivial line in `src/middleware/host.rs` on the gestiscilo side; the underlying tenancy gap is for a dedicated phase.

In scope: every "Suggested ferro improvement" in FRICTION.md whose payload is in `ferro-json-ui`, `framework/src/json_ui/`, `ferro-mcp`, or the v2 documentation set, restricted to the four migrated controllers and the API-surface decisions needed to unblock the remaining migration phases. The exact ordering is locked in `<decisions>` below.

</domain>

<decisions>
## Implementation Decisions

### New components — homogeneous-options gap

- **D-01:** Add a `CheckboxList` first-class component to `ferro-json-ui` with props: `field: String` (shared form field name; each selected checkbox submits as `field=value`), `options: Vec<SelectOption>` OR `options: { $data: "/path" }` (data-driven array), `selected_path: Option<String>` (data path to a `Vec<String>` of pre-selected values), `label: Option<String>`, `description: Option<String>`, `disabled: Option<bool>`, `error: Option<String>`. Renderer emits one `<input type="checkbox" name="{field}" value="{option.value}">` per option with the standard form-field chrome. This closes the data-driven multi-select gap (onboarding step 2 — services list).
- **D-02:** `CheckboxList` lands as a new catalog entry. Existing `Checkbox` is unchanged (single-item primitive). The two are not unified — single-checkbox and multi-checkbox-from-data are semantically different.

### DataTable per-row action interpolation

- **D-03:** Extend `DataTableProps.row_actions[i].action.url` to support `{row_key}` placeholder interpolation, using the same substitution logic already applied to `row_href`. The substitution happens at render time, per row. Without this, `row_actions` is unusable for any per-row navigation (closing publish/delete/QR-download regression on `/dashboard/pagine`).
- **D-04:** Generalize the placeholder grammar to support any column key bound at render time (`{label}`, `{slug_path}`, `{status}`, …), not only `{row_key}`. The renderer iterates over the row's columns and substitutes by name. Missing keys leave the placeholder text unsubstituted (no panic, no silent removal); the test suite asserts this.

### Container chrome — borderless composition

- **D-05:** The auth layout (`templates/auth.{html,hbs}` or equivalent) wraps its content in a card today. When a spec's root is also `Card`, the page renders a double-card. Phase 162 resolves this by **removing** the layout-level card wrapper. The auth layout becomes structural only (centering + max-width). Each spec is responsible for declaring its own `Card` root if it wants card chrome. This is a breaking change to the auth layout, but auth-using pages all use `Card` roots and will render identically after the change.
- **D-06:** Do NOT introduce a new `Fragment` / `Group` borderless container. Reason: the underlying problem in FRICTION (double-card on auth layout) is solved by D-05; a new borderless container would be a parallel solution adding catalog surface without a forcing use case. If a future phase finds a use case that D-05 + existing containers (Grid 1-col, FormSection without title) cannot express, revisit then.

### Spec validation — structural integrity

- **D-07:** Spec validator (the existing `Spec::from_json` / `Spec::validate` path) MUST emit an error when a footer-referenced element ID is missing from the `elements` map. Today the spec silently renders without the missing footer element; the consumer has no signal.
- **D-08:** Spec validator MUST emit a warning when the same element ID appears in both `props.footer` and `children` of the same parent. The element renders once (in `props.footer`); the duplicate listing is dead config and should be caught early.

### Handler-name discoverability

- **D-09:** Add a `json_ui_verify_action` MCP tool to `ferro-mcp` accepting `{ handler: String, method: Option<String> }` and returning `Ok(RouteInfo)` if a route is registered under that name + method, `Err(NotFound)` with the closest-by-Levenshtein candidate name otherwise. Closes the "I had to read routes.rs to verify the handler name" friction repeated in three FRICTION entries.
- **D-10:** Do NOT add a `#[handler(name = "...")]` attribute. Reason: route names are already registered at `route!`/`get!`/`post!` macro call sites via `.name("…")`; adding a second site for the same string would invite drift. The MCP tool reads the existing single source of truth.

### Variant type-safety

- **D-11:** Add `#[derive(strum::AsRefStr)]` (or equivalent serde-compatible derive) to `AlertVariant`, `BadgeVariant`, `ButtonVariant`, `ToastVariant`, `DialogVariant`, `NotifyVariant`, so consumers can pass typed enum values into a `to_string()` site instead of hand-typing lowercase strings. The JSON wire format is unchanged — these are call-site ergonomics only.
- **D-12:** Spec parsing on the v2 side already accepts the variant strings case-insensitively; this decision does not change the spec wire format.

### Blast-radius API surface (decisions, not full implementations)

These items came from compiling gestiscilo against the patched v2 ferro and are scoped to **resolving the API decision** — not necessarily shipping the full feature in Phase 162. Each gets a documented v2 path so consumers migrating in Phases 139–143 know what to write.

- **D-13:** `JsonUiView`, `Component`, `ComponentNode`. **Decision:** These are removed in v2 (already done on the branch). The migration pattern is `JsonUi::render_file("src/views/.../*.json", data)` returning a `Response`. Documentation: add a top-of-page migration banner to `docs/src/json-ui/components.md` linking to the `pagamenti.json` reference. No code change.
- **D-14:** `FormProps.fields`, `CardProps.children`, `GridProps.children`, `CollapsibleProps.children`, `FormSectionProps.children`, `ButtonGroupProps.buttons`. **Decision:** All removed in v2 (already done). The migration pattern is "container element's `children: Vec<String>` holds IDs into the flat `Spec.elements` map." Documentation: add a worked example to `docs/src/json-ui/components.md` showing `Card` with `children: ["heading", "form_login"]` and the corresponding `elements` entries. No code change.
- **D-15:** `DetailFormProps`, `DetailField`, `EditMode`. **Decision:** Do NOT re-add as v2 component. The v2 equivalent is a standard `Form` element whose `children` include `DescriptionList`-style read-only items in view mode and `Input` elements in edit mode, with `visible` conditions branching on a `?mode=edit` query parameter. Add a worked example in `docs/src/json-ui/components.md` under a new "Inline view/edit" section. No new catalog component.
- **D-16:** `SwitchProps.compact`. **Decision:** Re-add the `compact: Option<bool>` field to `SwitchProps` — it is a pure CSS-class toggle (`scale-75`), trivially re-implementable, used in 6 settings.rs sites. No spec-format break.
- **D-17:** `ImageProps::inline_svg`. **Decision:** Re-add the `ImageSource::InlineSvg { svg: String }` enum variant. Phase 148 added it on master; the branch's v2 cleanup removed it. The use case (server-constructed bar charts) is legitimate and the safety story is unchanged (verbatim emission, alt text required, server-only). Restore the variant + the `ImageProps::inline_svg(svg, alt)` factory + the safety rustdoc.
- **D-18:** `RichTextEditorProps`, `RichTextEditorPlugin`. **Decision:** Re-implement as a v2 element type, NOT as a top-level `Component` variant. Add `RichTextEditorProps` as a leaf element, the Quill 2.0.3 plugin registration (asset injection only), and the runtime IIFE. Use the existing v2 plugin surface, not v1's `Component::RichTextEditor`. Two consumer sites in documenti templates wait on this.
- **D-19:** `PluginProps`. **Decision:** Document the v2 plugin authoring surface in `docs/src/json-ui/plugins.md`. The branch already has the `JsonUiPlugin` trait + `register_plugin` + `Asset` system; what's missing is the consumer-facing doc page that explains how a custom Stripe / WhatsApp widget defines itself, including how its props flow through `Element.props`. No new code in `ferro-json-ui` if the existing surface is sufficient; the gate is documentation, not implementation.

### Migration documentation

- **D-20:** Add `docs/src/json-ui/migration-v1-to-v2.md` — a focused migration guide for app authors moving controllers off the v1 builder API. Sections: (a) `JsonUi::render_file` vs `Spec::builder()`, (b) `Card + Form + Alert` depth-flattening pattern (the account.rs case), (c) per-row action interpolation in DataTable, (d) the read+edit detail pattern (D-15 worked example), (e) data-driven options with `CheckboxList` (D-01 worked example), (f) variant string round-trip with the new derives (D-11), (g) handler-name verification with the new MCP tool (D-09). Length target: 300–500 lines of focused worked examples, not exhaustive component reference.

### Catalog and MCP surface

- **D-21:** Every new or changed catalog entry (`CheckboxList`, `DataTable.row_actions` placeholder grammar, `Image` SVG variant, `Switch.compact`, `RichTextEditor`) MUST update both `ferro-json-ui/src/catalog.rs` (the in-process catalog) and `ferro-mcp/src/tools/json_ui_catalog.rs` (the MCP-exposed catalog). The exhaustive-list assertion in ferro-mcp tests is bumped to match the new component count.
- **D-22:** The MCP `code_templates` tool MUST surface the v1→v2 migration patterns as code-snippet templates (one per D-20 section) so agents authoring migrations have direct introspection access.

### Version and release

- **D-23:** Phase 162 does NOT publish to crates.io and does NOT bump the workspace version. Gestiscilo consumes ferro via local-path patch (`ferro = { path = "../ferro" }`) for the entire Phases 138–143 migration sequence. Publishing prematurely would freeze the v2 API surface before the friction loop has validated it — every API decision in this CONTEXT may be revised by Phase 163 or Phase 164 friction findings. The single publish for v12.0 happens AFTER gestiscilo's full migration is complete, AFTER Phase 160 (v1 deletion), and AS PART OF Phase 161 (merge v12.0/json-ui-v2 → master and publish the resulting crate set). Until then, the workspace version remains 0.2.35.
- **D-24:** The FRICTION.md "Suggested ferro improvement: publish 0.2.36" is **wrong** about the publish cadence and MUST NOT drive a Phase 162 action. The friction file's authors assumed the unblock path was "ferro publishes a version with `render_file`," but the correct path is "gestiscilo uses local-path patch." If gestiscilo's `Cargo.toml` has `ferro = ">=0.2.36"` it should be relaxed to `>=0.2.35` (or the path-based patch in `[patch.crates-io]`, which ignores the version constraint, used as-is). This is a one-line consumer-side fix, not a ferro publish.
- **D-25:** CHANGELOG.md entries for every Phase 162 decision land at the time of implementation, not at the time of publish. The CHANGELOG accumulates Phase 162→163→164 entries; the publish at Phase 161 emits the combined entry.

### Claude's discretion

- Exact prop names within `CheckboxListProps` (`options` vs `items`; `selected_path` vs `default_value_path`) — pick to match existing convention in catalog.
- Whether `CheckboxList` shares the `<datalist>` / suggested-keys infrastructure with the future `RichTextEditor` plugin — implementation detail.
- Whether to land D-15 (DetailForm replacement docs) before or after the consumer-side migration of documenti — the docs can ship independently; the consumer migration is on gestiscilo's side.
- Phase 162 does not own the publish gate — that is Phase 161's responsibility. The planner does not need to allocate a release plan within Phase 162.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Friction source
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md` — every decision above traces to a specific entry in this file. Plans MUST cite the exact entry header when justifying a change.

### v2 catalog and component surface (the place new components register)
- `ferro-json-ui/src/catalog.rs` — built-in component registry. Adding `CheckboxList` adds an entry here.
- `ferro-json-ui/src/component.rs` — all `*Props` structs and serde wire format. `SwitchProps`, `ImageProps`, new `CheckboxListProps` live here.
- `ferro-json-ui/src/render.rs` — per-component render functions. `render_data_table` is the site for D-03/D-04 placeholder interpolation. `render_image` is the site for D-17 SVG branch.
- `ferro-json-ui/src/spec.rs` — `Spec`, `Element`, validation logic. D-07 and D-08 land here.
- `ferro-json-ui/src/plugin.rs` — `JsonUiPlugin` trait, `register_plugin`, asset system. D-18 (RichTextEditor) re-uses this. D-19 documents this.

### ferro-mcp surface
- `ferro-mcp/src/tools/json_ui_catalog.rs` — MCP-exposed catalog. Exhaustive-list assertion must be bumped on every component addition (D-21).
- `ferro-mcp/src/tools/code_templates.rs` (or equivalent) — D-22 surfaces migration templates here.
- `ferro-mcp/src/tools/` — D-09 lands a new `json_ui_verify_action.rs` here.

### Documentation set
- `docs/src/json-ui/components.md` — every component prop change updates this. D-14 worked example. D-15 inline view/edit section.
- `docs/src/json-ui/migration-v1-to-v2.md` — D-20 new file.
- `docs/src/json-ui/plugins.md` — D-19 plugin author guide.
- `docs/src/SUMMARY.md` — nav entries for new pages.

### Sample reference (the working v2 model)
- `app/static/pagamenti.json` and `app/src/controllers/pagamenti.rs` — Phase 121 field test. The canonical "what a correct v2 page looks like" reference. The migration guide should cite this.

### Workspace publishing pipeline
- `.github/workflows/publish.yml` — Wave layout. New components in `ferro-json-ui` ride the existing wave; no new crates.
- `CHANGELOG.md` — D-24 entry lands here.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-json-ui/src/component.rs` already hosts `SwitchProps` (line 349), `DataTableProps` (line 741 with `row_actions` and `row_href` fields), `AlertVariant`, `BadgeVariant`, `ButtonVariant`, `ToastVariant` enums — D-11 (strum derive), D-16 (Switch.compact), D-17 (Image SVG), D-18 (RichTextEditor) re-add to existing structs/enums rather than scaffolding new files.
- `ferro-json-ui/src/render/data.rs` already implements `row_href` substitution (`row_actions` rendering at lines 86, 199, 233). D-03/D-04 placeholder interpolation reuses the existing substitution path — no new render function.
- `ferro-json-ui/src/spec.rs` `Spec::from_json` / validation path is the existing target for D-07 (missing footer ID error) and D-08 (duplicate footer/children warning). No new validator module.
- `ferro-json-ui/src/plugin.rs` provides the v2 plugin authoring surface (`JsonUiPlugin` trait, `register_plugin`, `Asset` system). D-18 (RichTextEditor) is a plugin consumer, not a new framework primitive. D-19 documents this surface as-is.
- `ferro-mcp/src/tools/json_ui_catalog.rs`, `json_ui_inspect.rs`, `code_templates.rs` are the existing MCP surfaces D-21 / D-22 extend; the exhaustive-list assertion in tests is the bump site for D-21.
- `app/static/pagamenti.json` + `app/src/controllers/pagamenti.rs` (Phase 121) is the canonical v2 reference cited throughout D-20 migration docs.

### Established Patterns
- **Catalog + MCP dual update** — every catalog change touches `ferro-json-ui/src/catalog.rs` AND `ferro-mcp/src/tools/json_ui_catalog.rs`; the exhaustive-list assertion in ferro-mcp tests catches drift. D-21 codifies this for batch 1.
- **Variant string round-trip case-insensitive on parse, lowercase on emit** — `AlertVariant`/`BadgeVariant`/`ButtonVariant`/`ToastVariant` already accept case-insensitive input (D-12); the strum derive in D-11 is purely call-site ergonomics on the consumer side.
- **Render-time placeholder substitution** — `row_href` already substitutes `{row_key}` at render time; D-03/D-04 generalize to all column keys without changing the wire format.
- **Plugin asset injection over framework component** — `RichTextEditor` (D-18) follows the existing plugin surface (`JsonUiPlugin` + `Asset` system), not a new `Component::RichTextEditor` variant. Keeps the catalog clean.
- **Spec validation emits structured errors via `Spec::validate`** — D-07 and D-08 reuse the existing validator path. No parallel validation pipeline.

### Integration Points
- New `CheckboxList` (D-01/D-02) registers in `ferro-json-ui/src/catalog.rs` + ships a render function alongside existing `Checkbox` (single-item primitive) — the two are NOT unified.
- New `json_ui_verify_action` MCP tool (D-09) lands as a new file in `ferro-mcp/src/tools/` and registers in the tool dispatcher. Reads route names from the existing route registry (no new source of truth — D-10 explicitly rejects `#[handler(name = "...")]`).
- New `docs/src/json-ui/migration-v1-to-v2.md` (D-20) registers in `docs/src/SUMMARY.md`; consumer-facing docs only, no code.
- Auth layout `templates/auth.{html,hbs}` (D-05) is the breaking-change site; remove the card wrapper, leaving structural centering + max-width only. All auth-using specs already declare `Card` roots.

</code_context>

<specifics>
## Specific Ideas

- The four migrated controllers in gestiscilo Phase 138 — `auth.rs`, `account.rs`, `onboarding.rs`, `pages.rs` — are the canonical sites the decisions were calibrated against. Every D-XX in this CONTEXT traces to a concrete FRICTION.md entry from those four migrations.
- The canonical "what a correct v2 page looks like" reference is `app/static/pagamenti.json` + `app/src/controllers/pagamenti.rs` (Phase 121 field test). The migration guide (D-20) cites this throughout as the worked example.
- The Planning Note above (bidirectional adaptation) governs every decision: before adding ferro complexity to satisfy a friction entry, ask first whether the v1 UI pattern was already wrong and whether the v2-native pattern delivers the same user value via gestiscilo redesign. D-03/D-04 (DataTable per-row interpolation) is the canonical example — the planner must confirm with the gestiscilo author that per-row actions on list pages are not better solved by "navigate to detail page" before shipping the interpolation feature.
- Single-consumer needs go in the consumer's `Spec::builder()` escape hatch. Cross-consumer patterns become first-class catalog components or directives. This heuristic kept `Fragment`/`Group` containers OUT of the catalog (D-06) and kept `$each`/`$if`/`$template` deferred to Phase 163 where heterogeneous-iteration patterns will provide the forcing function.

</specifics>

<deferred>
## Deferred Ideas

Phase 162 deliberately stays narrow — only items justified by the four Phase 138 controllers AND the blast-radius API decisions. Items observed but explicitly deferred:

- `$each` / `$if` / `$template` spec-level iteration directives — deferred to Phase 163 (gestiscilo Phase 140 cassa, where heterogeneous iteration provides the forcing function).
- `SpecBuilder` ergonomic nested DSL — deferred to Phase 163.
- `ferro json-ui:migrate-v1` codemod — deferred to Phase 163.
- Multi-step form patterns, `visible` rule expressiveness at depth, PDF preview routing — deferred to Phase 164 (gestiscilo Phase 142 documenti).
- Host-based tenancy gap — out of JSON-UI scope; tracked in `.planning/backlog/host-based-tenancy.md` for a dedicated tenancy-layer phase. The `PreRouteMiddleware.rewrite` → `handle` rename surfaced in the blast radius is a one-line gestiscilo-side fix and does not bring the tenancy work forward.
- `Fragment` / `Group` borderless container — D-06 explicitly rejects this for Phase 162. Revisit only if a future phase finds a use case that D-05 + existing containers (Grid 1-col, FormSection without title) cannot express.
- `#[handler(name = "...")]` attribute — D-10 explicitly rejects this. Revisit only if a future use case justifies a second source of truth for route names.

Detailed next-phase inputs are tracked in `<followups>` below.

</deferred>

<followups>
## Follow-ups (next-phase inputs)

When gestiscilo Phase 140 (cassa/calendario) and Phase 142 (documenti) produce their FRICTION.md files, the following items are expected to land in Phase 163 / Phase 164:

- `$each` directive — spec-level iteration over a data array for homogeneous element shapes (closes 3 of 4 cassa heterogeneous sites). FRICTION.md "Extended Iteration Gap" suggested improvement #1.
- `$if` directive — conditional element emission (closes orders detail action case). FRICTION.md suggested improvement #2.
- `$template` element with auto-suffixed IDs — closes products detail edit-mode case. FRICTION.md suggested improvement #3.
- `SpecBuilder` ergonomic nested DSL — reduces Rust-side spec-construction friction where heterogeneous iteration cannot be expressed declaratively.
- `ferro json-ui:migrate-v1` codemod — auto-rewrites `make_node(id, Component::X(props))` call trees into stub JSON spec entries.

The Phase 162 planner is NOT responsible for these items. They are recorded here so the Phase 163/164 planners can connect their CONTEXT to the running thread.

Host-based tenancy gap (separate concern, not JSON-UI): see `.planning/backlog/host-based-tenancy.md`.

</followups>
