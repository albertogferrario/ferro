---
phase: 143
plan: "03"
subsystem: ferro-json-ui, framework
tags: [ferro-json-ui, framework, config, routing, head-injection, tests]
dependency_graph:
  requires: [143-01, 143-02]
  provides: [static-css-pipeline-wired, stylesheet-urls-config, ferro-base-css-route]
  affects: [framework, ferro-json-ui]
tech_stack:
  added: []
  patterns: [html-escape-defense-in-depth, stylesheet-link-injection, static-bytes-from-static]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/config.rs
    - framework/src/json_ui/mod.rs
    - framework/src/server.rs
decisions:
  - "html_body() test helper added to outer tests module to get raw HTML strings (response_body uses Debug format)"
  - "html_escape promoted from #[cfg(test)] to production scope — needed for href attribute escaping"
  - "Cache-Control: public, max-age=86400 per plan RESEARCH.md Open Question 3 resolution"
metrics:
  duration_minutes: 35
  completed_date: "2026-04-20"
  tasks_completed: 4
  tasks_total: 4
  files_created: 0
  files_modified: 3
---

# Phase 143 Plan 03: Static CSS Pipeline Wiring Summary

End-to-end wiring of the static CSS pipeline: `JsonUiConfig` gains `stylesheet_urls`, `tailwind_cdn` default flips to `false`, head assembly emits `<link>` tags with HTML-escaped URLs, theme injection uses plain `<style>`, and `/_ferro/ferro-base.css` is registered as a static route serving the embedded CSS.

## Tasks Completed

| Task | Name | Commit | Status |
|------|------|--------|--------|
| 1 | Update JsonUiConfig — add stylesheet_urls, flip tailwind_cdn default, add builder | 8e9a16fb | Done |
| 2 | Update framework head assembly — emit stylesheet_urls links, plain style theme injection | 19d756a4 | Done |
| 3 | Register /_ferro/ferro-base.css route in server.rs dispatch | 5acd0d74 | Done |
| 4 | Full workspace validation — fmt, clippy, tests | 01b0464f | Done |

## What Was Built

### ferro-json-ui/src/config.rs

`JsonUiConfig` now has:
- `pub stylesheet_urls: Vec<String>` field (after `tailwind_cdn`)
- `Default` implementation: `tailwind_cdn: false`, `stylesheet_urls: vec!["/_ferro/ferro-base.css".to_string()]`
- `stylesheet_urls(mut self, urls: Vec<String>) -> Self` consuming builder
- Updated doctest showing new defaults
- Four unit tests: `default_has_tailwind_cdn_false_and_default_stylesheet_urls`, `stylesheet_urls_builder_replaces_entire_list`, `stylesheet_urls_builder_accepts_empty_vec`, `json_schema_derives_with_new_field`

### framework/src/json_ui/mod.rs

- `html_escape` promoted from `#[cfg(test)]` to production scope (needed by head assembly for href escaping)
- `build_response` iterates `config.stylesheet_urls`, emitting `<link rel="stylesheet" href="{html_escape(url)}">` per entry before the optional CDN script
- Theme injection changed: `<style type="text/tailwindcss">` → plain `<style>` (standard CSS vars work without Tailwind runtime)
- Three existing theme tests updated: fixtures changed from `@theme { ... }` to `:root { ... }`, assertions check for plain `<style>` tag
- Five new tests added (using `html_body()` for raw HTML assertions): default link emission, CDN coexistence with link ordering, custom urls replacing default, empty urls, URL HTML-escaping in href
- `html_body()` helper added: extracts raw `&str` from `HttpResponse` (vs `response_body` which uses Debug format)

### framework/src/server.rs

- `"/_ferro/ferro-base.css" => serve_ferro_base_css()` arm added to `/_ferro/*` dispatch block
- `serve_ferro_base_css()` function: 200 OK, `Content-Type: text/css; charset=utf-8`, `Cache-Control: public, max-age=86400`, `Content-Length`, `Bytes::from_static(FERRO_BASE_CSS.as_bytes())` (zero-copy)
- Two integration tests in `mod ferro_base_css_route_tests`: headers/status assertion, body equality assertion

## Test Results

