---
phase: 132-implement-ferro-storage-s3-driver
plan: "01"
subsystem: ferro-storage
tags: [s3, storage, aws-sdk, object-storage]
dependency_graph:
  requires: []
  provides: [S3Driver implementation, s3-tests feature gate]
  affects: [ferro-storage]
tech_stack:
  added: [aws-credential-types]
  patterns: [aws_sdk_s3::config::Builder sync construction, presigned URLs via PresigningConfig, paginator API for listing]
key_files:
  created:
    - ferro-storage/tests/s3_integration.rs
  modified:
    - ferro-storage/src/drivers/s3.rs
    - ferro-storage/src/facade.rs
    - ferro-storage/Cargo.toml
decisions:
  - behavior_version_latest() required on S3ConfigBuilder for client to function
  - normalize_dir_prefix strips both leading and trailing slashes before appending single trailing slash
  - Integration tests skip gracefully (not fail) when AWS_BUCKET env var is absent
metrics:
  duration: "~11 minutes"
  completed: "2026-04-14"
  tasks_completed: 2
  files_changed: 4
---

# Phase 132 Plan 01: Implement S3Driver — Summary

Full S3Driver implementation replacing the stub: all 15 StorageDriver trait methods backed by real `aws-sdk-s3` calls, facade wiring updated to `S3Driver::new()`, unit tests for URL construction and path normalization, integration test scaffold gated behind `s3-tests`.

## Tasks Completed

| Task | Commit | Description |
|------|--------|-------------|
| Task 1: S3Driver implementation | f25bd9b7 | Full 15-method impl, facade wiring, unit tests, Cargo.toml features |
| Task 2: Integration test scaffold | 94f3aa14 | 9 integration tests behind s3-tests, skip when AWS_BUCKET unset |

## What Was Built

**S3Driver struct** (`ferro-storage/src/drivers/s3.rs`):
- Holds `aws_sdk_s3::Client`, `bucket: String`, `region: String`, `url_base: Option<String>`
- Sync constructor using `aws_sdk_s3::config::Builder` with `aws_credential_types::Credentials::from_keys()` — no async config init needed
- `behavior_version_latest()` set on builder (required by SDK to function)
- Custom endpoint URL + `force_path_style(true)` when `AWS_URL` env var is set (for MinIO/R2)

**All 15 StorageDriver methods implemented:**
- `exists`, `get`, `put`, `delete`, `copy`, `size`, `metadata`, `url`, `temporary_url`
- `files`, `all_files`, `directories`, `make_directory`, `delete_directory`
- All listing methods use the SDK paginator API (`.into_paginator().send()`)
- `delete_directory` batches up to 1000 objects per `delete_objects` call
- `url()` returns `{url_base}/{path}` when configured, else `https://{bucket}.s3.{region}.amazonaws.com/{path}`
- `temporary_url()` uses `PresigningConfig` + `.presigned()` for real SigV4 presigned URLs

**Facade wiring** (`ferro-storage/src/facade.rs`):
- `DiskDriver::S3` arm replaced: `S3Driver::new(bucket, region, url_base, endpoint_url)`
- Removed the `tracing::warn!` stub message

**Cargo.toml** (`ferro-storage/Cargo.toml`):
- Added `aws-credential-types = { version = "1", features = ["hardcoded-credentials"], optional = true }`
- Added `s3 = ["aws-sdk-s3", "aws-config", "aws-credential-types"]`
- Added `s3-tests = ["s3"]`

**Integration tests** (`ferro-storage/tests/s3_integration.rs`):
- 9 tests covering: put/get/delete, exists missing, content-type, size, copy, url, temporary_url, files/directories, make/delete directory
- Skip gracefully (return early) when `AWS_BUCKET` env var is absent

## Verification

- `cargo clippy -p ferro-storage --all-targets --features s3 -- -D warnings`: PASSED
- `cargo test -p ferro-storage --features s3`: 30/30 tests pass
- `cargo clippy --all --all-targets -- -D warnings`: PASSED (full workspace)
- `cargo test --all-features`: 0 failures (integration tests skip without env vars)
- `grep -c "not_implemented" ferro-storage/src/drivers/s3.rs`: 0
- `grep "S3Driver::new" ferro-storage/src/facade.rs`: found

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SDK requires behavior_version on S3ConfigBuilder**
- **Found during:** Task 1 — unit tests panicked at runtime
- **Issue:** `aws-sdk-s3` v1 requires `.behavior_version_latest()` on the config builder; without it, the client panics on construction with "A behavior major version must be set"
- **Fix:** Added `.behavior_version_latest()` to the `S3ConfigBuilder::new()` call chain
- **Files modified:** `ferro-storage/src/drivers/s3.rs`
- **Commit:** f25bd9b7

**2. [Rule 1 - Bug] normalize_dir_prefix double-slash for paths with trailing slash**
- **Found during:** Task 1 — `test_normalize_dir_prefix` failed: `"photos/"` produced `"photos//"` instead of `"photos/"`
- **Fix:** Changed `trim_start_matches('/')` to `trim_matches('/')` so both leading and trailing slashes are stripped before appending single `/`
- **Files modified:** `ferro-storage/src/drivers/s3.rs`
- **Commit:** f25bd9b7

**3. [Rule 1 - Bug] Integration tests fail (not skip) when AWS_BUCKET is absent**
- **Found during:** Task 2 — `cargo test --all-features` ran integration tests and they panicked on `expect("s3 disk not configured")`
- **Issue:** Plan said "tests compile but skip without env vars" but tests used `.expect()` which panics
- **Fix:** Added `s3_disk_or_skip()` helper that returns `None` and uses `let Some(disk) = ... else { return; }` to skip gracefully
- **Files modified:** `ferro-storage/tests/s3_integration.rs`
- **Commit:** 94f3aa14

## Known Stubs

None — all 15 StorageDriver methods have real implementations. No `Error::not_implemented` calls remain.

## Self-Check: PASSED
