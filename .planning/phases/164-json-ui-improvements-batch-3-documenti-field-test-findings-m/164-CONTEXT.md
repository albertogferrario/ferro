# Phase 164: JSON-UI improvements batch 3 — closing cleanup, validator polish, COMPLETED.md — Context

**Gathered:** 2026-05-16
**Status:** Ready for planning
**Source:** `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md` — residual items not absorbed by Phase 162 or Phase 163, plus the v1-deletion-readiness audit.

## Phase Boundary

Phase 164 is the **closing batch** of the v12.0 friction loop. Three responsibilities:

1. **Residual items** from FRICTION.md that don't fit cleanly in Phase 162 (components/API) or Phase 163 (iteration directives/ergonomics). Mostly low-impact: validator polish, MCP tool improvements, documentation gaps.
2. **v1-deletion-readiness audit.** Phase 160 (delete v1 JSON-UI API) gates on the v2 surface being complete enough to replace v1 in every legitimate use case. Phase 164 sweeps the codebase and verifies this — any "v2 can't express what v1 could" gap surfaces here and either gets fixed or gets documented as an intentional drop.
3. **COMPLETED.md.** Single document summarising every improvement shipped across Phases 162–164 and any intentional gaps retained for future milestones. Input to Phase 160's gate and to the v12.0 closing argument in Phase 161.

The original three-phase split (one ferro phase per gestiscilo migration phase) is dropped. The three ferro phases now slice the unified Phase 138 FRICTION.md by concern.

## Planning Note — Bidirectional Adaptation

See Phase 162 CONTEXT for the full statement. Phase 164 is the **last** phase before v1 deletion — the bar for adding new catalog surface is highest here. Default to "no new component" unless the gap blocks a real consumer migration AND no v2-native redesign expresses the same intent.

The deletion-readiness audit (responsibility 2) is itself a form of bidirectional adaptation: if a v1 capability has no v2 equivalent AND no redesigned UI path, the planner decides whether to (a) add the v2 capability, (b) document the intentional gap in COMPLETED.md, or (c) push back the deletion. Default: (b) for any capability that wasn't load-bearing in the gestiscilo migration.

## Slice from FRICTION.md

Residual items not in Phase 162 / 163:

- **"Low-Impact Entries"** section (lines 107–186): five items, three of which are partially addressed by Phase 162 D-09 (MCP tool) and D-11 (variant derives). Remaining: source-level tests pattern (lines 169–186) — purely consumer-side, no ferro change needed but worth documenting as a migration pattern.
- **"Codebase-Wide Blast Radius" item 4** (lines 252–254): `PluginProps` removal blocks custom embed widgets. Phase 162 D-19 documents the v2 plugin surface; Phase 164 audits whether the documentation is sufficient by attempting (on paper) to migrate Stripe and WhatsApp plugin sites.
- **"Codebase-Wide Blast Radius" item 5** (lines 256–257): `DetailFormProps` / `DetailField` / `EditMode` v2 replacement. Phase 162 D-15 specifies the replacement pattern; Phase 164 audits whether the documented pattern survives contact with the documenti edit flows (the largest consumer).
- **"Codebase-Wide Blast Radius" item 7** (lines 260–266): auth layout double-card. Phase 162 D-05 resolves by removing the layout-level card. Phase 164 audits whether any other layout has the same issue (other dashboard layouts, modal layouts).

## Implementation Decisions

### v1-deletion-readiness audit

- **D-01:** Run a sweep over the v1 public surface (`view.rs`, `Component` enum, all `*Props` removed in v2, `ComponentNode`, `JsonUiView`, plugin v1 API) and produce a `V1-DELETION-AUDIT.md`. For each v1 surface element, note: (a) is there a v2 equivalent? (b) was it used in the gestiscilo migration? (c) is the gap intentional or unintentional? Output: a table with columns "v1 surface | v2 equivalent | gestiscilo usage | resolution."
- **D-02:** Resolutions are one of: `MIGRATED` (v2 has equivalent, all consumers ported), `INTENTIONAL_DROP` (no v2 equivalent, documented in COMPLETED.md as a future-milestone gap), `BLOCKER` (no v2 equivalent, real consumer needs it → Phase 164 ships the fix). If any row is `BLOCKER`, that fix lands in this phase.
- **D-03:** Phase 160 (v1 deletion) gates on zero `BLOCKER` rows in the audit. If Phase 164 cannot ship a fix for a `BLOCKER`, it must be reclassified as `INTENTIONAL_DROP` with explicit rationale, or Phase 160 is blocked until a follow-up phase.

### Spec validator polish

