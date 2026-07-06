# Conduit (RealWorld) benchmark — Ferro vs Laravel

A like-for-like comparison of two RealWorld/Conduit backends on a shared Postgres
server: the **Ferro Conduit** (purpose-built to the frozen Conduit collection, Plans
01-06) and a **vendored community Laravel Conduit** (f1amy, pinned). Conformance is
the same frozen Postman/Newman collection; static compression counts application code
with the hand-rolled JWT carved out on both sides; perf is a read-path load test.

- Date: 2026-06-15 · Hardware: Apple M1 Pro, 8 cores, 16 GB · See `meta.json`.
- Ferro: ferro-rs 0.2.65, `--release`, tokio multi-threaded.
- Laravel: f1amy/laravel-realworld-example-app @ `c14fb83` (MIT), Laravel 9, PHP 8.2.
  Served as php-fpm (pm=static, 20 workers, opcache) and Octane + RoadRunner (16 workers).
- Shared Postgres 16.4 server; one logical database per app (`conduit_ferro`,
  `conduit_laravel`) — both own the same canonical schema, so they cannot share one
  database; same engine and host resources, isolated schema.

## 1. Conformance (full Conduit Newman collection)

The same frozen RealWorld/Conduit Postman collection was run against both backends with
identical globals seeding (a primary user + one `dragons`-tagged article; all other
variables are produced by requests during the run).

| Backend | Requests | Assertions passed | Failed |
|---------|---------:|------------------:|-------:|
| Ferro Conduit | 75 / 75 | **422 / 422** | 0 |
| Laravel Conduit (f1amy) | 75 / 75 | 340 / 382 | 42 |

**Ferro is fully green (422/422).** The Ferro Conduit was iterated against this exact
collection in Plans 01-06, so it matches the collection's strict error wording, status
codes, and envelope shapes byte-for-byte.

**The vendored Laravel app passes the functional contract but not the collection's
strict error assertions.** All 75 requests execute; the 42 failing assertions break
down as:

- **~57 strict error-contract assertions** (counting downstream cascades): the
  collection asserts the *original Node.js reference's* error wording (e.g.
  `"can't be blank"`) and shapes (`{"errors":{"article":[...]}}` for a 404). f1amy
  returns Laravel's stock messages instead (`"The username field is required."`,
  `{"message":"Resource not found."}`). With `Accept: application/json` the app
  *does* return correct status codes and a valid `{"errors":{...}}` envelope on
  validation (422) and auth (401) — the divergence is message text and the 404/409
  envelope, not broken behavior.
- **6 happy-path/pagination assertions**: f1amy returns `200` (not `204`) on delete,
  and its `articlesCount` reflects the returned page size rather than the unpaginated
  total.

This is an honest property of comparing a purpose-built backend against a vendored
community one: no modern community Laravel RealWorld backend reproduces the frozen
collection's exact error strings (both f1amy and the archived gothinkster app use
Laravel's default validation messages). Per the benchmark's honesty rule, the vendored
app's logic was **not modified** to force conformance — only run configuration (DB env,
PHP version, a Carbon dependency bump) was changed. The functional API (register, login,
articles CRUD, comments, favorites, tags, profiles, follow/unfollow, feed) works on both.

Raw reports: `newman-ferro.json`, `newman-laravel.json`.

## 2. Static compression (application code)

`tokei` over application source only (vendored dependencies, build output, and Docker
recipes excluded). **The hand-rolled JWT is counted separately on both sides** so the
framework-provided application-code figure is not overstated.

| Metric | Ferro Conduit | Laravel Conduit (f1amy) |
|--------|--------------:|------------------------:|
| Total app code (lines) | 1,957 | 4,595 |
| Files | 37 | 142 |
| Hand-rolled JWT (lines) | 89 | 353 |
| **Framework-provided app code (lines)** | **1,868** | **4,242** |
| Source tokens | 7,448 | 22,036 |

**JWT carveout — not framework-provided (the honesty hook, D-10):**

