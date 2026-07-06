# Phase 164: JSON-UI improvements batch 3 — runtime frictions (F1–F10), v1-deletion-readiness audit, COMPLETED.md — Context

**Gathered:** 2026-05-16
**Re-scoped:** 2026-05-17 — absorbs `V7-RUNTIME-FRICTION.md` (post-patch runtime findings)
**Status:** Ready for planning
**Sources:**
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md` (2026-05-16) — compile-time friction accumulated across migration phases 138→143; most items absorbed by ferro 162/163/163.1.
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/V7-RUNTIME-FRICTION.md` (2026-05-17) — runtime friction discovered only after the patched ferro went active and the v7.0 dashboard actually rendered. Ten frictions F1–F10; F1/F2 already worked around on the gestiscilo side, F3/F4/F7/F8/F9/F10 require ferro changes, F5/F6 are gestiscilo-side fixes tracked here for cross-repo visibility.

## Phase Boundary

Phase 164 is the **closing batch** of the v12.0 friction loop. Four responsibilities:

1. **Residual compile-time items** from Phase 138 FRICTION.md not absorbed by Phase 162 or Phase 163. Mostly low-impact (validator polish, MCP tool improvements, documentation gaps).
2. **V7-RUNTIME ferro-side fixes** (F1, F3, F4, F7, F8, F9, F10). New material from 2026-05-17 runtime walkthrough. These are the dominant scope of the phase by code volume.
3. **v1-deletion-readiness audit.** Phase 160 (delete v1 JSON-UI API) gates on the v2 surface being complete enough to replace v1 in every legitimate use case. Phase 164 sweeps the codebase and verifies this — any "v2 can't express what v1 could" gap surfaces here and either gets fixed or gets documented as an intentional drop.
4. **COMPLETED.md.** Single document summarising every improvement shipped across Phases 162–164 and any intentional gaps retained for future milestones. Input to Phase 160's gate and to the v12.0 closing argument in Phase 161.

The original three-phase split (one ferro phase per gestiscilo migration phase) is dropped. The three ferro phases now slice the unified friction surface by concern.

## Planning Note — Bidirectional Adaptation

See Phase 162 CONTEXT for the full statement. Phase 164 is the **last** phase before v1 deletion — the bar for adding new catalog surface is highest here. Default to "no new component" unless the gap blocks a real consumer migration AND no v2-native redesign expresses the same intent.

The V7-RUNTIME items are mostly **enrichment of existing components** (add `variant` to Card, add `data_path` to Image/DescriptionList/KanbanBoard, raise depth limit) — not new components. F9 (Plugin component) is the only candidate for new surface; the planner decides whether to (a) reintroduce a `Plugin` element type, (b) document migration to v2 plugin-registered components, or (c) introduce a generic `RawHtml` / `Slot` primitive.

The deletion-readiness audit (responsibility 3) is itself a form of bidirectional adaptation: if a v1 capability has no v2 equivalent AND no redesigned UI path, the planner decides whether to (a) add the v2 capability, (b) document the intentional gap in COMPLETED.md, or (c) push back the deletion. Default: (b) for any capability that wasn't load-bearing in the gestiscilo migration.

## Slice from Phase 138 FRICTION.md

Residual items not in Phase 162 / 163:

- **"Low-Impact Entries"** section (lines 107–186): five items, three of which are partially addressed by Phase 162 D-09 (MCP tool) and D-11 (variant derives). Remaining: source-level tests pattern (lines 169–186) — purely consumer-side, no ferro change needed but worth documenting as a migration pattern.
- **"Codebase-Wide Blast Radius" item 4** (lines 252–254): `PluginProps` removal blocks custom embed widgets. Phase 162 D-19 documents the v2 plugin surface; Phase 164 audits whether the documentation is sufficient by attempting (on paper) to migrate Stripe and WhatsApp plugin sites.
- **"Codebase-Wide Blast Radius" item 5** (lines 256–257): `DetailFormProps` / `DetailField` / `EditMode` v2 replacement. Phase 162 D-15 specifies the replacement pattern; Phase 164 audits whether the documented pattern survives contact with the documenti edit flows (the largest consumer).
- **"Codebase-Wide Blast Radius" item 7** (lines 260–266): auth layout double-card. Phase 162 D-05 resolved by removing the layout-level card. **V7-RUNTIME F10 reopens this** — exposed Card has wrong styling for auth/marketing use; resolved by D-18 below (CardVariant enum).

