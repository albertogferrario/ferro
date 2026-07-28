---
phase: 261-asset-ergonomics
verified: 2026-07-26T00:00:00Z
status: passed
score: 10/10
overrides_applied: 0
human_verification:
  - test: "Run `ferro assets fetch iconify lucide/home` in a scratch directory"
    expected: "A file `assets/lucide/home.svg` is created containing a valid SVG document fetched over HTTPS from api.iconify.design"
    why_human: "The offline tests use a tempdir for path/layout verification only. The live network path (actual HTTPS GET to api.iconify.design over rustls) cannot be exercised without running the built binary against the real network — not safe to trigger in automated CI."
  - test: "Run `ferro assets fetch fontsource inter` in the same or a different scratch directory"
    expected: "A file `assets/inter/latin-400-normal.woff2` is created containing valid WOFF2 font bytes fetched over HTTPS from cdn.fontsource.com"
    why_human: "Same reason as above. The offline woff2_dest/write tests confirm the file-output layout and validate_woff2_url confirms the SSRF guard, but the actual download over rustls must be confirmed manually."
---

# Phase 261: `asset!()` Ergonomics — Verification Report

**Phase Goal:** Collapse the boot-time bundle builder to a one-line `asset!("path")` at the use site, and give an opt-in author-time fetch for Iconify sets and Fontsource families, all flowing through the existing content-hashed pipeline with no new infrastructure.
**Verified:** 2026-07-26T00:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `asset!("path")` expands to `include_bytes!` + lazy `OnceLock` registration returning `&'static str` | VERIFIED | `ferro-macros/src/asset.rs` contains `include_bytes!(#path_lit)`, `static OnceLock<String>`, `.get_or_init(...)`, `.as_str()`. Registered as `#[proc_macro] pub fn asset` in `ferro-macros/src/lib.rs`. Trybuild pass fixture compiles `let _url: &'static str = ferro::asset!("fixture.js")` cleanly. |
| 2 | The returned URL is content-hashed and stable across evaluations for unchanged bytes | VERIFIED | `OnceLock::get_or_init` guarantees exactly-once registration per call site. `Bundle::new` hashes bytes via SHA-256, stores the short hex in the URL. `hash_is_deterministic` test in `ferro-bundle/src/lib.rs` confirms the hash is stable. |
| 3 | Content-type is inferred from the path extension via `ferro::bundle::mime_from_ext` | VERIFIED | `ferro-macros/src/asset.rs` line 26–29 extracts the extension, lowercases it, passes it to `#ferro::bundle::mime_from_ext(#ext)` in the expansion. `test_mime_from_ext` covers all 16 known extensions; `mime_from_ext_unknown_is_octet_stream` covers the passthrough guarantee. |
| 4 | Unrecognized extensions pass through byte-identical (`application/octet-stream`) | VERIFIED | `mime_from_ext` wildcard arm returns `"application/octet-stream"`. Unit test `mime_from_ext_unknown_is_octet_stream` asserts `mime_from_ext("xyz")` and `mime_from_ext("")` both return `"application/octet-stream"`. |
| 5 | `ferro::bundle::Bundle`, `ferro::bundle::mime_from_ext`, `ferro::bundle::BundleResponse` resolve from the framework crate | VERIFIED | `framework/src/bundle.rs` line 23: `pub use ferro_bundle::{mime_from_ext, Bundle, BundleResponse};`. `framework/src/lib.rs` line 19: `pub mod bundle;`. |
| 6 | `ferro::bundle::serve(&req)` returns an `HttpResponse` equivalent to pre-decouple behavior (200/301/304/404 + headers + body) | VERIFIED | `framework/src/bundle.rs` line 33: `pub fn serve(req: &Request) -> HttpResponse` (free function — correct; E0116 prevents inherent impl on foreign type; plan explicitly allowed this). `framework/tests/bundle_serve.rs` has four async tests: `serve_200_cold_path`, `serve_304_conditional_get`, `serve_301_alias_redirect`, `serve_404_unknown_path` — all four paths covered. |
| 7 | `ferro::asset!` is re-exported from framework | VERIFIED | `framework/src/lib.rs` line 354: `pub use ferro_macros::asset;` (alphabetically placed after `action`, before `injectable` — correct per rustfmt sort order). |
| 8 | `ferro-bundle` is a leaf crate with no `ferro-rs` dependency | VERIFIED | `ferro-bundle/Cargo.toml` dependencies: `sha2`, `hex`, `dashmap`, `bytes`, `thiserror` only. `grep ferro-rs ferro-bundle/Cargo.toml` exits non-zero. Line 135 of `lib.rs` mentions "does NOT depend on ferro-rs" in a doc comment — not a code reference. |
| 9 | `ferro assets fetch iconify <set>` and `ferro assets fetch fontsource <family>` are wired into the CLI and write files under the output dir | VERIFIED (automated) | `ferro-cli/src/commands/assets.rs` has `AssetsCommand`, `FetchSource`, `fetch_iconify`, `fetch_fontsource`. `commands/mod.rs` line 4: `pub mod assets;`. `main.rs` line 545: `Assets { subcommand: commands::assets::AssetsCommand }`. Dispatch arm at line 827. Offline tempdir tests (`write_icon_lands_under_out_dir`, `woff2_dest_is_expected_shape`, etc.) pass. Live network download is human-only. |
| 10 | Set/family/icon names that would escape the fixed Iconify/Fontsource host or the output dir are rejected; woff2 URLs from API responses are validated against an HTTPS+host allowlist | VERIFIED | `validate_segment` rejects `.`, `:`, `%`, `/`, uppercase, empty strings. `validate_woff2_url` requires HTTPS and host in `{"cdn.fontsource.com", "api.fontsource.org"}`. `is_safe_svg_body` guards against `<script`, `<foreignObject`, `javascript:`, event handler attributes. All three are tested by unit tests in `assets.rs`. |

