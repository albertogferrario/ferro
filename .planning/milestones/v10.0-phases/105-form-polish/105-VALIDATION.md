---
phase: 105
slug: form-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 105 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none (workspace-level Cargo.toml) |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 105-01-01 | 01 | 1 | FRM-01 | unit | `cargo test -p ferro-json-ui select_renders_chevron_wrapper` | ❌ W0 | ⬜ pending |
| 105-01-02 | 01 | 1 | FRM-02 | unit | `cargo test -p ferro-json-ui input_renders_error_with_red_border` | ✅ (extend) | ⬜ pending |
| 105-01-03 | 01 | 1 | FRM-03 | unit | `cargo test -p ferro-json-ui input_renders_transition_classes` | ❌ W0 | ⬜ pending |
| 105-01-04 | 01 | 1 | FRM-04 | unit | `cargo test -p ferro-json-ui input_disabled_renders_disabled_classes` | ❌ W0 | ⬜ pending |
| 105-01-05 | 01 | 1 | FRM-05 | unit | `cargo test -p ferro-json-ui select_renders_error` | ✅ (extend) | ⬜ pending |
| 105-01-06 | 01 | 1 | FRM-06 | unit | `cargo test -p ferro-json-ui textarea_renders_error_focus_ring` | ❌ W0 | ⬜ pending |
| 105-01-07 | 01 | 1 | FRM-07 | unit | `cargo test -p ferro-json-ui input_description_order` | ❌ W0 | ⬜ pending |
| 105-01-08 | 01 | 1 | FRM-07 | unit | `cargo test -p ferro-json-ui select_description_order` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `select_renders_chevron_wrapper` — asserts `relative` wrapper div, `aria-hidden`, `<svg` presence — covers FRM-01
- [ ] `input_renders_transition_classes` — asserts `transition-colors`, `duration-150`, `motion-reduce:transition-none` — covers FRM-03
- [ ] `input_disabled_renders_disabled_classes` — asserts `disabled:opacity-50`, `disabled:cursor-not-allowed` — covers FRM-04
- [ ] `textarea_renders_error_focus_ring` — asserts `ring-destructive` when error set on `InputType::Textarea` — covers FRM-06
- [ ] `input_description_order` — asserts description `<p>` appears AFTER `<input` in HTML — covers FRM-07
- [ ] `select_description_order` — asserts description `<p>` appears AFTER `</select>` in HTML — covers FRM-07

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SVG chevron visible across browsers | FRM-01 | Visual rendering cross-browser | Open a page with select element in Chrome, Firefox, Safari; verify chevron appears |
| 150ms transition smooth visual effect | FRM-03 | Visual timing perception | Focus/unfocus form elements; verify smooth color transition |
| Reduced motion suppresses animation | FRM-03 | Requires OS accessibility setting | Enable "Reduce motion" in OS; verify no transition animation |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
