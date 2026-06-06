---
phase: 183-ferro-bundle-capability-new-crate
verified: 2026-06-06T20:30:00Z
status: human_needed
score: 6/6 must-haves verified (5/6 fully automated + 1/6 PASSED with documented human action)
overrides_applied: 0
re_verification: false
human_verification:
  - test: "Manual first-publish bootstrap to crates.io"
    expected: "User runs `cargo publish -p ferro-bundle` from local terminal at workspace root; crates.io listing returns HTTP 200 for version 0.2.43 within ~60 seconds"
    why_human: "Per project memory `project_ferro_publish_token_scoping.md`, the CI publish token has `publish-update` scope only — only the maintainer's local CARGO_REGISTRY_TOKEN has `publish-new` scope required to create the crate. Explicitly deferred by user per Plan 04 Task 2 (checkpoint:human-action); SUMMARY records `skipped` disposition."
    deferred: true
    reopen_condition: "When Phase 182 + 183 are on master AND `ferro-rs 0.2.42`+ is on crates.io, user runs `cd /Users/alberto/repositories/albertogferrario/ferro && cargo publish -p ferro-bundle`"
---

# Phase 183: ferro-bundle capability (new crate) — Verification Report

**Phase Goal:** Ship a new top-level crate `ferro-bundle` for in-memory immutable byte blobs registered at boot, with content-hashed URLs, one-year immutable caching, 304 fast-path, and 301 alias redirects.

**Verified:** 2026-06-06T20:30:00Z
**Status:** human_needed (5/6 fully automated; BUNDLE-06 publish bootstrap deferred to user per explicit Plan 04 stance)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (Success Criteria BUNDLE-01..06)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | BUNDLE-01: `Bundle::new("embed-v1", BYTES).content_type("application/javascript").hashed_url()` returns `/bundles/embed-v1.{8hex}.js` deterministically from SHA-256 | VERIFIED | Unit test `hash_is_deterministic` pins `/bundles/test1.2cf24dba.txt` for `b"hello"` (SHA-256 first 8 chars). `cargo test -p ferro-bundle --lib hash_is_deterministic` passes. Code uses `Sha256::digest(bytes)` at lib.rs:138 + `hex::encode(digest)` at lib.rs:139 + `[..8].to_string()` at lib.rs:140 |
| 2 | BUNDLE-02: `Bundle::serve(req)` returns 200 + Cache-Control + ETag on cold; 304 on If-None-Match exact match | VERIFIED | Integration tests `serve_cold_returns_200_with_cache_headers` (asserts status 200, Content-Type, `Cache-Control: public, max-age=31536000, immutable`, ETag quoted 66 chars, body == registered bytes) and `serve_304_on_if_none_match_exact` (asserts status 304, ETag round-trip, Cache-Control round-trip, empty body) both pass. Code at lib.rs:238 quotes ETag per RFC 7232 §2.3; lib.rs:244 emits Cache-Control on 304 per RFC 7232 §4.1 |
| 3 | BUNDLE-03: `.with_alias("/embed/v1.js")` registers a 301 redirect to current hashed URL | VERIFIED | Integration test `alias_path_redirects_301_to_hashed_url` asserts status 301 + `Location` header == `bundle.hashed_url()`. Code at lib.rs:230-234 implements alias-first dispatch (D-08) using inline `HttpResponse::new().status(301).header("Location", target)` pattern |
| 4 | BUNDLE-04: Content-type caller-provided; default `application/octet-stream` if unspecified | VERIFIED | Unit test `default_content_type_is_octet_stream` asserts `Bundle::new("test2", b"x")` produces `/bundles/test2.{8hex}` (no extension because octet-stream maps to `""`). lib.rs:141 sets default `"application/octet-stream"`; lib.rs:104 `_ => ""` returns empty extension for unknown types. `duplicate_name_panics` test (lib.rs:332-338) verifies D-06 panic-on-duplicate |
| 5 | BUNDLE-05: README documents bundle-vs-filesystem split | VERIFIED | `grep -F 'do not fold' ferro-bundle/README.md` returns 1 line containing the phrase twice (D-10 load-bearing wording). README §"Bundle vs filesystem static files" (line 35) contains a comparison table contrasting freshness models; the bolded "Do not fold these — they target different freshness models." (line 44) plus parenthetical "(do not fold these paths into one)" double-pin the architectural assertion |
| 6 | BUNDLE-06: Publishes to crates.io via existing GH Actions workflow | VERIFIED (split: CI wiring complete; manual bootstrap deferred to user) | **CI-wiring side (Plan 01):** `grep -F 'ferro-bundle' .github/workflows/publish.yml` returns 1 match at line 302 under `WAVE3_CRATES="ferro-cli ferro-bundle"` (Wave 3 = framework-consumers, correctly post-Wave-2 `ferro-rs`). Step name at line 297: `Publish Wave 3 (framework-consumers)`. **Bootstrap side (Plan 04):** Plan 04 Task 1 verified `cargo publish -p ferro-bundle --dry-run` exit 0 (SUMMARY records `Packaged 9 files, 128.1KiB compressed`); Task 2 explicitly skipped per user directive (user does not want to publish now; Phase 182 + 183 not yet on master). Bootstrap = single user action documented with prerequisites + recovery procedure; counted as human-verification item |

