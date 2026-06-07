---
phase: 186-ferro-deployments-immutable-deployments-atomic-promote
fixed_at: 2026-06-07T00:00:00Z
review_path: .planning/phases/186-ferro-deployments-immutable-deployments-atomic-promote/186-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 186: Code Review Fix Report

**Fixed at:** 2026-06-07
**Source review:** .planning/phases/186-ferro-deployments-immutable-deployments-atomic-promote/186-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: Path traversal via unvalidated `path` parameter in `DeploymentStorage`

**Files modified:** `ferro-deployments/src/storage.rs`
**Commit:** 656529a8
**Applied fix:** Added `if path.contains("..") || path.starts_with('/')` guard at the top of `store`, `retrieve`, and `remove` in `StorageDeploymentStorage`, returning `Error::custom(...)` on violation. Added four unit tests: `path_traversal_store_rejected`, `path_traversal_retrieve_rejected`, `path_traversal_remove_rejected`, and `absolute_path_store_rejected`.

---

### WR-02: `ph()` fallback produces invalid placeholders for unsupported backends

**Files modified:** `ferro-deployments/src/deployment.rs`
**Commit:** 9876ff67
**Applied fix:** Changed `ph(backend, n) -> String` to `ph(backend, n) -> Result<String, Error>`. The wildcard arm is replaced with an explicit `DatabaseBackend::Sqlite` arm returning `Ok(format!("?{n}"))` and a catch-all `_ => Err(Error::UnsupportedBackend)`. All seven call sites in `create`, `mark_ready`, `mark_failed`, `get`, `list`, `active`, and `rollback` were updated to propagate with `?`.

---

### WR-03: `query_one` returning `Ok(None)` silently treated as "first promotion"

**Files modified:** `ferro-deployments/src/promote.rs`
**Commit:** 4b5215ea
**Applied fix:** In both `promote_sqlite` and `promote_postgres`, replaced the `Ok(r) => r` arm (where `r: Option<QueryResult>`) with explicit `Ok(Some(r)) => r` and `Ok(None) => { rollback; return Err(Error::custom(...)) }` arms. Replaced the `.and_then(|r| r.try_get_by(...).ok().flatten())` tail with explicit `try_get_by(...).map_err(|e| Error::custom(...))` propagation, matching the REVIEW.md recommended snippet exactly.

---

## Verification

```
cargo clippy -p ferro-deployments --all-targets -- -D warnings
  Finished `dev` profile — 0 warnings

cargo test -p ferro-deployments
  running 25 tests — all passed
  race_promote_sqlite: 1 test — passed
  doc-tests: 1 passed, 3 ignored
```

All 25 unit tests and the SQLite race test pass. No pre-existing tests were broken.

---

_Fixed: 2026-06-07_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
