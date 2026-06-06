---
phase: 183-ferro-bundle-capability-new-crate
plan: 03
subsystem: infra
tags: [bundle, tests, integration, etag, cache, 304, 301, alias]

# Dependency graph
requires:
  - phase: 183-ferro-bundle-capability-new-crate
    provides: "Plan 02 core impl: Bundle struct + pub(crate) serve_inner dispatcher + OnceLock<DashMap<...>> registries"
provides:
  - "#[doc(hidden)] pub mod __test_internals containing a thin pub fn wrapper around the crate-private serve_inner dispatcher (integration-test reachability)"
  - "ferro-bundle/tests/serve_cold.rs — BUNDLE-02 cold path (200 + Content-Type + Cache-Control + quoted SHA-256 ETag + body bytes)"
  - "ferro-bundle/tests/serve_304.rs — BUNDLE-02 304 fast-path on If-None-Match exact-quoted match (RFC 7232 §4.1 — ETag + Cache-Control round-trip; empty body)"
  - "ferro-bundle/tests/alias_redirect.rs — BUNDLE-03 alias 301 redirect to current hashed URL"
affects:
  - 183-04-plan (publish bootstrap + README polish; the crate is functionally complete after this plan)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Doc-hidden pub mod __test_internals as integration-test bridge: thin pub fn wrapper delegating to a pub(crate) dispatcher (Rust forbids pub use of pub(crate); the wrapper is the visibility-conforming idiom)"
    - "One integration test per binary (cargo's tests/*.rs default): OS-level process isolation across registry-mutating tests; no in-binary reset() coordination needed"
    - "Per-file header_value(&[(String,String)], &str) -> Option<&str> case-insensitive header lookup helper (duplicated by design — promoting to tests/common/ would add a separate compilation unit per common file, overkill for a 5-line helper)"

key-files:
  created:
    - ferro-bundle/tests/serve_cold.rs
    - ferro-bundle/tests/serve_304.rs
    - ferro-bundle/tests/alias_redirect.rs
    - .planning/phases/183-ferro-bundle-capability-new-crate/183-03-SUMMARY.md
  modified:
    - ferro-bundle/src/lib.rs (+23 lines: __test_internals shim module)

key-decisions:
  - "Replace planned `pub use crate::serve_inner;` with a thin `pub fn` wrapper inside the same `#[doc(hidden)] pub mod __test_internals`. Rust rejects pub-use of pub(crate) items at greater visibility (E0364); the wrapper is the visibility-conforming idiom and is zero-cost (#[inline])."
  - "Bundle names are globally unique across test binaries by design (serve-cold-sdk / serve-304-sdk / alias-redirect-sdk) so even under unforeseen test-runner pooling, the duplicate-name panic guard from D-06 will not fire."
  - "static BYTES: &[u8] = b\"...\"; in each test gives the &'static [u8] lifetime Bundle::new requires, without forcing the test author to think about lifetimes."

patterns-established:
  - "Doc-hidden pub fn wrapper for crate-private integration-test reachability: when a downstream consumer (here, an integration test binary) needs to reach a pub(crate) symbol and pub-use is rejected by E0364, wrap it in a #[doc(hidden)] pub fn shim. Module name prefixed with __ signals 'do not call from production'."

requirements-completed: [BUNDLE-02, BUNDLE-03]

# Metrics
duration: ~13min
completed: 2026-06-06
---

# Phase 183 Plan 03: Integration Tests Summary

**Three single-test binaries under `ferro-bundle/tests/` verify BUNDLE-02 (cold 200 + 304 fast-path) and BUNDLE-03 (alias 301 redirect) against the Plan 02 dispatcher, reached through a new `#[doc(hidden)] pub mod __test_internals` wrapper that bridges integration-test binaries to the crate-private `serve_inner`.**

## Performance

- **Duration:** ~13 min
- **Started:** 2026-06-06T18:10:11Z
- **Completed:** 2026-06-06T18:22:48Z
- **Tasks:** 2 (shim + three integration tests)
- **Files created:** 4 (3 test files + this SUMMARY)
- **Files modified:** 1 (`ferro-bundle/src/lib.rs`)

