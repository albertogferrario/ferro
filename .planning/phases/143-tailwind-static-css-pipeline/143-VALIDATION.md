---
phase: 143
slug: tailwind-static-css-pipeline
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-20
audited: 2026-04-21
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
| 143-01-01 | 01 | 1 | D-01 | T-143-02 | Install signed Tailwind CLI from tailwindlabs GitHub Releases (HTTPS) | manual checkpoint + smoke | `.tooling/bin/tailwindcss --help \| head -1 \| grep -qi "tailwindcss"` | ✅ (binary at `.tooling/bin/tailwindcss`) | ✅ green |
| 143-01-02 | 01 | 1 | D-01, D-02 | T-143-01, T-143-03 | CSS regeneration is idempotent; scanned paths are source-only (no secrets) | integration | `test -s ferro-json-ui/assets/ferro-base.css && python3 -c "css=open('ferro-json-ui/assets/ferro-base.css').read(); assert 'flex' in css and 'rounded-md' in css, 'missing utility classes'"` | ✅ (36 KB generated) | ✅ green |
| 143-01-03 | 01 | 1 | D-03, D-17 | T-143-01 | `include_str!` enforces compile-time UTF-8 validation | unit | `cargo test -p ferro-json-ui ferro_base_css_non_empty -- --exact` | ✅ (`ferro-json-ui/src/assets.rs`) | ✅ green |
| 143-01-04 | 01 | 1 | D-02 | T-143-01 | CI drift check regenerates CSS and diffs against committed file | integration (CI) | `grep -q "ferro-base-css-drift" .github/workflows/ci.yml && grep -q "gen-ferro-base-css.sh" .github/workflows/ci.yml && grep -q "git diff --exit-code ferro-json-ui/assets/ferro-base.css" .github/workflows/ci.yml` | ✅ (job uses `install-tailwind.sh`, not inline binary) | ✅ green |
| 143-01-05 | 01 | 1 | (Plan-01 end-to-end gate) | — | Human confirms the pipeline works on the dev machine | manual checkpoint | `bash scripts/gen-ferro-base-css.sh && git diff --quiet ferro-json-ui/assets/ferro-base.css && cargo test -p ferro-json-ui ferro_base_css_non_empty -- --exact` | ✅ (UAT 8/8 passed) | ✅ green |
| 143-02-01 | 02 | 1 | D-09, D-10, D-11 | T-143-05, T-143-06 | Injectable CSS: no unknown at-rules (browsers ignore `@theme`) | unit | `! grep -q '@import "tailwindcss"' ferro-theme/assets/default.css && ! grep -q "@theme" ferro-theme/assets/default.css && grep -q ":root {" ferro-theme/assets/default.css && grep -q '@media (prefers-color-scheme: dark)' ferro-theme/assets/default.css && grep -q '\\[data-theme="dark"\\]' ferro-theme/assets/default.css && cargo test -p ferro-theme --lib` | ✅ (file exists; content rewritten) | ✅ green |
| 143-03-01 | 03 | 2 | D-06, D-07, D-12, D-16 | — | `JsonUiConfig::default()` ships static-CSS-first; `schemars::JsonSchema` derive holds | unit | `cargo test -p ferro-json-ui --lib default_has_tailwind_cdn_false_and_default_stylesheet_urls stylesheet_urls_builder_replaces_entire_list stylesheet_urls_builder_accepts_empty_vec json_schema_derives_with_new_field` | ✅ (`ferro-json-ui/src/config.rs`) | ✅ green |
| 143-03-02 | 03 | 2 | D-06, D-09, D-11, D-14, D-15, D-16 | T-143-09, T-143-10 | Stylesheet URLs are HTML-escaped in `href`; theme CSS injected as plain `<style>` | unit | `cargo test -p ferro-rs --all-features default_config_emits_ferro_base_css_link_and_no_cdn_script tailwind_cdn_opt_in_coexists_with_default_stylesheet_urls stylesheet_urls_emitted_in_order_and_replaces_default empty_stylesheet_urls_emits_no_ferro_base_link stylesheet_urls_are_html_escaped_in_href_attribute theme_css_injected_into_head_when_theme_active theme_css_injected_after_tailwind_cdn` | ✅ (`framework/src/json_ui/mod.rs`) | ✅ green |
| 143-03-03 | 03 | 2 | D-04, D-05 | T-143-07, T-143-08, T-143-11 | Exact-string-match route (no path parsing); zero-copy `Bytes::from_static`; 24h cache | integration | `cargo test -p ferro-rs --all-features serve_ferro_base_css_returns_200_with_text_css_content_type serve_ferro_base_css_body_equals_embedded_constant` | ✅ (`framework/src/server.rs`) | ✅ green |
| 143-03-04 | 03 | 2 | (phase pre-commit gate) | — | Zero clippy warnings, all tests green | workspace | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ✅ (runs existing suites) | ✅ green |
| 143-04-01 | 04 | 3 | D-10, D-15 | T-143-13, T-143-14 | Scaffolder output is plain CSS; tests enforce absence of Tailwind-CDN syntax | unit | `cargo test -p ferro-cli test_make_theme_creates_directory_structure test_make_theme_tokens_css_has_all_23_token_slots test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax test_make_theme_tokens_css_has_dark_mode_block test_make_theme_theme_json_is_empty_object test_make_theme_fails_if_directory_exists test_make_theme_succeeds_once_fails_on_repeat` | ✅ (`ferro-cli/src/commands/make_theme.rs`) | ✅ green |
| 143-04-02 | 04 | 3 | (phase pre-commit gate) | — | Full workspace passes | workspace | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

