# Phase 154: ferro-reservation — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 154-CONTEXT.md — this log preserves the alternatives that were considered and recommended-default rationale under `--auto` mode.

**Date:** 2026-05-13
**Phase:** 154-ferro-reservation-crate-generic-hold-commit-release-with-ttl
**Mode:** `--auto` (recommended defaults selected for every gray area; no interactive AskUserQuestion calls)
**Areas auto-discussed:** Crate placement & scope · Resource trait API · Kernel API & connection model · Status state machine & concurrency · Sweeper integration · Event emission · Audit emission · TTL & extend semantics · ReservationHandle shape · Multi-tenancy · Schema & migration · Error model · Testing strategy · Documentation & MCP · Wave placement & release

---

## Crate placement & scope

| Option | Description | Selected |
|--------|-------------|----------|
| New top-level workspace crate `ferro-reservation/` (Wave 1b leaf) | Mirrors Phase 152 / 153 placement; isolated dep graph; named in the roadmap | ✓ |
| Inside `framework/src/reservation/` | Forces every consumer to depend on full framework crate | |
| Inside `ferro-orm` | Conflates ORM primitives with a domain-shaped kernel | |

**Auto choice:** New top-level workspace crate. Locked as D-01.
**Reason:** Roadmap explicitly names `ferro-reservation`; downstream apps import it as `use ferro_reservation::...`; pattern matches the two completed sibling phases.

---

## Resource trait API

| Option | Description | Selected |
|--------|-------------|----------|
| Trait per design doc with `&DatabaseConnection` | Exact match to INVENTORY-PRIMITIVES.md | |
| Trait generic over `<C: ConnectionTrait>` | Matches Phase 152 / 153 connection pattern; lets consumer wrap calls in transactions | ✓ |
| Sync trait with blocking DB calls | Incompatible with async kernel + ferro-events | |

**Auto choice:** Generic over `<C: ConnectionTrait>`. Locked as D-05, D-06.
**Reason:** Consistency with `GuardedUpdate::exec_one` and `AuditEntry::write` — both already accept `<C: ConnectionTrait>`. Deliberate deviation from the design doc to keep the workspace convention uniform.

---

## Kernel API & connection model

| Option | Description | Selected |
|--------|-------------|----------|
| Kernel owns `DatabaseConnection`; per-call methods take `&C: ConnectionTrait` | Owned conn for sweeper-internal calls; explicit conn for caller-driven transactions | ✓ |
| Kernel owns conn only; no per-call conn | Forces sweeper-style usage; loses transactional composition | |
| All methods take conn explicitly; no owned conn | Forces sweeper consumers to inject a conn on every tick; adds friction | |

**Auto choice:** Owned + per-call. Locked as D-09 through D-15.
**Reason:** Sweeper has no natural caller; per-call methods benefit from explicit conn for transactional composition. Same shape `ferro_audit::AuditEntry::write` chose for the same reason.

---

## Status state machine & concurrency

| Option | Description | Selected |
|--------|-------------|----------|
| Four distinct statuses: held / committed / released / expired | Clean audit + event distinction; matches design doc | ✓ |
| Three statuses (collapse expired into released with reason=Expired) | Smaller state machine; loses sweeper-vs-cancel distinction in queries | |
| Two statuses (active / settled, with reason discriminator) | Even smaller; loses the time-vs-action distinction | |

**Auto choice:** Four distinct statuses + `ReleaseReason` enum. Locked as D-16, D-17, D-18.
**Reason:** Design doc explicit. Distinct statuses make the sweeper path queryable (`status = 'held' AND expires_at < now()`) without needing to filter by reason. Audit / event consumers benefit from the structured distinction.

| Option | Description | Selected |
|--------|-------------|----------|
| All state transitions through `ferro_orm::GuardedUpdate` (single `UPDATE` statement) | Race-free by construction; matches v11.11 design | ✓ |
| Application-side mutex around transitions | Single-process correctness only; breaks under multiple workers | |
| `SELECT … FOR UPDATE` + UPDATE inside transaction | Two round-trips; SQLite doesn't honor `FOR UPDATE`; portability cost | |

**Auto choice:** `GuardedUpdate`-only. Locked as D-12, D-19.
**Reason:** This is precisely why Phase 152 was built. Using anything else would defeat the milestone's correctness story.

