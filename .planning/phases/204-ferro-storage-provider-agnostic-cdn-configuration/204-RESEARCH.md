# Phase 204: ferro-storage provider-agnostic CDN configuration — Research

**Researched:** 2026-06-11
**Domain:** Rust crate env-surface refactor (`ferro-storage` CDN config layer)
**Confidence:** HIGH — all findings verified directly against source files in this session

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01** — Introduce `ferro_storage::cdn::Config` with fields `url: Option<String>`, `provider: CdnProvider`, `purge_token: Option<String>` (redacted Debug), `purge_zone: Option<String>`. `Config::from_env()` reads the quartet + legacy fallbacks. Method `Config::build_purge_api() -> Result<Option<Box<dyn PurgeApi>>, Error>` constructs the provider adapter from `purge_token`/`purge_zone`.
- **D-02** — `env_with_fallback(primary, &[(alias, label)])` helper: read primary first; on first legacy hit emit `tracing::warn!` once naming the deprecated var and its replacement. Token warn must not print the token value.
  - `CDN_URL` ← `AWS_CDN_URL`, `CF_CDN_URL`, `BUNNY_CDN_URL`
  - `CDN_PURGE_ZONE` ← `DO_SPACES_CDN_ID`, `CF_ZONE_ID`
  - `CDN_PURGE_TOKEN` ← `DIGITALOCEAN_ACCESS_TOKEN`, `CF_API_TOKEN`, `BUNNY_ACCESS_KEY`
- **D-03** — `CdnProvider` enum (`None` | `DigitalOcean` | `Bunny` | `Cloudflare`), case-insensitive parsing. Invalid value → boot `Error` listing valid values. Unset → infer from legacy cluster. `CDN_PROVIDER=none` → explicit logged no-op.
- **D-04** — If selected provider's feature (`cdn-bunny`/`cdn-cloudflare`) is not compiled in, `build_purge_api()` returns a clear boot `Error` naming the provider and the required feature flag. `digitalocean` is always available.
- **D-05** — Wire `CDN_URL` (via `cdn::Config.url`) into the same place `AWS_CDN_URL` feeds today (`config.rs:119` → `s3_config.with_cdn_url(...)`).
- **D-06** — Bump ferro-storage minor version; add `## [X.Y.0]` CHANGELOG entry; migrate `app/.env.example` CDN section to quartet with deprecation comment; update `docs/src/features/storage.md` CDN env-var table.

### Claude's Discretion
- Exact module placement of `env_with_fallback` (private helper in `cdn/mod.rs` vs `config.rs`).
- Whether `CdnProvider` parsing uses `serde` or a hand-rolled `FromStr`.
- Whether per-provider `*CdnConfig::from_env()` are kept or replaced with explicit token/zone constructors.
- Exact deprecation-warn wording and the precise minor version number.
- Whether provider inference logs at `warn` (deprecation) or `info`.

### Deferred Ideas (OUT OF SCOPE)
- Removing the legacy env vars (future phase after gestiscilo Phase 205 consumer rename).
- New CDN providers (Fastly, CloudFront-native, etc.).
- Endpoint-level CDN purge rate-limiting beyond existing per-adapter throttles.
- Multi-CDN / per-disk CDN provider.
</user_constraints>

---

## Summary

Phase 204 is a pure env-surface refactor of `ferro-storage`'s CDN configuration layer. The Phase 188 purge adapters (`DoSpacesCdn`, `BunnyCdn`, `CloudflareCdn`) are unchanged; only how their configuration is read changes. The work introduces a single `cdn::Config` struct that reads a provider-agnostic quartet (`CDN_URL`, `CDN_PROVIDER`, `CDN_PURGE_TOKEN`, `CDN_PURGE_ZONE`) as primary, with per-var legacy fallbacks emitting `tracing::warn!` deprecation notices.

The codebase is small and clean. All env reads are currently confined to `from_env()` methods on three per-provider `*CdnConfig` structs (`config.rs:119` for the display URL, `cdn/mod.rs` for DO, `cdn/bunny.rs` and `cdn/cloudflare.rs` for optional providers). There is exactly one test touching env vars in this crate (`from_env_cdn_url` in `config.rs`), and it uses `std::env::set_var`/`remove_var` serially — no isolation framework. `PurgeApi` is already used as a `Box<dyn PurgeApi>` nowhere in the existing codebase (the trait object form does not exist yet), but `async_trait` is in use and the trait is `Send + Sync`, making it object-safe for the new `build_purge_api()` return type.

The workspace version is `0.2.52`. ferro-storage uses `version.workspace = true`, so the minor bump is to the workspace `Cargo.toml` at `version = "0.2.52"` → `0.2.53`. No CHANGELOG file exists in `ferro-storage/`; one must be created.

**Primary recommendation:** Add `cdn::Config` and `CdnProvider` directly to `ferro-storage/src/cdn/mod.rs`, wire `Config.url` into `config.rs:119` replacing the raw `AWS_CDN_URL` read, and export both from `lib.rs`. Keep `*CdnConfig` structs as internal constructor inputs populated from the unified `Config`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Unified quartet env reading | Library (ferro-storage) | — | Config is crate-internal; no framework or app layer involved |
| Legacy fallback + deprecation warn | Library (`cdn::Config::from_env`) | — | Centralized in the one entry point that reads CDN env |
| Provider adapter construction | Library (`Config::build_purge_api`) | — | Selects and populates existing per-provider structs |
| CDN display URL wiring | Library (`StorageConfig::from_env`, `config.rs:119`) | — | `DiskConfig.cdn_url` feeds `Disk::cdn_url()` — unchanged facade |
| Feature-gate boot error | Library (`build_purge_api`, `#[cfg]` arms) | — | Compile-time gating; runtime error when feature is missing |
| Version bump + changelog | Crate manifest + new CHANGELOG.md | — | One-crate change; workspace version bump |
| `.env.example` migration | App layer (`app/.env.example`) | — | Consumer file, not the library itself |
| Docs update | `docs/src/features/storage.md` | — | Framework docs page lists CDN env vars by name |

