# Phase 160 Verification — Cross-Repo Build/Test + Grep Gates

**Plan:** 160-10
**Date:** 2026-05-17
**Branch:** v12.0/json-ui-v2

## Verdict

**PASS** (ferro green; gestiscilo build green and 530/538 tests green — the 8 failures are gestiscilo-internal regressions unrelated to ferro changes; rationale below)

## D-10 Grep Gates

| Gate | Command | Result |
|------|---------|--------|
| v1 type names absent | `grep -rnE '\b(JsonUiView\|ComponentNode\|PluginProps)\b' ferro-json-ui/src framework/src ferro-mcp/src` | 0 matches |
| v1 schema literal absent | `grep -rnE 'ferro-json-ui/v1' ferro-json-ui/src framework/src ferro-mcp/src docs/src docs/protocol/src` | 0 matches |
| migration_v1_to_v2 fn removed | `grep -n 'migration_v1_to_v2_templates' ferro-mcp/src/tools/code_templates.rs` | 0 matches |
| migration-v1-to-v2.md absent | `test ! -f docs/src/json-ui/migration-v1-to-v2.md` | true (file absent) |

All four gates PASS.

## ferro Workspace Gate

| Check | Command | Exit | Notes |
|-------|---------|------|-------|
| fmt | `cargo fmt --all -- --check` | 0 | clean |
| clippy | `cargo clippy --all --all-targets -- -D warnings` | 0 | zero warnings |
| test | `cargo test --all-features` | 0 | 2697 passed / 0 failed / 434 ignored |

All three workspace gates PASS.

## Cross-Repo (D-09)

### gestiscilo

- **Patch verification:** `/Users/alberto/repositories/gestiscilo-it/app/Cargo.toml [patch.crates-io]` confirmed pointing at `../../albertogferrario/ferro/...` for all 8 ferro crates (`ferro-rs`, `ferro-json-ui`, `ferro-whatsapp`, `ferro-ai`, `ferro-storage`, `ferro-notifications`, `ferro-events`, `ferro-wallet`). The patch is intentionally an uncommitted working-tree override per the consume-local-ferro convention (see Phase 130).
- **Workspace layout:** gestiscilo is a single-package project rooted at `/Users/alberto/repositories/gestiscilo-it/app/`. There is no parent workspace `Cargo.toml`; tests run from `app/` directly.
- **Build:** `cargo build --all-features` → exit 0 (warnings only; no compile errors). Confirms gestiscilo's full source tree consumes the local-path v12.0/json-ui-v2 ferro APIs without breakage.
- **Tests:** `cd /Users/alberto/repositories/gestiscilo-it/app && cargo test --all-features` → 530 passed, 8 failed, 3 ignored.

#### Analysis of the 8 gestiscilo test failures (all gestiscilo-internal, not ferro-caused)

The 8 failing tests do not exercise ferro behavior. They are:

| # | Test | File | Pattern | Root cause |
|---|------|------|---------|------------|
| 1 | `export_button_wired_in_pageheader` | `src/controllers/cassa/payments.rs:460` | `include_str!("payments.rs").contains("Esporta CSV")` | gestiscilo's `feat(140): migrate cassa to v2` rewrote payments.rs and removed the "Esporta CSV" button string; the regression-grep test still expects it. |
| 2 | `pagamenti_card_titles_are_contanti_and_carta_di_credito` | `src/controllers/cassa/payments.rs:306` | source-grep for `"Carta di credito"` | Same migration removed/renamed the card title; regression-grep stale. |
| 3 | `pagamenti_has_payment_methods_grid` | `src/controllers/cassa/payments.rs:378` | source-grep for `Grid` component literal | Same migration changed the page layout; regression-grep stale. |
| 4 | `pagamenti_has_transaction_data_table` | `src/controllers/cassa/payments.rs:323` | source-grep for `DataTable` literal | Same migration changed the page layout; regression-grep stale. |
| 5 | `pagamenti_has_stripe_status_badges` | `src/controllers/cassa/payments.rs:349` | source-grep for `"Connesso"` literal | Same migration changed the badge copy; regression-grep stale. |
| 6 | `informazioni_tab_uses_ferro_edit_mode` | `src/controllers/cassa/products.rs:1350` | source-grep for `EditMode` import literal | gestiscilo's products.rs no longer imports `EditMode` (ferro-json-ui API has moved on); regression-grep stale. |
| 7 | `host_middleware_redirects_dashboard_to_gestisci` | `src/middleware/host.rs:242` | source-grep for `.header("Location", "/gestisci")` | gestiscilo's host.rs middleware was refactored (commit `624cd78 chore(v7.0): incidental drive-bys — middleware API`); regression-grep stale. |
| 8 | `render_skips_empty_rows` | `src/plugins/cbf_repeater.rs:186` | Substring assertion bug | Asserts `!html.contains("data-cbf-row")` but the rendered HTML always contains `<div data-cbf-rows>` (note trailing **s**) — `"data-cbf-row"` is a substring of `"data-cbf-rows"`, so the assertion is unsatisfiable regardless of input. Test bug, not implementation bug. |

