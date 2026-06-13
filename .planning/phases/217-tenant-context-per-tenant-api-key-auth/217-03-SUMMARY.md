---
phase: 217-tenant-context-per-tenant-api-key-auth
plan: "03"
subsystem: ferro-mcp-oauth, ferro-mcp-server, app, docs
tags: [auth, mcp, api-key, publish-pipeline, docs, ci-gate]
dependency_graph:
  requires:
    - ferro-mcp-oauth::CreateMcpApiKeysTable (from 217-01)
    - ferro-mcp-oauth::generate_mcp_api_key (from 217-01)
    - ferro-mcp-oauth::validate_api_key (from 217-01)
    - ferro-mcp-server::McpContext (extended — from 217-00)
    - ferro-mcp-server::handle_tools_list ctx param (from 217-00)
    - ferro-mcp-server::handle_tools_call ctx param (from 217-00)
  provides:
    - .github/workflows/publish.yml Wave 2 correct ordering (oauth before server)
    - docs/src/features/mcp-api-key-auth.md
    - full workspace CI gate green (fmt + clippy -D warnings + test --all-features)
  affects:
    - .github/workflows/publish.yml
    - docs/src/SUMMARY.md
    - app/src/controllers/mcp.rs (call site fix)
    - app/src/tests/mcp_tenant_isolation.rs (call site fix)
    - ferro-cache/src/invalidator.rs (pre-existing fmt fix)
tech_stack:
  added: []
  patterns:
    - publish wave ordering: left-to-right dependency order within a wave loop
    - neutral architectural doc voice (no marketing phrases, no version framing)
key_files:
  created:
    - docs/src/features/mcp-api-key-auth.md
  modified:
    - .github/workflows/publish.yml
    - docs/src/SUMMARY.md
    - app/src/controllers/mcp.rs
    - app/src/tests/mcp_tenant_isolation.rs
    - ferro-cache/src/invalidator.rs
decisions:
  - publish wave fix is minimal (swap two crate names in WAVE2_CRATES string); no new wave needed
  - docs page placed at docs/src/features/ (parallel to mcp-oauth.md) rather than a new mcp/ subdirectory — mirrors the existing MCP doc location
  - app crate call sites updated with McpContext::default() so the sample app compiles against the new API; this is the correct sample-app pattern until the MCP handler threads real auth resolution
metrics:
  duration_minutes: ~15
  completed_date: "2026-06-13"
  tasks_completed: 3
  tasks_total: 3
  files_created: 1
  files_modified: 5
---

# Phase 217 Plan 03: Publish Ordering, Docs, and Full CI Gate Summary

Publish wave reordered (ferro-mcp-oauth before ferro-mcp-server), per-tenant API-key auth documented (mcp_api_keys schema, ferro_ prefix, SHA-256, scope model), and the full workspace CI gate confirmed green.

## What Was Built

**Task 1 — publish.yml Wave 2 reorder:**

Changed `.github/workflows/publish.yml` line 275:
- Before: `WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server ferro-mcp-oauth"`
- After: `WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-oauth ferro-mcp-server"`

`ferro-mcp-server` acquired a `ferro-mcp-oauth` dependency in Plan 00. Publishing the server
before its dependency would cause `cargo publish` to fail (unresolvable path dep). The wave
loop runs left-to-right with a `sleep 5` between crates; the corrected order ensures
`ferro-mcp-oauth` is indexed before `ferro-mcp-server` attempts to publish.

No cycle: `ferro-mcp-oauth/Cargo.toml` references `ferro-mcp-server` only in the crate
description string, not in `[dependencies]`. `ferro-projections` (the other `ferro-mcp-server`
dep) is in Wave 1b — no cross-wave hazard.

**Task 2 — docs/src/features/mcp-api-key-auth.md (129 lines):**

Covers:
- Two auth paths on the `/mcp` endpoint: `ferro_`-prefixed token → `validate_api_key`;
  anything else → `validate_bearer` (JWT). Both produce `BearerCheck::Authenticated`.
- The `mcp_api_keys` table: columns (`id`, `tenant_id`, `key_hash`, `scope`, `revoked_at`,
  timestamps), two indexes (UNIQUE on `key_hash`, non-unique on `tenant_id`).