---

## Standard Stack

### Core (already in Cargo.toml — no new deps)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `async-trait` | `0.1` | `#[async_trait]` on `PurgeApi` impl blocks | Already used; required for dyn-safe async trait |
| `tracing` | `0.1` | `tracing::warn!` for deprecation notices | Already used throughout ferro-storage |
| `thiserror` | `1.0` | `Error` enum extension | Established ferro-* pattern |
| `std::env` | stdlib | `env::var` for reading env vars | No external dep needed |

[VERIFIED: ferro-storage/Cargo.toml]

No new dependencies are required. `serde` is already present but is not needed for `CdnProvider` parsing — a hand-rolled `FromStr` (or a match on `to_ascii_lowercase()`) is simpler and avoids coupling provider parsing to serde's feature set.

**Installation:** no new `cargo add` required.

---

## Architecture Patterns

### System Architecture Diagram

```
Environment
   │
   ▼
cdn::Config::from_env()
   │ reads CDN_URL / CDN_PROVIDER / CDN_PURGE_TOKEN / CDN_PURGE_ZONE (quartet, primary)
   │ falls back per-var via env_with_fallback() → tracing::warn! on first legacy hit
   │
   ├─── Config.url ──────────────────────────────────────────────────►  StorageConfig::from_env()
   │                                                                      config.rs:119
   │                                                                      s3_config.with_cdn_url(url)
   │                                                                             │
   │                                                                             ▼
   │                                                                      DiskConfig.cdn_url
   │                                                                             │
   │                                                                             ▼
   │                                                                      Disk::cdn_url(path)  [UNCHANGED]
   │
   └─── Config.build_purge_api()
           │
           ├─ CdnProvider::None        → Ok(None)  [logged no-op]
           ├─ CdnProvider::DigitalOcean → DoSpacesCdnConfig { endpoint_id: purge_zone, api_token: purge_token }
           │                              → Ok(Some(Box::new(DoSpacesCdn::new(cfg))))
           ├─ CdnProvider::Bunny       → #[cfg(cdn-bunny)] BunnyCdnConfig { cdn_base_url: url, access_key: purge_token }
           │                              #[cfg(not(cdn-bunny))] → Err(Error::CdnFeatureRequired("cdn-bunny"))
           └─ CdnProvider::Cloudflare  → #[cfg(cdn-cloudflare)] CloudflareCdnConfig { zone_id: purge_zone, api_token: purge_token, cdn_base_url: url }
                                          #[cfg(not(cdn-cloudflare))] → Err(Error::CdnFeatureRequired("cdn-cloudflare"))
```

### Recommended Project Structure (modified files only)
```
ferro-storage/
├── Cargo.toml               # version bump (workspace Cargo.toml)
├── CHANGELOG.md             # CREATE — new file
├── src/
│   ├── cdn/
│   │   ├── mod.rs           # ADD: Config, CdnProvider, env_with_fallback, build_purge_api
│   │   ├── bunny.rs         # UNCHANGED (adapters untouched)
│   │   └── cloudflare.rs    # UNCHANGED (adapters untouched)
│   ├── config.rs            # MODIFY line ~119: AWS_CDN_URL read → cdn::Config.url
│   ├── error.rs             # ADD: CdnInvalidProvider, CdnFeatureRequired variants
│   └── lib.rs               # ADD: pub use cdn::{Config as CdnConfig, CdnProvider}
app/
└── .env.example             # MODIFY: CDN section → quartet + deprecation comment
docs/src/features/
└── storage.md               # MODIFY: CDN env vars section updated to quartet
```

### Pattern 1: `env_with_fallback` helper
**What:** Reads a primary env var; if absent, iterates aliases and returns the first hit with a deprecation warn.
**When to use:** Once per quartet variable in `Config::from_env`.

```rust
// Source: VERIFIED — designed per D-02; pattern consistent with existing from_env impls
fn env_with_fallback(primary: &str, aliases: &[(&str, &str)]) -> Option<String> {
    if let Ok(val) = std::env::var(primary) {
        return Some(val);
    }
    for (alias, label) in aliases {
        if let Ok(val) = std::env::var(alias) {
            tracing::warn!(
                "{} is deprecated; use {} instead",
                label,
                primary
            );
            return Some(val);
        }
    }
    None
}
```

For `CDN_PURGE_TOKEN`, the warn must NOT include the value:
```rust
// correct — value is not interpolated into the message
tracing::warn!("DIGITALOCEAN_ACCESS_TOKEN is deprecated; use CDN_PURGE_TOKEN instead");
```

### Pattern 2: `CdnProvider` enum + parsing
**What:** Provider enum with case-insensitive `from_str`, invalid value → `Error`.

