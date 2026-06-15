---
phase: 230-framework-benchmark-1b-ferro-conduit
plan: 06
subsystem: benchmark/ferro-conduit
tags: [conduit, realworld, comments, favorites, tags, feed, newman, sea-orm, junction]
requires:
  - "230-02: comment/favorite/tag/follow entities + junctions"
  - "230-03: CommentDto, CreateCommentEnvelope, error_envelope, Newman runner"
  - "230-04: to_article_dto, article CRUD, feed_placeholder, route ordering"
  - "230-05: follows junction + viewer-relative following pattern"
provides:
  - "Comments add/list/delete endpoints"
  - "Favorite/unfavorite endpoints + favoritesCount/favorited flow"
  - "GET /api/tags"
  - "Real followed-author article feed (replaces feed_placeholder)"
  - "Full single-app RealWorld Newman collection GREEN (422/422)"
affects:
  - "230-07: Laravel parity + cross-impl harness (single-app Ferro gate now met)"
tech-stack:
  added: []
  patterns:
    - "Path-keyed middleware: GET+POST on /articles/{slug}/comments share the optional-auth group; mutations self-enforce require_viewer"
    - "Application-level title uniqueness guard (count-before-insert) for the Conduit 409 contract"
key-files:
  created:
    - benchmark/apps/ferro-conduit/src/controllers/comments.rs
    - benchmark/apps/ferro-conduit/src/controllers/tags.rs
  modified:
    - benchmark/apps/ferro-conduit/src/controllers/articles.rs
    - benchmark/apps/ferro-conduit/src/controllers/mod.rs
    - benchmark/apps/ferro-conduit/src/routes.rs
decisions:
  - "Enforce article-title uniqueness at the application layer (409 errors.title) to match the reference Conduit contract, keeping the random slug suffix for distinct titles"
  - "Comment add/delete sit in the optional-auth group and self-enforce auth, because the public comment-list shares their path and Ferro middleware is path-keyed"
metrics:
  duration: ~35m
  completed: 2026-06-15
  tasks: 2
  files: 5
  commits: 3
---

# Phase 230 Plan 06: Comments, Favorites, Tags, Real Feed — Conduit Feature-Complete Summary

Comments (add/list/delete), favorites (favorite/unfavorite with `favoritesCount`/`favorited` flowing through `to_article_dto`), `GET /api/tags`, and the real followed-author feed (replacing `feed_placeholder`) are implemented and the **full single-app RealWorld Newman collection passes 422/422 assertions** against the running Ferro Conduit.

## What shipped

- **`controllers/comments.rs`** — `store` (201, required auth, self-enforced), `index` (public, oldest-first, viewer-relative author `following`), `destroy` (204, required auth; resolves the article by slug first then asserts `comment.author_id == uid`, 403 otherwise). Reuses the established viewer-relative `following` junction query.
- **`controllers/tags.rs`** — `index` (no auth): flat `{"tags":[name,...]}` from the `tags` table.
- **`controllers/articles.rs`** — `favorite`/`unfavorite` mutate the `favorites` composite-PK junction (idempotent insert / no-op delete); both return the article envelope with the recomputed `favoritesCount` and `favorited`. `feed` replaces `feed_placeholder`: `follower_id = viewer` → followed ids → `article.author_id IN (...)`, `articlesCount` before pagination, `created_at` desc, `limit`/`offset`. Article `store` now rejects a duplicate title with `409 {"errors":{"title":["has already been taken"]}}`.
- **`routes.rs`** — `GET /api/tags` public; comment GET/POST/DELETE in the optional-auth article group (path-keyed middleware constraint); favorite/unfavorite in the required-auth group; `/articles/feed` repointed from `feed_placeholder` to the real `feed`, still declared before `{slug}`.

## Tasks → commits

1. **Task 1: Comments + Tags controllers** — `601318e1` (feat)
2. **Task 2: Favorites + real feed + wire all routes** — `7b4bba8e` (feat)
3. **Conformance fixes surfaced by the live full-collection run** — `1ddb31f8` (fix)

## Full single-app RealWorld Newman collection — LIVE, 422/422 assertions GREEN

