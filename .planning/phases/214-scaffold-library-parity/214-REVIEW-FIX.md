---
phase: 214-scaffold-library-parity
fixed_at: 2026-06-13T00:00:00Z
review_path: .planning/phases/214-scaffold-library-parity/214-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 214: Code Review Fix Report

**Fixed at:** 2026-06-13
**Source review:** .planning/phases/214-scaffold-library-parity/214-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (CR-01, WR-01, WR-02)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: Full-stack scaffold controller templates emit non-existent `HttpResponse` methods

**Files modified:** `ferro-cli/src/templates/scaffold.rs`
**Commit:** 0d50b346
**Applied fix:** Replaced all `HttpResponse::internal_server_error`, `HttpResponse::not_found`, `HttpResponse::bad_request`, and `HttpResponse::redirect` calls in both `scaffold_controller_template` and `scaffold_controller_with_fk_template` with the correct published-facade equivalents:

- Error arms: `ferro::error_response!(500, ...)`, `ferro::error_response!(404, ...)`, `ferro::error_response!(400, ...)`
- Post-`req.input()` redirects in `store` and `update` (req consumed): `Inertia::redirect_ctx(&ctx, format!(...))`
- `destroy` redirect (req still in scope): `Inertia::redirect(&req, "/{plural}")`
- Dropped `HttpResponse` from the `use ferro::{ http::{Request, Response, ...} }` import in both templates (no longer referenced)

The FK fetch helper strings (`fk_index_fetches`, `fk_create_fetches`, `fk_edit_fetches`) built dynamically in the FK variant were also fixed from `HttpResponse::internal_server_error(e.to_string())` to `ferro::error_response!(500, e.to_string())`.

`cargo build -p ferro-cli` and `cargo clippy -p ferro-cli --all-targets -- -D warnings` both pass clean. The guard test (see WR-01) confirmed the generated `post_controller.rs` compiles without errors against the workspace ferro.

### WR-01: Compile smoke test covers only the `--api` template family

**Files modified:** `ferro-cli/tests/benchmark_new_project.rs`
**Commit:** 8f042845
**Applied fix:** Added Step 3d — `ferro make:scaffold Post title:string body:text` (no `--api` flag) between the existing `--api` scaffolds and the `cargo build` step. This exercises `scaffold_controller_template` and the Inertia page templates on every PR. The full guard test (`scaffold_builds_against_workspace_ferro`) passed in 63s with the non-`--api` Post controller compiling cleanly.

No additional template drift was surfaced by the extended test — the CR-01 fix was sufficient for the generated code to compile.

### WR-02: `Validator::passes()` / `fails()` ignore pre-seeded errors

**Files modified:** `framework/src/validation/validator.rs`
**Commit:** 2590a9c4
**Applied fix:** Added the `pre_errors` drain loop at the top of `passes()`, mirroring the pattern already in `validate()`. A validator with only a `with_error` pre-error now correctly reports `passes() == false` / `fails() == true`. `fails()` delegates to `passes()` so no change needed there. `cargo build -p ferro-rs` and `cargo clippy -p ferro-rs --all-targets -- -D warnings` both pass clean.

---

_Fixed: 2026-06-13_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
