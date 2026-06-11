# Phase 204: ferro-storage provider-agnostic CDN configuration — Pattern Map

**Mapped:** 2026-06-11
**Files analyzed:** 7 (5 modified, 1 created, 1 version bump)
**Analogs found:** 7 / 7

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-storage/src/cdn/mod.rs` (modify) | config + service | request-response | `ferro-storage/src/cdn/mod.rs` (existing structs) | exact — adds new types beside existing ones |
| `ferro-storage/src/config.rs` (modify) | config | request-response | `ferro-storage/src/config.rs` lines 119-121 | exact — replace two lines in the same function |
| `ferro-storage/src/error.rs` (modify) | error | — | `ferro-storage/src/error.rs` existing variants | exact |
| `ferro-storage/src/lib.rs` (modify) | re-export | — | `ferro-storage/src/lib.rs` lines 60-65 | exact |
| `ferro-storage/Cargo.toml` (modify) | manifest | — | `ferro-storage/Cargo.toml` lines 27-32 | exact |
| `ferro-storage/CHANGELOG.md` (create) | changelog | — | `ferro-stripe/CHANGELOG.md` | role-match |
| `app/.env.example` (modify) | config | — | `app/.env.example` lines 79-91 (replace) | exact |

---

## Pattern Assignments

### `ferro-storage/src/cdn/mod.rs` — add `Config`, `CdnProvider`, `env_with_fallback`, `build_purge_api`

**Analog:** the existing content of this same file — `DoSpacesCdnConfig` struct + manual `Debug` impl + `from_env` + `DoSpacesCdn::new`.

**Redacted-`Debug` pattern** (lines 67-75 — copy verbatim for `Config`):
```rust
impl std::fmt::Debug for DoSpacesCdnConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoSpacesCdnConfig")
            .field("endpoint_id", &self.endpoint_id)
            .field("api_token", &"<redacted>")
            .field("api_base", &self.api_base)
            .finish()
    }
}
```
For `Config`, mirror this exactly but substitute the struct name and fields:
```rust
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("cdn::Config")
            .field("url", &self.url)
            .field("provider", &self.provider)
            .field("purge_token", &"<redacted>")
            .field("purge_zone", &self.purge_zone)
            .finish()
    }
}
```

**`DoSpacesCdnConfig::from_env` pattern** (lines 82-89 — the env-read style to replicate):
```rust
pub fn from_env() -> Self {
    Self {
        endpoint_id: std::env::var("DO_SPACES_CDN_ID").ok(),
        api_token: std::env::var("DIGITALOCEAN_ACCESS_TOKEN").unwrap_or_default(),
        api_base: None,
    }
}
```
`Config::from_env` replaces direct `env::var` calls with `env_with_fallback` and adds provider inference. Infallible return type (`-> Self`) to avoid changing `StorageConfig::from_env`'s signature (see config.rs pattern below).

**`DoSpacesCdn::new(config)` construction pattern** (lines 103-109 — how the adapter is instantiated):
```rust
pub fn new(config: DoSpacesCdnConfig) -> Self {
    Self {
        config,
        client: reqwest::Client::new(),
        request_times: Mutex::new(VecDeque::new()),
    }
}
```
`build_purge_api` passes a populated `DoSpacesCdnConfig { endpoint_id: self.purge_zone.clone(), api_token: ..., api_base: None }` to this constructor.

**Feature-gate `#[cfg]` module + re-export pattern** (lines 186-194 — the pattern for `build_purge_api` cfg arms):
```rust
#[cfg(feature = "cdn-bunny")]
pub mod bunny;
#[cfg(feature = "cdn-bunny")]
pub use bunny::{BunnyCdn, BunnyCdnConfig};

#[cfg(feature = "cdn-cloudflare")]
pub mod cloudflare;
#[cfg(feature = "cdn-cloudflare")]
pub use cloudflare::{CloudflareCdn, CloudflareCdnConfig};
```
Each `build_purge_api` match arm for Bunny/Cloudflare uses the same `#[cfg(feature = "cdn-bunny")]` / `#[cfg(not(feature = "cdn-bunny"))]` block structure. Structure each arm as two non-overlapping block expressions returning `Result` — not as sequential `return` statements — to avoid the clippy `unreachable_code` lint:
```rust
CdnProvider::Bunny => {
    #[cfg(feature = "cdn-bunny")]
    {
        let cfg = BunnyCdnConfig {
            cdn_base_url: self.url.clone().unwrap_or_default(),
            access_key: self.purge_token.clone().unwrap_or_default(),
        };
        Ok(Some(Box::new(BunnyCdn::new(cfg))))
    }
    #[cfg(not(feature = "cdn-bunny"))]
    {
        Err(crate::Error::cdn_feature_required("bunny", "cdn-bunny"))
    }
}
```

