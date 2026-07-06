---
phase: 214-scaffold-library-parity
plan: "01"
subsystem: framework + ferro-cli + ferro-mcp + docs
tags: [scaffold, templates, api-drift, facade, macros, sea-orm]
dependency_graph:
  requires: []
  provides: [ferro::error_response!, ferro::ActiveValue, corrected-scaffold-templates]
  affects: [ferro-cli/templates, framework/src/lib.rs, ferro-mcp/code_templates]
tech_stack:
  added: [error_response! macro]
  patterns: [macro_export-at-crate-root, ferro-facade-re-export]
key_files:
  created: []
  modified:
    - framework/src/lib.rs
    - ferro-cli/src/templates/scaffold.rs
    - ferro-cli/src/templates/make.rs
    - ferro-cli/src/templates/auth.rs
    - docs/src/the-basics/action-handlers.md
    - docs/src/features/database.md
    - ferro-mcp/src/tools/code_templates.rs
decisions:
  - "error_response! macro body uses HttpResponse::json().status() chain — matches verified HttpResponse API (no two-arg constructor exists)"
  - "ActiveValue moved from sea_orm import to ferro facade in all four scaffold template paths for consistency"
  - "auth.rs ActiveValue also moved to ferro facade (previously sea_orm::ActiveValue)"
metrics:
  duration: 416s
  completed: "2026-06-13"
  tasks: 3
  files: 7
---

# Phase 214 Plan 01: Scaffold–Library Parity (Template + Facade Fixes) Summary

Seven API-drift fixes that make a freshly scaffolded ferro app compile against the published `ferro-rs` crate. Two framework exports (D-01 `error_response!` macro, D-02 `ActiveValue` re-export) plus five template corrections spanning `scaffold.rs`, `make.rs`, and `auth.rs`, with matching docs and a new MCP code template.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Framework exports — error_response! + ActiveValue | 11acc90d | framework/src/lib.rs |
| 2 | Template fixes — scaffold.rs + make.rs | e4c067c7 | ferro-cli/src/templates/scaffold.rs, ferro-cli/src/templates/make.rs |
| 3 | auth.rs fixes + docs + ferro-mcp surface | f7f5c58f | ferro-cli/src/templates/auth.rs, docs/src/the-basics/action-handlers.md, docs/src/features/database.md, ferro-mcp/src/tools/code_templates.rs |

## What Was Built

### D-01 — `error_response!` macro (framework/src/lib.rs)
Added `#[macro_export] macro_rules! error_response!` after `text_response!`. Returns a bare
`HttpResponse` (json + status chain), not `Result`, so it works in `.map_err(|e| ...)` and
`.ok_or_else(|| ...)` closures where `?` does the unwrapping. Uses `$crate::HttpResponse::json`
and `$crate::serde_json::json!` — same path pattern as `json_response!`.

### D-02 — `ActiveValue` facade re-export (framework/src/lib.rs:122)
Added `ActiveValue` to the `pub use sea_orm::{...}` block (alphabetical order, after
`ActiveModelTrait`). Makes `ferro::ActiveValue` available without a direct `sea_orm` dependency.

### D-03/D-07 — job_template queue import (ferro-cli/src/templates/make.rs)
Changed `use ferro_queue::{async_trait, Error, Job, Queueable}` to
`use ferro::{async_trait, queue::{Error, Job, Queueable}}`. Generated apps need only
`ferro = { package = "ferro-rs", version = "0.2" }` in Cargo.toml — no `ferro-queue` dep.

### D-04 — ValidateRules derive on API form structs (ferro-cli/src/templates/scaffold.rs)
Both `api_controller_template` and `api_controller_with_fk_template` now derive
`ValidateRules` on the `{name}Form` struct. The `--api` path emits `#[rule(...)]` attributes
into `{form_fields}`, so without this derive the generated code failed with unknown-attribute
errors.

### Pitfall 2 — Inertia template `ActiveValue` consistency (ferro-cli/src/templates/scaffold.rs)
Both `scaffold_controller_template` and `scaffold_controller_with_fk_template` previously
imported `ActiveValue` from `use sea_orm::{...}`. Moved `ActiveValue` to the `use ferro::{...}`
block and removed it from the `sea_orm` block — consistent with the API templates.

