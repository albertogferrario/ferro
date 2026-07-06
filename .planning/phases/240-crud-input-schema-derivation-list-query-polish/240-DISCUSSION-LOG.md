# Phase 240: CRUD input-schema derivation + `list_` query polish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-23
**Phase:** 240-crud-input-schema-derivation-list-query-polish
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen)
**Areas discussed:** Phase scope split, create/update field-set derivation, delete schema, list_ range filters, sort, dispatch execution, testing

---

## Phase scope split (schema-emission vs execution)

| Option | Description | Selected |
|--------|-------------|----------|
| Emit write tools + schemas now, defer execution to 241 | create/update/delete appear in tools/list with correct schemas; calling them is wired in Phase 241 | ✓ |
| Build schema + execution together | Fold derive_crud_plan/kernel wiring into this phase | |

**Choice:** Emit schemas + list tools now; defer execution. **Rationale:** matches spec
"Within-Track sequencing" items 3 (schema derivation) vs 4 (`derive_crud_plan`). `list_`
polish lands fully (schema + dispatch) because `list_` already executes.

## create/update field-set derivation

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse `is_server_injected_field` + exclude Sensitive/list/UpdatedAt | Compose 239 boundary; one shared predicate for create & update | ✓ |
| New standalone exclusion logic | Re-derive exclusions from FieldMeaning directly | |

**Choice:** Reuse the 239 boundary, share one predicate across create+update. Status
excluded when an SM exists, ordinary writable field when no SM. UpdatedAt also excluded.

## delete_<svc> schema

| Option | Description | Selected |
|--------|-------------|----------|
| identifier (required) + confirmation token, destructiveHint=true | Schema shape only; mechanism/execution in 241/242 | ✓ |
| Omit delete entirely until 241 | Defer all delete surface | |

**Choice:** Emit the delete schema shape so the derived surface is complete; mechanism +
soft-delete execution deferred. (CRUD-03 is formally a 241 requirement; only schema shape
is in 240 scope.)

## list_ range/comparison filters

| Option | Description | Selected |
|--------|-------------|----------|
| Flat sibling keys `<field>__op`, new `is_range_filter_field` for ordered ops | `__ne`/`__in` for all filterable; `__gt/gte/lt/lte` for numeric+date | ✓ |
| Mutate `is_filter_field` to add numeric meanings globally | Single allowlist for equality + range | |

**Choice:** Flat `<field>__op` params; dedicated range-eligibility predicate so equality
schemas stay byte-for-byte back-compatible. `__in` typed as array.

## sort

| Option | Description | Selected |
|--------|-------------|----------|
| Single `sort` string (`field`/`-field`), allowlisted, Identifier tiebreaker kept | Single key; stable offset pagination preserved | ✓ |
| Multi-key sort (`sort=a,-b`) | Comma-separated sort list | |

**Choice:** Single sort key this phase (multi-key deferred — YAGNI).

## dispatch execution for query polish

| Option | Description | Selected |
|--------|-------------|----------|
| Extend WHERE assembly: split key on last `__`, allowlist op+base, bound params | Reuse equality allowlist+bind shape; non-disclosing error on unknown | ✓ |

**Choice:** Parameterized `__op` predicates + validated `sort` ORDER BY, applied with the
existing tenant + `deleted_at IS NULL` predicates untouched. `limit`/`offset` already
exist — not re-implemented.

## Claude's Discretion

- Builder/predicate naming (`build_create_input_schema`, `is_range_filter_field`, shared
  exclusion predicate).
- Crate location of the write-field exclusion predicate (prefer ferro-projections).
- JSON Schema niceties (per-op descriptions, format propagation).
- Feature-gating delete-tool emission behind `confirmation`.

## Deferred Ideas

- `derive_crud_plan` + create/update/delete execution (Phase 241).
- Write authz + tenant injection + non-disclosure (Phase 242).
- App flip + e2e + catalog/docs (Phase 243).
- Multi-key sort; dedicated `get_<svc>`; per-field immutable/read_only overrides.