**Score:** 10/10 truths verified (live network download classified as human verification, not a gap)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-bundle/src/lib.rs` | `pub fn mime_from_ext` + `BundleResponse` + `serve_path` | VERIFIED | All three present. No `ferro_rs` import. `__test_internals` shim removed. Line count ~417. |
| `ferro-bundle/Cargo.toml` | No `ferro-rs` dependency | VERIFIED | Dependencies: `sha2`, `hex`, `dashmap`, `bytes`, `thiserror` only. |
| `.github/workflows/publish.yml` | `ferro-bundle` in `WAVE1A_CRATES`, removed from `WAVE3_CRATES` | VERIFIED | Line 217: `...ferro-assets ferro-bundle` (ends Wave 1a string). Line 335: `WAVE3_CRATES="ferro-cli"` (bundle absent). |
| `framework/src/bundle.rs` | `pub use ferro_bundle::` re-exports + `pub fn serve` adapter | VERIFIED | Line 23: `pub use ferro_bundle::{mime_from_ext, Bundle, BundleResponse};`. Line 33: `pub fn serve(req: &Request) -> HttpResponse`. |
| `framework/Cargo.toml` | `ferro-bundle` dependency | VERIFIED | Line 42: `ferro-bundle = { path = "../ferro-bundle", version = "0.2" }`. |
| `framework/src/lib.rs` | `pub mod bundle;` + `pub use ferro_macros::asset;` | VERIFIED | Line 19: `pub mod bundle;`. Line 354: `pub use ferro_macros::asset;`. |
| `framework/tests/bundle_serve.rs` | Four async tests proving 200/304/301/404 parity | VERIFIED | Four `#[tokio::test]` functions cover all four response paths. |
| `ferro-macros/src/asset.rs` | `asset_impl` with `include_bytes!` + `OnceLock` + `Bundle::new` + `mime_from_ext` | VERIFIED | All four patterns present. 59 lines (above 25-line minimum). Lowercase bundle name fix (WR-01) applied at line 37. |
| `ferro-macros/src/lib.rs` | `mod asset;` + `#[proc_macro] pub fn asset` | VERIFIED | Line 14: `mod asset;`. Lines 286–288: `#[proc_macro] pub fn asset(input: TokenStream) -> TokenStream { asset::asset_impl(input) }`. |
| `ferro-macros/tests/asset_macro.rs` | Trybuild harness with `t.pass` | VERIFIED | `fn asset_macro_ui()` with `t.pass("tests/ui/asset/pass/*.rs")`. |
| `ferro-macros/tests/ui/asset/pass/fixture.js` | Real embeddable file | VERIFIED | File exists. Contents: `console.log(1);` |
| `ferro-macros/tests/ui/asset/pass/minimal.rs` | Trybuild pass fixture binding to `&'static str` | VERIFIED | `let _url: &'static str = ferro::asset!("fixture.js");` — uses sibling `"fixture.js"` path (trybuild resolves relative to fixture file dir). |
| `ferro-cli/src/commands/assets.rs` | `AssetsCommand` + `FetchSource` + fetch impls + validation | VERIFIED | All present. `validate_segment`, `validate_woff2_url`, `is_safe_svg_body`, `write_icon`, `woff2_dest` all present with tests. |
| `ferro-cli/src/commands/mod.rs` | `pub mod assets;` | VERIFIED | Line 4. |
| `ferro-cli/src/main.rs` | `Commands::Assets` variant + dispatch arm | VERIFIED | Variant at line 545, dispatch at line 827. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-macros/src/asset.rs expansion` | `::ferro::bundle::Bundle` + `::ferro::bundle::mime_from_ext` | `crate::utils::ferro()` root-path helper | VERIFIED | `#ferro::bundle::Bundle::new(...)` and `#ferro::bundle::mime_from_ext(#ext)` in quote! expansion |
| `ferro-macros/tests/ui/asset/pass/minimal.rs` | `ferro::asset!` | trybuild pass fixture embeds `fixture.js` | VERIFIED | Fixture uses `ferro::asset!("fixture.js")` and `fixture.js` sibling is present |
| `framework/src/bundle.rs Bundle::serve` (now free fn `serve`) | `ferro_bundle::serve_path` | wraps `BundleResponse` into `HttpResponse` | VERIFIED | Line 36: `let resp = ferro_bundle::serve_path(&path, if_none_match.as_deref());` followed by status/headers/body reconstruction |
| `ferro-bundle/Cargo.toml` | (no ferro-rs) | dependency removed | VERIFIED | `grep ferro-rs ferro-bundle/Cargo.toml` exits non-zero |
| `ferro-cli/src/main.rs Commands::Assets` | `commands::assets::run` | match arm dispatch | VERIFIED | Lines 827–829 |
| `ferro-cli/src/commands/assets.rs fetch_iconify` | `https://api.iconify.design` | reqwest blocking GET over rustls | VERIFIED | Lines 132, 140 hardcode the HTTPS host |

