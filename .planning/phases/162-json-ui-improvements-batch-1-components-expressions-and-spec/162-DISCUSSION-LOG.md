# Phase 162: JSON-UI improvements batch 1 — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-16
**Phase:** 162-json-ui-improvements-batch-1-components-expressions-and-spec
**Mode:** `--auto` (auto-selected from existing context + gestiscilo Phase 138 FRICTION.md)
**Areas discussed:** New components, DataTable interpolation, Container chrome, Spec validation, Handler discoverability, Variant type-safety, Blast-radius API surface, Migration documentation, Catalog/MCP surface, Version/release

---

## New components — homogeneous-options gap

| Option | Description | Selected |
|--------|-------------|----------|
| Unify `Checkbox` and `CheckboxList` under one component | Single component handles both single-item and data-driven multi-select cases | |
| Add `CheckboxList` as a separate first-class catalog entry | Two semantically different concerns kept independent | ✓ |
| Skip — let consumers compose with `Checkbox` + `$each` | Defer until iteration directives land in Phase 163 | |

**Selected:** Separate `CheckboxList` (D-01/D-02).
**Rationale:** Closes the onboarding step 2 (services list) friction immediately. Iteration directives are deferred to Phase 163; the data-driven multi-select gap needs a closed solution before then.

---

## DataTable per-row action interpolation

| Option | Description | Selected |
|--------|-------------|----------|
| Add only `{row_key}` placeholder | Minimum that closes the publish/delete/QR-download regression on `/dashboard/pagine` | |
| Generalize to any column key (`{label}`, `{slug_path}`, …) | Single substitution loop iterates over the row's columns | ✓ |
| Reject — require consumers to use detail pages for row-level actions | Per bidirectional adaptation, "navigate to detail page" is often the right v2 pattern | |

**Selected:** Generalized column-key placeholder grammar (D-03/D-04).
**Rationale:** Missing keys leave placeholders unsubstituted (no panic, no silent removal). Tests assert this. Generalization costs nothing extra over the targeted `{row_key}` fix.

**Note:** Planner MUST confirm with gestiscilo author before shipping that per-row actions are still the right pattern (vs detail-page navigation) — see Planning Note in CONTEXT.md.

---

## Container chrome — borderless composition

| Option | Description | Selected |
|--------|-------------|----------|
| Add `Fragment` / `Group` borderless container | New catalog entry for compositions that need structure without chrome | |
| Remove the layout-level card wrapper from `templates/auth.{html,hbs}` | Layout becomes structural only; each spec declares its own `Card` root | ✓ |
| Leave double-card and document as known limitation | Lowest-cost; ships nothing | |

**Selected:** Remove layout-level card (D-05). Reject new container (D-06).
**Rationale:** All auth-using pages already use `Card` roots — render identically after the change. A new borderless container would be parallel surface without a forcing use case.

---

## Spec validation — structural integrity

| Option | Description | Selected |
|--------|-------------|----------|
| Emit error for missing footer-referenced ID | Validator already exists; extend its checks | ✓ |
| Emit warning for duplicate footer/children listing | Dead config caught early | ✓ |
| Defer validation enhancements until v1.0 polish phase | Lower-priority papercut | |

**Selected:** Both validations land in Phase 162 (D-07/D-08).
**Rationale:** Both ride the existing `Spec::validate` path; near-zero implementation cost; closes silent-render papercuts surfaced in FRICTION.

---

## Handler-name discoverability

| Option | Description | Selected |
|--------|-------------|----------|
| Add `#[handler(name = "...")]` attribute | Second site for the same string — invites drift | |
| Add `json_ui_verify_action` MCP tool with Levenshtein suggestion | Reads existing single source of truth (route registry) | ✓ |
| Document the existing `list_routes` MCP tool as the verification path | Lowest-cost; no new tool | |

**Selected:** New MCP tool (D-09). Reject `#[handler(name)]` attribute (D-10).
**Rationale:** Closes a friction repeated in three FRICTION entries. The MCP tool is the agent-first discoverability surface; route names stay registered at `route!`/`get!`/`post!` macro sites.

---

## Variant type-safety

| Option | Description | Selected |
|--------|-------------|----------|
| Add `#[derive(strum::AsRefStr)]` to all variant enums | Typed enums at call sites; wire format unchanged | ✓ |
| Add full builder pattern for each variant struct | Heavier API surface | |
| Leave consumers hand-typing strings | Status quo | |

**Selected:** Strum derives on all six variant enums (D-11/D-12).
**Rationale:** Call-site ergonomics only; JSON wire format unchanged; v2 spec parsing already case-insensitive.

---

## Blast-radius API surface

For each removed-on-branch type, the question was: re-add as v2 element, replace with v2-native pattern, or document only.