- **D-04:** Surface every validator error and warning produced by `Spec::validate` in `ferro-mcp`'s `json_ui_validate_spec` tool (or equivalent — name TBD per Phase 162). Agents authoring specs should get the same diagnostics from MCP that the runtime gives at startup.
- **D-05:** Add validator coverage for the directives introduced in Phase 163: `$each.path` resolves to a JSON array, `$if.path` resolves to a boolean (or coerces cleanly), no circular references in templated elements, no `children` references to absent elements unless the absent element has an `$if` (Phase 163 D-12 documented this).

### Plugin surface audit

- **D-06:** Walk through the v2 plugin author guide (Phase 162 D-19's `docs/src/json-ui/plugins.md`) and verify a fresh plugin author could implement: (a) a Stripe payment widget, (b) a WhatsApp connection flow, (c) a chart renderer. Run this as a paper exercise — do NOT actually implement plugins. If the docs are insufficient for any of these three cases, file the gap and fix it in this phase.
- **D-07:** If the audit reveals a load-bearing missing primitive (e.g., asset-injection ordering, plugin-to-plugin communication), this becomes a `BLOCKER` row in D-01's audit table and ships in Phase 164.

### Documentation pass

- **D-08:** Final sweep over the v2 documentation set: `docs/src/json-ui/components.md`, `migration-v1-to-v2.md` (Phase 162 D-20), `expressions.md` (Phase 163 directive sections), `spec-construction.md` (Phase 163 D-08), `plugins.md` (Phase 162 D-19). Cross-link missing references. Confirm every component in `ferro-json-ui/src/catalog.rs` has a matching doc section.
- **D-09:** Add a "v1 → v2 cheat sheet" at the top of `migration-v1-to-v2.md` with a 10-row table of the most common v1 patterns and their v2 equivalents. Drawn from the gestiscilo migration's actual code rewrites.

### COMPLETED.md

- **D-10:** Produce `.planning/phases/164-.../COMPLETED.md` with sections:
  - **Shipped across Phases 162-164** — every D-* decision from all three phases that landed, organized by category (components, directives, ergonomics, docs, validator, MCP).
  - **Intentional gaps** — what we explicitly chose not to add and why (e.g., `Fragment` container — Phase 162 D-06 rejected; `$template` separate element — Phase 163 D-05 rejected).
  - **Deferred to future milestones** — items surfaced in friction but pushed past v12.0 (e.g., host-based tenancy → its own backlog; codemod directory mode → v12.2; advanced expression operators if any).
  - **v1 → v2 surface migration table** — the audit from D-01, with all rows resolved.
- **D-11:** COMPLETED.md is the input to Phase 160 (v1 deletion). Phase 160's planner reads it to confirm the deletion can proceed. COMPLETED.md is also the basis of the CHANGELOG entry written at Phase 161 (publish).

### Claude's discretion

- Exact column structure of D-01's audit table — planner picks.
- Whether D-04's MCP surface is a new tool or an extension of an existing tool — implementation choice.
- Whether D-06 audit produces a written artefact or is a verbal checkpoint with the user — depends on how many gaps it surfaces.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Friction source
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md` — residual entries cited above by line range. Phase 164 plans MUST cite specific FRICTION lines, not summary text.

### v1 surface to audit (D-01 input)
- `ferro-json-ui/src/view.rs` — `JsonUiView`, `SCHEMA_VERSION = "ferro-json-ui/v1"`. Deletion target for Phase 160.
- `ferro-json-ui/src/component.rs` (historical reference via git log) — `Component` enum and v1 `*Props` structs that were removed in v2.
- `framework/src/lib.rs` — `pub use ferro_json_ui::` block. Anything still re-exported from v1 path is a deletion candidate.

### v2 surface to validate against (D-01 input)
- `ferro-json-ui/src/spec.rs` — `Spec`, `Element`, `SpecBuilder`, validation.
- `ferro-json-ui/src/catalog.rs` — the catalog v2 component set.
- `ferro-json-ui/src/plugin.rs` — v2 plugin trait + asset system.

### Documentation set (D-08 sweep)
- `docs/src/json-ui/components.md`, `migration-v1-to-v2.md`, `expressions.md`, `spec-construction.md`, `plugins.md`, `data-binding.md`.

### Phase artefacts
- `.planning/phases/162-.../162-CONTEXT.md` — Phase 162 decisions D-01..D-25.
- `.planning/phases/163-.../163-CONTEXT.md` — Phase 163 decisions D-01..D-13.

## Predecessor and successor

- Phases 162 and 163 land first. Phase 164 audits their combined output.
- Phase 164 produces COMPLETED.md which gates Phase 160 (v1 deletion).
- Phase 160's deletion lands the v12.0 close; Phase 161 merges and publishes.

## Release cadence

Same as Phase 162 D-23/D-24/D-25 — no mid-loop publish. Phase 164 closes its CHANGELOG entries into the accumulated v12.0 release notes; Phase 161 emits the single publish.
