# Run notes — 2026-06-15 (re-run with production Laravel server)

This directory holds two runs collapsed into one date:

- **Run 1A** (initial): Laravel served by `php artisan serve` (single-process dev server).
  Numbers were not representative — see the git history for those values.
- **Run 1B** (this file): Laravel served by **php-fpm 8.3 + nginx** (supervisord).
  Pool: `pm=static`, `pm.max_children=20`, `pm.max_requests=1000`, opcache on
  (`validate_timestamps=0`). The single-worker bottleneck is resolved.

All JSON, internal.md, and public.md in this directory reflect Run 1B only.

## Static compression — fair and citable

Both apps satisfy the identical contract (conformance: 4/4 each). The line/token counts
are a like-for-like comparison of idiomatic code:

Ferro 344 LoC / 1158 tokens vs Laravel 1448 / 8976
(≈4.2× fewer lines, ≈7.7× fewer tokens).

The Laravel Dockerfile is larger in 1B because it describes the nginx+fpm serving stack;
this is an honest cost of the production configuration and is included symmetrically on
both sides.

## Raw performance — now a representative comparison

With php-fpm + nginx:

| Endpoint    | Laravel rps | Ferro rps  | p50 Laravel | p50 Ferro |
|-------------|-------------|------------|-------------|-----------|
| /json       | 620         | 211,704    | 395 ms      | 0.9 ms    |
| /db         | 487         | 11,001     | 508 ms      | 12 ms     |
| /queries    | 451         | 1,043      | 548 ms      | 202 ms    |
| /updates    | 239         | 486        | 1,062 ms    | 525 ms    |

The Rust/tokio vs PHP/fpm throughput gap is real. The remaining honest caveats:

1. **Micro-endpoints only.** These four routes have no middleware stack, no session
   handling, no auth — they isolate the raw framework dispatch and DB path. A real
   application will narrow the gap on the CPU-bound dimension and widen it on the
   I/O-bound dimension depending on workload.

2. **Shared Postgres.** Both apps talk to the same container. Under `/db`-class load
   the DB is the shared constraint; the ratio reflects framework overhead on top of
   equal DB time.

3. **Laravel Octane not tested.** Swoole or RoadRunner would push Laravel's numbers
   materially higher on `/json` (no-DB). Octane is a documented Phase 3 addition.
   The gap at `/queries` and `/updates` is DB-bound and Octane narrows it less.

4. **pm.max_children=20.** A defensible choice for 8 cores under sustained c=256.
   Higher values (e.g. 40) would increase Laravel's `/json` rps further at the cost of
   more RAM per worker; we chose a standard, not a tuned-to-win, value.

## Tooling / environment

oha 1.9.0, tokei 12.1.2, PostgreSQL 16.4. Apple M1 Pro, 8 cores, 16 GB, macOS
Darwin 23.6.0. c=256, 30s timed run after 5s warmup. See `meta.json`.