### D-05 — model module path (ferro-cli/src/templates/auth.rs)
Renamed all `crate::models::users` (plural) to `crate::models::user` (singular) throughout
`auth_controller_template`. The base scaffold emits `pub mod user;` (singular) in
`models/mod.rs`; `make:auth` does not emit a separate model file.

### D-06 — DB connection call site (ferro-cli/src/templates/auth.rs)
Replaced the broken `&ferro::database::connection().await` (calling a module as a function)
with the correct two-step pattern:
```rust
let db = ferro::DB::connection()
    .map_err(|e| ferro::error_response!(500, e.to_string()))?;
user::Entity::insert(user).exec_with_returning(db.inner()).await ...
```
`ferro::DB::connection()` is synchronous and returns `Result<DbConnection, _>`;
`db.inner()` yields the `&DatabaseConnection` that SeaORM expects.

### Docs (D-01, D-02)
- `docs/src/the-basics/action-handlers.md`: added `## error_response! macro` subsection with
  usage example and explanation of bare-HttpResponse semantics.
- `docs/src/features/database.md`: added one-line note in Creating Records that `ferro::ActiveValue`
  is available as a facade re-export.

### ferro-mcp surface
Added `error_response_arm` CodeTemplate to `handler_templates()` in
`ferro-mcp/src/tools/code_templates.rs`. Surfaces `ferro::error_response!` to MCP-assisted
code generation so agents produce the same error-arm idiom as the scaffold.

## Verification

- `cargo build -p ferro-rs` exits 0
- `cargo build -p ferro-cli` exits 0
- `cargo build -p ferro-mcp` exits 0
- `cargo doc -p ferro-rs --no-deps` exits 0
- `cargo fmt --all -- --check` exits 0
- `cargo clippy -p ferro-rs/ferro-cli/ferro-mcp --all-targets -- -D warnings` exits 0

## Deviations from Plan

### Minor adaptation — D-06 error arm uses ferro::error_response! instead of raw HttpResponse chain

**Found during:** Task 3 (D-06)
**Issue:** The RESEARCH.md example showed a raw `HttpResponse::json(serde_json::json!(...)).status(500)` in the D-06 fix. Since Task 1 already added `ferro::error_response!`, using it in the auth template error arm is both shorter and consistent with the rest of the generated code.
**Fix:** Used `ferro::error_response!(500, e.to_string())` in the `DB::connection()` map_err.
**Files modified:** ferro-cli/src/templates/auth.rs

### Minor deviation — auth.rs ActiveValue import moved to ferro facade (not in plan)

**Found during:** Task 3 review
**Issue:** auth.rs still had `use sea_orm::ActiveValue` after the D-05 fix. Task 2 moved `ActiveValue` to the ferro facade in all scaffold templates; auth.rs had an inconsistency.
**Fix:** Changed to `use ferro::ActiveValue` for consistency with scaffold templates. No behavioral change — both resolve to the same type.
**Files modified:** ferro-cli/src/templates/auth.rs

## Known Stubs

None. All template fixes emit resolved symbols. The `{form_fields}`, `{insert_fields}`, and `{update_fields}` slots remain parameterized (as intended — they are filled at scaffolding time by the CLI command).

## Threat Flags

None. Changes are confined to the developer-facing scaffold surface (local CLI, developer-controlled inputs). The `error_response!` macro was reviewed against T-214-03 (elevation of privilege) — the macro expands only to an existing `HttpResponse` builder chain; no new capability is granted.

## Self-Check: PASSED

- `framework/src/lib.rs` modified — commit 11acc90d confirmed
- `ferro-cli/src/templates/scaffold.rs` modified — commit e4c067c7 confirmed
- `ferro-cli/src/templates/make.rs` modified — commit e4c067c7 confirmed
- `ferro-cli/src/templates/auth.rs` modified — commit f7f5c58f confirmed
- `docs/src/the-basics/action-handlers.md` modified — commit f7f5c58f confirmed
- `docs/src/features/database.md` modified — commit f7f5c58f confirmed
- `ferro-mcp/src/tools/code_templates.rs` modified — commit f7f5c58f confirmed
