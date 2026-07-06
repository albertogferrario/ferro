---
phase: 187-ferro-assets-asset-pipeline-composer
plan: "02"
subsystem: ferro-assets
tags: [new-crate, asset-pipeline, html-minify, css-minify, js-minify, inject-before-tag, replace-tokens, lol_html, lightningcss, swc, tdd, wave-2]
dependency_graph:
  requires:
    - 187-01 (Asset/ContentType/Transform/Pipeline/Error API)
  provides:
    - HtmlMinify transform (lol_html, opaque script/style, SC-2 proof)
    - CssMinify transform (lightningcss =1.0.0-alpha.71)
    - JsMinify transform (swc 66.0.0 Compiler::minify)
    - InjectBeforeTag transform (lol_html on_end_tag insertion)
    - ReplaceTokens transform (raw bytes, all content types)
    - SC-2 inline-script regression fixture (tests/inline_script_fixture.rs)
  affects:
    - ferro-assets/src/transforms/ (5 new transform modules)
    - ferro-assets/Cargo.toml (swc_common = "23" added)
    - ferro-assets/tests/ (inline_script_fixture.rs + fixtures)
tech_stack:
  added:
    - swc_common = "23" (direct dep for SourceMap/GLOBALS/FileName, not re-exported by swc)
  patterns:
    - lol_html element handler without text handler = opaque content (C-02 prevention)
    - end_tag! + el.on_end_tag() for insertion before closing tags
    - swc GLOBALS.set + try_with_handler + Compiler::minify high-level API
    - raw byte linear-scan find-and-replace (no regex, no eval)
    - map_matching gates HTML/CSS/JS transforms; ReplaceTokens iterates all assets
key_files:
  created:
    - ferro-assets/src/transforms/html_minify.rs
    - ferro-assets/src/transforms/css_minify.rs
    - ferro-assets/src/transforms/js_minify.rs
    - ferro-assets/src/transforms/inject_before_tag.rs
    - ferro-assets/src/transforms/replace_tokens.rs
    - ferro-assets/tests/inline_script_fixture.rs
    - ferro-assets/tests/fixtures/inline_script.html
    - ferro-assets/tests/fixtures/inline_script_expected_script.txt
    - ferro-assets/tests/fixtures/inline_script_expected_style.txt
  modified:
    - ferro-assets/src/transforms/mod.rs (added 5 module declarations + pub use)
    - ferro-assets/Cargo.toml (added swc_common = "23")
decisions:
  - "swc_common = '23' added as direct dep: swc crate uses swc_common internally but does not pub re-export SourceMap/GLOBALS/FileName; version 23.0.1 resolved from Cargo.lock"
  - "lol_html element! handler without text! = opaque content: text handler on script/style would corrupt template literals and JSON blobs (C-02)"
  - "end_tag! + el.on_end_tag() pattern for inject_before_tag: inserts immediately before the matching element's closing tag, verified against lol_html 2.9.0 docs"
  - "closing_tag_to_selector validates tag name chars (alphanumeric + hyphen only): prevents '<<//>' or empty name from becoming invalid lol_html selectors"
  - "ReplaceTokens intentionally omits map_matching: applies to ALL ContentType variants so tokens work in JSON, JS, HTML attributes, and text"
  - "HtmlMinify text collapse uses text!('body *') with replace(' ', LolContentType::Text): conservative whitespace collapse, only in visible body elements"
metrics:
  duration: "769s (~13 min)"
  completed: "2026-06-07T21:19:38Z"
  tasks: 3
  files: 11
---

# Phase 187 Plan 02: Text Transforms — HtmlMinify, CssMinify, JsMinify, InjectBeforeTag, ReplaceTokens

Five text transforms implemented and tested. The regression-critical `HtmlMinify` opaque-content guarantee is proved by the SC-2 fixture: template literals, `${}` interpolations, multi-line strings, and a JSON blob inside `<script>` survive `html_minify` byte-identical.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | HtmlMinify + SC-2 inline-script regression fixture | cc45266d | html_minify.rs, mod.rs, inline_script_fixture.rs, 3 fixture files |
| 2 | CssMinify (lightningcss) + JsMinify (swc Compiler::minify) | e73c7657 | css_minify.rs, js_minify.rs, mod.rs, Cargo.toml |
| 3 | InjectBeforeTag (lol_html) + ReplaceTokens (raw bytes) | 9c05bc75 | inject_before_tag.rs, replace_tokens.rs, mod.rs |

## Acceptance Criteria Status