- **Ferro:** `src/jwt.rs` + `src/middleware/jwt_auth.rs` + `src/middleware/optional_jwt.rs`
  = **89 lines**, labeled *not framework-provided (JWT auth — Ferro is session-based)*.
  Ferro's own auth is session-based, so JWT is an application capability here, not a
  framework feature.
- **Laravel (f1amy):** the app **also hand-rolls its JWT in application code** —
  `app/Jwt/*` + `app/Auth/JwtGuard.php` + `app/Contracts/Jwt*` = **353 lines**. It does
  **not** use the `tymon/jwt-auth` composer package (that dependency is absent). This
  is a *symmetry*, not the asymmetry the plan anticipated: neither framework provides
  JWT for free, and both carveouts are shown so the comparison is
  framework-provided-app-code vs framework-provided-app-code.

**Result:** the Ferro Conduit expresses the same RealWorld backend in **~1,868 vs
~4,242** framework-provided application lines (~2.3x less), across 37 vs 142 files.

Caveats:
- The Laravel total includes l5-swagger OpenAPI annotation scaffolding and a richer
  resource/request class layout; the Ferro total includes SeaORM entities and migration
  modules. The two trees are organized differently, so the line counts compare the
  *shape and volume of application code a developer maintains*, not identical units.
- Both counts are application code only; framework/runtime code lives in dependencies
  (Rust crates / `vendor/`) and is excluded from both.

Raw data: `static-ferro-conduit.json`, `static-laravel-conduit.json`.

## 3. Performance (read-path load test)

`oha` 1.9.0, 30s per endpoint, concurrency 256, 5s warm-up (discarded), against a
shared Postgres pre-seeded with a `celeb` user and 25 `dragons`-tagged articles.
Same load parameters as the Phase 229 micro benchmark for comparability.

**Re-measured 2026-06-16** after the article-list N+1 fix (commit `edf71f9c`). All four
apps (Ferro, Laravel php-fpm, Laravel Octane) were re-run **back-to-back in one fresh
host state** (post Docker restart, freed host disk) so the table is internally
consistent. Absolute throughput on every endpoint is higher than the original
2026-06-15 run because of the changed host conditions; the substantive change is the
article list, which the N+1 fix moved from ~9x slower to near-parity. The conformance
(§1) and static-compression (§2) sections are unchanged from 2026-06-15.

| Endpoint | Ferro (rps) | Laravel fpm (rps) | Laravel Octane (rps) |
|----------|------------:|------------------:|---------------------:|
| `/api/tags` | **18,664** | 2,447 | 3,185 |
| `/api/articles?limit=20` | 2,252 | 2,682 | **2,915** |
| `/api/profiles/celeb` | **17,326** | 2,729 | 2,798 |

p50 / p99 latency (ms):

| Endpoint | Ferro p50/p99 | Laravel fpm p50/p99 | Octane p50/p99 |
|----------|--------------:|--------------------:|---------------:|
| `/api/tags` | 12.8 / 31.6 | 100.0 / 181.4 | 75.7 / 150.0 |
| `/api/articles?limit=20` | 110.1 / 185.8 | 91.8 / 162.5 | 81.0 / 214.9 |
| `/api/profiles/celeb` | 13.9 / 31.3 | 90.0 / 165.5 | 84.5 / 201.8 |

**Honest reading — Ferro wins two, the article list is now near-parity:**

- **`/api/tags` and `/api/profiles/celeb`:** Ferro is **~6-7x faster** with far lower
  tail latency (p50 ~13ms vs ~80-100ms). These are simple single-query / few-row
  endpoints where Ferro's compiled async stack and connection reuse dominate.
- **`/api/articles?limit=20`:** after the N+1 fix Ferro is **2,252 rps** (p50 110ms),
  up from **273 rps** (p50 879ms) before the fix — an **8.3x throughput gain**, and a
  ~12x drop in p99. The fix batches the per-article relation loads (author, tag list,
  favorites count, viewer-relative `favorited`/`following`) into ~6 grouped queries per
  page instead of ~6×N round-trips. Ferro is now within ~16% of Laravel php-fpm (2,682
  rps) and ~23% of Octane (2,915 rps) on this endpoint, versus the ~9x deficit before.
  The residual gap is the endpoint's DB-bound nature: the request time is dominated by
  the grouped relation queries, so Ferro's compiled-stack edge (the 6-7x seen on the
  single-query endpoints) is muted here, where Laravel's eager-loading reaches the same
  rows in a comparable number of queries.

