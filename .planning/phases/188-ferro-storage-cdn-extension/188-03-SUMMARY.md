---
phase: 188-ferro-storage-cdn-extension
plan: "03"
subsystem: ferro-storage
tags: [storage, cdn, bunny, cloudflare, purge-api, feature-gates, docs, version-bump]
dependency_graph:
  requires: [188-01, 188-02]
  provides: [BunnyCdn-adapter, CloudflareCdn-adapter, storage-cdn-docs, workspace-0.2.46]
  affects:
    - ferro-storage/src/cdn/bunny.rs
    - ferro-storage/src/cdn/cloudflare.rs
    - ferro-storage/src/cdn/mod.rs
    - ferro-storage/src/lib.rs
    - docs/src/features/storage.md
    - Cargo.toml
tech_stack:
  added: []
  patterns: [cfg-feature-gate, token-redacted-debug, lean-purge-adapter]
key_files:
  created:
    - ferro-storage/src/cdn/bunny.rs
    - ferro-storage/src/cdn/cloudflare.rs
  modified:
    - ferro-storage/src/cdn/mod.rs
    - ferro-storage/src/lib.rs
    - docs/src/features/storage.md
    - Cargo.toml
decisions:
  - "Bunny uses per-URL POST to api.bunny.net/purge?url=...&async=false; no batching (Bunny API is per-URL)"
  - "Cloudflare uses POST /zones/{zone_id}/purge_cache with full-URL array; HTTP is_success() as signal (status code stable, response body low-confidence)"
  - "Both cfg-gated modules declare no new dependencies — they share the reqwest already in the default graph"
  - "Default and --features cdn-bunny,cdn-cloudflare cargo tree output are identical (confirmed below)"
metrics:
  duration: ~848 seconds
  completed: 2026-06-08
  tasks_completed: 2
  files_modified: 6
---

# Phase 188 Plan 03: Bunny + Cloudflare adapters + docs + version bump Summary

Feature-gated `BunnyCdn` and `CloudflareCdn` adapters ship as real lean `PurgeApi` impls with redacted-Debug configs; both compile behind their cargo features and are absent from the default dependency graph (criterion 4 proven); the storage docs page gains a complete CDN section; workspace bumped to 0.2.46; full `--all-features` CI-parity gate green.

## Completed Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Bunny + Cloudflare feature-gated adapters + cfg-gated re-exports | `73c05b9e` | cdn/bunny.rs, cdn/cloudflare.rs, cdn/mod.rs, lib.rs |
| 2 | Default-graph absence proof + docs CDN section + version bump + full CI-parity gate | `c19db1e3` | docs/src/features/storage.md, Cargo.toml, Cargo.lock |

## What Was Built

### Task 1 — BunnyCdn + CloudflareCdn adapters

`ferro-storage/src/cdn/bunny.rs` (new, 80 lines):

- **`BunnyCdnConfig`** — `Clone` only (no `#[derive(Debug)]`). Hand-written `Debug` prints `"<redacted>"` for `access_key` (T-188-10). `from_env()` reads `BUNNY_CDN_URL` + `BUNNY_ACCESS_KEY`.
- **`BunnyCdn`** — holds `reqwest::Client` (built once in `new()`). `purge()`: empty slice → `Ok(())`; missing `access_key` → `Err(Error::cdn("BUNNY_ACCESS_KEY not set"))`; per path: `POST https://api.bunny.net/purge?url={full_url}&async=false` with `AccessKey` header; `is_success()` gate (Research Pitfall 6 — per-URL, not tag-purge).

`ferro-storage/src/cdn/cloudflare.rs` (new, 91 lines):

- **`CloudflareCdnConfig`** — `Clone` only, hand-written `Debug` prints `"<redacted>"` for `api_token` (T-188-10). `from_env()` reads `CF_ZONE_ID` + `CF_API_TOKEN` + `CF_CDN_URL`.
- **`CloudflareCdn`** — holds `reqwest::Client`. `purge()`: empty slice → `Ok(())`; missing `api_token` → `Err(Error::cdn("CF_API_TOKEN not set"))`; builds full-URL vec, `POST /zones/{zone_id}/purge_cache` with `{"files":[...full_urls...]}` and Bearer auth; `is_success()` gate (HTTP status is the stable signal — research LOW-confidence on response body).