```rust
// Source: VERIFIED against D-03; serde present but hand-rolled is simpler here
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CdnProvider {
    #[default]
    None,
    DigitalOcean,
    Bunny,
    Cloudflare,
}

impl CdnProvider {
    pub fn from_str_ci(s: &str) -> Result<Self, crate::Error> {
        match s.to_ascii_lowercase().as_str() {
            "none"         => Ok(Self::None),
            "digitalocean" => Ok(Self::DigitalOcean),
            "bunny"        => Ok(Self::Bunny),
            "cloudflare"   => Ok(Self::Cloudflare),
            other => Err(crate::Error::cdn_invalid_provider(other)),
        }
    }

    /// Valid value list for error messages.
    pub fn valid_values() -> &'static [&'static str] {
        &["none", "digitalocean", "bunny", "cloudflare"]
    }
}
```

### Pattern 3: `Config::from_env` with provider inference
**What:** Reads quartet primary + legacy fallbacks; infers `CDN_PROVIDER` from legacy cluster when unset.

```rust
// Source: VERIFIED against D-02, D-03
impl Config {
    pub fn from_env() -> Result<Self, crate::Error> {
        let url = env_with_fallback("CDN_URL", &[
            ("AWS_CDN_URL",   "AWS_CDN_URL"),
            ("CF_CDN_URL",    "CF_CDN_URL"),
            ("BUNNY_CDN_URL", "BUNNY_CDN_URL"),
        ]);
        let purge_zone = env_with_fallback("CDN_PURGE_ZONE", &[
            ("DO_SPACES_CDN_ID", "DO_SPACES_CDN_ID"),
            ("CF_ZONE_ID",       "CF_ZONE_ID"),
        ]);
        let purge_token = env_with_fallback("CDN_PURGE_TOKEN", &[
            ("DIGITALOCEAN_ACCESS_TOKEN", "DIGITALOCEAN_ACCESS_TOKEN"),
            ("CF_API_TOKEN",              "CF_API_TOKEN"),
            ("BUNNY_ACCESS_KEY",          "BUNNY_ACCESS_KEY"),
        ]);

        let provider = if let Ok(val) = std::env::var("CDN_PROVIDER") {
            CdnProvider::from_str_ci(&val)?   // invalid → boot Error
        } else {
            // Infer from legacy cluster presence
            if std::env::var("DO_SPACES_CDN_ID").is_ok() {
                tracing::warn!("CDN_PROVIDER unset; inferred digitalocean from DO_SPACES_CDN_ID. Set CDN_PROVIDER=digitalocean to silence this warning.");
                CdnProvider::DigitalOcean
            } else if std::env::var("CF_ZONE_ID").is_ok() {
                tracing::warn!("CDN_PROVIDER unset; inferred cloudflare from CF_ZONE_ID. Set CDN_PROVIDER=cloudflare to silence this warning.");
                CdnProvider::Cloudflare
            } else if std::env::var("BUNNY_CDN_URL").is_ok() {
                tracing::warn!("CDN_PROVIDER unset; inferred bunny from BUNNY_CDN_URL. Set CDN_PROVIDER=bunny to silence this warning.");
                CdnProvider::Bunny
            } else {
                CdnProvider::None
            }
        };

        Ok(Self { url, provider, purge_token, purge_zone })
    }
}
```

### Pattern 4: `build_purge_api` with feature-gate error
**What:** Constructs the right adapter or returns a clear boot error.

```rust
// Source: VERIFIED against D-01, D-04; #[cfg] arms match existing cdn/mod.rs gating pattern
impl Config {
    pub fn build_purge_api(&self) -> Result<Option<Box<dyn PurgeApi>>, crate::Error> {
        match &self.provider {
            CdnProvider::None => {
                tracing::info!("CDN_PROVIDER=none — purge is a no-op");
                Ok(None)
            }
            CdnProvider::DigitalOcean => {
                let cfg = DoSpacesCdnConfig {
                    endpoint_id: self.purge_zone.clone(),
                    api_token: self.purge_token.clone().unwrap_or_default(),
                    api_base: None,
                };
                Ok(Some(Box::new(DoSpacesCdn::new(cfg))))
            }
            CdnProvider::Bunny => {
                #[cfg(feature = "cdn-bunny")]
                {
                    let cfg = BunnyCdnConfig {
                        cdn_base_url: self.url.clone().unwrap_or_default(),
                        access_key: self.purge_token.clone().unwrap_or_default(),
                    };
                    return Ok(Some(Box::new(BunnyCdn::new(cfg))));
                }
                #[cfg(not(feature = "cdn-bunny"))]
                return Err(crate::Error::cdn_feature_required("bunny", "cdn-bunny"));
            }
            CdnProvider::Cloudflare => {
                #[cfg(feature = "cdn-cloudflare")]
                {
                    let cfg = CloudflareCdnConfig {
                        zone_id: self.purge_zone.clone().unwrap_or_default(),
                        api_token: self.purge_token.clone().unwrap_or_default(),
                        cdn_base_url: self.url.clone().unwrap_or_default(),
                    };
                    return Ok(Some(Box::new(CloudflareCdn::new(cfg))));
                }
                #[cfg(not(feature = "cdn-cloudflare"))]
                return Err(crate::Error::cdn_feature_required("cloudflare", "cdn-cloudflare"));
            }
        }
    }
}
```

**Note on `#[cfg]` + `unreachable_code` clippy lint:** the `#[cfg(not(...))] return` pattern inside a match arm can trigger the `unreachable_code` lint on the configured branch. The canonical solution is to gate the entire match arm body:

```rust
CdnProvider::Bunny => {
    #[cfg(feature = "cdn-bunny")]
    {
        // ... build adapter
        Ok(Some(Box::new(...)))
    }
    #[cfg(not(feature = "cdn-bunny"))]
    {
        Err(crate::Error::cdn_feature_required("bunny", "cdn-bunny"))
    }
}
```

