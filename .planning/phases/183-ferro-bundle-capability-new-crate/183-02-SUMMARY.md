---
phase: 183-ferro-bundle-capability-new-crate
plan: 02
subsystem: infra
tags: [bundle, sha256, etag, cache, dashmap, oncelock, http]

# Dependency graph
requires:
  - phase: 183-ferro-bundle-capability-new-crate
    provides: "Plan 01 scaffold: ferro-bundle/Cargo.toml + ferro-bundle/src/lib.rs stub + README + workspace member + publish.yml Wave 3 entry"
provides:
  - "pub struct Bundle with new/content_type/with_alias/hashed_url/serve public API"
  - "pub(crate) fn serve_inner(path, if_none_match) -> HttpResponse dispatcher (RESEARCH OQ #3 resolution; Plan 03 integration tests call this directly)"
  - "Two process-global OnceLock<DashMap<...>> registries (BUNDLE_REGISTRY by hashed URL, ALIAS_REGISTRY by alias path) + NAME_INDEX secondary index"
  - "pub enum Error { NotFound, DuplicateName } via thiserror"
  - "#[cfg(test)] pub(crate) fn reset() registry clearer for test isolation (D-13)"
  - "5 passing unit tests covering BUNDLE-01 + BUNDLE-04 + D-06 + Error Display strings"
affects:
  - 183-03-plan (integration tests via serve_inner: 200/304 cold + alias 301)
  - 183-04-plan (README docs + publish bootstrap)

# Tech tracking
tech-stack:
  added: []  # No new workspace deps; all deps were declared in Plan 01's Cargo.toml
  patterns:
    - "OnceLock<DashMap<...>> process-global registry (mirrors ferro-json-ui plugin pattern with RwLock swapped for DashMap)"
    - "Eager registration on Bundle::new + remove+reinsert key rotation in .content_type"
    - "pub(crate) helper exposing the dispatcher to integration tests without synthetic Request construction"

key-files:
  created:
    - .planning/phases/183-ferro-bundle-capability-new-crate/183-02-SUMMARY.md
  modified:
    - ferro-bundle/src/lib.rs (8 lines -> 331 lines)

key-decisions:
  - "Add NAME_INDEX (OnceLock<DashMap<String,String>>) as a secondary name -> hashed_url index: lets .content_type and .hashed_url avoid O(n) registry scans. Plan source mentioned this as a workspace-introduced refinement on top of RESEARCH Example 1."
  - "ETag formatted as format!(\"\\\"{}\\\"\", entry.sha256_full_hex) for RFC 7232 §2.3 quoting; comparison against If-None-Match is exact string match."
  - "404 fallback in serve_inner sets Content-Type: text/plain but no body, since Bundle::serve is expected to receive only /bundles/... or registered alias paths."

patterns-established:
  - "Crate-visibility dispatcher: when an integration test needs to bypass a hyper-incoming-only Request constructor, expose the dispatch fn as pub(crate)."
  - "Builder remove+reinsert pattern for DashMap-keyed entries whose key derives from a mutable field (content_type → ext → hashed_url): the entry value is logically owned by the builder until the chain ends."

requirements-completed: [BUNDLE-01, BUNDLE-04]

# Metrics
duration: ~18min
completed: 2026-06-06
---

# Phase 183 Plan 02: Bundle core API Summary

**Bundle struct + two OnceLock<DashMap<...>> registries + pub(crate) serve_inner dispatcher implementing content-hashed URLs, ETag-quoted 304 fast-path, and 301 alias redirects; 5 unit tests pin SHA-256 determinism and registration semantics.**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-06-06T (plan start)
- **Completed:** 2026-06-06T (post-GREEN commit)
- **Tasks:** 1 (TDD: RED + GREEN)
- **Files modified:** 1 (ferro-bundle/src/lib.rs)

## Accomplishments

