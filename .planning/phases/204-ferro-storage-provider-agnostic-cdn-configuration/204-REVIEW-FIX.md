---
phase: 204-ferro-storage-provider-agnostic-cdn-configuration
fixed_at: 2026-06-11T18:10:00Z
review_path: .planning/phases/204-ferro-storage-provider-agnostic-cdn-configuration/204-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 204: Code Review Fix Report

**Fixed at:** 2026-06-11T18:10:00Z
**Source review:** `.planning/phases/204-ferro-storage-provider-agnostic-cdn-configuration/204-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed: 5
- Skipped: 0

## Fixed Issues

### WR-01: CDN_PURGE_TOKEN fallback order can pick the wrong provider's token

**Files modified:** `ferro-storage/src/cdn/mod.rs`
**Commit:** bf0d6671
**Applied fix:** Moved token/zone resolution to after provider inference. Each `CdnProvider` arm
now reads ONLY its own legacy aliases (e.g. `CdnProvider::DigitalOcean` reads
`DIGITALOCEAN_ACCESS_TOKEN` / `DO_SPACES_CDN_ID`; `CdnProvider::Cloudflare` reads
`CF_API_TOKEN` / `CF_ZONE_ID`; `CdnProvider::Bunny` reads `BUNNY_ACCESS_KEY` only;
`CdnProvider::None` reads no aliases). Cross-provider credential contamination is now
structurally impossible. Existing tests `cdn_fallback_do_zone_and_token` and
`cdn_fallback_aws_cdn_url` remain green.

---

### WR-02: Invalid CDN_PROVIDER value silently disables purging instead of failing boot

**Files modified:** `ferro-storage/src/cdn/mod.rs`
**Commit:** bf0d6671
**Applied fix:** Added `provider_error: Option<String>` field to `Config` (and `#[derive(Default)]`
to the struct). When `CDN_PROVIDER` is set but `from_str_ci` fails, `provider_error` is set to the
raw bad value and a `tracing::error!` is emitted. `build_purge_api` now checks `provider_error`
first and returns `Err(Error::cdn_invalid_provider(bad))` immediately, surfacing the misconfiguration
as a boot error (D-03 / SC-5b). The new `cdn_invalid_provider_from_env_errors` test asserts this
path. `from_env` remains infallible (`-> Self`) to preserve `StorageConfig::from_env` compatibility.

---

### WR-03: Bunny inference trigger diverges from Bunny's token alias

**Files modified:** `ferro-storage/src/cdn/mod.rs`
**Commit:** bf0d6671
**Applied fix:** Added `|| std::env::var("BUNNY_ACCESS_KEY").is_ok()` to the Bunny inference branch.
A deployment that sets `BUNNY_ACCESS_KEY` without `BUNNY_CDN_URL` now correctly infers
`CdnProvider::Bunny` instead of falling through to `CdnProvider::None`. The deprecation warn
message was updated to name both signals.

---

### IN-01: `env_with_fallback` label tuple element is always identical to alias name

**Files modified:** `ferro-storage/src/cdn/mod.rs`
**Commit:** bf0d6671
**Applied fix:** Simplified `env_with_fallback` signature from `aliases: &[(&str, &str)]` to
`aliases: &[&str]`. The warn message now uses `{alias}` directly (no separate label). All call
sites updated — post WR-01 rework there are fewer call sites and each is provider-scoped.

---

### IN-02: `StorageConfig::from_env()` doc comment references `AWS_CDN_URL` as primary CDN var

**Files modified:** `ferro-storage/src/config.rs`
**Commit:** bf0d6671
**Applied fix:** Updated the doc comment to list `CDN_URL` as the primary variable, with
`AWS_CDN_URL` / `CF_CDN_URL` / `BUNNY_CDN_URL` described as deprecated aliases. Also added
mention of `CDN_PROVIDER` / `CDN_PURGE_TOKEN` / `CDN_PURGE_ZONE` with a reference to
`crate::cdn::Config` for full documentation.

---

## Verification

- `cargo fmt -p ferro-storage -- --check`: clean
- `cargo clippy -p ferro-storage --all-features --all-targets -- -D warnings`: clean
- `cargo test -p ferro-storage --all-features`: **63 passed, 0 failed** (including new
  `cdn_invalid_provider_from_env_errors` test confirming the env path now errors via
  `build_purge_api`)

---

_Fixed: 2026-06-11T18:10:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
