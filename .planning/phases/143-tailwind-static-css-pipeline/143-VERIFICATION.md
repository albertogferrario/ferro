---
phase: 143-tailwind-static-css-pipeline
verified: 2026-04-21T10:00:00Z
status: human_needed
score: 7/7
overrides_applied: 0
human_verification:
  - test: "Open any ferro-json-ui page in Safari (desktop + iOS). Network tab: confirm zero requests to cdn.jsdelivr.net, one 200 request to /_ferro/ferro-base.css with Content-Type: text/css."
    expected: "Page renders fully styled immediately — no FOUC, no WASM download, no third-party CDN script for CSS."
    why_human: "Safari/WebKit rendering and network waterfall require a real browser. Automated checks cannot confirm FOUC absence or WASM-download elimination."
  - test: "Load gestiscilo login, dashboard, and /s/{slug}/ pages in Chrome, Safari, and Firefox side-by-side and compare against a pre-change screenshot."
    expected: "All pages visually identical to the pre-change baseline — no regressions in layout, colors, typography, or component shapes."
    why_human: "Visual regression across three browsers requires human inspection; no automated pixel-diff baseline is committed."
  - test: "Enable a dark theme (gestiscilo or any app theme) and toggle dark mode — both via system preference and via [data-theme='dark'] attribute."
    expected: "Dark theme variables applied correctly. body class='dark bg-background text-text font-sans' renders the dark token set."
    why_human: "Dynamic CSS variable inheritance and theme-toggle behavior require live browser observation."
---

# Phase 143: Tailwind Static CSS Pipeline — Verification Report

**Phase Goal:** Replace Tailwind v4's in-browser JIT runtime with a build-time CSS pipeline. Pre-build `ferro-base.css`, serve it as a framework-owned route, flip `JsonUiConfig::tailwind_cdn` default to `false`, add `stylesheet_urls` field, and switch theme injection from `<style type="text/tailwindcss">` to plain `<style>`.