## Accomplishments

- `ferro-bundle/src/lib.rs` gains a `#[doc(hidden)] pub mod __test_internals` containing a thin `pub fn serve_inner` wrapper that delegates to the crate-private `crate::serve_inner`. Integration test binaries (separate compilation units) can now dispatch directly without constructing a synthetic `Request` (RESEARCH OQ #3 resolution, finalized here).
- `tests/serve_cold.rs / serve_cold_returns_200_with_cache_headers` registers a bundle, asserts the hashed URL shape (`/bundles/serve-cold-sdk.<sha8>.js`), dispatches via `serve_inner(&hashed, None)`, and asserts status == 200, `Content-Type: application/javascript`, `Cache-Control: public, max-age=31536000, immutable`, quoted 66-char ETag (RFC 7232 §2.3), and body bytes equal to the registered slice.
- `tests/serve_304.rs / serve_304_on_if_none_match_exact` registers a bundle, captures the ETag from a cold dispatch, then dispatches again with `If-None-Match` == that exact quoted ETag, and asserts status == 304, ETag round-trip, `Cache-Control` present (RFC 7232 §4.1), and body is empty.
- `tests/alias_redirect.rs / alias_path_redirects_301_to_hashed_url` registers `Bundle::new(...).content_type(...).with_alias("/embed/v1.js")`, dispatches `serve_inner("/embed/v1.js", None)`, and asserts status == 301 and `Location` header == `bundle.hashed_url()`.
- ferro-bundle test count: 5 unit + 3 integration = **8 total**, all passing.
- Full workspace `cargo test --all-features` passes with no regressions.

## Task Commits

Each task was committed atomically:

1. **Task 1: `__test_internals` shim** — `1b18624f` (feat)
2. **Task 2: three integration tests** — `45f2dedd` (test)

## Files Created/Modified

- `ferro-bundle/src/lib.rs` (+23 lines) — new `#[doc(hidden)] pub mod __test_internals` between the dispatcher and the `#[cfg(test)] reset()` helper.
- `ferro-bundle/tests/serve_cold.rs` (62 lines) — BUNDLE-02 cold-path test.
- `ferro-bundle/tests/serve_304.rs` (54 lines) — BUNDLE-02 304 fast-path test.
- `ferro-bundle/tests/alias_redirect.rs` (35 lines) — BUNDLE-03 alias 301 test.
- `.planning/phases/183-ferro-bundle-capability-new-crate/183-03-SUMMARY.md` — this file.

## Decisions Made

- **Wrapper, not `pub use`.** Plan 03's action block specified `pub use crate::serve_inner;` inside `__test_internals`. Rust rejected it (E0364: re-export visibility cannot exceed imported item's visibility — `serve_inner` is `pub(crate)`). Resolution: a thin `#[inline] pub fn serve_inner(path: &str, if_none_match: Option<&str>) -> HttpResponse` wrapper that delegates to `crate::serve_inner`. Semantically equivalent, satisfies the plan's stated intent ("reachability from integration tests without polluting the public API"), and the inline directive ensures zero overhead.
- **`#[doc(hidden)]` on the module, not the function.** The module-level attribute is sufficient — `cargo doc -p ferro-bundle --no-deps` emits no warnings and the module does not appear on the crate landing page. The `__test_internals` name (leading underscore) is the secondary social-convention deterrent.
- **Bundle names globally unique across test binaries.** Even though cargo runs each test binary in its own process (D-13 / process-level isolation), names are unique per file (`serve-cold-sdk`, `serve-304-sdk`, `alias-redirect-sdk`) so the duplicate-name panic guard from D-06 stays defensive.
- **No `reset()` call from integration tests.** Each `tests/*.rs` file is its own binary; the process-global registries start empty for each. The crate-private `reset()` helper remains for in-binary unit tests only.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `pub use crate::serve_inner` rejected by E0364**

