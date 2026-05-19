---
phase: 116
slug: flat-element-renderer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-18
---

# Phase 116 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `116-RESEARCH.md` §"Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[cfg(test)]` + `cargo test` (ferro workspace convention) |
| **Config file** | `Cargo.toml` per crate (no separate test runner) |
| **Quick run command** | `cargo test -p ferro-json-ui --lib` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~15s quick (lib-only); ~90s full (workspace CI parity) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui --lib`
- **After every plan wave:** Run `cargo test --all-features -p ferro-json-ui -p ferro`
- **Before `/gsd-verify-work`:** Full suite must be green (`cargo fmt --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features`)
- **Max feedback latency:** 90 seconds (full suite)

---

## Per-Task Verification Map

> Mapped against the 6 ROADMAP success criteria (SC-1…SC-6). Task IDs are provisional — the planner refines them when slicing plans.

| Task ID | Plan | Wave | Success Criterion | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 116-01-01 | 01 | 1 | Prereq (slot fields) | — | N/A | unit | `cargo test -p ferro-json-ui --lib component::schema_smoke_tests` | ❌ W0 (adds fields) | ⬜ pending |
| 116-01-02 | 01 | 1 | SC-4 (visibility eval) | — | visible=false renders nothing (no data leak) | unit | `cargo test -p ferro-json-ui --lib visibility::tests` | ❌ W0 (adds `evaluate`) | ⬜ pending |
| 116-02-01 | 02 | 2 | SC-1/5/6 scaffolding | — | Unknown type_name → comment, no panic; no `render_to_html` symbol | unit + grep | `cargo test -p ferro-json-ui --lib render::mod::tests && ! grep -rn 'render_to_html\b' ferro-json-ui/src framework/src` | ❌ W0 (new `render/mod.rs`) | ⬜ pending |
| 116-02-02 | 02 | 2 | SC-2 (missing child) | — | Missing child ID → HTML comment diagnostic | unit | `cargo test -p ferro-json-ui --lib walker_missing_child` | ❌ W0 | ⬜ pending |
| 116-02-03 | 02 | 2 | SC-3 (action URL) | — | Unresolved handler → `href="#"` + diagnostic | unit | `cargo test -p ferro-json-ui --lib walker_action_url_inlined walker_action_url_unresolved` | ❌ W0 | ⬜ pending |
| 116-02-04 | 02 | 2 | SC-4 (visibility gate) | — | Invisible root → `<!-- root hidden -->` only | unit | `cargo test -p ferro-json-ui --lib walker_root_hidden walker_visible_hides_element walker_visible_hides_children` | ❌ W0 | ⬜ pending |
| 116-02-05 | 02 | 2 | SC-5 (plugin dispatch) | — | Plugin render path exercised; assets collected | unit | `cargo test -p ferro-json-ui --lib walker_plugin_dispatch walker_plugin_asset_collection` | ❌ W0 | ⬜ pending |
| 116-03-* | 03 | 3 | SC-1 (atoms) | — | Every atom emits expected HTML markers | unit | `cargo test -p ferro-json-ui --lib render::atoms::tests` | ❌ W0 (ports v1 tests) | ⬜ pending |
| 116-04-* | 04 | 3 | SC-1 (containers) | — | Containers recurse via `render_element` and honor slot fields | unit | `cargo test -p ferro-json-ui --lib render::containers::tests` | ❌ W0 (ports v1 tests) | ⬜ pending |
| 116-05-* | 05 | 3 | SC-1 (form + data) | — | Form/Data renderers consume `data::resolve_path_string` correctly; `#[allow(dead_code)]` removed | unit + clippy | `cargo test -p ferro-json-ui --lib render::form::tests render::data::tests && cargo clippy -p ferro-json-ui --all-targets -- -D warnings` | ❌ W0 (ports v1 tests) | ⬜ pending |
| 116-06-01 | 06 | 4 | All SCs (framework wiring) | — | `framework/src/json_ui/mod.rs` tests assert real HTML markers, not placeholder | unit + integration | `cargo test -p ferro --lib json_ui::mod::tests` | ⚠️ existing tests assert placeholder (must rewrite) | ⬜ pending |
| 116-06-02 | 06 | 4 | Phase gate | — | Workspace clean | full CI | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` | ✅ (always runs) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/visibility.rs` — add `impl Visibility { pub fn evaluate(&self, data: &Value) -> bool }` + `evaluate_condition` covering all 11 `VisibilityOperator` variants (blocks SC-4).
- [ ] `ferro-json-ui/src/component.rs` — re-add 5 `Vec<String>` slot fields (CardProps.footer, ModalProps.footer, Tab.children, KanbanColumnProps.children, PageHeaderProps.actions) per CONTEXT D-06 (blocks SC-1 multi-slot coverage).
- [ ] `ferro-json-ui/src/render/mod.rs` — NEW file (replaces current `src/render.rs`). Public API, dispatch match, `BUILTIN_TYPES`, `render_element`, HTML helpers, plugin asset collection.
- [ ] `ferro-json-ui/src/render/atoms.rs` — NEW file (~950 LOC, 22 leaf renderers).
- [ ] `ferro-json-ui/src/render/containers.rs` — NEW file (~460 LOC, 9 containers).
- [ ] `ferro-json-ui/src/render/form.rs` — NEW file (Form/Input/Select/Checkbox/Switch).
- [ ] `ferro-json-ui/src/render/data.rs` — NEW file (Table/DataTable/DescriptionList/Pagination).
- [ ] `ferro-json-ui/src/data.rs` — drop `#[allow(dead_code)]` on `resolve_path` and `resolve_path_string` once consumed by the renderers.
- [ ] `framework/src/json_ui/mod.rs` — update integration tests to assert real HTML markers instead of the Phase 115 placeholder marker; un-`#[ignore]` the Leaflet plugin test if applicable.

*If planning reveals that component.rs slot-field additions should not ship in the same plan as `Visibility::evaluate`, the planner may split 116-01 into two sub-plans — both still Wave 1.*

---

## Manual-Only Verifications

| Behavior | Success Criterion | Why Manual | Test Instructions |
|----------|-------------------|------------|-------------------|
| Visual parity with v1 for sample `app/` pages | SC-1 (all built-ins render) | HTML string assertions can miss pixel-level regressions (spacing, Tailwind class order); full visual inspection belongs to Phase 121 gestiscilo field test | Run `cargo run -p app` (if runnable), load sample page, compare against pre-Phase-115 screenshots if any exist. Non-blocking for Phase 116 completion — the byte-level HTML assertions in ported v1 tests are the enforceable contract. |
| COMPONENT_CATALOG string accuracy for re-added slot fields | HIGH-1 risk mitigation (research) | `COMPONENT_CATALOG` is a documentation string consumed by AI tools — no automated test verifies its content matches actual Props shape | Diff `ferro-json-ui/src/lib.rs` lines 88–168 against the slot field additions in 116-01. Alberto's call at plan time whether to fix inline or defer to Phase 117. |

---

## Validation Sign-Off

- [ ] All tasks have automated verify commands or Wave 0 dependencies listed
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING file references (visibility.evaluate, render/*, slot fields)
- [ ] No watch-mode flags (cargo test is one-shot)
- [ ] Feedback latency < 90s (full CI-parity suite)
- [ ] `nyquist_compliant: true` set in frontmatter once plans are written

**Approval:** pending
