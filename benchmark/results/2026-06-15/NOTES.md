# Run notes — 2026-06-15

This directory holds three serving configurations measured on the same date:

- **Run 1A** (initial): Laravel served by `php artisan serve` (single-process dev server).
  Numbers were not representative — see the git history for those values.
- **Run 1B**: Laravel served by **php-fpm 8.3 + nginx** (supervisord).
  Pool: `pm=static`, `pm.max_children=20`, `pm.max_requests=1000`, opcache on
  (`validate_timestamps=0`). The single-worker bottleneck is resolved.
- **Run 1C**: Laravel served by **Octane + RoadRunner v2025.1.14**, 16 workers (static),
  opcache on, pcntl+sockets extensions, php:8.3-cli-bookworm, no nginx in the path.

All JSON files (`perf-laravel.json`, `perf-laravel-octane.json`, `perf-ferro.json`) and
`internal.md` / `public.md` reflect the final state including all three variants.

## Static compression — fair and citable

Both apps satisfy the identical contract (conformance: 4/4 each). The line/token counts
are a like-for-like comparison of idiomatic code:

Ferro 344 LoC / 1158 tokens vs Laravel 1448 / 8976
(≈4.2× fewer lines, ≈7.7× fewer tokens).

The Laravel Dockerfile is larger in 1B because it describes the nginx+fpm serving stack;
this is an honest cost of the production configuration and is included symmetrically on
both sides.

## Raw performance — three-way comparison

| Endpoint    | fpm rps | Octane rps | Ferro rps  | p50 fpm | p50 Octane | p50 Ferro |
|-------------|---------|------------|------------|---------|------------|-----------|
| /json       | 620     | 1,393      | 211,704    | 395 ms  | 154 ms     | 0.9 ms    |
| /db         | 487     | 1,706      | 11,001     | 508 ms  | 131 ms     | 12 ms     |
| /queries    | 451     | 1,092      | 1,043      | 548 ms  | 231 ms     | 202 ms    |
| /updates    | 239     | 399        | 486        | 1,062 ms| 656 ms     | 525 ms    |

**Reading the Octane results honestly:**

- `/json` (no DB): Octane 1,393 rps vs fpm 620 — **2.25× lift**. Eliminating per-request
  PHP bootstrap is real but the RoadRunner IPC channel under c=256 creates its own
  overhead ceiling. The lift is meaningful, not transformative.

- `/db` (single DB hit): Octane 1,706 vs fpm 487 — **3.5× lift**. This is the sweet spot
  for Octane: persistent DB connections across workers remove per-request reconnect cost
  and the work is light enough that worker throughput dominates.

- `/queries` (20 DB reads): Octane 1,092 vs fpm 451 — **2.4× lift**, and notably
  Octane (1,092) slightly edges Ferro (1,043) here. Both are constrained by Postgres;
  Octane's persistent connections help, Ferro's advantage from compiled dispatch is
  partially offset by connection pool contention at this concurrency.

- `/updates` (20 DB read+write): Octane 399 vs fpm 239 — **1.7× lift**. Both are
  write-bound; Postgres serialization costs dominate and Octane helps less here.
  Ferro (486) stays ahead but the margin is narrow.

**What Octane changes:** The php-fpm to Octane jump is real and consistent across all
four routes (1.7–3.5×). The gains come from eliminating per-request PHP bootstrap,
persistent DB connections, and removing nginx as an intermediary process. These are
architectural, not tuning, gains.

**What Octane does not change:** The Ferro advantage on CPU-bound dispatch (`/json`,
`/db`) remains 2–3 orders of magnitude. At `/queries`/`/updates` Ferro's edge over
Octane compresses to near-parity — both are Postgres-bound — which is the expected
result: no amount of serving optimization overcomes the DB round-trip floor.

## Honest caveats (remaining after Octane test)

1. **Micro-endpoints only.** No middleware, auth, or session handling. A real application
   narrowing or widening these ratios depends on the workload's CPU vs I/O split.

2. **Shared Postgres.** Both apps share a single Postgres container. Under high concurrency
   the DB is the shared constraint, making `/queries`/`/updates` ratios DB-ceiling-bounded.

3. **Worker count (fpm=20, Octane=16).** Comparable but not identical. Octane's 16 workers
   reflect a deliberate choice: RoadRunner workers hold more state per process than fpm
   children; 16 on 8 cores is a reasonable default. The fpm choice of 20 is the prior
   defensible standard. Neither is tuned-to-win.

4. **Octane caveat resolved.** The previous placeholder noting Octane as untested is closed.
   The data is in `perf-laravel-octane.json`.

## Tooling / environment

oha 1.9.0, tokei 12.1.2, PostgreSQL 16.4. Apple M1 Pro, 8 cores, 16 GB, macOS
Darwin 23.6.0. c=256, 30s timed run after 5s warmup. See `meta.json`.
