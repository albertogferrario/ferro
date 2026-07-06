---
phase: 204-ferro-storage-provider-agnostic-cdn-configuration
plan: "01"
subsystem: ferro-storage/cdn
tags: [cdn, config, env, provider, purge-api]
dependency_graph:
  requires: []
  provides: [cdn::Config, cdn::CdnProvider, Error::CdnInvalidProvider, Error::CdnFeatureRequired]
  affects: [ferro-storage/src/cdn/mod.rs, ferro-storage/src/error.rs, ferro-storage/src/lib.rs]
tech_stack:
  added: []
  patterns: [redacted-Debug, env_with_fallback, cfg-block-expressions, thiserror-variant-constructor]
key_files:
  created: []
  modified:
    - ferro-storage/src/error.rs
    - ferro-storage/src/cdn/mod.rs
    - ferro-storage/src/lib.rs
decisions:
  - "Config::from_env() is infallible (-> Self); invalid CDN_PROVIDER stores CdnProvider::None + tracing::error, surfaces as Error::CdnInvalidProvider via build_purge_api()"
  - "build_purge_api Bunny/Cloudflare arms use complete #[cfg]/#[cfg(not)] block expressions (not sequential returns) to satisfy clippy unreachable_code under -D warnings"
  - "env_with_fallback returns the raw String without normalization to preserve SC-3 byte-parity"
  - "Fixed cdn_feature_required_bunny test: used match instead of unwrap_err() because Box<dyn PurgeApi> does not implement Debug"
metrics:
  duration: "4m 41s"
  completed_date: "2026-06-11"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 3
  files_created: 0
---

# Phase 204 Plan 01: CDN Config Layer Summary

Provider-agnostic `cdn::Config` + `CdnProvider` enum with quartet env reading, legacy fallbacks, redacted Debug, and `build_purge_api()` dispatching to the existing Phase 188 purge adapters.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Add CdnInvalidProvider + CdnFeatureRequired error variants | bb4dbd96 | ferro-storage/src/error.rs |
| 2 | Add Config, CdnProvider, env_with_fallback, build_purge_api | bca2a7a9 | ferro-storage/src/cdn/mod.rs |
| 3 | Export CdnConfig and CdnProvider from lib.rs | b46bfcf8 | ferro-storage/src/lib.rs |

## What Was Built

**`ferro-storage/src/error.rs`**
- `CdnInvalidProvider(String)` variant: `CDN_PROVIDER value '{0}' is not valid; valid values: none, digitalocean, bunny, cloudflare`
- `CdnFeatureRequired(String, &'static str)` variant: `CDN_PROVIDER={0} requires the '{1}' cargo feature`
- `cdn_invalid_provider()` and `cdn_feature_required()` constructors on `impl Error`

**`ferro-storage/src/cdn/mod.rs`**
- `CdnProvider` enum (`None | DigitalOcean | Bunny | Cloudflare`) with `from_str_ci()` (case-insensitive, invalid → `CdnInvalidProvider`)
- `env_with_fallback(primary, aliases)` private helper: reads primary env var; on first legacy alias hit emits `tracing::warn!(label, primary)` — never interpolates the value (T-204-DEPRECATION-LEAK)
- `Config` struct (`url`, `provider`, `purge_token`, `purge_zone`) with hand-written `Debug` printing `<redacted>` for `purge_token` (T-204-TOKEN-REDACT)
- `Config::from_env()` infallible: reads `CDN_URL/CDN_PROVIDER/CDN_PURGE_TOKEN/CDN_PURGE_ZONE` quartet as primary; falls back per-var to legacy aliases; infers `CDN_PROVIDER` from legacy cluster presence when unset
- `Config::build_purge_api()`: `None` → logged `Ok(None)` (T-204-SILENT-NOOP); `DigitalOcean` → always available; `Bunny`/`Cloudflare` → `#[cfg]`/`#[cfg(not)]` block expressions returning `Result` (no `unreachable_code` clippy lint)
- 7 new unit tests covering SC-1/SC-2/SC-5a/b/c and T-204-TOKEN-REDACT

**`ferro-storage/src/lib.rs`**
- Unconditional re-export updated: `pub use cdn::{CdnProvider, Config as CdnConfig, DoSpacesCdn, DoSpacesCdnConfig, PurgeApi};`

## Verification Results

```
cargo clippy -p ferro-storage --all-targets -- -D warnings  → clean (0 warnings)
cargo test -p ferro-storage -- --test-threads=1             → 46 passed, 0 failed
```

SC-1 `cdn_config_from_env_quartet` — green
SC-2 `cdn_fallback_aws_cdn_url` + `cdn_fallback_do_zone_and_token` — green
SC-5a `cdn_provider_none_no_op` — green
SC-5b `cdn_invalid_provider` — green
SC-5c `cdn_feature_required_bunny` — green
T-204-TOKEN-REDACT `cdn_config_debug_redacts_token` — green

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Debug constraint on unwrap_err() in cdn_feature_required_bunny test**
- **Found during:** Task 2 compile
- **Issue:** `result.unwrap_err()` on `Result<Option<Box<dyn PurgeApi>>, Error>` requires `Debug` on the `Ok` type (`Box<dyn PurgeApi>`), which `PurgeApi` does not implement.
- **Fix:** Replaced `result.unwrap_err().to_string()` with a `match result { Err(e) => e.to_string(), Ok(_) => unreachable!() }` pattern that only requires `Debug` on `Err`.
- **Files modified:** ferro-storage/src/cdn/mod.rs
- **Commit:** bca2a7a9

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries were introduced. All threat model items from the plan are implemented:

| Threat ID | Mitigation Status |
|-----------|------------------|
| T-204-TOKEN-REDACT | Implemented — hand-written Debug prints `<redacted>`; test asserts absence of value |
| T-204-DEPRECATION-LEAK | Implemented — env_with_fallback warn interpolates only label/primary, never val |
| T-204-MISCONFIG | Implemented — invalid provider → CdnInvalidProvider; feature off → CdnFeatureRequired |
| T-204-SILENT-NOOP | Implemented — provider None logs `tracing::info!` explicitly |

## Self-Check: PASSED

Files exist:
- ferro-storage/src/error.rs — FOUND
- ferro-storage/src/cdn/mod.rs — FOUND
- ferro-storage/src/lib.rs — FOUND

Commits exist:
- bb4dbd96 — FOUND (error variants)
- bca2a7a9 — FOUND (cdn/mod.rs types + tests)
- b46bfcf8 — FOUND (lib.rs re-exports)
