---
phase: 237
slug: actiongroup-component-dropdownmenu-replacement
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-22
---

# Phase 237 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none — `ferro-json-ui/Cargo.toml` (no separate test config) |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30–90s quick (`-p ferro-json-ui`); full gate minutes (workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui`
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~90 seconds (quick), full gate before publish step

---

## Per-Task Verification Map

| SC | Behavior | Test Type | Automated Command | File Exists | Status |
|----|----------|-----------|-------------------|-------------|--------|
| SC-1 | Inline ≤ `max_inline`, overflow ≥ `max_inline`; destructive always in kebab, last, regardless of input order | unit | `cargo test -p ferro-json-ui render_action_group` | ❌ W0 | ⬜ pending |
| SC-1 | Kebab hidden when nothing overflows | unit | `cargo test -p ferro-json-ui action_group_no_overflow_hides_kebab` | ❌ W0 | ⬜ pending |
| SC-2 | `{"$data":"/x/actions"}` binding renders identically to literal list | unit | `cargo test -p ferro-json-ui action_group_data_binding_parity` | ❌ W0 | ⬜ pending |
| SC-2 | `visible_if` row gate, fail-closed (absent/falsy hides) | unit | `cargo test -p ferro-json-ui action_group_visible_if` | ❌ W0 (pattern exists in data.rs) | ⬜ pending |
| SC-3 | Non-GET inline action renders inside `<form method="post">` | unit | `cargo test -p ferro-json-ui action_group_non_get_wraps_form` | ❌ W0 | ⬜ pending |
| SC-3 | GET inline action renders as plain link/button (no form) | unit | `cargo test -p ferro-json-ui action_group_get_renders_link` | ❌ W0 | ⬜ pending |
| SC-4 | `DropdownMenu` absent from `BUILTIN_TYPES`; `ActionGroup` present | unit | `cargo test -p ferro-json-ui builtin_types_count_drift_guard` (stays 47) | ✅ (update count comment + name) | ⬜ pending |
| SC-4 | `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` + runtime length guard | unit | `cargo test -p ferro-json-ui builtin_specs_len_matches_dispatch` | ✅ relational | ⬜ pending |
| SC-4 | Schema export lists `ActionGroup`, omits `DropdownMenu` | unit | `cargo test -p ferro-json-ui` (schema export test) | ✅ existing | ⬜ pending |
| SC-5 | `emit_actions_placeholder` emits `ActionGroup` + `ActionGroupProps` | unit | `cargo test -p ferro-json-ui actions_slot_emits` (rename + decode type) | ✅ (update existing) | ⬜ pending |
| SC-5 | ferro-mcp name list contains `ActionGroup`, not `DropdownMenu`, count 47 (fix pre-existing 45-entry gap: add SegmentedControl + SidebarLayout) | unit | `cargo test -p ferro-mcp test_all_components_present` | ✅ (update `expected[]`) | ⬜ pending |
| SC-6 | Workspace version `0.2.73` (NOT 0.2.72 — already shipped); ferro-json-ui + ferro-rs publish operator-gated | manual | `grep 'version = "0.2.73"' Cargo.toml` + operator publish | ❌ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/render/containers.rs` (or atoms.rs) — `render_action_group` unit tests: inline-vs-overflow partition, destructive-last ordering, form-wrapping for non-GET, kebab-hidden-when-no-overflow, `visible_if` gate, `$data` binding parity.
- [ ] `ferro-json-ui/src/component.rs` — `schema_for_action_group_props_generates()` + `schema_for_action_item_generates()` (pattern at component.rs:1509).

*Existing infrastructure (drift-guard tests, projection-builder tests, ferro-mcp catalog test) covers all other requirements via in-place updates.*

---

## Manual-Only Verifications

| Behavior | SC | Why Manual | Test Instructions |
|----------|----|-----------|--------------------|
| crates.io publish of ferro-json-ui + ferro-rs at 0.2.73 | SC-6 | Operator-gated publish (CI `publish.yml` Wave 1A); not automatable from a plan per project convention | After full gate green + version bump committed + tag, operator triggers the gated publish; verify on crates.io |
| CSS regen output | — | `scripts/gen-ferro-base-css.sh` downloads Tailwind CLI; run after component code lands | Run script, confirm `ferro-base.css` regenerated with no errors; ActionGroup likely introduces no new classes (reuses Button + kebab classes) |

---

## Validation Sign-Off

- [ ] All SC tasks have an `<automated>` verify or Wave 0 dependency (SC-6 publish is manual/operator-gated by design)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (render_action_group tests + schema tests)
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s (quick run)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
