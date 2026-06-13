# Phase 224 — Cache Invalidator Metrics + Introspection — CONTEXT

## One-line scope

Make cache invalidation observable: counts, timings, and a read-only registry of what's wired to what.

## Why now (deferred from Phase 222)

Phase 222 shipped the registration surface with a single visibility primitive: `tracing::warn!` on per-tag flush failure. That's enough for incident response — an operator who notices stale reads can grep logs and find the failed flush. It's not enough for SLO dashboards, capacity planning, or "is the cache layer healthy?"

The Phase 222 honest framing called this out:

> **No metrics / introspection.** Can't query "how many invalidations fired in the last hour?" or "what invalidators are registered for `BookingCreated`?". Just `tracing::warn!` on failure.

## Surface to add

- **Structured tracing on every fire** — `info!` event with `event_name`, `tags_flushed`, `duration_us` fields on success; `warn!` with `error` field on failure (preserves the Phase 222 behaviour as a special case).
- **Optional `metrics` feature flag** — wires the same counts/timings into the `metrics` crate (counters: `ferro_cache.invalidations.fired`, `ferro_cache.invalidations.failed`; histogram: `ferro_cache.invalidations.duration`). Operators with a Prometheus/OTLP exporter get them for free.
- **Introspection API** — `ferro_cache::list_invalidators_for::<E>() -> Vec<InvalidatorInfo>` returns the count + last-fire timestamp of registered invalidators per event type. Counts only — Rust closures can't be introspected for body content.

## Locked decisions (to refine in discuss-phase)

- Tracing fields are stable contract: external dashboards key off them. Renaming = breaking change.
- `metrics` is a feature flag, not a default dependency. Consumers that don't enable it pay zero (no transitive `metrics` crate pull).
- Introspection API is read-only, lock-cheap (one read on the dispatcher's internal `RwLock<HashMap<TypeId, ...>>`).

## Open decisions

| # | Question | Lean | Alternatives |
|---|---|---|---|
| D-01 | Metric naming convention | Dotted (`ferro_cache.invalidations.fired`) | Snake (`ferro_cache_invalidations_fired`) — depends on operator's exporter conventions |
| D-02 | Histogram buckets for `duration_us` | metrics-crate defaults | Custom buckets (10us, 100us, 1ms, 10ms, 100ms, 1s) — better resolution for our typical workload |
| D-03 | Should introspection return closure metadata (registered call site, registration time)? | No — closures can't be introspected; capture file:line at registration via `#[track_caller]` if needed | Yes — adds value, requires Phase 222 surface change |
| D-04 | Per-tag granularity in metrics? | No — aggregate per event type; per-tag explodes cardinality | Yes — sample fixed-cardinality tag prefixes |
| D-05 | Failure-rate alerting opinion | Out of scope (operator's exporter handles it) | Expose a `FailureRateGuard` that emits at threshold |

Default leans: D-01 dotted, D-02 custom buckets, D-03 file:line via `#[track_caller]`, D-04 aggregate, D-05 out of scope.

## Anti-scope

- No metrics endpoint server. ferro-cache produces the metrics; the operator's exporter (`metrics-exporter-prometheus`, etc.) serves them.
- No alerting. Operator-side concern.
- No log aggregation. `tracing` is the structured event stream; the operator's subscriber chain decides where it goes.

## Provenance

Named gap in Phase 222 honest-framing review. Operator-acknowledged deferral 2026-06-13.

## Next step

Wait for consumer demand (operator asking for SLO dashboards on cache hit-rate / invalidation-rate). When demand lands: `/gsd-discuss-phase 224`, lock D-01..D-05, plan, execute.
