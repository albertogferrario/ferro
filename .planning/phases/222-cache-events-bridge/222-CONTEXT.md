# Phase 222 — Cache-Events Bridge — CONTEXT

## One-line scope

Add an event-driven invalidation surface to `ferro-cache` so consumers register `(event_type → tag flush)` once at boot instead of writing per-app `impl Listener<E>` glue that knows about the cache.

## Why now

Surfaced 2026-06-13 in gestiscilo's availability-perf investigation. The per-request structural fix (gestiscilo `booking_api::availability` rewrite — already shipped at `gestiscilo@7983043a`) cut a single request from ~1k DB queries to ~5 by hoisting prefetches out of a per-slot loop. The remaining gap is cross-request redundancy: every browser open rebuilds the same window from scratch. The natural next layer is a short-TTL read-through cache invalidated by domain events — and the operator's framing (cross-cutting primitive → ferro, concrete cache-key + invalidations → gestiscilo) maps cleanly to the existing `feedback_ferro_first_primitives` discipline.

`ferro-cache` already ships `Cache`, `TaggedCache`, `MemoryStore`, `RedisStore`. `ferro-events` already ships `Event`, `Listener`, `EventDispatcher`, `dispatch`. The missing piece is the seam between them.

## Locked decisions (ferro-side)

- Implementation crate: `ferro-cache` (additive). `ferro-events` becomes a non-optional workspace dep of `ferro-cache` — both are already taken by every ferro app that needs either, so adding the link costs nothing.
- Tag scheme: stays consumer-defined (strings). ferro does not impose `tenant:N:resource:M` formatting — too domain-specific.
- Listener failure semantics: best-effort + logged. A degraded cache MUST NOT brick the original write path (`EventDispatcher::dispatch` keeps its success path even if the cache store is unavailable).
- No new crate. No feature flag (the bridge is small + always-on for consumers that opt into both crates).
- Tag-flush API uses the existing `TaggedCache::flush(tags).await` — no new flush primitive needed.

## Open decisions (lock during /gsd-discuss-phase 222)

| # | Question | Default lean | Alternatives |
|---|---|---|---|
| D-01 | `keys()` return type | `Vec<String>` (simplest, most ergonomic) | `impl IntoIterator<Item = String>` (more flexible, slightly more type-machinery) |
| D-02 | Listener execution model | Synchronous in-dispatch (cache flush completes before next listener runs) | Queued (`ShouldQueue` marker) — but TTL caches are best invalidated synchronously so the next read is guaranteed fresh |
| D-03 | Single vs multi-invalidator | Allow multiple `register_invalidator` calls per event type — all run | Restrict to one per event type — simpler but constrains future composition |
| D-04 | Closure type for the convenience helper | `Fn(&E) -> Vec<String> + Send + Sync + 'static` (clone-safe across listener invocations) | `FnOnce` — wrong, listeners run repeatedly |
| D-05 | Where the `register_invalidator` helper attaches the cache | Closure captures `Arc<Cache>` clone (simplest) | A dedicated `InvalidatorRegistry` type the consumer threads through boot — over-engineered for v1 |

Lean defaults: D-01 `Vec<String>`, D-02 synchronous, D-03 multi, D-04 `Fn`, D-05 closure-captured `Arc<Cache>`.

## Anti-scope

- No Redis pub/sub. Tag flushes go through `TaggedCache::flush` exactly as today; if the store is `RedisStore`, the existing `MULTI/DEL` path handles it. Cross-process invalidation for Redis-backed caches is a separate phase (would need a fanout channel).
- No metrics / observability layer in v1 (a simple `tracing::warn!` on listener failure is enough; the operator can add `tracing` subscribers if they want SLOs).
- No "cache-aware ServiceDef" — projections stay out of this. Consumers wire `register_invalidator` at boot in their app's bootstrap code, not in `ServiceDef`.

## Success criteria (mirrors ROADMAP §222)

1. `ferro_cache::CacheInvalidator<E: Event>` is a trait with `fn keys(&self, event: &E) -> Vec<String>`; `register_invalidator::<E, F>(cache, key_fn)` registers a blanket impl using a closure.
2. Insert tagged entry → dispatch event → assert `get` returns `None` (unit test).
3. Multiple invalidators per event type all run; order unspecified but documented.
4. Listener failure is logged and swallowed; `EventDispatcher::dispatch` does not propagate the error.
5. Doc example shows the full end-to-end pattern.
6. CHANGELOG + version bump (0.2.58 → 0.2.59).
7. `cargo test -p ferro-cache` exits 0 against both `MemoryStore` and `RedisStore` (feature-gated).

## Consumer-side companion

Pairs with gestiscilo Phase 210 (mounts a `TaggedCache` on `GET /api/v1/businesses/{slug}/bookings/availability`, registers invalidators for `BookingCreated` / `BookingCancelled` / `ClosedDayChanged` / `InventoryUnitStatusChanged`). Gestiscilo phase blocks on ferro 0.2.59 publish.

## Next step

`/gsd-discuss-phase 222` — lock D-01..D-05, then plan + execute.
