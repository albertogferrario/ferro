---
phase: 230-framework-benchmark-1b-ferro-conduit
plan: 04
subsystem: benchmark/ferro-conduit
tags: [conduit, articles, crud, seaorm, pagination, slug, jwt]
requires:
  - "230-01: JwtAuthMiddleware / OptionalJwtMiddleware / UserId"
  - "230-02: article/article_tag/tag/favorite/follow/user entities + article::generate_slug"
  - "230-03: ArticleDto/ProfileDto + error_envelope/validation_error_envelope + route_ordering test"
provides:
  - "Articles CRUD (store/show/update/destroy) with slug generation + author-ownership enforcement"
  - "Article list (index) with tag/author/favorited filters + limit/offset + articlesCount"
  - "to_article_dto: full Conduit article envelope assembly (tagList, author.following, favorited, favoritesCount)"
  - "feed_placeholder (Plan 06 must replace with the real followed-author feed)"
  - "All /api/articles* routes wired (feed before {slug})"
affects:
  - "Plan 05 (favorites/comments) reuses to_article_dto"
  - "Plan 06 replaces feed_placeholder + completes the Newman Articles folder"
tech-stack:
  added: []
  patterns:
    - "Optional-auth-everywhere + handler-level required-auth (Ferro route middleware is path-keyed, not method-keyed)"
    - "M:N filter via parameterized is_in over junction-derived article ids"
    - "pre-pagination PaginatorTrait::count for articlesCount"
key-files:
  created:
    - benchmark/apps/ferro-conduit/src/controllers/articles.rs
  modified:
    - benchmark/apps/ferro-conduit/src/controllers/mod.rs
    - benchmark/apps/ferro-conduit/src/routes.rs
    - benchmark/contracts/conduit/.gitignore
decisions:
  - "All /api/articles* share OptionalJwtMiddleware; mutations self-enforce auth via require_viewer() (path-keyed middleware can't split GET vs POST on the same path)"
  - "destroy returns 204 No Content (the Conduit collection asserts 204 on successful article delete)"
  - "tagList sorted alphabetically for deterministic output"
  - "limit clamped to 100 (T-230-14 DoS bound); unknown author/tag/favorited filter yields an empty result set, not a 404"
metrics:
  duration: ~40 min
  tasks: 2
  files: 4
  completed: 2026-06-15
---

# Phase 230 Plan 04: Articles CRUD + List Summary

Full Conduit Articles resource on Ferro: create (unique slug + tag association), read-by-slug, update (author-only), delete (author-only, 204), and list with tag/author/favorited filters + limit/offset + pre-pagination `articlesCount`. The article envelope is assembled by a single reusable `to_article_dto` helper (camelCase: `tagList`, `author` profile with `following`, `favorited`, `favoritesCount`, `createdAt`/`updatedAt`). Reads run optional-auth (guest-safe); mutations are JWT-gated and ownership-checked.

## What shipped

- `controllers/articles.rs`:
  - `to_article_dto(db, article, viewer)` — tagList (junction→tags, sorted), favoritesCount (count), favorited (viewer count), author ProfileDto + following (follows count). Count queries, not per-field N+1 inside a loop.
  - `store` — required auth; 422 on blank title/description/body; slug via `article::generate_slug`, retry up to 3× on UNIQUE conflict; find-or-create each tag and link via `article_tags`; 201 + envelope.
  - `show` — optional auth; 404 on unknown slug; full envelope.
  - `update` — required auth; 403 if not author; applies present fields; envelope.
  - `destroy` — required auth; 403 if not author; FK cascade removes tags/favorites/comments; **204 No Content**.
  - `index` — optional auth; tag/author/favorited filters (parameterized `is_in`/`eq`), limit (clamped 100)/offset, `articlesCount` = filtered count before pagination, `order_by_desc(created_at)`.
  - `feed_placeholder` — required auth; returns empty `{"articles":[],"articlesCount":0}` (Plan 06 replaces).
