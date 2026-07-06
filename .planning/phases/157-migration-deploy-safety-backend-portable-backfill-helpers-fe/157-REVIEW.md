---
phase: 157-migration-deploy-safety-backend-portable-backfill-helpers-fe
reviewed: 2026-05-14T00:00:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - .github/workflows/publish.yml
  - CLAUDE.md
  - Cargo.toml
  - app/src/main.rs
  - ferro-cli/src/doctor/check.rs
  - ferro-cli/src/doctor/checks/migrate_gate.rs
  - ferro-cli/src/doctor/checks/mod.rs
  - ferro-cli/src/doctor/registry.rs
  - ferro-cli/src/templates/do.rs
  - ferro-cli/src/templates/files/backend/main.rs.tpl
  - ferro-cli/tests/fixtures/gestiscilo/app.yaml
  - ferro-cli/tests/gestiscilo_fixture.rs
  - ferro-migration/Cargo.toml
  - ferro-migration/README.md
  - ferro-migration/src/backfill.rs
  - ferro-migration/src/error.rs
  - ferro-migration/src/lib.rs
  - framework/src/app.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 157: Code Review Report

**Reviewed:** 2026-05-14T00:00:00Z
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

This phase introduces `ferro-migration` (backend-portable backfill helpers), a `MigrateGateCheck` doctor check that detects missing PRE_DEPLOY migrate jobs in `.do/app.yaml`, an updated `main.rs` template with `run_migrations_silent` abort-on-failure semantics, and the `Application` builder in `framework/src/app.rs` with the same abort-on-failure guard.

The core logic is sound. Three warnings were found: one data-correctness bug in the odd-`hex_len` path of `backfill_random_hex`, one internal reference in a committed doc comment that exposes a tenant name and incident date, and one unsafe `.expect()` call in the server startup path of `framework/src/app.rs`. Two info items cover a missing `hex_len = 0` guard and an unvalidated `table`/`column` injection surface.

## Warnings

### WR-01: Integer truncation produces wrong output for odd `hex_len` values

**File:** `ferro-migration/src/backfill.rs:35`
**Issue:** `let byte_len = hex_len / 2;` uses integer (floor) division. Callers requesting `hex_len = 5` get `byte_len = 2`, producing a 4-character hex string instead of 5. The doc comment says the function "fills … with a random hex string of `hex_len` characters", so the silent truncation violates the contract. The README example uses `hex_len = 16` (even), which hides the bug in practice, but odd values are a reachable input.

**Fix:** Either reject odd values with a clear error, or round up so the output is at least `hex_len` characters long and document the rounding:
```rust
// Round up so output is always >= hex_len characters.
let byte_len = hex_len.div_ceil(2);  // Rust 1.73+ stable
```
Or validate before calling:
```rust
if hex_len % 2 != 0 {
    return Err(Error::UnsupportedBackend(
        format!("hex_len must be even, got {hex_len}"),
    ));
}
let byte_len = hex_len / 2;
```
A test covering `hex_len = 5` should accompany whichever fix is chosen.

---

### WR-02: Internal tenant identity and incident reference committed to a framework doc comment

**File:** `framework/src/app.rs:395`
**Issue:** The doc comment reads:
```
/// motivated this guard (see Phase 157, gestiscilo-it 2026-05-13 incident).
```
Per CLAUDE.md §6 "Project-agnostic crates", `ferro-*` crates must not hardcode any application identity. A tenant name (`gestiscilo-it`) and a dated incident reference are internal context that belongs in a memory file, not in a committed doc comment on a published crate. The same principle applies to `framework/`.

**Fix:** Replace with a neutral description of the invariant:
```rust
/// "Silent" refers only to the success path (no progress logs that would
/// interleave with server startup). On failure this method writes to stderr
/// and aborts the process to prevent the server from accepting traffic with
/// a stale schema.
```

---

### WR-03: `.expect()` in server startup path panics without actionable context

**File:** `framework/src/app.rs:354`
**Issue:** `Server::from_config(router).run().await.expect("Failed to start server")` panics with a bare string on server startup failure. The equivalent code in `app/src/main.rs` uses the `fail_with` helper that prints the cause and a "How to fix" list before exiting, making the error actionable. The template (`ferro-cli/src/templates/files/backend/main.rs.tpl` line 110) also uses `.expect()`, so generated apps inherit this gap.

**Fix:** Mirror `app/src/main.rs`'s pattern — unwrap with an explicit error log and `std::process::exit(1)` rather than a bare panic:
```rust
if let Err(e) = Server::from_config(router).run().await {
    eprintln!("Server failed to start: {e}");
    std::process::exit(1);
}
```
The template at `ferro-cli/src/templates/files/backend/main.rs.tpl:110` needs the same change.

---

## Info

### IN-01: `hex_len = 0` produces a syntactically valid but semantically useless UPDATE

**File:** `ferro-migration/src/backfill.rs:35`
**Issue:** `hex_len = 0` yields `byte_len = 0`, which generates `randomblob(0)` on SQLite (returns an empty blob → empty hex string) and `gen_random_bytes(0)` on Postgres (also valid, returns empty). The column gets filled with empty strings, which immediately matches the `OR "" = ''` filter on the next run. No error is raised; the migration silently does nothing useful.

**Fix:** Add a guard at the top of `sql_for_random_hex`:
```rust
if hex_len == 0 {
    return Err(Error::UnsupportedBackend(
        "hex_len must be > 0".into(),
    ));
}
```

---

### IN-02: `table` and `column` are interpolated directly into SQL without validation

**File:** `ferro-migration/src/backfill.rs:37-48`
**Issue:** `table` and `column` are passed directly into the SQL string via `format!`. These values come from migration source code written by the developer (not from user input), so this is not an injection risk in normal use. However, there is no validation layer, so a typo like `table = "bookings; DROP TABLE users"` would produce malformed but potentially harmful SQL. The pattern is common in migration helpers, but noting it as a documented scope constraint would prevent future misuse.

**Fix:** Add a doc note to the public API:
```rust
/// # Safety
/// `table` and `column` must be valid SQL identifiers controlled by the
/// migration author. Values from user input must never be passed here.
```

---

_Reviewed: 2026-05-14T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
