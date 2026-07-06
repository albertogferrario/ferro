# Phase 223 — Redis Pub/Sub Cross-Replica Invalidation Channel — CONTEXT

## One-line scope

Make `register_invalidator` invalidations visible to every replica of a multi-instance deploy, not just the dispatching replica.

## Why now (deferred from Phase 222)

Phase 222 shipped the registration surface and the local-flush behaviour. The honest framing called out three gaps; this phase closes the one that matters for multi-replica deploys:

> **Multi-process Redis fanout missing.** `RedisStore::tag_flush` invalidates the local Redis state — fine for one-Redis-many-apps, but if a consumer has multiple Redis clusters or wants pub/sub-driven cross-replica invalidation, this bridge doesn't do it.

Today: a `BookingCreated` dispatched on replica A flushes A's local `MemoryStore` (or A's view of `RedisStore`). Replica B's cache view is untouched and may serve stale availability until B's own TTL expires.

## Surface to add

- **Opt-in publish path** — extend `register_invalidator` (or add `register_invalidator_with_pubsub`) so the registered listener, in addition to the local flush, publishes the tag set to a Redis pub/sub channel.
- **Receiver loop** — a background task on every replica subscribes to the channel and runs the local flush when a payload arrives.
- **Origin filtering** — pub/sub payload carries an `origin: "replica-id"` field; receivers skip flushing their own publish (it already flushed locally).
- **Failure isolation** — receiver loop survives Redis disconnects (reconnect with exponential backoff). Data-plane reads/writes are not affected by receiver-loop failures.

## Locked decisions (to refine in discuss-phase)

- Channel name default: `ferro-cache:invalidations` (overridable via env or builder).
- Payload format: JSON `{ "tags": ["business:1:product:7"], "origin": "<replica-id>" }`. Simple, debuggable; cost of one JSON parse per invalidation is negligible vs the network round-trip.
- Subscription lifecycle: one receiver per `Cache` instance; spawned when the consumer wires the bridge; cancelled on `Cache` drop.
- Backwards compatibility: Phase 222 single-process consumers keep working with zero config. The pub/sub path is purely additive and opt-in.

## Open decisions

| # | Question | Lean | Alternatives |
|---|---|---|---|
| D-01 | Channel name configuration | env `FERRO_CACHE_PUBSUB_CHANNEL`, default `ferro-cache:invalidations` | hardcoded; builder-only |
| D-02 | Replica ID source | env `FERRO_CACHE_REPLICA_ID` falling back to `hostname:pid` | auto-generated UUID per process; require explicit ID |
| D-03 | Receiver task ownership | spawned by `Cache` via `tokio::spawn` (lives until cache dropped) | exposed `start_receiver(&cache)` for explicit lifecycle control |
| D-04 | What about non-Redis stores? | Pub/sub path is `RedisStore`-only; `MemoryStore` consumers must already be single-process | abstract `BroadcastChannel` trait — over-engineered for v1 |
| D-05 | Failure on subscribe error at boot | log + continue (degrade to local-only) | hard-fail at boot |

Default leans: env-based config (D-01, D-02), cache-owned receiver (D-03), Redis-only (D-04), graceful degrade (D-05).

## Anti-scope

- No multi-region / cross-cluster fanout. Single Redis instance assumed; if the consumer runs Redis cluster, they wire one bridge per cluster.
- No guaranteed delivery. Pub/sub is fire-and-forget; if a receiver is down, its local cache may serve stale until its TTL expires. For consumers that need stronger guarantees, the answer is shorter TTLs or a streams-backed channel — separate phase.
- No payload signing / auth. The Redis channel is assumed to be within the same trust boundary as the cache itself.

## Provenance

Named gap in Phase 222 honest-framing review. Operator-acknowledged deferral 2026-06-13.

## Next step

Wait for consumer demand (multi-replica gestiscilo deploy, or another consumer asking for it). When demand lands: `/gsd-discuss-phase 223`, lock D-01..D-05, plan, execute.
