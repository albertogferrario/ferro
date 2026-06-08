---
phase: 188-ferro-storage-cdn-extension
verified: 2026-06-08T00:00:00Z
status: passed
score: 4/4
overrides_applied: 0
re_verification: false
---

# Phase 188: ferro-storage CDN Extension — Verification Report

**Phase Goal:** Extend ferro-storage with CDN awareness — full CDN URLs for stored objects and a cache-purge abstraction with a DigitalOcean Spaces CDN default adapter, so promote-then-purge is a two-call sequence for any consumer.
**Verified:** 2026-06-08T00:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `Storage::cdn_url(path)` returns CDN edge URL when configured, falls back to origin when not | VERIFIED | `Storage::cdn_url` (facade.rs:326) delegates to `Disk::cdn_url` (facade.rs:407-415). Four unit tests confirmed passing: `cdn_url_returns_cdn_when_configured`, `cdn_url_falls_back_to_origin`, `cdn_url_no_double_slash`, `cdn_url_via_storage_facade`. `AWS_CDN_URL` env read confirmed in config.rs:119. |
| 2 | `PurgeApi` trait exposes `purge(paths)`; DO adapter calls `DELETE /v2/cdn/endpoints/{id}/cache`, batches ≤50, honors 5 req/10s throttle, wildcard = 1 slot | VERIFIED | `PurgeApi` trait in cdn/mod.rs:42-48. `DoSpacesCdn::purge` uses `paths.chunks(BATCH_SIZE)` (BATCH_SIZE=50, line 164), loop-based sliding-window throttle (lines 120-144, WR-01 fix present). Wiremock tests all pass: `do_adapter_batches_over_50` (2 requests for 55 paths), `do_adapter_wildcard_slot` (2 requests for 51 elements), `do_adapter_throttle_serializes` (elapsed ≥ 9s for 6 chunks). |
| 3 | DO adapter config reads `DO_SPACES_CDN_ID` + API token from env; missing CDN id makes `purge` a logged no-op | VERIFIED | `DoSpacesCdnConfig::from_env()` reads `DO_SPACES_CDN_ID` (line 84) and `DIGITALOCEAN_ACCESS_TOKEN` (line 85). Missing id path at cdn/mod.rs:153-156 logs via `tracing::info!` and returns `Ok(())`. `do_adapter_noop_missing_id` test uses wiremock `.expect(0)` and passes. |
| 4 | Bunny and Cloudflare adapters compile behind cargo features without entering the default dependency graph | VERIFIED | `cdn-bunny = []` and `cdn-cloudflare = []` in ferro-storage/Cargo.toml. `#[cfg(feature = "cdn-bunny")]` gates on cdn/mod.rs:186-189 and lib.rs:62-63; `#[cfg(feature = "cdn-cloudflare")]` on cdn/mod.rs:191-194 and lib.rs:64-65. `cargo tree -p ferro-storage` default (343 lines) = `cargo tree -p ferro-storage --features cdn-bunny,cdn-cloudflare` (343 lines) — identical crate sets, no new deps. |

