---
phase: 175
slug: json-ui-v2-runtime-patches-staff-domain-field-test
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-20
---

# Phase 175 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test runner via `cargo test` |
| **Config file** | none (workspace `Cargo.toml`) |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60s quick / ~3–4 min full |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui` (quick)
- **After every plan wave:** Run the full suite (`fmt --check && clippy -D warnings && test --all-features`)
- **Before `/gsd-verify-work`:** Full suite must be green AND the consumer staff-domain UAT must be re-run end-to-end against the patched runtime
- **Max feedback latency:** 60 seconds for the quick run

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Finding | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|---------|-----------------|-----------|-------------------|-------------|--------|
| 175-01-01 | 01 | 1 | F1 depth | Depth-17 spec rejected with `DepthExceeded` | unit | `cargo test -p ferro-json-ui from_json_rejects_depth_17` | ❌ W0 | ⬜ pending |
| 175-01-02 | 01 | 1 | F1 accept | Depth-8 spec accepted and renders end-to-end | unit | `cargo test -p ferro-json-ui from_json_accepts_depth_8` | ❌ W0 | ⬜ pending |
| 175-01-03 | 01 | 1 | F1 diagnostic | Walker tripwire emits "depth limit exceeded" not "cycle guard" | unit | `cargo test -p ferro-json-ui walker_depth_tripwire` | ❌ W0 | ⬜ pending |
| 175-01-04 | 01 | 1 | F1 cycle | Cycle detector emits "cycle detected" only on real revisit | unit | `cargo test -p ferro-json-ui cycle_detector_only_on_revisit` | ❌ W0 | ⬜ pending |
| 175-02-01 | 02 | 2 | F3 tabs JS | `FERRO_RUNTIME_JS` contains `initTabFromUrl` + `URLSearchParams` | unit | `cargo test -p ferro-json-ui runtime_contains_init_tab_from_url` | ❌ W0 | ⬜ pending |
| 175-02-02 | 02 | 2 | F3 panels | Non-default tab panel has `hidden` class at server render | unit | `cargo test -p ferro-json-ui render::containers` (existing + new assertion) | ✅ | ⬜ pending |
| 175-03-01 | 03 | 2 | F6 interp | `{row.delete_url}` placeholder resolves in DataTable action URL | unit | `cargo test -p ferro-json-ui data_table_row_prefix_placeholder_resolved` | ❌ W0 | ⬜ pending |
| 175-03-02 | 03 | 2 | F6 back-compat | Bare `{delete_url}` placeholder still resolves (no regression) | unit | `cargo test -p ferro-json-ui data_table_bare_placeholder_resolved` | ✅ | ⬜ pending |
| 175-04-01 | 04 | 3 | F2 catalog | `global_catalog().lookup("CheckboxGroup")` returns `Some` | unit | `cargo test -p ferro-json-ui catalog_contains_checkbox_group` | ❌ W0 | ⬜ pending |
| 175-04-02 | 04 | 3 | F2 render | `CheckboxGroup` renders fieldset with N checkboxes sharing `name=field[]` | unit | `cargo test -p ferro-json-ui checkbox_group_renders_fieldset` | ❌ W0 | ⬜ pending |
| 175-04-03 | 04 | 3 | F2 types | `BUILTIN_TYPES.len()` assertion bumped 42 → 43 | unit | `cargo test -p ferro-json-ui builtin_types_count` | ✅ | ⬜ pending |
| 175-05-01 | 05 | 3 | F4 verify | Depth-8 spec with `Switch` renders `role="switch"` after F1 | unit | `cargo test -p ferro-json-ui switch_at_depth_8_renders_role_switch` | ❌ W0 | ⬜ pending |
| 175-05-02 | 05 | 3 | F4 docs | `docs/src/json-ui/components.md` contains "Switch" + substitution note | grep | `grep -q 'variant.*switch' docs/src/json-ui/components.md` | ❌ W0 | ⬜ pending |
| 175-06-01 | 06 | 4 | F5 file input | `Input[input_type=file]` emits `<input type="file" accept="...">` | unit | `cargo test -p ferro-json-ui input_file_renders_file_type_and_accept` | ❌ W0 | ⬜ pending |
| 175-06-02 | 06 | 4 | F5 enctype | `Form` with `enctype="multipart/form-data"` emits the attribute | unit | `cargo test -p ferro-json-ui form_enctype_emitted_when_set` | ❌ W0 | ⬜ pending |
| 175-06-03 | 06 | 4 | F5 e2e | End-to-end spec → DOM round-trip for an avatar-upload form | unit | `cargo test -p ferro-json-ui multipart_form_roundtrip` | ❌ W0 | ⬜ pending |
| 175-PHASE-01 | — | final | Phase gate | Full workspace suite passes with zero warnings | suite | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*File Exists column: ✅ infrastructure or test already present; ❌ W0 = needs to be added in Wave 0 of the owning plan*

---

## Wave 0 Requirements

Wave 0 of each plan creates the test scaffolding the implementation tasks will satisfy.

- [ ] `ferro-json-ui/src/spec.rs` (tests module) — rewrite `from_json_rejects_six_level_nesting` → `from_json_rejects_depth_17`; add `from_json_accepts_depth_8`
- [ ] `ferro-json-ui/src/render/mod.rs` (tests module) — `walker_depth_tripwire`, `cycle_detector_only_on_revisit`
- [ ] `ferro-json-ui/src/render/form.rs` (tests module) — `checkbox_group_renders_fieldset`, `input_file_renders_file_type_and_accept`, `form_enctype_emitted_when_set`, `switch_at_depth_8_renders_role_switch`
- [ ] `ferro-json-ui/src/render/data.rs` (tests module) — `data_table_row_prefix_placeholder_resolved`, `data_table_bare_placeholder_resolved` (back-compat assertion)
- [ ] `ferro-json-ui/src/runtime/mod.rs` (tests module) — `runtime_contains_init_tab_from_url`
- [ ] `ferro-json-ui/src/catalog.rs` (tests module) — `catalog_contains_checkbox_group`
- [ ] `ferro-json-ui/tests/` or in-module suite — `multipart_form_roundtrip` end-to-end fixture

Framework install: not required — Rust's built-in test runner is the only dependency.

---

## Manual-Only Verifications

| Behavior | Finding | Why Manual | Test Instructions |
|----------|---------|------------|-------------------|
| Consumer staff-domain UAT-2 (copy-source-to-N-targets ≤30s gate) passes against patched runtime | F1+F2+F3+F4 combined | Requires running the gestiscilo-it consumer app end-to-end with a real operator timing the flow; cannot be reduced to a Rust unit test | Re-run `gestiscilo-it/.planning/phases/151-staff-domain/151-UAT.md` UAT-2 against this branch as a local-path dependency; confirm the form is interactive and the timer is ≤30s |
| Tab switching feels instant (no flash, no roundtrip) | F3 | Visual / perceptual quality of the IIFE behavior | Open a `DetailPage` with two tabs in a browser, click between tabs, confirm no network request fires and no flash of stale content |
| Multipart avatar upload reaches the controller with the file body | F5 | Requires a browser to construct the multipart body; Rust unit tests can verify the HTML emission but not the browser's transmission | In a consumer app, render a staff-create form, attach a JPEG, submit; confirm the controller receives a multipart body and the avatar file is saved |

---

## Validation Sign-Off

- [ ] All implementation tasks have an `<automated>` verify command OR a Wave 0 test dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all ❌ W0 references listed above
- [ ] No watch-mode flags in test commands
- [ ] Feedback latency < 60s for the quick run
- [ ] `nyquist_compliant: true` set in frontmatter before phase verification

**Approval:** pending