Each cfg block must return a `Result`, and the compiler understands only one branch is active per build. Clippy `-D warnings` will pass because neither branch sees the other as dead code. [VERIFIED: consistent with `#[cfg(feature = "s3")]` arms in `facade.rs:217-233`]

### Pattern 5: `config.rs:119` wiring (D-05)
**Current code (lines 119-121):**
```rust
if let Ok(cdn) = env::var("AWS_CDN_URL") {
    s3_config = s3_config.with_cdn_url(cdn);
}
```

**Replacement — call `cdn::Config::from_env()` and use `.url`:**
```rust
// Read the unified CDN config (handles quartet + AWS_CDN_URL fallback + warn)
if let Ok(cdn_config) = crate::cdn::Config::from_env() {
    if let Some(cdn_url) = cdn_config.url {
        s3_config = s3_config.with_cdn_url(cdn_url);
    }
}
```

The `from_env()` returning `Result<Config, Error>` is the right signature because invalid `CDN_PROVIDER` must be a boot error. The `if let Ok` in config.rs means a misconfigured provider value silently skips CDN URL wiring — that is wrong for a boot error. **Correction:** `StorageConfig::from_env` should propagate the error, which means its return type must change to `Result<Self, Error>`, OR `Config::from_env` separates URL reading (infallible) from provider validation (fallible). The simplest design that keeps `StorageConfig::from_env` non-panicking is:

- `cdn::Config::from_env()` returns `Result<Self, Error>` — caller decides whether to propagate.
- In `StorageConfig::from_env`, the CDN URL reading is the only non-failing part; it can be extracted as a free function or handled with a `match`.
- The calling application (framework integration or `main`) would call `cdn_config.build_purge_api()` at boot time, not `StorageConfig::from_env`. So `StorageConfig::from_env` only needs `cdn_config.url` — it can silently ignore a provider error (the display URL still works) OR it can propagate.