**Score:** 6/6 truths verified (BUNDLE-06 split: automated CI wiring complete; manual bootstrap is the one human-verification item)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-bundle/Cargo.toml` | Crate manifest with locked deps (sha2 0.10, hex 0.4, dashmap 6, bytes 1, thiserror 2, ferro-rs path+version) | VERIFIED | 24 lines; matches CONTEXT.md D-09 verbatim. `name = "ferro-bundle"`, `version.workspace = true`, `description` is project-agnostic (no tenant identity), `ferro-rs = { path = "../framework", version = "0.2" }` includes the version required for crates.io publish-time resolution |
| `ferro-bundle/src/lib.rs` | Full public API + registries + dispatcher + tests | VERIFIED | 354 lines (target was ~300; Plan 02 produced 331 + Plan 03's `__test_internals` shim added 23 lines). Contains all required symbols: `pub struct Bundle` (line 121), `pub enum Error` with `NotFound`+`DuplicateName` variants (lines 46-51), `pub fn new/content_type/with_alias/hashed_url/serve` (lines 133-220), `pub(crate) fn serve_inner` (line 228), `pub mod __test_internals` (line 272), `static BUNDLE_REGISTRY/ALIAS_REGISTRY/NAME_INDEX: OnceLock<DashMap<...>>` (lines 69-73), `#[cfg(test)] reset()` (line 288). `Bytes::from_static(entry.bytes)` zero-copy at line 247 |
| `ferro-bundle/README.md` | Bundle-vs-filesystem split section with "do not fold" wording | VERIFIED | 53 lines, neutral architectural voice. Contains §Features (line 7), §Usage with code example (line 14), §Bundle vs filesystem static files (line 35) with comparison table (lines 39-42), bolded D-10 assertion (line 44), §Security note (line 46), §License (line 50). No trigger phrases (no "killer feature", "load-bearing", "we bet on", etc. — only neutral technical description) |
| `ferro-bundle/tests/serve_cold.rs` | BUNDLE-02 cold path test | VERIFIED | 62 lines; `fn serve_cold_returns_200_with_cache_headers` passes; imports `ferro_bundle::__test_internals::serve_inner`; asserts status 200, Content-Type, Cache-Control verbatim, ETag quoted 66 chars, body bytes match |
| `ferro-bundle/tests/serve_304.rs` | BUNDLE-02 304 fast-path test | VERIFIED | 55 lines; `fn serve_304_on_if_none_match_exact` passes; round-trips ETag from cold to conditional; asserts 304 + ETag + Cache-Control + empty body |
| `ferro-bundle/tests/alias_redirect.rs` | BUNDLE-03 alias 301 test | VERIFIED | 37 lines; `fn alias_path_redirects_301_to_hashed_url` passes; asserts 301 + Location header equals hashed URL |
| `Cargo.toml` (workspace root) | Workspace registers ferro-bundle, version 0.2.43 | VERIFIED | Line 30: `"ferro-bundle",` (last entry in members list). Line 34: `version = "0.2.43"` (bumped from 0.2.42 per Plan 01 Task 2) |
| `Cargo.lock` | Synced to 0.2.43 across all workspace crates | VERIFIED | 26 entries match `^version = "0.2.43"` (every workspace crate); 0 entries match `^version = "0.2.42"`; `name = "ferro-bundle"` present with `version = "0.2.43"` |
| `.github/workflows/publish.yml` | Wave 3 publishes ferro-bundle alongside ferro-cli | VERIFIED | Line 297: `Publish Wave 3 (framework-consumers)` step name. Line 302: `WAVE3_CRATES="ferro-cli ferro-bundle"`. For-loop structure (lines 304-) mirrors Wave 2 verbatim. NOT in Wave 1A (line 211) or Wave 1B (line 246) — correct per RESEARCH §critical correction since `ferro-rs` publishes in Wave 2 (line 274) and `ferro-bundle` depends on it |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| Cargo.toml [workspace] members | ferro-bundle/Cargo.toml | workspace member registration | WIRED | Line 30 `"ferro-bundle",` present; `cargo build --workspace` succeeds |
| ferro-bundle/Cargo.toml [dependencies] | framework/Cargo.toml (crate name ferro-rs) | path + version dep | WIRED | `ferro-rs = { path = "../framework", version = "0.2" }` at line 18 — `version = "0.2"` is mandatory for crates.io publish-time resolution (verified by Plan 04 dry-run success) |
| .github/workflows/publish.yml Wave 3 | ferro-bundle | post-framework publish wave | WIRED | Line 302 `WAVE3_CRATES="ferro-cli ferro-bundle"` |
| Bundle::new | BUNDLE_REGISTRY (OnceLock<DashMap<...>>) | eager insertion keyed by hashed URL | WIRED | lib.rs:155 `bundle_registry().insert(hashed_url.clone(), entry);` |
| Bundle::with_alias | ALIAS_REGISTRY | alias_path → hashed_url mapping | WIRED | lib.rs:196 `alias_registry().insert(alias_path.to_string(), target);` |
| Bundle::serve | serve_inner(path, if_none_match) | thin wrapper extracting Request path + If-None-Match | WIRED | lib.rs:216-220 (4-line wrapper); test bypass via `__test_internals::serve_inner` (lib.rs:272-280) confirmed working in 3 integration tests |
| serve_inner | ALIAS_REGISTRY then BUNDLE_REGISTRY | alias-first ordering (D-08) | WIRED | lib.rs:230 `if let Some(target) = alias_registry().get(path)` precedes lib.rs:237 `if let Some(entry) = bundle_registry().get(path)` |

