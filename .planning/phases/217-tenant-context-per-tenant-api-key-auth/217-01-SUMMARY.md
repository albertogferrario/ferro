---
phase: 217-tenant-context-per-tenant-api-key-auth
plan: "01"
subsystem: ferro-mcp-oauth
tags: [auth, mcp, api-key, tenant-context, tdd-green, migration]
dependency_graph:
  requires:
    - ferro-mcp-oauth::validate_api_key (skeleton from 217-00)
    - ferro-mcp-oauth::generate_mcp_api_key (skeleton from 217-00)
  provides:
    - ferro-mcp-oauth::generate_mcp_api_key (real — BASE62 CSPRNG, ferro_ prefix, SHA-256 hash)
    - ferro-mcp-oauth::validate_api_key (real — async SHA-256 DB lookup, fail-closed)
    - ferro-mcp-oauth::CreateMcpApiKeysTable (new migration, mcp_api_keys schema)
  affects:
    - ferro-mcp-oauth/src/migration.rs
    - ferro-mcp-oauth/src/validate.rs
    - ferro-mcp-oauth/src/lib.rs
tech_stack:
  added: []
  patterns:
    - BASE62 CSPRNG key generation (rand::thread_rng, 43 chars + ferro_ prefix = 49 chars total)
    - SHA-256 hex hash storage via sha2::Sha256 — plaintext never persisted
    - SeaORM raw Statement::from_sql_and_values for parameterized async DB lookup
    - Fail-closed: Ok(None) and Err(_) both return BearerCheck::Invalid
    - revoked_at TEXT column read as Option<String> for SQLite/Postgres portability
key_files:
  created: []
  modified:
    - ferro-mcp-oauth/src/migration.rs
    - ferro-mcp-oauth/src/validate.rs
    - ferro-mcp-oauth/src/lib.rs
decisions:
  - revoked_at read as Option<String> (not TimeDateTimeWithTimeZone) so the same test fixture works for both SQLite TEXT and Postgres timestamptz without a type branch
  - BASE62 constant placed at module level in validate.rs (not inside the function) for clippy compliance
  - idx_mcp_api_keys_key_hash created as UNIQUE index; a separate UNIQUE constraint on the column was omitted to keep the migration portable (SeaORM .unique() on the column def would double-define uniqueness)
metrics:
  duration_minutes: ~8
  completed_date: "2026-06-13"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 3
---

# Phase 217 Plan 01: mcp_api_keys Migration + Real API Key Implementation Summary

Real `generate_mcp_api_key` (BASE62 CSPRNG, `ferro_`-prefixed 49-char key) and `validate_api_key` (async SHA-256 DB lookup against `mcp_api_keys`) replacing Plan 00 stubs; all RED tests GREEN.

## What Was Built

**Task 1 — `mcp_api_keys` migration:**

- `MigrationMcpApiKeys` struct in `ferro-mcp-oauth/src/migration.rs` alongside the existing `CreateOauthClientsTable` migration
- `up()` creates `mcp_api_keys` table: `id` (BIGINT PK autoincrement), `tenant_id` (BIGINT NOT NULL), `key_hash` (TEXT NOT NULL), `scope` (TEXT NOT NULL DEFAULT 'read'), `revoked_at` (TIMESTAMP WITH TIME ZONE NULL), `created_at`, `updated_at`
- Two indexes: `idx_mcp_api_keys_key_hash` (UNIQUE on `key_hash`) and `idx_mcp_api_keys_tenant_id` (non-unique on `tenant_id`)
- `down()` drops the table
- Migration test (`mcp_api_keys_migration_creates_table_and_indexes`) verifies table + both indexes via `sqlite_master` queries and verifies `down()` drops the table
- Exported as `CreateMcpApiKeysTable` from `ferro-mcp-oauth/src/lib.rs`

**Task 2 — Real key generator + validator (TDD GREEN):**

- `generate_mcp_api_key()`: `BASE62` constant (62 alphanumeric chars), `rand::thread_rng()` draws 43 indices, formats as `ferro_{random}` (49 chars), returns `(raw_key, hash_mcp_api_key(&raw_key))`
- `validate_api_key()`: strips `Bearer ` prefix, guards `ferro_` prefix, hashes token via `hash_mcp_api_key`, executes parameterized `SELECT id, tenant_id, scope, revoked_at FROM mcp_api_keys WHERE key_hash = ?`, reads `revoked_at` as `Option<String>` (non-null → `Invalid`), checks `expected_tenant`, returns `BearerCheck::Authenticated(json!({"sub": id.to_string(), "tenant_id": tenant_id, "scope": scope}))` on success

**Plan 00 RED tests now GREEN (5/5):**

| Test | Was | Now |
|------|-----|-----|
| `generate_mcp_api_key_is_prefixed_and_hash_matches` | RED (panic: raw_key must start with ferro_) | GREEN |
| `valid_api_key_returns_authenticated` | RED (got Invalid, expected Authenticated) | GREEN |
| `unknown_api_key_returns_invalid` | trivially GREEN | GREEN |
| `revoked_api_key_returns_invalid` | trivially GREEN | GREEN |
| `wrong_expected_tenant_returns_forbidden` | RED (got Invalid, expected Forbidden) | GREEN |

Total: `cargo test -p ferro-mcp-oauth` → 84 passed, 0 failed.

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written.

The one implementation detail not specified in the plan: `revoked_at` is read as `Option<String>` rather than `Option<DateTime<...>>`. This is a pragmatic choice: the test fixture inserts `revoked_at TEXT` in SQLite, and reading as `Option<String>` (non-null = revoked) is portable. The semantic is identical to what a timestamptz column would produce — the revocation check is "present or not", not a time comparison.

## Known Stubs

None — all stubs from Plan 00 replaced with real implementations.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes at new trust boundaries. All changes are in `ferro-mcp-oauth` (library crate, no HTTP surface). The `mcp_api_keys` schema was already declared in the threat model as T-217-02/T-217-03/T-217-04; no new surface beyond the plan's threat register.

## Self-Check: PASSED

- `ferro-mcp-oauth/src/migration.rs` — contains MigrationMcpApiKeys + mcp_api_keys + idx_mcp_api_keys_key_hash: FOUND
- `ferro-mcp-oauth/src/lib.rs` — contains CreateMcpApiKeysTable: FOUND
- `ferro-mcp-oauth/src/validate.rs` — contains FROM mcp_api_keys WHERE key_hash + BASE62 + format!("ferro_: FOUND
- `cargo test -p ferro-mcp-oauth` — 84 passed, 0 failed: VERIFIED
- No plaintext key column in migration.rs: VERIFIED
- Commit 76e91e50 (Task 1) exists: FOUND
- Commit 28f1d642 (Task 2) exists: FOUND
- Commit d03e7da7 (fmt) exists: FOUND
