---
phase: 186-ferro-deployments-immutable-deployments-atomic-promote
reviewed: 2026-06-07T00:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - ferro-deployments/src/config.rs
  - ferro-deployments/src/deployment.rs
  - ferro-deployments/src/error.rs
  - ferro-deployments/src/lib.rs
  - ferro-deployments/src/migration.rs
  - ferro-deployments/src/promote.rs
  - ferro-deployments/src/storage.rs
  - ferro-deployments/tests/race_promote_sqlite.rs
  - ferro-deployments/tests/race_promote_postgres.rs
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 186: Code Review Report

**Reviewed:** 2026-06-07
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

`ferro-deployments` is a well-structured leaf crate. The core security requirements are met: all caller-supplied strings are bound through `Statement::from_sql_and_values` with `Value::*` parameters — no string interpolation of dynamic data anywhere in the production SQL paths (T-186-04 compliant). The `conn.begin()` transaction pinning is applied on both backends, satisfying the atomicity requirement. Status-transition guards (`status = 'building'` predicate on UPDATE; `NotReady`/`ArtifactDeleted` guards before promote) are present and correct. Preview URL domain comes exclusively from the env var via `DeploymentConfig` — no hardcoded app identity found.

Three warnings and three info items were found. No critical issues.

---

## Warnings

### WR-01: Path traversal via unvalidated `path` parameter in `DeploymentStorage`

**File:** `ferro-deployments/src/storage.rs:68-80`

**Issue:** The `path` argument to `store`, `retrieve`, and `remove` is concatenated directly onto the per-deployment prefix without sanitization:

```rust
let full = format!("{}{}", Self::prefix(deployment_id), path);
```

A caller supplying `path = "../2/secret.json"` produces the key `deployments/1/../2/secret.json`. On the S3 driver this is likely benign (object keys are opaque), but on the `ferro-storage` `DiskDriver` the resulting path resolves outside the deployment's directory, allowing cross-deployment artifact access. The doc comment acknowledges the S3 driver but does not address the disk driver.

**Fix:** Reject or strip path-traversal components before constructing the key. A minimal guard:

```rust
async fn store(&self, deployment_id: i64, path: &str, bytes: Bytes) -> Result<(), Error> {
    if path.contains("..") || path.starts_with('/') {
        return Err(Error::custom(format!(
            "invalid artifact path: {path:?}"
        )));
    }
    let full = format!("{}{}", Self::prefix(deployment_id), path);
    self.disk.put(&full, bytes).await.map_err(Error::from)
}
```

Apply the same guard to `retrieve` and `remove`. Alternatively, enforce this invariant in the `DeploymentStorage` trait contract (doc comment) and let `StorageDeploymentStorage::store/retrieve/remove` validate at the impl level.

---

### WR-02: `ph()` fallback produces invalid placeholders for unsupported backends

**File:** `ferro-deployments/src/deployment.rs:434-438`

**Issue:** The `ph()` helper returns `?{n}` for all non-Postgres backends:

```rust
fn ph(backend: DatabaseBackend, n: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${n}"),
        _ => format!("?{n}"),   // SQLite accepts ?N; MySQL does not
    }
}
```

`promote()` in `promote.rs` correctly returns `Err(UnsupportedBackend)` for non-Postgres/SQLite backends. But all methods in `deployment.rs` (`create`, `mark_ready`, `mark_failed`, `get`, `list`, `active`, `rollback`) fall through the wildcard arm and emit `?1`, `?2`-style placeholders. MySQL (DatabaseBackend::MySql) uses plain `?` without a positional index, so these statements would reach the DB with malformed syntax and fail with a confusing driver error rather than a clean `UnsupportedBackend`.

**Fix:** Mirror the `promote.rs` guard inside `ph()` or add a backend check at the start of each `Deployments` method:

```rust
fn ph(backend: DatabaseBackend, n: usize) -> Result<String, Error> {
    match backend {
        DatabaseBackend::Postgres => Ok(format!("${n}")),
        DatabaseBackend::Sqlite => Ok(format!("?{n}")),
        _ => Err(Error::UnsupportedBackend),
    }
}
```

This makes `ph()` return `Result<String, Error>` and callers use `?` — the same posture `promote.rs` already takes.

---

### WR-03: `query_one` returning `Ok(None)` silently treated as "first promotion"

**File:** `ferro-deployments/src/promote.rs:73-86` (SQLite) and `117-130` (Postgres)

**Issue:** After executing the `INSERT … ON CONFLICT DO UPDATE … RETURNING` statement, both backends handle the result identically:

```rust
let row = match txn.query_one(stmt).await {
    Ok(r) => r,    // r: Option<QueryResult>
    Err(e) => { let _ = txn.rollback().await; return Err(Error::Db(e)); }
};
txn.commit().await.map_err(Error::Db)?;

Ok(row.and_then(|r| {
    r.try_get_by::<Option<i64>, _>("previous_deployment_id")
        .ok()
        .flatten()
}))
```

