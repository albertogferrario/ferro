# Phase 64: Ferro Projections [FERRO REPO] - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-05
**Phase:** 64-ferro-projections-ferro-repo
**Areas discussed:** Template context structure, Field-to-key mapping, Action representation, State machine exposure
**Mode:** Auto (all decisions auto-selected from recommended defaults)

---

## Template Context Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Flat key-value | All fields at top level (product_name, product_price, business_name) | |
| Grouped by semantic category | Nested by domain entity (business.name, products[].price) | ✓ |
| MiniJinja-native context | Typed Rust struct implementing minijinja::value::Object | |

**User's choice:** [auto] Grouped by semantic category
**Notes:** More natural for template authors. Consistent with how MiniJinja templates reference nested data.

---

## Field-to-Key Mapping

| Option | Description | Selected |
|--------|-------------|----------|
| Field names as-is | Use ServiceDef field names directly in template context | ✓ |
| FieldMeaning-based canonical keys | Map Money→"price", EntityName→"name" regardless of field name | |

**User's choice:** [auto] Field names as-is
**Notes:** Predictable for template authors who know the data model. FieldMeaning drives inclusion, not renaming.

---

## Action Representation

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal strings | Just action name strings for data-action attributes | |
| Rich objects | Full objects with name, display_name, inputs, preconditions | ✓ |

**User's choice:** [auto] Rich objects
**Notes:** Phase 66 runtime needs metadata for button labels and input forms. Minimal strings would require a second lookup.

---

## State Machine Exposure

| Option | Description | Selected |
|--------|-------------|----------|
| Current state only | Just the current state name and display name | |
| All states as typed enums | Full set of states with display names and transitions | ✓ |
| Full graph definition | Complete StateMachine with all validation details | |

**User's choice:** [auto] All states as typed enums
**Notes:** PROJ-04 requires "state machine states as typed enums". Templates need the full set for status badges and kanban columns.

---

## Claude's Discretion

- Internal module organization within ferro-projections/src/render/
- Helper function decomposition
- Test structure and fixture design
- Whether to add a TemplateContext wrapper struct or return raw serde_json::Value

## Deferred Ideas

- "Add granular module selection to onboarding step 2" — reviewed, not folded (unrelated to ferro framework work)