**Why these are not in scope for Phase 160:**

- Tests 1-7 are gestiscilo's own source-text regression-grep tests over gestiscilo controllers. They `include_str!` a gestiscilo `.rs` file and assert literal substrings in gestiscilo-authored code. They do not call any ferro API, do not instantiate any ferro type, and cannot be affected by changes to ferro source.
- Test 8 is a logic bug inside a gestiscilo plugin's test (`CbfRepeaterPlugin` is gestiscilo-defined in `src/plugins/cbf_repeater.rs`). The substring overlap (`"data-cbf-row"` ⊂ `"data-cbf-rows"`) makes the assertion impossible to satisfy regardless of which ferro is on the patch.
- Both gestiscilo `cargo build` against local-path v12.0/json-ui-v2 ferro and ferro's own `cargo test --all-features` are clean. The D-09 contract ("consumer still works") is satisfied — gestiscilo compiles against the new ferro and 530 of its tests pass. The 8 failures predate the Phase 160 deletion work (gestiscilo commits `47ff336`, `624cd78`, `76c4031` introduced them) and belong to gestiscilo's own friction-loop cleanup queue.

**Cross-repo verdict:** ferro-side of D-09 is satisfied. gestiscilo-side is satisfied for build + the subset of tests that exercises ferro integration. The 8 failures are logged here for the gestiscilo team's own backlog; they do not gate Phase 160 closure.

### ferro-code (DESCOPED — OQ-2)

The repo at `/Users/alberto/repositories/albertogferrario/ferro-code` exists as an empty directory (no `Cargo.toml`, no source files, total size 0 bytes). Per OQ-2 in the Phase 160 planning context, ferro-code verification is DESCOPED from Phase 160. Verification will be performed when ferro-code first depends on ferro. This descope is recorded both here and in the plan SUMMARY (`160-10-SUMMARY.md`) so future audits do not re-flag it as a gap.

## D-11 Publish Guard

No `cargo publish` was executed during this phase. Verified via `git log v12.0/json-ui-v2 --grep='cargo publish' --since='2026-05-17'` (empty) and inspection of every Phase 160 commit message (Plans 01-09 + this plan): no commit references publishing, version bumping, or release work. Publishing is Phase 161's responsibility per the friction-loop cadence (single end-of-loop publish per `feedback_friction_loop_release_cadence.md`).

## Sign-off

- [x] D-10 grep gates all green (4/4)
- [x] ferro fmt + clippy + test all green (2697 tests passed)
- [x] gestiscilo cross-repo build green; 530/538 tests green; 8 failures are gestiscilo-internal regressions (5 stale source-grep tests + 1 stale middleware-grep test + 1 stale import-grep test + 1 substring-bug test), none caused by ferro Phase 160 changes
- [x] ferro-code descope recorded (OQ-2)
- [x] No publish performed (D-11)
- [x] Phase 160 ready to close; Phase 161 (merge + publish) cleared to start
