# Phase 188: ferro-storage CDN Extension - Research

**Researched:** 2026-06-08
**Domain:** CDN URL generation, HTTP cache purge API, Rust async HTTP client, rate limiting primitives
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**cdn_url() — URL generation (STOR-F-01 / criterion 1)**
- D-01: `cdn_url: Option<String>` field on `DiskConfig`, `with_cdn_url()` consuming builder. Facade-level only, not in `StorageDriver` impls.
- D-02: `Disk::cdn_url(path)` and `Storage::cdn_url(path)`: if CDN base configured, return `{cdn_base}/{path}` (one `/`); otherwise fall back to `self.url(path).await`. Signature mirrors `url()` (async, `Result<String, Error>`).
- D-03: `StorageConfig::from_env()` reads CDN base from `AWS_CDN_URL`. Optionally add `FILESYSTEM_{DISK}_CDN_URL` form if cheap.

**PurgeApi trait & feature gating (STOR-F-02 / criteria 2 & 4)**
- D-04: `PurgeApi` is `#[async_trait]` with `async fn purge(&self, paths: &[String]) -> Result<(), Error>`. No new deps in default feature set.
- D-05: DO Spaces adapter + `reqwest` are in the **default** dependency graph (not feature-gated). Lean: `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }`.
- D-06: `BunnyCdn` behind `cdn-bunny` feature; `CloudflareCdn` behind `cdn-cloudflare`. Real lean impls, not stubs.

**DO Spaces adapter operational details**
- D-07: `DELETE https://api.digitalocean.com/v2/cdn/endpoints/{id}/cache` with JSON body `{"files": [<paths>]}` and `Authorization: Bearer {token}`.
- D-08: Batching: ≤50 files per request. Wildcard (`some/dir/*`) counts as 1 file slot.
- D-09: Rate limit: ≤5 req/rolling 10s, internal throttle. Primitive is planner discretion.
- D-10: Config via `DoSpacesCdnConfig::from_env()` reading `DO_SPACES_CDN_ID` + `DIGITALOCEAN_ACCESS_TOKEN`. Missing `DO_SPACES_CDN_ID` → logged no-op.

**Module layout, error handling, finalize**
- D-11: `ferro-storage/src/cdn/mod.rs` (trait + DO adapter), `src/cdn/bunny.rs` (`#[cfg(feature = "cdn-bunny")]`), `src/cdn/cloudflare.rs` (`#[cfg(feature = "cdn-cloudflare")]`).
- D-12: Extend `Error` enum with `Cdn(String)` variant. thiserror **1.0** (do not bump to 2). No `.unwrap()` on network paths.
- D-13: Existing published crate. Bump workspace version `0.2.45 → 0.2.46`. Update `docs/src/features/storage.md`. CI publishes normally (publish-update token works).

### Claude's Discretion
- Exact `PurgeApi::purge` return detail; whether a `purge_all` / wildcard helper is added.
- Throttle primitive (token bucket vs. timestamp ring).
- Exact `reqwest` minor version and whether a shared internal HTTP-client helper is factored for three adapters.
- Whether `cdn_url()` lands on `StorageDriver` trait (facade-only recommended).
- Bunny/Cloudflare exact endpoint/auth shapes.
- Test doubles for HTTP calls (mock server vs. trait-level fake).

### Deferred Ideas (OUT OF SCOPE)
- Signed/temporary CDN URLs.
- CDN endpoint provisioning.
- Automatic purge-on-`delete()`/`put()`.
- Per-key purge policy helpers.
- Lifecycle-aware purge (B-03 coordination).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| STOR-F-01 | `Storage::cdn_url(path)` returns full CDN URL; falls back to origin when no CDN configured | D-01..D-03: facade-level field + builder + env reading confirmed against existing `url`/`with_url()` pattern |
| STOR-F-02 | `PurgeApi` trait + DO Spaces adapter (≤50 batch, 5 req/10s, wildcard); Bunny+CF feature-gated | D-04..D-10: DO API shape verified against official docs; rate-limit primitive researched; Bunny/CF endpoints confirmed |
</phase_requirements>

## Summary

Phase 188 extends the existing `ferro-storage` crate with two capabilities: CDN URL generation (`cdn_url()`) and a cache-purge abstraction (`PurgeApi`). Both are contained within `ferro-storage` — no new crate, no new workspace member.

The CDN URL work is entirely string composition at the facade layer: `DiskConfig` gains a `cdn_url: Option<String>` field mirroring the existing `url`/`with_url()` pattern, and `StorageConfig::from_env()` reads it from `AWS_CDN_URL`. This adds zero dependencies and zero complexity.

The `PurgeApi` work is the heavier lift. The DO Spaces adapter requires `reqwest` for the `DELETE /v2/cdn/endpoints/{id}/cache` call. The workspace already resolves `reqwest` at `0.12.28` (used by `ferro-ai`, `ferro-notifications`, `ferro-whatsapp`, `ferro-mcp`); adding it to `ferro-storage` with `default-features = false, features = ["json", "rustls-tls"]` reuses the same resolution. The internal throttle (≤5 req/10 s) is implemented as a hand-rolled sliding-window timestamp ring using `tokio::time::sleep` — no new crate dependency. Tests use `wiremock` (already a popular async mock server in the Rust ecosystem) to assert request shape, batching, and the no-op behavior without hitting the live API.