All key links wired. Dispatch order matches D-03: alias → bundle → 404 fallback.

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|---------------------|--------|
| `serve_inner` HTTP responses | `entry.bytes` (200 body), `entry.sha256_full_hex` (ETag), `entry.content_type` (header), `target.value()` (Location header) | `BUNDLE_REGISTRY` + `ALIAS_REGISTRY` (populated by `Bundle::new` / `.with_alias`) | Yes — each integration test asserts non-empty body bytes (cold), non-empty ETag with 66-char quoted form (cold + 304), non-empty Location header equal to hashed URL (alias) | FLOWING |
| `Bundle::hashed_url()` return | `name_index().get(&self.name).value().clone()` | `NAME_INDEX` (populated by `Bundle::new` and re-keyed by `.content_type`) | Yes — assert in `hash_is_deterministic` test pins exact string `/bundles/test1.2cf24dba.txt`; integration tests' `bundle.hashed_url()` calls also produce non-empty deterministic strings | FLOWING |

No HOLLOW or STATIC artifacts. The registry → response path is real data end-to-end.

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 5 unit tests pass | `cargo test -p ferro-bundle --lib` | `test result: ok. 5 passed; 0 failed; 0 ignored` | PASS |
| All 3 integration tests pass | `cargo test -p ferro-bundle --test serve_cold --test serve_304 --test alias_redirect` | Each binary: `test result: ok. 1 passed; 0 failed; 0 ignored` (3 total) | PASS |
| Crate-private symbols exported correctly to tests | `cargo build -p ferro-bundle` + integration tests using `__test_internals::serve_inner` compile and run | All 8 tests pass; no E0364 visibility errors | PASS |
| `ferro-bundle` is in workspace members and Cargo.lock | `grep -F '"ferro-bundle",' Cargo.toml && grep -F 'name = "ferro-bundle"' Cargo.lock` | Both grep matches found; Cargo.lock entry has `version = "0.2.43"` | PASS |
| Wave 3 publish entry correct | `grep -F 'WAVE3_CRATES="ferro-cli ferro-bundle"' .github/workflows/publish.yml` | 1 match at line 302 | PASS |
| D-10 README assertion present | `grep -F 'do not fold' ferro-bundle/README.md` | 1 line returned containing the phrase (both bolded sentence and parenthetical on same line) | PASS |
| Cargo.lock fully synced (no drift) | `grep -c '^version = "0.2.42"' Cargo.lock` | 0 (zero stale 0.2.42 entries); 26 entries at 0.2.43 | PASS |