---

## Sweeper integration

| Option | Description | Selected |
|--------|-------------|----------|
| Expose `run_sweep_once(&self) -> SweepReport`; no runtime ferro-queue dep | Consumers wire scheduling via their own queue/cron/tokio interval | ✓ |
| Ship a `ReservationSweeperJob: ferro_queue::Job` impl; runtime dep on ferro-queue | Plug-and-play but couples to one scheduler | |
| Spawn a background tokio task at kernel construction | Hidden lifecycle; hard to test deterministically | |

**Auto choice:** `run_sweep_once` primitive only; documented scheduling patterns in rustdoc. Locked as D-21, D-22.
**Reason:** Keeps the crate scheduler-agnostic. ferro-queue is one valid wiring; cron / tokio interval are equally valid. Reduces runtime dep graph. Future v0.x can add a `ReservationSweeperJob` convenience without breaking changes.

---

## Event emission (ferro-events)

| Option | Description | Selected |
|--------|-------------|----------|
| Crate emits `ReservationEvent` via `ferro_events::dispatch` automatically on every state transition | Predictable contract; consumer just attaches listeners | ✓ |
| Crate returns the event; consumer dispatches | Manual; easy to miss; defeats the purpose of integrated event emission | |
| Optional emission via feature flag | Adds API surface for a marginal benefit | |

**Auto choice:** Automatic emission post-transition. Locked as D-25, D-26.
**Reason:** Design doc explicit; consumer wiring is just `Listener` registration. Event failure logged but does not roll back state — the audit log is the source of truth for replay.

| Option | Description | Selected |
|--------|-------------|----------|
| Event payload uses typed `R::Key` / `R::Window` via generic event | Type-safe but ferro-events fanout is JSON-shaped at the bus | |
| Event payload uses `JsonValue` for `resource_key` / `window` | Matches event-bus boundary; subscribers re-deserialize if typed access needed | ✓ |

**Auto choice:** JSON-shaped payload. Locked as D-25.
**Reason:** ferro-events boundary is JSON; making the event generic over `R` complicates listener registration and breaks the inventory-style registry pattern.

---

## Audit emission (ferro-audit)

| Option | Description | Selected |
|--------|-------------|----------|
| Audit emission unconditional on every state transition; `ReservationContext` carries actor metadata | Strongest historical-evidence story; matches v11.11's audit-trail promise | ✓ |
| Audit opt-in per call | Easy to skip; weakens the kernel's evidence guarantee | |
| Audit opt-in at kernel construction (`with_audit` constructor) | Two construction paths; opaque to listeners | |
| Audit emission deferred entirely to caller (wrap the call) | Matches Phase 152 / 153 pattern but loses the integrated story for v11.11 | |

**Auto choice:** Unconditional emission with per-call `ReservationContext` bundle. Locked as D-28, D-29, D-30.
**Reason:** ferro-reservation is the *integrated* primitive at the top of the v11.11 stack — its job is to compose 152 + 153 into a single typed contract. Making audit optional undermines that. `ReservationContext` keeps the per-call API single-parameter clean while threading actor / correlation / tenant.

---

## TTL & extend semantics

| Option | Description | Selected |
|--------|-------------|----------|
| `extend(handle, by: Duration)` — relative extension; multiple extends compound | Matches design doc | ✓ |
| `extend_until(handle, absolute: DateTime<Utc>)` | Caller does arithmetic; less ergonomic | |
| `extend(handle, by, max_total)` — cap on cumulative extension | Useful safety rail but adds caller burden | |
| Auto-extend on `commit` failure / retry | Hidden semantics; surprises consumers | |

**Auto choice:** Relative `Duration`, no cap. Documented operational risk. Locked as D-31, D-32, D-33.
**Reason:** Design doc explicit. v0 stays minimal; consumers wanting a max-TTL cap enforce it at the call site.

---

## ReservationHandle shape

| Option | Description | Selected |
|--------|-------------|----------|
| Full snapshot: id + resource_kind + key + window + quantity + held_at + expires_at + tenant_id | Caller can embed in side-channel (Stripe metadata, queue job payload) without re-fetch | ✓ |
| Opaque `id: Uuid` only | Minimal but forces re-query for every check | |
| `id + expires_at` only | Compromise; loses key/window for callers that need to dispatch | |

