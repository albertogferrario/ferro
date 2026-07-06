# Phase 202 — Quality Gate Evidence

**Gate run date:** 2026-06-11
**Executor:** claude-sonnet-4-6
**Plan:** 202-05 (SC-5 gate)

---

## Gate Commands + Outcomes

### 1. `cargo fmt --all -- --check`

**Result:** FAIL on first run → FIXED → PASS

**Diff found in:** `app/src/tests/oauth_magic_link_resume_flow.rs`

Two formatting issues:
- Import order (`use` statement reordering: `ferro::Cache` and `ferro_mcp_oauth::*`)
- Multi-line `assert_eq!` formatting for a 302-status assertion

**Fix applied:** `cargo fmt --all` — auto-formatted.

**Re-check result:** EXIT 0 (clean)

---

### 2. `cargo clippy --all --all-targets --all-features -- -D warnings`

**Result:** PASS (zero warnings)

**Output:** All workspace crates checked; `Finished dev profile` with no warnings or errors.

No clippy warnings introduced by Phase 202.

---

### 3. `cargo test --all-features`

**Result:** FAIL on first run → ROOT CAUSE FIXED → PASS

**Failures (first run):**
```
test tests::magic_link::magic_link_single_use ... FAILED
test tests::oauth_magic_link_resume_flow::oauth_magic_link_resume_flow ... FAILED
```

**Root cause:** `bootstrap_test_cache()` used `App::bind::<dyn CacheStore>(...)` which writes to the **global** container. When multiple async tests run in parallel, a second call to `bootstrap_test_cache()` from a concurrent test overwrites the global binding with a fresh empty `InMemoryCache`. The first test's subsequent `Cache::get` then resolves the new empty cache — the previously inserted token is not found.

This is a parallel-test data race on the global container. It is not a logic bug in the cache or the token lifecycle.

**Fix applied (Rule 1 — Bug):**

Updated `ferro-mcp-oauth/src/lib.rs` — `bootstrap_test_cache()` now uses the thread-local `TestContainer` (via `TestContainer::fake()` + `TestContainer::bind()`) instead of the global `App::bind()`. It returns a `TestContainerGuard` that the caller must hold for the test's duration; the guard's `Drop` impl clears the thread-local container, ensuring isolation.

**Files modified:**
- `ferro-mcp-oauth/src/lib.rs` — `bootstrap_test_cache()` uses `TestContainer::fake()` + `TestContainer::bind()`, returns `TestContainerGuard`
- `ferro-mcp-oauth/src/token.rs` — two call sites: `bootstrap_test_cache()` → `let _cache = bootstrap_test_cache()`
- `ferro-mcp-oauth/tests/flow_integration.rs` — one call site updated
- `app/src/tests/magic_link.rs` — two call sites updated
- `app/src/tests/oauth_magic_link_resume_flow.rs` — one call site updated

**Re-run result:**

```
test result: ok. 16 passed; 0 failed; 0 ignored; ...  (app)
test result: ok. 55 passed; 0 failed; 0 ignored; ...  (ferro-mcp-oauth)
test result: ok. 1 passed; 0 failed; 0 ignored; ...   (flow_integration)
```

All workspace test suites: **PASS**

**Key tests confirmed present and green:**
- `tests::magic_link::magic_link_single_use`
- `tests::magic_link::magic_link_expired`
- `tests::magic_link::magic_link_dev_surface`
- `tests::oauth_magic_link_resume_flow::oauth_magic_link_resume_flow`
- `tests::oauth_magic_link_resume_flow::oauth_magic_link_resume_flow_no_key_falls_back_to_default`
- `controllers::auth_controller::tests::login_view_is_valid_and_posts_to_login`
- `token::tests::forget_before_validate_single_use`
- `token::tests::replay_code_returns_none_after_forget`
- `resume::tests::*` (5 tests)
- `tests::mcp_tenant_isolation::tests::*` (3 tests)

---

### 4. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`

**Result:** PASS

Zero doc warnings across the workspace. The new `bootstrap_test_cache()` doc comment (with `# Example`) renders correctly.

**Output:** `Finished dev profile` → `Generated .../target/doc/app/index.html and 31 other files`

---

## CWD-Independent Boot Check (SC-5)

### Static check — no new `from_path` at startup

**Command:** `grep -n "from_path(" app/src/bootstrap.rs`
**Result:** No matches — `from_path(` is absent from `bootstrap.rs`.

**Command:** `grep -q "default_theme" app/src/bootstrap.rs`
**Result:** Match found at line 75:
```rust
global_middleware!(ThemeMiddleware::new().default_theme(Theme::default_theme()));
```
The embedded default theme (commit 10263291) is retained. No CWD-sensitive startup code was introduced by Phase 202.

**Phase 202 diff surface grep** (ferro-mcp-oauth/src/, app/src/controllers/auth_controller.rs, app/src/tests/):
**Result:** `NO from_path in Phase 202 surface` — confirmed clean.

### Live boot check from repo root

**Working directory used:** `/Users/alberto/repositories/albertogferrario/ferro` (repo root, NOT `app/`)

**Command:**
```
APP_ENV=local DATABASE_URL=sqlite:///tmp/ferro_gate_boot_test.db \
SESSION_SECRET=test_session_secret_for_gate_boot_check_32b \
timeout 8 ./target/debug/app
```

**Output observed:**
```
[seed] Dogfood fixture seeded: 2 tenants, 2 users, 4 orders
Ferro server running on http://127.0.0.1:8080
```

**Result:** PASS — the app process started, ran the DB seed, and reached the "Ferro server running" listening state without any CWD-relative startup panic or "No such file or directory" error. SC-5 CWD-independent boot confirmed.

---

## Summary

| Gate | Result | Notes |
|------|--------|-------|
| `cargo fmt --all -- --check` | PASS (after auto-fix) | 1 file reformatted |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | PASS | Zero warnings |
| `cargo test --all-features` | PASS (after bug fix) | `bootstrap_test_cache` data race fixed |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | PASS | Zero doc warnings |
| CWD-independent boot (SC-5) | PASS | Boots from repo root; no from_path at startup |
