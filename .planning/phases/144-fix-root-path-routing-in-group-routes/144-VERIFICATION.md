---
phase: 144-fix-root-path-routing-in-group-routes
verified: 2026-04-21T22:00:00Z
status: passed
score: 13/13
overrides_applied: 0
---

# Phase 144: Fix Root Path Routing in Group Routes — Verification Report

**Phase Goal:** Fix root path ("/") routing in `group!`/`Router::group` so `group!("/api", { get!("/", h) })` reaches `h` at BOTH `/api` and `/api/`; middleware runs for both variants; no double-slash regressions; no duplicate RouteInfo entries.

**Verified:** 2026-04-21T22:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `group!("/prefix", { get!("/", h) })` reaches h at both `/prefix` and `/prefix/` (D-01) | VERIFIED | `macros.rs` `register_with_inherited` calls `combine_group_path` which returns `(canonical, Some(alternate))` for non-root prefixes; both leaves inserted via `insert_get` + `insert_get_alias`; test `group_root_handler_matches_both_variants` asserts both match |
| 2 | `group!("/", { get!("/", h) })` registers exactly one `/` (no `//`) (D-02) | VERIFIED | `combine_group_path("/", "/")` returns `("/", None)` per the `stripped.is_empty()` branch; tested in `root_prefix_root_handler_is_single_slash` |
| 3 | `group!("/api/", { get!("/x", h) })` produces `/api/x`, not `/api//x` (D-03) | VERIFIED | `combine_group_path` strips one trailing `/` from prefix via `strip_suffix('/')` before concatenation; test `trailing_slash_prefix_is_stripped` confirms `/api//x` returns None |
| 4 | Non-root routes inside groups retain current behavior — no unwanted alternate (D-04) | VERIFIED | `combine_group_path` returns `(combined, None)` for non-`"/"` route paths; test `non_root_prefix_non_root_path_unchanged` asserts `/api-d04/users/` does not match |
| 5 | Fix applies to BOTH `macros.rs` and `group.rs` implementations (D-05) | VERIFIED | Both files import `use super::path::combine_group_path`; both call the helper in their registration paths; parallel test matrices pass |
| 6 | Nested groups recursively apply the same rules (D-06) | VERIFIED | Edit B in `macros.rs` strips trailing slash from `parent_prefix` before concatenating `self.prefix`; test `nested_group_root_matches_both_variants` covers both clean and trailing-slash outer prefix cases |
| 7 | `RouteInfo` via `get_registered_routes()` contains ONE entry per logical handler (D-07) | VERIFIED | Alias methods (`insert_get_alias`, etc.) skip `register_route`; no call to `register_route` in any alias method body; integration test `no_duplicate_route_info` asserts delta == 1 and path filter count == 1 |
| 8 | Named-route lookup via `route_url` returns canonical path (D-08) | VERIFIED | `register_route_name(name, canonical_path)` called once; test `named_route_resolves_to_canonical` asserts `route("home_canonical_test", &[])` returns `"/api"` not `"/api/"` |
| 9 | Table-driven tests cover the full 8-row D-09 matrix | VERIFIED | `path.rs` `combine_group_path_matrix` test covers all 8 rows exactly as specified; all pass |
| 10 | Integration test asserts no duplicate RouteInfo entries (D-10) | VERIFIED | `routing_group_trailing_slash.rs` `no_duplicate_route_info` + `no_duplicate_route_info_multi_handler_group` both pass with delta and path-filter assertions |
| 11 | Both `group.rs` and `macros.rs` have equivalent mirrored tests (D-11) | VERIFIED | `group.rs` has 6-test inline `mod tests` mirroring the Plan 02 macro matrix; D-11 parity table documented in 03-SUMMARY.md |
| 12 | Workspace version bumped to 0.2.13 (D-12) | VERIFIED | `Cargo.toml` line 27: `version = "0.2.13"` |
| 13 | CHANGELOG entry present in neutral voice, no downstream project named (D-13 as overridden by CLAUDE.md neutral-voice rule) | VERIFIED | `CHANGELOG.md` has `## ferro-rs` + `### [0.2.13] — 2026-04-21` entry; `grep gestiscilo CHANGELOG.md` returns no matches; "production field application" phrasing confirmed |

