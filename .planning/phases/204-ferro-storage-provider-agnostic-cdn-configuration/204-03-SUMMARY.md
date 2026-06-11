---
phase: 204-ferro-storage-provider-agnostic-cdn-configuration
plan: "03"
subsystem: ferro-storage/release
tags: [cdn, changelog, version-bump, docs, env, gate]
dependency_graph:
  requires: [cdn::Config, cdn::CdnProvider, config.rs-rewired, SC-3-parity, SC-4-parity]
  provides: [version-0.2.53, CHANGELOG-0.2.53, quartet-env-example, quartet-storage-docs, SC-7-gate]
  affects: [Cargo.toml, ferro-storage/CHANGELOG.md, app/.env.example, docs/src/features/storage.md]
tech_stack:
  added: []
  patterns: [keep-a-changelog, env-quartet-migration, workspace-version-bump]
key_files:
  created:
    - ferro-storage/CHANGELOG.md
  modified:
    - Cargo.toml
    - app/.env.example
    - docs/src/features/storage.md
    - ferro-storage/src/cdn/mod.rs  # fmt-only; no logic change
decisions:
  - "Workspace version 0.2.52 → 0.2.53 via [workspace.package]; ferro-storage inherits via version.workspace = true"
  - "Deprecated fallback table documents all 8 legacy vars with their quartet replacements"
  - "cdn_url_parity_aws_fallback flakiness is an env-collision under parallel test harness, confirmed by serial pass (62/62)"
  - "cargo fmt reformatted cdn/mod.rs call sites from Plans 01/02; no logic changes"
metrics:
  duration: "~45m (dominated by full workspace compilation under --all-features)"
  completed_date: "2026-06-11"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 4
  files_created: 1
---

# Phase 204 Plan 03: Version Bump, CHANGELOG, Docs Migration, and SC-7 Gate Summary

Workspace bumped to 0.2.53; `ferro-storage/CHANGELOG.md` created with the `## [0.2.53]` deprecation table; `app/.env.example` migrated to the CDN quartet; `docs/src/features/storage.md` CDN section updated to lead with the quartet; full SC-7 gate (fmt + clippy -D warnings + test --all-features) passed.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Bump workspace to 0.2.53 + create CHANGELOG.md | 9e14e1b3 | Cargo.toml, ferro-storage/CHANGELOG.md |
| 2 | Migrate .env.example and storage.md to quartet | 373f43d1 | app/.env.example, docs/src/features/storage.md |
| fmt-fix | Apply cargo fmt to cdn/mod.rs (Plans 01/02 code) | 407a4522 | ferro-storage/src/cdn/mod.rs, Cargo.lock |
| 3 | SC-7 gate: fmt + clippy + test --all-features | (gate only; no new files) | — |

## What Was Built

**`Cargo.toml` (workspace root)**
- `[workspace.package] version` bumped from `0.2.52` to `0.2.53`
- `ferro-storage` inherits via `version.workspace = true` (no change to ferro-storage/Cargo.toml)
- `cdn-bunny` / `cdn-cloudflare` remain non-default features (D-04 preserved)

**`ferro-storage/CHANGELOG.md` (new file)**
- Header: Keep-a-Changelog format modeled on `ferro-stripe/CHANGELOG.md`
- `## [0.2.53] - 2026-06-11` entry:
  - `### Added`: `cdn::Config`, `CdnProvider`, `Config::from_env()`, `Config::build_purge_api()`
  - `### Changed`: `StorageConfig::from_env` now reads `CDN_URL` primary via `cdn::Config`
  - `### Deprecated`: full 8-var deprecation table (`AWS_CDN_URL` / `BUNNY_CDN_URL` / `CF_CDN_URL` / `DO_SPACES_CDN_ID` / `CF_ZONE_ID` / `DIGITALOCEAN_ACCESS_TOKEN` / `CF_API_TOKEN` / `BUNNY_ACCESS_KEY`) mapped to quartet replacements; one-release removal policy
  - `### Notes`: SC-3 and SC-4 parity confirmed

**`app/.env.example`**
- Replaced the old per-provider CDN block (lines 79-91) with the provider-agnostic quartet:
  `CDN_URL=`, `CDN_PROVIDER=none`, `CDN_PURGE_TOKEN=`, `CDN_PURGE_ZONE=`
- Deprecation comment maps legacy vars to quartet replacements (D-06)
- `CDN_PROVIDER=none` default (no purge unless configured)
- All 8 legacy var bare assignments removed; they appear only in the comment

**`docs/src/features/storage.md`**
- `### CDN Edge URLs` now leads with the quartet env block + deprecation table
- Per-adapter env var references updated: DO adapter → `CDN_PURGE_ZONE`/`CDN_PURGE_TOKEN`; Bunny → quartet; Cloudflare → quartet
- Existing purge-adapter prose and code examples unchanged

## SC-7 Gate Results

| Step | Result |
|------|--------|
| `cargo fmt --all -- --check` | PASS (after auto-fmt of cdn/mod.rs) |
| `cargo clippy --all --all-targets -- -D warnings` | PASS (0 warnings, 1m 50s) |
| `cargo test -p ferro-storage --all-features -- --test-threads=1` | PASS (62/62) |
| `cargo test --all-features --workspace --exclude ferro-storage` | PASS (exit 0) |
| `cargo tree -e features \| grep cdn-bunny\|cdn-cloudflare` (default graph) | 0 matches — non-default confirmed |

**Note on parallel test flakiness:** The full-workspace `cargo test --all-features` showed 1 failure in `config::tests::cdn_url_parity_aws_fallback`. Root cause: `from_env_cdn_url` and `cdn_url_parity_aws_fallback` both set `AWS_CDN_URL` and run in parallel; the first test's value leaks into the second test's read. Confirmed env-collision (not a code defect) by running ferro-storage serially (`--test-threads=1` → 62/62 pass). The plan explicitly anticipates this and prescribes the serial re-run.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] cargo fmt reformatted ferro-storage/src/cdn/mod.rs**
- **Found during:** Task 3 (gate step 1)
- **Issue:** `cargo fmt --all -- --check` reported diffs in `cdn/mod.rs` (Plans 01/02 code): `env_with_fallback` call sites and `Self { url, provider, ... }` struct literal needed trailing commas and multi-line expansion; one `assert!` in a test needed multi-line format.
- **Fix:** `cargo fmt -p ferro-storage` applied; no logic changes.
- **Files modified:** ferro-storage/src/cdn/mod.rs, Cargo.lock (version bumped in lockfile)
- **Commit:** 407a4522

## Known Stubs

None.

## Threat Surface Scan

All threat model items confirmed:

| Threat ID | Status |
|-----------|--------|
| T-204-TOKEN-REDACT | Confirmed — .env.example uses `CDN_PURGE_TOKEN=` (empty assignment); CHANGELOG names vars only, no values |
| T-204-MISCONFIG | Confirmed — SC-7 gate compiled and ran both `--all-features` arms of `build_purge_api`; no misconfig regressions |
| T-204-DEPRECATION-LEAK | Accepted — CHANGELOG table names deprecated vars only, no token values |

## Self-Check: PASSED

Files exist:
- ferro-storage/CHANGELOG.md — FOUND
- Cargo.toml — FOUND (version = "0.2.53")
- app/.env.example — FOUND (CDN_PROVIDER present)
- docs/src/features/storage.md — FOUND (CDN_PROVIDER present)

Commits exist:
- 9e14e1b3 — FOUND (version bump + CHANGELOG)
- 373f43d1 — FOUND (.env.example + storage.md)
- 407a4522 — FOUND (fmt fix)