| Type | Decision | D-XX |
|------|----------|------|
| `JsonUiView`, `Component`, `ComponentNode` | Document only — replaced by `JsonUi::render_file` | D-13 |
| `FormProps.fields`, `CardProps.children`, `GridProps.children`, `CollapsibleProps.children`, `FormSectionProps.children`, `ButtonGroupProps.buttons` | Document only — replaced by flat `elements` map + `children: Vec<String>` | D-14 |
| `DetailFormProps`, `DetailField`, `EditMode` | Document only — replace with `Form` + `DescriptionList` + `visible` on `?mode=edit` | D-15 |
| `SwitchProps.compact` | Re-add as CSS-class toggle | D-16 |
| `ImageProps::inline_svg` | Re-add `ImageSource::InlineSvg` variant | D-17 |
| `RichTextEditorProps`, `RichTextEditorPlugin` | Re-implement as v2 element + plugin (not catalog component) | D-18 |
| `PluginProps` | Document the v2 plugin authoring surface; no new code | D-19 |

**Selected:** Mix of "document only" (D-13/D-14/D-15/D-19) and "re-add" (D-16/D-17/D-18) based on whether v2-native pattern serves the use case or the original primitive is structurally needed.

---

## Migration documentation

| Option | Description | Selected |
|--------|-------------|----------|
| One focused `migration-v1-to-v2.md` page with worked examples | 300–500 lines; covers the seven friction-driven sections | ✓ |
| Inline migration notes per-component in `components.md` | Distributed; harder to find | |
| Skip dedicated migration doc — rely on CHANGELOG entries | Lowest-cost | |

**Selected:** Dedicated migration page (D-20).
**Rationale:** Consumer migrations in Phases 139–143 need a single navigable surface; CHANGELOG fragments are not discoverable.

---

## Catalog and MCP surface

| Option | Description | Selected |
|--------|-------------|----------|
| Update only `ferro-json-ui/src/catalog.rs` | Saves the ferro-mcp dual-update | |
| Dual-update `catalog.rs` AND `ferro-mcp/src/tools/json_ui_catalog.rs` | Existing pattern; the exhaustive-list assertion catches drift | ✓ |

**Selected:** Dual-update enforced for every catalog change (D-21). MCP `code_templates` surfaces migration patterns (D-22).
**Rationale:** Existing pattern preserves catalog↔MCP consistency by construction.

---

## Version and release

| Option | Description | Selected |
|--------|-------------|----------|
| Publish 0.2.36 after Phase 162 | Matches FRICTION.md "Suggested ferro improvement: publish 0.2.36" | |
| Stay on local-path patch through Phases 138–143 | Friction loop validates the API before crates.io freezes it | ✓ |

**Selected:** No publish in Phase 162 (D-23). FRICTION.md publish suggestion explicitly rejected (D-24). CHANGELOG accumulates Phase 162→163→164 entries; single publish at Phase 161 (D-25).
**Rationale:** Premature publish would freeze API decisions before Phase 163/164 friction findings can revise them. Local-path patch (`ferro = { path = "../ferro" }`) plus `[patch.crates-io]` ignores version constraints. See `project_friction_loop_release_cadence.md` memory.

---

## Claude's Discretion

Areas where the planner has flexibility (documented in CONTEXT.md `<decisions>` "Claude's discretion" subsection):

- Exact prop names within `CheckboxListProps` (`options` vs `items`; `selected_path` vs `default_value_path`) — match existing catalog convention.
- Whether `CheckboxList` shares `<datalist>` / suggested-keys infrastructure with the future `RichTextEditor` plugin — implementation detail.
- Whether D-15 (DetailForm replacement docs) lands before or after the consumer-side migration of `documenti` — docs can ship independently.
- Phase 162 does not own the publish gate — that is Phase 161's responsibility.

---

## Deferred Ideas

Carried forward to Phase 163 / Phase 164 / future phases:

- `$each` / `$if` / `$template` spec-level iteration directives — Phase 163.
- `SpecBuilder` ergonomic nested DSL — Phase 163.
- `ferro json-ui:migrate-v1` codemod — Phase 163.
- Multi-step form patterns, `visible` rule expressiveness at depth, PDF preview routing — Phase 164.
- Host-based tenancy gap — `.planning/backlog/host-based-tenancy.md`.
- `Fragment` / `Group` borderless container — only on forcing-use-case.
- `#[handler(name = "...")]` attribute — only on forcing-use-case.

---

*Generated 2026-05-16 in `--auto` mode. CONTEXT.md decisions D-01 through D-25 were scaffolded from `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md` and remain consistent with the current ferro-json-ui surface as of master merge into v12.0/json-ui-v2.*