- **Found during:** Task 1 build verification (`cargo build -p ferro-bundle`).
- **Issue:** The plan's action block specified `pub use crate::serve_inner;` inside `#[doc(hidden)] pub mod __test_internals`. Rust rejected with `error[E0364]: 'serve_inner' is private` — `pub use` cannot raise visibility above the source item's `pub(crate)`.
- **Fix:** Replaced the `pub use` with a thin `pub fn serve_inner(path: &str, if_none_match: Option<&str>) -> HttpResponse` wrapper marked `#[inline]`, delegating to `crate::serve_inner`. Updated the rustdoc on the module to explain why the wrapper is used instead of a `pub use`, and removed the broken intra-doc link `` [`serve_inner`] `` so `cargo doc` emits no warnings.
- **Files modified:** `ferro-bundle/src/lib.rs`.
- **Verification:** `cargo build -p ferro-bundle` exits 0; `cargo doc -p ferro-bundle --no-deps` emits no warnings; integration tests compile and pass.
- **Committed in:** `1b18624f`.

**2. [Rule 2 - Missing Critical] `clippy::uninlined_format_args` in `serve_cold.rs`**

- **Found during:** Task 2 commit gate (`cargo clippy --all --all-targets --all-features -- -D warnings`).
- **Issue:** The plan's verbatim test source used `"unexpected hashed URL: {}", hashed` and similar trailing-arg `format!` shapes. CLAUDE.md's `-D warnings` clippy gate fails on `clippy::uninlined_format_args` (4 sites in `serve_cold.rs`).
- **Fix:** Converted all four sites to inline format args: `"unexpected hashed URL: {hashed}"`, `"expected .js extension; got {hashed}"`, `"ETag must be quoted per RFC 7232 §2.3; got {etag}"`, `"ETag length unexpected: {etag}"`. `serve_304.rs` and `alias_redirect.rs` were already clean (no trailing-arg format strings).
- **Files modified:** `ferro-bundle/tests/serve_cold.rs`.
- **Verification:** `cargo clippy --all --all-targets --all-features -- -D warnings` exits 0; `cargo fmt --all -- --check` exits 0.
- **Committed in:** `45f2dedd` (same as the test bodies — the clippy fix was applied before the test commit).

**3. [Rule 1 - Bug] rustfmt rewrapping after Write**

- **Found during:** Task 2 commit gate (`cargo fmt --all -- --check`).
- **Issue:** The plan's verbatim test source wrote single-line `assert!(hashed.ends_with(".js"), "expected .js extension; got {}", hashed);` and `assert_eq!(header_value(resp.headers(), "Location"), Some(hashed.as_str()));` which exceeded rustfmt's default line width once expanded.
- **Fix:** Ran `cargo fmt --all`. Two files (`serve_cold.rs` and `alias_redirect.rs`) were reformatted to the multi-line style.
- **Files modified:** `ferro-bundle/tests/serve_cold.rs`, `ferro-bundle/tests/alias_redirect.rs` (whitespace + line breaks only).
- **Verification:** `cargo fmt --all -- --check` exits 0.
- **Committed in:** `45f2dedd`.

---

**Total deviations:** 3 (1 Rule 3 blocking — visibility-rule discovery during build; 1 Rule 2 lint conformance; 1 Rule 1 fmt). No scope change. All three fixes preserve plan semantics exactly.

**Impact on plan:** The `__test_internals` shim now uses a `pub fn` wrapper instead of a `pub use`. The plan's hand-off prediction in 183-02-SUMMARY's "Hand-off to Plan 03" section anticipated reachability via "pub(crate) ... public-by-test surface" — which Rust does not actually permit. The wrapper closes this gap with zero observable effect on the consumer API or on test behavior.

## Issues Encountered

- Beyond the three auto-fixed deviations above, none. Integration tests compiled and passed on first run after the shim build was clean.

## User Setup Required

None.

## Verification Evidence