## Slice from V7-RUNTIME-FRICTION.md

Ten frictions discovered on 2026-05-17 against patched ferro at 162/163.1. Page-by-page result: 5 fully clean, 1 partial, 7 hard-broken.

Already absorbed on the gestiscilo side (no ferro change required, but ferro could pre-empt):
- **F1** Spec.title rejects `$data` bindings — 23 specs stripped by sed.
- **F2** HTTP method values must be uppercase — 26 specs sed-uppercased; ferro codemod (Phase 163.x) should emit uppercase to prevent recurrence.

Ferro-side action items (D-12..D-18 below):
- **F3** `KanbanBoard` has no `data_path` prop — blocks dashboard kanban.
- **F4** Spec nesting depth limit of 3 — blocks `/dashboard/cassa/pagamenti` (depth 4: root → grid → card → badge).
- **F7** `Image.src` / `DescriptionList.items` reject `data_path` — blocks `/dashboard/analisi/statistiche`.
- **F8** `Alert.variant` empty string fails enum before visibility evaluation — blocks `/dashboard/documenti/modelli`, `/dashboard/pagine`. Architectural: parse-validate runs against raw spec, not against the post-`expand_directives` + post-visibility view.
- **F9** Unknown component type `Plugin` — blocks `/dashboard/settings`.
- **F10** Card chrome regression on auth/error/marketing pages — needs `CardVariant` enum (`Bordered` default, `Elevated`).

Gestiscilo-side fixes (tracked here for cross-repo visibility, not in ferro scope):
- **F5** `Visibility` enum parse fail in `clienti/list.json` / `flotta/list.json` — controller emits a Visibility shape the untagged enum doesn't accept. Ferro action: improve error message to name the bad variant (D-19).
- **F6** `PageHeader.actions` rejects empty string — controller passes `""` when no actions; should pass `[]`. Optional ferro action: accept `actions: None` (D-19).

## Implementation Decisions

### V7-RUNTIME ferro-side fixes

- **D-12 (F1) — Allow `$data` bindings on `Spec.title`.** Change `Spec.title: Option<String>` to accept either a literal `String` or an expression binding (`{"$data": "/path"}`). Implementation choice at planning time: introduce a `TitleBinding` enum mirroring how `Element.props` accept bindings, or generalise the binding resolution. Touches `ferro-json-ui/src/spec.rs` and the renderer's title emission path. Closes a 23-spec authoring constraint.

- **D-13 (F3) — Add `data_path` to `KanbanBoardProps`.** Currently `columns: Vec<KanbanColumnProps>` must be inlined statically. Two implementation options for the planner:
  - **D-13a:** Add `data_path: Option<String>` to `KanbanBoardProps` + a column factory pattern (`column_template` + grouping key) that emits columns from a JSON array at runtime.
  - **D-13b:** Document the existing `$each` directive (Phase 163) as the way to template kanban columns from a data path; add a worked example to `docs/src/json-ui/expressions.md`.
  Default: ship D-13a (`data_path`) as the primary path; document D-13b as the templated alternative. D-13a aligns with `DataTable`'s pattern and removes the need to nest a directive inside the spec for a common use case.

- **D-14 (F4) — Raise `MAX_NESTING_DEPTH` from 3 to 5.** Real-world dashboard hit depth 4 (root → grid → card → badge). The depth-3 constraint forced awkward flattening in three Phase 138 medium-impact entries already, and now blocks runtime rendering. Implementation: change `pub const MAX_NESTING_DEPTH: usize = 3` to `5` in `ferro-json-ui/src/spec.rs:37`; update the test at line 1705 and any callers; consider warn-only at depth 6 if a soft cap is desired (planner decides). Document the constraint in `docs/src/json-ui/spec-construction.md`.