- [x] `grep -q 'element!("script"' ferro-assets/src/transforms/html_minify.rs`
- [x] `! grep -q 'text!("script"' ferro-assets/src/transforms/html_minify.rs` (NO text handler on script)
- [x] `! grep -q 'text!("style"' ferro-assets/src/transforms/html_minify.rs` (NO text handler on style)
- [x] `! grep -qE '\.unwrap\(\)' ferro-assets/src/transforms/html_minify.rs`
- [x] fixture files exist: `test -f ferro-assets/tests/fixtures/inline_script.html`
- [x] `cargo test -p ferro-assets --test inline_script_fixture` exits 0 (SC-2 byte-correct)
- [x] `grep -q 'StyleSheet::parse' ferro-assets/src/transforms/css_minify.rs`
- [x] `grep -q 'Compiler::new' ferro-assets/src/transforms/js_minify.rs`
- [x] `! grep -qE '\.unwrap\(\)' ferro-assets/src/transforms/css_minify.rs`
- [x] `! grep -qE '\.unwrap\(\)' ferro-assets/src/transforms/js_minify.rs`
- [x] `grep -q 'lightningcss = "=1.0.0-alpha.71"' ferro-assets/Cargo.toml` (exact pin intact)
- [x] `grep -q 'pub struct InjectBeforeTag' ferro-assets/src/transforms/inject_before_tag.rs`
- [x] `grep -q 'pub struct ReplaceTokens' ferro-assets/src/transforms/replace_tokens.rs`
- [x] `map_matching` not called in replace_tokens.rs code (applies to all content types)
- [x] `! grep -qE '\.unwrap\(\)' ferro-assets/src/transforms/inject_before_tag.rs`
- [x] `cargo test -p ferro-assets` green (39 tests pass)
- [x] `cargo clippy -p ferro-assets --all-targets -- -D warnings` clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] lol_html text!() replace() requires ContentType argument**
- **Found during:** Task 1 compile
- **Issue:** `t.replace(" ")` — lol_html 2.9.0 `TextChunk::replace` takes two arguments: content string + `ContentType`
- **Fix:** Added `use lol_html::html_content::ContentType as LolContentType` and passed `LolContentType::Text` as second argument
- **Files modified:** ferro-assets/src/transforms/html_minify.rs
- **Commit:** cc45266d

**2. [Rule 2 - Missing functionality] swc_common not re-exported by swc**
- **Found during:** Task 2 implementation research
- **Issue:** RESEARCH.md Pattern 4 uses `use swc_common::{SourceMap, GLOBALS, FileName}` but `swc` crate does not `pub use` these — they are internal imports. Direct dep required.
- **Fix:** Added `swc_common = "23"` to ferro-assets/Cargo.toml (version 23.0.1 from Cargo.lock)
- **Files modified:** ferro-assets/Cargo.toml
- **Commit:** e73c7657

**3. [Rule 1 - Bug] closing_tag_to_selector returned Some("/") for `"<//>"` input**
- **Found during:** Task 3 test run
- **Issue:** `"<//>"` strips `</` → `/>`, strips `>` → `/`, `"/"` is non-empty so returned `Some("/")`
- **Fix:** Added char validation: name must contain only ASCII alphanumeric or hyphen chars
- **Files modified:** ferro-assets/src/transforms/inject_before_tag.rs
- **Commit:** 9c05bc75

## Known Stubs

None. All five transforms are fully implemented and wired. The `transforms/mod.rs` TODO comments from Plan 01 are now replaced with actual module declarations.

`ferro-assets/src/transforms/mod.rs` still has no `image_transcode` or `responsive_images` — those are Plan 03 scope. The passthrough_proof.rs integration test already references those transforms; Plan 03 will resolve that stub.

## Threat Flags

None. No new network endpoints, auth paths, or trust boundaries introduced. All STRIDE mitigations from the plan's threat model are implemented:

| Threat ID | Mitigation Status |
|-----------|-------------------|
| T-187-05 | mitigated: StyleSheet::parse + try_with_handler both return Result; no .unwrap() on any parse path |
| T-187-06 | mitigated: element! handler only on script/style, NO text! handler; SC-2 fixture proves byte-correct bodies |
| T-187-07 | accepted: ReplaceTokens rustdoc documents caller responsibility for sanitizing token values |
| T-187-08 | mitigated: rewriter.write/end return Result, mapped to Error::transform; no .unwrap() |

## Self-Check: PASSED

Files exist:
- ferro-assets/src/transforms/html_minify.rs ✓
- ferro-assets/src/transforms/css_minify.rs ✓
- ferro-assets/src/transforms/js_minify.rs ✓
- ferro-assets/src/transforms/inject_before_tag.rs ✓
- ferro-assets/src/transforms/replace_tokens.rs ✓
- ferro-assets/tests/inline_script_fixture.rs ✓
- ferro-assets/tests/fixtures/inline_script.html ✓
- ferro-assets/tests/fixtures/inline_script_expected_script.txt ✓
- ferro-assets/tests/fixtures/inline_script_expected_style.txt ✓

Commits exist:
- cc45266d ✓
- e73c7657 ✓
- 9c05bc75 ✓
