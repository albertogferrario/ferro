# checkpoint_projection

`checkpoint_projection` is an MCP tool that runs a structured verification pass on a service projection and returns a single verdict. It is the primary tool for catching projection–model coherence issues before they surface at runtime.

## When to Use

Call `checkpoint_projection` after generating or editing a projection — specifically when a projection field might reference a model attribute that no migration created. The tool catches this class of issue statically, before a running application exposes it as a runtime error.

```json
{
  "name": "checkpoint_projection",
  "arguments": {
    "name": "booking_service"
  }
}
```

`name` is the projection function name as defined in `src/projections/` (e.g. `"booking_service"`). It is resolved by function name; a service/struct name such as `"Booking"` will not resolve.

## Verdict Shape

The tool returns a single `Verdict` object:

```json
{
  "status": "fail",
  "projection": "booking_service",
  "seams": [
    {
      "seam": "field_to_column",
      "status": "fail",
      "source": "checkpoint",
      "findings": [
        {
          "subject": "phantom_col",
          "detail": "no column `phantom_col` on entity `booking`",
          "fix": "add column `phantom_col` to `booking` migration, or remove the field from the projection"
        }
      ]
    },
    {
      "seam": "projection_well_formed",
      "status": "not_checked",
      "source": "validate_projection",
      "reason": "unproven_against_real_inputs"
    }
  ],
  "next_steps": [
    "add column `phantom_col` to `booking` migration, or remove the field from the projection (seam: field_to_column)"
  ]
}
```

### Top-level fields

| Field | Type | Description |
|-------|------|-------------|
| `status` | `SeamStatus` | Aggregate status across all seams. |
| `projection` | string | The projection name as supplied to the tool. |
| `seams` | `SeamResult[]` | One entry per seam, regardless of outcome. |
| `next_steps` | string[] | Ranked, deduplicated actionable strings, capped at 5. |

### SeamStatus values

| Value | Meaning |
|-------|---------|
| `pass` | Seam checked and no issues found. |
| `warn` | Seam checked; findings present but not blocking. |
| `fail` | Seam checked; one or more blocking findings present. |
| `not_checked` | Prerequisite absent — seam was not run. Never coerced to `pass`. |

### SeamResult fields

| Field | Type | Description |
|-------|------|-------------|
| `seam` | string | Seam identifier (e.g. `"field_to_column"`). |
| `status` | `SeamStatus` | Outcome for this seam. |
| `source` | string | Provenance tag naming the validator that produced the result. Seam 2 (`field_to_column`) uses `"checkpoint"`; seams 1, 3, 4, and 5 name their delegating validator (e.g. `"validate_projection"`, `"json_ui_verify_action"`). |
| `findings` | `Finding[]` | Populated when `status` is `fail` or `warn`. |
| `reason` | string? | Populated for `not_checked` or `warn` outcomes; describes why. |

### Finding fields

| Field | Type | Description |
|-------|------|-------------|
| `subject` | string | The field, entity, or structural element the finding concerns. |
| `detail` | string | Human-readable description of the problem. |
| `fix` | string | Concrete remediation step an agent can act on without a second call. |

## The field→column Seam

The `field_to_column` seam is the primary check delivered in this version. It verifies that every field in the projection has a corresponding column on the source model entity.

**What it checks:** each `FieldDef` in the reconstructed `ServiceDef` is present as a column name in the `list_models` output for the matching entity.

**What it does not check:** column type compatibility — this seam checks column presence only. Type mismatches are a separate concern deferred to a later phase.

**Model resolution:** the seam matches the projection's `service_name` (case-insensitive snake_case) against the entity struct name from SeaORM. If no entity matches, the seam returns `not_checked` with `reason: "source_model_unresolved"`.

**Relationship exemption:** `.has_many`, `.belongs_to`, and similar relationship builders populate `ServiceDef.relationships`, not `ServiceDef.fields`. They are never subject to the column-presence check.

## Coverage Honesty: not_checked

A `not_checked` seam means the prerequisite for running the check was absent — the model could not be resolved, the source could not be parsed, or the seam is not yet implemented. It does not mean the projection is clean.

`not_checked` never contributes to the aggregate `status` rising to `fail`. However, it is always listed in `seams[]` so the caller can see which checks did not run and why.

Do not treat a `not_checked` seam as equivalent to `pass`. A verdict of `pass` means all runnable seams passed. A verdict with `not_checked` seams means some checks were skipped.

### Seam coverage by default

Not all seams run on every call. The table below documents which seams are active and which report `not_checked` by default.

| Seam | Default outcome | Rationale |
|------|----------------|-----------|
| `field_to_column` | Active | Proven by an acceptance fixture (dangling field planted, exactly one finding). |
| `action_to_route` | Active | Proven against the in-repo sample application (4 unregistered actions detected). |
| `projection_well_formed` | Active | Runs via `validate_projection`; findings reported on name-collision path. |
| `rendered_view` | Active | Runs via `render_projection`; findings reported on name-collision path. |
| `props_to_contract` | `not_checked` by default | Produced zero findings across both dogfood inputs (Phase 196 acceptance run). Reported as `not_checked` with `reason: "unproven_against_real_inputs"` rather than a vacuous pass. |

A `not_checked` seam does not mean the projection is clean — it means the seam has not been exercised against real inputs that expose defects. When `props_to_contract` is `not_checked`, the aggregate status is determined by the other seams.

## Aggregate Status Logic

- `fail` if any seam is `fail`
- `warn` if no seam is `fail` but at least one is `warn`
- `pass` otherwise — including when all seams are `not_checked`

`not_checked` seams never raise the aggregate to `fail`.

## next_steps

The `next_steps` list is assembled from all `fail` and `warn` findings:

- Failures appear before warnings.
- Within a rank, seam order (seam 1 through 5) is preserved.
- Duplicate `(subject, fix)` pairs are deduplicated across seams.
- The list is capped at 5 entries.
- Each entry has the format: `"<fix> (seam: <seam_name>)"`.

## Status Cache

Every successful run writes `.ferro/checkpoints/{name}.json` with the full verdict plus two derived fields:

| Field | Description |
|-------|-------------|
| `ambient_status` | `"clean"` if `status == pass`, otherwise `"failing"`. |
| `checked_at` | ISO 8601 UTC timestamp of the run. |

This cache is read by other tools to surface ambient projection health without re-running the full check.

## Read-Only Contract

`checkpoint_projection` reads source files, the route registry, and the DB schema. It does not:

- invoke `cargo` or any compiler
- write or modify source files
- mutate database state

It is safe to call at any point in a development workflow, including in CI.

## Related Tools

| Tool | When to use |
|------|-------------|
| `validate_projection` | Structural validation — unreachable states, unused guards. |
| `projection_coverage` | Coverage gaps — which models have projections and which do not. |
| `inspect_projection` | Inspect the raw `ServiceDef` structure of a projection. |
| `render_projection` | Preview the JSON-UI output a projection produces. |
