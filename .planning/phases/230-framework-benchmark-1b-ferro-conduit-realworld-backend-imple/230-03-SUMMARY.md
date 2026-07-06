---
phase: 230-framework-benchmark-1b-ferro-conduit
plan: 03
subsystem: api
tags: [conduit, realworld, jwt, auth, newman, dto, routing, sea-orm]

requires:
  - phase: 230-01
    provides: jwt module (mint_token/jwt_secret), JwtAuthMiddleware, UserId extension
  - phase: 230-02
    provides: user entity (set_password/verify_password), migrations
provides:
  - Vendored official RealWorld Conduit Newman collection + runner (benchmark/contracts/conduit/)
  - Conduit request/response DTOs (camelCase envelopes) + error-envelope helper
  - Four auth endpoints (register/login/current_user/update_user) conforming to the Conduit contract
  - JWT-gated /api/user route group
  - feed-vs-{slug} route-ordering guard test
affects: [230-04, 230-05, 230-06, 230-07]

tech-stack:
  added: [newman (npx), hyper (dev-dep)]
  patterns:
    - "Nested-envelope request DTOs ({\"user\":{...}}) unwrapped via req.input::<Envelope>().await?"
    - "camelCase response DTOs (#[serde(rename_all=\"camelCase\")]); never serialize SeaORM models directly"
    - "Conduit error envelope helper: error_envelope(status, field, msgs) -> {\"errors\":{field:[...]}}"
    - "Stateless JWT: handlers mint_token + return; never Auth::login(); protected handlers read req.get::<UserId>()"

key-files:
  created:
    - benchmark/contracts/conduit/Conduit.postman_collection.json
    - benchmark/contracts/conduit/run_newman.sh
    - benchmark/contracts/conduit/README.md
    - benchmark/apps/ferro-conduit/src/dto/mod.rs
    - benchmark/apps/ferro-conduit/src/dto/responses.rs
    - benchmark/apps/ferro-conduit/src/dto/requests.rs
    - benchmark/apps/ferro-conduit/src/controllers/auth.rs
    - benchmark/apps/ferro-conduit/tests/route_ordering.rs
  modified:
    - benchmark/apps/ferro-conduit/src/main.rs
    - benchmark/apps/ferro-conduit/src/routes.rs
    - benchmark/apps/ferro-conduit/src/controllers/mod.rs
    - benchmark/apps/ferro-conduit/src/middleware/jwt_auth.rs
    - benchmark/apps/ferro-conduit/Cargo.toml

key-decisions:
  - "Newman auth conformance folder is 'Error Cases - Auth' — this vintage of the collection has no standalone happy-path 'Auth' folder; register/login happy path is inline setup, asserted explicitly in 'Error Cases - Auth'"
  - "Pinned the collection at gothinkster/realworld e7ab92bb (last commit before upstream removed the Postman collection in favour of Bruno/Hurl)"
  - "Error contract derived from the folder's actual assertions: register/login blank=422 'can't be blank'; duplicate=409 'has already been taken'; wrong password=401 credentials 'invalid'; no-auth=401 token 'is missing'"

patterns-established:
  - "Conduit error envelope helper reused by every controller wave"
  - "Literal-before-wildcard route ordering guarded by Router::match_route assertion"

requirements-completed: []

duration: ~40 min
completed: 2026-06-15
---

# Phase 230 Plan 03: Conduit Auth Endpoints + Newman Conformance Summary

**Vendored the official RealWorld Conduit Newman collection and implemented register/login/current-user/update-user with the Conduit `{"user":{...}}` + JWT envelope — the Newman auth conformance folder passes live (35/35 assertions) against the running Ferro Conduit backed by Postgres.**

## Performance

- **Duration:** ~40 min
- **Completed:** 2026-06-15
- **Tasks:** 3
- **Files modified:** 13 (8 created, 5 modified)

## Accomplishments

- Vendored `Conduit.postman_collection.json` (official RealWorld collection, pinned commit) + `run_newman.sh` (APIURL + optional folder, npx-newman fallback) + README (source/SHA/fetch-date, folder→endpoint mapping).
- Built the full Conduit DTO surface (UserDto/ProfileDto/ArticleDto/CommentDto + all request envelopes) reused by every later endpoint wave, plus the shared `error_envelope`/`validation_error_envelope` helper.
- Implemented the four auth endpoints with the exact Conduit error contract; the `/api/user` group is gated by `JwtAuthMiddleware`; tokens are minted via the Wave-1 `mint_token`.
- Landed the feed-vs-`{slug}` route-ordering guard (`Router::match_route` asserts feed resolves to the literal route, not `slug="feed"`) before the article routes arrive in Plan 04/05.
- **Ran the Newman auth folder live** against the running app + a throwaway Postgres: 9/9 requests, 35/35 assertions, 0 failures.

## Task Commits

1. **Task 1: Vendor Newman collection + runner** - `a6faa951` (feat)
2. **Task 2: Conduit request/response DTOs + error helper** - `e8791cf3` (feat)
3. **Task 3: Auth endpoints + JWT-gated routes + route-ordering test** - `c37618a5` (feat)

## Newman Auth — Live Run Evidence

Run against the release binary (Postgres on a throwaway `:5433` container, migrations applied):