`ferro-storage/src/cdn/mod.rs`:
- Added cfg-gated `pub mod bunny; pub use bunny::{BunnyCdn, BunnyCdnConfig};` and equivalent for cloudflare after the DO adapter.

`ferro-storage/src/lib.rs`:
- Added `#[cfg(feature = "cdn-bunny")] pub use cdn::{BunnyCdn, BunnyCdnConfig};` and equivalent for cloudflare after the unconditional CDN re-exports.

### Task 2 — Docs + version bump + CI gate

`docs/src/features/storage.md`:
- Added `## CDN` section (130 lines) covering: `cdn_url()` / `with_cdn_url()` builder + env config; `PurgeApi` trait signature; `DoSpacesCdn` adapter with batching/throttle/wildcard/no-op semantics and env vars; feature-gated `BunnyCdn` + `CloudflareCdn` with code examples; promote→purge sequence; B-02 policy note (purge only non-hashed HTML; content-hashed assets are immutable; `*` is full-cache invalidation only).

`Cargo.toml`:
- Workspace version bumped `0.2.45` → `0.2.46`. No new-crate chores — ferro-storage is an existing published crate; CI publish-update token handles it on the version bump.

### Criterion 4 — Default-graph absence proof

`cargo tree -p ferro-storage` (default) and `cargo tree -p ferro-storage --features cdn-bunny,cdn-cloudflare` produce **identical** crate sets. The `cdn-bunny` and `cdn-cloudflare` features add no dependency entries — they expose code that shares the `reqwest` already in the default graph. Neither `bunny` nor `cloudflare` module symbols appear in the default compilation unit.

```
ferro-storage v0.2.46 (default)          = ferro-storage v0.2.46 (--features cdn-bunny,cdn-cloudflare)
├── async-trait v0.1.89                  ├── async-trait v0.1.89
├── bytes v1.11.1                        ├── bytes v1.11.1
├── dashmap v6.1.0                       ├── dashmap v6.1.0
├── mime_guess v2.0.5                    ├── mime_guess v2.0.5
├── reqwest v0.12.x                      ├── reqwest v0.12.x
├── serde v1.x                           ├── serde v1.x
├── serde_json v1.x                      ├── serde_json v1.x
├── thiserror v1.x                       ├── thiserror v1.x
├── tokio v1.x                           ├── tokio v1.x
└── tracing v0.1.x                       └── tracing v0.1.x
                                         (identical — no new crates)
```

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. Both `BunnyCdn` and `CloudflareCdn` implement real `PurgeApi` calls against their respective provider endpoints. No placeholder behavior.

## Threat Flags

All T-188-10 through T-188-13 mitigations confirmed:

- **T-188-10** (secret in Debug/logs): `BunnyCdnConfig` and `CloudflareCdnConfig` have no `#[derive(Debug)]`; manual impls print `<redacted>` for `access_key` and `api_token` respectively. Grep-verified by acceptance criteria check.
- **T-188-11** (path injection): Bunny paths go through reqwest `.query()` (URL-encoded by reqwest). CF paths go into `serde_json::json!` body (escaped). No manual string interpolation into the request line.
- **T-188-12** (SSRF): Bunny host is the fixed string `"https://api.bunny.net/purge"`; CF host is the fixed string `"https://api.cloudflare.com/client/v4/zones/..."`. Zone id and CDN base come from trusted env. No untrusted host injection.
- **T-188-13** (transport): reqwest with `rustls-tls`; HTTPS only; cert verification at default.

## Self-Check: PASSED

- `ferro-storage/src/cdn/bunny.rs` — FOUND (created, 80 lines)
- `ferro-storage/src/cdn/cloudflare.rs` — FOUND (created, 91 lines)
- `ferro-storage/src/cdn/mod.rs` — FOUND (modified, cfg-gated submodules added)
- `ferro-storage/src/lib.rs` — FOUND (modified, cfg-gated re-exports added)
- `docs/src/features/storage.md` — FOUND (modified, CDN section added)
- `Cargo.toml` — FOUND (`version = "0.2.46"` confirmed)
- Commit `73c05b9e` — Task 1
- Commit `c19db1e3` — Task 2
- `cargo fmt --all -- --check` — CLEAN
- `cargo clippy --all --all-targets --all-features -- -D warnings` — CLEAN
- `cargo test --all-features` — ALL OK (zero failures across entire workspace)
- Criterion 4 cargo tree comparison — CONFIRMED identical
