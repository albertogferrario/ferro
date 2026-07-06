---
phase: 202-login-resume-contract-magic-link-sample-app
plan: "04"
subsystem: app
tags: [oauth, magic-link, acceptance-test, session, cache, single-use-token, offline-ci]
dependency_graph:
  requires:
    - 202-01 (ferro-mcp-oauth resume helpers: store_oauth_return_to, oauth_resume_redirect, with_test_session)
    - 202-02 (magic-link handlers: verify_magic_link, login; cache_test_helpers::bootstrap_test_cache)
  provides:
    - app::tests::oauth_magic_link_resume_flow (SC-3 offline acceptance test)
  affects:
    - app/src/tests/mod.rs (module registered)
tech_stack:
  added: []
  patterns:
    - staged offline acceptance test (bootstrap_test_cache + with_test_session, no live server)
    - SC-3 async-flow staged as sequential in-session steps
key_files:
  created:
    - app/src/tests/oauth_magic_link_resume_flow.rs
  modified:
    - app/src/tests/mod.rs
decisions:
  - "Two test functions: one for the full flow (steps 1-4), one for the no-key fallback (step 5); each in its own with_test_session scope"
  - "Removed grep-command example from module doc-comment — the literal forbidden symbols would cause the offline CI grep to false-positive"
  - "Used with_test_session to provide session continuity across all four SC-3 steps within a single test function"
metrics:
  duration: ~10 minutes
  completed: "2026-06-11"
  tasks_completed: 1
  files_modified: 1
  files_created: 1
---

# Phase 202 Plan 04: SC-3 offline acceptance test (OAuth magic-link resume flow) Summary

Staged async-flow acceptance test proving the login-resume contract end-to-end: `store_oauth_return_to` → `Cache::put` token → `Cache::get`/`forget` (single-use) → `oauth_resume_redirect` redirects 302 to the stored `/authorize` URL. Fully offline — no `reqwest`, no live server, no `render_file`.

## Completed Tasks

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | SC-3 staged acceptance test (TDD: file is the RED→GREEN artifact) | 85eb3e5c | oauth_magic_link_resume_flow.rs, tests/mod.rs |

## What Was Built

`app/src/tests/oauth_magic_link_resume_flow.rs` — two `#[tokio::test]` functions:

**`oauth_magic_link_resume_flow`** (SC-3 steps 1–4): drives the full logical sequence within a single `with_test_session` scope so the `oauth_return_to` key stored in step 1 is visible to the redirect in step 4:

- Step 1: `store_oauth_return_to("/authorize?client_id=test&...")` — simulates `/authorize` handler storing the return URL.
- Step 2: `Cache::put("magic_link:{token}", &user_id, Some(15min))` + assert present — simulates `POST /auth/login` issuing the token.
- Step 3: `Cache::get` → `Cache::forget` → assert second get returns `None` — proves single-use invariant (T-202-01, mirrors `token.rs` lines 62-64).
- Step 4: `oauth_resume_redirect("/")` → assert `302` + `Location == stored /authorize URL` — proves the resume contract.

**`oauth_magic_link_resume_flow_no_key_falls_back_to_default`** (SC-3 step 5): fresh `with_test_session` scope, no stored key — asserts `oauth_resume_redirect("/")` returns `302` to `"/"`.

`app/src/tests/mod.rs` — `pub mod oauth_magic_link_resume_flow;` added.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed useless `.into()` on `Cache::get` return value**
- **Found during:** Task 1 — clippy reported `useless_conversion` on `Option<i64>` → `Option<i64>`.
- **Issue:** The initial draft used `.into()` on the `Cache::get` result already typed as `Option<i64>`.
- **Fix:** Removed `.into()`.
- **Files modified:** `app/src/tests/oauth_magic_link_resume_flow.rs`
- **Commit:** 85eb3e5c (inline fix before commit)

**2. [Rule 1 - Bug] Removed grep-command example from module doc-comment**
- **Found during:** Task 1 — the offline acceptance criteria grep (`! grep -qE 'reqwest|...'`) matched the doc-comment that documented the grep itself.
- **Issue:** The module doc included a code block showing the exact grep command, which contains the forbidden symbols (`reqwest`, `TcpListener`, `render_file`, `bind(`). The acceptance criteria grep returned `FAIL`.
- **Fix:** Replaced the grep example with a plain prose statement of the offline guarantee.
- **Files modified:** `app/src/tests/oauth_magic_link_resume_flow.rs`
- **Commit:** 85eb3e5c (inline fix before commit)

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. The acceptance test is test-only code; it adds no runtime surface.

## Known Stubs

None.

## Verification

- `test -f app/src/tests/oauth_magic_link_resume_flow.rs`: PASS
- `grep -q 'pub mod oauth_magic_link_resume_flow' app/src/tests/mod.rs`: PASS
- `grep -q 'store_oauth_return_to' app/src/tests/oauth_magic_link_resume_flow.rs`: PASS
- `grep -q 'oauth_resume_redirect' app/src/tests/oauth_magic_link_resume_flow.rs`: PASS
- `grep -q 'Cache::forget' app/src/tests/oauth_magic_link_resume_flow.rs`: PASS
- `grep -q 'magic_link:' app/src/tests/oauth_magic_link_resume_flow.rs`: PASS
- `! grep -qE 'reqwest|render_file|TcpListener|bind\(' app/src/tests/oauth_magic_link_resume_flow.rs`: PASS
- `cargo test -p app oauth_magic_link_resume_flow`: 2 passed, 0 failed
- `cargo clippy -p app --all-targets -- -D warnings`: clean

## Self-Check: PASSED

- `app/src/tests/oauth_magic_link_resume_flow.rs`: EXISTS
- `pub mod oauth_magic_link_resume_flow` in tests/mod.rs: CONFIRMED
- `store_oauth_return_to` called in test: CONFIRMED
- `oauth_resume_redirect` called in test: CONFIRMED
- `Cache::forget` called in test (single-use step): CONFIRMED
- No forbidden offline symbols: CONFIRMED
- Commit 85eb3e5c: FOUND in git log