```
$ cargo test -p ferro-bundle --test serve_cold --test serve_304 --test alias_redirect
     Running tests/alias_redirect.rs
running 1 test
test alias_path_redirects_301_to_hashed_url ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/serve_304.rs
running 1 test
test serve_304_on_if_none_match_exact ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/serve_cold.rs
running 1 test
test serve_cold_returns_200_with_cache_headers ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p ferro-bundle
     Running unittests src/lib.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
     Running tests/alias_redirect.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
     Running tests/serve_304.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
     Running tests/serve_cold.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo fmt --all -- --check
(clean)

$ cargo clippy --all --all-targets --all-features -- -D warnings
(clean — full workspace)

$ cargo test --all-features
(all crate test suites pass; no regressions)

$ cargo doc -p ferro-bundle --no-deps 2>&1 | grep -E 'warning|error'
(no output — doc build clean; __test_internals does not appear on crate landing page)
```

## TDD Gate Compliance

This plan's Task 2 was marked `tdd="true"` in the PLAN.md. The cycle reduces to a single RED-then-GREEN motion: the three new test files act as RED at the moment of `Write` (they cannot compile yet because Task 1 was already in place and the integration tests reach the shim), and they transition to GREEN immediately because the Plan 02 dispatcher already implements the behavior under test. There is no separate failing-test commit, because the implementation is pre-existing and Task 1 (shim) is the only edit needed to make the tests reachable.

- **RED equivalent:** the test files were authored to assert the BUNDLE-02 / BUNDLE-03 contracts; they would fail against the pre-Plan-02 stub because no dispatcher existed.
- **GREEN gate:** commit `45f2dedd test(183-03): add BUNDLE-02 + BUNDLE-03 integration tests` — all three tests pass on first run; full workspace `cargo test --all-features` clean.
- **REFACTOR gate:** not used — tests are at target shape; no separate cleanup commit warranted.

The plan-level `type` is `execute` (not `tdd`), so no plan-level RED gate commit is required; the Task 2-level `tdd="true"` is collapsed into the GREEN commit because the implementation is already shipped.

## Hand-off to Plan 04

Plan 04 (`publish bootstrap`) is unblocked. Concretely, Plan 04 needs:

- Final README polish — the bundle-vs-filesystem split documentation is already drafted (Plan 01) but Plan 04 may want to add a "Usage" snippet showing the canonical `Bundle::new(...).content_type(...).with_alias(...)` chain plus a handler wiring example.
- `cargo publish -p ferro-bundle --dry-run` gate — confirms the crate metadata and dependency graph are valid for crates.io publication.
- Local-terminal bootstrap of the first `cargo publish -p ferro-bundle` (D-12: CI token cannot create new crates).

## Self-Check

- `ferro-bundle/tests/serve_cold.rs` exists — verified (62 lines).
- `ferro-bundle/tests/serve_304.rs` exists — verified (54 lines).
- `ferro-bundle/tests/alias_redirect.rs` exists — verified (35 lines).
- `ferro-bundle/src/lib.rs` contains `#[doc(hidden)] pub mod __test_internals` — verified.
- `ferro-bundle/src/lib.rs` contains a `pub fn serve_inner` wrapper delegating to `crate::serve_inner` inside `__test_internals` — verified.
- `serve_inner` retains its `pub(crate)` visibility at the crate root — verified.
- Commit `1b18624f feat(183-03): expose serve_inner via __test_internals shim` present — verified.
- Commit `45f2dedd test(183-03): add BUNDLE-02 + BUNDLE-03 integration tests` present — verified.
- `cargo test -p ferro-bundle` reports 5 unit + 3 integration = 8 tests total, all passing — verified.
- `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features` all exit 0 — verified.

## Self-Check: PASSED

## Next Phase Readiness

`ferro-bundle` is functionally complete: hash-determinism + registration semantics from Plan 02 unit tests, cold-200 / 304 / alias-301 behavior verified by Plan 03 integration tests. Plan 04 only needs the publish dry-run gate and the local-terminal first publish; no further code changes are required for v0.2.43 of the crate.

---
*Phase: 183-ferro-bundle-capability-new-crate*
*Plan: 03-integration-tests*
*Completed: 2026-06-06*
