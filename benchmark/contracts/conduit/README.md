# Conduit Conformance Collection (Newman)

`Conduit.postman_collection.json` is the official RealWorld/Conduit Postman
collection, used as the like-for-like conformance gate for **both** the Ferro
Conduit (`benchmark/apps/ferro-conduit`) and the vendored Laravel Conduit
(Plan 06). It is run with [Newman](https://github.com/postmanlabs/newman).

## Source

- Upstream: `gothinkster/realworld`, path `api/Conduit.postman_collection.json`
- Pinned commit: `e7ab92bba08ba93ed322f50fd33b2857c183a0cf`
- Fetched: 2026-06-15

> Note: upstream `gothinkster/realworld` later deleted the Postman collection
> (commit `8c5a0b3e…`, 2026-02-13) in favour of Bruno/Hurl suites in the
> `realworld-apps/realworld` repo. We vendor the last commit that still carried
> the canonical Postman/Newman collection so the contract is frozen for the
> benchmark. The collection content is unchanged for years (stable Conduit spec).

## Running

```bash
# Full collection against a local backend
./run_newman.sh http://localhost:3000/api

# A single folder (auth conformance)
./run_newman.sh http://localhost:3000/api "Error Cases - Auth"
```

`newman` is invoked via `npx newman` if not globally installed. The run exits
non-zero if any assertion fails. A machine-readable report is written to
`newman-result.json`.

## Folder → endpoint-group mapping

| Folder | Endpoint group | Covers |
|--------|----------------|--------|
| `Articles, Favorite, Comments` | Articles + Favorites + Comments | CRUD, favorite/unfavorite, comments, feed |
| `Profiles` | Profiles | get profile, follow/unfollow |
| `Pagination` | Articles/Feed pagination | `limit`/`offset`, `articlesCount` |
| `Tags` | Tags | tag list |
| `Error Cases - Auth` | **Auth** (register/login/current-user/update-user) | validation `422` envelopes, duplicate, wrong password, `401` no-auth gating |
| `Error Cases - Articles & Comments` | Articles/Comments error paths | `401`/`404`/`422` |
| `Error Cases - Profiles & Authorization` | Profiles/authz error paths | `401`/`403`/`404` |

### Auth conformance note

This vintage of the collection has no standalone happy-path "Auth" folder; the
register/login happy path is exercised inline as setup inside other folders and
asserted explicitly in **`Error Cases - Auth`** (register validation envelopes,
duplicate credentials, login empty/wrong password, and `GET`/`PUT /user`
unauthenticated `401`). `Error Cases - Auth` is therefore the dedicated auth
conformance folder for Plan 03's acceptance gate. The happy-path register/login
responses (user envelope + JWT token) are additionally covered by the
`ferro-conduit` route/handler tests and by the article/profile folders that
depend on a valid token.
