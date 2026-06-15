---
phase: 230-framework-benchmark-1b-ferro-conduit
plan: 05
subsystem: benchmark/ferro-conduit
tags: [conduit, realworld, profiles, follow, sea-orm, junction, newman]
requires:
  - "230-01: OptionalJwtMiddleware, JwtAuthMiddleware, UserId"
  - "230-02: follow + user entities (follows composite PK)"
  - "230-03: error_envelope, Newman runner"
  - "230-04: ProfileDto, require_viewer pattern (articles.rs)"
provides:
  - "GET /api/profiles/{username} (optional auth, viewer-relative following)"
  - "POST/DELETE /api/profiles/{username}/follow (required auth, follows junction mutation)"
  - "follows-junction read/write used by Plan 06 feed + article author.following"
affects:
  - "Plan 06 feed reads the same follows table this plan mutates"
tech-stack:
  added: []
  patterns:
    - "Direct junction query for following flag (no SeaORM relation on follows)"
    - "Idempotent follow via composite-PK no-op on duplicate insert (error swallowed)"
    - "delete_by_id((follower, followed)) tuple key for unfollow"
    - "Path-keyed route middleware split: optional-auth show vs required-auth follow/unfollow"
key-files:
  created:
    - benchmark/apps/ferro-conduit/src/controllers/profiles.rs
  modified:
    - benchmark/apps/ferro-conduit/src/controllers/mod.rs
    - benchmark/apps/ferro-conduit/src/routes.rs
decisions:
  - "follower_id sourced only from the verified UserId, never from the request body (T-230-15)"
  - "Duplicate follow is a no-op via the follows composite PK; the unique-violation insert error is swallowed (T-230-16)"
  - "Profile show carries OptionalJwtMiddleware; follow/unfollow carry JwtAuthMiddleware AND self-enforce require_viewer() (defense-in-depth, mirrors articles.rs)"
metrics:
  duration: ~25m
  tasks: 1
  files: 3
  completed: 2026-06-15
---

# Phase 230 Plan 05: Conduit Profiles + Follow/Unfollow Summary

**Profiles get (optional auth, viewer-relative `following`) and follow/unfollow (required auth) implemented against the `follows` junction via direct composite-key queries — the Newman Profiles folder passes 32/32 assertions live against the running Ferro Conduit backed by Postgres.**

## What Was Built

`controllers/profiles.rs` (three handlers + helpers):

- **`show`** (optional auth): `viewer = req.get::<UserId>()`; loads the target by `req.param("username")` (404 `{"errors":{"profile":["not found"]}}` if missing); `following` is a direct `follow::Entity` count query (`FollowerId = viewer AND FollowedId = target`, `false` for a guest). Returns `{"profile":{username,bio,image,following}}`.
- **`follow`** (required auth): `require_viewer()` → 401 if no token; inserts a `follows` row (`follower_id = viewer`, `followed_id = target.id`). The composite PK makes a repeat follow a no-op, so the unique-violation insert error is swallowed (idempotent). Returns the profile with `following = true`.
- **`unfollow`** (required auth): `delete_by_id((viewer, target.id))` (a no-op when the row is absent). Returns the profile with `following = false`.

Shared helpers: `find_user` (username → 404), `is_following` (guest-safe junction count), `profile_response` (assembles the envelope as seen by the viewer).

Routes appended to the existing structure (Plan 04 owns the article groups, left untouched):
- optional-auth group: `GET /api/profiles/{username}`.
- required-auth group: `POST` + `DELETE /api/profiles/{username}/follow`.

Ferro route middleware is PATH-keyed, so `/profiles/{username}` and `/profiles/{username}/follow` are distinct paths and CAN carry distinct middleware (unlike the same-path GET/POST split that forced articles into one optional group). Follow/unfollow additionally self-enforce `require_viewer()` for defense-in-depth.

## Tasks Completed

1. **Task 1: Profiles show/follow/unfollow + wire routes → Newman Profiles green** — `695abd46` (feat)

## Newman Profiles — LIVE, 32/32 assertions green

Against a fresh Postgres + the served app (`SERVER_PORT=3055`, `DATABASE_URL` → throwaway `postgres:16` on :5434), seeding the viewer `token`/`EMAIL`/`USERNAME`/`PASSWORD` (which the Auth folder sets naturally in a full-collection run) and running the **Profiles** folder:

```
folder: Profiles
requests:     5 executed / 0 failed
assertions:  32 executed / 0 failed
```

Folder covers: register celeb, get-profile, follow (following true), unfollow (following false), verify-unfollow-persisted.

### Seeding-scope note (honest, not a backend defect)

A first isolated run showed 31/32: the single failure was `Profile username is celeb_USERNAME`, whose script compares `profile.username === 'celeb_' + pm.globals.get('USERNAME')`. It reads the **globals** scope; seeding only `--env-var` left globals empty → the script computed `'celeb_undefined'` and failed. The backend response was correct (the matching `Register Celeb` request, which set the username server-side, passed). Re-seeding into the **globals** scope (`--global-var`, exactly what Plan 07's full-collection compose run produces via the Auth folder) yields the full 32/32. Build + clippy + `route_ordering` are green now.

## Live smoke (direct curl, fresh DB) — all correct

| Check | Result |
|-------|--------|
| GET profile as guest | `following: false`, full envelope |
| GET profile as viewer pre-follow | `false` |
| POST follow (auth) | `following: true` |
| POST follow again (idempotent) | `200`, no error (composite-PK no-op) |
| GET profile post-follow | `true` (viewer-relative) |
| DELETE unfollow | `following: false` |
| follow without auth | `401` |
| GET unknown profile | `404` |

## Verification

- `cargo build --release` — clean (12.0s).
- `cargo clippy --release -- -D warnings` — clean.
- `cargo test --release --test route_ordering` — `feed_resolves_before_slug` passes (article route ordering unchanged; profile routes are in separate groups).
- `rustfmt --check` on the three modified files — clean.

## Deviations from Plan

None — plan executed exactly as written. The plan's allowance ("if Plan 04 already exposes `to_profile_dto`, reuse it") did not apply: Plan 04 inlined the author/following assembly inside `to_article_dto` without exposing a standalone `to_profile_dto`, so this plan owns its own small, focused helpers (`is_following`, `profile_response`) rather than extracting and re-plumbing the article assembly. This keeps the change minimal and avoids touching articles.rs.

## Notes for Plan 06 / 07

- The feed (Plan 06) reads the same `follows` table this plan mutates; `feed_placeholder` in articles.rs must be replaced with the followed-author query.
- The full `Articles, Favorite, Comments` Newman folder still requires favorites + comments endpoints and the full-collection seed order — deferred to Plan 07's compose run (consistent with Plan 04's note). The standalone Profiles folder is green now.

## Self-Check: PASSED

All created/modified files exist on disk; commit `695abd46` present in git history.