### Data-Flow Trace (Level 4)

Not applicable — this phase delivers a proc-macro, a CLI binary, and a library crate. None of these are dynamic-data rendering components that need a Level 4 data-flow trace. The `OnceLock` registration is validated by the trybuild fixture (proves compile-time correctness) and the `hash_is_deterministic` unit test (proves runtime hash stability).

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `ferro-bundle` builds as leaf crate | `cargo build -p ferro-bundle` (evidence from 261-01 SUMMARY) | exit 0 | PASS (from executor) |
| `ferro-bundle` tests pass (mime + serve) | `cargo test -p ferro-bundle -- --test-threads=1` (evidence from 261-01 SUMMARY) | 10/10 pass | PASS (from executor) |
| `ferro-rs` (framework) builds with no cycle | `cargo build -p ferro-rs` (evidence from 261-02 SUMMARY) | exit 0 | PASS (from executor) |
| Framework bundle_serve tests pass | `cargo test -p ferro-rs --test bundle_serve -- --test-threads=1` (evidence from 261-02 SUMMARY) | 4/4 pass | PASS (from executor) |
| CLI assets validation tests pass (offline) | `cargo test -p ferro-cli --lib assets:: -- --test-threads=1` (evidence from 261-03 SUMMARY) | 7/7 pass | PASS (from executor) |
| Trybuild asset macro pass fixture compiles | `cargo test -p ferro-macros --test asset_macro -- --test-threads=1` (evidence from 261-04 CI gate) | pass | PASS (from executor) |
| Full CI-exact gate (fmt + clippy + test --all-features) | Run twice at phase end (evidence from 261-04 SUMMARY CI gate section) | exit 0, 0 warnings, 137 test suites, 0 failed | PASS (from executor) |
| Live network Iconify download | `ferro assets fetch iconify lucide/home` | — | ? HUMAN NEEDED |
| Live network Fontsource download | `ferro assets fetch fontsource inter` | — | ? HUMAN NEEDED |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| LIVE-03 | 261-01, 261-02, 261-03, 261-04 | One-line `asset!()` + opt-in author-time fetch for Iconify/Fontsource | SATISFIED | SC-1: `asset!()` macro fully wired. SC-2: `mime_from_ext` + `application/octet-stream` passthrough. SC-3: `ferro assets fetch` CLI wired with offline tests; live network is human-only. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `ferro-macros/tests/asset_macro.rs` | 11 | No `fail/` fixtures (IN-01 from review — informational, intentionally not fixed) | Info | Error paths of `asset!()` are unverified in trybuild; consistent with other macro tests in the project. Not a blocker. |
| `ferro-cli/src/commands/assets.rs` | 98–102 | `woff2_dest` is `pub` but doesn't call `validate_segment` internally (IN-02 from review — informational, intentionally not fixed) | Info | Production callers validate before invoking; function is `pub` for testability. Precondition should be documented. Not a blocker. |