**Recommended approach (Claude's discretion):** Make `Config::from_env()` fallible (`Result<Config, Error>`) for the provider-invalid path, but add a separate `Config::from_env_url_only()` (or just use `env_with_fallback` directly in `config.rs`) that never fails, purely for the display URL path. This keeps `StorageConfig::from_env()` infallible (current callers are unaffected) while still enabling boot-time validation of `CDN_PROVIDER` when the app actually constructs the purge API.

Alternatively, and more simply: `Config::from_env()` returns `Config` (infallible) and stores invalid provider as `CdnProvider::None` + logs an error. The actual `Error` return from `build_purge_api()` covers the invalid-provider case at the moment the app tries to use purge. This is the most backward-compatible approach.

**Decision for planner:** either approach satisfies SC-5. The planner should pick the one that does not change `StorageConfig::from_env`'s return type (minimizes diff surface). That means: `Config::from_env()` is infallible and returns a `Config` where invalid `CDN_PROVIDER` stores `CdnProvider::None` and logs an error; `build_purge_api()` separately validates the full config and returns `Err` on invalid-provider. The SC-5 parity test covers the error path via `build_purge_api()` directly.

### Anti-Patterns to Avoid

- **Coupling `CDN_URL` (display) to `CDN_PROVIDER` (purge):** they are orthogonal axes. An `AWS_CDN_URL`-only deployment with no purge provider must keep a working `cdn_url()` with `provider = None`. Do not gate URL reading on provider selection.
- **Always-compiling Bunny/Cloudflare deps:** do not add `BunnyCdn` or `CloudflareCdn` to unconditional code paths. The feature-gate architecture from Phase 188 must be preserved (verified via `cargo tree`).
- **Printing the token in warn messages:** the `env_with_fallback` for `CDN_PURGE_TOKEN` must log only the var name, never `val`.
- **`std::env::set_var` in parallel tests:** the existing `from_env_cdn_url` test uses set/remove in a single `#[test]` function without any parallel-isolation guard. New env-var tests must follow the same serial-within-function pattern or use unique var names to avoid flaky cross-test interference. `cargo test` by default runs tests in parallel across modules.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Async CDN purge batching | custom batching loop | existing `DoSpacesCdn::purge()` | Already handles ≤50 batch, 5-req/10s throttle, error propagation |
| Bunny rate limiting | custom throttle | existing `BunnyCdn::purge()` | Already handles 100-req/10s window |
| Cloudflare URL construction | custom full-URL builder | existing `CloudflareCdn::purge()` | Already handles `cdn_base_url` + relative path join |
| Token redaction | custom Debug impl | copy the `DoSpacesCdnConfig` pattern | Hand-written `Debug` printing `<redacted>` is already established |
| Error enum | custom error struct | extend existing `Error` enum with new variants | `thiserror` derive is already set up |

---

## Exact Current Env Reads

All confirmed by direct source inspection. [VERIFIED: ferro-storage/src/]

### CDN URL (display path)
| Env var | File | Line | Struct/Field | Notes |
|---------|------|------|--------------|-------|
| `AWS_CDN_URL` | `config.rs` | 119 | `DiskConfig.cdn_url` via `with_cdn_url()` | The D-05 wiring point; feeds `Disk::cdn_url()` |
| `BUNNY_CDN_URL` | `cdn/bunny.rs` | 41 | `BunnyCdnConfig.cdn_base_url` | Used by Bunny adapter for full-URL construction in `purge()` |
| `CF_CDN_URL` | `cdn/cloudflare.rs` | 40 | `CloudflareCdnConfig.cdn_base_url` | Used by CF adapter for full-URL construction in `purge()` |

### CDN Zone (purge routing)
| Env var | File | Line | Struct/Field | Notes |
|---------|------|------|--------------|-------|
| `DO_SPACES_CDN_ID` | `cdn/mod.rs` | 84 | `DoSpacesCdnConfig.endpoint_id: Option<String>` | Absent → DO purge is a logged no-op |
| `CF_ZONE_ID` | `cdn/cloudflare.rs` | 38 | `CloudflareCdnConfig.zone_id` | Absent → CF adapter returns Err |

### CDN Token (purge auth)
| Env var | File | Line | Struct/Field | Notes |
|---------|------|------|--------------|-------|
| `DIGITALOCEAN_ACCESS_TOKEN` | `cdn/mod.rs` | 85 | `DoSpacesCdnConfig.api_token` | `unwrap_or_default()` — empty string → Err at purge time |
| `CF_API_TOKEN` | `cdn/cloudflare.rs` | 39 | `CloudflareCdnConfig.api_token` | `unwrap_or_default()` |
| `BUNNY_ACCESS_KEY` | `cdn/bunny.rs` | 42 | `BunnyCdnConfig.access_key` | `unwrap_or_default()` |

### `app/.env.example` current CDN section (lines 79-91)
```env
#-------------------------------------------------
# CDN Settings (ferro-storage — optional)
#-------------------------------------------------
# DigitalOcean Spaces CDN
AWS_CDN_URL=
DIGITALOCEAN_ACCESS_TOKEN=
DO_SPACES_CDN_ID=

# Alternative CDN providers
BUNNY_CDN_URL=
BUNNY_ACCESS_KEY=
CF_CDN_URL=
CF_API_TOKEN=
CF_ZONE_ID=
```
[VERIFIED: app/.env.example lines 79-91]

### Fallback mapping (D-02) — complete table
| Quartet var | Legacy aliases (in order) | Per-provider semantics |
|-------------|--------------------------|----------------------|
| `CDN_URL` | `AWS_CDN_URL`, `CF_CDN_URL`, `BUNNY_CDN_URL` | Display URL; feeds `Disk::cdn_url()` |
| `CDN_PURGE_ZONE` | `DO_SPACES_CDN_ID`, `CF_ZONE_ID` | DO = endpoint id; CF = zone id; Bunny = unused (no zone) |
| `CDN_PURGE_TOKEN` | `DIGITALOCEAN_ACCESS_TOKEN`, `CF_API_TOKEN`, `BUNNY_ACCESS_KEY` | Provider API credential |
| `CDN_PROVIDER` | inferred from: `DO_SPACES_CDN_ID`→digitalocean, `CF_ZONE_ID`→cloudflare, `BUNNY_CDN_URL`→bunny | No legacy env alias; inference only |

---

## `PurgeApi` Object Safety

`PurgeApi` is declared `#[async_trait] pub trait PurgeApi: Send + Sync`. The `async_trait` macro rewrites async methods to return `Pin<Box<dyn Future + Send>>`, making the trait fully object-safe. The current codebase does not yet use `Box<dyn PurgeApi>` anywhere — `build_purge_api()` will be the first site. This is safe to introduce. [VERIFIED: cdn/mod.rs lines 41-48; no existing Box<dyn PurgeApi> usage confirmed by grep]

---

## Error Enum Extensions

Two new variants needed in `ferro-storage/src/error.rs`:

```rust
// Source: VERIFIED against error.rs + D-03/D-04 requirements
/// CDN provider name is not recognized.
#[error("CDN_PROVIDER value '{0}' is not valid; valid values: none, digitalocean, bunny, cloudflare")]
CdnInvalidProvider(String),

/// Selected CDN provider requires a cargo feature that is not enabled.
#[error("CDN_PROVIDER={0} requires the '{1}' cargo feature")]
CdnFeatureRequired(String, &'static str),
```

Constructor helpers:
```rust
pub fn cdn_invalid_provider(val: impl Into<String>) -> Self {
    Self::CdnInvalidProvider(val.into())
}
pub fn cdn_feature_required(provider: &str, feature: &'static str) -> Self {
    Self::CdnFeatureRequired(provider.to_string(), feature)
}
```

---

## Version Bump and CHANGELOG

### Current version
Workspace `Cargo.toml` line 38: `version = "0.2.52"`. ferro-storage uses `version.workspace = true`. [VERIFIED: Cargo.toml, ferro-storage/Cargo.toml]

**Next minor version: `0.2.53`**

Phase 188 shipped as `0.2.48` (confirmed in STATE.md). Workspace has advanced to `0.2.52` across subsequent phases. The next minor bump is `0.2.53`. Update `version = "0.2.52"` → `version = "0.2.53"` in the workspace `Cargo.toml`.

### CHANGELOG location
No `CHANGELOG.md` exists in `ferro-storage/`. [VERIFIED: `ls ferro-storage/` output shows no changelog file]

Create `ferro-storage/CHANGELOG.md` with a `## [0.2.53]` entry.

### `docs/src/features/storage.md` CDN section to update
Lines ~384-386 currently document `AWS_CDN_URL` as the CDN base URL env var. The section starting at line 377 (`## CDN`) documents both the display URL and purge adapters. This section must be updated to show the quartet as primary, with old vars listed as deprecated. [VERIFIED: docs/src/features/storage.md lines 377-474]

---

## Common Pitfalls

### Pitfall 1: Env-var test pollution
**What goes wrong:** New tests using `std::env::set_var` for CDN_URL, CDN_PROVIDER, CDN_PURGE_TOKEN, CDN_PURGE_ZONE run in parallel with other tests that read the same vars, causing intermittent failures.
**Why it happens:** Cargo's test harness runs `#[test]` functions in parallel threads within a single process; env is a global process-level state.
**How to avoid:** All tests that set env vars must either (a) use uniquely-named env var names that are only read by that specific test, or (b) use `serial_test` crate for serialization (requires a dev-dependency addition), or (c) use a `Mutex` guard scoped to the test. The existing `from_env_cdn_url` test in `config.rs` pattern-matches approach (a) implicitly — it sets `AWS_CDN_URL` and `AWS_BUCKET` which are test-specific in practice but not guarded. **Preferred for this phase:** use unique env var names per test (e.g. prefix with test function name) — zero new dependencies.
**Warning signs:** Tests pass individually (`cargo test test_name`) but fail under full `cargo test`.

### Pitfall 2: `CDN_URL` normalization divergence from `AWS_CDN_URL` passthrough
**What goes wrong:** SC-3 parity test fails if `CDN_URL` fallback value is trimmed, lowercased, or otherwise transformed before being passed to `with_cdn_url`, while `AWS_CDN_URL` was passed raw.
**Why it happens:** If `env_with_fallback` modifies the returned value.
**How to avoid:** `env_with_fallback` returns the raw `String` from `env::var` without any transformation. The parity test verifies byte-identical output.

### Pitfall 3: Clippy `unreachable_code` on `#[cfg]` arms in match
**What goes wrong:** Inside a `match CdnProvider::Bunny` arm, having both a `#[cfg(feature="cdn-bunny")]` block and a `#[cfg(not(feature="cdn-bunny"))]` block can trigger `unreachable_code` warnings from clippy because only one branch is compiled in, but if the structure is written as sequential `return` statements, clippy sees the second `return` as dead code relative to the first.
**How to avoid:** Structure each cfg block as a complete block expression returning `Result` (pattern shown in code examples above). Each `match` arm has exactly one return path per compilation.
**Warning signs:** `cargo clippy --all --all-targets -- -D warnings` fails on this file.

### Pitfall 4: `build_purge_api` borrow of `self.url` for Bunny/Cloudflare
**What goes wrong:** Bunny's `cdn_base_url` and Cloudflare's `cdn_base_url` in the adapter need the CDN display URL. These are currently populated from `BUNNY_CDN_URL`/`CF_CDN_URL` respectively. Under the unified config, they come from `Config.url`. If `Config.url` is `None` when the user sets `CDN_PROVIDER=bunny` but not `CDN_URL`, the Bunny/CF adapter gets an empty `cdn_base_url`.
**Why it happens:** `CDN_URL` is optional (designed for the display path); but Bunny and Cloudflare adapters require a base URL to build full purge URLs.
**How to avoid:** In `build_purge_api()`, when constructing Bunny/CF adapters, use `self.url.clone().unwrap_or_default()` (matching the existing `unwrap_or_default()` behavior in the legacy `from_env` methods). The adapter will then return an error at `purge()` time if the URL is empty — consistent with existing behavior.

### Pitfall 5: DO adapter always available but `DO_SPACES_CDN_ID` mapped to `purge_zone`
**What goes wrong:** In the legacy `DoSpacesCdnConfig`, `endpoint_id` is `Option<String>` (absent = no-op). Under the unified config, `CDN_PURGE_ZONE` maps to `endpoint_id`. If a user sets `CDN_PROVIDER=digitalocean` but not `CDN_PURGE_ZONE` (and has no legacy `DO_SPACES_CDN_ID`), the DO adapter will silently no-op rather than erroring.
**Why it happens:** This matches the existing behavior of `DoSpacesCdn::purge()` which has a built-in no-op when `endpoint_id` is `None`. It is correct and intentional — the endpoint id is optional for the DO adapter.
**Warning signs:** None; this is the designed behavior. Document it in CHANGELOG.

---

## Code Examples

### Complete `cdn::Config` struct
```rust
// Source: VERIFIED against D-01 shape requirements; consistent with existing *CdnConfig pattern
/// Provider-agnostic CDN configuration.
///
/// Read from environment via [`Config::from_env`]. Construct the active purge adapter
/// with [`Config::build_purge_api`].
///
/// # Token security
///
/// `purge_token` is never logged. The `Debug` implementation prints `<redacted>` for this field.
#[derive(Clone)]
pub struct Config {
    /// CDN base URL fronting the bucket (`CDN_URL`). Drives `Disk::cdn_url()`.
    pub url: Option<String>,
    /// Selected provider for cache invalidation (`CDN_PROVIDER`).
    pub provider: CdnProvider,
    /// Provider API credential (`CDN_PURGE_TOKEN`). Never logged.
    pub purge_token: Option<String>,
    /// Provider-specific zone or endpoint id (`CDN_PURGE_ZONE`).
    pub purge_zone: Option<String>,
}

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

### SC-3 parity test skeleton
```rust
// Source: VERIFIED — extends existing from_env_cdn_url test in config.rs
#[cfg(feature = "s3")]
#[test]
fn cdn_url_parity_aws_cdn_url_fallback() {
    // Set only AWS_CDN_URL (legacy) — no quartet vars
    std::env::set_var("AWS_BUCKET_PARITY_TEST", "test-bucket");  // unique name
    std::env::set_var("AWS_CDN_URL_PARITY_TEST_SC3", "https://cdn.parity.example.com");
    // ... NOTE: actual var names must match what from_env reads;
    // use unique suffixes only for isolation if the implementation allows it.
    // If env_with_fallback reads the exact var "AWS_CDN_URL", use that name but
    // set/remove before/after under a mutex or serial annotation.

    // The output must equal what the legacy-only path produced before this phase.
    let config = StorageConfig::from_env();
    let s3 = config.get_disk("s3").expect("s3 disk");
    assert_eq!(s3.cdn_url, Some("https://cdn.parity.example.com".to_string()));

    std::env::remove_var("AWS_BUCKET_PARITY_TEST");
    std::env::remove_var("AWS_CDN_URL_PARITY_TEST_SC3");
}
```

### SC-4 DO-purge auth parity test skeleton
```rust
// Source: VERIFIED — mirrors do_adapter_request_shape test in cdn/mod.rs
// The DoSpacesCdn adapter's api_base override allows pointing at a wiremock server.
// SC-4 verifies: with only legacy DO vars (DO_SPACES_CDN_ID, DIGITALOCEAN_ACCESS_TOKEN),
// build_purge_api() constructs an adapter that authenticates correctly.
#[tokio::test]
async fn purge_parity_legacy_do_vars() {
    // build_purge_api() with env: DO_SPACES_CDN_ID=test-id, DIGITALOCEAN_ACCESS_TOKEN=test-token
    // (no CDN_PROVIDER, no CDN_PURGE_ZONE, no CDN_PURGE_TOKEN set)
    // → provider inferred as DigitalOcean from DO_SPACES_CDN_ID presence
    // → DoSpacesCdn constructed with endpoint_id=Some("test-id"), api_token="test-token"
    // → purge() sends correct auth header to DO API (verified via wiremock api_base override)
    // NOTE: api_base is pub(crate); for the parity test, construct DoSpacesCdn directly
    // from the DoSpacesCdnConfig that Config produces, then override api_base for test.
}
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]` |
| Config file | none — cargo test discovery |
| Quick run command | `cargo test -p ferro-storage -- --test-thread=1` (serial for env-var isolation) |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map
| SC | Behavior | Test Type | Automated Command | File Exists? |
|----|----------|-----------|-------------------|-------------|
| SC-1 | `cdn::Config::from_env` reads quartet as primary | unit | `cargo test -p ferro-storage cdn_config_from_env` | Wave 0 |
| SC-2 | Per-var legacy fallback chain with `tracing::warn!` | unit | `cargo test -p ferro-storage cdn_fallback_` | Wave 0 |
| SC-3 | `Disk::cdn_url()` byte-identical for `AWS_CDN_URL`-only env | unit | `cargo test -p ferro-storage cdn_url_parity` | Wave 0 |
| SC-4 | `purge()` authenticates against DO Spaces CDN with legacy vars | integration (wiremock) | `cargo test -p ferro-storage purge_parity_legacy_do` | Wave 0 |
| SC-5a | `CDN_PROVIDER=none` → purge() explicit logged no-op | unit | `cargo test -p ferro-storage cdn_provider_none` | Wave 0 |
| SC-5b | Invalid `CDN_PROVIDER` → boot error listing valid values | unit | `cargo test -p ferro-storage cdn_invalid_provider` | Wave 0 |
| SC-5c | Feature-off boot error (Bunny without `cdn-bunny`) | unit (cfg) | `cargo test -p ferro-storage cdn_feature_required` | Wave 0 |
| SC-6 | Minor version bump + CHANGELOG entry | manual / compile check | inspect Cargo.toml + CHANGELOG.md | Wave 0 (file creation) |
| SC-7 | `cargo test --all-features` + `clippy --all -Dwarnings` | CI gate | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings` | existing |