- **D-15 (F7) — Add `data_path` to `ImageProps` and `DescriptionListProps`.** Both currently enforce static fields (`src` for Image, `items` for DescriptionList). Add `data_path: Option<String>` that, when present, resolves the dynamic value from request data and overrides the static field. Same shape as D-13. Touches `ferro-json-ui/src/component.rs` and the respective renderers. Resolves `/dashboard/analisi/statistiche` runtime block.

- **D-16 (F8) — Validate after `expand_directives` + visibility.** The current `Spec::validate` runs against the raw spec, so elements gated by `visible` still trip enum-variant validation (Alert.variant=`""` fails even though the alert is hidden). The architectural fix is to reorder the pipeline: `parse → expand_directives → apply_visibility → validate`. Two sub-decisions for the planner:
  - **D-16a:** Should validation run twice (structural at parse-time, semantic after resolution)? Or fully deferred?
  - **D-16b:** What is the migration path for existing consumers — does the change break any spec that depended on early validation as a contract? (Unlikely; we control all consumers pre-1.0.)
  Recommended: full deferral of enum-shape validation; structural validation (element references, footer IDs, depth) remains at parse-time. The friction file also notes that consumers can migrate from `visible` to `$if` (Phase 163) to side-step the issue — that's a complementary path but D-16 is the architectural fix.

- **D-17 (F9) — Resolve `Plugin` component type.** Audit current state:
  - The catalog's "Plugin components" section refers to plugin-registered component types (Phase 162 D-19), not a built-in `Component::Plugin` variant.
  - Gestiscilo's `settings` specs reference `"type": "Plugin"` directly, which fails because it's neither in the built-in catalog nor plugin-registered.
  Three implementation options:
  - **D-17a:** Reintroduce `Component::Plugin` (or `Component::RawHtml`) as a built-in catalog type — a server-injected HTML island carrying sanitised HTML in props.
  - **D-17b:** Document that gestiscilo's Stripe / WhatsApp settings widgets must be reimplemented as registered plugin components per Phase 162 D-19's guide, with named types (e.g. `StripeConnectStatus`, `WhatsAppLinkStatus`).
  - **D-17c:** Add a generic `Slot` component that defers rendering to a server-side template lookup keyed by `slot_id`.
  Default: **D-17a** (`Component::RawHtml`) — the lowest-friction path for consumers; the v1 Plugin component was load-bearing for this exact pattern. Document the v2 plugin-registered path (D-17b) as the recommended alternative for richer widgets.

- **D-18 (F10) — Add `CardVariant` enum to `CardProps`.** The friction file ships a fully-specified solution (V7-RUNTIME §F10 lines 109–138). Implement verbatim:
  - `CardVariant::Bordered` (default): `border border-border bg-card shadow-sm overflow-visible` + `p-4`. Current dashboard look.
  - `CardVariant::Elevated`: `rounded-lg bg-card shadow-md overflow-visible` + `p-8`. Auth pages, error pages, standalone marketing cards.
  - `#[serde(default)]` on the `variant` field; serde rename `lowercase`.
  Schema regen via Phase 117 catalog. Codemod tweak in Phase 163.x to either emit `"variant": "bordered"` explicitly or omit and rely on default. Open questions from the friction file (strict enum vs forward-compat string, Modal chrome sweep, separate `padding`/`elevation` props) resolved at planning time — recommendation in the friction file is: strict enum, no Modal change, no separate props.

- **D-19 — Cross-repo coordination for gestiscilo-side items (F5, F6) + F2.**
  - **F2 ferro codemod**: extend the existing `ferro json-ui:migrate-v1` codemod (Phase 163.x) to upper-case HTTP method values on emission. One-line fix; ship as part of D-19.
  - **F5 ferro error message**: improve the `Visibility` untagged-enum parse error to name the rejected variant shape (current: `data did not match any variant of untagged enum Visibility`; target: include the offending JSON shape and list accepted variants). Touches `ferro-json-ui/src/component.rs` Visibility deserializer. Small change, ship in this phase.
  - **F6 ferro `PageHeader.actions`**: consider accepting `actions: None` as an alternative to requiring `[]`. Lower priority; planner decides whether to ship or defer to a follow-up.

