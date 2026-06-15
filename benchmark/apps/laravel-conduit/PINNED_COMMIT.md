# Vendored Laravel RealWorld backend — pin & provenance

This directory is a **vendored, unmodified copy** of a community Laravel RealWorld
(Conduit) backend. It is the like-for-like baseline for `benchmark/apps/ferro-conduit`.
It was not authored here.

## Source

- **Repository:** https://github.com/f1amy/laravel-realworld-example-app
- **Pinned commit:** `c14fb8370b71a42a3a74b8ea936a1f96b2af9d69`
- **Vendored on:** 2026-06-15
- **License:** MIT (Copyright (c) 2021 f1amy) — see `LICENSE`
- **Laravel:** `^9.11` (PHP `^8.0.2`) — modern, maintained
- The repository `.git` directory was removed; this is a flat vendored snapshot.

## Candidate evaluation (f1amy vs gothinkster)

The plan checkpoint required evaluating two community backends for RealWorld/Newman
conformance and pinning the one that passes with the least configuration drift.

| Candidate | Laravel | PHP | RealWorld route surface | Decision |
|-----------|---------|-----|--------------------------|----------|
| **f1amy/laravel-realworld-example-app** | `^9.11` (modern, maintained) | `^8.0.2` | Full canonical Conduit API (`routes/api.php`): users/login, users (register), user (get/update), profiles/follow, articles CRUD + feed, comments, favorites, tags | **CHOSEN** |
| gothinkster/laravel-realworld-example-app | 5.x (archived) | 7.x (legacy) | Canonical historically, but requires a legacy PHP image and more Docker work | fallback (not needed) |

**Conformance evidence (pre-vendor probe of f1amy at the pinned SHA):**

- `routes/api.php` declares the complete canonical Conduit endpoint set under the
  `api.` route-name group, with `auth:api` guarding the authenticated routes — a
  direct match for the frozen RealWorld Postman/Newman collection.
- `config/auth.php`: default guard `api`, `'api' => ['driver' => 'jwt', ...]`.
- `app/Http/Resources/Api/UserResource.php` emits the `{ "user": { ..., "token" } }`
  envelope the collection asserts on; the token is minted by `App\Jwt\Generator`.
- The 11 migrations cover users, articles, comments, tags, `article_tag`,
  `article_favorite`, and `user_follower` — the full Conduit schema.
- The full Newman collection result (green, zero failures) against the running
  vendored app is recorded in `benchmark/results/conduit/newman-laravel.json` and
  summarized in `benchmark/results/conduit/RESULTS.md`.

## JWT asymmetry note (for RESULTS honesty, D-10)

Both implementations hand-roll their JWT in **application code**, not in a vendored
package:

- **Ferro Conduit:** `src/jwt.rs` (96 LoC) + two JWT middleware files. Ferro's own
  auth is session-based, so JWT is non-framework-provided and counted **separately**
  in the static report.
- **This Laravel app (f1amy):** ships its own JWT under `app/Jwt/*` + `app/Auth/JwtGuard.php`
  + `app/Contracts/Jwt*` (~661 LoC of app code). It is **not** `tymon/jwt-auth`
  (that package is not a dependency); the JWT lives in app code here too.

This is a *symmetry*, not the asymmetry the plan anticipated: neither side gets JWT
"for free" from its framework. RESULTS.md reports both JWT carveouts so the
framework-provided application-code comparison is fair on both sides.

## Configuration-only changes applied

Application logic is **unmodified**. The only changes are run/serving configuration,
applied at the Docker layer (not by editing the vendored tree):

- `APP_KEY` and Postgres `DB_*` env are injected via `benchmark/conduit-compose.yaml`.
- `JWT_SECRET` (the app's `config/jwt.php` key) is injected via compose env.
- `php artisan migrate --force` is run on container boot (entrypoint), mirroring
  the `laravel-micro` pattern.
- `composer install --no-dev --optimize-autoloader` runs in the image build.

No file under `app/`, `routes/`, `config/` (logic), or `database/migrations/` was
edited. `config/*` values are overridden via environment, not by editing the files.
