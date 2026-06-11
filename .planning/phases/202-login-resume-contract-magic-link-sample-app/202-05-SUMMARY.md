---
phase: 202
plan: "05"
subsystem: quality-gate
tags: [gate, clippy, test, cargo-doc, cwd-boot, sc-5]
dependency_graph:
  requires: [202-01, 202-02, 202-03, 202-04]
  provides: [SC-5]
  affects: [ferro-mcp-oauth, app]
tech_stack:
  added: []
  patterns: [TestContainer thread-local isolation, TestContainerGuard RAII]
key_files:
  created:
    - .planning/phases/202-login-resume-contract-magic-link-sample-app/202-GATE.md
  modified:
    - ferro-mcp-oauth/src/lib.rs
    - ferro-mcp-oauth/src/token.rs
    - ferro-mcp-oauth/tests/flow_integration.rs
    - app/src/tests/magic_link.rs
    - app/src/tests/oauth_magic_link_resume_flow.rs
decisions:
  - "bootstrap_test_cache() returns TestContainerGuard (thread-local isolation) instead of writing to global App::bind"
metrics:
  duration: "~20 minutes"
  completed: "2026-06-11"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 6
---

# Phase 202 Plan 05: SC-5 Quality Gate Summary

**One-liner:** Full CI gate (fmt + clippy --all-features + test --all-features + cargo doc -D warnings) green; CWD-independent boot confirmed from repo root; parallel-test data race in bootstrap_test_cache fixed.

---

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Run fmt + clippy + test + cargo doc; fix issues; record evidence | e0a39965 | ferro-mcp-oauth/src/lib.rs, token.rs, flow_integration.rs, app/src/tests/magic_link.rs, oauth_magic_link_resume_flow.rs, 202-GATE.md |
| 2 | Confirm CWD-independent boot; no new from_path startup code | e0a39965 | 202-GATE.md (evidence recorded) |

---

## Gate Results

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | PASS (after auto-fix of import order + assert_eq formatting in oauth_magic_link_resume_flow.rs) |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | PASS (zero warnings) |
| `cargo test --all-features` | PASS (after bug fix — see Deviations) |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | PASS (zero doc warnings) |
| CWD-independent boot from repo root | PASS |

All four CI-matching gate commands exit 0. SC-5 satisfied.

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed parallel-test data race in `bootstrap_test_cache()`**

- **Found during:** Task 1 — `cargo test --all-features` first run
- **Failing tests:** `tests::magic_link::magic_link_single_use`, `tests::oauth_magic_link_resume_flow::oauth_magic_link_resume_flow`
- **Root cause:** `bootstrap_test_cache()` called `App::bind::<dyn CacheStore>(Arc::new(InMemoryCache::new()))` which writes to the **global** container (`APP_CONTAINER: OnceLock<RwLock<Container>>`). When Rust test runner executes async tests in parallel, a second invocation of `bootstrap_test_cache()` from a concurrent test replaces the global binding with a fresh empty `InMemoryCache`. The first test's subsequent `Cache::get` then resolves the new empty cache — the previously inserted token is not found, causing the assert to fail.
- **Why it appeared now:** These are new async tests (Phase 202) that happen to run concurrently. The older `ferro-mcp-oauth` tests used the same pattern but appeared to avoid the race by coincidence of scheduling order. Isolation was always absent; Phase 202 exposed it.
- **Fix:** `bootstrap_test_cache()` now uses `TestContainer::fake()` to create a thread-local container, then `TestContainer::bind()` to register the cache in that thread-local scope. Returns a `TestContainerGuard` that callers store as `let _cache = bootstrap_test_cache()`. The guard's `Drop` impl clears the thread-local container, ensuring each test is isolated regardless of parallel execution.
- **Files modified:** `ferro-mcp-oauth/src/lib.rs`, `ferro-mcp-oauth/src/token.rs` (×2), `ferro-mcp-oauth/tests/flow_integration.rs`, `app/src/tests/magic_link.rs` (×2), `app/src/tests/oauth_magic_link_resume_flow.rs`
- **Commit:** e0a39965

**2. [Rule 1 - fmt] Auto-formatted `oauth_magic_link_resume_flow.rs`**

- **Found during:** Task 1 — `cargo fmt --all -- --check` first run
- **Issue:** Import order (`use ferro::Cache` should follow `use ferro::session::with_test_session`; rustfmt reorders) and a multi-line `assert_eq!` expansion
- **Fix:** `cargo fmt --all` — trivial mechanical formatting
- **Files modified:** `app/src/tests/oauth_magic_link_resume_flow.rs`

---

## CWD-Independent Boot (SC-5)

- **Static check:** No `from_path(` in `app/src/bootstrap.rs`. `Theme::default_theme()` (embedded, CWD-independent) confirmed at line 75.
- **Phase 202 surface grep:** No `from_path(` found in any of the Phase 202 changed files.
- **Live boot check:** Binary launched from `/Users/alberto/repositories/albertogferrario/ferro` (repo root). Output: `Ferro server running on http://127.0.0.1:8080` — no panic, no CWD-relative file error.

---

## Decisions Made

1. `bootstrap_test_cache()` returns `TestContainerGuard` (breaking change to existing callers, all updated). Thread-local isolation is the correct pattern for test helpers that bind global-container services — `App::bind()` is not safe for parallel tests.

---

## Known Stubs

None — this plan contains no application code stubs. All changes are test infrastructure and gate evidence.

---

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Gate plan only.

---

## Self-Check: PASSED

- [x] `202-GATE.md` exists: `/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/202-login-resume-contract-magic-link-sample-app/202-GATE.md`
- [x] Commit e0a39965 exists in git log
- [x] All four gate commands recorded in GATE.md (`grep -q 'clippy'`, `grep -q 'test --all-features'`, `grep -q 'cargo doc'` all match)
- [x] CWD boot outcome recorded in GATE.md (`grep -qi 'boot'` matches)
- [x] No co-author lines in commit
