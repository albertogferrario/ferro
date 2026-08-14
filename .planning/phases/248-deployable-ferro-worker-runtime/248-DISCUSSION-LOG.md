# Phase 248: Deployable `ferro worker` runtime - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-08-14
**Phase:** 248-deployable-ferro-worker-runtime
**Areas discussed:** Worker CLI surface, Fault-domain routing, WR-01 wiring ownership, Deploy scaffolder scope

---

## Worker CLI surface

### Q: Worker subcommand selector — how does an operator pick which queues a `<app-bin> worker` process consumes?

| Option | Description | Selected |
|--------|-------------|----------|
| `--queue`, repeatable | Maps directly to WorkerConfig.queues; groups queues into one fault domain; no new concept | ✓ |
| `--class` (needs registry) | Matches roadmap wording but requires a class→queues registry (rejected new concept) | |
| positional queues | `worker reports emails`; terse, less discoverable, clashes with future positional args | |

**User's choice:** `--queue`, repeatable.

### Q: When `worker` runs with NO queue selector, what does it consume?

| Option | Description | Selected |
|--------|-------------|----------|
| All registered queues | Catch-all; opt into isolation via --queue; needs distinct-queue-set derivation | ✓ |
| default queue only | Matches WorkerConfig::default(); silently ignores non-default queues | |
| Require explicit --queue | Safest for prod, most friction; bare `worker` does nothing | |

**User's choice:** All registered queues.

---

## Fault-domain routing

### Q: How does an `#[offload]` method get routed to a specific queue/class for fault-domain isolation (SC#3)?

| Option | Description | Selected |
|--------|-------------|----------|
| `#[offload(queue=…)]` | Declared on the method; defaults to `default`; introspectable single source of truth | ✓ |
| runtime `.on_queue()` | Per-dispatch; flexible but not introspectable, splits the contract | |
| single offload queue | One shared fault domain; defeats SC#3 | |

**User's choice:** `#[offload(queue=…)]` attribute arg.

### Q: Which queues does `serve`'s default in-process worker consume (single-binary "just works")?

| Option | Description | Selected |
|--------|-------------|----------|
| All registered queues | Every offloaded method runs under bare `serve` in dev; prod uses --no-worker + worker | ✓ |
| default only (current) | Keeps WorkerConfig::default(); non-default `#[offload]` methods silently never run | |

**User's choice:** All registered queues.

---

## WR-01 wiring ownership

### Q: Who turns `config.transport_redis_url` into a live RedisTransport and attaches it via `.with_transport()`?

| Option | Description | Selected |
|--------|-------------|----------|
| Framework boot, every app | Auto-wire at boot; project-agnostic; feature-off + URL-set warns and falls back | ✓ |
| Sample-app bootstrap.rs only | Per-app copy; duplicate control surface; violates framework conventions | |

**User's choice:** Framework boot, every app.
**Notes:** Env key already decided in code — `BROADCAST_REDIS_URL` with `REDIS_URL` fallback (`BroadcastConfig::from_env()`). RedisTransport is feature-gated; feature-off + URL-set → warn + in-process fallback.

---

## Deploy scaffolder scope

### Q: Does Phase 248 emit deploy `workers:` components from `[package.metadata.ferro.deploy]`, or defer that?

| Option | Description | Selected |
|--------|-------------|----------|
| Defer; runtime + verify + WR-01 | 248 stays focused; scaffolder emission is a separate deploy concern (122.x line) | ✓ |
| Include scaffolder emission now | `do:init`/`docker:init` emit per-class components; larger coupled scope | |
| Metadata schema + docs only | Define the `workers` schema + example, no generator changes | |

**User's choice:** Defer; 248 = runtime + verification + WR-01.

---

## Claude's Discretion

- Factoring of the shared boot path into a reusable framework `run_worker(queues)` (serve boot minus HTTP listener), reused by both `serve` and `worker`; must avoid a duplicate control surface.
- Derivation of the distinct registered-queue set for the "all registered queues" default.
- Test construction for SC#2 (two replicas, no double-process — atomic claim already guarantees) and SC#3 (fault-domain isolation).

## Deferred Ideas

- Deploy `workers:` scaffolder emission (extend `[package.metadata.ferro.deploy]` with `workers`) — deploy-scaffolder line.
- Multi-replica metrics export + `DB_MAX_CONNECTIONS` × replicas / PgBouncer guidance.
- Autonomous machine lifecycle / scale-to-zero — milestone 2.0 non-goal.
