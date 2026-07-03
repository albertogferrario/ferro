---
phase: 250
slug: token-vocabulary-v2-default-theme-refresh
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-03
---

# Phase 250 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[test]`) + `tempfile` for CLI scaffold tests |
| **Config file** | none — standard `cargo test` |
| **Quick run command** | `cargo test -p ferro-theme -p ferro-cli -p ferro-json-ui` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~60s (crate-scoped); full suite several minutes |

---

## Sampling Rate

- **After every task commit:** Run the quick crate-scoped command for the touched crate(s) — one CPU-intensive cargo run at a time, never in parallel (project convention)
- **After every plan wave:** Run `cargo test -p ferro-theme -p ferro-cli -p ferro-json-ui`
- **Before `/gsd-verify-work`:** Full CI-exact gate green: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| tokens-30 | TBD | 1 | DS-01 | — | N/A | unit | `cargo test -p ferro-theme` (assert `ALL_TOKENS.len() == 30`) | ❌ W0 | ⬜ pending |
| base-css-utilities | TBD | 1 | DS-01 | — | N/A | unit/grep | assert regenerated `ferro-base.css` contains `duration-fast`, `ease-base`, `var(--motion-duration-fast,` | ❌ W0 | ⬜ pending |
| reduced-motion | TBD | 1 | DS-01 | — | N/A | grep | assert `ferro-base.css` contains `prefers-reduced-motion` collapse block | ❌ W0 | ⬜ pending |
| make-theme-scaffold | TBD | 1 | DS-01 | — | N/A | unit | `cargo test -p ferro-cli -- test_make_theme` (rename 23→30-slot test, extend) | ✅ (extend) | ⬜ pending |
| default-css-30 | TBD | 1 | DS-02 | — | N/A | unit | `cargo test -p ferro-theme -- default_theme` (extend to assert 7 new declarations, light + both dark blocks) | ✅ (extend) | ⬜ pending |
| v1-render-identical | TBD | 1 | DS-01 | — | N/A | unit | assert `ferro-base.css` fallback syntax (`var(--motion-duration-fast,` etc.) — the structural guarantee | ❌ W0 | ⬜ pending |
| themes-docs | TBD | 2 | DS-02 | — | N/A | grep | `docs/src/features/themes.md` contains v2 token reference + root-font-size recipe | ✅ (extend) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Test asserting `ALL_TOKENS.len() == 30` — in `ferro-theme/src/token.rs` tests
- [ ] Test asserting regenerated `ferro-base.css` contains the new utilities and `var()` fallbacks — in ferro-json-ui (or post-regen assertion)
- [ ] Extend `default_theme_returns_non_empty_css_with_color_primary` to cover the 7 new tokens

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Default theme reads as the documented design language (cool-tinted neutrals, single accent, dark not gloomy) | DS-02 | Aesthetic judgment | Chrome MCP screenshots of sample `app/` pages, light + dark, before/after comparison |
| Assumptions A1/A2 (Tailwind utility class names `duration-fast`/`ease-base`) | DS-01 | Requires regen output inspection | `grep 'duration-fast\|ease-base' ferro-json-ui/assets/ferro-base.css` immediately after first `scripts/gen-ferro-base-css.sh` run |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
