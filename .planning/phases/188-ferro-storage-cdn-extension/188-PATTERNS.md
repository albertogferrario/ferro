# Phase 188: ferro-storage CDN Extension - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 9 (7 modified/new source files + Cargo.toml + docs page)
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-storage/src/facade.rs` | facade/config | request-response | itself — existing `url`/`with_url()` block | exact (same file) |
| `ferro-storage/src/config.rs` | config | batch (env-read) | itself — existing `AWS_*` block in `from_env()` | exact (same file) |
| `ferro-storage/src/error.rs` | error | — | itself — existing variants + constructors | exact (same file) |
| `ferro-storage/src/lib.rs` | module root | — | itself — existing `#[cfg(feature="s3")]` re-export | exact (same file) |
| `ferro-storage/src/cdn/mod.rs` | service/adapter | request-response + rate-limited | `ferro-whatsapp/src/client.rs` (reqwest + bearer_auth + json) + `ferro-storage/src/storage.rs` (async_trait) | role-match (best external analog) |
| `ferro-storage/src/cdn/bunny.rs` | service/adapter | request-response | `ferro-storage/src/cdn/mod.rs` (DO sibling) + `ferro-storage/src/drivers/mod.rs` (cfg-feature gate) | role-match |
| `ferro-storage/src/cdn/cloudflare.rs` | service/adapter | request-response | `ferro-storage/src/cdn/mod.rs` (DO sibling) + `ferro-storage/src/drivers/mod.rs` (cfg-feature gate) | role-match |
| `ferro-storage/Cargo.toml` | build config | — | itself — existing `[features] s3 = [...]` block | exact (same file) |
| `ferro-storage/tests/cdn_*.rs` | test | request-response (wiremock) | `ferro-storage/tests/s3_integration.rs` (feature-gated integration test pattern) | role-match |

---

## Pattern Assignments

### `ferro-storage/src/facade.rs` (MODIFY — add `cdn_url` field, builder, method)

**Analog:** itself, lines 13–87 (DiskConfig) and lines 296–399 (Disk)

**Field addition pattern — DiskConfig struct** (lines 13–27):
```rust
// Existing struct — add cdn_url alongside url using the same Option<String> shape
#[derive(Debug, Clone)]
pub struct DiskConfig {
    pub driver: DiskDriver,
    pub root: Option<String>,
    pub url: Option<String>,      // ← existing field; cdn_url mirrors this exactly
    // ... cfg-gated s3 fields ...
}
```
The `Default` impl at lines 41–53 sets `url: None`; `cdn_url: None` follows the same initialization.

**Builder method pattern — `with_url`** (lines 83–86):
```rust
pub fn with_url(mut self, url: impl Into<String>) -> Self {
    self.url = Some(url.into());
    self
}
```
Copy verbatim for `with_cdn_url`:
```rust
pub fn with_cdn_url(mut self, cdn_url: impl Into<String>) -> Self {
    self.cdn_url = Some(cdn_url.into());
    self
}
```

**Disk struct** (lines 297–299) — add `cdn_url: Option<String>` field alongside `driver`:
```rust
pub struct Disk {
    driver: Arc<dyn StorageDriver>,
    // NEW
    cdn_url: Option<String>,
}
```

**`Disk::new` signature** (lines 303–306) — currently `fn new(driver: Arc<dyn StorageDriver>) -> Self`. Add second param:
```rust
pub fn new(driver: Arc<dyn StorageDriver>, cdn_url: Option<String>) -> Self {
    Self { driver, cdn_url }
}
```

**`Disk::url` method pattern** (lines 366–368) — the fallback the CDN method calls:
```rust
pub async fn url(&self, path: &str) -> Result<String, Error> {
    self.driver.url(path).await
}
```
The CDN method wraps this:
```rust
pub async fn cdn_url(&self, path: &str) -> Result<String, Error> {
    match &self.cdn_url {
        Some(base) => {
            let path = path.trim_start_matches('/');
            Ok(format!("{}/{}", base.trim_end_matches('/'), path))
        }
        None => self.url(path).await,
    }
}
```
The double-slash fix (`trim_end_matches('/') + trim_start_matches('/')`) is already established by `S3Driver::url` at `ferro-storage/src/drivers/s3.rs` line 267.