All 7 spot-checks pass.

---

### Requirements Coverage

REQUIREMENTS.md does not enumerate Phase 183 (per RESEARCH §phase_requirements — "Phase 183 is not enumerated in `.planning/REQUIREMENTS.md` (that document scopes the v12.1 AI milestone, REQ-IDs `AISDK-*` / `AISSE-*` / `AICLI-*`)"). The phase uses informal IDs BUNDLE-01..06 aligned 1:1 with ROADMAP §1981-1987 success criteria.

| Requirement | Source Plan(s) | Description | Status | Evidence |
|-------------|----------------|-------------|--------|----------|
| BUNDLE-01 | 02 | Deterministic SHA-256 → 8-hex URL handle | SATISFIED | Unit test `hash_is_deterministic` pins canonical SHA-256 output |
| BUNDLE-02 | 03 | 200 cold + 304 fast-path on If-None-Match | SATISFIED | Two integration tests verify both branches of the cache-validator code path |
| BUNDLE-03 | 03 | Alias path 301 redirect | SATISFIED | Integration test verifies status 301 + Location header equality with hashed URL |
| BUNDLE-04 | 02 | Default Content-Type octet-stream | SATISFIED | Unit test verifies URL has no extension when content_type unset; default content-type assigned at lib.rs:141 |
| BUNDLE-05 | 01 | README documents split | SATISFIED | D-10 wording present; comparison table comparing freshness models present |
| BUNDLE-06 | 01 (CI) + 04 (bootstrap) | Publishes via existing GH Actions workflow | PARTIALLY SATISFIED — CI wiring done; first publish deferred to user (NEEDS HUMAN) | `WAVE3_CRATES` entry present + dry-run exit 0; real `cargo publish -p ferro-bundle` deferred per user directive (Plan 04 SUMMARY records `skipped`) |

---

### Anti-Patterns Found

Scanned `ferro-bundle/src/lib.rs` and all integration tests for: TODO/FIXME/PLACEHOLDER, empty implementations, hardcoded empty data, console.log-only logic, gestiscilo/tenant identity leakage.

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none found) | — | — | — | — |

All scans clean:
- `grep -i -E "gestiscilo|jetski|adriatic" ferro-bundle/{src/lib.rs,README.md,Cargo.toml,tests/*.rs}` → exit 1 (no matches). Project-agnostic crate rule honored per CLAUDE.md.
- `grep -n -E "TODO|FIXME|XXX|HACK|PLACEHOLDER|unimplemented|todo!" ferro-bundle/{src/lib.rs,tests/*.rs}` → exit 1 (no matches).
- No empty `return null` / `return [].` / `=> {}` patterns. The `match` returns at lib.rs:171, 177 (`return self`) are documented unreachable defensive guards after `Bundle::new()` panics on missing entries, not stubs.
- No trigger phrases in README. The bolded "Do not fold these — they target different freshness models." is technical architectural language describing a design split, not internal-strategy voice.