**Test: `debug_does_not_contain_token`** (lines 214-229 — copy structure for `Config` debug test):
```rust
#[test]
fn debug_does_not_contain_token() {
    let config = DoSpacesCdnConfig {
        endpoint_id: Some("ep-123".into()),
        api_token: "secret-token-abc".into(),
        api_base: None,
    };
    let dbg = format!("{config:?}");
    assert!(
        !dbg.contains("secret-token-abc"),
        "Debug output must not contain the token: {dbg}"
    );
    assert!(
        dbg.contains("<redacted>"),
        "Debug output must show <redacted>: {dbg}"
    );
}
```

**`BunnyCdnConfig::from_env` field shapes** (`ferro-storage/src/cdn/bunny.rs` lines 39-44):
```rust
pub fn from_env() -> Self {
    Self {
        cdn_base_url: std::env::var("BUNNY_CDN_URL").unwrap_or_default(),
        access_key: std::env::var("BUNNY_ACCESS_KEY").unwrap_or_default(),
    }
}
```
`BunnyCdnConfig { cdn_base_url, access_key }` — `build_purge_api` populates these from `Config.url` and `Config.purge_token`.

**`CloudflareCdnConfig::from_env` field shapes** (`ferro-storage/src/cdn/cloudflare.rs` lines 36-43):
```rust
pub fn from_env() -> Self {
    Self {
        zone_id: std::env::var("CF_ZONE_ID").unwrap_or_default(),
        api_token: std::env::var("CF_API_TOKEN").unwrap_or_default(),
        cdn_base_url: std::env::var("CF_CDN_URL").unwrap_or_default(),
    }
}
```
`CloudflareCdnConfig { zone_id, api_token, cdn_base_url }` — `build_purge_api` populates: `zone_id` from `Config.purge_zone`, `api_token` from `Config.purge_token`, `cdn_base_url` from `Config.url`.

---

### `ferro-storage/src/config.rs` — replace `AWS_CDN_URL` wiring at line 119

**Analog:** the same file, lines 119-121 (the exact insertion point).

**Current code** (lines 119-121):
```rust
if let Ok(cdn) = env::var("AWS_CDN_URL") {
    s3_config = s3_config.with_cdn_url(cdn);
}
```

**Replacement pattern:**
```rust
// Read the unified CDN config (handles CDN_URL quartet + AWS_CDN_URL fallback + deprecation warn).
let cdn_config = crate::cdn::Config::from_env();
if let Some(cdn_url) = cdn_config.url {
    s3_config = s3_config.with_cdn_url(cdn_url);
}
```
`Config::from_env()` is infallible (`-> Config`) so the `if let Ok` becomes a direct call. This is the minimum-diff approach: no change to `StorageConfig::from_env`'s return type.

**SC-3 parity test baseline** (lines 184-195 — must stay green after the change):
```rust
#[cfg(feature = "s3")]
#[test]
fn from_env_cdn_url() {
    std::env::set_var("AWS_BUCKET", "test-bucket");
    std::env::set_var("AWS_CDN_URL", "https://cdn.test.example.com");
    let config = StorageConfig::from_env();
    let s3_disk = config.get_disk("s3").expect("s3 disk should be configured");
    assert_eq!(
        s3_disk.cdn_url,
        Some("https://cdn.test.example.com".to_string())
    );
    std::env::remove_var("AWS_BUCKET");
    std::env::remove_var("AWS_CDN_URL");
}
```
After the change, `AWS_CDN_URL` becomes a fallback alias for `CDN_URL` in `env_with_fallback`. The test still sets `AWS_CDN_URL` without setting `CDN_URL`, so it exercises the fallback path. The test must continue to pass without modification; the deprecation warn fires but has no test-visible effect.

---

### `ferro-storage/src/error.rs` — add `CdnInvalidProvider` + `CdnFeatureRequired` variants

**Analog:** existing `Cdn(String)` variant with its constructor (lines 44-45 + 74-76).