**`Storage::url` delegation pattern** (lines 291–293):
```rust
pub async fn url(&self, path: &str) -> Result<String, Error> {
    self.default_disk()?.url(path).await
}
```
`Storage::cdn_url` mirrors this:
```rust
pub async fn cdn_url(&self, path: &str) -> Result<String, Error> {
    self.default_disk()?.cdn_url(path).await
}
```

**`Storage::disk` construction** (lines 220–228) — currently returns `Disk { driver }`. Must be updated to thread `cdn_url` through. The `create_driver` call in `with_config` (line 140) and `with_storage_config` (line 175) discards `DiskConfig`; those call sites must also pass `cdn_url`:
```rust
// Current (line 228):
Ok(Disk { driver })

// New pattern — pass cdn_url from DiskConfig at the call site:
Ok(Disk { driver, cdn_url: config.cdn_url.clone() })
```
The `register_disk` path (line 237) has no DiskConfig, so it constructs `Disk::new(driver, None)`.

---

### `ferro-storage/src/config.rs` (MODIFY — add `AWS_CDN_URL`)

**Analog:** itself, lines 86–118 (S3 disk block inside `from_env()`)

**env-read pattern for optional URL fields** (lines 72–74):
```rust
if let Ok(url) = env::var("FILESYSTEM_LOCAL_URL") {
    local_config = local_config.with_url(url);
}
```
Apply the same pattern inside the `#[cfg(feature = "s3")]` block after `s3_config.url = public_url`:
```rust
if let Ok(cdn) = env::var("AWS_CDN_URL") {
    s3_config = s3_config.with_cdn_url(cdn);
}
```
This is inside the existing `if let Ok(bucket) = env::var("AWS_BUCKET")` guard (line 87), matching D-03.

---

### `ferro-storage/src/error.rs` (MODIFY — add `Cdn` variant + constructor)

**Analog:** itself, lines 1–68

**Existing variant shape** (lines 29–32) — the S3 variant is the closest match (provider error, String payload, cfg-gated):
```rust
#[cfg(feature = "s3")]
#[error("S3 error: {0}")]
S3(String),
```
The CDN variant is NOT cfg-gated (it covers all three adapters):
```rust
/// CDN operation error.
#[error("CDN error: {0}")]
Cdn(String),
```

**Constructor pattern** (lines 43–67) — every existing variant has a corresponding constructor using `impl Into<String>`:
```rust
pub fn disk_not_configured(disk: impl Into<String>) -> Self {
    Self::DiskNotConfigured(disk.into())
}
```
Add:
```rust
/// Create a CDN error.
pub fn cdn(msg: impl Into<String>) -> Self {
    Self::Cdn(msg.into())
}
```

**thiserror version:** 1.0 (line 4: `use thiserror::Error;`). Do NOT bump to 2.

---

### `ferro-storage/src/lib.rs` (MODIFY — `pub mod cdn;` + re-exports)

**Analog:** itself, lines 49–68

**cfg-gated module + re-export pattern** (lines 55–56):
```rust
#[cfg(feature = "s3")]
pub use drivers::S3Driver;
```

**Plain module declaration pattern** (lines 49–53):
```rust
mod config;
mod drivers;
mod error;
mod facade;
mod storage;
```
Add `pub mod cdn;` here (public because external consumers implement `PurgeApi`).

**Re-exports to add:**
```rust
// Unconditional (default feature set — trait + DO adapter)
pub use cdn::{DoSpacesCdn, DoSpacesCdnConfig, PurgeApi};

// Feature-gated (Bunny / Cloudflare)
#[cfg(feature = "cdn-bunny")]
pub use cdn::BunnyCdn;
#[cfg(feature = "cdn-cloudflare")]
pub use cdn::CloudflareCdn;
```

---

### `ferro-storage/src/cdn/mod.rs` (NEW — PurgeApi trait + DoSpacesCdn adapter)

**Analog 1 (async_trait pattern):** `ferro-storage/src/storage.rs` lines 100–103
```rust
#[async_trait]
pub trait StorageDriver: Send + Sync {
    async fn exists(&self, path: &str) -> Result<bool, Error>;
    // ...
}
```
Mirror for `PurgeApi`:
```rust
use async_trait::async_trait;
use crate::Error;

#[async_trait]
pub trait PurgeApi: Send + Sync {
    async fn purge(&self, paths: &[String]) -> Result<(), Error>;
}
```