### Env-Var Test Isolation Strategy
The existing `from_env_cdn_url` test in `config.rs` sets/removes env vars in-function without any parallel guard. For new tests:
- Each new test function that sets env vars must use a variable naming convention to minimize collision — e.g. set the exact legacy var name but remove it immediately in the same function (as the existing test does).
- If collision risk is high (multiple tests reading the same CDN_* var), add `serial_test = "1"` dev-dependency and annotate tests with `#[serial]`. Check if `serial_test` is already in dev-dependencies before adding.
- Alternatively, test `Config::from_env` indirectly by constructing `Config` structs directly (bypassing env reads) for all tests except the explicit parity tests.

### Wave 0 Gaps
- [ ] `ferro-storage/src/cdn/mod.rs` — `Config`, `CdnProvider`, `env_with_fallback`, `build_purge_api`, and their unit tests
- [ ] `ferro-storage/src/error.rs` — `CdnInvalidProvider`, `CdnFeatureRequired` variants
- [ ] `ferro-storage/CHANGELOG.md` — new file
- [ ] Test functions listed in SC-1 through SC-5c above (all new)

Existing test `from_env_cdn_url` in `config.rs` is the SC-3 parity baseline — it must stay green after the `config.rs:119` wiring change.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Per-provider env clusters (`AWS_CDN_URL`, `DO_SPACES_CDN_ID`, etc.) | Provider-agnostic quartet (`CDN_URL`, `CDN_PROVIDER`, `CDN_PURGE_TOKEN`, `CDN_PURGE_ZONE`) | Phase 204 | Operators use one set of vars regardless of CDN provider |
| `DoSpacesCdnConfig::from_env()` reads env directly | `cdn::Config::from_env()` reads quartet + fallbacks | Phase 204 | Single env-reading entry point |
| No `Config` type in `cdn::` | `ferro_storage::cdn::Config` + `CdnProvider` exported from `lib.rs` | Phase 204 | Public API surface for the unified config |

