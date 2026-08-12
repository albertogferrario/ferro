# Offload — Work Distribution via Service-Trait Methods

Date: 2026-06-24
Status: Design (pre-milestone anchor)

## Problem

Ferro applications need to move work off the request path and spread it across
worker capacity that scales independently of the web tier: report generation,
imports, media processing, model inference, fan-out notifications. Today this
requires hand-authoring a `ferro-queue` Job — a separate struct, a manual
payload, an enqueue call — and the queue is fire-and-forget, so there is no
typed path back to a result. The work is declared twice (once as logic, once as
a Job wrapper) and the return value is lost.

This design adds a single declaration that turns a service-trait method into a
distributable unit of work, reusing the existing queue, projection, and
broadcast infrastructure rather than introducing a parallel mechanism.

## Goal

A method on a `#[service]` trait can be marked `#[offload]`. The framework then:

1. Generates the `ferro-queue` Job and its serializable payload from the method
   signature — the trait stays the single source of truth.
2. Executes the method on a worker fleet instead of in the request path.
3. Returns control to the caller immediately (the request does not block on the
   result).
4. Delivers the result through Ferro's existing read-model + streaming path:
   the worker writes a `ferro-projection` snapshot, whose delta streams to the
   client over `ferro-broadcast`.

The same trait remains callable in-process for code that wants the synchronous
local path; `#[offload]` governs the distributed path.

## Non-Goals

- **Synchronous, request-path, cross-machine reads.** This design does not make
  a service call transparently block on a result computed elsewhere. A handler
  that must obtain a value from another service before responding should make an
  explicit call (e.g. an HTTP client to an external boundary). Hiding a network
  round-trip behind an in-process method signature is explicitly rejected (see
  Alternatives).
- **A general service mesh / synchronous RPC layer.**
- **Data-tier scaling** (sharding, replicas, connection pooling) — orthogonal.
- **Autonomous machine lifecycle / scale-to-zero.** This milestone ships a
  horizontally scalable worker run at N replicas, managed externally (by an
  operator, a platform, or a cluster scheduler). The framework deciding *when*
  to wake or sleep machines is cost-optimization, not capacity, and is deferred
  (see Future direction).

## Design

### Authoring surface

```rust
#[service(impl = ReportBuilder)]
#[async_trait]
pub trait Reports: Send + Sync {
    #[offload]
    async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report;
}
```

`#[offload]` constrains the method to a serializable contract. Its parameters
and return type must implement `Serialize` + `DeserializeOwned`. This is checked
at compile time. The constraint is load-bearing in two ways:

- It is the wire contract for the queue payload and the result snapshot.
- It is the **isolation boundary**: a method that cannot leak non-serializable
  internals across the offload boundary is, by construction, a sealed unit. The
  same requirement that lets work move is the requirement that keeps the module
  isolated — one constraint, both properties.

### Result path (fire-and-forward)

The caller does not wait for the result. Offloading returns a handle that
identifies where the result will land:

```
offload → queue → worker runs the method → worker writes a projection snapshot
        → projection delta streams to the subscribed client (broadcast)
```

This is the scale-native shape: at high concurrency, blocking a request while
awaiting a worker holds connections and does not scale. Returning immediately
and streaming the result when ready does. The handle is the projection key the
client subscribes to.

### Prerequisite: multi-replica broadcast

The result path above assumes a delta published by a worker reaches a client
regardless of which process holds that client's socket. `ferro-broadcast` is
currently an in-process hub (its only transport dependencies are `tokio` and
`tokio-tungstenite`), so a delta published in one process is not observed by
subscribers attached to another. At a single replica this is invisible; at N web
replicas a client subscribed on replica B never receives a result written by a
worker and published on replica A.

The offload result path therefore depends on `ferro-broadcast` gaining a shared
fan-out transport (a Redis pub/sub channel, Postgres `LISTEN`/`NOTIFY`, or an
equivalent bus), selected by configuration with the in-process hub retained as
the default for single-node and development use. This work must land before the
delta-streaming step; without it the fire-and-forward promise holds only at one
replica.

### Scaling model

Capacity scales by running more workers. The worker is a deployable consumer
process: **the application's own binary under a `worker` subcommand**, run at N
replicas against the shared queue. N is managed externally (an operator, a
platform, or a cluster scheduler); the framework does not decide it.

The worker cannot be a subcommand of the `ferro` CLI binary. Offloaded methods
and their job handlers are defined in the application crate and registered
through `WorkerLoop::from_registry`; a framework binary does not link them and
so cannot execute them. The application binary is already a subcommand
dispatcher (`serve`, `db:migrate`, `schedule:work`), so `worker` follows an
established pattern rather than introducing a second process model. The
framework supplies the runtime — an entrypoint equivalent to the `serve` boot
path without the HTTP listener, establishing the database, projection, and
broadcast context the result path requires — and the scaffolded `main.rs` gains
a `worker` arm that calls it.

A **worker class is its set of queues**, selected by `--queue`, which populates
`WorkerConfig.queues`. No separate class concept is introduced. Each class is an
independent fault domain: a saturated `media` class does not starve `default`.

`serve` retains its in-process `WorkerLoop`, enabled by default, so a single
binary continues to drain its own queue for development and single-node
deployments. A new `serve --no-worker` flag disables it. The documented split:
one process (`serve`) for development and single-node; for scale-out, web
replicas running `serve --no-worker` alongside worker replicas running
`worker --queue <class>`, so the web tier does not also consume the queue.