**Analog 2 (reqwest client construction):** `ferro-ai/src/classifier/anthropic.rs` lines 24–30
```rust
pub fn new(api_key: String) -> Self {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("failed to build reqwest client");
    Self { client, api_key }
}
```
`DoSpacesCdn::new` follows this: construct `reqwest::Client::new()` once, store on struct. Do NOT rebuild per `purge()` call.

**Analog 3 (bearer_auth + json body):** `ferro-whatsapp/src/client.rs` lines 73–79
```rust
let response = client
    .post(&url)
    .bearer_auth(&config.access_token)
    .json(&payload)
    .send()
    .await
    .map_err(|e| Error::NetworkError(e.to_string()))?;
```
DO adapter uses `.delete()` instead of `.post()`, otherwise same chain:
```rust
let resp = self.client
    .delete(&url)
    .bearer_auth(&self.config.api_token)
    .json(&serde_json::json!({ "files": chunk }))
    .send()
    .await
    .map_err(|e| Error::cdn(e.to_string()))?;
```

**Analog 4 (non-success response handling):** `ferro-whatsapp/src/client.rs` lines 80–97
```rust
let status = response.status();
let body_text = response.text().await.map_err(|e| Error::NetworkError(e.to_string()))?;
if status.is_success() {
    // parse
} else {
    Err(map_response_error(status.as_u16(), &body_text))
}
```
DO adapter checks for exactly 204 (not generic `.is_success()`):
```rust
if resp.status().as_u16() != 204 {
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    return Err(Error::cdn(format!("status {status}: {body}")));
}
```

**Analog 5 (from_env config with optional/required vars):** `ferro-whatsapp/src/config.rs` lines 40–57
```rust
pub fn from_env(is_owner: Box<dyn Fn(&str) -> bool + Send + Sync>) -> Result<Self, Error> {
    let access_token = std::env::var("WHATSAPP_ACCESS_TOKEN")
        .map_err(|_| Error::Config("WHATSAPP_ACCESS_TOKEN not set".into()))?;
    // ...
}
```
`DoSpacesCdnConfig::from_env()` diverges because a missing CDN id is NOT an error (D-10: logged no-op):
```rust
impl DoSpacesCdnConfig {
    pub fn from_env() -> Self {
        Self {
            endpoint_id: std::env::var("DO_SPACES_CDN_ID").ok(),      // optional
            api_token: std::env::var("DIGITALOCEAN_ACCESS_TOKEN")
                .unwrap_or_default(),                                   // required only when id is set
        }
    }
}
```
Missing token when id IS set returns `Error::cdn("DIGITALOCEAN_ACCESS_TOKEN not set")` inside `purge()`.

**Analog 6 (batch chunking in existing driver):** `ferro-storage/src/drivers/s3.rs` lines 411–435
```rust
for chunk in all_keys.chunks(1000) {
    // build delete request per chunk
}
```
DO adapter uses the same `.chunks(N)` pattern with `BATCH_SIZE = 50`.

**Throttle struct shape:** No direct analog in this workspace (hand-rolled). Use `tokio::sync::Mutex<VecDeque<Instant>>` as specified in RESEARCH.md Pattern 3. Key constraint: `tokio::sync::Mutex` (not `std::sync::Mutex`) because the lock is held across an `.await` inside the sleep path.

**Imports for cdn/mod.rs:**
```rust
use crate::Error;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::Instant;
use tokio::sync::Mutex;
```

---

### `ferro-storage/src/cdn/bunny.rs` (NEW, `#[cfg(feature = "cdn-bunny")]`)

**Analog:** `ferro-storage/src/drivers/mod.rs` lines 9–12 (cfg-gated module pattern):
```rust
#[cfg(feature = "s3")]
mod s3;
#[cfg(feature = "s3")]
pub use s3::S3Driver;
```
The file itself is compiled only under its feature; within `cdn/mod.rs` it is conditionally declared:
```rust
#[cfg(feature = "cdn-bunny")]
pub mod bunny;
#[cfg(feature = "cdn-bunny")]
pub use bunny::BunnyCdn;
```