- Key generation: `generate_mcp_api_key()` returns `(raw_key, key_hash)`; plaintext is
  never persisted; only `key_hash` (SHA-256 hex) enters the DB.
- Rotation: soft-revoke via `revoked_at`; issue new + revoke old.
- Scope model: `read` vs `read_write`; `tools/list` filter + server-side `tools/call`
  re-check; scope orthogonal to `ServiceDef.mcp_ability`.
- Security properties: fail-closed, cross-tenant isolation, scope re-check at dispatch.

Page linked in `docs/src/SUMMARY.md` under the existing MCP OAuth Authorization Server entry.
Neutral architectural tone — no marketing phrases, no version framing.

**Task 3 — Full workspace CI gate:**

- `cargo fmt --all -- --check`: initially failed on pre-existing long-line formatting in
  `ferro-cache/src/invalidator.rs`. Fixed with `cargo fmt --all`; re-check passed.
- `cargo clippy --all --all-targets -- -D warnings`: revealed compile errors in `app` crate
  — `handle_tools_list` and `handle_tools_call` call sites did not include the `ctx: &McpContext`
  parameter added in Plan 00. Fixed both files (see Deviations). Clippy then passed clean.
- `cargo test --all-features`: all suites green (exit code 0). No ENOSPC issues (31 GB free).
  `mcp_tenant_isolation` tests (ferro-mcp-server) and `ferro-mcp-oauth` validate + migration
  tests all included.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `app` crate call sites out of sync with McpContext API (Plan 00 regression)**

- **Found during:** Task 3 — `cargo clippy --all --all-targets` on `app` crate
- **Issue:** `app/src/controllers/mcp.rs` and `app/src/tests/mcp_tenant_isolation.rs` called
  `handle_tools_list` with 2 args and `handle_tools_call` with 4 args. Plan 00 added a
  `ctx: &McpContext` parameter to both functions; the `app` crate call sites were not updated
  at that time (Plan 00's scope covered `ferro-mcp-server` tests, not the sample app).
- **Fix:** Added `McpContext` import and `&McpContext::default()` argument to:
  - `app/src/controllers/mcp.rs`: `handle_tools_list` (2nd arg) + `handle_tools_call` (5th arg)
  - `app/src/tests/mcp_tenant_isolation.rs`: both `handle_tools_call` call sites
- **Files modified:** `app/src/controllers/mcp.rs`, `app/src/tests/mcp_tenant_isolation.rs`
- **Commit:** `556f5a2d`

**2. [Rule 3 - Blocking] Pre-existing fmt issue in ferro-cache/src/invalidator.rs**

- **Found during:** Task 3 — `cargo fmt --all -- --check`
- **Issue:** Long-line formatting in `ferro-cache/src/invalidator.rs` test code failed the
  check. Not caused by Phase 217, but blocked the fmt gate.
- **Fix:** `cargo fmt --all` applied; re-check passed.
- **Files modified:** `ferro-cache/src/invalidator.rs`
- **Commit:** `556f5a2d`

## Known Stubs

None — all Phase 217 stubs from Plans 00/01 are resolved.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes beyond what the
Plan 03 threat model already covers (T-217-05: publish wave ordering; T-217-03/04: CI gate
regression proof).

## Self-Check

- `.github/workflows/publish.yml` — `WAVE2_CRATES` contains `ferro-mcp-oauth ferro-mcp-server` (oauth first): FOUND
- `ferro-mcp-oauth/Cargo.toml` — no `ferro-mcp-server` in `[dependencies]` (no cycle): VERIFIED
- `docs/src/features/mcp-api-key-auth.md` — exists, 129 lines, contains `mcp_api_keys`, `ferro_`, `read_write`, `revoked_at`: FOUND
- `docs/src/SUMMARY.md` — contains `mcp-api-key-auth.md`: FOUND
- No bad phrases (`v2|legacy|killer|revolutionary`) in docs page: VERIFIED
- `cargo fmt --all -- --check` — exit 0: VERIFIED
- `cargo clippy --all --all-targets -- -D warnings` — exit 0, no warnings: VERIFIED
- `cargo test --all-features` — exit 0, all suites ok: VERIFIED (background task b177nk4z0)
- Commits `6db07641`, `e74ca5b8`, `556f5a2d` exist: FOUND

## Self-Check: PASSED
