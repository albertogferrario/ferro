# Phase 152: ferro-orm GuardedUpdate — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-
**Mode:** `--auto` (no interactive questions; Claude selected recommended defaults)
**Areas discussed:** Crate placement & scope, Builder API, Error model, Concurrency contract, Testing, Documentation, Release

---

## Crate placement & scope

| Option | Description | Selected |
|--------|-------------|----------|
| New top-level `ferro-orm` crate, thin v0 (GuardedUpdate only) | Wave 1a leaf crate. Claims the namespace, ships one primitive. Future extractions deferred. | ✓ |
| Extract full `framework/src/database/` into `ferro-orm` in this phase | Migrates `query_builder.rs`, `model.rs`, `connection.rs`, etc. Heavy refactor; cross-cutting. | |
| Add `GuardedUpdate` inside `framework/src/database/guarded.rs` | No new crate. Consumers depend on the whole framework crate. | |

**Claude's choice:** Option 1.
**Rationale:** The roadmap explicitly names `ferro-orm::GuardedUpdate`; phases 154 + downstream consumers expect to `use ferro_orm::...`. A full extraction would derail v11.11. Putting it inside `framework` forces every consumer to take the whole framework crate. A thin v0 claims the namespace cheaply and leaves the extraction question for a future milestone.

---

## SeaORM re-export scope

| Option | Description | Selected |
|--------|-------------|----------|
| Targeted re-exports (only what the public API uses) | Re-export `EntityTrait`, `ColumnTrait`, `ConnectionTrait`, `IntoCondition`, `SimpleExpr`, `Value`, `DbErr`. | ✓ |
| Full `pub use sea_orm::*` pass-through | Anything importable from sea-orm is importable from ferro-orm. | |
| No re-exports — consumers import sea-orm directly | `ferro-orm` only exposes its own types. | |

**Claude's choice:** Option 1.
**Rationale:** Targeted re-exports keep the surface inspectable (MCP-friendly), insulate consumers from incidental SeaORM 1.x → 2.x churn for the symbols `ferro-orm` actually uses, and avoid the maintenance debt of a blanket pass-through. Consumers that need the full sea-orm API depend on sea-orm directly.

---

## Multi-column `set_*` chaining

| Option | Description | Selected |
|--------|-------------|----------|
| Allow multiple chained `set_expr` / `set_value` calls; all applied in one UPDATE | Counter-decrement + `updated_at` in one atomic statement. | ✓ |
| Single set per builder (one column only) | Simpler, but forces two statements (race window) for combined updates. | |

**Claude's choice:** Option 1.
**Rationale:** Real-world race-free updates routinely need to mutate a counter AND a timestamp atomically. Forcing two statements would reintroduce the race window the type exists to prevent.

---

## Execution methods

| Option | Description | Selected |
|--------|-------------|----------|
| Ship both `exec_one` and `exec_at_most_one` | `exec_one` errors on 0 matches (predicate failure is load-bearing). `exec_at_most_one` returns `Ok(false)` (predicate failure is normal). | ✓ |
| Only `exec_one` (always-error semantics) | Simpler, but forces callers to catch `NoRowsAffected` even when 0 matches is expected. | |
| Only `exec` (returns rows_affected: u64) | Maximally raw; punts the error-vs-bool decision to the caller every time. | |

**Claude's choice:** Option 1.
**Rationale:** Both shapes are load-bearing for different call sites — `exec_one` for counter decrements where capacity exhaustion is the contract, `exec_at_most_one` for optimistic updates where "no match" is fine. Design doc specifies both.

---

## `TooManyRows` error variant

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `TooManyRows { affected: u64 }` — both exec methods reject `affected > 1` | Loudly surfaces index/uniqueness bugs. | ✓ |
| Drop it; treat `affected > 1` as success | Smaller error enum, but silently masks broken indexes. | |

**Claude's choice:** Option 1.
**Rationale:** Every guarded update is morally a unique-key operation. If two rows match, the assumption is broken; surfacing it is the whole point of the type.

---

## Filter API

| Option | Description | Selected |
|--------|-------------|----------|
| `filter(impl IntoCondition)` chainable; multiple calls AND-combine | Matches SeaORM `QueryFilter::filter` — feels native to anyone fluent in sea-orm. | ✓ |
| Custom `where_eq`, `where_gte`, etc. fluent helpers | More guided API but invents vocabulary parallel to sea-orm. | |
| Take a single `Condition` once via `with_filter(Condition)` | One filter call, more rigid. | |

**Claude's choice:** Option 1.
**Rationale:** Sticking to `IntoCondition` reuses every column comparator sea-orm already provides for free and keeps the builder feeling like an extension of sea-orm, not a parallel API.

---

## Connection type

| Option | Description | Selected |
|--------|-------------|----------|
| Generic `<C: ConnectionTrait>` — caller passes `&DatabaseConnection` or `&DatabaseTransaction` explicitly | Forces the caller to decide whether the update belongs in a transaction. | ✓ |
| Global `DB::connection()` shortcut | One less parameter, but invites accidental cross-connection race windows. | |

**Claude's choice:** Option 1.
**Rationale:** Race-free updates frequently belong inside a larger transaction (e.g. inventory decrement + audit-log write). Forcing the caller to pass the connection makes that decision explicit.

---

## Return value of `exec_one`

| Option | Description | Selected |
|--------|-------------|----------|
| Return `Result<(), GuardedError>` | Caller re-fetches the row if it needs the post-update value. | ✓ |
| `UPDATE … RETURNING` — return the post-update row | One round-trip but blocked on cross-dialect SeaORM support. | |