No blockers or warnings found (all CR/WR findings from 261-REVIEW.md were fixed in commits `d6aca914`, `c059b02f`, `96d52739`, `4d5c6435`, `86a543bc`).

### Human Verification Required

#### 1. Live Iconify Network Download

**Test:** In a scratch directory, run: `ferro assets fetch iconify lucide/home`
**Expected:** File `assets/lucide/home.svg` is created. It contains a complete `<svg>` document fetched over HTTPS from `api.iconify.design`. No error is printed.
**Why human:** The offline tests verify path layout and SSRF guards but do not exercise the actual HTTPS connection. The live network path (reqwest blocking + rustls) must be confirmed manually. This also validates that the `lucide` set prefix and `home` icon name are currently valid on the Iconify API.

#### 2. Live Fontsource Network Download

**Test:** In a scratch directory, run: `ferro assets fetch fontsource inter`
**Expected:** File `assets/inter/latin-400-normal.woff2` is created. It contains valid WOFF2 font bytes. The file is fetched over HTTPS from `cdn.fontsource.com` (after the family metadata is fetched from `api.fontsource.org`). No error is printed.
**Why human:** Same reason as above. The `validate_woff2_url` unit tests confirm the SSRF allowlist is correct in code, but the actual two-step fetch (metadata then woff2 binary) must be exercised against the live CDN.

### Gaps Summary

No gaps. All 10 observable truths are VERIFIED. The two human verification items cover the live network download path of SC-3, which is by design unverifiable without running the built binary against real endpoints. The VALIDATION.md explicitly designates this as a "Manual-Only Verification". All automated checks (CI-exact gate, unit tests, trybuild, integration tests) passed.

---

_Verified: 2026-07-26T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