### v1-deletion-readiness audit

- **D-01:** Run a sweep over the v1 public surface (`view.rs`, `Component` enum, all `*Props` removed in v2, `ComponentNode`, `JsonUiView`, plugin v1 API) and produce a `V1-DELETION-AUDIT.md`. For each v1 surface element, note: (a) is there a v2 equivalent? (b) was it used in the gestiscilo migration? (c) is the gap intentional or unintentional? Output: a table with columns "v1 surface | v2 equivalent | gestiscilo usage | resolution."
- **D-02:** Resolutions are one of: `MIGRATED` (v2 has equivalent, all consumers ported), `INTENTIONAL_DROP` (no v2 equivalent, documented in COMPLETED.md as a future-milestone gap), `BLOCKER` (no v2 equivalent, real consumer needs it → Phase 164 ships the fix). If any row is `BLOCKER`, that fix lands in this phase.
- **D-03:** Phase 160 (v1 deletion) gates on zero `BLOCKER` rows in the audit. If Phase 164 cannot ship a fix for a `BLOCKER`, it must be reclassified as `INTENTIONAL_DROP` with explicit rationale, or Phase 160 is blocked until a follow-up phase.

### Spec validator polish

- **D-04:** Surface every validator error and warning produced by `Spec::validate` in `ferro-mcp`'s `json_ui_validate_spec` tool (or equivalent — name TBD per Phase 162). Agents authoring specs should get the same diagnostics from MCP that the runtime gives at startup.
- **D-05:** Add validator coverage for the directives introduced in Phase 163: `$each.path` resolves to a JSON array, `$if.path` resolves to a boolean (or coerces cleanly), no circular references in templated elements, no `children` references to absent elements unless the absent element has an `$if` (Phase 163 D-12 documented this).

### Plugin surface audit

