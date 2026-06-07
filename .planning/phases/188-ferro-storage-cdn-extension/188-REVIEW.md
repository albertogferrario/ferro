---
phase: 188-ferro-storage-cdn-extension
reviewed: 2026-06-08T00:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - ferro-storage/src/cdn/mod.rs
  - ferro-storage/src/cdn/bunny.rs
  - ferro-storage/src/cdn/cloudflare.rs
  - ferro-storage/src/facade.rs
  - ferro-storage/src/config.rs
  - ferro-storage/src/error.rs
  - ferro-storage/src/lib.rs
  - ferro-storage/Cargo.toml
findings:
  critical: 0
  warning: 4
  info: 2
  total: 6
status: issues_found
---

# Phase 188: Code Review Report

**Reviewed:** 2026-06-08
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

The CDN extension is well-structured. Secret redaction is implemented correctly across all three adapters — no derived `Debug` leaks tokens; all hand-written `Debug` impls redact the credential field; no token appears in any `Error::cdn(...)` string or tracing call. TLS is rustls with default-features disabled. Feature gating for `cdn-bunny` and `cdn-cloudflare` is consistent between `cdn/mod.rs` and `lib.rs`.

Four warnings require attention before shipping:

1. The `throttle()` sliding-window does not re-check the slot count after sleeping, leaving a concurrent-caller race where the limit can be exceeded.
2. `BunnyCdn::purge()` violates the `PurgeApi` contract (no rate limiting), risks Bunny API rejection at volume.
3. `CloudflareCdn::purge()` sends all files in a single request; Cloudflare's API rejects more than 30 files per call.
4. `BunnyCdn` and `CloudflareCdn` do not validate required config fields (base URL / zone ID) before making HTTP calls, producing confusing downstream errors.

---

## Warnings

### WR-01: `throttle()` does not re-check slot count after sleep — rate limit can be exceeded under concurrency

**File:** `ferro-storage/src/cdn/mod.rs:130-149`

**Issue:** After `drop(times)` (line 134) and before the re-acquire (line 137), other concurrent callers can acquire the lock and fill remaining slots. When the sleeping task wakes and re-acquires the lock, it re-evicts stale entries but unconditionally pushes a new entry without verifying `times.len() < RATE_LIMIT_MAX`. If concurrent callers consumed all available slots during the sleep, the window limit is exceeded.

For `DoSpacesCdn` as currently used the concurrency risk is low (a single `purge()` call serializes its own chunks), but the invariant is broken for any caller sharing a `DoSpacesCdn` instance across tasks.

**Fix:** Re-check the count after re-eviction; loop back to sleep again if slots are still exhausted:

```rust
async fn throttle(&self) {
    loop {
        let mut times = self.request_times.lock().await;
        let now = Instant::now();
        while times
            .front()
            .map(|t| now.duration_since(*t) >= RATE_LIMIT_WINDOW)
            .unwrap_or(false)
        {
            times.pop_front();
        }
        if times.len() < RATE_LIMIT_MAX {
            times.push_back(Instant::now());
            return;
        }
        // Sleep until the oldest entry expires, then re-check.
        let oldest = *times.front().unwrap();
        let sleep_for = RATE_LIMIT_WINDOW - now.duration_since(oldest);
        drop(times);
        tokio::time::sleep(sleep_for).await;
        // Loop back to re-acquire and re-check.
    }
}
```

---

### WR-02: `BunnyCdn::purge()` has no rate limiting — violates `PurgeApi` contract and risks API rejection

**File:** `ferro-storage/src/cdn/bunny.rs:57-87`

**Issue:** The `PurgeApi` trait doc states "implementations handle batching and rate limiting internally." `BunnyCdn` issues one HTTP request per path with no throttle. Bunny's API enforces a rate limit (1000 req/min per zone). A caller passing hundreds of paths will hit the limit and receive 429 responses; the current code surfaces these as `Error::Cdn("Bunny purge status 429: ...")` with no retry or backoff.

This also makes `BunnyCdn` a footgun compared to `DoSpacesCdn`: the same trait, different safety contract.