**Primary recommendation:** Implement exactly as specified in CONTEXT.md D-01..D-13. The DO adapter is the compressive unit of work; `cdn_url()` is the cheap enabler. Bunny and Cloudflare adapters are "real but lean" feature-gated companions.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CDN URL generation (`cdn_url()`) | Storage facade (`Disk` / `Storage`) | — | URL is a presentation concern over the stored path; orthogonal to the driver; no driver-level change needed |
| CDN cache purge (`PurgeApi`) | Storage crate (new `cdn` module) | Consumer orchestration | The purge API call is a CDN operation against stored paths; lives alongside storage, not in the HTTP handler or deployment crate |
| Rate limiting (≤5 req/10s) | `cdn/mod.rs` `DoSpacesCdn` struct | — | Consumer must never manage this; adapter owns the internal throttle |
| Config reading (`from_env()`) | `cdn/mod.rs` `DoSpacesCdnConfig` | `config.rs` `StorageConfig::from_env()` | Mirrors existing `from_env()` pattern; `DoSpacesCdnConfig` reads its own vars, `StorageConfig` reads `AWS_CDN_URL` for the disk |
| Feature gating (Bunny/CF) | `Cargo.toml` `[features]` | `cdn/bunny.rs`, `cdn/cloudflare.rs` | Same pattern as existing `s3 = ["aws-sdk-s3"]` feature gate |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12.28 (already in workspace) | HTTP client for DO/Bunny/CF purge API calls | Already used by ferro-ai, ferro-notifications, ferro-mcp, ferro-whatsapp with same version constraint; no new resolution |
| async-trait | 0.1 (already in ferro-storage) | `#[async_trait]` on `PurgeApi` | Already a dep; already re-exported from `lib.rs` |
| tokio | 1 (already in ferro-storage) | `tokio::time::{Instant, sleep}` for throttle | Already a dep with `fs` + `io-util` features; need to add `time` feature |
| thiserror | 1.0 (already in ferro-storage) | `Cdn(String)` error variant | Already at 1.0; D-12 locks it there |
| serde_json | 1 (already in ferro-storage) | JSON body `{"files":[...]}` | Already a dep |
| tracing | 0.1 (already in ferro-storage) | Log no-op purge when `DO_SPACES_CDN_ID` missing | Already a dep |

### Supporting (dev-dependencies)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| wiremock | 0.6.5 | HTTP mock server for DO adapter tests | Assert request shape (DELETE + JSON body), batching, wildcard slot accounting, no-op path — all without hitting real DO API |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled timestamp ring throttle | `governor` crate (0.10.4) | `governor` is well-tested but adds a dependency and is designed for higher-throughput scenarios; 5 req/10 s is trivially served by a `VecDeque<Instant>` + `tokio::time::sleep` with zero extra deps |
| `wiremock` | `httpmock` (0.8.3) | Both work; `wiremock` is more idiomatic with tokio and the zero-config `MockServer` is simpler for our assert-request-shape use case; `httpmock` requires explicit server spawn boilerplate |
| `wiremock` | Trait-level request-sender seam | A trait seam would require making `DoSpacesCdn` generic over a sender type, polluting the public API; `wiremock` keeps the production struct concrete |

**Cargo.toml additions:**

```toml
# ferro-storage/Cargo.toml

[dependencies]
# ... existing deps unchanged ...
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
# tokio already present — add "time" to its feature list:
tokio = { version = "1", features = ["fs", "io-util", "time"] }

[features]
default = []
s3 = ["aws-sdk-s3", "aws-config", "aws-credential-types"]
s3-tests = ["s3"]
cdn-bunny = []        # no extra dep; uses the reqwest already in default
cdn-cloudflare = []   # same

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
tempfile = "3"
wiremock = "0.6.5"
```

**Version verification:**
- reqwest 0.12.28 — confirmed from `Cargo.lock` in this repo [VERIFIED: Cargo.lock]
- wiremock 0.6.5 — confirmed from `cargo search wiremock` [VERIFIED: crates.io registry]

## Architecture Patterns

### System Architecture Diagram

```
Consumer (gestiscilo Phase 190)
        │
        ├── storage.cdn_url("path/to/file.html")
        │       │
        │       └── DiskConfig.cdn_url.is_some()?
        │               YES → format!("{cdn_base}/{path}")   [pure string]
        │               NO  → driver.url(path).await         [origin fallback]
        │
        └── purger.purge(&["path/to/index.html", "path/to/*.html"])
                │
                └── DoSpacesCdn
                        │
                        ├── DoSpacesCdnConfig.endpoint_id.is_none?
                        │       YES → tracing::info!("no CDN id"); return Ok(())
                        │
                        ├── paths.chunks(50)   [≤50 files/request]
                        │       │
                        │       └── for each chunk:
                        │               ┌── throttle (≤5 req/10s rolling window)
                        │               │       │
                        │               │       └── VecDeque<Instant> — slide out old entries
                        │               │           if len >= 5: sleep(10s - oldest_age)
                        │               │
                        │               └── reqwest DELETE
                        │                   URL:  https://api.digitalocean.com/v2/cdn/endpoints/{id}/cache
                        │                   Body: {"files": ["path1", "path2", ...]}
                        │                   Auth: Authorization: Bearer {token}
                        │                   ← 204 No Content (success)
                        │                   ← non-204 → Error::Cdn(status + body)
                        │
                        └── Ok(())
```