**Claude's choice:** Option 1.
**Rationale:** RETURNING is not portable across SQLite/Postgres in SeaORM 1.0 without dialect-specific code. Deferred until SeaORM abstracts it or we accept dialect-specific code.

---

## Empty-builder behaviour

| Option | Description | Selected |
|--------|-------------|----------|
| Error at `exec_*` time with `GuardedError::EmptyUpdate` | Catches programming bugs loudly at runtime. | ✓ |
| Compile-time type-state (`MissingSetMarker` → `ReadyToExec`) | Maximally safe but adds API surface and a type parameter. | |
| Silently no-op | Worst — produces an invalid UPDATE or silent success. | |

**Claude's choice:** Option 1.
**Rationale:** Runtime error reads better in tracebacks for a tool aimed at AI-authored code. Type-state would double the public surface for one bug class.

---

## Test backend

| Option | Description | Selected |
|--------|-------------|----------|
| In-memory SQLite only; unit + one concurrent integration test | Reuses existing framework testing harness, no docker, fast CI. | ✓ |
| SQLite + dockerised Postgres in CI | Highest coverage but heavy CI surface for one primitive. | |
| SQLite + property-based tests | Skip Postgres but add proptest. | |

**Claude's choice:** Option 1.
**Rationale:** SQLite's serial-writer model + the underlying SQL pattern's well-known atomicity on Postgres (`READ COMMITTED`) make additional backends disproportionate for this phase. Property-test budget is allocated to Phase 154. Postgres CI deferred to a dedicated infra phase if needed.

---

## Documentation

| Option | Description | Selected |
|--------|-------------|----------|
| Module-level rustdoc + `docs/src/database/atomic-updates.md` new page | Discoverable from both the API and the docs site. | ✓ |
| Rustdoc only | Cheap but less discoverable for users browsing the docs site. | |
| Full chapter with multi-example deep-dive | Heavier than the v0 surface justifies. | |

**Claude's choice:** Option 1.
**Rationale:** One user-facing page documenting the pattern (the anti-pattern it replaces, the API, the misuse footgun) is enough at v0 and matches how every other ferro-* primitive is documented.

---

## Release

| Option | Description | Selected |
|--------|-------------|----------|
| Patch bump + add to publish.yml Wave 1a + bootstrap first publish manually | Standard ferro process; CI token is publish-update-only. | ✓ |
| Hold publish until Phase 153/154 also land | Bigger release surface, more risk per ship. | |

**Claude's choice:** Option 1.
**Rationale:** Wave 1a is the correct wave (zero internal ferro deps). Bootstrap once, let CI take over for subsequent versions — same operational reality as ferro-wallet Phase 151.

---

## Naming

| Option | Description | Selected |
|--------|-------------|----------|
| `GuardedUpdate` | Matches design doc; "guarded" reads as "conditional/checked" without overclaiming. | ✓ |
| `AtomicUpdate` | "Atomic" implies more than DB-level (overclaim); used by other crates for different semantics. | |
| `ConditionalUpdate` | Accurate but vague — every UPDATE has a WHERE. | |

**Claude's choice:** Option 1.
**Rationale:** No reason to deviate from the spec; "Guarded" is recognisable and precise.

---

## Scope of related operations

| Option | Description | Selected |
|--------|-------------|----------|
| Just `GuardedUpdate` in Phase 152 | Tightest possible scope. | ✓ |
| `GuardedUpdate` + `GuardedDelete` together | Two primitives, twice the surface. | |
| `GuardedUpdate` + `GuardedDelete` + `GuardedInsert` | Full conditional-DML toolkit upfront. | |

**Claude's choice:** Option 1.
**Rationale:** v11.11's critical path needs only `GuardedUpdate` (Phase 154 calls it). `GuardedDelete` / `GuardedInsert` are plausible future additions, but speculatively adding them violates the "no design for hypothetical future requirements" rule.

---

## Audit-log integration

| Option | Description | Selected |
|--------|-------------|----------|
| Pure ORM primitive; no audit emission. Consumer wraps in `audit_log!()`. | Keeps Phase 152 leaf-pure. ferro-audit (Phase 153) owns audit. | ✓ |
| Optional audit-log hook injected via builder method | Couples two crates that should ship in parallel. | |

**Claude's choice:** Option 1.
**Rationale:** Phase 152 and Phase 153 are designed to ship in parallel as independent additive crates. Coupling them would force serial shipping.

---

## Claude's Discretion

The following decisions are explicitly left to the planner/executor (CONTEXT.md captures the boundary, not the implementation):

- Internal module layout of `ferro-orm/src/` (single `lib.rs` vs `lib.rs + guarded.rs`)
- Internal `SetTarget` enum shape (public surface is the chainable methods)
- Exact rustdoc prose and code-block formatting
- Test file names within `ferro-orm/tests/`
- Whether to expose `into_query()` for diagnostics (probably no — keeps the surface tight)

## Deferred Ideas

- `GuardedDelete`, `GuardedInsert` (out-of-scope for v11.11)
- Full `framework/src/database/` → `ferro-orm` extraction (future milestone)
- `UPDATE … RETURNING` (blocked on cross-dialect SeaORM support)
- Postgres CI integration tests
- Property-based tests (Phase 154 budget)
- `ferro::prelude` re-export of `GuardedUpdate`
- Audit-log / event emission on success (Phases 153 / 154 territory)