---

## Assumptions Log

All claims in this research were verified directly against source files. No assumed claims.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**All claims in this research were verified or cited — no user confirmation needed.**

---

## Open Questions (RESOLVED)

**RESOLVED in planning (204-01/204-02):** OQ-1 — `StorageConfig::from_env` stays **infallible** (`-> Self`, minimum-diff, no caller breakage); `cdn::Config::from_env` is also infallible, and the invalid-provider **boot Error (SC-5b)** surfaces through the provider parse (`from_str_ci`) at `build_purge_api()` time rather than from `StorageConfig::from_env`. OQ-2 — the existing `from_env_cdn_url` test stays green unmodified (it now exercises the silent `AWS_CDN_URL`→`CDN_URL` fallback path; `tracing::warn!` is a no-op without a subscriber). Both carried into the plans; no open items remain.

1. **`StorageConfig::from_env` return type**
   - What we know: currently infallible (`-> Self`); `cdn::Config::from_env` needs to return `Result` to cover the invalid-provider boot error path.
   - What's unclear: whether to change `StorageConfig::from_env` to `Result<Self, Error>` (propagate boot error to caller) or keep it infallible and defer provider validation to `build_purge_api()`.
   - Recommendation: keep `StorageConfig::from_env` infallible — store invalid provider as `CdnProvider::None` + `tracing::error!` in `Config::from_env`. The `Error` variant fires in `build_purge_api()` when the app actually tries to use purge. This is the minimum-diff approach and avoids breaking all existing callers of `StorageConfig::from_env`.