**Score:** 13/13 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `framework/src/routing/path.rs` | `pub(crate) fn combine_group_path` + 8-row matrix test | VERIFIED | File exists, function present, test `combine_group_path_matrix` covers all 8 D-09 rows |
| `framework/src/routing/mod.rs` | `mod path;` declaration | VERIFIED | Line 3: `mod path;` present in alphabetical order |
| `framework/src/routing/router.rs` | 5 `pub(crate) insert_{method}_alias` methods | VERIFIED | All 5 methods exist; none call `register_route`; each stores `canonical_path.to_string()` as matchit value |
| `framework/src/routing/macros.rs` | `GroupDef::register_with_inherited` using helper + alias inserts + 7 new tests | VERIFIED | `use super::path::combine_group_path` imported; `combine_group_path(&full_prefix, &converted_route_path)` called; canonical+alias inserts for all 5 HTTP methods; 7 new tests in `mod tests` |
| `framework/src/routing/group.rs` | `GroupBuilder::finalize` using helper + alias inserts + 6-test module | VERIFIED | `use super::path::combine_group_path` imported; `combine_group_path(&self.prefix, &route.path)` called; alias inserts for all 4 GroupMethod variants; `#[cfg(test)] mod tests` with 6 tests |
| `framework/tests/routing_group_trailing_slash.rs` | Integration test file with 5 serial tests | VERIFIED | File exists; 5 tests present; all `#[serial]`-guarded; `extern crate ferro_rs` at top |
| `CHANGELOG.md` | `## ferro-rs` + `### [0.2.13]` entry | VERIFIED | Entry present; `## ferro-rs` before `## ferro-stripe`; neutral voice confirmed |
| `Cargo.toml` | `version = "0.2.13"` | VERIFIED | Line 27 confirmed |
| `docs/src/the-basics/routing.md` | New `### Root routes inside a group` subsection | VERIFIED | Subsection present between `## Route Groups` and `## Named Routes` |
| `docs/src/the-basics/middleware.md` | Invariant note before `## Middleware Execution Order` | VERIFIED | "applies uniformly to root-path routes inside the group" paragraph present |
| `framework/src/routing/macros.rs` rustdoc | Updated `# Path Combination` block | VERIFIED | New three-rule description present; old "full path is just the group prefix" wording removed |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `framework/src/routing/macros.rs` | `framework/src/routing/path.rs` | `use super::path::combine_group_path` | WIRED | Import present; function called at `combine_group_path(&full_prefix, &converted_route_path)` |
| `framework/src/routing/macros.rs` | `framework/src/routing/router.rs` | `router.insert_{method}_alias(alt, handler, canonical)` | WIRED | All 5 alias calls present in HTTP method match arms |
| `framework/src/routing/group.rs` | `framework/src/routing/path.rs` | `use super::path::combine_group_path` | WIRED | Import present; called at `combine_group_path(&self.prefix, &route.path)` |
| `framework/src/routing/group.rs` | `framework/src/routing/router.rs` | `self.outer_router.insert_{method}_alias(alt, handler, &canonical)` | WIRED | All 4 alias calls present for GET/POST/PUT/DELETE GroupMethod variants |
| `framework/src/routing/mod.rs` | `framework/src/routing/path.rs` | `mod path;` | WIRED | Declaration present; sibling modules can use `super::path::combine_group_path` |
| `framework/tests/routing_group_trailing_slash.rs` | ferro_rs crate public API | `extern crate ferro_rs as ferro` | WIRED | File uses `ferro_rs::{get, get_registered_routes, group, ...}` |

---

### Data-Flow Trace (Level 4)

Not applicable. Phase 144 is a routing bug fix — no new data-fetching components, no dynamic data rendering. All changed code operates at startup-time route registration, not request-time data flow.

---

### Behavioral Spot-Checks

| Behavior | Verification Method | Result | Status |
|----------|-------------------|--------|--------|
| `combine_group_path` 8-row matrix | Read `path.rs` test code: all 8 cases match D-09 spec exactly | All rows match spec | PASS |
| No `register_route` in alias methods | Read `router.rs` lines 269–345: none of the 5 alias method bodies contain `register_route` | Confirmed absent | PASS |
| Middleware key is canonical in alias leaf | Read `router.rs` alias methods: value tuple is `(handler, canonical_path.to_string())` | Confirmed | PASS |
| Old bug site removed from `macros.rs` | `grep "full_prefix.clone()" macros.rs` — no matches | Bug site gone | PASS |
| Old bug site removed from `group.rs` | `grep 'format!("{}{}", self.prefix, route.path)' group.rs` — no matches | Bug site gone | PASS |
| Strategy A middleware in `group.rs` | `add_middleware(&canonical, mw.clone())` — only canonical key used | Confirmed (2 call sites, both use `&canonical`) | PASS |
| Strategy A middleware in `macros.rs` | `add_middleware(canonical_path, mw.clone())` — only canonical key used | Confirmed (2 call sites, both use `canonical_path`) | PASS |
| No "gestiscilo" in CHANGELOG | Grep CHANGELOG.md | 0 matches | PASS |
| Version 0.2.13 in Cargo.toml | Read Cargo.toml line 27 | `version = "0.2.13"` | PASS |
| Integration test has 5 serial tests | Read routing_group_trailing_slash.rs | 5 `#[serial]` test functions present | PASS |