All 6 new tests pass, all 3 updated theme tests pass:

| Test | Status |
|------|--------|
| `default_has_tailwind_cdn_false_and_default_stylesheet_urls` | PASS |
| `stylesheet_urls_builder_replaces_entire_list` | PASS |
| `stylesheet_urls_builder_accepts_empty_vec` | PASS |
| `json_schema_derives_with_new_field` | PASS |
| `default_config_emits_ferro_base_css_link_and_no_cdn_script` | PASS |
| `tailwind_cdn_opt_in_coexists_with_default_stylesheet_urls` | PASS |
| `stylesheet_urls_emitted_in_order_and_replaces_default` | PASS |
| `empty_stylesheet_urls_emits_no_ferro_base_link` | PASS |
| `stylesheet_urls_are_html_escaped_in_href_attribute` | PASS |
| `serve_ferro_base_css_returns_200_with_text_css_content_type` | PASS |
| `serve_ferro_base_css_body_equals_embedded_constant` | PASS |
| `theme_css_injected_into_head_when_theme_active` | PASS |
| `theme_css_injected_after_tailwind_cdn` | PASS |
| `theme_css_does_not_duplicate_custom_head_content` | PASS |

## Output Metrics

- **ferro-base.css byte size (Content-Length):** 36,626 bytes (bootstrap placeholder — full Tailwind output after CLI regeneration will be larger)
- **Cache-Control:** `public, max-age=86400` (24 hours, per RESEARCH.md Open Question 3 resolution)
- **`cargo fmt --all -- --check`:** PASSED
- **`cargo clippy --all --all-targets -- -D warnings`:** PASSED (zero warnings)
- **`cargo test --all-features`:** PASSED (all tests pass)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added html_body() test helper for raw HTML assertions**
- **Found during:** Task 2
- **Issue:** The existing `response_body()` helper uses `format!("{body_bytes:?}")` which Debug-formats the byte body as `Full { data: Some(b"...") }`. HTML attributes containing `"` are double-escaped in this representation. New tests asserting on `<link rel="stylesheet" href="...">` (which contains quotes) failed because the assertions used literal HTML but the helper produced Debug-escaped output.
- **Fix:** Added `html_body(response: HttpResponse) -> String` helper that calls `response.body().to_string()` to get the raw HTML string. Updated all five new tests to use `html_body` instead of `response_body`.
- **Files modified:** `framework/src/json_ui/mod.rs`
- **Commit:** 19d756a4

## Known Stubs

None. The plan's goals are fully achieved:
- Default config produces `<link rel="stylesheet" href="/_ferro/ferro-base.css">` (verified by test)
- Route serves embedded bytes at correct content-type (verified by test)
- Theme injection uses plain `<style>` (verified by updated tests)

## Threat Surface Scan

No new threat surface beyond what the plan's threat model covers. `/_ferro/ferro-base.css` is an exact-match static route with no path parsing (T-143-07). `stylesheet_urls` values are HTML-escaped before href emission (T-143-09, test `stylesheet_urls_are_html_escaped_in_href_attribute` locks the contract). Body is compile-time embedded static bytes, no per-request allocation (T-143-11).

## Self-Check

- [x] `ferro-json-ui/src/config.rs` — FOUND, contains `pub stylesheet_urls: Vec<String>`, `tailwind_cdn: false`
- [x] `framework/src/json_ui/mod.rs` — FOUND, contains `for url in &config.stylesheet_urls`, `html_escape(url)`, plain `<style>` injection
- [x] `framework/src/server.rs` — FOUND, contains route arm, `serve_ferro_base_css`, `FERRO_BASE_CSS`, `text/css; charset=utf-8`, `public, max-age=86400`, `Bytes::from_static`
- [x] `.planning/phases/143-tailwind-static-css-pipeline/143-03-SUMMARY.md` — FOUND
- [x] Commits: 8e9a16fb, 19d756a4, 5acd0d74, 01b0464f — all verified in git log
- [x] `cargo fmt --all -- --check` — PASSED
- [x] `cargo clippy --all --all-targets -- -D warnings` — PASSED
- [x] `cargo test --all-features` — PASSED

## Self-Check: PASSED