```
newman run ... --folder "Error Cases - Auth" --env-var APIURL=http://127.0.0.1:3000/api \
  --env-var EMAIL=authtest@x.com --env-var USERNAME=authtest

iterations   1 / 0 failed
requests     9 / 0 failed
test-scripts 9 / 0 failed
assertions  35 / 0 failed
total run duration: 418ms
```

Covered: register empty username/email/password (422 `can't be blank`), register duplicate (409 `has already been taken`), login empty email/password (422), login wrong password (401 `credentials: invalid`), `GET`/`PUT /user` no-auth (401 `token: is missing`).

Manual happy-path smoke (same run) confirmed: register → 201 user envelope + JWT; current-user (auth'd) → 200 with re-minted token; update-user → 200 with applied `bio`.

## Decisions Made

- **"Error Cases - Auth" is the auth conformance folder.** The vendored collection version has no standalone happy-path "Auth" folder; happy-path register/login is inline setup, and the dedicated auth assertions (validation envelopes, 401 gating, duplicate, wrong-password) live in "Error Cases - Auth". Documented in the contracts README and the SUMMARY.
- **Error contract derived from the folder's actual assertion strings**, not from the spec prose — e.g. duplicate is **409** (not 422) and login failure is **401 `credentials: invalid`**, which differ from a naive reading.
- **The folder's `{{EMAIL}}`/`{{USERNAME}}` globals** (used by duplicate + wrong-password requests) are normally set by a happy-path register earlier in a full-collection run; for the isolated folder run they were seeded via `--env-var` matching a pre-registered user. Plan 06's compose run executes the full collection in order, so these globals are set naturally there.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] JWT 401 envelope did not match the Conduit contract**
- **Found during:** Task 3 (deriving the Error Cases - Auth contract)
- **Issue:** Wave-1 `JwtAuthMiddleware` returned `{"errors":{"body":["Unauthorized"]}}`; the collection asserts `{"errors":{"token":["is missing"]}}` for no-auth `GET`/`PUT /user`.
- **Fix:** Changed the middleware's 401 envelope to `{"errors":{"token":["is missing"]}}`; the in-handler fallback uses the same envelope.
- **Files modified:** src/middleware/jwt_auth.rs, src/controllers/auth.rs
- **Verification:** Newman "Current User - no auth" / "Update User - no auth" assertions green.
- **Committed in:** c37618a5

**2. [Rule 3 - Blocking] `routes!` macro already defines `register()`**
- **Found during:** Task 3 (first routes.rs draft wrapped the macro in a `pub fn register`)
- **Issue:** The `routes!` macro expands to `pub fn register() -> Router`; wrapping it produced a nested-fn conflict.
- **Fix:** Used `routes! { ... }` directly at module level (matching Wave-1's original form), keeping the literal-before-slug ordering comment.
- **Files modified:** src/routes.rs
- **Verification:** `cargo build --release` clean; `main.rs` `routes::register()` resolves.
- **Committed in:** c37618a5

**3. [Rule 1 - Bug] clippy unnecessary_get_then_check in the route test**
- **Found during:** Task 3 (clippy gate)
- **Issue:** `params.get("slug").is_none()` tripped `-D warnings`.
- **Fix:** `!params.contains_key("slug")`.
- **Files modified:** tests/route_ordering.rs
- **Committed in:** c37618a5

---

**Total deviations:** 3 auto-fixed (2 bug, 1 blocking)
**Impact on plan:** All necessary for contract conformance / clean gate. No scope creep.

## Issues Encountered

- The upstream Postman collection was removed from `gothinkster/realworld` (moved to Bruno/Hurl in `realworld-apps/realworld`). Resolved by pinning the last commit that still carried the canonical Postman collection (`e7ab92bb`), per the plan's "if main 404s, try the default branch via the GitHub API" fallback. Documented in the contracts README.
- The benchmark compose Postgres (`benchmark-db-1`) has no host port mapping, so the live run used a dedicated throwaway `postgres:16.4` container on host `:5433`, torn down after the Newman run.

## User Setup Required

None - no external service configuration required. The live Newman run used a local throwaway Postgres (torn down). Plan 06 wires the conduit app + Postgres into the benchmark compose for the full-collection run.

## Newman Auth: Live (not deferred)

The acceptance gate ran **live** in this plan (35/35 assertions green), not deferred to Plan 06. Plan 06 additionally runs the full collection in compose.

## Next Phase Readiness

- DTOs + error helper + auth + JWT gating are ready for Plans 04/05 (articles, profiles, comments, tags, feed, favorites).
- Route-ordering constraint is guarded by a test; the comment in `routes.rs` reminds Plan 04/05 to declare `/api/articles/feed` before `/api/articles/{slug}`.

## Self-Check: PASSED

- All 8 created files exist on disk; 5 modified files present.
- Commits a6faa951, e8791cf3, c37618a5 confirmed in `git log`.
- `cargo build --release` + `cargo clippy --release --all-targets -- -D warnings` clean; `cargo test --test route_ordering` passes; Newman "Error Cases - Auth" 35/35 live.

---
*Phase: 230-framework-benchmark-1b-ferro-conduit-realworld-backend-imple*
*Completed: 2026-06-15*
