# Phase 241: `derive_crud_plan` + wire CRUD verbs into `framework::write` - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-23
**Phase:** 241-derive-crud-plan-wire-crud-verbs-into-framework-write
**Mode:** `--auto` (no interactive questions; recommended defaults auto-selected)
**Areas discussed:** CrudPlan shape, Kernel-wiring representation, Generic CRUD execution location, Delete confirmation reuse, Tenant-predicate boundary, Idempotency/audit reuse, Override key naming

---

## CrudPlan type shape

| Option | Description | Selected |
|--------|-------------|----------|
| Enum (Create/Update/Delete variants), pure serializable | Mirrors `TransitionPlan` "data not behavior" convention | ✓ |
| Single struct with optional fields per verb | Fewer types, but smears divergent verb shapes | |

**Choice:** Enum mirroring `TransitionPlan` (D-01). **Notes:** keeps the three verbs' divergent
SQL shapes explicit and serializable for the structured-envelope path.

## Kernel-wiring representation

| Option | Description | Selected |
|--------|-------------|----------|
| Thin CRUD verb discriminant through the same `dispatch_write` | One kernel, `transition_guard=None`, no fork | ✓ |
| Synthesize derived `ActionDef`s for create/update/delete | Reuses ActionDef path but smuggles transition semantics | |

**Choice:** Thin verb discriminant (D-03). **Notes:** SC#4 requires exactly one `dispatch_write`
with no second CRUD dispatcher and no transition `match` re-encoded on the CRUD path.

## Generic CRUD execution location

| Option | Description | Selected |
|--------|-------------|----------|
| Framework-provided generic CRUD executor interpreting `CrudPlan` | Zero hand-written tool code; override hook for the 20% | ✓ |
| App-supplied `ExecutorFn` for CRUD | Defeats the compression win of Track A | |

**Choice:** Framework-provided generic executor invoked through `dispatch_write` (D-04).
**Notes:** the entire Track A payoff is one declaration → working write surface with no per-verb code.

## Delete confirmation reuse

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse existing confirmation seam; flag CRUD delete destructive | Reuses ConfirmationStore + token binding; synthesize confirm tools | ✓ |
| New CRUD-specific confirmation token system | Duplicate control surface, forbidden | |

**Choice:** Reuse the existing seam, mark delete destructive, synthesize
`request_confirm_delete_<svc>`/`confirm_delete_<svc>` (D-06). **Notes:** delete schema already
advertises `confirmation_token` (Phase 240).

## Tenant-predicate boundary (241 ↔ 242)

| Option | Description | Selected |
|--------|-------------|----------|
| Defer tenant injection/authz/non-disclosure to Phase 242; leave a plan slot | Matches roadmap SC split; no double-build | ✓ |
| Implement tenant injection now in derive_crud_plan | Scope bleed into Phase 242 | |

**Choice:** Defer to 242; design `CrudPlan` with a tenant-predicate extension slot (D-09).
**Notes:** Phase 241 SC#1–#2 deliberately omit tenant.

## Idempotency / audit reuse

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse kernel idempotency + channel-parameterized audit unchanged | Only the verb identity string is new | ✓ |
| CRUD-specific idempotency/audit path | New mechanism, against spec | |

**Choice:** Reuse unchanged (D-08). **Notes:** audit label string is Claude's discretion.

## Override key naming

| Option | Description | Selected |
|--------|-------------|----------|
| Per-verb tool-name keys (`create_order` etc.) via existing `with_override` | No new mechanism; matches SC#3 | ✓ |
| New CRUD override registry | Duplicate registry | |

**Choice:** Reuse the keyed `with_override` registry with tool-name keys (D-07).

---

## Claude's Discretion

- `CrudVerb` enum placement and `CrudPlan` internal column/value representation.
- Audit label string for CRUD verbs.
- Generic CRUD executor form (free function vs method) and SQL builder.
- New `Error`/`WriteError` variant names (verb-not-enabled, row-not-found).

## Deferred Ideas

- Phase 242: write authz + tenant injection + non-disclosure.
- Phase 243: app flip + e2e + regression-guard extension + catalog/docs.
- Spec non-goals: dedicated `get_<svc>`, per-field `immutable()`/`read_only()`.
