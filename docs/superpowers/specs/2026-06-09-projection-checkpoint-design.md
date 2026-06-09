# Projection Checkpoint — agent write→verify loop

**Date:** 2026-06-09
**Status:** Design approved, pending implementation plan
**Crate:** `ferro-mcp`
**Axis:** AX (agent experience)

## Problem

Ferro's introspection surface is rich on the *write* side: agents generate
artifacts through `generate_projection`, `json_ui_generate`, `code_templates`,
and `generate_types`. The *verify* side is fragmented across single-purpose
validators — `validate_projection`, `validate_contracts`,
`json_ui_validate_spec`, `json_ui_verify_action`, `render_projection`,
`diagnose_error`, `test_route`. An agent that has just authored a
projection-anchored slice must know which validators apply, call them in the
right order, and aggregate the results itself.

The failure mode this leaves open is silent cross-artifact incoherence: a
projection field references a model attribute the migration never created, a
projection action targets a route that is not registered, or a rendered view's
props do not match the frontend contract. Each individual artifact can be
locally valid while the slice is broken at the seams. The
`PageHeader.children` silent-drop (v12.0 friction F11) is the canonical example
of a seam failure that produces a 200 response with missing content and no
diagnostic.

## Goal

Provide a single, projection-anchored verification call that walks the slice
spine, dispatches to the existing validators at each seam, runs the one seam
no validator covers today, and returns a single structured verdict with ranked
next steps. Make this verification close by default after generation, and make
unverified or failing projections visible in project-level introspection.

The unit of verification is the **intent slice**, anchored on the
projection / `ServiceDef`, because the projection already ties model → intent →
derived view. This bounds the slice to one spine rather than an arbitrary set
of files.

## Non-goals

- Not a compiler. The checkpoint is read-only and introspective (source, route
  registry, DB schema). It does not invoke `cargo check`; compilation remains
  the agent's separate step. This keeps the loop fast and consistent with every
  other MCP tool.
- Not a generator. It reports what to fix; it does not mutate code.
- Model-anchored fan-out (checkpoint every projection/route/view touching a
  given model) is deferred. The anchor is the projection name only.
- No new validation logic except the field→column seam. Every other seam
  dispatches to an existing validator; findings carry the producing validator
  as provenance.

## Architecture

### Tool surface

A new `ferro-mcp` tool, `checkpoint_projection`:

```
checkpoint_projection { name: "Booking" }
```

It loads the named `ServiceDef` from `src/projections/`, walks the five seams
below, and returns the verdict described under *Output*.

### The spine walk (five seams)

A `ServiceDef` exposes `name`, `fields` (typed), `actions`, `guards`,
`relationships`, `intent_hints`, and `state_machine`. The checkpoint walks:

| # | Seam | Check | Source |
|---|------|-------|--------|
| 1 | projection well-formed | `ServiceDef::validate()` round-trip from source | reuse `validate_projection` |
| 2 | field → model column | each `FieldDef` resolves to a real entity/migration column | **new** (checkpoint-owned) |
| 3 | action → route | each `ActionDef` handler is a registered route | reuse `json_ui_verify_action` |
| 4 | projection → rendered view | render the projection, validate the resulting spec | reuse `render_projection` + `json_ui_validate_spec` |
| 5 | rendered props → frontend contract | Rust props ↔ TypeScript interface match | reuse `validate_contracts` |

Seam 2 is the primary new value: it is the silent gap nothing checks today. The
remaining seams are thin dispatches over existing validators, aggregated into
one verdict.

### Coverage honesty

Each seam reports its state distinctly — `pass`, `fail`, `warn`, or
`not_checked`. The checkpoint never collapses "not checked" into "pass". A seam
is `not_checked` when its prerequisite is absent (e.g. no rendered view exists
yet, so seams 4–5 cannot run). The verdict surfaces the unchecked seams
explicitly so the agent never reads more assurance into a `pass` than was
actually verified.

### Closing the loop by default

The standalone tool covers the edit case. To prevent the verification being
forgotten on the create path:

1. `generate_projection` and `json_ui_generate` return the checkpoint verdict
   inline in their response after generating, so the agent receives it whether
   or not it issues a separate call.
2. `application_info` and `projection_coverage` surface per-projection
   checkpoint status (`unverified` / `failing` / `clean`), so an agent
   surveying the project sees verification debt without probing for it.

## Output

```
{
  status: "pass" | "warn" | "fail",
  projection: "Booking",
  seams: [
    {
      seam: "field_to_column",
      status: "fail",
      source: "checkpoint",
      findings: [
        {
          subject: "starts_at",
          detail: "no column `starts_at` on entity `booking`",
          fix: "add column in migration, or remove field from projection"
        }
      ]
    },
    { seam: "action_to_route", status: "pass", source: "json_ui_verify_action", findings: [] },
    { seam: "rendered_view", status: "not_checked", source: "json_ui_validate_spec", findings: [] }
  ],
  next_steps: [
    "Add `starts_at` to booking migration (seam: field_to_column)"
  ]
}
```

- `status` is `fail` if any seam fails, `warn` if only warnings exist, `pass`
  if every checked seam passes. Unchecked seams do not raise `status` to `fail`
  but are listed.
- `next_steps` is the ranked, deduplicated, actionable list the agent acts on.
  Failures rank above warnings; within a rank, earlier seams first.
- Each seam finding names its producing validator in `source` for provenance.

## Components and boundaries

- **`checkpoint_projection` tool** (`ferro-mcp/src/tools/checkpoint_projection.rs`)
  — orchestrates the spine walk, owns seam 2, aggregates, ranks `next_steps`.
  Does not contain validation logic for seams 1, 3, 4, 5.
- **Field→column resolver** — the one new check. Resolves the projection to its
  source model using the same `src/projections/` ↔ `src/models/` mapping
  `projection_coverage` already performs, then compares the `ServiceDef` fields
  against that model's entity/migration columns and reports fields with no
  backing column. Depends on the model/schema introspection already used by
  `db_schema` and `explain_model`. When the source model cannot be resolved,
  seam 2 reports `not_checked` (never `pass`).
- **Inline hook** — `generate_projection` and `json_ui_generate` call the
  checkpoint after generating and embed the verdict. They depend on the tool;
  the tool does not depend on them.
- **Status surfacing** — `application_info` and `projection_coverage` add a
  per-projection checkpoint status field. Read-only consumers of the tool.

## Testing

- Per-seam unit tests with fixtures: a projection with a dangling field (seam 2
  fail), an action targeting an unregistered route (seam 3 fail), a projection
  with no rendered view (seams 4–5 `not_checked`), and a fully coherent slice
  (all `pass`).
- Aggregation tests: mixed seam results produce the correct overall `status`
  and correctly ranked, deduplicated `next_steps`.
- Coverage-honesty tests: an absent prerequisite yields `not_checked`, never
  `pass`.
- Dogfood acceptance: run the checkpoint across the synthetic app catalog and
  one live consumer application. Acceptance requires it to surface at least one
  real seam defect in a real project; a checkpoint that finds nothing real in
  real apps fails acceptance and the design is revisited rather than shipped.

## Implementation slice

First cut, in priority order:

1. The tool with seam 2 (field→column) and aggregation/`next_steps`.
2. The inline hook on `generate_projection` / `json_ui_generate`.
3. Status surfacing in `application_info` / `projection_coverage`.
4. The wrapper seams (1, 3, 4, 5), added as each earns its place against the
   dogfood results.

Seams that do not catch a real defect in dogfood acceptance may be deferred
without blocking the first release.
