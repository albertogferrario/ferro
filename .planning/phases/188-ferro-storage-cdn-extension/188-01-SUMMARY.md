---
phase: 188-ferro-storage-cdn-extension
plan: "01"
subsystem: ferro-storage
tags: [storage, cdn, reqwest, rustls, url-generation]
dependency_graph:
  requires: []
  provides: [cdn_url-facade, Error::Cdn, reqwest-dep, cdn-features]
  affects: [ferro-storage/src/facade.rs, ferro-storage/src/config.rs, ferro-storage/src/error.rs, ferro-storage/Cargo.toml]
tech_stack:
  added: [reqwest-0.12-rustls-tls, tokio-time-feature]
  patterns: [facade-cdn-url, dashmap-tuple-threading, tdd-red-green]
key_files:
  created: []
  modified:
    - ferro-storage/Cargo.toml
    - ferro-storage/src/error.rs
    - ferro-storage/src/facade.rs
    - ferro-storage/src/config.rs
decisions:
  - "cdn_url is facade-level only (DiskConfig + Disk); StorageDriver trait unchanged"
  - "StorageInner.disks changed to DashMap<String, (Arc<dyn StorageDriver>, Option<String>)> to thread cdn_url without changing create_driver() return type"
  - "reqwest added as non-optional dep (lean rustls, no native-tls) per D-05 — DO adapter is default graph"
  - "Error::Cdn not cfg-gated — covers all three CDN adapters"
metrics:
  duration: ~12 minutes
  completed: 2026-06-08
  tasks_completed: 2
  files_modified: 4
---

# Phase 188 Plan 01: CDN URL Presentation Layer + Cargo Scaffolding Summary

CDN URL generation (`Storage::cdn_url()`, `Disk::cdn_url()`) with origin fallback, plus `reqwest` lean-rustls dependency, `tokio time` feature, `cdn-bunny`/`cdn-cloudflare` cargo features, and `Error::Cdn` variant — all compiling, unit-tested, no new C-binding crates.

## Completed Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Cargo.toml deps/features + Error::Cdn variant | `4b1eab8c` | Cargo.toml, error.rs |
| 2 | cdn_url field/builder/method + AWS_CDN_URL + unit tests | `322fb7de` | facade.rs, config.rs |

## What Was Built

### Task 1 — Cargo Scaffolding + Error::Cdn

`ferro-storage/Cargo.toml`:
- `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }` — lean rustls, no OpenSSL
- `tokio` updated to include `"time"` feature (required for Plan 02 throttle primitive)
- `cdn-bunny = []` and `cdn-cloudflare = []` cargo features declared
- `wiremock` dev-dep deferred to Plan 02 (per plan instructions)

`ferro-storage/src/error.rs`:
- `Error::Cdn(String)` variant (not cfg-gated — covers all three CDN adapters)
- `Error::cdn(msg: impl Into<String>) -> Self` constructor following existing pattern

### Task 2 — CDN URL Facade

`ferro-storage/src/facade.rs`:
- `DiskConfig.cdn_url: Option<String>` field (initialized to `None` in all three constructors: `Default`, `local()`, `memory()`)
- `DiskConfig::with_cdn_url()` consuming builder mirroring `with_url()`
- `StorageInner.disks` type changed from `DashMap<String, Arc<dyn StorageDriver>>` to `DashMap<String, (Arc<dyn StorageDriver>, Option<String>)>` to carry cdn_url at the facade layer
- All 4 insertion sites updated: `new()` (cdn_url=None), `with_config()` (config.cdn_url.clone()), `with_storage_config()` (disk_config.cdn_url.clone()), `register_disk()` (None)
- `Disk::new()` updated to 2-arg form: `(driver: Arc<dyn StorageDriver>, cdn_url: Option<String>)`
- `Disk::cdn_url()`: double-slash-safe via `base.trim_end_matches('/') + path.trim_start_matches('/')`, falls back to `self.url(path).await` when `cdn_url` is None
- `Storage::cdn_url()`: delegates to `self.default_disk()?.cdn_url(path).await`

`ferro-storage/src/config.rs`:
- `cdn_url: None` added to the S3 struct literal in `from_env()`
- `if let Ok(cdn) = env::var("AWS_CDN_URL") { s3_config = s3_config.with_cdn_url(cdn); }` reads CDN base for the s3 disk
- Doc comment updated to mention `AWS_CDN_URL`

## Test Results

```
running 4 tests
test facade::tests::cdn_url_no_double_slash ... ok
test facade::tests::cdn_url_falls_back_to_origin ... ok
test facade::tests::cdn_url_via_storage_facade ... ok
test facade::tests::cdn_url_returns_cdn_when_configured ... ok
test result: ok. 4 passed; 0 failed; 0 ignored

test config::tests::from_env_cdn_url ... ok   (--features s3)

Full suite: 28 passed; 0 failed
```

## cargo tree No-New-*-sys Evidence

Before this plan: zero `*-sys` crates in `ferro-storage` tree.

After adding `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }`:

```
$ cargo tree -p ferro-storage 2>/dev/null | grep -E '\-sys' | sort -u
(no output)
```

Zero `*-sys` crates. The `rustls-tls` feature path reuses `aws-lc-rs`/`ring` already present in the workspace lockfile via `ferro-assets`, `ferro-deployments`, and the S3 SDK. No new C-binding crates were introduced.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. `cdn_url()` is fully wired: returns the CDN URL when configured, origin URL when not. The `Error::Cdn` variant and `cdn-bunny`/`cdn-cloudflare` features are correctly declared scaffolding for Plans 02/03, not stubs blocking this plan's goal.

## Threat Flags

None found. No new network endpoints, auth paths, file access patterns, or schema changes introduced. `AWS_CDN_URL` is a non-secret public-facing config value (T-188-02: accepted). The double-slash normalization for `cdn_url()` string composition is implemented (T-188-01: mitigated, asserted by `cdn_url_no_double_slash` test).

## Self-Check: PASSED

- `ferro-storage/src/facade.rs` — FOUND (modified)
- `ferro-storage/src/config.rs` — FOUND (modified)
- `ferro-storage/src/error.rs` — FOUND (modified)
- `ferro-storage/Cargo.toml` — FOUND (modified)
- Commit `4b1eab8c` — FOUND
- Commit `322fb7de` — FOUND
- 4 cdn_url tests green — VERIFIED
- from_env_cdn_url test green — VERIFIED
- Full 28-test suite green — VERIFIED
- No new *-sys crates — VERIFIED
