---
phase: 152
slug: ferro-orm-guardedupdate-atomic-conditional-updates-for-race
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-13
---

# Phase 152 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `tokio` async tests via `#[tokio::test]` (workspace convention; no separate test framework) |
| **Config file** | `ferro-orm/Cargo.toml` `[dev-dependencies]` — no separate test config |
| **Quick run command** | `cargo test -p ferro-orm` |
| **Full suite command** | `cargo test --all-features` (from workspace root) |
| **Estimated runtime** | ~5 seconds (in-memory SQLite, 7 unit tests + 1 integration test) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-orm`
- **After every plan wave:** `cargo test --all-features` from workspace root + `cargo clippy --all --all-targets -- -D warnings` + `cargo fmt --all -- --check`
- **Before `/gsd-verify-work`:** Full suite must be green; first-publish bootstrap (RESEARCH Pitfall 5) is post-merge and manual.
- **Max feedback latency:** ~10 seconds (single-crate run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| T-16-1 | TBD | TBD | (D-16) | — | predicate match → 1 row → `exec_one` returns `Ok(())` | unit | `cargo test -p ferro-orm predicate_matches_one_row_succeeds` | ❌ W0 | ⬜ pending |
| T-16-2 | TBD | TBD | (D-16) | — | predicate fails → 0 rows → `exec_one` → `Err(NoRowsAffected)`; `exec_at_most_one` → `Ok(false)` | unit | `cargo test -p ferro-orm predicate_fails_zero_rows` | ❌ W0 | ⬜ pending |
| T-16-3 | TBD | TBD | (D-16) | — | predicate matches >1 row → both methods return `Err(TooManyRows { affected: 2 })` | unit | `cargo test -p ferro-orm predicate_matches_multiple_rows` | ❌ W0 | ⬜ pending |
| T-16-4 | TBD | TBD | (D-12) | T-152-V5-01 | `EmptyUpdate` returned when no `set_*` called — distinguishes programmer bug from predicate miss | unit | `cargo test -p ferro-orm empty_update_no_sets` | ❌ W0 | ⬜ pending |
| T-16-5 | TBD | TBD | (D-07) | — | Multiple `.set_expr` / `.set_value` calls produce a single UPDATE that mutates all columns atomically | unit | `cargo test -p ferro-orm multi_column_set_atomic` | ❌ W0 | ⬜ pending |
| T-16-6 | TBD | TBD | (D-09) | — | Builder works inside `&DatabaseTransaction` — rollback rolls back the guarded update | unit | `cargo test -p ferro-orm transaction_rollback` | ❌ W0 | ⬜ pending |
| T-16-7 | TBD | TBD | (D-06) | T-152-V5-02 | Multiple `.filter` calls AND-combine — composed condition, no silent OR or last-wins | unit | `cargo test -p ferro-orm filter_and_combine` | ❌ W0 | ⬜ pending |
| T-17-1 | TBD | TBD | (D-14) | T-152-V5-03 | 10 tokio tasks vs counter K=3 → exactly 3 `Ok(())`, 7 `NoRowsAffected`, final counter = 0 — race-free claim is empirically demonstrated | integration | `cargo test -p ferro-orm --test concurrent_decrement` | ❌ W0 | ⬜ pending |

*Plan/Wave columns are TBD — populated when the planner produces PLAN.md and assigns waves.*

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-orm/Cargo.toml` — crate metadata + deps (`sea-orm`, `thiserror`) + dev-deps (`tokio` features matching `ferro-events`)
- [ ] `ferro-orm/src/lib.rs` — module rustdoc + targeted SeaORM re-exports
- [ ] `ferro-orm/src/error.rs` — `GuardedError` enum (`NoRowsAffected | TooManyRows { affected } | EmptyUpdate | Db(#[from] DbErr)`)
- [ ] `ferro-orm/src/guarded.rs` — `GuardedUpdate<E>` builder + `#[cfg(test)] mod tests` covering T-16-1 through T-16-7
- [ ] `ferro-orm/tests/concurrent_decrement.rs` — T-17-1 with `sqlite:file::memory:?cache=shared` (or tempfile sqlite) + `max_connections >= 4` + `#[tokio::test(flavor = "multi_thread")]` (per RESEARCH Pitfall 3 — the framework's standard `sqlite::memory:` + `max_connections=1` setup would serialize tasks at the pool layer and prove nothing)
- [ ] `ferro-orm/README.md` — one-paragraph crate purpose + canonical example (mirror `ferro-wallet/README.md`)
- [ ] `Cargo.toml` (workspace root) — append `"ferro-orm"` to `[workspace.members]`
- [ ] `.github/workflows/publish.yml` — append `ferro-orm` to `WAVE1A_CRATES` string
- [ ] `CHANGELOG.md` — new `## ferro-orm` section with version entry
- [ ] `CLAUDE.md` — add `ferro-orm` row to the Workspace Structure table
- [ ] `docs/src/SUMMARY.md` — add the new atomic-updates page entry (planner decides nesting — current docs has a single Database page)
- [ ] `docs/src/database/atomic-updates.md` — new user-facing doc page covering the `read → check → write` anti-pattern, the API, and the `exec_one` vs `exec_at_most_one` decision tree

No framework install needed — workspace already has Rust 1.88.0, `sea-orm` 1.0, and `tokio` in scope.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First-publish bootstrap to crates.io | D-24 | CI publish token has publish-update only — first publish of a NEW crate requires a personal `publish-new` token from a local terminal (per `project_ferro_publish_token_scoping.md` and Phase 151 PLAN-09 experience) | After Phase 152 verifies and merges to `master`: `cd ferro-orm && cargo publish --token $PUBLISH_NEW_TOKEN`. Subsequent versions auto-publish via the existing GH Actions workflow on master push. |
| `docs/src/SUMMARY.md` nesting decision | (D-21 ergonomic) | mdBook navigation is a small editorial choice that the planner picks; no automated test can ratify "navigation reads sensibly" | Planner picks nested-under-Database vs sibling top-level entry; reviewer eyeballs `mdbook serve` once for sensible TOC ordering. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
