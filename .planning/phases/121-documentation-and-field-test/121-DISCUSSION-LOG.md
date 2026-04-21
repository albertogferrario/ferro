# Phase 121: Documentation & Field Test - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-21
**Phase:** 121-documentation-and-field-test
**Mode:** --auto (all areas auto-selected; recommended options chosen)
**Areas discussed:** Doc Rewrite Scope, Expression System Placement, Field Test Location, JSON Schema Doc Depth

---

## Doc Rewrite Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Rewrite all 6 docs + overview (7 files) | Complete v1→v2 rewrite across all JSON-UI pages including features/json-ui.md | ✓ |
| Rewrite only json-ui/ section (6 files) | Leave features/json-ui.md as-is | |
| Partial rewrite (getting-started + components only) | Only the most-used pages | |

**User's choice:** [auto] Rewrite all 7 pages (recommended default — all contain v1 API, partial rewrite leaves broken code in docs)
**Notes:** All 6 docs/src/json-ui/*.md pages + docs/src/features/json-ui.md contain JsonUiView, ComponentNode, Component:: references (Phase 115 deleted these types). Any partial rewrite leaves invalid code in docs.

---

## Expression System Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Standalone expressions.md page | New dedicated page; hard cap gets prominent section | ✓ |
| Integrate into data-binding.md | Extend existing page with $data/$template section | |

**User's choice:** [auto] Standalone expressions.md (recommended — new concept deserves its own page; hard cap rationale is substantive enough to warrant visibility)
**Notes:** Phase 118 added $data/$template as a new v2 primitive. Currently zero docs exist for this. The "hard cap" explanation (no $if/$for/$state) is an architectural statement that should be findable, not buried in a longer page.

---

## Field Test Location

| Option | Description | Selected |
|--------|-------------|----------|
| Add pagamenti demo to ferro app/ | Self-contained within ferro repo; realistic payments dashboard | ✓ |
| Convert gestiscilo-it page | Would require gestiscilo-it/app to have content | |

**User's choice:** [auto] Ferro app/ demo (recommended — gestiscilo-it/app is empty; adding to ferro app/ keeps phase self-contained)
**Notes:** ls ~/repositories/albertogferrario/gestiscilo-it/app/ returned empty. The ferro app/ sample app is the right location for framework-level demonstrations. The pagamenti demo uses Italian payment domain data (importo, data, stato, descrizione) to match gestiscilo's domain.

---

## JSON Schema Doc Depth

| Option | Description | Selected |
|--------|-------------|----------|
| All 3 use cases (IDE + AI + external validation) | Full json-schema.md page covering ferro json-ui:schema CLI, $schema field, AI structured output, jsonschema crate | ✓ |
| CLI-only documentation | Just documents the ferro json-ui:schema command | |

**User's choice:** [auto] All 3 use cases (recommended — success criterion 2 explicitly calls for "IDE validation, external tool integration, AI structured output")
**Notes:** Phase 120 added json_schema and component_schemas fields to the json_ui_catalog MCP tool. Phase 117 added ferro json-ui:schema CLI. Both need documentation. AI structured output via tool_use (Pass 2 generation) is a key use case for the schema.

---

## Claude's Discretion

- components.md: keep as single file (not split)
- pagamenti data shape: Italian payment records (importo, data, stato, descrizione)
- expressions.md: include `\{` escape syntax documentation
- json-schema.md: show trimmed example output from `ferro json-ui:schema`

## Deferred Ideas

- `ferro json-ui:validate` CLI command
- Per-component doc pages (splitting components.md)
- Migration guide (v1 → v2) — not needed pre-1.0
- Interactive schema explorer