**Verified:** 2026-04-21
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `ferro-json-ui/assets/ferro-base.css` exists as a committed, non-empty file containing compiled Tailwind utility classes | VERIFIED | File is 36,626 bytes (1 minified line); contains `flex`, `rounded-md`, `bg-primary`, `font-sans` confirmed via grep |
| 2 | `ferro_json_ui::FERRO_BASE_CSS` is a `pub &'static str` embedded at compile time via `include_str!` | VERIFIED | `ferro-json-ui/src/assets.rs:12` declares `pub const FERRO_BASE_CSS: &str = include_str!("../assets/ferro-base.css")`. Re-exported in `lib.rs:58` as `pub use assets::FERRO_BASE_CSS;` |
| 3 | `JsonUiConfig::tailwind_cdn` defaults to `false` | VERIFIED | `ferro-json-ui/src/config.rs:51` — `tailwind_cdn: false`. Test `default_has_tailwind_cdn_false_and_default_stylesheet_urls` locks the contract (UAT #4 passed) |
| 4 | `JsonUiConfig::stylesheet_urls` field exists defaulting to `["/_ferro/ferro-base.css"]` | VERIFIED | `ferro-json-ui/src/config.rs:40,52`. Builder method `stylesheet_urls(mut self, urls: Vec<String>) -> Self` at line 77. Test coverage via four unit tests and five head-assembly integration tests |
| 5 | Framework registers `GET /_ferro/ferro-base.css` route serving the embedded CSS | VERIFIED | `framework/src/server.rs:223` dispatches to `serve_ferro_base_css()` at line 335. Route uses exact-string comparison (path traversal structurally impossible). Cache-Control: `public, max-age=86400`. Two integration tests (UAT #6 passed) |
| 6 | Theme injection uses plain `<style>` not `<style type="text/tailwindcss">` | VERIFIED | `framework/src/json_ui/mod.rs` — no production emission of `type="text/tailwindcss"`. Line 1024-1025 is a test assertion checking for *absence* of the old magic MIME type. Updated theme tests confirm plain `<style>` output (UAT #7 passed) |
| 7 | CI drift job fails if `ferro-base.css` is out of date vs Tailwind CLI output | VERIFIED | `.github/workflows/ci.yml:93-107` — `ferro-base-css-drift` job runs `gen-ferro-base-css.sh` and diffs with `git diff --exit-code ferro-json-ui/assets/ferro-base.css`. Uses pinned v4.2.3 via `scripts/install-tailwind.sh` (idempotent, installs to `.tooling/bin/`) |

**Score:** 7/7 truths verified

---

### Required Artifacts

| Artifact | Status | Details |
|----------|--------|---------|
| `ferro-json-ui/assets/ferro-base.css` | VERIFIED | 36,626 bytes, minified, committed. Generated from Tailwind v4.2.3 CLI via `input.css` with `@source` directives covering both crates and `@source inline(...)` safelist |
| `ferro-json-ui/assets/input.css` | VERIFIED | Contains `@import "tailwindcss"`, `@source "../../ferro-json-ui/src"`, `@source "../../framework/src"`, `@source inline(...)` safelist |
| `ferro-json-ui/src/assets.rs` | VERIFIED | `pub const FERRO_BASE_CSS: &str = include_str!("../assets/ferro-base.css")` with `ferro_base_css_non_empty` test |
| `ferro-json-ui/src/lib.rs` | VERIFIED | `pub mod assets;` and `pub use assets::FERRO_BASE_CSS;` both present |
| `scripts/gen-ferro-base-css.sh` | VERIFIED | Executable (-rwxr-xr-x), delegates to `scripts/install-tailwind.sh` for pinned CLI, then runs `tailwindcss -i ... -o ... --minify` |
| `scripts/install-tailwind.sh` | VERIFIED | Pins Tailwind v4.2.3, installs to `.tooling/bin/tailwindcss` (gitignored), idempotent |
| `ferro-json-ui/src/config.rs` | VERIFIED | `pub stylesheet_urls: Vec<String>` field, `tailwind_cdn: false` default, `stylesheet_urls()` builder |
| `framework/src/json_ui/mod.rs` | VERIFIED | Iterates `config.stylesheet_urls`, emits `<link rel="stylesheet" href="{html_escape(url)}">`, plain `<style>` for theme injection, `html_escape()` promoted to production scope |
| `framework/src/server.rs` | VERIFIED | `/_ferro/ferro-base.css` route, `serve_ferro_base_css()`, `Bytes::from_static(FERRO_BASE_CSS.as_bytes())` zero-copy serving |
| `ferro-theme/assets/default.css` | VERIFIED | No `@import "tailwindcss"`, no `@theme`. Uses `:root { ... }`, `@media (prefers-color-scheme: dark) { :root { ... } }`, `[data-theme="dark"] { ... }`. All 23 tokens present |
| `ferro-cli/src/commands/make_theme.rs` | VERIFIED | `tokens_css_template()` returns plain `:root { ... }` with dark-mode `@media` block. No `@theme {`. All 7 scaffolder tests pass (UAT #8) |
| `.github/workflows/ci.yml` | VERIFIED | `ferro-base-css-drift` job at line 93, uses `gen-ferro-base-css.sh` and `git diff --exit-code` |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|---|-----|--------|---------|
| `ferro-json-ui/src/assets.rs` | `ferro-json-ui/assets/ferro-base.css` | `include_str!("../assets/ferro-base.css")` | WIRED | Compile-time embed; file exists at 36,626 bytes |
| `ferro-json-ui/src/lib.rs` | `ferro-json-ui/src/assets.rs` | `pub use assets::FERRO_BASE_CSS` | WIRED | Both declarations present; public re-export confirmed |
| `framework/src/server.rs` | `ferro_json_ui::FERRO_BASE_CSS` | `Bytes::from_static(FERRO_BASE_CSS.as_bytes())` | WIRED | Route handler at line 336 uses the constant directly |
| `framework/src/json_ui/mod.rs` | `config.stylesheet_urls` | iteration in `build_response` at line 105 | WIRED | Emits one `<link>` per URL with HTML-escaped href |
| `.github/workflows/ci.yml` | `scripts/gen-ferro-base-css.sh` | `run: bash scripts/gen-ferro-base-css.sh` | WIRED | Drift job at line 99 invokes the script directly |
| `scripts/gen-ferro-base-css.sh` | `scripts/install-tailwind.sh` | `bash scripts/install-tailwind.sh` | WIRED | Pinned v4.2.3, idempotent — aligns with VALIDATION.md audit correction |

---

### Data-Flow Trace (Level 4)

Static asset phase — no dynamic data rendering. All paths are compile-time embedded constants or config-driven URL lists. Level 4 not applicable.

---

### Behavioral Spot-Checks

| Behavior | Evidence | Status |
|----------|----------|--------|
| `ferro_base_css_non_empty` test passes | UAT #1: `cargo test -p ferro-json-ui assets::tests::ferro_base_css_non_empty` — 1 passed | PASS |
| Default config disables CDN and sets stylesheet_urls | UAT #4: `default_has_tailwind_cdn_false_and_default_stylesheet_urls` test passes | PASS |
| Head assembly emits `<link>` to `/_ferro/ferro-base.css` | UAT #5: `default_config_emits_ferro_base_css_link_and_no_cdn_script` passes | PASS |
| Route serves CSS with correct headers | UAT #6: `serve_ferro_base_css_returns_200_with_text_css_content_type` and `serve_ferro_base_css_body_equals_embedded_constant` pass | PASS |
| Theme injection uses plain `<style>` | UAT #7: `theme_css_injected_into_head_when_theme_active` and `theme_css_injected_after_tailwind_cdn` pass | PASS |
| Scaffolder emits plain CSS | UAT #8: all 7 `make_theme` tests pass including `test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax` | PASS |

---

### Requirements Coverage

| Requirement | Description | Status | Evidence |
|-------------|-------------|--------|----------|
| D-01 | Tailwind v4 CLI generates `ferro-base.css` from source scan | SATISFIED | `input.css` with `@source` directives; `gen-ferro-base-css.sh` + `install-tailwind.sh` (pinned v4.2.3) |
| D-02 | `ferro-base.css` committed; CI detects drift | SATISFIED | File committed at 36,626 bytes; `ferro-base-css-drift` CI job with `git diff --exit-code` |
| D-03 | `include_str!` embed at compile time | SATISFIED | `assets.rs:12`; UTF-8 guaranteed at compile time |
| D-04 | Framework registers `GET /_ferro/ferro-base.css` automatically | SATISFIED | `server.rs:223`, exact-match route, no user configuration required |
| D-05 | No separate `ferro.use_json_ui()` init call needed | SATISFIED | Route is part of standard dispatch; no API change required |
| D-06 | `stylesheet_urls: Vec<String>` field, default `["/_ferro/ferro-base.css"]` | SATISFIED | `config.rs:40,52` |
| D-07 | `stylesheet_urls(Vec<String>)` builder replaces entire list | SATISFIED | `config.rs:77-79`; test `stylesheet_urls_builder_replaces_entire_list` |
| D-08 | App-level token URL injection via `stylesheet_urls` (no separate theme URL field) | SATISFIED | Mechanism exists; app migration is a separate consumer phase (gestiscilo) — explicitly deferred per CONTEXT.md Phase Boundary |
| D-09 | Remove `<style type="text/tailwindcss">` injection | SATISFIED | No production emission found in `mod.rs`; test at line 1024 asserts absence |
| D-10 | Theme CSS converted to plain `:root { ... }` declarations | SATISFIED | `ferro-theme/assets/default.css` converted; `make_theme` scaffolder output updated |
| D-11 | Inline `<style>` injection preserved for theme overrides | SATISFIED | Plain `<style>` emitted for active themes; UAT #7 confirms |
| D-12 | `tailwind_cdn` default flipped to `false` | SATISFIED | `config.rs:51`; breaking change accepted (pre-1.0) |
| D-13 | `tailwind_cdn(true)` remains functional as opt-in | SATISFIED | Builder method at `config.rs:66-68`; CDN path preserved |
| D-14 | No automatic mutual-exclusion logic between CDN and `stylesheet_urls` | SATISFIED | Both can coexist; test `tailwind_cdn_opt_in_coexists_with_default_stylesheet_urls` confirms |
| D-15 | Existing tests updated for plain `<style>` | SATISFIED | Three theme tests updated; fixture changed from `@theme` to `:root` |
| D-16 | Test: `default()` emits `<link>` to `/_ferro/ferro-base.css`, no CDN `<script>` | SATISFIED | `default_config_emits_ferro_base_css_link_and_no_cdn_script` test |
| D-17 | Embedded CSS is non-empty and valid UTF-8 | SATISFIED | `include_str!` guarantees UTF-8 at compile time; `ferro_base_css_non_empty` test asserts non-empty |

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `ferro-json-ui/assets/ferro-base.css` | Note: file was initially a 4,751-byte hand-crafted placeholder (Plan 01 SUMMARY). Current committed file is 36,626 bytes — the full Tailwind CLI output. Placeholder has been replaced. | Info | Resolved before UAT |

No blockers, no stubs in final committed code.

---

### Human Verification Required

The automated scaffold is fully verified. Three roadmap success criteria (SC-1, SC-3, SC-4, SC-5) require browser observation and are deferred to the gestiscilo consumer phase, which was explicitly called out as out-of-scope in the CONTEXT.md Phase Boundary section. However, they remain open items on the roadmap success criteria list and must be confirmed before the v11.7 milestone is closed.

**1. Safari rendering — no WASM, no CDN**

**Test:** Open a ferro-json-ui page in iPhone Safari and desktop Safari. Open the Network tab before loading.
**Expected:** Page renders fully styled immediately (no unstyled flash). Network tab shows zero requests to `cdn.jsdelivr.net`. One request to `/_ferro/ferro-base.css` returns 200 with `Content-Type: text/css`.
**Why human:** Safari/WebKit rendering fidelity and network waterfall cannot be verified programmatically from the framework side.

**2. Cross-browser visual regression**

**Test:** Load the gestiscilo login, dashboard, and `/s/{slug}/` pages in Chrome, Safari, and Firefox simultaneously after bumping to this ferro version. Compare against a pre-change screenshot.
**Expected:** Visual output identical across browsers. No layout breaks, color regressions, missing shadows, or font changes vs the CDN baseline.
**Why human:** Pixel-level visual comparison across browsers requires human inspection. No automated baseline committed.

**3. Dark theme toggle**

**Test:** With an active ferro theme, toggle dark mode both via `prefers-color-scheme: dark` (OS setting) and via `[data-theme="dark"]` attribute programmatically. Inspect computed styles for `--color-background` and `--color-primary`.
**Expected:** CSS variables update correctly in both toggle paths. `body.dark` renders the dark token set. Theme injection does not re-trigger the Tailwind CDN script.
**Why human:** Dynamic CSS variable inheritance and runtime theme-switching require live browser DevTools observation.

---

### Notes

**D-08 tech debt (no gap):** D-08 designates `stylesheet_urls` as the injection point for app-level `tokens.css`. The mechanism exists and is documented. The gestiscilo consumer migration (converting `themes/gestiscilo/tokens.css` from `@theme` to `:root` and adding the URL to `stylesheet_urls`) is a separate consumer phase, explicitly excluded from this phase's boundary. No action required here.

**CI drift job implementation:** The committed CI job (`ferro-base-css-drift`) differs slightly from the Plan 01 Task 4 template — it invokes `gen-ferro-base-css.sh` (which calls `install-tailwind.sh` internally for version pinning) rather than downloading the binary inline. This is an improvement over the plan: the pinned-version installer (`scripts/install-tailwind.sh`) is shared between local dev and CI, and CI uses `git diff --exit-code` rather than `diff -u`. The VALIDATION.md audit (2026-04-21) documents and approves this deviation.

**Plan 04 disk exhaustion:** `cargo test --all-features` failed during Plan 04 execution due to disk exhaustion on the build machine (460 Gi used, 299 Mi available), not a code failure. The ferro-cli crate and all phase-relevant crates passed. UAT #8 confirms all 7 scaffolder tests pass. Full workspace validation passed in Plan 03 (prior wave).

---

## Gaps Summary

No gaps. All 7 observable truths are verified in the codebase. UAT completed 8/8 passed with zero issues. Status is `human_needed` solely because three ROADMAP success criteria require real-browser observation — these are explicitly deferred to the gestiscilo consumer phase per the Phase 143 boundary definition and are not actionable within this framework phase.

---

_Verified: 2026-04-21T10:00:00Z_
_Verifier: Claude (gsd-verifier)_
