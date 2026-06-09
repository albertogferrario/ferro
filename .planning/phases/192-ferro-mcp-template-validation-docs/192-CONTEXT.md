# Phase 192: ferro-mcp Template + Validation Docs - Context

**Gathered:** 2026-06-09
**Status:** Ready for planning
**Mode:** focused inline capture (docs/template phase; targets pre-scouted)

<domain>
## Phase Boundary

Make the two-layer uniqueness pattern (proactive async `unique` rule + defensive
`ConstraintMap` at the write site) **discoverable** to an agent and a human, so
neither layer is used in isolation. No new runtime — this is the ferro-mcp
`action_handler` code template plus the validation docs page (VALID-06).

In scope (VALID-06):
- ferro-mcp `code_templates` `action_handler` template demonstrates BOTH layers
  together (proactive `AsyncValidator`+`unique`, defensive `ConstraintMap::try_map`
  at the insert/update site).
- `docs/src/features/validation.md` gains a dedicated async-rules section and a
  dedicated constraint-mapping section, cross-referenced.

Out of scope: any change to the validation runtime (Phases 190/191 shipped it).
</domain>

<decisions>
## Implementation Decisions

### ferro-mcp template (SC1)
- **D-01:** Enrich the existing `action_handler` template in
  `ferro-mcp/src/tools/code_templates.rs` (currently at ~line 291, category
  `handler`) so it shows a realistic create/update form flow with BOTH layers:
  build an `AsyncValidator` with `.async_rule(field, unique(table, col))` (with
  `.ignore(id)` on the edit path), then map a write-site `DbErr` through a
  `ConstraintMap` via `MapConstraintExt::map_constraint` (or `try_map`). The
  guiding constraint (SC1): **no generated handler template shows `unique`
  without a downstream `ConstraintMap`.**
- **D-02 (Claude's discretion):** Whether to (a) modify the single existing
  `action_handler` template in place, or (b) keep the generic one and add a
  focused `unique_form_handler` template, provided the SC1 invariant holds across
  the whole catalog (no `unique`-only template). Recommended: enrich the existing
  `action_handler` (matches ROADMAP SC1 wording literally — "the `action_handler`
  code template includes both"). Update its `description`, `code`, `imports`
  (add `AsyncValidator`, `unique`, `ConstraintMap`, `MapConstraintExt`), and
  placeholders (add `{{field}}`, `{{table}}`, `{{constraint_name}}`).
- **D-03:** The template code must compile-shape-match the real public API shipped
  in 190/191: `ferro_rs::{AsyncValidator, unique, ConstraintMap, MapConstraintExt,
  rules, required, string}`. Verify names against `framework/src/lib.rs` re-exports.

### validation docs (SC2, SC3)
- **D-04:** Add to `docs/src/features/validation.md` a dedicated **"Async Rules
  (DB-backed)"** section showing `unique(table, col)` with and without
  `.ignore(id)` exclude-self, via `AsyncValidator::new(&data).async_rule(...)
  .validate_async().await` returning `Result<(), AsyncValidationError>`.
- **D-05:** Add a dedicated **"Constraint Mapping"** section showing the
  `ConstraintMap` builder (`.on(constraint, field, message)` + `.sqlite("table.col")`),
  `try_map` / `map_constraint`, and the two-layer rationale: the proactive rule
  catches the UX case before the write; the defensive mapping closes the TOCTOU
  race at the write. Note the Postgres-vs-SQLite identity bifurcation briefly.
- **D-06:** The two sections are **cross-referenced** (each links to the other) so
  a reader of either discovers the other (SC3). Update the `## MCP Tools` section
  so it mentions the `handler` category template now demonstrates the two-layer
  pattern.

### Project-agnostic / accuracy
- **D-07:** Doc and template example strings (`pages`, `slug`, `pages_slug_unique`)
  are illustrative samples — the sanctioned exception to the project-agnostic rule
  (they live in docs/template text, not in a `ferro-*` crate's logic).

### Claude's Discretion
- Modify-in-place vs add-a-variant for the template (D-02).
- Exact docs section placement (under "Built-in Rules" vs new top-level sections)
  and heading wording.
- Whether `map_constraint` or raw `try_map` is the primary call shown (recommend
  `map_constraint` — it's the ergonomic the phase delivered).
</decisions>

<canonical_refs>
## Canonical References

### Phase contract
- `.planning/ROADMAP.md` § "Phase 192: ferro-mcp Template + Validation Docs" — 3 success criteria (template shows both layers; docs has both sections; cross-referenced).
- `.planning/REQUIREMENTS.md` — VALID-06.

### Edit targets
- `ferro-mcp/src/tools/code_templates.rs` — `action_handler` `CodeTemplate` at ~line 291 (category `handler`); `CodeTemplate`/`Placeholder` structs at top of file.
- `docs/src/features/validation.md` — 578 lines; headings: Basic Usage, Built-in Rules, Validation Examples, Custom Messages, ..., `## MCP Tools` (line 572). Add the two new sections and cross-links.

### The API to represent accurately (shipped 190/191)
- `framework/src/lib.rs` — crate-root re-exports: `AsyncValidator`, `AsyncValidationError`, `unique`, `ConstraintMap`, `MapConstraintExt` (confirm exact names/signatures).
- `framework/src/validation/rules_async.rs` (`unique`, `.ignore`/`.ignore_on`), `async_validator.rs` (`AsyncValidator`, `validate_async`), `constraint_map.rs` (`ConstraintMap`, `.on`, `.sqlite`, `try_map`, `MapConstraintExt::map_constraint`).
- `.planning/phases/190-async-rule-infrastructure-unique-rule/190-SUMMARY.md` and `.planning/phases/191-.../191-01-SUMMARY.md` — what shipped, for accurate example code.

No external specs.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- The `action_handler` template already uses `#[action]` + `ActionResult` redirect ergonomics — the two-layer example slots into that handler shape (validate, then insert with constraint mapping, then `Ok(())`).
- `docs/src/features/validation.md` already has a `## MCP Tools` → `code_templates` section to update, and a consistent heading style to mirror.

### Established Patterns
- ferro-mcp code templates are `CodeTemplate { name, category, description, code, imports, placeholders }` structs in a `vec!` — add/enrich one entry.
- Docs use fenced ```rust blocks; mirror the existing section structure (intro sentence → code block → notes).

### Integration Points
- `code_templates.rs` (the MCP tool output an agent reads) and `docs/src/features/validation.md` (the human/SUMMARY docs). Per CLAUDE.md, both ferro-mcp and docs are part of the framework surface held to the same bar.
</code_context>

<specifics>
## Specific Ideas
- This phase closes v12.4. After it lands, the milestone is feature-complete (190 proactive, 191 defensive, 192 surface) and can be marked shipped.
- The cross-reference is the point: an agent scaffolding a unique field via `code_templates` should never get the proactive rule without the defensive net, and a human reading either docs section should be pointed at the other.
</specifics>

<deferred>
## Deferred Ideas
- Foreign-key / check constraint docs — out of scope (v12.4 is UNIQUE-only).

### Reviewed Todos (not folded)
None.
</deferred>

---

*Phase: 192-ferro-mcp-template-validation-docs*
*Context gathered: 2026-06-09 (focused inline capture)*