**Existing variant format** (lines 9-45 — copy struct + constructor style):
```rust
/// CDN operation error.
#[error("CDN error: {0}")]
Cdn(String),
```
```rust
/// Create a CDN error.
pub fn cdn(msg: impl Into<String>) -> Self {
    Self::Cdn(msg.into())
}
```

**New variants to add** (insert after `Cdn(String)`, before the closing `}`):
```rust
/// CDN provider name is not recognized.
#[error("CDN_PROVIDER value '{0}' is not valid; valid values: none, digitalocean, bunny, cloudflare")]
CdnInvalidProvider(String),

/// Selected CDN provider requires a cargo feature that is not enabled.
#[error("CDN_PROVIDER={0} requires the '{1}' cargo feature")]
CdnFeatureRequired(String, &'static str),
```

**New constructors to add** (insert after existing `cdn` constructor in `impl Error`):
```rust
/// Create a CDN invalid provider error.
pub fn cdn_invalid_provider(val: impl Into<String>) -> Self {
    Self::CdnInvalidProvider(val.into())
}

/// Create a CDN feature-required error.
pub fn cdn_feature_required(provider: &str, feature: &'static str) -> Self {
    Self::CdnFeatureRequired(provider.to_string(), feature)
}
```

---

### `ferro-storage/src/lib.rs` — add `cdn::Config` + `CdnProvider` re-exports

**Analog:** lines 60-65 (the existing CDN re-export block).

**Existing re-export block** (lines 60-65):
```rust
pub use cdn::{DoSpacesCdn, DoSpacesCdnConfig, PurgeApi};

#[cfg(feature = "cdn-bunny")]
pub use cdn::{BunnyCdn, BunnyCdnConfig};
#[cfg(feature = "cdn-cloudflare")]
pub use cdn::{CloudflareCdn, CloudflareCdnConfig};
```

**Addition** — append to the unconditional `pub use cdn::{...}` line:
```rust
pub use cdn::{CdnProvider, Config as CdnConfig, DoSpacesCdn, DoSpacesCdnConfig, PurgeApi};
```
`Config as CdnConfig` mirrors the `cdn::Config` public name the CONTEXT specifies while keeping the module-level name `Config` unambiguous within `cdn/mod.rs`.

---

### `ferro-storage/Cargo.toml` — version bump only

**Analog:** `Cargo.toml` (workspace root) line 38.

`ferro-storage/Cargo.toml` uses `version.workspace = true`. The bump is in the **workspace** `Cargo.toml`:

**Current** (workspace `Cargo.toml` line 38):
```toml
version = "0.2.52"
```
**Target:**
```toml
version = "0.2.53"
```
The `cdn-bunny` and `cdn-cloudflare` feature names are already correct (`ferro-storage/Cargo.toml` lines 31-32):
```toml
cdn-bunny = []
cdn-cloudflare = []
```
No new features or dependencies are added.

---

### `ferro-storage/CHANGELOG.md` — create new file

**Analog:** `ferro-stripe/CHANGELOG.md` (the only sibling crate with a CHANGELOG).

**Header structure to copy** (`ferro-stripe/CHANGELOG.md` lines 1-4):
```markdown
# Changelog

All notable changes to `ferro-stripe` are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/).
```

**Entry structure to copy** (`ferro-stripe/CHANGELOG.md` lines 6-32 — the `## [0.9.0]` entry):
```markdown
## [0.9.0] - 2026-06-10

### Added

- ...item...

### Notes

- ...note...
```