**HTTP pattern:** mirrors the DO adapter's reqwest + `.json()` call, but uses `.post()` with query params instead of `.delete()` with a body. No internal batching — Bunny's API is per-URL (see RESEARCH.md Pattern 5). The `BunnyCdnConfig` stores `cdn_base_url: String` and `access_key: String`.

---

### `ferro-storage/src/cdn/cloudflare.rs` (NEW, `#[cfg(feature = "cdn-cloudflare")]`)

**Analog:** same as `bunny.rs` above (sibling cfg-gated adapter). Uses `.post()` + `.bearer_auth()` + `.json()` — closest to DO adapter shape. Requires full URLs (`cdn_base_url` stored in config, prepended to each path before building body).

---

### `ferro-storage/Cargo.toml` (MODIFY)

**Analog:** itself, lines 12–30

**Existing optional dep pattern** (lines 22–24):
```rust
aws-sdk-s3 = { version = "1", optional = true }
aws-config = { version = "1", optional = true }
aws-credential-types = { version = "1", features = ["hardcoded-credentials"], optional = true }
```
`reqwest` goes in `[dependencies]` (NOT optional) per D-05 — the DO adapter is in the default graph:
```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```
Note: ferro-whatsapp uses `features = ["json"]` (line 14 of its Cargo.toml) which pulls native-tls. ferro-storage uses `default-features = false, features = ["json", "rustls-tls"]` to avoid the native-tls pull.

**Existing feature pattern** (lines 26–29):
```toml
[features]
default = []
s3 = ["aws-sdk-s3", "aws-config", "aws-credential-types"]
s3-tests = ["s3"]
```
Add:
```toml
cdn-bunny = []        # no extra dep; uses reqwest already in default
cdn-cloudflare = []   # same
```

**tokio feature addition** — current: `features = ["fs", "io-util"]` (line 14). Add `"time"`:
```toml
tokio = { version = "1", features = ["fs", "io-util", "time"] }
```

**wiremock dev-dep** (no existing analog — first wiremock usage in workspace):
```toml
[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }   # already present
tempfile = "3"                                                  # already present
wiremock = "0.6.5"                                             # NEW
```

---

### `ferro-storage/tests/` (NEW — cdn_url unit + wiremock DO adapter tests)

**Analog:** `ferro-storage/tests/s3_integration.rs`

**Feature gate pattern** (line 11):
```rust
#![cfg(feature = "s3-tests")]
```
New CDN tests use a `tokio::test` harness with wiremock — no feature gate needed since the DO adapter is in default. Bunny/CF compilation tests can use `#[cfg(feature = "cdn-bunny")]` guards.

**Test function shape** (lines 32–50):
```rust
#[tokio::test]
async fn test_put_get_delete() {
    let Some(disk) = s3_disk_or_skip() else { return; };
    // ... arrange / act / assert
}
```
wiremock tests follow the same `#[tokio::test]` shape; no skip guard needed (mock server, no live credentials).

**wiremock server setup** — no existing workspace analog; use RESEARCH.md Pattern (lines 550–574) directly:
```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path_regex, header};

#[tokio::test]
async fn test_do_adapter_batches_over_50_paths() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path_regex(r"/v2/cdn/endpoints/test-id/cache"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(204))
        .expect(2)   // 55 paths → 50 + 5 = 2 requests
        .mount(&server)
        .await;
    // build DoSpacesCdn with server.uri() as api_base override
    // call purger.purge(&paths).await.unwrap()
}
```
The `api_base` override field on `DoSpacesCdnConfig` (see Open Question 1 in RESEARCH.md): use `pub(crate) api_base: Option<String>` so it's available in tests without polluting the public API.

---

## Shared Patterns

### async_trait on trait definitions
**Source:** `ferro-storage/src/storage.rs` lines 100–103
**Apply to:** `PurgeApi` trait in `cdn/mod.rs`
```rust
#[async_trait]
pub trait StorageDriver: Send + Sync {
    async fn exists(&self, path: &str) -> Result<bool, Error>;
}
```

