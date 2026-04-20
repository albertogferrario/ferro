---
phase: 143
slug: tailwind-static-css-pipeline
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-20
---

# Phase 143 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) — workspace-wide |
| **Config file** | None — workspace `Cargo.toml` drives test discovery |
| **Quick run command** | `cargo test -p framework -p ferro-json-ui -p ferro-theme -p ferro-cli --all-features` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~120 seconds for the full suite on dev hardware; ~30 seconds for the scoped quick run |

---

## Sampling Rate

- **After every task commit:** Run the quick command above (scoped to the four crates modified by this phase).
- **After every plan wave:** Run the full suite command.
- **Before `/gsd-verify-work`:** Full suite must be green with zero clippy warnings under `-D warnings`.
- **Max feedback latency:** ~120 seconds (full suite); well under the 143s budget for this phase.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 143-01-01 | 01 | 1 | D-01 | T-143-02 | Install signed Tailwind CLI from tailwindlabs GitHub Releases (HTTPS) | manual checkpoint + smoke | `command -v tailwindcss && tailwindcss --help \| head -1 \| grep -qi "tailwindcss"` | ❌ W0 (binary install) | ⬜ pending |
| 143-01-02 | 01 | 1 | D-01, D-02 | T-143-01, T-143-03 | CSS regeneration is idempotent; scanned paths are source-only (no secrets) | integration | `test -s ferro-json-ui/assets/ferro-base.css && grep -q "flex" ferro-json-ui/assets/ferro-base.css && grep -q "bg-primary" ferro-json-ui/assets/ferro-base.css && grep -q "rounded-md" ferro-json-ui/assets/ferro-base.css` | ❌ W0 (generated in task) | ⬜ pending |
| 143-01-03 | 01 | 1 | D-03, D-17 | T-143-01 | `include_str!` enforces compile-time UTF-8 validation | unit | `cargo test -p ferro-json-ui ferro_base_css_non_empty -- --exact` | ❌ W0 (test created in task) | ⬜ pending |
| 143-01-04 | 01 | 1 | D-02 | T-143-01 | CI drift check regenerates CSS and diffs against committed file | integration (CI) | `grep -q "ferro-base-css-drift" .github/workflows/ci.yml && grep -q "tailwindcss-linux-x64" .github/workflows/ci.yml && grep -q "diff -u ferro-json-ui/assets/ferro-base.css" .github/workflows/ci.yml && python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` | ❌ W0 (wired in task) | ⬜ pending |
| 143-01-05 | 01 | 1 | (Plan-01 end-to-end gate) | — | Human confirms the pipeline works on the dev machine | manual checkpoint | `bash scripts/gen-ferro-base-css.sh && git diff --quiet ferro-json-ui/assets/ferro-base.css && cargo test -p ferro-json-ui ferro_base_css_non_empty -- --exact && python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` | ✅ (depends on Tasks 1-4) | ⬜ pending |
| 143-02-01 | 02 | 1 | D-09, D-10, D-11 | T-143-05, T-143-06 | Injectable CSS: no unknown at-rules (browsers ignore `@theme`) | unit | `! grep -q '@import "tailwindcss"' ferro-theme/assets/default.css && ! grep -q "@theme" ferro-theme/assets/default.css && grep -q ":root {" ferro-theme/assets/default.css && grep -q '@media (prefers-color-scheme: dark)' ferro-theme/assets/default.css && grep -q '\\[data-theme="dark"\\]' ferro-theme/assets/default.css && cargo test -p ferro-theme --lib` | ✅ (file exists; content rewritten) | ⬜ pending |
| 143-03-01 | 03 | 2 | D-06, D-07, D-12, D-16 | — | `JsonUiConfig::default()` ships static-CSS-first; `schemars::JsonSchema` derive holds | unit | `cargo test -p ferro-json-ui --lib default_has_tailwind_cdn_false_and_default_stylesheet_urls stylesheet_urls_builder_replaces_entire_list stylesheet_urls_builder_accepts_empty_vec json_schema_derives_with_new_field` | ❌ W0 (tests added in task) | ⬜ pending |
| 143-03-02 | 03 | 2 | D-06, D-09, D-11, D-14, D-15, D-16 | T-143-09, T-143-10 | Stylesheet URLs are HTML-escaped in `href`; theme CSS injected as plain `<style>` | unit | `cargo test -p framework --all-features default_config_emits_ferro_base_css_link_and_no_cdn_script tailwind_cdn_opt_in_coexists_with_default_stylesheet_urls stylesheet_urls_emitted_in_order_and_replaces_default empty_stylesheet_urls_emits_no_ferro_base_link stylesheet_urls_are_html_escaped_in_href_attribute theme_css_injected_into_head_when_theme_active theme_css_injected_after_tailwind_cdn` | ❌ W0 (tests added in task) | ⬜ pending |
| 143-03-03 | 03 | 2 | D-04, D-05 | T-143-07, T-143-08, T-143-11 | Exact-string-match route (no path parsing); zero-copy `Bytes::from_static`; 24h cache | integration | `cargo test -p framework --all-features serve_ferro_base_css_returns_200_with_text_css_content_type serve_ferro_base_css_body_equals_embedded_constant` | ❌ W0 (tests added in task) | ⬜ pending |
| 143-03-04 | 03 | 2 | (phase pre-commit gate) | — | Zero clippy warnings, all tests green | workspace | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ✅ (runs existing suites) | ⬜ pending |
| 143-04-01 | 04 | 3 | D-10, D-15 | T-143-13, T-143-14 | Scaffolder output is plain CSS; tests enforce absence of Tailwind-CDN syntax | unit | `cargo test -p ferro-cli test_make_theme_creates_directory_structure test_make_theme_tokens_css_has_all_23_token_slots test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax test_make_theme_tokens_css_has_dark_mode_block test_make_theme_theme_json_is_empty_object test_make_theme_fails_if_directory_exists test_make_theme_succeeds_once_fails_on_repeat` | ✅ (tests exist; renamed/updated in task) | ⬜ pending |
| 143-04-02 | 04 | 3 | (phase pre-commit gate) | — | Full workspace passes | workspace | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