**Auto choice:** Full snapshot. Serde-derived. Locked as D-34, D-35.
**Reason:** Stripe-payment-flow consumers embed the handle in `payment_intent.metadata`; ticketing consumers embed in queued-job payloads. Full snapshot removes a round-trip on every commit/release path.

| Option | Description | Selected |
|--------|-------------|----------|
| Handle passed by value to `commit` / `release` / `extend` (use-once semantics enforced at type level) | Compile-time prevents handle re-use bugs | ✓ |
| Handle passed by reference (re-usable) | Allows querying same handle multiple times but risks double-commit bugs | |

**Auto choice:** By value. Locked as D-11.
**Reason:** Use-once at the type level is the Rust idiom for "this handle represents a one-shot transition." Re-use is rare and the caller can always reconstruct a `ReservationHandle` from the row id if needed.

---

## Multi-tenancy

| Option | Description | Selected |
|--------|-------------|----------|
| `tenant_id: Option<String>` on row + in `ReservationContext` | Matches ferro-audit's stringly-typed convention; forward-compatible | ✓ |
| Typed `TenantId` newtype | No framework-wide tenant primitive exists; would force consumers to convert | |
| Tenant baked into `Resource::Key` | Works but ad-hoc; loses the orthogonal "audit by tenant" dimension | |

**Auto choice:** `Option<String>`. Locked as D-36, D-37.
**Reason:** Mirrors Phase 153 D-13 and the existing codebase reality (no first-class tenant primitive). Stays domain-neutral.

---

## Schema & migration

| Option | Description | Selected |
|--------|-------------|----------|
| Migration as public re-export (`pub use migration::Migration as CreateReservationsTable`) | Consumers register explicitly in their `Migrator`; matches Phase 153 D-18 | ✓ |
| Run migrations automatically at kernel construction | Hidden side effects; breaks the `Migrator` discipline consumers already follow | |
| Ship schema as raw SQL files for consumer to run | Loses portability across SQLite / Postgres | |

**Auto choice:** Public re-export. Locked as D-38.
**Reason:** Phase 153 established the pattern; consumers expect it; SeaORM `Migrator` is the canonical wiring.

| Option | Description | Selected |
|--------|-------------|----------|
| `status` as VARCHAR | Cross-dialect; easy MCP introspection | ✓ |
| `status` as SeaORM `ActiveEnum` | Typed but forces enum-as-column; more migration complexity | |

**Auto choice:** VARCHAR. Locked as D-16.
**Reason:** Stringly-typed status is fine for a four-value enum and keeps the migration trivially portable.

---

## Error model

| Option | Description | Selected |
|--------|-------------|----------|
| Single `ReservationError` umbrella enum with variants for each failure mode | Matches one-error-per-crate convention; `?` works cleanly | ✓ |
| Separate `HoldError` / `CommitError` / `ReleaseError` enums | Tighter per-method typing but breaks `?` ergonomics and bloats the API | |
| `anyhow::Error` opaque return | Loses structured error introspection | |

**Auto choice:** Single umbrella enum. Locked as D-43 through D-46.
**Reason:** Workspace convention (ferro-orm, ferro-audit, ferro-stripe all use one error enum). `?` ergonomics matter for kernel internals that compose ferro-orm + ferro-audit + ferro-events.

---

## Testing strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Unit tests in `src/` + 1 concurrency integration test + property tests via proptest + cross-crate integration test | Covers correctness + property-test budget for v11.11 + showcase of three-crate composition | ✓ |
| Unit tests only | Misses the concurrency correctness claim | |
| Skip property tests; rely on hand-written concurrency tests | Phase 153 D-32 documented the budget lands here; can't punt twice | |

**Auto choice:** Full suite (unit + integration + property + cross-crate). Locked as D-47 through D-52.
**Reason:** v11.11 design doc places the property-test budget in this crate. The cross-crate integration test is the milestone's showcase: a single test that proves 152 + 153 + 154 compose as designed.

| Option | Description | Selected |
|--------|-------------|----------|
| `proptest 1` as dev-dep | Modern, good shrinking, widely used | ✓ |
| `quickcheck` | Older API; weaker shrinking | |
| Hand-written randomized tests with `rand` | Re-invents proptest poorly | |

