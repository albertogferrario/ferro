# Phase 248: Deployable `ferro worker` runtime - Context

**Gathered:** 2026-08-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Make background capacity horizontally scalable (OFFLOAD-05): a deployable consumer
process — the application's **own binary** under a `worker` subcommand with a job-queue
selector — runnable at N replicas against the shared queue, so a Ferro app absorbs growing
background load by adding workers, each queue an independent fault domain. No
framework-managed autoscaling; N is external.

**Folded in — WR-01** (from the v16.4 milestone audit): wire the shared broadcast transport
from configuration at framework boot so multi-replica offload delivery (OFFLOAD-04) actually
works and the Phase 246.1 multi-replica UAT is unblocked. Currently `BroadcastConfig` reads
`transport_redis_url` but nothing constructs a `RedisTransport` from it.

Discussion clarifies HOW to implement within this boundary — new capabilities belong in
other phases.

</domain>

<decisions>
## Implementation Decisions

### Worker CLI surface
- **D-01:** The worker is the **application's own binary** under a `worker` subcommand
  (`<app-bin> worker`), NOT a `ferro` CLI subcommand — offload job handlers live in the app
  crate and are registered via `WorkerLoop::from_registry`; the `ferro` binary does not link
  them. (Carried forward from the offload design decisions; reconciles the ROADMAP SC#1
  shorthand `ferro worker`.)
- **D-02:** Queue selector is `--queue <name>`, **repeatable**
  (`worker --queue reports --queue emails`), mapping directly to the existing
  `WorkerConfig { queues: Vec<String> }` / `WorkerConfig::new(queues)`. No `--class` flag and
  no class→queues registry — a worker class *is* its queue set (no new concept).
- **D-03:** `worker` with **no** `--queue` consumes **all registered queues**. Requires
  deriving the distinct queue set from the job registry (small addition — the registry today
  exposes registered jobs, not a distinct-queue-name set). Operators opt into isolation by
  passing `--queue`.

### Fault-domain routing
- **D-04:** An `#[offload]` method declares its queue via an attribute argument
  `#[offload(queue = "name")]`, defaulting to `default` when omitted. The queue lives on the
  method declaration (single source of truth, `ferro-mcp` introspectable), consistent with the
  derive-everything-from-the-declaration principle — not only at a runtime `.on_queue()` call
  site.
- **D-05:** `serve`'s in-process worker consumes **all registered queues** (not just
  `default`), so every offloaded method runs under a bare `serve` in dev regardless of its
  declared queue (preserves the single-binary "just works" promise, Phase 185 D-01). Scale-out
  deployments run `serve --no-worker` web replicas + dedicated `<app-bin> worker --queue X`
  replicas. (`serve --no-worker` itself carried forward from the offload design decisions.)

### WR-01 — shared broadcast transport wiring
- **D-06:** The **framework** owns transport wiring, not each app. At boot (inside
  `Broadcaster::with_config` or the shared serve/worker boot step), when
  `BroadcastConfig.transport_redis_url` is set, construct the `RedisTransport` and attach it via
  the existing `Broadcaster::with_transport(...)`. Every ferro app gets multi-replica delivery
  when the URL is configured — no per-app hand-wiring (project-agnostic-crates + no-duplicate-
  control-surface). The sample app's `bootstrap.rs` stops hand-assembling this.
- **D-07:** `RedisTransport` is feature-gated (Phase 246.1-02, optional `redis` dep). When the
  feature is **off** but a transport URL is set, emit a warning and fall back to the in-process
  hub — no hard failure. With no URL set, behaviour is unchanged (in-process hub default).
- **Already decided (not re-litigated):** the env key is `BROADCAST_REDIS_URL` with a
  `REDIS_URL` fallback — `BroadcastConfig::from_env()` already reads it. No new env var.

### Deploy scaffolder scope
- **D-08:** Deploy `workers:` component emission is **deferred**. Phase 248 = worker runtime +
  queue routing + WR-01 wiring + the SC#2/#3 verification tests. "N is external" stays
  documented (Phase 249 owns the scaling docs). Extending `[package.metadata.ferro.deploy]`
  with a `workers` array and emitting one deploy component per class from `do:init` /
  `docker:init` is a separate deploy-scaffolder concern (the 122.x deploy line), plannable
  independently. See Deferred Ideas.

### Claude's Discretion
- The exact factoring of the shared boot path — extract a reusable framework entry point
  (e.g. `run_worker(queues)`) that is the `serve` boot minus the HTTP listener (still inits DB
  + queue + projection + broadcast + offload hooks), reused by both `serve` and `worker`.
  MUST avoid a duplicate control surface (do not fork the boot logic).
- How the distinct registered-queue set is derived for the "all registered queues" default
  (D-03, D-05).
