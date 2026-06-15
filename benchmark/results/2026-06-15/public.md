# Ferro vs Laravel — micro-endpoints (2026-06-15)

> First harness run. Read `NOTES.md` before citing any number — the raw-performance
> figures are **not** a fair framework comparison in this run (see below).

## Static compression (a fair, meaningful result)

Both apps implement the identical four-endpoint contract.

| Metric | Laravel | Ferro | Ferro advantage |
|---|---|---|---|
| Lines of code | 1427 | 344 | 4.1× fewer |
| Files | 44 | 14 | 3.1× fewer |
| Source tokens | 8874 | 1158 | 7.7× fewer |

This reflects real authoring compression for this workload and is directly comparable —
same contract, idiomatic code on both sides.

## Raw performance — harness validation only, NOT a Laravel verdict

| Endpoint | Laravel | Ferro | p99 (Laravel) |
|---|---|---|---|
| /json | 98 rps | 148,852 rps | **5,142 ms** |
| /db | 74 rps | 16,571 rps | **7,849 ms** |

The Laravel app runs stock `php artisan serve` — a **single-process development server**
that serializes requests. Under 256 concurrent connections its p99 latency is **5–8
seconds**: requests are queuing, not being served slowly. 98 rps for plaintext JSON is an
artifact of that dev server, **not** Laravel's production ceiling (php-fpm/nginx or Laravel
Octane reach thousands of rps). Ferro here is a multi-threaded `--release` build.

So these ratios measure "a tokio release server vs a single-process PHP dev server" — a
foregone conclusion that says nothing useful about Ferro vs Laravel. They are published only
to show the harness runs end-to-end. **A fair performance comparison requires a
production-representative Laravel server (php-fpm or Octane) and is deferred to Phase 2.**

---

Measured on Apple M1 Pro (8 cores, 16 GB, macOS Darwin 23.6.0), oha 1.9.0, c=256, 30s timed
after 5s warmup, PostgreSQL 16.4 shared by both apps. All values are a strict subset of
`internal.md`. Full methodology: `benchmark/README.md`.
