---
phase: 99
slug: semantic-theme-system-with-intent-driven-templates
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-12
---

# Phase 99 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none — workspace `cargo test --all-features` |
| **Quick run command** | `cargo test -p ferro-theme` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-theme && cargo clippy -p ferro-theme -- -D warnings`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 99-01-01 | 01 | 1 | THEME-01 | unit | `cargo test -p ferro-theme loader` | ❌ W0 | ⬜ pending |
| 99-01-02 | 01 | 1 | THEME-02 | unit | `cargo test -p ferro-theme token` | ❌ W0 | ⬜ pending |
| 99-01-03 | 01 | 1 | THEME-03 | unit | `cargo test -p ferro-theme template` | ❌ W0 | ⬜ pending |
| 99-02-01 | 02 | 2 | THEME-04 | unit | `cargo test -p ferro-rs theme::middleware` | ❌ W0 | ⬜ pending |
| 99-02-02 | 02 | 2 | THEME-05 | unit | `cargo test -p ferro-rs theme::middleware` | ❌ W0 | ⬜ pending |
| 99-02-03 | 02 | 2 | THEME-06 | unit | `cargo test -p ferro-rs theme::context` | ❌ W0 | ⬜ pending |
| 99-03-01 | 03 | 2 | THEME-07 | unit | `cargo test -p ferro-json-ui render` | ✅ (update) | ⬜ pending |
| 99-03-02 | 03 | 2 | THEME-08 | unit | `cargo test -p ferro-json-ui layout` | ✅ (update) | ⬜ pending |
| 99-03-03 | 03 | 2 | THEME-09 | unit | `cargo test -p ferro-rs json_ui` | ❌ W0 | ⬜ pending |
| 99-04-01 | 04 | 2 | THEME-10 | unit | `cargo test -p ferro-projections render::json_ui` | ✅ (update) | ⬜ pending |
| 99-05-01 | 05 | 3 | THEME-11 | unit | `cargo test -p ferro-cli make_theme` | ❌ W0 | ⬜ pending |
| 99-05-02 | 05 | 3 | THEME-12 | unit | `cargo test -p ferro-cli make_theme` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-theme/src/lib.rs` — crate skeleton with re-exports
- [ ] `ferro-theme/src/error.rs` — ThemeError enum
- [ ] `ferro-theme/src/template.rs` — IntentTemplate, ThemeTemplates types
- [ ] `ferro-theme/src/loader.rs` — Theme::from_path(), Theme::default_theme()
- [ ] `ferro-theme/assets/default.css` — embedded default CSS (include_str!)
- [ ] `framework/src/theme/mod.rs` — module skeleton
- [ ] `framework/src/theme/resolver.rs` — ThemeResolver trait
- [ ] `framework/src/theme/middleware.rs` — ThemeMiddleware
- [ ] `framework/src/theme/context.rs` — current_theme() task-local
- [ ] `ferro-cli/src/commands/make_theme.rs` — scaffold command

*Framework install: no new packages, ferro-theme added to workspace and framework optional deps*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Dark mode toggle via data-theme attribute | Token vocabulary | CSS media query is browser-level | Open rendered page, toggle data-theme="dark", verify token switch |
| Theme visual appearance (colors, radius) | render.rs migration | Visual correctness requires human eye | Render a Browse view, verify semantic tokens produce expected visual |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