**Caveats (apply to all numbers):**
- Single-host, Docker-on-macOS (Docker Desktop VM); container/NAT overhead affects
  absolute throughput and is not production-representative. The 2026-06-16 re-run was
  done in a fresh VM (post-restart, freed disk); absolute numbers are not comparable to
  the 2026-06-15 run across the host-state change — only within the 2026-06-16 table.
- The two backends are different *implementations* of the same contract. The N+1 (now
  fixed on the Ferro side) was the clearest example; the table measures the apps as
  written, not a pure framework-vs-framework floor.
- php-fpm uses 20 static workers; Octane uses 16 workers — defensible, not tuned to win.
  No app-level query tuning was applied to either side beyond what each ships.
- Octane edges php-fpm here on all three endpoints, but the article list is DB-bound, so
  the warm-process advantage is modest (2,915 vs 2,682 rps) — within the expected range
  for a query-bound read path.
- Harness note: the perf runner keys results by path (`?`-stripped), so the plain and
  `tag=dragons` article-list queries collapse to one `/api/articles` row (last wins =
  the tag-filtered query). Both exercise the same batched-relations read path and return
  the same 20-row page (all 25 seed articles are `dragons`-tagged), so the number is
  representative of the article list either way. This behavior is identical in the
  2026-06-15 and 2026-06-16 runs, so the before/after comparison is like-for-like.

Raw data: `perf-ferro-conduit.json`, `perf-laravel-conduit.json`, `perf-laravel-octane.json`.

## 4. Validation sign-off

- [x] **A community Laravel RealWorld backend is vendored (not authored) at a pinned
      commit SHA**, after evaluating f1amy vs gothinkster — f1amy @ `c14fb83`
      (`apps/laravel-conduit/PINNED_COMMIT.md`).
- [x] **The full Conduit Newman collection was run against both backends** (fair
      like-for-like). Ferro is 422/422 green; the vendored Laravel app passes the
      functional contract with 42 strict-error-assertion divergences, reported honestly
      above and left unmodified (vendored, not authored).
- [x] **Static compression counts the hand-rolled JWT separately and labels it
      "not framework-provided"** on both sides (Ferro 89 lines; Laravel 353 lines).
- [x] **Perf runs Ferro vs Laravel (php-fpm + Octane) on a shared Postgres**, with
      documented pool/worker config and honest caveats. The article-list N+1 originally
      found on the Ferro side was fixed (commit `edf71f9c`) and the full table
      re-measured 2026-06-16; the endpoint is now near-parity (§3).

> Note on the "both green" goal: an exact 422/422 on the vendored Laravel app would
> require editing its application logic to match the frozen collection's error strings
> and status codes. The benchmark forbids modifying vendored app logic, and the honesty
> rule (D-10 / T-230-22) prefers reporting the conformance gap to manufacturing a green.
> The gap is fully characterized in §1.

## Update — N+1 fix re-measured (2026-06-16)

Commit `edf71f9c` fixes the per-article N+1 in the article list/feed handlers (batched
tags/favorites/authors/follows: ~6 fixed queries per page instead of ~6×N), preserving
the 422/422 conformance by construction (DTO output unchanged). The full perf table in
§3 has been **re-measured** with this fix in place (all four apps re-run back-to-back in
one fresh host state). Outcome: `/api/articles` rose from **273 → 2,252 rps** (p50
879ms → 110ms), closing the ~9x deficit to near-parity with Laravel (~16% behind
php-fpm, ~23% behind Octane). The original 2026-06-15 perf numbers (Ferro article list
273 rps) are superseded by the 2026-06-16 table above. See §3 for the full data and the
host-state caveat.