All Wave 0 setup happens inside plan tasks — no separate pre-phase bootstrap is required. The plans themselves create every missing test and asset:

- [x] Plan 01 Task 1 — install `tailwindcss` CLI (pinned v4.2.3 at `.tooling/bin/tailwindcss` via `scripts/install-tailwind.sh`).
- [x] Plan 01 Task 2 — created `ferro-json-ui/assets/input.css`, `ferro-json-ui/assets/ferro-base.css`, `scripts/gen-ferro-base-css.sh`.
- [x] Plan 01 Task 3 — created `ferro-json-ui/src/assets.rs` with the `ferro_base_css_non_empty` test.
- [x] Plan 01 Task 4 — added the `ferro-base-css-drift` CI job (uses `install-tailwind.sh` + `git diff --exit-code`).
- [x] Plan 03 Task 1 — added four `JsonUiConfig` tests in `ferro-json-ui/src/config.rs`.
- [x] Plan 03 Task 2 — promoted `html_escape()` to production scope; added seven `build_response` / `theme_tests` in `framework/src/json_ui/mod.rs`.
- [x] Plan 03 Task 3 — created `ferro_base_css_route_tests` module in `framework/src/server.rs` with two integration tests.

`wave_0_complete: true` — all Wave 0 scaffolds committed and green as of 2026-04-21.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tailwind CLI binary installed | D-01 | `scripts/install-tailwind.sh` downloads the signed binary into `.tooling/bin/tailwindcss` (gitignored). First-run requires network access; subsequent runs are no-ops when the pinned version is already present. | Run `bash scripts/install-tailwind.sh && .tooling/bin/tailwindcss --help` and confirm `tailwindcss v4.2.3` banner. |
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

---

## Validation Audit 2026-04-21

| Metric | Count |
|--------|-------|
| Tasks audited | 12 |
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Status corrections | 12 (all ⬜ pending → ✅ green) |
| Command corrections | 2 (143-01-02: removed stale `bg-primary` grep; 143-01-04: updated to match `install-tailwind.sh` implementation) |

All 12 tasks are COVERED. Phase is fully Nyquist-compliant.