- `Bundle::new(name, bytes)` registers a SHA-256-keyed entry in `BUNDLE_REGISTRY` and a name → hashed_url mapping in `NAME_INDEX`; panics on duplicate name (D-06).
- `.content_type(ct)` re-keys the entry by removing it under the old URL and reinserting under `/bundles/{name}.{sha8}.{ext}` derived from a 13-entry content-type → extension table.
- `.with_alias(path)` inserts an `alias_path → current_hashed_url` mapping in `ALIAS_REGISTRY`.
- `.hashed_url()` returns the current URL via `NAME_INDEX` (O(1)).
- `Bundle::serve(req)` is a thin 4-line wrapper that extracts `req.path()` and `req.header("if-none-match")` and delegates to `serve_inner`.
- `pub(crate) fn serve_inner(path, if_none_match)` does alias-first (301), then bundle (304 on ETag match / 200 with cache headers), then 404 — D-03 ordering.
- 5 unit tests pass: hash determinism (`/bundles/test1.2cf24dba.txt` from `b"hello"`), default octet-stream (no extension), `#[should_panic(expected = "duplicate")]`, and per-variant `Error::to_string()`.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Failing unit tests for Bundle API** — `ba2a7ee2` (test)
2. **Task 1 GREEN: Bundle struct + serve_inner dispatcher** — `472daa77` (feat)

_Note: TDD task split across RED and GREEN commits per the plan's explicit instruction (action block: "Commit RED → GREEN in two separate commits")._

## Files Created/Modified

- `ferro-bundle/src/lib.rs` (8 → 331 lines) — Full public API + registries + dispatcher + tests. Plan 01's placeholder stub replaced with the working implementation.
- `.planning/phases/183-ferro-bundle-capability-new-crate/183-02-SUMMARY.md` — this file.

## Decisions Made

- **NAME_INDEX secondary index** — added a third `OnceLock<DashMap<String,String>>` for name → current-hashed-url. Plan source noted this as an O(1) replacement for the alternative O(n) registry scan in `hashed_url()`. The plan explicitly endorsed it ("workspace-introduced O(1) name → hashed_url index").
- **Default octet-stream URL has no extension** — `ext_from_content_type("application/octet-stream")` returns `""`, and `hashed_url_for` emits `/bundles/{name}.{sha8}` (no trailing dot). This matches BUNDLE-04 and the `default_content_type_is_octet_stream` test that asserts no `.txt`/`.js`/`.css` suffix.
- **Strong ETag full SHA-256** — `format!("\"{}\"", entry.sha256_full_hex)` produces the RFC 7232 §2.3 quoted strong tag (64-hex + 2 quotes = 66 chars). `If-None-Match` is compared by exact string equality including the quotes (Pitfall 3).
- **304 path emits Cache-Control** — per RFC 7232 §4.1, otherwise browsers re-revalidate immediately on every conditional request. The test in Plan 03 will pin this.

## OQ Resolutions Implemented (from RESEARCH)

- **OQ #1 (builder mutation order)** — `.content_type` does `bundle_registry().remove(old_url)` + mutate `entry.content_type` and `entry.ext` + `bundle_registry().insert(new_url, entry)` + `name_index().insert(name, new_url)`. Boot-time-only by contract; documented in the rustdoc on `.content_type`.
- **OQ #2 (alias tracking)** — aliases live in `ALIAS_REGISTRY` keyed by alias path; value is the current hashed URL. The rustdoc on `.with_alias` and the crate-level `# Builder order` section both state the required order: `Bundle::new → .content_type → .with_alias`.
- **OQ #3 (Request construction for tests)** — `pub(crate) fn serve_inner(path: &str, if_none_match: Option<&str>) -> HttpResponse` is the dispatcher. `Bundle::serve(req: Request)` is a 4-line wrapper around it. Plan 03 integration tests will call `serve_inner` directly, bypassing the hyper `Incoming` problem entirely.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Applied clippy `uninlined_format_args` fixes**
- **Found during:** Task 1 GREEN gate (`cargo clippy --all --all-targets --all-features -- -D warnings`)
- **Issue:** The plan's inlined source used `format!("/bundles/{}.{}", name, sha8)`, `panic!("...{:?}", name)`, and `assert!(..., "...{}", url)` patterns; CLAUDE.md's `-D warnings` gate fails on `clippy::uninlined_format_args` (3 sites in lib code + 3 in test code).
- **Fix:** Ran `cargo clippy --fix --allow-dirty -p ferro-bundle --lib --tests --all-features` (6 fixes applied); converted to inline `{name}` / `{sha8}` / `{ext}` / `{url}` / `{suffix}` format args.
- **Files modified:** `ferro-bundle/src/lib.rs` (6 format! / panic! / assert! sites).
- **Verification:** `cargo clippy --all --all-targets --all-features -- -D warnings` exits 0; `cargo fmt --all -- --check` exits 0; all 5 unit tests still pass.
- **Committed in:** `472daa77` (GREEN commit, applied before push).