### Recommended Project Structure

```
ferro-storage/src/
├── cdn/
│   ├── mod.rs          # PurgeApi trait + DoSpacesCdn + DoSpacesCdnConfig
│   ├── bunny.rs        # #[cfg(feature = "cdn-bunny")]  BunnyCdn
│   └── cloudflare.rs   # #[cfg(feature = "cdn-cloudflare")]  CloudflareCdn
├── config.rs           # add AWS_CDN_URL reading for s3 disk
├── drivers/
│   └── s3.rs           # unchanged
├── error.rs            # add Error::Cdn(String) variant
├── facade.rs           # add DiskConfig.cdn_url + with_cdn_url() + Disk::cdn_url() + Storage::cdn_url()
├── lib.rs              # pub mod cdn; pub use cdn::{PurgeApi, DoSpacesCdn, DoSpacesCdnConfig};
│                       # #[cfg(feature="cdn-bunny")] pub use cdn::BunnyCdn;
│                       # #[cfg(feature="cdn-cloudflare")] pub use cdn::CloudflareCdn;
└── storage.rs          # unchanged
```

### Pattern 1: cdn_url() Implementation

`Disk` does not hold `cdn_url` today (it only holds the `Arc<dyn StorageDriver>`). The `cdn_url` field must be stored on `Disk` itself — introduced when a `Disk` is constructed from a `DiskConfig` that has the field set.

```rust
// facade.rs — DiskConfig additions
#[derive(Debug, Clone)]
pub struct DiskConfig {
    // ... existing fields ...
    /// CDN base URL for generating edge URLs.
    pub cdn_url: Option<String>,
}

impl DiskConfig {
    pub fn with_cdn_url(mut self, cdn_url: impl Into<String>) -> Self {
        self.cdn_url = Some(cdn_url.into());
        self
    }
}

// facade.rs — Disk struct additions
pub struct Disk {
    driver: Arc<dyn StorageDriver>,
    cdn_url: Option<String>,  // NEW
}

impl Disk {
    /// Returns the CDN edge URL for a stored path.
    ///
    /// Falls back to the origin URL when no CDN base is configured.
    pub async fn cdn_url(&self, path: &str) -> Result<String, Error> {
        match &self.cdn_url {
            Some(base) => {
                let path = path.trim_start_matches('/');
                Ok(format!("{}/{}", base.trim_end_matches('/'), path))
            }
            None => self.url(path).await,
        }
    }
}

// Storage::cdn_url delegates to default_disk().cdn_url()
impl Storage {
    pub async fn cdn_url(&self, path: &str) -> Result<String, Error> {
        self.default_disk()?.cdn_url(path).await
    }
}
```

**Critical:** `Disk::new()` currently takes only `Arc<dyn StorageDriver>`. It must be updated to also accept `cdn_url: Option<String>`. The internal `create_driver()` path in `Storage::with_config()` and `Storage::with_storage_config()` must pass `config.cdn_url.clone()` when constructing `Disk`.

### Pattern 2: PurgeApi Trait

