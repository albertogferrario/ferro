# Phase 163: JSON-UI improvements batch 2 — cassa and calendario field-test findings — Context

**Status:** Awaiting friction file. Do not plan yet.

## Phase Boundary

Phase 163 consumes the FRICTION.md files produced by gestiscilo Phases 140 (cassa) and 141 (calendario). Cassa surfaces list/detail/form patterns at scale; calendario surfaces the kanban board and real-time SSE interactions.

## Expected friction sources (when ready)

- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/140-*/FRICTION.md` — cassa migration (orders, products, payments). Heterogeneous iteration cases already previewed in Phase 138's friction file: `cassa/orders.rs:230` (status-dependent badges), `cassa/orders.rs:380` (dynamic ProductTiles), `cassa/orders.rs:754` (conditional header actions), `cassa/products.rs:290` (edit-mode dynamic rows).
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/141-*/FRICTION.md` — calendario migration (booking kanban, calendar view, SSE real-time updates).

## Expected scope (pre-friction-file estimate, will be revised)

The Phase 138 friction file's "Extended Iteration Gap" section already named three directives that will land here based on cassa's heterogeneous iteration needs:

1. `$each` directive — spec-level iteration over a `$data` array path, templated per row. Addresses homogeneous lists not natively supported by `KanbanBoard` / `DataTable` (e.g., custom Card lists, custom Description grids).
2. `$if` directive — conditional element emission. Differs from existing `visible` operator in that absent elements are not rendered at all (no hidden DOM); missing IDs in `children` arrays are silently skipped.
3. `$template` element with `$each` for auto-suffixed IDs (`row_template-0`, `row_template-1`, …) — addresses runtime-computed element IDs.

Likely companion work surfaced by Phase 138 blast-radius observation:

- `SpecBuilder` ergonomic nested DSL — Rust-side spec construction for cases that even with `$each` / `$if` cannot be expressed declaratively. Reduces `Spec::builder()` boilerplate; emits the flat element map automatically.
- `ferro json-ui:migrate-v1` codemod — auto-rewrites v1 `make_node(id, Component::X(props))` call trees into stub JSON spec entries with the flat element map pre-generated.

## Predecessor

Phase 162 (auth/account/onboarding/pages friction loop). Decisions D-01..D-24 in `../162-.../162-CONTEXT.md`. Phase 163 does not depend on Phase 162 completing for friction-file collection but does inherit the catalog and surface-level decisions made there.

## Planning gate

Do not run `/gsd-plan-phase 163` until both 140-FRICTION.md and 141-FRICTION.md exist. The planner should read both, classify every entry, and rewrite this CONTEXT with locked decisions before plan-creation begins.
