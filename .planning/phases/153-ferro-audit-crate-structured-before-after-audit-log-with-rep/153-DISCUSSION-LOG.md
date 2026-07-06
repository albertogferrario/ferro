# Phase 153: ferro-audit — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 153-ferro-audit-crate-structured-before-after-audit-log-with-rep
**Mode:** `--auto` (recommended defaults applied without interactive selection)
**Areas discussed:** Crate placement & scope · Actor & target model · Builder vs macro API · Schema & migration delivery · Replay (`reconstruct_state`) · Retention · Testing · Release

---

## Crate placement & scope

| Option | Description | Selected |
|--------|-------------|----------|
| Top-level workspace crate `ferro-audit/` (Wave 1a, leaf) | Mirrors Phase 152's `ferro-orm/` placement; no internal ferro deps; ships in parallel with 152 | ✓ |
| Submodule inside `framework/src/audit/` | Forces every consumer onto the full framework crate for an independently-useful primitive | |
| Inside `ferro-orm/` as an `audit` submodule | Couples two unrelated primitives; bloats `ferro-orm` past its v0 GuardedUpdate-only intent | |

**Selected option:** Top-level workspace crate `ferro-audit/`, Wave 1a publish, no internal ferro deps.
**Rationale:** Aligns with `INVENTORY-PRIMITIVES.md` cross-crate diagram (`ferro-audit` is near-leaf) and lets phases 152 + 153 ship truly in parallel. Captured in D-01 .. D-04.

---

## Actor model

| Option | Description | Selected |
|--------|-------------|----------|
| `AuditActor` enum with stringly-keyed variants | `User(String) \| System \| Job(String) \| ApiClient(String) \| Anonymous` — domain-agnostic, never binds to a consumer's user-id type | ✓ |
| Generic `AuditActor<Id>` parameterized on the consumer's id type | Type-safe but propagates the generic everywhere; breaks the "ferro doesn't know your User model" rule | |
| `AuditActor` as a free-form `String` | Loses the typed-variant ergonomics; no compile-time exhaustiveness on the actor kind | |

**Selected option:** `AuditActor` typed enum, stringly-keyed payloads.
**Rationale:** Schema is stringly-typed (`actor_kind`, `actor_id`); typed-enum variants give compile-time exhaustiveness without binding ferro to a consumer's id shape. Captured in D-05.

---

## "Current actor" pickup from request

| Option | Description | Selected |
|--------|-------------|----------|
| No automatic pickup — caller passes `AuditActor` explicitly | Keeps the crate at Wave 1a (no `framework` dep); explicit is testable | ✓ |
| `AuditActor::from_request(&Request)` helper | Adds a `framework` dep, breaks leaf-crate placement | |
| Task-local / `tracing` span pickup of `actor_id` | Needs framework-level plumbing not currently in place | |

**Selected option:** Explicit caller-supplied actor.
**Rationale:** Future addition once framework plumbs a current-user task-local; not blocking. Captured in D-06.

---

## Target model

| Option | Description | Selected |
|--------|-------------|----------|
| `AuditTarget { kind: String, id: String }` struct | Open-ended; consumer-defined dotted-namespace convention; no upstream-blocking enum variants | ✓ |
| Closed `AuditTarget` enum with consumer-defined variants | Forces every consumer to upstream variants or carry a `Custom(String, String)` escape hatch | |
| `AuditTarget` generic over the consumer's id type | Type-safety theatre — collapses to `ToString` at the DB layer anyway | |

**Selected option:** `AuditTarget` struct with `kind` + `id` strings.
**Rationale:** Schema is stringly-typed; ferro stays domain-agnostic. Captured in D-07, D-08.

---

## Write API — builder vs macro

| Option | Description | Selected |
|--------|-------------|----------|
| Typed builder: `AuditEntry::record(action).actor(…).target(…).before(…).after(…).write(&conn).await` | Introspectable in MCP `code_templates` / `generation_context`; consumes-self pattern matches framework convention | ✓ |
| `audit_log!(…)` macro (as shown in `INVENTORY-PRIMITIVES.md`) | Less introspectable; harder to compose conditionally; can be a v0.x thin wrapper over the builder | |
| Free function `audit::log(...)` | Mid-point; doesn't compose as cleanly as a builder | |

**Selected option:** Typed builder; macro deferred to v0.x.
**Rationale:** Ferro convention strongly favors typed introspectable surfaces. Captured in D-09 .. D-14.

---

## Required vs optional fields

| Option | Description | Selected |
|--------|-------------|----------|
| `action` required (returns `MissingAction`); `target` optional but warns; `actor` defaults to `System` | Append-only audit must never refuse a write that has *something*; missing `action` is uninterpretable | ✓ |
| All fields required at compile time via type-state | Heavyweight; doesn't add safety vs a runtime error for an obvious bug | |
| All fields optional, including action | Loses the "this is uninterpretable" signal at write time | |