**Fix (minimal):** Add a per-call rate limiter (or at minimum document the missing guarantee in the adapter's doc comment). The simplest approach matching the DO adapter's structure:

```rust
// In BunnyCdn, add a Mutex<VecDeque<Instant>> rate_times field, same as DoSpacesCdn.
// Bunny limit: 1000 req/min → ~16/s; a conservative 100/10s window is safe.
```

If the full rate-limit implementation is deferred, at minimum update the `BunnyCdn` doc comment to explicitly document the absence of rate limiting and that callers are responsible for it, so the behavioral difference from `DoSpacesCdn` is visible.

---

### WR-03: `CloudflareCdn::purge()` sends all files in one request — Cloudflare rejects more than 30 files per call

**File:** `ferro-storage/src/cdn/cloudflare.rs:64-102`

**Issue:** Cloudflare's `POST /client/v4/zones/{id}/purge_cache` API rejects requests containing more than 30 URLs per call with a 400 error. The current implementation collects all `full_urls` into a single request body (line 71-80). Any caller passing more than 30 paths gets an opaque 400/API error rather than correct chunked behavior.

**Fix:** Chunk `full_urls` the same way `DoSpacesCdn` chunks paths:

```rust
const CF_BATCH_SIZE: usize = 30;

for chunk in full_urls.chunks(CF_BATCH_SIZE) {
    let resp = self
        .client
        .post(&url)
        .bearer_auth(&self.config.api_token)
        .json(&serde_json::json!({ "files": chunk }))
        .send()
        .await
        .map_err(|e| Error::cdn(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::cdn(format!(
            "Cloudflare purge status {status}: {body}"
        )));
    }
}
```

---

### WR-04: `BunnyCdn` and `CloudflareCdn` do not validate required config fields — produces confusing remote errors

**File:** `ferro-storage/src/cdn/bunny.rs:58-87`, `ferro-storage/src/cdn/cloudflare.rs:64-102`

**Issue:**

- `BunnyCdn::purge()` does not check `config.cdn_base_url.is_empty()`. When `BUNNY_CDN_URL` is not set, `full_url` becomes `"/path/file.js"` (no scheme, no host). Bunny's purge API rejects this with a non-200 status, surfacing as a confusing `Error::Cdn("Bunny purge status NNN: ...")` rather than a clear misconfiguration message.

- `CloudflareCdn::purge()` does not check `config.zone_id.is_empty()` or `config.cdn_base_url.is_empty()`. An empty `zone_id` sends a request to `.../zones//purge_cache` (double-slash), producing a 404 from Cloudflare. An empty `cdn_base_url` produces malformed files like `"/path.js"` with no origin.

**Fix:** Add early validation guards mirroring the `api_token` check:

```rust
// In BunnyCdn::purge()
if self.config.cdn_base_url.is_empty() {
    return Err(Error::cdn("BUNNY_CDN_URL not set"));
}

// In CloudflareCdn::purge()
if self.config.zone_id.is_empty() {
    return Err(Error::cdn("CF_ZONE_ID not set"));
}
if self.config.cdn_base_url.is_empty() {
    return Err(Error::cdn("CF_CDN_URL not set"));
}
```

---

## Info

### IN-01: `register_disk` silently drops CDN URL

**File:** `ferro-storage/src/facade.rs:254-256`

**Issue:** `Storage::register_disk()` always inserts with `cdn_url: None`. There is no API path to register a pre-built driver with a CDN URL through this method. Callers expecting `cdn_url()` to work on a programmatically-registered disk will silently get the origin fallback.

**Fix:** Consider adding a `register_disk_with_cdn` overload, or change the signature to accept an optional CDN URL:

```rust
pub fn register_disk(
    &self,
    name: impl Into<String>,
    driver: Arc<dyn StorageDriver>,
    cdn_url: Option<String>,
) {
    self.inner.disks.insert(name.into(), (driver, cdn_url));
}
```

---

### IN-02: `DoSpacesCdnConfig::from_env()` silently accepts empty token when endpoint ID is absent

**File:** `ferro-storage/src/cdn/mod.rs:82-88`

**Issue:** `from_env()` unconditionally calls `unwrap_or_default()` for `DIGITALOCEAN_ACCESS_TOKEN`, storing an empty string when the var is unset. The runtime check at purge time (`api_token.is_empty()`) only fires after `endpoint_id` is also set, so this is not a safety hole — `purge()` returns `Ok(())` (no-op) when `endpoint_id` is `None`. However, an operator who sets `DO_SPACES_CDN_ID` without `DIGITALOCEAN_ACCESS_TOKEN` will see a runtime error on first purge rather than at construction time.

Documenting this "lazy validation" pattern in the `from_env()` doc comment would help operators understand when the error surfaces:

```rust
/// Note: token absence is validated at purge time, not at construction,
/// to support configurations where the CDN endpoint ID is also absent
/// (in which case purge is a no-op and the token is never used).
```

No code change required — this is a documentation improvement.

---

_Reviewed: 2026-06-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
