---
phase: 204-ferro-storage-provider-agnostic-cdn-configuration
plan: "02"
subsystem: ferro-storage/config
tags: [cdn, config, parity, env, s3]
dependency_graph:
  requires: [cdn::Config, cdn::CdnProvider, DoSpacesCdn, DoSpacesCdnConfig]
  provides: [SC-3-parity, SC-4-parity, config.rs-rewired]
  affects: [ferro-storage/src/config.rs]
tech_stack:
  added: []
  patterns: [env_with_fallback-fallback-path, wiremock-api-base-override, serial-env-test-isolation]
key_files:
  created: []
  modified:
    - ferro-storage/src/config.rs
decisions:
  - "StorageConfig::from_env sources S3 CDN URL from cdn::Config::from_env().url (CDN_URL primary, AWS_CDN_URL fallback) — direct env::var read removed"
  - "PurgeApi trait must be in scope in config.rs tests; imported locally with use crate::cdn::PurgeApi inside the async test fn"
  - "SC-4 test constructs DoSpacesCdnConfig directly with api_base override to avoid live network calls"
metrics:
  duration: "~8m"
  completed_date: "2026-06-11"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 1
  files_created: 0
---

# Phase 204 Plan 02: CDN URL Wiring + Parity Tests Summary

Rewired `config.rs:119` to source the S3 CDN display URL from `cdn::Config::from_env().url` (unified CDN_URL primary + AWS_CDN_URL deprecated fallback), and proved SC-3/SC-4 parity guarantees with two new tests.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Rewire config.rs:119 to source CDN URL from cdn::Config | 59d58956 | ferro-storage/src/config.rs |
| 2 | Add SC-3 display-URL parity and SC-4 DO purge-auth parity tests | cd794b47 | ferro-storage/src/config.rs |

## What Was Built

**`ferro-storage/src/config.rs`**
- Replaced `if let Ok(cdn) = env::var("AWS_CDN_URL") { s3_config.with_cdn_url(cdn) }` at line ~119 with `crate::cdn::Config::from_env().url` pattern (D-05 wiring)
- `StorageConfig::from_env` stays infallible (`-> Self`) — no caller breakage
- `use std::env;` import retained (still used by other vars in the function)
- `cdn_url_parity_aws_fallback` test (SC-3): `AWS_CDN_URL`-only env with no `CDN_URL` → `Disk::cdn_url` byte-identical to pre-phase direct read
- `purge_parity_legacy_do` test (SC-4): legacy DO vars (`DO_SPACES_CDN_ID` + `DIGITALOCEAN_ACCESS_TOKEN`) → `DoSpacesCdn` sends `DELETE /v2/cdn/endpoints/legacy-id/cache` with `Authorization: Bearer legacy-token` via wiremock mock server

## Verification Results

```
cargo clippy -p ferro-storage --features s3 --all-targets -- -D warnings  → clean (0 warnings)
cargo test -p ferro-storage --features s3 from_env_cdn_url -- --test-threads=1        → ok
cargo test -p ferro-storage --features s3 cdn_url_parity_aws_fallback -- --test-threads=1 → ok
cargo test -p ferro-storage purge_parity_legacy_do -- --test-threads=1                → ok
```

All three tests green (baseline unmodified + 2 new parity tests).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added `use crate::cdn::PurgeApi` import inside purge_parity_legacy_do test**
- **Found during:** Task 2 compile
- **Issue:** `DoSpacesCdn::purge` is a trait method from `PurgeApi`; calling it from `config.rs` tests requires the trait to be in scope. The plan's code skeleton omitted the import.
- **Fix:** Added `use crate::cdn::PurgeApi;` as the first `use` inside the test function body.
- **Files modified:** ferro-storage/src/config.rs
- **Commit:** cd794b47

## Known Stubs

None.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Tests use a synthetic `legacy-token` value — no real credential in source (T-204-TOKEN-REDACT: accepted per plan).

| Threat ID | Status |
|-----------|--------|
| T-204-PURGE-PARITY | Implemented — `purge_parity_legacy_do` asserts identical DELETE endpoint + Bearer token |
| T-204-TOKEN-REDACT | Accepted — test uses synthetic `legacy-token`; no real credential |
| T-204-MISCONFIG | Implemented — `cdn_url_parity_aws_fallback` proves URL works with provider=None (orthogonal axes) |

## Self-Check: PASSED

Files exist:
- ferro-storage/src/config.rs — FOUND

Commits exist:
- 59d58956 — FOUND (config.rs rewire)
- cd794b47 — FOUND (parity tests)