- **D-06:** Walk through the v2 plugin author guide (Phase 162 D-19's `docs/src/json-ui/plugins.md`) and verify a fresh plugin author could implement: (a) a Stripe payment widget, (b) a WhatsApp connection flow, (c) a chart renderer. Run this as a paper exercise — do NOT actually implement plugins. If the docs are insufficient for any of these three cases, file the gap and fix it in this phase. **Related: D-17 may reintroduce `Component::RawHtml` as a simpler escape hatch for HTML-island cases.**
- **D-07:** If the audit reveals a load-bearing missing primitive (e.g., asset-injection ordering, plugin-to-plugin communication), this becomes a `BLOCKER` row in D-01's audit table and ships in Phase 164.

### Documentation pass

- **D-08:** Final sweep over the v2 documentation set: `docs/src/json-ui/components.md`, `migration-v1-to-v2.md` (Phase 162 D-20), `expressions.md` (Phase 163 directive sections), `spec-construction.md` (Phase 163 D-08), `plugins.md` (Phase 162 D-19). Cross-link missing references. Confirm every component in `ferro-json-ui/src/catalog.rs` has a matching doc section. **Add doc sections for D-12 (Spec.title binding), D-13 (KanbanBoard.data_path), D-14 (new depth limit), D-15 (Image/DescriptionList.data_path), D-16 (validation pipeline order), D-17 (Plugin/RawHtml resolution), D-18 (CardVariant).**
- **D-09:** Add a "v1 → v2 cheat sheet" at the top of `migration-v1-to-v2.md` with a 10-row table of the most common v1 patterns and their v2 equivalents. Drawn from the gestiscilo migration's actual code rewrites.

### COMPLETED.md

- **D-10:** Produce `.planning/phases/164-.../COMPLETED.md` with sections:
  - **Shipped across Phases 162-164** — every D-* decision from all three phases that landed, organized by category (components, directives, ergonomics, docs, validator, MCP).
  - **Runtime frictions resolved** — F1–F10 from V7-RUNTIME-FRICTION.md with resolution status (ferro fix shipped / gestiscilo workaround / deferred).
  - **Intentional gaps** — what we explicitly chose not to add and why (e.g., `Fragment` container — Phase 162 D-06 rejected; `$template` separate element — Phase 163 D-05 rejected).
  - **Deferred to future milestones** — items surfaced in friction but pushed past v12.0 (e.g., host-based tenancy → its own backlog; codemod directory mode → v12.2; advanced expression operators if any).
  - **v1 → v2 surface migration table** — the audit from D-01, with all rows resolved.
- **D-11:** COMPLETED.md is the input to Phase 160 (v1 deletion). Phase 160's planner reads it to confirm the deletion can proceed. COMPLETED.md is also the basis of the CHANGELOG entry written at Phase 161 (publish).

### Claude's discretion

- Exact column structure of D-01's audit table — planner picks.
- Whether D-04's MCP surface is a new tool or an extension of an existing tool — implementation choice.
- Whether D-06 audit produces a written artefact or is a verbal checkpoint with the user — depends on how many gaps it surfaces.
- D-13 implementation choice (data_path vs $each example) — planner picks; default is "ship both".
- D-16 sub-decisions (validation pipeline split, breaking-change ledger) — planner picks; default is "full deferral with structural validation retained at parse-time".
- D-17 implementation choice (RawHtml vs plugin-only vs Slot) — planner picks; default is D-17a (RawHtml) + document D-17b.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Friction sources
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md` — compile-time friction. Phase 164 plans MUST cite specific FRICTION lines for the residual items, not summary text.
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/V7-RUNTIME-FRICTION.md` — runtime friction (F1–F10). Plans implementing D-12..D-19 MUST cite the F-number and the friction file's recommended action, and note any divergence with rationale.

### v1 surface to audit (D-01 input)
- `ferro-json-ui/src/view.rs` — `JsonUiView`, `SCHEMA_VERSION = "ferro-json-ui/v1"`. Deletion target for Phase 160.
- `ferro-json-ui/src/component.rs` (historical reference via git log) — `Component` enum and v1 `*Props` structs that were removed in v2.
- `framework/src/lib.rs` — `pub use ferro_json_ui::` block. Anything still re-exported from v1 path is a deletion candidate.

### v2 surface to validate against (D-01 input) and to extend (D-12..D-18)
- `ferro-json-ui/src/spec.rs` — `Spec`, `Element`, `SpecBuilder`, validation. `MAX_NESTING_DEPTH` at line 37 (D-14 target). `Spec.title` field (D-12 target).
- `ferro-json-ui/src/component.rs` — props structs. `CardProps` at line 153 (D-18 target); `ImageProps`, `DescriptionListProps`, `KanbanBoardProps` (D-13/D-15 targets); `Visibility` enum (D-19 F5 target).
- `ferro-json-ui/src/catalog.rs` — catalog v2 component set. Plugin component map (D-17 reference).
- `ferro-json-ui/src/plugin.rs` — v2 plugin trait + asset system. D-06/D-17 reference.
- `ferro-json-ui/src/render/containers.rs:54` — `render_card` (D-18 implementation site).

### Documentation set (D-08 sweep)
- `docs/src/json-ui/components.md`, `migration-v1-to-v2.md`, `expressions.md`, `spec-construction.md`, `plugins.md`, `data-binding.md`.

### Phase artefacts
- `.planning/phases/162-.../162-CONTEXT.md` — Phase 162 decisions D-01..D-25.
- `.planning/phases/163-.../163-CONTEXT.md` — Phase 163 decisions D-01..D-13.
- `.planning/phases/163.1-.../` — codemod multi-root fix; D-19 F2 extends this codemod.

## Predecessor and successor

- Phases 162 and 163 land first. Phase 164 audits their combined output and absorbs runtime frictions surfaced after their patches were live.
- Phase 164 produces COMPLETED.md which gates Phase 160 (v1 deletion).
- Phase 160's deletion lands the v12.0 close; Phase 161 merges and publishes.

## Release cadence

Same as Phase 162 D-23/D-24/D-25 — no mid-loop publish. Phase 164 closes its CHANGELOG entries into the accumulated v12.0 release notes; Phase 161 emits the single publish. Per `feedback_friction_loop_release_cadence`: a mid-loop publish would freeze the API before D-12..D-19 land and force gestiscilo onto a stale surface.