Against a fresh `postgres:16` (throwaway, :5435) with migrations applied and the release binary served (`SERVER_PORT=3060`), running the **entire** collection (no `--folder`):

```
requests     75 / 0 failed
assertions  422 / 0 failed
duration    2.2s
```

Per-folder (assertions / failed):

| Folder | Assertions | Failed |
|--------|-----------:|-------:|
| Articles, Favorite, Comments | 241 | 0 |
| Profiles | 32 | 0 |
| Pagination | 17 | 0 |
| Tags | 3 | 0 |
| Error Cases - Auth | 35 | 0 |
| Error Cases - Articles & Comments | 71 | 0 |
| Error Cases - Profiles & Authorization | 23 | 0 |
| **Total** | **422** | **0** |

### Globals-scope seeding (collection fixture, not a backend defect)

This vintage of the official Conduit collection has **no request that registers the primary user or creates the first article**; it consumes pre-seeded globals `USERNAME`, `EMAIL`, `PASSWORD`, `token`, and `slug` (the canonical RealWorld flow folds those into environment setup). Reproducing the prior plans' globals-scope seeding: register a user against the live API (capture the JWT into `token`), create one `dragons`-tagged article ("How to train your dragon") to populate `slug`, write those into a Newman globals file, then run with `--globals`. Every `slug2`/`celeb_*`/`commentId`/`neg_*` variable is set by requests during the run. With that seed the full collection is 422/422; an unseeded run fails everything downstream of the first authed request (`{{token}}`/`{{slug}}` left literal), which is a fixture expectation, not a backend defect.

## Deviations from Plan

### Auto-fixed Issues (Rule 1/2 — contract correctness, surfaced by the live gate)

The first full-collection run (after correct seeding) was 405/411 with 6 distinct backend defects; all fixed in `1ddb31f8`:

**1. [Rule 1 - Bug] Comment list forced to 401 for guests**
- **Issue:** `GET /articles/{slug}/comments` returned 401 to guests. Ferro route middleware is path-keyed; placing the auth'd `POST /articles/{slug}/comments` in a `JwtAuthMiddleware` group made the *shared path* require auth, so the public GET inherited it.
- **Fix:** Moved comment `store`/`destroy` into the optional-auth article group; both self-enforce `require_viewer()` (mirroring the article mutation handlers).

**2. [Rule 1 - Bug] Comment create/delete status codes**
- **Issue:** `store` returned 200 (contract: 201); `destroy` returned 200 (contract: 204).
- **Fix:** `store` → 201, `destroy` → 204.

**3. [Rule 1 - Bug] Delete-comment on unknown article returned the wrong error key**
- **Issue:** `destroy` looked the comment up by id directly, so an unknown article slug produced `errors.comment` instead of the contract's `errors.article` 404.
- **Fix:** Resolve the article by slug first (404 `errors.article`), then the comment scoped to that article (404 `errors.comment`).

**4. [Rule 2 - Missing contract behavior] Duplicate article title accepted**
- **Issue:** The random slug suffix let duplicate titles through; the Conduit contract expects `409 {"errors":{"title":["has already been taken"]}}` (reference Conduit treats the title as unique).
- **Fix:** Application-level title-uniqueness guard (count-before-insert) in `store`; the random slug suffix is retained for genuinely distinct titles that slugify the same.

All four are real backend conformance fixes against the frozen RealWorld contract, not workarounds.

## Known Stubs

None. `feed_placeholder` is fully removed and replaced by the real followed-author feed.

## Verification

- `cargo build --release` — clean.
- `cargo clippy --release -- -D warnings` — clean.
- `cargo test --release --test route_ordering` — `feed_resolves_before_slug` ok (feed before `{slug}`).
- Full single-app RealWorld Newman collection — **422/422 assertions, 0 failures** against the running Ferro Conduit (Postgres + served release binary). Live services torn down after the run.

## Self-Check: PASSED

- Created files present: `controllers/comments.rs`, `controllers/tags.rs`.
- Commits present: `601318e1`, `7b4bba8e`, `1ddb31f8`.