- `routes.rs` — all `/api/articles*` under `OptionalJwtMiddleware`, `feed` before `{slug}`; user routes stay under `JwtAuthMiddleware`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Optional-auth article reads returned 401 under the planned two-group route layout**
- **Found during:** Task 2 live run. The plan's structure put GET `/api/articles` + `/api/articles/{slug}` in an `OptionalJwtMiddleware` group and POST/PUT/DELETE of the same paths in a `JwtAuthMiddleware` group.
- **Issue:** Ferro's route middleware is keyed by **path string only**, method-agnostic (`framework/src/routing/router.rs:205` `route_middleware.entry(path)`). Both groups register the same canonical paths, so the GET routes inherited `JwtAuthMiddleware` and rejected guests with 401 — breaking optional-auth reads.
- **Fix:** Single `OptionalJwtMiddleware` group for all `/api/articles*`; mutation handlers self-enforce required auth via `require_viewer()` (401 when no `UserId`). Feed-before-`{slug}` ordering preserved.
- **Files modified:** `src/routes.rs`
- **Commit:** 363056af

**2. [Rule 1 - Bug] destroy returned 200, collection asserts 204**
- **Found during:** Task 2 seeded Newman run. Every successful article delete in the collection asserts `Response code is 204`.
- **Fix:** `destroy` returns `HttpResponse::new().status(204)` (empty body).
- **Files modified:** `src/controllers/articles.rs`
- **Commit:** 363056af

## Newman Articles — LIVE (env-seeded), one folder DEFERRED to Plan 06

The vendored official Conduit collection references `{{token}}` (41×) and `{{slug}}` (22×) that **no request in the collection ever sets** — they are required external environment inputs the harness must seed (the canonical RealWorld Postman environment provides them). `run_newman.sh` only sets `APIURL`, so an unseeded folder run 401s on every token-dependent article request (not a backend defect).

Driven correctly — fresh Postgres, seed `token` (a registered user) + `slug` (one created article), then run the `Articles, Favorite, Comments` folder:

- **Article-only assertions: 109/110 PASS** (`Create Second Article`, both pagination `articlesCount` checks, `All Articles`, `…with auth`, `Articles by Author` (+auth), `Single Article by slug`, `Update Article`, `Delete Article` (204), pagination cleanup — all PASS).
- The 1 remaining article failure (`Articles by Tag` → `tagList[1]==="training"`) is a **test-fixture-chaining artifact**: it asserts on an article whose two-tag creation depends on a chain step that aborts because favorites/comments are not yet implemented. `to_article_dto` tagList assembly is correct and ordered (a direct create returns `tagList:["dragons","training"]`).
- The Favorite/Comment/Profile assertions in the same folder fail because those endpoints are Plan 05/06 — **the complete Newman Articles folder green is deferred to Plan 06's compose**, when favorites + comments close the chain and the harness seeds `token`/`slug`. Build + `route_ordering` are green now.

### Live smoke (direct curl, fresh DB) — all correct
- create (with tags) → full camelCase envelope; show (optional, guest) 200; unknown slug 404; list filters tag/author (+unknown→count 0)/limit; update author 200 / no-auth 401 / wrong-author 403; create empty-title 422 / no-auth 401; delete author 204 / get-after-delete 404; feed_placeholder auth→empty / no-auth 401.

## Verification

- `cargo build --release` clean; `cargo clippy --release` clean (0 warnings).
- `cargo test --release --test route_ordering` — `feed_resolves_before_slug` PASS (feed not shadowed by `{slug}`).
- Live: ferro-conduit served against dedicated Postgres (`conduit-db` on :5433), migrations applied, Articles exercised via curl + env-seeded Newman.

## Known Stubs

- `feed_placeholder` (`controllers/articles.rs`) returns an empty multiple-articles envelope. **Plan 06 must replace it** with the real followed-author feed. Intentional and documented in the plan; the route resolves and guards auth (401 when no token).

## Notes for Plan 06

- Replace `feed_placeholder` with the real feed (followed authors, limit/offset, `articlesCount`); it can reuse `to_article_dto`.
- The full Newman Articles folder green requires: favorites + comments endpoints (Plan 05) AND a harness that seeds `token`/`slug` on a fresh DB (the `/tmp/seed_newman.sh` pattern used here, or a committed Postman environment file in `benchmark/contracts/conduit/`).

## Self-Check: PASSED

- articles.rs: FOUND
- 230-04-SUMMARY.md: FOUND
- commit 563aee6c (Task 1): FOUND
- commit 363056af (Task 2): FOUND