```rust
// cdn/mod.rs
use crate::Error;
use async_trait::async_trait;

/// Cache invalidation abstraction for CDN backends.
#[async_trait]
pub trait PurgeApi: Send + Sync {
    /// Purge cached content at the given paths.
    ///
    /// Paths may include wildcards (`path/to/dir/*`).
    /// Implementations handle batching and rate limiting internally.
    async fn purge(&self, paths: &[String]) -> Result<(), Error>;
}
```

### Pattern 3: DO Spaces Adapter — Complete Shape

```rust
// cdn/mod.rs
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::Instant;

const DO_CDN_API: &str = "https://api.digitalocean.com/v2/cdn/endpoints";
const BATCH_SIZE: usize = 50;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(10);
const RATE_LIMIT_MAX: usize = 5;

pub struct DoSpacesCdnConfig {
    pub endpoint_id: Option<String>,
    pub api_token: String,
}

impl DoSpacesCdnConfig {
    pub fn from_env() -> Self {
        Self {
            endpoint_id: std::env::var("DO_SPACES_CDN_ID").ok(),
            api_token: std::env::var("DIGITALOCEAN_ACCESS_TOKEN").unwrap_or_default(),
        }
    }
}

pub struct DoSpacesCdn {
    config: DoSpacesCdnConfig,
    client: reqwest::Client,
    // Internal throttle state — Mutex<VecDeque<Instant>>
    request_times: tokio::sync::Mutex<VecDeque<Instant>>,
}

// The purge() impl:
// 1. If endpoint_id.is_none() → log info, return Ok(())
// 2. chunks(BATCH_SIZE) over paths
// 3. For each chunk: throttle_check().await, then DELETE request
// 4. Non-204 response → Error::Cdn(format!("DO CDN purge {status}: {body}"))

#[async_trait]
impl PurgeApi for DoSpacesCdn {
    async fn purge(&self, paths: &[String]) -> Result<(), Error> {
        let Some(id) = &self.config.endpoint_id else {
            tracing::info!("DO_SPACES_CDN_ID not set — purge is a no-op");
            return Ok(());
        };
        let url = format!("{DO_CDN_API}/{id}/cache");
        for chunk in paths.chunks(BATCH_SIZE) {
            self.throttle().await;
            let resp = self.client
                .delete(&url)
                .bearer_auth(&self.config.api_token)
                .json(&serde_json::json!({ "files": chunk }))
                .send()
                .await
                .map_err(|e| Error::cdn(e.to_string()))?;
            if resp.status().as_u16() != 204 {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                return Err(Error::cdn(format!("status {status}: {body}")));
            }
        }
        Ok(())
    }
}
```

**Throttle primitive — sliding-window timestamp ring:**

```rust
impl DoSpacesCdn {
    async fn throttle(&self) {
        let mut times = self.request_times.lock().await;
        let now = Instant::now();
        // Evict entries older than the window
        while times.front().map(|t| now.duration_since(*t) >= RATE_LIMIT_WINDOW).unwrap_or(false) {
            times.pop_front();
        }
        if times.len() >= RATE_LIMIT_MAX {
            // Sleep until the oldest entry falls out of the window
            let oldest = *times.front().unwrap();
            let sleep_for = RATE_LIMIT_WINDOW - now.duration_since(oldest);
            drop(times);
            tokio::time::sleep(sleep_for).await;
            // Re-acquire and re-evict after sleeping
            let mut times = self.request_times.lock().await;
            let now = Instant::now();
            while times.front().map(|t| now.duration_since(*t) >= RATE_LIMIT_WINDOW).unwrap_or(false) {
                times.pop_front();
            }
            times.push_back(Instant::now());
        } else {
            times.push_back(now);
        }
    }
}
```

This adds no new dependencies. `tokio::sync::Mutex` and `tokio::time::Instant`/`sleep` are already in scope via the existing tokio dep (with the `time` feature added).

### Pattern 4: Error Variant Addition

```rust
// error.rs
#[derive(Error, Debug)]
pub enum Error {
    // ... existing variants ...

    /// CDN operation error.
    #[error("CDN error: {0}")]
    Cdn(String),
}

impl Error {
    pub fn cdn(msg: impl Into<String>) -> Self {
        Self::Cdn(msg.into())
    }
}
```

### Pattern 5: Bunny CDN Adapter (feature-gated)

Bunny uses a per-URL POST approach. For N paths: N sequential `POST https://api.bunny.net/purge?url={encoded_url}&async=false` calls with `AccessKey: {key}` header. At the "works, not gold-plated" bar — no internal batching because Bunny's API is per-URL.

```rust
// cdn/bunny.rs  (only compiled with cfg(feature = "cdn-bunny"))
pub struct BunnyCdnConfig {
    pub cdn_base_url: String,  // e.g. "https://myzone.b-cdn.net"
    pub access_key: String,    // BUNNY_ACCESS_KEY env var
}

// purge(): for each path, POST https://api.bunny.net/purge?url={cdn_base_url}/{path}&async=false
// Header: AccessKey: {access_key}
// Success: 200
```

### Pattern 6: Cloudflare CDN Adapter (feature-gated)

Cloudflare uses `POST /zones/{zone_id}/purge_cache` with `{"files": [...full_urls...]}`. Cloudflare requires **full URLs**, not paths — so the adapter needs the base URL to prepend.

```rust
// cdn/cloudflare.rs  (only compiled with cfg(feature = "cdn-cloudflare"))
pub struct CloudflareCdnConfig {
    pub zone_id: String,      // CF_ZONE_ID env var
    pub api_token: String,    // CF_API_TOKEN env var
    pub cdn_base_url: String, // e.g. "https://example.com" — prepended to paths
}

// purge(): POST https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache
// Body: {"files": ["{cdn_base_url}/{path}", ...]}
// Header: Authorization: Bearer {api_token}
// Success: response.success == true (parse JSON body; HTTP status 200)
```

### Anti-Patterns to Avoid

- **Storing `DoSpacesCdn` as `Arc<dyn PurgeApi>`:** fine, but the throttle `Mutex` must be inside the struct, not on a separate `Arc`. The struct is already cheaply cloneable via `Arc::clone` at the call site.
- **Calling `purge()` with full CDN URLs as paths:** DO API expects **relative paths** (e.g. `"index.html"`, not `"https://cdn.example.com/index.html"`). Cloudflare is the opposite — it needs full URLs. Do not conflate the two in the trait contract. The `PurgeApi` trait uses relative paths; Cloudflare adapter prepends the base internally.
- **Panicking on missing `DIGITALOCEAN_ACCESS_TOKEN` when endpoint id IS set:** this is a structured `Error::Cdn("missing token")` not a panic.
- **Building a new `reqwest::Client` per `purge()` call:** construct the client once in `DoSpacesCdn::new()` and keep it on the struct.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP DELETE with JSON body | Custom raw TCP/hyper layer | reqwest + `.json()` + `.bearer_auth()` | reqwest handles connection pooling, TLS, redirect, retry-able errors, header encoding |
| JSON serialization of `{"files":[...]}` | Manual string formatting | `serde_json::json!()` | Already a dep; avoids escaping bugs on paths with special chars |
| Async HTTP mock in tests | Spin up a real DO endpoint | `wiremock` `MockServer` | Port randomization, parallel test safety, declarative request matching |
| Wildcard path validation | Custom parser | Treat all strings as opaque | DO accepts wildcards natively; the adapter's job is to count slots, not validate path syntax |

**Key insight:** The DO rate-limit throttle looks simple (5 req/10 s) but requires correct behavior under concurrent callers: the `tokio::sync::Mutex<VecDeque<Instant>>` ensures serialized access. A naive `Arc<AtomicU32>` counter would fail under bursty concurrent calls.

## Common Pitfalls

### Pitfall 1: Double-slash in cdn_url()

**What goes wrong:** `format!("{cdn_base}/{path}")` where `cdn_base = "https://cdn.example.com/"` and `path = "/index.html"` produces `"https://cdn.example.com//index.html"`.

**Why it happens:** Both sides may carry a slash. The existing `S3Driver::url()` has the same issue and solves it with `base.trim_end_matches('/')` + path `trim_start_matches('/')`.

**How to avoid:** Mirror the exact S3Driver pattern: `format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'))`. Add a unit test: `cdn_url("https://cdn.example.com/", "/index.html")` → `"https://cdn.example.com/index.html"`.

### Pitfall 2: tokio "time" feature missing

**What goes wrong:** `tokio::time::Instant` / `tokio::time::sleep` compile-fail at build with `error[E0433]: failed to resolve: use of undeclared crate or module`.

**Why it happens:** `ferro-storage` currently has `tokio = { version = "1", features = ["fs", "io-util"] }`. The `time` feature is not in the default set and is not pulled by any existing dep in ferro-storage.

**How to avoid:** Add `"time"` to the tokio features list in `ferro-storage/Cargo.toml`. The `[dev-dependencies]` block already has `tokio = { version = "1", features = ["full"] }` so tests would pass — but the main dep would fail. Verify with `cargo build -p ferro-storage` (not just `cargo test`).

### Pitfall 3: Disk struct not carrying cdn_url

**What goes wrong:** `cdn_url()` is added to `Disk` but the `Disk` struct does not store the value — it only wraps `Arc<dyn StorageDriver>`. The method would always fall back to `url()` even when `DiskConfig.cdn_url` is set.

**Why it happens:** `Storage::create_driver()` currently discards all `DiskConfig` fields after constructing the driver. The `cdn_url` field is presentation-layer, not driver-layer, so it must be threaded through to `Disk` separately.

**How to avoid:** Add `cdn_url: Option<String>` to the `Disk` struct. Planner task: update `Storage::create_driver()` (now returns `Arc<dyn StorageDriver>`) to instead return `Disk` directly (it already constructs the driver; returning `Disk` is the natural next step). Alternatively, keep `create_driver()` returning `Arc<dyn StorageDriver>` and construct `Disk { driver, cdn_url: config.cdn_url.clone() }` at each call site.

### Pitfall 4: Throttle not thread-safe under concurrent purge() calls

**What goes wrong:** Two consumers call `purger.purge()` concurrently. Both check the window before either records a new timestamp → both proceed → 6 requests fire in 10 s → DO returns 429.

**Why it happens:** A non-mutex-guarded read-compute-write on the VecDeque would allow concurrent callers to both see `len < 5` before either pushes.

**How to avoid:** Use `tokio::sync::Mutex` (not `std::sync::Mutex`) around the `VecDeque<Instant>`. The lock is held for the entire check-compute-push-sleep sequence per request slot. Since `purge()` is naturally serial within a single promote sequence, contention is low; the mutex overhead is negligible.

### Pitfall 5: DO API returns 204 on success but non-2xx on partial batch failure

**What goes wrong:** DO may return 422 if the `files` array is empty. A batch that results from `paths.chunks(50)` where the last chunk is empty (impossible from `chunks()` semantics but worth being explicit) would cause a spurious error.

**Why it happens:** `[T]::chunks(n)` never yields an empty chunk, but calling `purge(&[])` should be a no-op, not a 422.

**How to avoid:** Short-circuit: `if paths.is_empty() { return Ok(()); }` at the top of `purge()`. Test: `purger.purge(&[])` → `Ok(())` without making any HTTP request.

### Pitfall 6: Bunny purge API shape mismatch

**What goes wrong:** Using the pull-zone tag-purge endpoint (`POST /pullzone/{id}/purgeCache`) instead of the URL-purge endpoint (`POST /purge?url=...`) causes a 404 on paths that have no cache tags.

**Why it happens:** Bunny has two purge mechanisms: tag-based (requires tagged content) and URL-based (works for all content). The lean adapter should use URL-based purge.

**How to avoid:** Use `POST https://api.bunny.net/purge?url={full_url}&async=false` with the `AccessKey` header. Since Bunny needs full URLs, `BunnyCdnConfig` must store the CDN base URL.

## Code Examples

### DO Purge API — Verified Request Shape

```rust
// Source: https://docs.digitalocean.com/reference/api/reference/cdn-endpoints/
// Method: DELETE
// URL: https://api.digitalocean.com/v2/cdn/endpoints/{cdn_id}/cache
// Body: {"files": ["path/to/file.html", "assets/*"]}
// Auth: Authorization: Bearer {DIGITALOCEAN_ACCESS_TOKEN}
// Success: 204 No Content

let resp = client
    .delete(format!("https://api.digitalocean.com/v2/cdn/endpoints/{id}/cache"))
    .bearer_auth(&api_token)
    .json(&serde_json::json!({ "files": chunk }))
    .send()
    .await?;

assert_eq!(resp.status().as_u16(), 204);
```

### Bunny Purge API — Verified Request Shape

```rust
// Source: https://docs.bunny.net/cdn/purge-cache
// Method: POST (no body)
// URL: https://api.bunny.net/purge?url={encoded_url}&async=false
// Auth: AccessKey: {BUNNY_ACCESS_KEY}
// Success: 200

let full_url = format!("{}/{}", cdn_base.trim_end_matches('/'), path.trim_start_matches('/'));
let resp = client
    .post("https://api.bunny.net/purge")
    .query(&[("url", &full_url), ("async", &"false".to_string())])
    .header("AccessKey", &access_key)
    .send()
    .await?;
// 200 = success
```

### Cloudflare Purge API — Verified Request Shape

```rust
// Source: https://developers.cloudflare.com/api/resources/cache/methods/purge/
// Method: POST
// URL: https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache
// Body: {"files": ["https://example.com/path/to/file.html"]}
// Auth: Authorization: Bearer {CF_API_TOKEN}
// Success: 200 + response body {"success": true}
// Note: Cloudflare requires FULL URLs (with scheme+host), not relative paths

let full_urls: Vec<String> = chunk.iter()
    .map(|p| format!("{}/{}", cdn_base.trim_end_matches('/'), p.trim_start_matches('/')))
    .collect();
let resp = client
    .post(format!("https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache"))
    .bearer_auth(&api_token)
    .json(&serde_json::json!({ "files": full_urls }))
    .send()
    .await?;
// Parse response: resp.json::<serde_json::Value>().await?["success"] == true
```

### wiremock Test Pattern for DO Adapter

```rust
// dev-dependency: wiremock = "0.6.5"
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path_regex, header, body_json};

#[tokio::test]
async fn test_do_adapter_batches_over_50_paths() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path_regex(r"/v2/cdn/endpoints/test-id/cache"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(204))
        .expect(2)  // 55 paths → 2 requests (50 + 5)
        .mount(&server)
        .await;

    let config = DoSpacesCdnConfig {
        endpoint_id: Some("test-id".to_string()),
        api_token: "test-token".to_string(),
        api_base: server.uri(),  // override for test
    };
    let purger = DoSpacesCdn::new_with_base(config, server.uri());
    let paths: Vec<String> = (0..55).map(|i| format!("file{i}.html")).collect();
    purger.purge(&paths).await.unwrap();
    // wiremock asserts exactly 2 DELETE requests were made
}
```

Note: `DoSpacesCdn` needs an `api_base: String` field (defaulting to `"https://api.digitalocean.com"`) to allow the test to redirect requests to `wiremock`'s local server.

### cdn_url() Unit Test (no HTTP, no mock server)

```rust
#[tokio::test]
async fn test_cdn_url_with_cdn_configured() {
    let storage = Storage::with_config(
        "s3",
        vec![("s3", DiskConfig::memory()
            .with_url("https://origin.example.com")
            .with_cdn_url("https://cdn.example.com"))],
    );
    let url = storage.disk("s3").unwrap().cdn_url("images/photo.jpg").await.unwrap();
    assert_eq!(url, "https://cdn.example.com/images/photo.jpg");
}

#[tokio::test]
async fn test_cdn_url_falls_back_to_origin() {
    let storage = Storage::with_config(
        "local",
        vec![("local", DiskConfig::memory().with_url("https://origin.example.com"))],
    );
    let url = storage.disk("local").unwrap().cdn_url("images/photo.jpg").await.unwrap();
    assert_eq!(url, "https://origin.example.com/images/photo.jpg");
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| reqwest `rustls-tls` (ring crypto) in 0.12 | reqwest 0.12.28 workspace uses hyper-rustls 0.27.7 with rustls 0.23 + aws-lc-rs | Workspace lockfile current | Both ring and aws-lc-rs are already present in the workspace; adding reqwest with `rustls-tls` adds no new C dependencies beyond what aws-sdk-s3 already pulls |
| reqwest 0.13 "rustls" feature | reqwest 0.12 "rustls-tls" feature | Context7 docs show 0.13 API | D-05 locks us at 0.12 (workspace consistency); 0.12 uses feature name `"rustls-tls"` not `"rustls"` |
| `governor` crate for rate limiting | Hand-rolled VecDeque<Instant> + tokio::time::sleep | — | For ≤5 req/10s, hand-rolled is zero-dep and correct; governor is for high-throughput scenarios |

**Deprecated/outdated:**
- reqwest 0.12 `default-features = true` in ferro-storage: existing workspace crates use `features = ["json"]` which resolves with default TLS (native-tls included). For ferro-storage the explicit `default-features = false, features = ["json", "rustls-tls"]` avoids adding the native-tls pull to this crate's direct dep declaration.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | DO CDN API returns `204 No Content` on success | Code Examples | If DO returns 200, the `!= 204` check fails; fix is trivial (`!= 200`) |
| A2 | Bunny URL-purge endpoint at `api.bunny.net/purge?url=...` accepts relative paths after prepending CDN base | Pattern 5 | If Bunny requires a different URL format, the adapter returns error on first purge; fix is to update URL construction |
| A3 | Cloudflare `purge_cache` returns HTTP 200 with `{"success":true}` JSON rather than 204 | Code Examples | If Cloudflare returns a different success code, the success-check logic needs updating |
| A4 | `tokio::sync::Mutex` (async) is the right primitive for the throttle (not `std::sync::Mutex`) | Pattern 3 | `std::sync::Mutex` would deadlock if held across an `.await` in the sleep path; async Mutex is correct |

## Open Questions

1. **`api_base` override on `DoSpacesCdn` for testability**
   - What we know: `wiremock` runs on a random localhost port; the production URL is hardcoded.
   - What's unclear: whether to expose `api_base` in the public `DoSpacesCdnConfig` struct or use a builder method only visible in tests (`#[cfg(test)]`).
   - Recommendation: Add `pub(crate) api_base: Option<String>` to `DoSpacesCdnConfig`; `DoSpacesCdn` uses it in tests. Avoids polluting the public API with a test-only field.

2. **`Disk::new()` signature change**
   - What we know: currently `Disk::new(driver: Arc<dyn StorageDriver>) -> Self`.
   - What's unclear: whether to add `cdn_url: Option<String>` as a second parameter or use a `Disk` builder.
   - Recommendation: Add the `cdn_url` parameter directly; it mirrors the existing `DiskConfig` pattern. `Disk` is a lightweight handle, not a builder target. Public callers constructing `Disk::new()` directly are rare — the `register_disk` path uses `Arc<dyn StorageDriver>` and those callers get `cdn_url = None` (no CDN).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| reqwest (via Cargo) | DO/Bunny/CF HTTP calls | ✓ | 0.12.28 (workspace lockfile) | — |
| tokio `time` feature | Throttle sleep/Instant | ✓ (tokio 1.x already present, add "time" feature) | 1.x | — |
| wiremock (dev only) | DO adapter tests | ✓ | 0.6.5 (crates.io registry) | httpmock 0.8.3 (fallback) |
| DigitalOcean API | Live integration test | — (not needed for unit tests) | — | wiremock mock |

## Validation Architecture

`workflow.nyquist_validation` is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[tokio::test]`, `#[test]`) + wiremock 0.6.5 |
| Config file | None (cargo test defaults) |
| Quick run command | `cargo test -p ferro-storage` |
| Full suite command | `cargo test --all-features -p ferro-storage` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STOR-F-01 | `cdn_url()` returns CDN URL when configured | unit | `cargo test -p ferro-storage cdn_url` | ❌ Wave 0 |
| STOR-F-01 | `cdn_url()` falls back to origin when unconfigured | unit | `cargo test -p ferro-storage cdn_url_fallback` | ❌ Wave 0 |
| STOR-F-01 | `cdn_url()` normalizes double-slash (trailing slash on base, leading slash on path) | unit | `cargo test -p ferro-storage cdn_url_no_double_slash` | ❌ Wave 0 |
| STOR-F-01 | `AWS_CDN_URL` env var sets CDN base in `StorageConfig::from_env()` | unit | `cargo test -p ferro-storage from_env_cdn_url` | ❌ Wave 0 |
| STOR-F-02 | DO adapter sends DELETE to correct URL with `{"files":[...]}` body | integration (wiremock) | `cargo test -p ferro-storage do_adapter_request_shape` | ❌ Wave 0 |
| STOR-F-02 | DO adapter batches >50 paths into multiple requests | integration (wiremock) | `cargo test -p ferro-storage do_adapter_batches_over_50` | ❌ Wave 0 |
| STOR-F-02 | Wildcard path counts as 1 slot (50 paths + 1 wildcard = 1 request) | integration (wiremock) | `cargo test -p ferro-storage do_adapter_wildcard_slot` | ❌ Wave 0 |
| STOR-F-02 | Missing `DO_SPACES_CDN_ID` → `purge()` returns `Ok(())` (logged no-op) | unit | `cargo test -p ferro-storage do_adapter_noop_missing_id` | ❌ Wave 0 |
| STOR-F-02 | `purge(&[])` → `Ok(())` without any HTTP request | unit | `cargo test -p ferro-storage purge_empty_noop` | ❌ Wave 0 |
| STOR-F-02 | Non-204 response → `Error::Cdn` | integration (wiremock) | `cargo test -p ferro-storage do_adapter_error_on_non_204` | ❌ Wave 0 |
| criterion 4 | Bunny/CF adapters compile behind features | compilation | `cargo build -p ferro-storage --features cdn-bunny,cdn-cloudflare` | ❌ Wave 0 |
| criterion 4 | Default build has no Bunny/CF types in scope | compilation | `cargo build -p ferro-storage` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-storage`
- **Per wave merge:** `cargo test --all-features -p ferro-storage`
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` full CI-parity gate before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-storage/src/cdn/mod.rs` — covers STOR-F-02 (trait + DO adapter)
- [ ] `ferro-storage/src/cdn/bunny.rs` — covers criterion 4 compilation
- [ ] `ferro-storage/src/cdn/cloudflare.rs` — covers criterion 4 compilation
- [ ] Tests in `ferro-storage/src/cdn/mod.rs` `#[cfg(test)]` block — covers all req/criterion rows above
- [ ] `ferro-storage/Cargo.toml` addition of `reqwest`, `tokio time` feature, `cdn-bunny`/`cdn-cloudflare` features, `wiremock` dev-dep

## Security Domain

`security_enforcement` is absent from `.planning/config.json` — treated as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (partial) | Paths passed to `purge()` are forwarded as-is; no injection vector since they are JSON-encoded by serde_json |
| V6 Cryptography | no | API tokens sent over HTTPS via reqwest+rustls; no custom crypto |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| `DIGITALOCEAN_ACCESS_TOKEN` in env logs | Information Disclosure | Never log the token; log only that a purge was attempted and whether it succeeded — `tracing::info!("purged {} paths in {} requests", n, batches)` |
| Wildcard purge (`*`) triggered by consumer-supplied paths | Tampering (accidental) | This is consumer policy (the crate is a mechanism, not a policy enforcer). Doc note: "purging `*` invalidates the entire CDN cache; use only for deployment promotes" |
| Path traversal in purge keys | Tampering | DO accepts relative paths verbatim. serde_json encodes them correctly. No server-side consequence since this is a DELETE-with-body to DO's API, not a filesystem operation |

## Sources

### Primary (HIGH confidence)
- `ferro-storage/src/facade.rs` — existing `DiskConfig`, `Disk`, `Storage` patterns [VERIFIED: read this session]
- `ferro-storage/src/config.rs` — existing `StorageConfig::from_env()` pattern [VERIFIED: read this session]
- `ferro-storage/src/error.rs` — existing `Error` enum, thiserror 1.0 [VERIFIED: read this session]
- `ferro-storage/src/drivers/s3.rs` — double-slash fix pattern, feature gate precedent [VERIFIED: read this session]
- `ferro-storage/Cargo.toml` — existing features, deps [VERIFIED: read this session]
- DigitalOcean CDN Endpoints API — `DELETE /v2/cdn/endpoints/{id}/cache`, `{"files":[...]}`, 204, ≤50/req, 5 req/10 s [VERIFIED: docs.digitalocean.com fetched this session]
- DO rate limits (50 files/20s + 5 req/10s) — two separate limits; the 5-req/10s is the binding constraint for our batching [VERIFIED: docs.digitalocean.com]
- reqwest 0.12.28 in workspace lockfile — `Cargo.lock` [VERIFIED: read this session]
- reqwest 0.12 feature name `"rustls-tls"` — docs.rs/reqwest/0.12.28 [VERIFIED: fetched this session]
- Workspace version 0.2.45 in `Cargo.toml` [VERIFIED: read this session]

### Secondary (MEDIUM confidence)
- Bunny CDN URL purge: `POST https://api.bunny.net/purge?url=...&async=false`, `AccessKey: ...` header [CITED: docs.bunny.net/cdn/purge-cache]
- Cloudflare cache purge: `POST /zones/{zone_id}/purge_cache`, `{"files":[...full_urls...]}`, `Authorization: Bearer ...` [CITED: developers.cloudflare.com/api/resources/cache/methods/purge/]
- wiremock 0.6.5 on crates.io [VERIFIED: cargo search wiremock]
- `tokio::sync::Mutex` required for async-context throttle (not `std::sync::Mutex`) [ASSUMED — well-known tokio pattern]

### Tertiary (LOW confidence)
- Cloudflare success response is `{"success": true}` JSON at HTTP 200 (not 204) [ASSUMED from Cloudflare docs summary; exact status code field in the fetched docs was not spelled out]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — reqwest 0.12 already in workspace at confirmed version; tokio features verified; wiremock confirmed on crates.io
- Architecture: HIGH — facade pattern directly mirrors existing `url`/`with_url()` implementation; DO API shape verified against official docs
- DO API pitfalls: HIGH — exact request shape confirmed (method, URL, body key, auth, 204 success, rate limits)
- Bunny/CF API: MEDIUM — endpoint shapes verified but response parsing details (exact HTTP status) are cited from docs summaries
- Throttle primitive: HIGH — hand-rolled VecDeque<Instant> pattern is standard tokio idiom; governor alternative confirmed but rejected

**Research date:** 2026-06-08
**Valid until:** 2026-09-08 (stable APIs; DO CDN API has been unchanged for years)