All Wave 0 setup happens inside plan tasks — no separate pre-phase bootstrap is required. The plans themselves create every missing test and asset:

- [ ] Plan 01 Task 1 — install `tailwindcss` CLI on the dev machine (human checkpoint).
- [ ] Plan 01 Task 2 — create `ferro-json-ui/assets/input.css`, `ferro-json-ui/assets/ferro-base.css`, `scripts/gen-ferro-base-css.sh`.
- [ ] Plan 01 Task 3 — create `ferro-json-ui/src/assets.rs` with the `ferro_base_css_non_empty` test.
- [ ] Plan 01 Task 4 — add the `ferro-base-css-drift` CI job.
- [ ] Plan 03 Task 1 — add four `JsonUiConfig` tests (`default_has_*`, `stylesheet_urls_builder_*`, `json_schema_derives_with_new_field`).
- [ ] Plan 03 Task 2 — promote `html_escape()` to production scope and add five `build_response` tests (`default_config_emits_*`, `tailwind_cdn_opt_in_coexists_*`, `stylesheet_urls_emitted_in_order_*`, `empty_stylesheet_urls_*`, `stylesheet_urls_are_html_escaped_in_href_attribute`); update three existing `theme_tests`.
- [ ] Plan 03 Task 3 — create `ferro_base_css_route_tests` module with two integration tests.

`wave_0_complete: false` in the frontmatter reflects that these items are planned but not yet executed. The checker flips to `true` once Plan 01 Tasks 2-4 and Plan 03 Tasks 1-3 have committed their test scaffolds.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tailwind CLI binary on PATH | D-01 | Installing a binary into `/usr/local/bin` may require sudo; writes to a global PATH location outside Claude's safe-to-automate boundary. CI installs the Linux binary automatically in a GitHub Actions runner — only the dev machine needs the manual step. | Run `curl -sLo /usr/local/bin/tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64 && chmod +x /usr/local/bin/tailwindcss && tailwindcss --help` and confirm v4.x.x banner. |
| End-to-end Safari render verification | Context (production failure motivation) | Only a real Safari/WebKit browser can prove the WASM-download regression is gone. Unit tests cannot substitute for real-browser smoke. | After phase lands and gestiscilo bumps the ferro dep in a separate consumer phase, open gestiscilo.it login page in iPhone Safari and desktop Safari, confirm: (a) page is fully styled, (b) Network tab shows zero requests to `cdn.jsdelivr.net`, (c) one request to `/_ferro/ferro-base.css` returned 200 with `text/css`. |

All other behaviors have automated verification in the per-task map above.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify blocks; manual-only steps are documented above with explicit human-action instructions.
- [x] Sampling continuity: every plan has at least one automated test task; no run of 3 consecutive tasks lacks an automated verify.
- [x] Wave 0 covers all MISSING test references — each missing test is scheduled inside a plan task with concrete file+behavior.
- [x] No `--watch` flags; all commands terminate.
- [x] Feedback latency < 143s (full suite ~120s on dev hardware).
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-04-20
