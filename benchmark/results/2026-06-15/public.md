# Ferro vs Laravel — micro-endpoints (2026-06-15)

Both apps implement the same four-endpoint contract (conformance 4/4 each).
Laravel is served by **php-fpm 8.3 + nginx** (`pm=static`, `max_children=20`,
opcache on) — a production-representative stack, not the dev server used in
the initial run. Ferro runs as a multi-threaded tokio `--release` build.

## Static compression

| Metric | Laravel | Ferro | Ferro advantage |
|---|---|---|---|
| Lines of code | 1,448 | 344 | 4.2× fewer |
| Source tokens | 8,976 | 1,158 | 7.7× fewer |

Same contract, idiomatic code on both sides.

## Raw performance (requests/sec)

| Endpoint | Laravel | Ferro | ratio |
|---|---|---|---|
| /json | 620 | 211,704 | 341× |
| /db | 487 | 11,001 | 23× |
| /queries?n=20 | 451 | 1,043 | 2.3× |
| /updates?n=20 | 239 | 486 | 2.0× |

The throughput gap reflects Rust compiled vs PHP interpreted dispatch and
memory model. It narrows sharply when the workload is DB-bound: at
`/queries` and `/updates` (20 round-trips each) the ratio drops to 2–2.3×
because both apps wait on the same Postgres instance.

Honest caveats: these are micro-endpoints with no middleware, auth, or
session handling; Laravel Octane (Swoole/RoadRunner) would raise the
PHP numbers further on CPU-bound routes; the pool size (`max_children=20`)
is a defensible standard choice, not a tuned-to-win value.

---

Measured on Apple M1 Pro (8 cores, 16 GB, macOS Darwin 23.6.0), oha 1.9.0,
c=256, 30 s timed after 5 s warmup, PostgreSQL 16.4 shared by both apps.
Full methodology and all latency percentiles: `internal.md`, `NOTES.md`.
