---
phase: 146
slug: add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-22
---

# Phase 146 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none — standard cargo workspace |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 146-01-01 | 01 | 1 | R1 | — | `html_escape` on all dynamic values | unit | `cargo test -p ferro-json-ui render_key_value_editor` | ❌ W0 | ⬜ pending |
| 146-01-02 | 01 | 1 | R2 | — | Pre-filled rows from data_path object | unit | `cargo test -p ferro-json-ui render_key_value_editor` | ❌ W0 | ⬜ pending |
| 146-01-03 | 01 | 1 | R3 | — | Error state applies `border-destructive` to inputs | unit | `cargo test -p ferro-json-ui render_key_value_editor` | ❌ W0 | ⬜ pending |
| 146-01-04 | 01 | 1 | R4 | — | `allow_custom_keys=false` emits `<select>` | unit | `cargo test -p ferro-json-ui render_key_value_editor` | ❌ W0 | ⬜ pending |
| 146-01-05 | 01 | 1 | R5 | — | Non-empty `suggested_keys` emits `<datalist>` | unit | `cargo test -p ferro-json-ui render_key_value_editor` | ❌ W0 | ⬜ pending |
| 146-01-06 | 01 | 1 | R6 | — | Empty data_path → hidden field value is `{}` | unit | `cargo test -p ferro-json-ui render_key_value_editor` | ❌ W0 | ⬜ pending |
| 146-01-07 | 01 | 1 | R9 | — | Serde round-trip: serialize + deserialize KeyValueEditor | unit | `cargo test -p ferro-json-ui` | ❌ W0 | ⬜ pending |
| 146-02-01 | 02 | 2 | R7 | — | `setupKeyValueEditor` present in JS bundle | unit | `cargo test -p ferro-json-ui bundle_contains_all_setup_functions` | ❌ W0 | ⬜ pending |
| 146-02-02 | 02 | 2 | R8 | — | `setupKeyValueEditor();` in dispatcher string | unit | `cargo test -p ferro-json-ui dispatcher_invokes_every_setup` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/render.rs` — add `render_key_value_editor` unit tests in `#[cfg(test)]` block for: empty state, pre-filled rows, error state, select variant, datalist presence, hidden field serialization
- [ ] `ferro-json-ui/src/component.rs` — add serde round-trip test for `KeyValueEditorProps`
- [ ] `ferro-json-ui/src/runtime/mod.rs` — update `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup` test arrays to include `setupKeyValueEditor`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Add row button appends empty row | D-06 | DOM interaction requires browser | Open a page with KeyValueEditor, click "Add row", verify new row appears |
| Delete row removes row and syncs JSON | D-07 | DOM interaction requires browser | Click × on a row, verify row removed and hidden field value updated |
| Input event syncs hidden field | D-09 | DOM interaction requires browser | Type in key/value inputs, verify hidden field JSON updates |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
