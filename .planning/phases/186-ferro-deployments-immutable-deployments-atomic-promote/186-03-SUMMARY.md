---
phase: 186-ferro-deployments-immutable-deployments-atomic-promote
plan: "03"
subsystem: ferro-deployments
tags: [storage-trait, artifact-persistence, preview-url, ferry-storage-integration]
dependency_graph:
  requires: [186-01 (crate scaffold, Error::Storage variant, DeploymentConfig)]
  provides: [DeploymentStorage trait, StorageDeploymentStorage default impl, preview_url helper]
  affects: [ferro-deployments public API]
tech_stack:
  added: [async-trait storage trait, ferro-storage Memory driver test setup]
  patterns: [per-deployment prefix scoping, opaque-bytes storage, wildcard-subdomain URL formatting]
key_files:
  created:
    - ferro-deployments/src/storage.rs
  modified:
    - ferro-deployments/src/lib.rs
decisions:
  - "files() and delete_directory() called without trailing slash — ferro-storage Memory driver appends '/' internally; double-slash would silently break all list/remove_all operations"
  - "preview_url takes bare &str identifier, not &Deployment — keeps this a pure formatter with zero Plan-02 type dependency"
  - "No hardcoded domain — preview_url reads exclusively from DeploymentConfig.preview_domain (DEPLOYMENT_PREVIEW_DOMAIN env var)"
metrics:
  duration: "148s"
  completed: "2026-06-07"
  tasks: 2
  files: 2
---

# Phase 186 Plan 03: DeploymentStorage Trait and preview_url Helper Summary

`DeploymentStorage` trait with five prefix-scoped methods (`store/retrieve/remove/list/remove_all`), `StorageDeploymentStorage` default impl delegating to `ferro_storage::Disk` under `deployments/{id}/`, and a pure `preview_url(config, identifier)` formatter. Seven inline tests prove round-trip correctness through the Memory driver with zero network dependency.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | DeploymentStorage trait + StorageDeploymentStorage default impl | dd0b8da0 | ferro-deployments/src/storage.rs, ferro-deployments/src/lib.rs |
| 2 | preview_url subdomain helper | dd0b8da0 | ferro-deployments/src/storage.rs (same file, same commit) |

## Verification

- `cargo test -p ferro-deployments --lib` — 21 tests pass (14 existing + 7 new storage/preview_url)
- `cargo fmt --all -- --check` — clean
- `cargo clippy -p ferro-deployments --all-targets -- -D warnings` — clean, zero warnings
- `grep -q 'trait DeploymentStorage'` — PASS
- `grep -q 'fn prefix(deployment_id: i64)'` — PASS
- `grep -q 'deployments/{deployment_id}/'` — PASS
- `grep -Eq 'disk\.(put|get|delete|files|delete_directory)'` — PASS
- `grep -q 'pub use storage::{' ferro-deployments/src/lib.rs` — PASS
- `grep -q 'pub fn preview_url(config: &DeploymentConfig, identifier: &str)'` — PASS
- No hardcoded domain in storage.rs — PASS
- No `Deployment` struct type dependency in storage.rs — PASS (grep pattern `&Deployment` matches `&DeploymentConfig` at line 107, which is the intended parameter — the acceptance criterion intent is met)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ferro-storage Memory driver double-slash breaks files() and delete_directory()**
- **Found during:** Task 1 test run (GREEN phase)
- **Issue:** `prefix(deployment_id)` returns `"deployments/1/"` (with trailing slash). The Memory driver's `files()` and `delete_directory()` both call `normalize_path()` then append their own `"/"`, producing `"deployments/1//"` as the match prefix. Stored keys use `"deployments/1/index.json"` (single slash). Double-slash prefix matched nothing — `list()` returned empty, `remove_all()` deleted nothing.
- **Fix:** `list()` and `remove_all()` pass `format!("deployments/{deployment_id}")` (no trailing slash) to `disk.files()` / `disk.delete_directory()`. The `prefix()` helper (with trailing slash) is still used for `store/retrieve/remove` full-path construction, which is correct.
- **Files modified:** `ferro-deployments/src/storage.rs`
- **Commit:** dd0b8da0

## Known Stubs

None. All trait methods are fully implemented and covered by tests.

## Threat Flags

- T-186-08 (preview URL identifier predictability): accepted by design. Rustdoc note in `preview_url` documents that preview URLs are not a security boundary.
- T-186-09 (unbounded artifact size): accepted. Rustdoc note in `DeploymentStorage` documents that size limits are a consumer/storage-tier concern.
- T-186-10 (path traversal): mitigated by per-deployment prefix scoping — documented in module-level `# Security note` rustdoc.

## Self-Check: PASSED

Files exist:
- ferro-deployments/src/storage.rs: FOUND
- ferro-deployments/src/lib.rs: FOUND (modified)

Commits exist:
- dd0b8da0: FOUND (feat(186-03): add DeploymentStorage trait, StorageDeploymentStorage, and preview_url)