Deployment scaffolding follows from this. The current `.do/app.yaml` generator
derives its `workers:` block from separately declared `[[bin]]` targets, a shape
the single-binary model does not produce, so offload workers would never be
emitted. Worker components are instead declared through deploy metadata (a
`worker_queues` / `workers` key under `[package.metadata.ferro.deploy]`): the
scaffolder emits one worker component per class, each sharing the web image and
differing only in its run command (`<bin> worker --queue <class>`), with the web
service running `serve --no-worker`. Detection of genuinely separate binaries
remains for applications that declare them.

This is the distinction that matters for serving many users: the *capability* to
absorb growing background load comes from the ability to run more workers, not
from the framework deciding when to run them. A fixed fleet of workers serves
high load; autonomously sizing that fleet is cost-optimization layered on top
(see Future direction), not capacity.

Each worker class is an independent fault domain — a saturated media-processing
class does not affect the web tier or an unrelated class — which delivers the
operational properties associated with service decomposition (independent
scaling, blast-radius containment) without the latency and availability cost of
synchronous decomposition.

### Introspection

Because the trait is the single source of truth, an offloadable method is
introspectable through `ferro-mcp` (`list_services`) — the same trait is the
in-process contract, the wire payload schema, and the agent-readable spec.

## Alternatives considered

- **Synchronous location-transparent RPC** (Service Weaver / Orleans lineage):
  generate a proxy + dispatcher so a trait call is transparently local or
  remote, chosen by deployment config. Rejected: location transparency is a
  leaky abstraction (latency, partial failure, and concurrency cannot be hidden
  behind a method call), and synchronous internal fan-out multiplies both
  latency and unavailability at scale. It tends toward a distributed monolith —
  the costs of distribution without the isolation benefit.
- **Message-passing actors**: services become actors with mailboxes. Rejected:
  it replaces "call a trait method" with "send a message," a paradigm shift that
  enlarges the core mental model.
- **Plain manual `ferro-queue` Jobs** (status quo): functional but declares the
  work twice and provides no typed result path. This design is the ergonomic and
  result-bearing layer over exactly this mechanism.

## Phase decomposition

The numbered items below map 1:1 to phases 244–249. The multi-replica broadcast
prerequisite described above is additional work that must be scheduled before
item 4 (delta streaming); it is not covered by any of the six.

1. **`#[offload]` macro → Job + payload generation.** Mark a service-trait method
   offloadable; generate the `ferro-queue` Job and its serializable payload from
   the method signature.
2. **Typed result handle + compile-time serializable-contract enforcement.**
   Return a handle from the offload call; reject non-serializable parameter and
   return types at compile time with a clear diagnostic.
3. **Result → read-model integration.** Worker writes the method's return value
   into a `ferro-projection` snapshot keyed by the handle.
4. **Read-model delta → broadcast streaming.** Stream the snapshot delta to the
   subscribed client over `ferro-broadcast`; document the subscribe/await
   client pattern.
5. **Deployable worker runtime.** A framework-provided worker entrypoint (the
   `serve` boot path without the HTTP listener) plus a scaffolded `worker`
   subcommand on the application binary, invoked as `<bin> worker --queue <class>`
   and runnable at N replicas against the shared queue. `serve` keeps its
   in-process worker by default and gains `--no-worker` for scale-out
   deployments. Deploy scaffolding emits one worker component per class from
   deploy metadata. No autoscaler; capacity scales by running more workers.
   Independent fault domain per class.
6. **`ferro-mcp` introspection + docs.** Surface offloadable methods through
   `list_services`; document the authoring surface, the result path, and the
   non-goals.

## Testing

- Macro expansion (trybuild): a valid `#[offload]` method generates the Job and
  payload; a non-serializable signature fails to compile with the intended
  message.
- Round-trip: offloaded method runs on a worker, writes the projection snapshot,
  and the delta is observed on the broadcast channel for the handle key.
- Fault isolation: a worker started with `--queue media` drains `media` and
  leaves `default` untouched; saturating one class does not block another.
- Run modes: `serve` starts an in-process worker, `serve --no-worker` starts
  none.
- Cross-process delta: a snapshot written in one process is observed by a
  subscriber attached to a second process (the multi-replica broadcast
  prerequisite); with the in-process transport this test is single-process only.
- Deploy scaffolding: deploy metadata declaring two worker classes produces a
  web component running `serve --no-worker` and one worker component per class
  with the expected run command.

## Honest limitations

- No synchronous cross-machine request-path reads (see Non-Goals).
- Result latency is worker-scheduling-bound; this path suits deferred-result
  work, not sub-second interactive computation that must complete before the
  response.
- Queue-backend throughput bounds total offload rate; very high rates require
  partitioning the backend.

## Future direction (2.0+)

Deferred — not built in this milestone. The queue-consumer worker model above
does not preclude any of it:

- **Elastic scale-to-zero.** Delegate autoscaling to the deployment platform:
  derive a KEDA `ScaledObject` from queue depth so worker replicas scale 0→N
  (and back to zero when idle) on Kubernetes. The framework emits the manifest;
  the orchestrator actuates.
- **Warm-start workers.** Checkpoint/restore (CRIU-style, as in Lambda
  SnapStart) to make scale-from-zero cold starts sub-second rather than
  pod-boot-bound.
- **Non-Kubernetes actuation.** A `WorkerFleetProvider` port with adapters (e.g.
  Nomad for process-level scheduling) for environments without a container
  orchestrator.
- **Lighter execution unit.** WASM/WASI isolates as a fast-scheduling,
  sub-process worker unit.

These are operational-efficiency and elasticity layers — they reduce idle cost
and ops toil. They do not add the capability to serve many users, which the
deployable worker already provides.