**Auto choice:** `proptest`. Locked as D-49.
**Reason:** Industry standard for Rust property tests; first appearance of property testing in the workspace warrants the best-in-class library.

| Option | Description | Selected |
|--------|-------------|----------|
| Postgres CI integration tests in this phase | Catches dialect bugs early | |
| Postgres CI integration deferred (SQLite-only in CI) | Matches Phase 152 D-19 and Phase 153 D-33 | ✓ |

**Auto choice:** Deferred. Locked as D-51.
**Reason:** Disproportionate for one primitive; cross-phase pattern is consistent. Risk acknowledged.

---

## Documentation & MCP

| Option | Description | Selected |
|--------|-------------|----------|
| Module rustdoc + user-facing `docs/src/database/reservations.md` + state diagram | Matches Phase 152 / 153 doc placement | ✓ |
| Rustdoc only, no user-facing doc page | Misses the new-consumer onboarding story | |
| User-facing doc only | Rust ecosystem expects rustdoc; cargo doc users would be lost | |

**Auto choice:** Both. Locked as D-53, D-54.
**Reason:** Phase 152 + 153 set the precedent under `docs/src/database/`. Three new entries (atomic-updates / audit-log / reservations) form the v11.11 doc trilogy.

| Option | Description | Selected |
|--------|-------------|----------|
| No new MCP tools; rely on `application_info` + `db_schema` auto-pickup | Minimal; consistent with Phase 152 / 153 | ✓ |
| Add `reservation_check_capacity` MCP tool in this phase | Useful but scope creep | |

**Auto choice:** No new MCP tools. Locked as D-55.
**Reason:** Same call as Phase 153 D-37. The MCP introspection layer is the right home for agent-facing queries, but v0.x once a real consumer surfaces.

---

## Wave placement & release

| Option | Description | Selected |
|--------|-------------|----------|
| Wave 1b (depends on Wave 1a `ferro-orm` + `ferro-events` + `ferro-audit`) | Correct per dep graph; ships after the three Wave 1a crates | ✓ |
| Wave 1a leaf | Impossible — ferro-reservation has internal ferro-* deps | |
| Wave 2 | Unnecessarily late; nothing in this phase depends on ferro-rs or ferro-mcp | |

**Auto choice:** Wave 1b. Locked as D-04, D-57.
**Reason:** Strict dep graph placement; matches `ferro-notifications` (Wave 1b) which depends on `ferro-whatsapp` (Wave 1b) and `ferro-broadcast` (Wave 1a).

| Option | Description | Selected |
|--------|-------------|----------|
| Workspace version bumps 0.2.31 → 0.2.32 on Phase 154 verify | Matches per-phase patch cadence (152 → 0.2.30, 153 → 0.2.31, 154 → 0.2.32) | ✓ |
| Skip version bump until full v11.11 milestone ships | Delays consumer access to ferro-reservation; breaks the per-phase cadence | |

**Auto choice:** 0.2.32 on verify. Locked as D-56.
**Reason:** Matches the cadence the last two phases established; consumers of ferro-orm and ferro-audit are already on 0.2.31 — adding 0.2.32 with ferro-reservation lets them adopt incrementally.

---

## Claude's Discretion

The following decisions were intentionally left to the planner / executor:

- Internal module layout of `ferro-reservation/src/` (likely `lib.rs` + `kernel.rs` + `resource.rs` + `handle.rs` + `context.rs` + `event.rs` + `error.rs` + `migration.rs` + `entity.rs` + `sweeper.rs` — consolidation acceptable)
- Public re-export of SeaORM `Entity` / `Model` / `ActiveModel` for native SeaORM queries (recommended; matches ferro-audit)
- Exact wording of `tracing::warn!` diagnostics
- Whether `SweepReport` is public (recommended yes)
- Exact `proptest` strategy generators (the properties are locked; the generators are open)
- Test file naming inside `ferro-reservation/tests/`
- Whether to ship a `ReservationKernel::available_capacity` convenience helper

## Deferred Ideas

Captured in 154-CONTEXT.md `<deferred>` section. Highlights:

- `try_hold` non-blocking variant
- Bulk cancel / multi-resource hold
- Distributed locks
- Postgres CI tests
- MCP introspection tool
- WebSocket fanout (Phase 155 territory)
- ferro-queue Job impl
- Per-call audit suppression
- CLI `ferro reservation:sweep` subcommand
