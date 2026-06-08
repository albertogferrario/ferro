---
phase: 188-ferro-storage-cdn-extension
fixed_at: 2026-06-08T00:00:00Z
review_path: .planning/phases/188-ferro-storage-cdn-extension/188-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 188: Code Review Fix Report

**Fixed at:** 2026-06-08
**Source review:** .planning/phases/188-ferro-storage-cdn-extension/188-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (WR-01, WR-02, WR-03, WR-04, IN-01)
- Fixed: 5
- Skipped: 0

## Fixed Issues

### WR-01: throttle() race under concurrency

**Files modified:** `ferro-storage/src/cdn/mod.rs`
**Commit:** d6f81517
**Applied fix:** Replaced the if/else throttle body with a `loop` that re-acquires the lock and
re-checks `times.len() < RATE_LIMIT_MAX` on every iteration. On finding a free slot it pushes
the timestamp and returns while still holding the lock. On a full window it computes the sleep
duration, drops the lock, sleeps, then loops back. This closes the race where concurrent callers
could both pass the count check before either recorded a timestamp.

### WR-02: BunnyCdn has no rate limiting

**Files modified:** `ferro-storage/src/cdn/bunny.rs`
**Commit:** 07197a9c
**Applied fix:** Added `request_times: Mutex<VecDeque<Instant>>` to `BunnyCdn` and a `throttle()`
method that mirrors the DO adapter's loop-based sliding-window pattern. Window is 100 req/10s —
a conservative bound well below Bunny's documented 1000 req/min limit. `throttle()` is called
before each per-URL POST. Also added unit tests for empty noop, missing key, and missing cdn_url.

### WR-03: CloudflareCdn sends all files in one request

**Files modified:** `ferro-storage/src/cdn/cloudflare.rs`
**Commit:** ef20bfd5 (fmt fix: f2bc9f93)
**Applied fix:** Added `const CF_BATCH_SIZE: usize = 30` and replaced the single POST with a
`for chunk in full_urls.chunks(CF_BATCH_SIZE)` loop, issuing one POST per chunk. Each chunk
contains at most 30 full URLs, matching Cloudflare's documented per-call limit.

### WR-04: Missing empty-config validation in BunnyCdn and CloudflareCdn

**Files modified:** `ferro-storage/src/cdn/bunny.rs`, `ferro-storage/src/cdn/cloudflare.rs`
**Commits:** 07197a9c (Bunny), ef20bfd5 (Cloudflare)
**Applied fix:**
- `BunnyCdn::purge()` now checks `cdn_base_url.is_empty()` and returns
  `Error::cdn("BUNNY_CDN_URL not set")` before any HTTP call.
- `CloudflareCdn::purge()` now checks `zone_id.is_empty()` → `Error::cdn("CF_ZONE_ID not set")`
  and `cdn_base_url.is_empty()` → `Error::cdn("CF_CDN_URL not set")`, both before any HTTP call.
  The existing `api_token` check was retained in first position.

### IN-01: register_disk silently drops CDN URL

**Files modified:** `ferro-storage/src/facade.rs`
**Commit:** e8ab9352
**Applied fix:** Added `Storage::register_disk_with_cdn(name, driver, cdn_url: Option<String>)`
that inserts the disk with the provided CDN URL. The existing `register_disk` is preserved as a
convenience wrapper (now documented as inserting with `cdn_url: None`). Two unit tests verify:
(1) `register_disk_with_cdn(Some(url))` makes `cdn_url()` return the CDN-prefixed URL, and
(2) `register_disk_with_cdn(None)` registers the disk accessibly (falls back to origin).

## Gate Result

All three gate steps passed against `-p ferro-storage --all-features`:

```
cargo fmt --all -- --check          PASS
cargo clippy -p ferro-storage \
  --all-targets --all-features \
  -- -D warnings                    PASS  (0 warnings)
cargo test -p ferro-storage \
  --all-features                    PASS  54 tests, 0 failures (10s)
```

---

_Fixed: 2026-06-08_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
