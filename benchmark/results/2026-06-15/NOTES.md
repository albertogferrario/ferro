# Run notes — 2026-06-15 (read before citing any number)

This is the first end-to-end harness run (Phase 1A). Its primary purpose was to prove the
pipeline works (build → containerize → conformance → load-test → static-count → report), and
it does. Treat the numbers with the following honesty caveats.

## Static compression — fair and citable
Both apps satisfy the identical contract (conformance: 4/4 each). The line/file/token counts
are a like-for-like comparison of idiomatic code and are a real result:
Ferro 344 LoC / 14 files / 1158 tokens vs Laravel 1427 / 44 / 8874 (≈4× fewer lines,
≈7.7× fewer tokens).

## Raw performance — NOT a fair framework comparison in this run
The Laravel app is served by stock `php artisan serve`, a single-process development server.
The evidence it is the bottleneck, not the framework:
- Laravel p99 latency is 5,142 ms (/json) and 7,849 ms (/db) at c=256 — requests are queuing
  ~5–8 seconds behind a single worker, not being computed slowly.
- 98 rps for a plaintext JSON response is far below any production Laravel stack; php-fpm/nginx
  or Laravel Octane (Swoole/RoadRunner) routinely serve thousands of rps for the same endpoint.

Ferro, by contrast, is a multi-threaded tokio `--release` build. So the rps ratios
(1517× /json, 225× /db, etc.) measure "release tokio server vs single-process PHP dev server,"
which is a foregone conclusion and must not be cited as a Ferro-vs-Laravel performance result.

## Required before raw-perf is publishable (Phase 2)
Re-run the Laravel side under a production-representative server — php-fpm + nginx, or Laravel
Octane — at the same concurrency, and re-measure. The design already scopes the Octane variant
to Phase 2. Until then, the raw-perf JSON in this directory is harness-validation evidence
only, not a benchmark result.

## Tooling / environment
oha 1.9.0 (bumped from the planned 1.4.7 — `--output-format json` was introduced in 1.9.0),
tokei 12.1.2, PostgreSQL 16.4 shared by both apps. Apple M1 Pro, 8 cores, 16 GB, macOS
Darwin 23.6.0. c=256, 30s timed run after 5s warmup. See `meta.json`.
