---
phase: 230
slug: framework-benchmark-1b-ferro-conduit
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-15
---

# Phase 230 — Validation Strategy

> The Ferro Conduit must conform to the RealWorld/Conduit API spec. Validation is contract
> conformance against the official RealWorld test suite, not bespoke assertions.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Conformance gate** | The official RealWorld API tests — the `Conduit.postman_collection.json` run via **Newman** (`newman run ... --env-var APIURL=...`) against a running Ferro Conduit. This is the same suite the vendored Laravel backend passes, so it is a like-for-like contract check. |
| **Unit tests** | Rust `#[test]` for the hand-rolled JWT (mint→decode round-trip, expiry, bad-signature reject) and for the feed-vs-slug route-ordering (matchit priority) |
| **Quick run** | `cargo test -p ferro-conduit` (scoped to the benchmark app) |
| **Harness** | The Phase 229 `benchmark/harness` static + perf runners, applied to the Conduit workload |

## Sampling Rate

- After each endpoint group lands: run the relevant Newman folder (Auth, Articles, Comments, Profiles, Favorites, Feed).
- Before declaring the app done: the FULL Newman collection passes against the Ferro Conduit AND against the vendored Laravel Conduit (both green = fair comparison).
- Never run the full root-workspace `cargo test --all-features` — scope to `-p ferro-conduit`.

## Per-Task Verification Map (high level — planner refines)

| Capability | Test | Gate |
|-----------|------|------|
| JWT mint/decode/middleware (hand-rolled — the one non-framework piece) | `cargo test -p ferro-conduit jwt` | unit |
| Register/login/current-user/update | Newman "Auth" folder | conformance |
| Articles CRUD + slugs + list/filter/pagination | Newman "Articles" folder | conformance |
| Comments add/delete/list | Newman "Comments" folder | conformance |
| Profiles + follow/unfollow | Newman "Profiles" folder | conformance |
| Favorite/unfavorite + favoritesCount | Newman "Favorites" folder | conformance |
| Feed (followed users) + global + tags | Newman "Feed"/"Tags" | conformance |
| Route ordering `/articles/feed` vs `/articles/{slug}` | registration test | unit |

## Wave 0 Requirements

Newman + the official RealWorld `Conduit.postman_collection.json` must be available (vendor the
collection into `benchmark/contracts/conduit/`). The harness static/perf runners already exist
(Phase 229). No other test scaffolding needed.

## Honesty hook (D-10 carries forward)

The JWT code is hand-rolled (not framework-provided) — the static-compression report MUST count
it separately and label it as such, so the "framework-provided" line count is not overstated.

## Validation Sign-Off

- [ ] Full RealWorld Newman collection green against Ferro Conduit AND vendored Laravel Conduit
- [ ] JWT unit tests + route-ordering test green
- [ ] static + perf harness run on the Conduit workload
- [ ] JWT counted separately in the compression report

**Approval:** pending
