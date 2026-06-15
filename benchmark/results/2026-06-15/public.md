# Ferro vs Laravel — micro-endpoints (2026-06-15)

Both apps implement the same four-endpoint contract (conformance 4/4 each).
Laravel is measured under two production serving modes: **php-fpm 8.3 + nginx**
(`pm=static`, `max_children=20`, opcache on) and **Laravel Octane + RoadRunner**
(v2025.1.14, 16 workers, no nginx). Ferro runs as a multi-threaded tokio `--release` build.

## Static compression

| Metric | Laravel | Ferro | Ferro advantage |
|---|---|---|---|
| Lines of code | 1,448 | 344 | 4.2× fewer |
| Source tokens | 8,976 | 1,158 | 7.7× fewer |

Same contract, idiomatic code on both sides.

## Raw performance (requests/sec)

| Endpoint | Laravel fpm | Laravel Octane | Ferro | fpm→Octane | Octane→Ferro |
|---|---|---|---|---|---|
| /json | 620 | 1,393 | 211,704 | 2.2× | 152× |
| /db | 487 | 1,706 | 11,001 | 3.5× | 6.4× |
| /queries?n=20 | 451 | 1,092 | 1,043 | 2.4× | ~1× |
| /updates?n=20 | 239 | 399 | 486 | 1.7× | 1.2× |

Octane (RoadRunner) lifts the PHP numbers consistently across all routes — 1.7–3.5× over
fpm — by eliminating per-request PHP bootstrap and keeping persistent DB connections.
The lift is largest on `/db` (single read, connection overhead dominates) and smallest on
`/updates` (write-bound, Postgres serialization dominates).

At `/queries` and `/updates` (20 DB round-trips each), Ferro and Octane converge:
both are bounded by the same Postgres instance. The Ferro edge on CPU-bound routes
(`/json`, `/db`) remains large because Rust compiled dispatch and the tokio async runtime
operate at a different scale than any PHP serving mode.

Honest caveats: these are micro-endpoints with no middleware, auth, or session handling;
the pool sizes (fpm `max_children=20`, Octane `workers=16`) are comparable defensible
choices, not tuned-to-win values; both PHP variants share a Postgres container with Ferro.

---

Measured on Apple M1 Pro (8 cores, 16 GB, macOS Darwin 23.6.0), oha 1.9.0,
c=256, 30 s timed after 5 s warmup, PostgreSQL 16.4 shared by all three apps.
Full methodology and all latency percentiles: `internal.md`, `NOTES.md`.
