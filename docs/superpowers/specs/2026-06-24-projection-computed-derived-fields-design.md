# Projection Computed / Derived Fields — Design

**Date:** 2026-06-24
**Status:** Design approved; implementation plan pending
**Scope:** `ferro-projections` (field write-exclusion), `framework::write` (recompute orchestration), sample `app` (field test)

## Problem

A projection field whose value is a function of related data — the canonical case
`order.total = SUM(line_items.amount)` — is currently declared as a writable
`.field()`. The CRUD derivation (Phase 240) therefore exposes it as an
agent-supplied input on `create_<svc>` and `update_<svc>`, so a client can set a
value that contradicts the underlying records (e.g. a `total` that does not match
the order's line items).

The framework already has the vocabulary to express "readable but not writable":
`ServiceDef::read_only_field()` sets `writable: false`. However, the write-input
schema derivation does not honor it. The single filter behind
`build_create_input_schema` / `build_update_input_schema` is
`ServiceDef::is_write_excluded_field` (`ferro-projections/src/service.rs`), whose
gates cover Identifier / CreatedAt / tenant column / UpdatedAt / Sensitive / list
fields / Status-under-StateMachine — but **not** the `writable` flag. As a result
a `read_only_field` still appears in write-input schemas, and the `writable`
control surface diverges from what the write surface actually accepts.

Beyond the prohibition, there is a correctness dimension: marking a field
non-writable forbids the lie but does not keep the value correct. Something must
recompute the derived value when its inputs change.

## Goals

- A field declared read-only is excluded from `create_`/`update_` input schemas
  and rejected by the write kernel (single source of truth for `writable`).
- A derived field's value is recomputed server-side when its inputs change, on
  every write surface (MCP and visual), with read-your-writes consistency.
- The recompute mechanism reuses the existing `framework::write` override-hook
  registry — no new control surface.
- An agent reading the projection can tell a field is derived and why.

## Non-Goals (deferred — see Future Direction)

- A declarative aggregation formula owned by the framework
  (`computed(SUM(line_items.amount))`) with a relation-aware dependency graph.
- Asynchronous / event-driven recomputation for high fan-out aggregations.
- Closing the post-persist crash window with a surrounding transaction.

## Approach

Two parts. Part A is a prerequisite for any derived-field story and is a pure
subtraction. Part C orchestrates recomputation by reusing the existing write
kernel seam.

### Part A — honor `writable` in write-input derivation

Add a gate to `ServiceDef::is_write_excluded_field`:

```
// Gate F: read-only field — declared non-writable, never an agent write input.
if !field.writable {
    return true;
}
```

This makes `read_only_field` (and any future `writable: false` field) disappear
from `create_<svc>` / `update_<svc>` input schemas, and — because the same
exclusion governs the write path — the kernel never accepts the field as input.
The `writable` flag becomes the single source of truth for write eligibility.

### Part C — framework-orchestrated recompute

1. **Declaration.** `order.total` is declared with `read_only_field(...,
   FieldMeaning::Money)` and a field `description` that states it is derived
   (e.g. "read-only; derived from line items"). The field remains in the read
   schema (`list_order`) and absent from the write schemas.

2. **Recompute hook.** The application registers a post-persist hook on the
   `line_item` write verbs (`create_line_item` / `update_line_item` /
   `delete_line_item`) via the existing override-hook registry on
   `WriteDispatcher`. The hook resolves the parent `order_id` from the persisted
   row and recomputes the parent total:

   ```sql
   UPDATE orders
      SET total = (SELECT COALESCE(SUM(amount), 0)
                     FROM line_items
                    WHERE order_id = ? AND deleted_at IS NULL)
    WHERE id = ?
   ```

   The framework owns the *when* (post-persist, on every write surface); the
   application owns the *how* (the formula, in Rust). The hook is plain code the
   app can unit-test.

3. **Create-time / NOT NULL.** A read-only derived field is excluded from the
   create input, so `execute_crud_plan`'s INSERT omits it. The column must
   therefore tolerate an omitted value: it must be nullable **or** carry a DB
   default. For `order.total` the field test adds `DEFAULT 0` to the column
   (migration), so creating an order with no line items inserts `total = 0`. As
   line items are added, their write hooks update the parent total. Declaring a
   sensible DB default (or nullability) for a derived column is an
   application-modeling responsibility — consistent with Part C, where the app
   owns the column definition and the formula.

### Recompute trigger — synchronous hook on `line_item` writes

The recompute fires synchronously inside the same request as the `line_item`
write, post-persist, through the override-hook registry. This gives
read-your-writes consistency: an agent that adds a line item and then reads the
order sees the correct total. It reuses the seam established in Phases 231/232
(single write kernel, channel-agnostic, override hooks) and adds no new
infrastructure. The aggregation is a single `SUM` scoped to one parent — bounded
and cheap for the parent-child cardinality this addresses.

Two alternatives were considered and deferred:

- **Declared dependency / auto-invalidation** — the framework owns a
  relation dependency graph and re-triggers the parent recompute automatically.
  This is half of the Future Direction (B): it introduces graph traversal and
  fan-out handling, the genuinely hard part. Deferred.
- **Event-driven** — `line_item` writes emit a domain event consumed by a
  recompute subscriber. Decoupled and suited to expensive or cross-aggregate
  recomputation, but introduces an eventual-consistency window that breaks
  read-your-writes for this case. Retained as the escape hatch for the
  expensive/asynchronous case alongside the Future Direction.

## Known Semantics (consistent with WR-01)

The recompute runs post-persist with **no surrounding transaction**, identical to
the existing override-hook contract. If the process crashes between the
`line_item` write and the recompute, the line item is persisted and the parent
total is briefly stale. This is the same crash window already accepted by the
write kernel. A stronger guarantee (recompute-on-read, or a periodic reconcile)
is noted as future work, not built here.

## Introspection / Agent Experience

- `create_order` / `update_order` input schemas no longer list `total`.
- `list_order` continues to read `total`.
- The field `description` signals the value is derived, so an agent understands
  *why* it cannot write it.
- Behavior is identical across the MCP and visual write surfaces (single kernel).

## Field Test (sample app)

The sample `order` projection references `line_item` via
`.has_many("line_items", "line_item")` but no `line_item` projection or table
exists yet. The field test requires:

- A `line_item` table (`id`, `order_id`, `amount`, `deleted_at`) and a
  `line_item` CRUD projection routed through the write kernel.
- `order.total` redeclared as `read_only_field`, and a migration adding
  `DEFAULT 0` to the `orders.total` column (so create-without-total inserts `0`).
- The recompute hook registered on the `line_item` write verbs.

End-to-end drive against `:8090`: create an order (total `0`) → add two line
items → total reflects the sum → delete a line item → total updates. Confirms
read-your-writes and the Phase 205 envelope on each verb.

## Testing

- `is_write_excluded_field`: a `read_only_field` is excluded from create/update
  schemas (Gate F).
- In-process e2e: `create_order` rejects a `total` argument; the recompute hook
  updates the parent total after each `line_item` write/delete.
- The Phase 205 structured-envelope guard is unchanged for all verbs.

## Future Direction (not built here)

**B — framework-owned declarative computation.** A projection declares the
formula directly (`computed(SUM(line_items.amount))`), backed by an aggregation
engine and relation-aware invalidation, introspectable via MCP so an agent reads
the derivation as data, cross-modal. This is the formulation most aligned with
the projection-as-single-source-of-truth direction. The Part C hook is the
incremental step toward it: when the engine exists, the registered hook is
replaced by the declared formula — an evolution, not a breaking change. For
high fan-out or expensive recomputation, an event-driven variant (ferro-events /
ferro-projection) is the asynchronous counterpart. Deferred until more
application classes require it, to avoid building the engine ahead of validated
demand.
