---
phase: 186-ferro-deployments-immutable-deployments-atomic-promote
plan: "04"
subsystem: ferro-deployments
tags: [doc-test, criterion-5, docs, version-bump, publish-dry-run]
dependency_graph:
  requires: [186-01 (crate scaffold), 186-02 (lifecycle API), 186-03 (storage trait)]
  provides: [criterion-5 doc-test, deployments feature docs page, workspace version 0.2.45]
  affects: [ferro-deployments/src/lib.rs, docs/src/SUMMARY.md, Cargo.toml workspace version]
tech_stack:
  added: []
  patterns: [tokio::runtime::Runtime::new().unwrap().block_on() doc-test harness, hidden # lines in doc-test for clean rendered output]
key_files:
  created:
    - docs/src/features/deployments.md
  modified:
    - ferro-deployments/src/lib.rs
    - docs/src/SUMMARY.md
    - Cargo.toml
decisions:
  - "Doc-test uses tokio::runtime::Runtime::new().unwrap().block_on(async { ... }) — no tokio_test dev-dep needed; tokio is already a [dependencies] entry with macros+rt features"
  - "DeploymentStorage trait explicitly imported in doc-test hidden section — trait methods (store/retrieve) require it in scope"
  - "Version bump committed before publish dry-run — cargo publish rejects uncommitted changes without --allow-dirty"
  - "First manual publish reminder: ferro-deployments does not yet exist on crates.io; CI token is publish-update only; one-time cargo publish -p ferro-deployments from a local terminal required before first CI push"
metrics:
  duration: "471s"
  completed: "2026-06-07"
  tasks: 3
  files: 4
---

# Phase 186 Plan 04: Phase Closeout — Doc-test, Docs Page, Version Bump Summary

Criterion 5 proven: a non-HTML JSON spec bundle (`{"intent":"browse","fields":[]}`) stores through the full lifecycle API in a passing crate-level doc-test. The crate contains zero HTML and zero app-identity assumptions. Feature docs page written in neutral architectural voice. Workspace bumped to 0.2.45. Publish dry-run packages cleanly.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Criterion-5 doc-test (non-HTML JSON artifact through full API) | 9ab56a03 | ferro-deployments/src/lib.rs |
| 2 | Feature docs page + SUMMARY.md entry | f3f0003b | docs/src/features/deployments.md, docs/src/SUMMARY.md |
| 3 | Workspace version bump + publish dry-run gate | cf248726 | Cargo.toml |

## Verification

- `cargo test -p ferro-deployments --doc` — 1 passed (criterion-5 doc-test), 3 ignored (existing `ignore` examples)
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean, zero warnings, all 29 workspace crates checked at v0.2.45
- `cargo test --all-features` — full workspace green, no failures
- `cargo publish -p ferro-deployments --dry-run` — packages cleanly: 14 files, 147.2 KiB (36.3 KiB compressed); verification compile succeeded

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] DeploymentStorage trait not imported in doc-test hidden section**
- **Found during:** Task 1 `cargo test -p ferro-deployments --doc` (first run)
- **Issue:** `storage.store(...)` and `storage.retrieve(...)` call trait methods on `StorageDeploymentStorage`. Rust requires the trait to be in scope (`DeploymentStorage`) for trait method dispatch. The initial hidden import block omitted the trait.
- **Fix:** Added `DeploymentStorage` to the `# use ferro_deployments::{...}` hidden import line in the doc-test.
- **Files modified:** `ferro-deployments/src/lib.rs`
- **Commit:** 9ab56a03

**2. [Rule 3 - Blocking] Version bump must be committed before publish dry-run**
- **Found during:** Task 3 `cargo publish -p ferro-deployments --dry-run` (first run)
- **Issue:** `cargo publish` rejects with "files in the working directory contain changes that were not yet committed" when `Cargo.toml` has the version bump uncommitted.
- **Fix:** Committed the version bump (`cf248726`) before running the dry-run.
- **Files modified:** `Cargo.toml`
- **Commit:** cf248726

## Manual First-Publish Reminder

`ferro-deployments` is a new crate — it does not yet exist on crates.io. The CI publish token has `publish-update` scope only (not `publish-new`). Before the first CI push:

```bash
cargo publish -p ferro-deployments
```

Run from a local terminal with a full-scope API token. This is the same pattern as `ferro-bundle` (Phase 183 reminder). After the first manual publish succeeds, all subsequent publishes are handled by the Wave 1b CI job.

## Known Stubs

None. All tasks deliver complete implementations — the doc-test runs and passes, the docs page is fully written, and the version bump is committed.

## Threat Flags

No new threat surface. The changes in this plan are: a crate-level doc comment, a Markdown documentation page, and a Cargo.toml version bump. No new network endpoints, auth paths, or file access patterns are introduced.

T-186-08 (preview URL addressability): mitigated as planned — `docs/src/features/deployments.md` explicitly states "Preview URLs are publicly addressable by design. The subdomain identifier is not an access-control token and does not restrict who can fetch the URL. The consumer application owns authorization for preview routes."

T-186-11 (doc-test information disclosure): accepted — the doc-test uses placeholder `owner_key` "project:demo" and a generic JSON spec; zero app identity, zero secrets.

## Self-Check: PASSED

Files exist:
- ferro-deployments/src/lib.rs: FOUND (modified — doc-test added)
- docs/src/features/deployments.md: FOUND
- docs/src/SUMMARY.md: FOUND (modified — nav entry added)
- Cargo.toml: FOUND (modified — version bumped to 0.2.45)

Commits exist:
- 9ab56a03: FOUND (feat(186-04): add criterion-5 doc-test storing JSON spec through full lifecycle)
- f3f0003b: FOUND (docs(186-04): add ferro-deployments feature docs page and SUMMARY.md entry)
- cf248726: FOUND (chore(186-04): bump workspace version to 0.2.45)