### reqwest Client — construct once, store on struct
**Source:** `ferro-ai/src/classifier/anthropic.rs` lines 24–30; `ferro-whatsapp/src/client.rs` lines 32–34
**Apply to:** `DoSpacesCdn`, `BunnyCdn`, `CloudflareCdn` structs
```rust
// from ferro-ai
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .build()
    .expect("failed to build reqwest client");
Self { client, api_key }

// from ferro-whatsapp — simpler, no timeout
let client = reqwest::Client::new();
```
For CDN adapters, `reqwest::Client::new()` is sufficient (no timeout needed for purge operations; failures return `Error::Cdn`).

### bearer_auth + json body (reqwest 0.12)
**Source:** `ferro-whatsapp/src/client.rs` lines 73–79
**Apply to:** `DoSpacesCdn::purge` and `CloudflareCdn::purge` (both use Bearer token auth)
```rust
client
    .post(&url)
    .bearer_auth(&config.access_token)
    .json(&payload)
    .send()
    .await
    .map_err(|e| Error::NetworkError(e.to_string()))?;
```
Bunny uses a custom header instead:
```rust
client
    .post(purge_url)
    .query(&[("url", &full_url), ("async", &"false".to_string())])
    .header("AccessKey", &access_key)
    .send()
    .await
    .map_err(|e| Error::cdn(e.to_string()))?;
```

### from_env() config reading
**Source:** `ferro-whatsapp/src/config.rs` lines 40–57 (required vars → `map_err`) + `ferro-storage/src/config.rs` lines 72–74 (optional vars → `.ok()`)
**Apply to:** `DoSpacesCdnConfig::from_env()`, `BunnyCdnConfig::from_env()`, `CloudflareCdnConfig::from_env()`
- Optional env var (no CDN configured is valid): `std::env::var("DO_SPACES_CDN_ID").ok()`
- Required env var (missing is a structured error): `std::env::var("DIGITALOCEAN_ACCESS_TOKEN").map_err(|_| Error::cdn("DIGITALOCEAN_ACCESS_TOKEN not set"))?`

### Error constructor helper
**Source:** `ferro-storage/src/error.rs` lines 43–67
**Apply to:** all CDN error sites — use `Error::cdn(msg)` not `Error::Cdn(msg.into())` directly
```rust
pub fn cdn(msg: impl Into<String>) -> Self {
    Self::Cdn(msg.into())
}
```

### cfg-gated module + re-export
**Source:** `ferro-storage/src/drivers/mod.rs` lines 9–12; `ferro-storage/src/lib.rs` lines 55–56
**Apply to:** `cdn/mod.rs` (for bunny/cloudflare sub-modules) and `lib.rs` (for re-exports)
```rust
// in cdn/mod.rs:
#[cfg(feature = "cdn-bunny")]
pub mod bunny;
#[cfg(feature = "cdn-bunny")]
pub use bunny::BunnyCdn;

// in lib.rs:
#[cfg(feature = "cdn-bunny")]
pub use cdn::BunnyCdn;
```

### double-slash normalization in URL building
**Source:** `ferro-storage/src/drivers/s3.rs` lines 265–267
```rust
async fn url(&self, path: &str) -> Result<String, Error> {
    let key = normalize_path(path);   // strips leading '/'
    match &self.url_base {
        Some(base) => Ok(format!("{}/{}", base.trim_end_matches('/'), key)),
        None => Ok(format!(".../{}", key)),
    }
}
```
Apply in `Disk::cdn_url`:
```rust
let path = path.trim_start_matches('/');
Ok(format!("{}/{}", base.trim_end_matches('/'), path))
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `ferro-storage/src/cdn/mod.rs` (throttle primitive) | utility | rate-limited | No rate-limiting primitives exist in this workspace. Hand-roll `tokio::sync::Mutex<VecDeque<tokio::time::Instant>>` per RESEARCH.md Pattern 3. |
| wiremock test setup | test | HTTP mock | First wiremock usage in the workspace. No existing test to copy from; use RESEARCH.md wiremock pattern directly. |

---

## Metadata

**Analog search scope:** `ferro-storage/src/`, `ferro-whatsapp/src/`, `ferro-ai/src/`, `ferro-notifications/src/`
**Files read:** 16 source files + 3 Cargo.toml files
**Pattern extraction date:** 2026-06-08