**Score: 4/4 truths verified**

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-storage/src/facade.rs` | `DiskConfig.cdn_url` + `with_cdn_url()` + `Disk.cdn_url` + `Disk::cdn_url()` + `Storage::cdn_url()` | VERIFIED | All fields and methods present. `DiskConfig.cdn_url: Option<String>` (line 23), `with_cdn_url()` builder (line 94), `Disk.cdn_url` field (line 335), `Disk::cdn_url()` (lines 407-415), `Storage::cdn_url()` (lines 326-328). DashMap stores `(Arc<dyn StorageDriver>, Option<String>)` (line 107); cdn_url threaded through `with_config` (line 155), `with_storage_config` (line 193), `disk()` (lines 243-245). `register_disk_with_cdn` added for IN-01 fix (lines 262-269). |
| `ferro-storage/src/config.rs` | `AWS_CDN_URL` env read in `from_env()` | VERIFIED | `env::var("AWS_CDN_URL")` read at line 119 inside the s3 feature block. Doc comment updated to include `AWS_CDN_URL` description (line 55). `from_env_cdn_url` test under `#[cfg(feature = "s3")]` passes. |
| `ferro-storage/src/error.rs` | `Error::Cdn(String)` variant + `Error::cdn()` constructor | VERIFIED | `Cdn(String)` variant with `#[error("CDN error: {0}")]` at line 44. `pub fn cdn()` constructor at line 74. Not cfg-gated. `thiserror = "1.0"` unchanged. |
| `ferro-storage/Cargo.toml` | `reqwest` lean rustls + tokio time + cdn-bunny/cloudflare features + wiremock dev-dep | VERIFIED | `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }` (line 22). `tokio` includes `"time"` (line 14). `cdn-bunny = []`, `cdn-cloudflare = []` in `[features]` (lines 31-32). `wiremock = "0.6.5"` in `[dev-dependencies]` (line 37). No `native-tls`. |
| `ferro-storage/src/cdn/mod.rs` | `PurgeApi` trait + `DoSpacesCdn` adapter + `DoSpacesCdnConfig` + throttle + wiremock tests | VERIFIED | 384 lines. All required elements present. Manual `Debug` impl for `DoSpacesCdnConfig` redacts token (lines 67-75). Loop-based throttle (WR-01 fix: push_back while lock held, line 134, then `return`). All 8 named wiremock tests + `debug_does_not_contain_token`. `pub mod bunny/cloudflare` and `pub use` are cfg-gated. |
| `ferro-storage/src/cdn/bunny.rs` | `BunnyCdn` implements `PurgeApi` behind `cdn-bunny` feature | VERIFIED | `impl PurgeApi for BunnyCdn` (line 98). Per-URL POST to `api.bunny.net/purge`. WR-02 fix: loop-based throttle with `BUNNY_RATE_LIMIT_MAX=100`/10s (lines 74-94). WR-04 fix: `cdn_base_url.is_empty()` guard (line 106). Manual Debug redacts `access_key`. Tests: `bunny_adapter_empty_noop`, `bunny_adapter_missing_key_errors`, `bunny_adapter_missing_cdn_url_errors`. |
| `ferro-storage/src/cdn/cloudflare.rs` | `CloudflareCdn` implements `PurgeApi` behind `cdn-cloudflare` feature | VERIFIED | `impl PurgeApi for CloudflareCdn` (line 66). `CF_BATCH_SIZE = 30` (line 7, WR-03 fix). `for chunk in full_urls.chunks(CF_BATCH_SIZE)` (line 94). WR-04 fix: `zone_id.is_empty()` and `cdn_base_url.is_empty()` guards (lines 74-79). Manual Debug redacts `api_token`. Tests: `cf_batch_size_chunks_correctly`, `cf_adapter_empty_noop`, `cf_adapter_missing_token_errors`, `cf_adapter_missing_zone_id_errors`, `cf_adapter_missing_cdn_url_errors`. |
| `ferro-storage/src/lib.rs` | `pub mod cdn` + unconditional re-exports + cfg-gated Bunny/CF re-exports | VERIFIED | `pub mod cdn` (line 49). `pub use cdn::{DoSpacesCdn, DoSpacesCdnConfig, PurgeApi}` (line 60). `#[cfg(feature = "cdn-bunny")] pub use cdn::{BunnyCdn, BunnyCdnConfig}` (line 62-63). `#[cfg(feature = "cdn-cloudflare")] pub use cdn::{CloudflareCdn, CloudflareCdnConfig}` (line 64-65). |
| `docs/src/features/storage.md` | CDN section: `cdn_url` + `PurgeApi` + DO adapter + purge-only-non-hashed-HTML note | VERIFIED | `## CDN` section at line 377. Covers `Storage::cdn_url`/`Disk::cdn_url`, `with_cdn_url`, `AWS_CDN_URL`, `PurgeApi` trait, `DoSpacesCdn` with batching/throttle/wildcard/missing-id doc, feature-gated Bunny/CF adapters, promote→purge two-call sequence, B-02 policy note ("purge only the non-hashed HTML keys after a promote... Content-hashed asset URLs are immutable"). |
| `Cargo.toml` (workspace) | Version bumped to 0.2.46 | VERIFIED | `version = "0.2.46"` at line 36. Old 0.2.45 not present. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Storage::disk()` | `Disk { driver, cdn_url }` | `config.cdn_url` threaded into DashMap as `(driver, cdn_url)` | VERIFIED | facade.rs:155 (`with_config`), :193 (`with_storage_config`), :243-245 (`disk()`). |
| `Disk::cdn_url()` | `self.url()` fallback | `None => self.url(path).await` | VERIFIED | facade.rs:413. |
| `DoSpacesCdn::purge()` | DO CDN API DELETE endpoint | `reqwest .delete().bearer_auth().json().send()` | VERIFIED | cdn/mod.rs:166-173. URL pattern `v2/cdn/endpoints/{id}/cache` at line 162. |
| `DoSpacesCdn::purge()` | `self.throttle()` | called once per chunk before the request | VERIFIED | cdn/mod.rs:165. |
| `lib.rs` | `cdn` module | `pub mod cdn; pub use cdn::{...}` | VERIFIED | lib.rs:49, 60. |

---

### Data-Flow Trace (Level 4)

Not applicable for this phase. All deliverables are a library crate with no dynamic data rendering (no components, no pages). The unit and integration tests serve as data-flow validation.

---

### Behavioral Spot-Checks

| Behavior | Command / Method | Result | Status |
|----------|-----------------|--------|--------|
| `cdn_url` returns CDN URL when configured | `cargo test -p ferro-storage cdn_url` | 5 passed | PASS |
| DO adapter batches + wildcards + throttle | `cargo test -p ferro-storage do_adapter` | 7 passed (10.01s) | PASS |
| Empty purge no-op | `cargo test -p ferro-storage purge_empty_noop` | 1 passed | PASS |
| s3 from_env reads AWS_CDN_URL | `cargo test -p ferro-storage --features s3 from_env_cdn_url` | 1 passed | PASS |
| Full feature suite compiles and passes | `cargo build -p ferro-storage --features cdn-bunny,cdn-cloudflare` then test | 47 tests passed | PASS |
| Default feature build unchanged | `cargo tree -p ferro-storage` == `cargo tree --features cdn-bunny,cdn-cloudflare` | 343 lines each | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| STOR-F-01 | 188-01-PLAN.md | CDN URL generation: `Storage::cdn_url(path)` returns CDN edge URL, falls back to origin | SATISFIED | `Disk::cdn_url` + `Storage::cdn_url` implemented and tested |
| STOR-F-02 | 188-02-PLAN.md, 188-03-PLAN.md | Cache-purge abstraction: `PurgeApi` trait + DO adapter + Bunny/CF feature-gated | SATISFIED | `PurgeApi` trait, `DoSpacesCdn` (wiremock-proven), `BunnyCdn` + `CloudflareCdn` (feature-gated, compile-tested) |

---

### Code Review Fix Verification

| Finding | Fix Required | Status | Evidence |
|---------|-------------|--------|---------|
| WR-01: throttle race (if/else, not re-checking after sleep) | Loop-based re-check: `push_back` while lock still held | VERIFIED | cdn/mod.rs:120-144: `loop { ... if times.len() < RATE_LIMIT_MAX { times.push_back(...); return; } ... drop(times); sleep(...).await; }` |
| WR-02: BunnyCdn no rate limiting | Add loop-based throttle matching DO adapter | VERIFIED | bunny.rs:58,67,74-94,110: `request_times: Mutex<VecDeque<Instant>>`, `throttle()` loop, `self.throttle().await` before each POST |
| WR-03: CloudflareCdn sends all files in one request | Chunk to CF_BATCH_SIZE=30 | VERIFIED | cloudflare.rs:7,94: `const CF_BATCH_SIZE: usize = 30` + `for chunk in full_urls.chunks(CF_BATCH_SIZE)` |
| WR-04: Missing empty-config validation in Bunny/CF | Add early guards for cdn_base_url/zone_id | VERIFIED | bunny.rs:106-108; cloudflare.rs:74-79 |
| IN-01: `register_disk` silently drops CDN URL | Add `register_disk_with_cdn` overload | VERIFIED | facade.rs:262-269; two tests at lines 604-630 |

---

### Anti-Patterns Found

No blockers or warnings found. Notable observations:

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `ferro-storage/src/cdn/cloudflare.rs` | No rate limiting for Cloudflare adapter | Info | `CloudflareCdn` has no throttle (unlike DO and Bunny). Cloudflare's free plan is 1000 purge API calls/day; Bunny got a throttle in the review fixes but Cloudflare did not. This is a minor consistency gap but not a functional regression — CF's limit is per-day not per-second, and the adapter is behind an optional feature for advanced users. Not a blocker. |

---

### Human Verification Required

None. All must-haves are programmatically verified. The throttle timing test (`do_adapter_throttle_serializes`) uses real wall-clock sleep and asserts `elapsed >= Duration::from_secs(9)`, providing strong behavioral proof of the rate-limit invariant without requiring manual testing.

---

## Summary

Phase 188 goal is fully achieved. All 4 ROADMAP success criteria are verified against the actual source code:

1. **SC-1 (cdn_url):** `Storage::cdn_url(path)` and `Disk::cdn_url(path)` implemented with double-slash normalization, env-driven config (`AWS_CDN_URL`), and 4 unit tests (configured, fallback, no-double-slash, via-storage-facade). The `Storage`-level entrypoint delegates to the default disk.

2. **SC-2 (PurgeApi + DO adapter):** `PurgeApi` trait with `async fn purge(&self, paths: &[String])`. `DoSpacesCdn` calls `DELETE /v2/cdn/endpoints/{id}/cache` with `{"files":[...]}`, batches ≤50 paths per request, has a loop-based sliding-window throttle (5 req/10s), treats wildcard paths as 1 slot. All 7 wiremock tests green including the ~10s throttle timing test.

3. **SC-3 (DO config + no-op):** `DoSpacesCdnConfig::from_env()` reads `DO_SPACES_CDN_ID` (optional) and `DIGITALOCEAN_ACCESS_TOKEN`. Missing CDN id → `tracing::info!` + `Ok(())` no-op. `do_adapter_noop_missing_id` asserts zero HTTP requests.

4. **SC-4 (feature gating):** `cdn-bunny` and `cdn-cloudflare` cargo features compile Bunny/CF adapters without entering the default graph. `cargo tree` default and feature builds produce identical crate sets (343 lines each, no new deps from either feature).

All 5 code review fixes (WR-01 through WR-04, IN-01) are present and correct in the code. `thiserror` stays at 1.0. Workspace version is 0.2.46. The storage docs page has a complete CDN section including the B-02 purge-policy note.

---

_Verified: 2026-06-08T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