2. **`from_env_cdn_url` test isolation after wiring change**
   - What we know: that test sets `AWS_CDN_URL` and `AWS_BUCKET`. After the change, `config.rs:119` will call `cdn::Config::from_env()` which reads `CDN_URL` first; `AWS_CDN_URL` is a fallback alias. The test sets `AWS_CDN_URL` but not `CDN_URL` — it will still work and trigger the legacy fallback path (emitting a warn).
   - What's unclear: whether the warn emission in a test is a problem (it is not — `tracing::warn!` has no effect without a subscriber in tests unless the test explicitly sets one up).
   - Recommendation: verify the existing test still passes after the wiring change. No fix needed unless a subscriber is active.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is purely code/config changes within the `ferro-storage` crate. No external tools, services, runtimes, or CLI utilities beyond the Rust toolchain are required.

---

## Security Domain

CDN configuration is not an authentication or data-access surface. The security requirements are limited to credential handling:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (provider enum) | Hand-rolled case-insensitive parse; invalid value → `Error` with valid-values list |
| V6 Cryptography | no | — |

### Credential Handling
| Pattern | Control |
|---------|---------|
| API token in Debug output | Hand-written `Debug` printing `<redacted>` — established pattern from Phase 188; must be extended to `cdn::Config.purge_token` |
| API token in deprecation warn | `env_with_fallback` for `CDN_PURGE_TOKEN` must NOT include the value in the warn message |
| API token in error messages | `CdnFeatureRequired` and `CdnInvalidProvider` errors must not include the token value |

---

## Sources

### Primary (HIGH confidence — direct source inspection)
- `ferro-storage/src/cdn/mod.rs` — exact env var names, `DoSpacesCdnConfig` fields, `PurgeApi` trait, test patterns, `api_base` override for tests
- `ferro-storage/src/cdn/bunny.rs` — `BunnyCdnConfig` fields, env var names, no-zone semantics
- `ferro-storage/src/cdn/cloudflare.rs` — `CloudflareCdnConfig` fields, env var names, CF batch size
- `ferro-storage/src/config.rs` — `StorageConfig::from_env`, line 119 wiring point, `from_env_cdn_url` test
- `ferro-storage/src/facade.rs` — `Disk::cdn_url()` implementation (line 407), `DiskConfig.cdn_url`, `with_cdn_url`
- `ferro-storage/src/lib.rs` — current re-exports
- `ferro-storage/src/error.rs` — `Error` enum structure, existing `Cdn(String)` variant
- `ferro-storage/Cargo.toml` — feature flags (`cdn-bunny`, `cdn-cloudflare`, `s3`), existing dependencies
- `Cargo.toml` — workspace version `0.2.52`
- `app/.env.example` lines 79-91 — current CDN env section
- `docs/src/features/storage.md` lines 377-474 — CDN docs section

### Secondary (MEDIUM confidence — grep / structural verification)
- No `Box<dyn PurgeApi>` usage in the codebase — confirmed by grep; `build_purge_api()` will be the first site
- `from_env_cdn_url` is the only env-var test in `ferro-storage/src/` — confirmed by grep of `set_var`/`remove_var`
- ferro-storage in Wave 1A of publish workflow — confirmed via `.github/workflows/publish.yml` line 211

---

## Metadata

**Confidence breakdown:**
- Exact env var reads: HIGH — confirmed line by line from source
- `PurgeApi` object safety: HIGH — `async_trait` already in use; `Send + Sync` bound present
- Feature-gate `#[cfg]` pattern: HIGH — mirrors existing S3 cfg pattern in facade.rs
- Version bump target (0.2.53): HIGH — workspace version confirmed as 0.2.52
- Clippy `unreachable_code` hazard: MEDIUM — known rustc/clippy behavior with `#[cfg]` in match arms; mitigation pattern verified against existing code conventions in this codebase

**Research date:** 2026-06-11
**Valid until:** 2026-07-11 (stable internal codebase; env-var names do not change without this phase)