- Test construction for SC#2 (two replicas split work without double-processing — the atomic
  claim already guarantees this, Phase 185) and SC#3 (saturating one queue does not stall an
  unrelated one — fault-domain isolation across separate worker processes / disjoint queue
  sets).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Offload milestone spec & phase definition
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` — the v16.4 anchor
  spec: worker shape (app binary under `worker`, queue-set = class), scaling model
  (stateless tier + replicable workers + cache + queue), rejected alternatives, and the parked
  2.0 elastic direction (out of scope).
- `.planning/ROADMAP.md` §"Phase 248: Deployable `ferro worker` runtime" — Goal, Depends-on
  (Phase 244), and Success Criteria SC#1–4. Note SC#1's `ferro worker --class` is loose
  shorthand; the decided shape is `<app-bin> worker --queue` (D-01/D-02).
- `.planning/REQUIREMENTS.md` — OFFLOAD-05 (deployable scalable worker) and the milestone
  scope decision (build the scalable primitive; defer auto-deciding).

### WR-01 origin
- `.planning/v16.4-MILESTONE-AUDIT.md` — the `gaps_found` audit that surfaced WR-01
  (multi-replica transport bootstrap wiring) as un-owned; Phase 248 is its home.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-queue/src/worker.rs` — `WorkerConfig { queues: Vec<String> }`, `WorkerConfig::new(queues)`,
  `WorkerLoop::from_registry(config)`, `.run()`. The loop already handles SIGTERM/Ctrl-C with
  in-flight drain + `requeue_claimed_by` on shutdown; per-queue reaper on `visibility_timeout`.
  The `worker` subcommand builds a `WorkerConfig` from `--queue` and calls `from_registry(...).run()`.
- `ferro-queue` atomic claim + reaper (Phase 185: `FOR UPDATE SKIP LOCKED` / `BEGIN IMMEDIATE`,
  QUEUE-F-02/03) — already provides at-least-once with idempotent claim and crashed-worker
  reaping. Phase 248 **verifies** SC#2/#3 against this; it does not rebuild queue mechanics.
- `framework/src/app.rs:399-456` (`run_server_internal`) — the boot sequence to factor: queue
  init (`Queue::init`), offload-hook registration (broadcaster-aware vs persist-only fallback),
  in-process `WorkerLoop` spawn, then `Server::from_config(router).run()`. The `worker`
  subcommand reuses everything **except** the final HTTP `Server::...run()`.
- `ferro-broadcast/src/config.rs` — `BroadcastConfig.transport_redis_url`, `from_env()` reading
  `BROADCAST_REDIS_URL`/`REDIS_URL`; `ferro-broadcast/src/broadcaster.rs:90`
  `Broadcaster::with_transport(...)` (the attach point for D-06).
- `ferro-cli/src/deploy/bin_detect.rs`, `ferro-cli/src/project.rs` — existing
  `[package.metadata.ferro.deploy]` reader (`web_bin`, `copy_dirs`, `runtime_apt`). Relevant
  only to the **deferred** scaffolder work (D-08), not to 248's build surface.

### Established Patterns
- `app/src/main.rs` clap `Commands` enum: `Serve { no_migrate }`, `db:migrate`/`db:status`/…,
  `schedule:work`/… . Add a bare `Worker { queue: Vec<String> }` runtime command (top-level like
  `serve`), and a `no_worker` flag on `Serve` (D-05).
- Offloaded jobs enqueue through `framework/src/offload.rs::enqueue_and_mark_pending` →
  `Offloadable::offload()`; the job's queue governs which worker drains it (D-04 threads the
  declared queue into the derived Job).

### Integration Points
- `app/src/bootstrap.rs:186` — `Broadcaster::with_config(BroadcastConfig::from_env())`: today it
  reads the transport URL but never attaches a transport. D-06 moves the attach into framework
  boot; the app bootstrap stops owning it.

</code_context>

<specifics>
## Specific Ideas

- Mental model to preserve across the CLI: **default = process everything, opt into isolation
  with `--queue`.** It holds identically for the bare `worker` command (D-03) and the in-process
  `serve` worker (D-05), so operators reason about one rule.
- `#[offload(queue = "reports")]` reads as the operational analog of an intent declaration:
  declare the routing once on the method, and the Job / worker selection / MCP payload schema
  all derive from it.

</specifics>

<deferred>
## Deferred Ideas

- **Deploy `workers:` scaffolder emission** — extend `[package.metadata.ferro.deploy]` with a
  `workers` array and emit one DigitalOcean/Docker worker component per class (shared web image,
  differing run command) from `do:init` / `docker:init`. Deferred out of 248 (D-08); belongs to
  the deploy-scaffolder line of work (cf. ROADMAP 122.x deploy phases).
- **Multi-replica operational guidance** — no OpenTelemetry/Prometheus metrics export in any
  generated manifest; `DB_MAX_CONNECTIONS` defaults to 10 per process (10 × replicas vs a
  Postgres ~100 ceiling → PgBouncer / guidance). Recorded in the offload design decisions as
  smaller follow-on gaps; not part of 248.
- **Autonomous machine lifecycle / scale-to-zero** (KEDA, CRIU, Nomad, WASM isolates) — the
  milestone's explicit 2.0 non-goal; the queue-consumer model does not preclude it.

</deferred>

---

*Phase: 248-deployable-ferro-worker-runtime*
*Context gathered: 2026-08-14*