**2. [Rule 1 - Bug] Re-applied `cargo fmt` after initial Write**
- **Found during:** Task 1 GREEN gate (`cargo fmt --all -- --check`)
- **Issue:** The plan's inlined source used a one-line `assert_eq!(e.to_string(), "duplicate bundle name: dup already registered");` and multi-arg `assert!(..., "expected /bundles/test2. prefix, got {}", url);`. After the clippy fixes inlined the format args, line lengths still tripped rustfmt's wrapping rules.
- **Fix:** Ran `cargo fmt -p ferro-bundle`.
- **Files modified:** `ferro-bundle/src/lib.rs` (whitespace only).
- **Verification:** `cargo fmt --all -- --check` exits 0.
- **Committed in:** `472daa77` (same GREEN commit).

---

**Total deviations:** 2 auto-fixed (both lint/format conformance, no scope change)
**Impact on plan:** Zero. Both fixes preserve plan semantics exactly; they only adjust how `format!` / `assert!` arguments are spelled to satisfy the CLAUDE.md `-D warnings` clippy gate the plan's action block explicitly required to pass.

## Issues Encountered

- None beyond the lint-conformance polish listed above.

## User Setup Required

None — no external service configuration.

## Verification Evidence

```
$ cargo test -p ferro-bundle --lib
running 5 tests
test tests::error_duplicate_name_displays_message ... ok
test tests::error_not_found_displays_message ... ok
test tests::default_content_type_is_octet_stream ... ok
test tests::hash_is_deterministic ... ok
test tests::duplicate_name_panics - should panic ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo fmt --all -- --check
(clean)

$ cargo clippy --all --all-targets --all-features -- -D warnings
(clean — full workspace)

$ cargo test --all-features
(all crate test suites pass; no regressions)
```

## TDD Gate Compliance

- **RED gate:** `ba2a7ee2 test(183-02): add failing unit tests for Bundle API (RED)` — confirmed build fails with `Bundle`, `Error`, `reset` undefined.
- **GREEN gate:** `472daa77 feat(183-02): implement Bundle core API + serve_inner dispatcher (GREEN)` — all 5 unit tests pass.
- **REFACTOR gate:** Not used — the GREEN code is already at target shape per the plan's verbatim source; no separate cleanup commit was warranted.

## Hand-off to Plan 03

Plan 03 layers integration tests on top of `serve_inner`:
- `ferro-bundle/tests/serve_cold.rs` — BUNDLE-02 cold path: 200 + Content-Type + Cache-Control + quoted ETag.
- `ferro-bundle/tests/serve_304.rs` — BUNDLE-02 fast path: 304 + ETag + Cache-Control on `If-None-Match` exact match.
- `ferro-bundle/tests/alias_redirect.rs` — BUNDLE-03 alias path: 301 + Location header to hashed URL.

All three tests call `ferro_bundle::serve_inner(path, if_none_match)` directly (the `pub(crate)` visibility means integration tests in `tests/` can reach it via the same crate's public-by-test surface; Plan 03's setup will mirror `ferro-bundle/src/lib.rs::reset()` access pattern). No synthetic `Request` is required.

## Self-Check

- `ferro-bundle/src/lib.rs` exists (331 lines) — verified.
- `pub struct Bundle` present at line 121 — verified.
- `pub enum Error` present at line 46 with `NotFound(String)` and `DuplicateName(String)` — verified.
- `pub(crate) fn serve_inner(path: &str, if_none_match: Option<&str>) -> HttpResponse` present at line 228 — verified.
- `static BUNDLE_REGISTRY: OnceLock<DashMap<String, BundleEntry>>` at line 69 — verified.
- `static ALIAS_REGISTRY: OnceLock<DashMap<String, String>>` at line 70 — verified.
- `Bytes::from_static(entry.bytes)` at line 247 — verified zero-copy per Pitfall 7.
- `format!("\"{}\", entry.sha256_full_hex)` shape at line 238 (`format!("\"{}\", ...)`) — verified ETag quoting.
- Commits `ba2a7ee2` (RED) and `472daa77` (GREEN) present in `git log` — verified.

## Self-Check: PASSED

## Next Phase Readiness

Plan 03 unblocked. The `pub(crate) serve_inner` dispatcher is the single integration-test entry point and is fully implemented. Plan 03 needs only to write three small integration test files (one per scenario), each calling `reset()` + `Bundle::new(...).content_type(...).[with_alias(...)]` + `serve_inner(path, inm)` and asserting on the returned `HttpResponse`.

---
*Phase: 183-ferro-bundle-capability-new-crate*
*Plan: 02-core-impl*
*Completed: 2026-06-06*