---

### Locked Decisions (D-01..D-13) Honored

| Decision | Honored | Evidence |
|----------|---------|----------|
| D-01: `&'static [u8]` storage | YES | `Bundle::new(name: &str, bytes: &'static [u8])` at lib.rs:133 |
| D-02: `OnceLock<DashMap<…>>` registry | YES | lib.rs:69-73 (`BUNDLE_REGISTRY`, `ALIAS_REGISTRY`, `NAME_INDEX` all `OnceLock<DashMap<...>>`) |
| D-03: Alias-first dispatch in serve_inner | YES | lib.rs:230 alias check precedes lib.rs:237 bundle check; lib.rs:254 404 fallback last |
| D-04: 8 hex chars from SHA-256 | YES | lib.rs:140 `&sha256_full_hex[..8]` |
| D-05: Strong ETag full SHA-256, quoted | YES | lib.rs:238 `format!("\"{}\"", entry.sha256_full_hex)` quotes the full 64-hex digest per RFC 7232 §2.3 |
| D-06: Eager registration, panic on duplicate name | YES | lib.rs:134-135 panics if `name_index().contains_key(name)`; unit test `duplicate_name_panics` pins the behavior with `#[should_panic(expected = "duplicate")]` |
| D-07: SHA-256 via `sha2` crate | YES | lib.rs:138 `Sha256::digest(bytes)` |
| D-08: Alias mechanism queried by serve, 301 redirect | YES | lib.rs:196 `alias_registry().insert(alias_path, target)` + lib.rs:231-233 `HttpResponse::new().status(301).header("Location", ...)` |
| D-09: Minimal crate dependencies | YES | Cargo.toml lists exactly 6 deps: sha2, hex, dashmap, bytes, thiserror, ferro-rs |
| D-10: README "do not fold" phrase present | YES | README.md line 44 (bolded) + parenthetical |
| D-11: Workspace + publish.yml integration | YES | Plus the Wave correction documented (Wave 3 instead of CONTEXT.md's incorrect Wave 1B; verified Wave 3 is correct since ferro-rs is in Wave 2) |
| D-12: Manual bootstrap correctly deferred | YES | Plan 04 Task 2 `checkpoint:human-action`; SUMMARY records `skipped` per user directive; Claude executor did NOT attempt real publish |
| D-13: `#[cfg(test)] reset()` helper exists | YES | lib.rs:287-298 |

All 13 locked decisions honored without override.

---

### Pre-commit Gate Compliance

CLAUDE.md mandates `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` before every commit.

- Plan 01 Task 3: gate green at scaffold level (ferro-bundle scoped); pre-existing ferro-json-ui fmt drift logged to `deferred-items.md` per scope-boundary rule; later fixed in commit `0fcc7be3`
- Plan 02 GREEN commit `472daa77`: full workspace gate green (clippy `--all-targets --all-features`, fmt, tests)
- Plan 03 commit `45f2dedd`: full workspace gate green
- Plan 04 Task 1: full workspace gate green; `cargo publish -p ferro-bundle --dry-run` exit 0 (resolved against `ferro-rs 0.2.41` from crates.io, the latest published 0.2.x; caret requirement `"0.2"` matches)

Verified at HEAD (commit `2dbfe5dd`): `cargo test -p ferro-bundle` returns 5 unit + 3 integration + 1 doc-test-ignored = 8 results, all passing.

---

### Project Rules Compliance (CLAUDE.md)

| Rule | Status | Evidence |
|------|--------|----------|
| No co-author lines in commits | VERIFIED | `git log b145fbaa..HEAD` grep for "co-author" returns no matches |
| Project-agnostic crate (no tenant identity in src/ or README) | VERIFIED | grep for gestiscilo/jetski/adriatic in `ferro-bundle/{src/lib.rs,README.md,Cargo.toml,tests/*.rs}` returns exit 1 (no matches). Planning files reference gestiscilo as the discovery context, which is acceptable per CLAUDE.md (planning artifacts can reference downstream consumers; crate source/README cannot) |
| New crate added to publish.yml | VERIFIED | Wave 3 entry confirmed |
| `version.workspace = true` inheritance | VERIFIED | ferro-bundle/Cargo.toml lines 3-5 |
| Repository documents read as neutral | VERIFIED | README uses architectural voice, no trigger phrases. The phrase "load-bearing design assertion" appears once in CONTEXT.md (planning, not repository public docs) describing a class of decisions; not in the crate README |

---

### Human Verification Required

#### 1. Manual first-publish bootstrap to crates.io (BUNDLE-06 bootstrap side)

**Test:** From local terminal at workspace root:

```bash
cd /Users/alberto/repositories/albertogferrario/ferro
cargo publish -p ferro-bundle
```

**Expected:** Output ends with `Published ferro-bundle v0.2.43 at registry crates-io`. Then `curl -sI https://crates.io/api/v1/crates/ferro-bundle | grep -E '^HTTP/'` returns `HTTP/2 200` within ~60 seconds.

**Why human:** Per project memory `project_ferro_publish_token_scoping.md`, the CI publish token has `publish-update` scope only — it cannot create new crates on crates.io. Only the maintainer's local `CARGO_REGISTRY_TOKEN` has `publish-new`. This is an architectural property of the project's token management, not a limitation of the verifier.

**Status:** Explicitly deferred by user. Plan 04 Task 2 (`checkpoint:human-action`) executed with `skipped` disposition per user directive ("Phase 182 + 183 are not yet on master / pushed; do not publish to crates.io now"). Plan 04 SUMMARY documents prerequisites, expected output, and four named failure-mode recoveries.

**Reopen condition:** When Phase 182 + 183 are on master AND `ferro-rs 0.2.42`+ is on crates.io (so the standalone-packaged ferro-bundle source can resolve `ferro-rs = "0.2"` against a fresh-enough version), the user runs the documented command.

**Phase impact:** Phase 183's code is complete and shippable. Only the crates.io publication step is intentionally outstanding. Every BUNDLE-01..05 success criterion is fully verified; BUNDLE-06's CI-wiring side is verified (Wave 3 entry + dry-run exit 0); only the bootstrap side awaits the single user command.

---

### Gaps Summary

No actionable gaps found. All 6 BUNDLE-01..06 success criteria are met or explicitly deferred to a documented user action:

- **BUNDLE-01..05 (5 of 6):** Fully verified via automated tests + grep checks against the codebase.
- **BUNDLE-06 (1 of 6):** Split into two halves. CI-wiring half (Wave 3 entry + dry-run gate) is automated and verified. First-publish bootstrap half is the single remaining user-only action; it is durably documented in `183-04-SUMMARY.md` with prerequisites, expected output, verification command, and four named failure-mode recovery paths. The defer is intentional and architecturally correct (CI token scope), not an oversight.

All 13 locked decisions (D-01..D-13) are honored. Pre-commit gate (fmt + clippy + tests) green at every plan commit. Project-agnostic crate rule honored. No co-author lines added. Project memory `feedback_friction_loop_release_cadence.md` honored — single publish at end of phase (still deferred but flagged in the SUMMARY as the one outstanding action).

The phase is **code-complete and shippable**. The status is `human_needed` solely because Phase 183 cannot mark BUNDLE-06's bootstrap side as fully closed until the user runs `cargo publish -p ferro-bundle` from local terminal. This is a single one-shot action, not a sequence of changes, and the deferral is the user's explicit choice per `feedback_friction_loop_release_cadence.md` (publish ONCE at end of friction loop, after gestiscilo Phase 185 confirms consumption).

---

_Verified: 2026-06-06T20:30:00Z_
_Verifier: Claude (gsd-verifier)_
