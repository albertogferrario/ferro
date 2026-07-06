---
phase: 147
slug: detailform-component-for-inline-edit-ferro-json-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-23
---

# Phase 147 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test -p ferro-json-ui --lib` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30-60 seconds (quick); ~3-5 minutes (full, incremental) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui --lib` (targeted tests for the component crate)
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` (matches CI)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds (quick); 300 seconds (full)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 147-01-01 | 01 | 0 | D-01, D-02 | — | EditMode enum defined, from_query parses correctly | unit | `cargo test -p ferro-json-ui edit_mode::` | ❌ W0 | ⬜ pending |
| 147-01-02 | 01 | 0 | D-03, D-04 | — | DetailField/DetailFormProps structs compile; serde round-trip | unit | `cargo test -p ferro-json-ui detail_form_props_roundtrip` | ❌ W0 | ⬜ pending |
| 147-01-03 | 01 | 0 | D-05, D-08 | — | View mode renders `<dl>` with `<dt>`/`<dd>`, no `<form>` | unit | `cargo test -p ferro-json-ui render_detail_form_view` | ❌ W0 | ⬜ pending |
| 147-01-04 | 01 | 0 | D-06, D-11, D-12 | — | Edit mode renders `<form>` around `<dl>`, renders each input | unit | `cargo test -p ferro-json-ui render_detail_form_edit` | ❌ W0 | ⬜ pending |
| 147-01-05 | 01 | 0 | D-09, D-10, D-14 | — | View shows Modifica link; Edit shows Salva+Annulla | unit | `cargo test -p ferro-json-ui render_detail_form_buttons` | ❌ W0 | ⬜ pending |
| 147-01-06 | 01 | 0 | D-11 (method spoofing) | T-147-01 (method spoofing integrity) | PUT/PATCH/DELETE emits hidden `_method` input | unit | `cargo test -p ferro-json-ui render_detail_form_method_spoofing` | ❌ W0 | ⬜ pending |
| 147-01-07 | 01 | 0 | D-15 (resolver) | — | Component::DetailForm gets action.url populated by resolver | unit | `cargo test -p ferro-json-ui detail_form_resolves_action_url` | ❌ W0 | ⬜ pending |
| 147-01-08 | 01 | 0 | html_escape discipline | T-147-02 (XSS via label/value/url) | Dynamic strings escaped in output | unit | `cargo test -p ferro-json-ui render_detail_form_escapes` | ❌ W0 | ⬜ pending |
| 147-02-01 | 02 | 1 | D-01..D-04, D-17, D-19 | — | Component enum + Serde + catalog entry compile and round-trip | unit | `cargo test -p ferro-json-ui --lib` | ✅ | ⬜ pending |
| 147-02-02 | 02 | 1 | D-05..D-14 (render) | T-147-01, T-147-02 | render_detail_form produces expected HTML in both modes | unit | `cargo test -p ferro-json-ui render_detail_form_` | ✅ | ⬜ pending |
| 147-02-03 | 02 | 1 | D-15 | — | Resolver arms populate action.url and walk fields[i].input | unit | `cargo test -p ferro-json-ui detail_form_resolves_` | ✅ | ⬜ pending |
| 147-02-04 | 02 | 1 | D-18 | — | ComponentNode::detail_form factory exists and produces valid nodes | unit | `cargo test -p ferro-json-ui component_node_detail_form` | ✅ | ⬜ pending |
| 147-02-05 | 02 | 1 | D-19, ferro-mcp catalog | — | ferro-mcp catalog includes "DetailForm" (and backfills "KeyValueEditor") | unit | `cargo test -p ferro-mcp json_ui_catalog::` | ✅ | ⬜ pending |
| 147-02-06 | 02 | 1 | Full gate | — | fmt + clippy + full suite green | integration | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/component.rs` — add test module stubs (tests for `detail_form_props_roundtrip`, `edit_mode_from_query_*`)
- [ ] `ferro-json-ui/src/render.rs` — add test module stubs for `render_detail_form_view`, `render_detail_form_edit`, `render_detail_form_buttons`, `render_detail_form_method_spoofing`, `render_detail_form_escapes`
- [ ] `ferro-json-ui/src/resolve.rs` — add test stub for `detail_form_resolves_action_url`
- [ ] `ferro-mcp/src/tools/json_ui_catalog.rs` — extend existing exhaustive-name-list assertion (~L1115) to include "DetailForm" (and backfill "KeyValueEditor")

All stubs should compile with `#[ignore]` or `unimplemented!()` bodies so Wave 0 passes cargo build/test (with the stub tests skipped). Wave 1 un-ignores them.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual correctness of rendered output in a real browser | D-05, D-09, D-14 | Token-class to visual rendering is a CSS/browser concern, not unit-testable | Render a sample DetailForm in the app/, load in browser, verify `<dl>` layout, Modifica button, Salva+Annulla placement |
| Empty-label input inside `<dd>` does not introduce visual duplication | Research Q6 resolution | Depends on how the empty `<label>` collapses in the rendered UI | Render a sample Edit-mode DetailForm, visually confirm no duplicated label text appears above the input (the `<dt>` already provides the visible label) |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