Full test suite (`cargo test --all-features`) was not run locally due to disk-space constraints documented in 144-05-SUMMARY.md. Targeted test gates were run by the executor and all passed: `cargo test -p ferro-rs --lib --features json-ui routing::` (22/22), `cargo test -p ferro-rs --test routing_group_trailing_slash --features json-ui` (5/5), `cargo fmt --all -- --check` (clean), `cargo clippy -p ferro-rs --all-targets --features json-ui -- -D warnings` (0 warnings).

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| D-01 | 144-02 | `get!("/", h)` inside non-root group reaches handler at both `/prefix` and `/prefix/` | SATISFIED | `combine_group_path` returns alternate; alias inserted via `insert_get_alias`; test `group_root_handler_matches_both_variants` passes |
| D-02 | 144-01 | Root-in-root `group!("/", { get!("/", h) })` yields single `/` | SATISFIED | `stripped.is_empty()` branch returns `("/", None)`; test `root_prefix_root_handler_is_single_slash` passes |
| D-03 | 144-01 | Trailing slash on group prefix stripped before concatenation | SATISFIED | `strip_suffix('/')` in `combine_group_path` and in `macros.rs` parent-prefix accumulation |
| D-04 | 144-01 | Non-root route paths concatenate normally, no alternate | SATISFIED | Non-`"/"` route_path branch returns `(combined, None)`; test `non_root_prefix_non_root_path_unchanged` confirms |
| D-05 | 144-03 | Fix applies to both `macros.rs` and `group.rs` | SATISFIED | Both files import and call `combine_group_path`; parallel test suites pass |
| D-06 | 144-02 | Nested groups follow same rules recursively | SATISFIED | `parent_prefix.strip_suffix('/')` in Edit B; test `nested_group_root_matches_both_variants` covers both cases |
| D-07 | 144-02 | `RouteInfo` has ONE entry at canonical path | SATISFIED | Alias methods skip `register_route`; integration test `no_duplicate_route_info` asserts delta==1 |
| D-08 | 144-02 | Named-route lookup returns canonical path | SATISFIED | `register_route_name(name, canonical_path)` called once; test `named_route_resolves_to_canonical` asserts `"/api"` |
| D-09 | 144-01 | Table-driven tests cover full 8-row matrix | SATISFIED | `combine_group_path_matrix` test in `path.rs` covers all 8 rows |
| D-10 | 144-04 | Integration test asserts no duplicate RouteInfo | SATISFIED | `no_duplicate_route_info` and `no_duplicate_route_info_multi_handler_group` in integration test file |
| D-11 | 144-03 | Both `group.rs` and `macros.rs` have equivalent tests | SATISFIED | D-11 parity table in 03-SUMMARY.md; 6-test module in `group.rs` mirrors `macros.rs` matrix |
| D-12 | 144-05 | Patch release 0.2.12 → 0.2.13 | SATISFIED | `Cargo.toml` workspace version is `"0.2.13"` |
| D-13 | 144-05 | CHANGELOG entry names source (neutral voice per CLAUDE.md override) | SATISFIED | Entry present; `gestiscilo` not mentioned; "production field application" phrasing used |

---

### Anti-Patterns Found

No blockers or warnings found:

- No TODO/FIXME/HACK/PLACEHOLDER comments in changed files
- No empty implementations or `return null` stubs
- No hardcoded empty data in routing logic
- The temporary `#[allow(dead_code)]` added in Plan 01 was removed in Plan 02 (confirmed by SUMMARY-02: "Also removed `#[allow(dead_code)]` from `combine_group_path` in `path.rs`")
- `Box::leak` usage in `macros.rs` follows the existing codebase convention for `'static` matchit paths (startup-time, bounded, not a leak in the pathological sense)

---

### Human Verification Required

None. All behaviors are verifiable programmatically. The gestiscilo-it production field-test upgrade (bump downstream project to 0.2.13 and verify `https://…/s/amaris-experience/` renders) is explicitly deferred to a post-release check outside the scope of this phase per VALIDATION.md.

---

## Gaps Summary

No gaps. All 13 must-have truths verified against the actual codebase.

---

_Verified: 2026-04-21T22:00:00Z_
_Verifier: Claude (gsd-verifier)_