When `query_one` returns `Ok(None)` (no row from RETURNING), the transaction is committed and `Ok(None)` is returned — indistinguishable from a successful first-promotion. The `RETURNING` clause on a `INSERT … ON CONFLICT DO UPDATE` always returns exactly one row on both SQLite and Postgres, so in practice this path is unreachable. However, if a driver version or backend quirk fails to return the row, the pointer UPDATE is silently committed and the caller believes no previous deployment existed. The bug would corrupt `previous_deployment_id` silently.

**Fix:** Treat `Ok(None)` as an internal error:

```rust
let row = match txn.query_one(stmt).await {
    Ok(Some(r)) => r,
    Ok(None) => {
        let _ = txn.rollback().await;
        return Err(Error::custom("promote: RETURNING yielded no row; pointer state unknown"));
    }
    Err(e) => {
        let _ = txn.rollback().await;
        return Err(Error::Db(e));
    }
};
txn.commit().await.map_err(Error::Db)?;
let previous_id = row
    .try_get_by::<Option<i64>, _>("previous_deployment_id")
    .map_err(|e| Error::custom(format!("promote: parse previous_deployment_id: {e}")))?;
Ok(previous_id)
```

Apply identically to `promote_sqlite` and `promote_postgres`.

---

## Info

### IN-01: `rollback` silently fails when the previous deployment's artifact is GC'd

**File:** `ferro-deployments/src/deployment.rs:320-353`

**Issue:** `rollback` delegates to `promote`, which re-applies the `ArtifactDeleted` guard. If `previous_deployment_id` references a deployment whose artifact has been garbage-collected, rollback returns `Err(Error::ArtifactDeleted { id: prev_id })`. This is correct behavior, but it is not documented in the `rollback` method's doc comment. Callers expecting rollback to always succeed (e.g., in a disaster-recovery path) will encounter an opaque `ArtifactDeleted` error without guidance on what to do.

**Fix:** Add to the `rollback` doc comment:

```
/// # Errors
///
/// - `Error::NoPreviousDeployment` — no pointer row, or pointer has no previous id.
/// - `Error::ArtifactDeleted` — the previous deployment's artifact was GC'd and
///   can no longer be promoted. Use `list()` to find an earlier eligible deployment.
/// - `Error::NotReady` — the previous deployment is in a non-ready state (unexpected
///   in normal operation but possible if the pointer was mutated externally).
```

---

### IN-02: `preview_url` has no validation of `identifier` as subdomain-safe

**File:** `ferro-deployments/src/storage.rs:107-112`

**Issue:** `preview_url` takes a plain `&str` and concatenates it as a subdomain:

```rust
format!("https://{identifier}.{domain}/")
```

In production the identifier is always a UUID v4 (`[0-9a-f-]`), which is subdomain-safe. But the function signature accepts any `&str`, so a caller passing a raw `owner_key` or an unvalidated identifier would produce a malformed URL without error. There is no type-level or runtime enforcement of this invariant.

**Fix:** Add a note in the doc comment and consider a validation guard:

```rust
/// `identifier` must be a valid DNS label (RFC 1123): `[a-zA-Z0-9-]`, max 63 chars.
/// When `Deployment::identifier` (UUID v4) is passed, this is always satisfied.
pub fn preview_url(config: &DeploymentConfig, identifier: &str) -> Option<String> {
    // UUID v4 identifiers are safe; validate for defensive robustness.
    if identifier.is_empty() || identifier.len() > 63
        || !identifier.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return None;
    }
    config
        .preview_domain
        .as_ref()
        .map(|domain| format!("https://{identifier}.{domain}/"))
}
```

---

### IN-03: Race test tolerates `Err(Db)` outcomes, weakening the "both must succeed" invariant

**File:** `ferro-deployments/tests/race_promote_sqlite.rs:79-116`

**Issue:** The SQLite race test comments state:

```
// Tolerate either Ok or Err(Db): the important invariant is the pointer row state below.
```

If both concurrent promoters fail with `Err(Db)` (e.g., both get `SQLITE_BUSY` under high contention with no retry), `pointer_row` is asserted `expect("pointer row must exist after at least one successful promote")`. If neither promoter succeeded, this assertion panics — the test fails, but the failure message says "pointer row must exist" rather than "both promoters failed". More importantly, a partial-success scenario (one Ok, one Err) where the Err was actually a logic error (not a lock error) would be masked.

The test is structurally sound — the pointer-state assertions after the join are the right invariant. The issue is that the comment implies Db errors are acceptable without distinguishing transient lock errors from genuine logic bugs.

**Fix:** After `tokio::join!`, inspect error variants and distinguish transient from logic errors. At minimum, add an assertion that at least one promoter succeeded:

```rust
let at_least_one_ok = r1.is_ok() || r2.is_ok();
assert!(
    at_least_one_ok,
    "both promoters failed — at least one must succeed: r1={r1:?}, r2={r2:?}"
);
```

This ensures the pointer-state assertions below are actually reachable and meaningful.

---

_Reviewed: 2026-06-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
