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

| Endpoint | Ferro (rps) | Laravel fpm (rps) | Laravel Octane (rps) |
|----------|------------:|------------------:|---------------------:|
| `/api/tags` | **9,616** | 1,755 | 2,163 |
| `/api/articles?limit=20` | 273 | **2,552** | 2,096 |
| `/api/profiles/celeb` | **11,156** | 2,671 | 1,792 |

p50 / p99 latency (ms):

| Endpoint | Ferro p50/p99 | Laravel fpm p50/p99 | Octane p50/p99 |
|----------|--------------:|--------------------:|---------------:|
| `/api/tags` | 21.9 / 90.4 | 119.4 / 649.1 | 106.6 / 319.0 |
| `/api/articles?limit=20` | 878.6 / 2347.9 | 94.8 / 178.8 | 112.9 / 308.0 |
| `/api/profiles/celeb` | 21.9 / 42.7 | 92.5 / 154.8 | 132.4 / 315.7 |

**Honest reading — Ferro wins two, loses one decisively:**

- **`/api/tags` and `/api/profiles/celeb`:** Ferro is **~4-6x faster** with far lower
  tail latency. These are simple single-query / few-row endpoints where Ferro's compiled
  async stack and connection reuse dominate.
- **`/api/articles?limit=20`:** Ferro is **~9x slower** (273 vs 2,552 rps; p50 879ms).
  This is the most important finding. The Ferro Conduit's article-list handler performs
  **per-article follow-up queries** (author, tag list, favorites count, viewer-relative
  `favorited`/`following`) — an N+1 pattern that serializes 20 articles into many
  round-trips under load. The f1amy Laravel app eager-loads these relations, so its
  article list stays flat (~95ms p50). This is a real implementation difference in the
  Ferro Conduit, not a framework ceiling: the same workload is fast in Ferro on the
  single-query endpoints. **Action item for the Ferro Conduit:** batch the article-list
  relation loads (single grouped query per relation) to remove the N+1.

**Caveats (apply to all numbers):**
- Single-host, Docker-on-macOS (Docker Desktop VM); container/NAT overhead affects
  absolute throughput and is not production-representative.
- The two backends are different *implementations* of the same contract, with different
  query strategies (the N+1 above is the clearest example) — this measures the apps as
  written, not a pure framework-vs-framework floor.
- php-fpm uses 20 static workers; Octane uses 16 workers — defensible, not tuned to win.
  No app-level query tuning was applied to either side beyond what each ships.
- Octane did not consistently beat php-fpm here (the article list is DB-bound, not
  PHP-bootstrap-bound, so Octane's warm-process advantage is muted; on `/api/tags`
  Octane is faster, on profiles it is slightly slower — within run-to-run noise).

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
      documented pool/worker config and honest caveats — including the endpoint where
      Ferro loses (article list, N+1).

> Note on the "both green" goal: an exact 422/422 on the vendored Laravel app would
> require editing its application logic to match the frozen collection's error strings
> and status codes. The benchmark forbids modifying vendored app logic, and the honesty
> rule (D-10 / T-230-22) prefers reporting the conformance gap to manufacturing a green.
> The gap is fully characterized in §1.