**Selected option:** `action` required, `target` optional with `tracing::warn!`, `actor` defaults to `System`.
**Rationale:** Pragmatic compromise; missing `target` is rare but valid for system-level events. Captured in D-10, D-15, D-16.

---

## Schema delivery

| Option | Description | Selected |
|--------|-------------|----------|
| Public re-export `pub use migration::Migration as CreateAuditLogTable` for consumers' `Migrator` | Standard SeaORM pattern; consumers control ordering; composable | ✓ |
| Raw SQL constants the consumer drops into a manual migration | Loses cross-dialect ergonomics; forces consumer to maintain the SQL | |
| Runtime auto-migration on first `write()` call | Magical; can race with consumer migrations; not idiomatic | |

**Selected option:** Public `CreateAuditLogTable` migration the consumer registers.
**Rationale:** Matches downstream consumer pattern; predictable migration ordering. Captured in D-18, D-19, D-20.

---

## `id` generation

| Option | Description | Selected |
|--------|-------------|----------|
| Client-side `Uuid::new_v4()` at `write()` time | Caller can attach the id to events / responses before the row hits the DB | ✓ |
| DB-side `gen_random_uuid()` / auto-increment | Cross-dialect headaches; can't reference the id pre-write | |

**Selected option:** Client-side UUIDv4.
**Rationale:** Cross-dialect, allows pre-write referencing. Captured in D-21.

---

## Replay (`reconstruct_state`)

| Option | Description | Selected |
|--------|-------------|----------|
| Shallow JSON object merge over `history_for_target` results | Simple, predictable, covers 80% of use cases; pure function (no DB); documents the limit | ✓ |
| Deep merge (recursive) | Surprising edge cases on arrays; defer to v0.x once a real consumer needs it | |
| No replay helper — consumer rolls their own fold | Loses the "replay" promise from the phase title | |

**Selected option:** Shallow merge, documented as v0 semantics; deep-merge deferred.
**Rationale:** Predictable, easy to reason about, easy to test. Captured in D-24.

---

## Retention

| Option | Description | Selected |
|--------|-------------|----------|
| `prune_older_than(cutoff, conn)` helper; no automatic enforcement; default "keep forever" | Consumer drives via ferro-queue cron; matches the design doc's stated default | ✓ |
| Automatic background sweeper inside `ferro-audit` | Hidden side effects; couples to ferro-queue | |
| No prune helper at all | Forces every consumer to write the SQL themselves | |

**Selected option:** Explicit `prune_older_than` helper, no auto-enforcement.
**Rationale:** Compliance-driven decision belongs at the consumer level. Captured in D-26, D-27.

---

## Testing scope

| Option | Description | Selected |
|--------|-------------|----------|
| Unit tests (in-source) + 1 integration test (`replay_round_trip.rs`); SQLite only | Sufficient for an append-only crate; matches Phase 152's pattern | ✓ |
| Add property-based tests for replay reconstruction | Phase 154 carries the milestone's property-test budget per `INVENTORY-PRIMITIVES.md` | |
| Add Postgres CI integration tests | Disproportionate for one primitive | |

**Selected option:** Hand-written unit + 1 integration test; SQLite in-memory; property + Postgres deferred.
**Rationale:** Mirrors Phase 152 testing scope; sufficient surface coverage. Captured in D-30 .. D-34.

---

## Release / publish wave

| Option | Description | Selected |
|--------|-------------|----------|
| Wave 1a alongside `ferro-orm` and `ferro-wallet`; workspace version 0.2.25 → 0.2.26 | No internal ferro-* deps per D-03; ships in parallel with Phase 152 | ✓ |
| Wave 1b (depends on `ferro-orm`) | Would serialize phases 152 and 153 unnecessarily; `ferro-audit` doesn't use `GuardedUpdate` | |

**Selected option:** Wave 1a, version bump 0.2.25 → 0.2.26.
**Rationale:** Decoupling from ferro-orm preserves the "ship in parallel" intent stated in `INVENTORY-PRIMITIVES.md` §`Migration / rollout`. Captured in D-38, D-39, D-40.

---

## Claude's Discretion

Internal module layout, exact rustdoc prose, JSON-merge implementation detail in `reconstruct_state`, test file naming, and whether to expose `migration` as a module or via a top-level alias — all left to the planner/executor within the D-01 .. D-40 envelope.

## Deferred Ideas

- `audit_log!` macro façade (post-v0)
- Automatic `correlation_id` pickup from `tracing` / task-local
- `AuditActor::from_request(&Request)` helper
- Ferro-events `AuditEntryRecorded` emission on write
- MCP tools to query the audit log from an agent
- Distributed audit-stream / log shipping
- Postgres CI integration tests
- Property-based tests (Phase 154 carries the budget)
- PII redaction / GDPR right-to-erasure tooling beyond `prune_older_than`
- Deep-merge `reconstruct_state` variant
- Pagination helpers (consumer uses SeaORM directly)