**`ferro-storage/CHANGELOG.md` initial content:**
```markdown
# Changelog

All notable changes to `ferro-storage` are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.2.53] - 2026-06-11

### Added

- `cdn::Config` — provider-agnostic CDN configuration struct with fields `url`,
  `provider` (`CdnProvider`), `purge_token` (redacted in `Debug`), `purge_zone`.
- `CdnProvider` enum — `None | DigitalOcean | Bunny | Cloudflare`, parsed
  case-insensitively from `CDN_PROVIDER`. Invalid value → boot error listing
  valid values.
- `Config::from_env()` — reads the unified quartet (`CDN_URL`, `CDN_PROVIDER`,
  `CDN_PURGE_TOKEN`, `CDN_PURGE_ZONE`) as primary, with per-var legacy fallbacks
  emitting a `tracing::warn!` deprecation notice on first use.
- `Config::build_purge_api()` — constructs the active `Box<dyn PurgeApi>` adapter
  from the unified config; returns a clear boot error if the selected provider's
  cargo feature (`cdn-bunny` / `cdn-cloudflare`) is not compiled in.

### Changed

- `StorageConfig::from_env` now sources the S3 disk CDN URL from `cdn::Config`
  (reads `CDN_URL` primary, `AWS_CDN_URL` as deprecated fallback) instead of
  reading `AWS_CDN_URL` directly.

### Deprecated

The following environment variables are deprecated and will be removed in a future
release. They remain as fallbacks for one release. Migrate to the quartet above.

| Deprecated var | Replacement | Notes |
|---|---|---|
| `AWS_CDN_URL` | `CDN_URL` | display URL fallback |
| `BUNNY_CDN_URL` | `CDN_URL` + `CDN_PROVIDER=bunny` | display URL fallback |
| `CF_CDN_URL` | `CDN_URL` + `CDN_PROVIDER=cloudflare` | display URL fallback |
| `DO_SPACES_CDN_ID` | `CDN_PURGE_ZONE` | zone fallback; also infers provider |
| `CF_ZONE_ID` | `CDN_PURGE_ZONE` | zone fallback; also infers provider |
| `DIGITALOCEAN_ACCESS_TOKEN` | `CDN_PURGE_TOKEN` | token fallback |
| `CF_API_TOKEN` | `CDN_PURGE_TOKEN` | token fallback |
| `BUNNY_ACCESS_KEY` | `CDN_PURGE_TOKEN` | token fallback |

### Notes

- Additive: no existing `PurgeApi` adapter signatures changed.
- SC-3 parity: `Disk::cdn_url()` returns byte-identical URLs for `AWS_CDN_URL`-only
  deployments (the legacy fallback path is exercised; only the deprecation warn is new).
- SC-4 parity: legacy DO vars (`DO_SPACES_CDN_ID`, `DIGITALOCEAN_ACCESS_TOKEN`)
  continue to authenticate against the DO Spaces CDN API unchanged.
```

---

## Shared Patterns

### Credential redaction in `Debug`
**Source:** `ferro-storage/src/cdn/mod.rs` lines 67-75, `ferro-storage/src/cdn/bunny.rs` lines 25-32, `ferro-storage/src/cdn/cloudflare.rs` lines 20-28
**Apply to:** `cdn::Config` (the `purge_token` field)
- Implement `std::fmt::Debug` by hand using `f.debug_struct(...)`.
- Token field: `.field("purge_token", &"<redacted>")` — the string literal `"<redacted>"`, not the value.
- All other fields print normally.

### `thiserror` variant + constructor pair
**Source:** `ferro-storage/src/error.rs` lines 9-77
**Apply to:** the two new error variants
- Variant: `#[error("...")]` attribute with the full user-facing message including valid-values.
- Constructor: a short `pub fn` on `impl Error` that calls `Self::VariantName(...)`.
- `&'static str` for the feature-flag parameter in `CdnFeatureRequired` — not `String` — so the compiler can embed the literal.

### Feature-gated `pub use` re-export
**Source:** `ferro-storage/src/lib.rs` lines 62-65
**Apply to:** the new `CdnConfig` / `CdnProvider` exports in `lib.rs`
- Unconditional types (`Config`, `CdnProvider`) go on the existing unconditional `pub use cdn::{...}` line.
- Feature-gated types stay on their own `#[cfg(feature = "...")]` lines as already established.

### Env-var test isolation
**Source:** `ferro-storage/src/config.rs` lines 184-195 (`from_env_cdn_url`)
**Apply to:** all new `#[test]` functions that call `std::env::set_var`
- Pattern: `set_var` at the top of the test function, `remove_var` at the bottom (even on the happy path).
- Use the exact env var names the production code reads (not aliases), unless testing the fallback path specifically.
- No parallel isolation framework needed for this crate; env var pollution risk is low given the unique quartet names (`CDN_URL`, `CDN_PROVIDER`, `CDN_PURGE_TOKEN`, `CDN_PURGE_ZONE`) not used by any other test today.

---

## No Analog Found

All files have close analogs. No RESEARCH.md-only patterns required.

---

## Metadata

**Analog search scope:** `ferro-storage/src/` (all files), `ferro-stripe/CHANGELOG.md` (sibling crate)
**Files scanned:** 8 source files + 2 CHANGELOG files
**Pattern extraction date:** 2026-06-11
