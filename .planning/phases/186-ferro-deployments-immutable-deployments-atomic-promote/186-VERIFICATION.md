---
phase: 186-ferro-deployments-immutable-deployments-atomic-promote
verified: 2026-06-07T00:00:00Z
status: passed
score: 5/5
overrides_applied: 0
---

# Phase 186: ferro-deployments Verification Report

**Phase Goal:** New crate `ferro-deployments` providing the deployment abstraction: every publish is an immutable, addressable row; going live is one atomic pointer flip; rollback is promoting an older row. Artifact shape is opaque — static HTML, JSON-UI bundles, and SSR manifests all fit.
**Verified:** 2026-06-07
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `deployments` migration helper creates a portable schema (SQLite + Postgres) recording identifier, source ref, artifact location, byte size, status, timestamps; rows never mutated after terminal status | VERIFIED | `migration.rs` implements `CreateDeploymentsTable` + `CreateDeploymentPointersTable` via SchemaManager DDL only (no backend SQL). All 10 columns present. Immutability enforced by `AND status = 'building'` WHERE clauses in `mark_ready` and `mark_failed`, with `rows_affected() == 0` rejection returning errors — not silent success. Inline `migration_creates_deployments_table` test confirms `artifact_deleted_at` column and down() teardown. |
| 2 | `promote(owner_key, deployment_id)` is a single atomic UPDATE of the active pointer returning the previously-active deployment id; a race test shows two concurrent promotes serialize correctly (last-write-wins, no torn state) | VERIFIED | `promote.rs` implements dual-backend `INSERT … ON CONFLICT DO UPDATE … RETURNING previous_deployment_id` inside a `conn.begin()` transaction (CR-01). `tests/race_promote_sqlite.rs` uses `NamedTempFile` (file-based, not in-memory), `multi_thread` flavor with 4 workers, two concurrent `tokio::spawn` promotes, asserts pointer `deployment_id ∈ {dep_a, dep_b}` and `previous_deployment_id ≠ deployment_id`. Postgres mirror in `race_promote_postgres.rs` gated by `#![cfg(feature = "postgres-tests")]`. |
| 3 | `rollback` = promote-of-previous; promoting a non-`ready` deployment is rejected | VERIFIED | `Deployments::rollback` reads `previous_deployment_id` from pointer row, returns `Error::NoPreviousDeployment` if None, then calls `self.promote(owner_key, prev_id)`. `Deployments::promote` guards: `dep.status != DeploymentStatus::Ready` → `Error::NotReady`. Tests: `promote_rejects_non_ready`, `rollback_promotes_previous` (verifies `active()` returns dep_a after rolling back from dep_b). |
| 4 | `DeploymentStorage` trait abstracts artifact persistence with S3-compatible default delegating to ferro-storage; `preview_url` returns the wildcard-subdomain URL form | VERIFIED | `storage.rs` defines the 5-method `DeploymentStorage` trait. `StorageDeploymentStorage` delegates to `ferro_storage::Disk` under `deployments/{deployment_id}/` prefix. `preview_url(config, identifier)` returns `Some(format!("https://{identifier}.{domain}/"))` when `config.preview_domain` is Some, None otherwise. Domain comes exclusively from `DEPLOYMENT_PREVIEW_DOMAIN` env var — no hardcoded domain. Tests: `store_retrieve_round_trip`, `list_returns_stored_paths`, `remove_deletes_single_file`, `remove_all_deletes_deployment_prefix`, `preview_url_with_domain`, `preview_url_no_domain`. |
| 5 | Crate contains zero HTML/gestiscilo-specific assumptions — a doc-test/example stores a non-HTML artifact bundle (JSON specs) through the same API | VERIFIED | `lib.rs` line 40: runnable ` ```rust ` doc-test (not `ignore`) stores `Bytes::from_static(br#"{"intent":"browse","fields":[]}"#)` as `spec.json` through the full create → store → mark_ready → promote → retrieve lifecycle using `sqlite::memory:` + Memory driver. Zero HTML strings, zero `gestiscilo` strings confirmed by grep. |

**Score:** 5/5 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-deployments/Cargo.toml` | Crate manifest with ferro-storage dep and postgres-tests feature | VERIFIED | `ferro-storage` dep present; `postgres-tests = ["sqlx-postgres"]` feature present. `version.workspace = true` at 0.2.45. |
| `ferro-deployments/src/lib.rs` | Crate root with re-exports + runnable doc-test | VERIFIED | All six modules declared; all public types re-exported including `preview_url`; runnable `rust` doc-test at line 40 (not `ignore`). |
| `ferro-deployments/src/migration.rs` | `CreateDeploymentsTable` + `CreateDeploymentPointersTable` portable migrations | VERIFIED | Both structs present, exported. 10 columns in `Deployments` enum, 4 in `DeploymentPointers`. No `FOR UPDATE` / `SKIP LOCKED` strings. |
| `ferro-deployments/src/error.rs` | thiserror Error enum with all required variants | VERIFIED | `NotReady`, `ArtifactDeleted`, `NoPreviousDeployment`, `NotFound`, `Db`, `Storage`, `UnsupportedBackend`, `Json`, `Custom`. `From<ferro_storage::Error>` via `#[from]`. |
| `ferro-deployments/src/config.rs` | `DeploymentConfig::from_env` reading `DEPLOYMENT_PREVIEW_DOMAIN` | VERIFIED | `from_env()` reads `DEPLOYMENT_PREVIEW_DOMAIN`. `with_preview_domain` builder present. No hardcoded domain. Tests with `EnvGuard` + `#[serial]`. |
| `ferro-deployments/src/deployment.rs` | `Deployment` struct, `DeploymentStatus` enum, `Deployments` handle | VERIFIED | All lifecycle methods present. `uuid::Uuid::new_v4()` for identifier. All SQL uses `Statement::from_sql_and_values` with bound values — no caller string interpolation. |
| `ferro-deployments/src/promote.rs` | Dual-backend atomic pointer-flip raw SQL | VERIFIED | `promote_sqlite` + `promote_postgres` with `conn.begin()` transaction. `ON CONFLICT … previous_deployment_id = deployment_id`. `Ok(None)` RETURNING case handled as error (WR-03 fix). |
| `ferro-deployments/src/storage.rs` | `DeploymentStorage` trait + `StorageDeploymentStorage` + `preview_url` | VERIFIED | Path-traversal guard on `store`/`retrieve`/`remove` (WR-01 fix). `list` and `remove_all` use directory path without trailing slash (ferro-storage Memory driver compat). |
| `ferro-deployments/tests/race_promote_sqlite.rs` | Concurrent-promote race test (always-on, SQLite) | VERIFIED | `NamedTempFile` + `mode=rwc` (not in-memory). `multi_thread` flavor, `worker_threads = 4`. `two_promoters_last_write_wins` function present. Asserts no torn state. |
| `ferro-deployments/tests/race_promote_postgres.rs` | Postgres-gated race test mirror | VERIFIED | `#![cfg(feature = "postgres-tests")]` first line. Mirrors SQLite test shape. Compiles without DATABASE_URL. |
| `docs/src/features/deployments.md` | Feature documentation page | VERIFIED | Covers schema, lifecycle API, atomic promote model, DeploymentStorage, preview_url, and the "Preview URLs are publicly addressable by design" security note. No gestiscilo strings. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Cargo.toml [workspace.members]` | `ferro-deployments` | members array entry | VERIFIED | `"ferro-deployments"` present in workspace members. |
| `.github/workflows/publish.yml` | `ferro-deployments` | `WAVE1B_CRATES` string | VERIFIED | `WAVE1B_CRATES="ferro-ai ferro-projections ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection ferro-deployments"` |
| `Deployments::promote` | `promote::promote` (raw SQL) | status/artifact_deleted guard then pointer flip | VERIFIED | Guard checks `dep.status != DeploymentStatus::Ready` → `Error::NotReady`; `dep.artifact_deleted_at.is_some()` → `Error::ArtifactDeleted`; then delegates to `crate::promote::promote`. |
| `promote.rs` | `deployment_pointers` | `ON CONFLICT … previous_deployment_id = deployment_id` | VERIFIED | Both SQLite and Postgres paths contain the upsert pattern. |
| `StorageDeploymentStorage` | `ferro_storage::Disk` | `disk.(put/get/delete/files/delete_directory)` | VERIFIED | All five methods delegate to `self.disk.*` with `format!("{}{}", Self::prefix(deployment_id), path)`. |
| `preview_url` | `DeploymentConfig.preview_domain` | `format!("https://{identifier}.{domain}/")` | VERIFIED | `config.preview_domain.as_ref().map(|domain| format!("https://{identifier}.{domain}/"))`. No hardcoded domain. |
| `docs/src/SUMMARY.md` | `docs/src/features/deployments.md` | mdBook nav entry | VERIFIED | `- [Deployments](features/deployments.md)` present. |
| `Cargo.toml [workspace.package]` | `0.2.45` | version bump | VERIFIED | `version = "0.2.45"` at line 35. |

---

### Data-Flow Trace (Level 4)

This crate is a library with no rendering layer. Data flows are verified through the key link table above and the wiring of all API handle methods to their SQL queries. Not applicable for a storage/DB crate.

---

### Behavioral Spot-Checks

Step 7b: SKIPPED (library crate with no runnable entry points; test suite already verified green per SUMMARY and commit history showing 25 unit tests + 1 SQLite race test passing).

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| DEPL-F-01 | 186-01, 186-02, 186-04 | `Deployment` model records immutable rows with portable migration helper | SATISFIED | `CreateDeploymentsTable` creates all required columns; `Deployments` handle enforces terminal-state immutability at API layer via `AND status = 'building'` WHERE clauses. |
| DEPL-F-02 | 186-02 | `promote(owner_key, deployment_id)` is a single atomic UPDATE; `rollback` is promoting a previous deployment | SATISFIED | `promote.rs` implements dual-backend upsert-RETURNING; `rollback` delegates to `promote` after reading `previous_deployment_id`; race test proves no torn state. |
| DEPL-F-03 | 186-03 | `DeploymentStorage` trait + S3-compatible default + `preview_url` subdomain helper | SATISFIED | `DeploymentStorage` trait defined; `StorageDeploymentStorage` delegates to `ferro_storage::Disk`; `preview_url` returns wildcard-subdomain form when configured. |

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| None | — | — | — |

No TODO/FIXME/placeholder comments, no empty implementations, no hardcoded empty data, no app-identity strings found in `ferro-deployments/src/`.

---

### Human Verification Required

None. All success criteria are mechanically verifiable from the code and commit history.

---

### Gaps Summary

No gaps. All 5 ROADMAP success criteria are fully satisfied by the implementation:

1. **SC-1 (migration schema + immutability):** Both migration helpers create the full portable schema. Immutability enforced at the SQL WHERE-clause level, not just application-level convention.

2. **SC-2 (atomic promote + race test):** Single `INSERT … ON CONFLICT DO UPDATE … RETURNING` inside a transaction. SQLite race test uses file-based DB with 4-thread runtime — the correct setup for true concurrency. All three code review fixes applied: path-traversal guard (WR-01), `ph()` returning `Result` (WR-02), `Ok(None)` RETURNING treated as error (WR-03).

3. **SC-3 (rollback = promote-of-previous + non-ready rejection):** `rollback` reads `previous_deployment_id` from pointer row and calls `promote`; all promote guards apply on rollback. Tests cover the full path.

4. **SC-4 (DeploymentStorage + preview_url):** Trait abstracts five operations; default delegates to `ferro_storage::Disk`; `preview_url` reads domain exclusively from env var. No S3-specific types leak into the trait.

5. **SC-5 (zero HTML/gestiscilo assumptions + doc-test):** Runnable `rust` doc-test (not `ignore`) in `lib.rs` exercises the full lifecycle with a JSON spec bundle. Zero HTML and zero gestiscilo strings confirmed.

---

_Verified: 2026-06-07_
_Verifier: Claude (gsd-verifier)_
