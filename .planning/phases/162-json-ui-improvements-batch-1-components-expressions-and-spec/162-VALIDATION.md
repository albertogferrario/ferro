---
phase: 162
slug: json-ui-improvements-batch-1-components-expressions-and-spec
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-16
---

# Phase 162 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | none (workspace `Cargo.toml`) |
| **Quick run command** | `cargo test -p ferro-json-ui --all-features` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~90 seconds (quick), ~6 minutes (full suite with clippy) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui --all-features 2>&1 | tail -5`
- **After every plan wave:** Run full suite (`cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --all-features`)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds (quick); 360 seconds (full wave gate)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Decision Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|--------------|-----------------|-----------|-------------------|-------------|--------|
| 162-W1-01 | 01 | 1 | D-01/D-02 | CheckboxList renders one `<input type="checkbox" name=field value=opt.value>` per option | unit | `cargo test -p ferro-json-ui render_checkbox_list` | ❌ W1 | ⬜ pending |
| 162-W1-02 | 01 | 1 | D-01/D-02 | CheckboxList `selected_path` resolves to `Vec<String>` and pre-checks matching options | unit | `cargo test -p ferro-json-ui checkbox_list_selected_path` | ❌ W1 | ⬜ pending |
| 162-W1-03 | 01 | 1 | D-01/D-02 | `CheckboxListProps` JSON Schema generates with `$defs/SelectOption` ref | unit | `cargo test -p ferro-json-ui schema_for_checkbox_list` | ❌ W1 | ⬜ pending |
| 162-W1-04 | 01 | 1 | D-21 | `BUILTIN_TYPES` length == 41 in `render/mod.rs` | unit | `cargo test -p ferro-json-ui builtin_types_count` | ✅ (update assertion) | ⬜ pending |
| 162-W1-05 | 01 | 1 | D-21 | MCP `json_ui_catalog` exhaustive list assertion accepts 41 components | unit | `cargo test -p ferro-mcp test_all_components_present` | ✅ (update assertion) | ⬜ pending |
| 162-W1-06 | 02 | 1 | D-03/D-04 | DataTable URL template substitutes any column key (`{label}`, `{slug_path}`) | unit | `cargo test -p ferro-json-ui data_table_url_template_replaces_column_key` | ❌ W1 | ⬜ pending |
| 162-W1-07 | 02 | 1 | D-03/D-04 | Missing key leaves placeholder text unsubstituted (no panic) | unit | `cargo test -p ferro-json-ui data_table_url_template_missing_key` | ❌ W1 | ⬜ pending |
| 162-W1-08 | 02 | 1 | D-04 | Existing `{row_key}` and `{id}` substitutions still work after generalization | unit | `cargo test -p ferro-json-ui data_table_row_href_legacy_placeholders` | ✅ (regression guard) | ⬜ pending |
| 162-W1-09 | 03 | 1 | D-16 | `SwitchProps { compact: true }` emits `scale-75` CSS class | unit | `cargo test -p ferro-json-ui switch_compact_adds_scale_class` | ❌ W1 | ⬜ pending |
| 162-W1-10 | 03 | 1 | D-17 | Inline SVG renders verbatim with required alt text (no `<img>` tag) | unit | `cargo test -p ferro-json-ui image_inline_svg_renders_svg` | ❌ W1 | ⬜ pending |
| 162-W1-11 | 03 | 1 | D-17 | `ImageProps::inline_svg(svg, alt)` factory compiles and round-trips through serde | unit | `cargo test -p ferro-json-ui image_inline_svg_factory` | ❌ W1 | ⬜ pending |
| 162-W1-12 | 04 | 1 | D-18/D-21 | RichTextEditor plugin registers via `JsonUiPlugin` trait, injects Quill 2.0.3 assets | unit | `cargo test -p ferro-json-ui rich_text_editor_plugin_registers` | ❌ W1 | ⬜ pending |
| 162-W2-01 | 05 | 2 | D-05 | AuthLayout HTML output contains no `bg-card rounded-lg shadow-md` wrapper | unit | `cargo test -p ferro-json-ui auth_layout_centers_content` | ✅ (update assertion) | ⬜ pending |
| 162-W2-02 | 06 | 2 | D-07 | `Spec::from_json` returns `Err(SpecError::FooterMissing)` when footer references unknown ID | unit | `cargo test -p ferro-json-ui from_json_rejects_missing_footer_id` | ❌ W2 | ⬜ pending |
| 162-W2-03 | 06 | 2 | D-08 | Spec with duplicate ID in both `props.footer` and `children` emits warning via tracing | unit | `cargo test -p ferro-json-ui spec_warns_duplicate_footer_child` | ❌ W2 | ⬜ pending |
| 162-W3-01 | 07 | 3 | D-11/D-12 | `AlertVariant::Success.as_ref()` returns `"success"` (snake_case serialize_all) | unit | `cargo test -p ferro-json-ui alert_variant_as_ref_str` | ❌ W3 | ⬜ pending |
| 162-W3-02 | 07 | 3 | D-11 | All six variant enums round-trip through `as_ref` → `from_str` → serde JSON | unit | `cargo test -p ferro-json-ui variant_enums_roundtrip` | ❌ W3 | ⬜ pending |
| 162-W3-03 | 08 | 3 | D-09 | `json_ui_verify_action { handler: "valid_route" }` returns `Ok(RouteInfo)` | unit | `cargo test -p ferro-mcp json_ui_verify_action_found` | ❌ W3 | ⬜ pending |
| 162-W3-04 | 08 | 3 | D-09 | `json_ui_verify_action { handler: "typoed_route" }` returns `Err(NotFound)` with Levenshtein candidate | unit | `cargo test -p ferro-mcp json_ui_verify_action_not_found_suggests_closest` | ❌ W3 | ⬜ pending |
| 162-W3-05 | 09 | 3 | D-20 | `docs/src/json-ui/migration-v1-to-v2.md` exists and is linked from `docs/src/SUMMARY.md` | unit | `mdbook build docs/ 2>&1 \| grep -q 'migration-v1-to-v2'` | ❌ W3 | ⬜ pending |
| 162-W3-06 | 09 | 3 | D-22 | MCP `code_templates` returns 7 migration templates matching D-20 sections | unit | `cargo test -p ferro-mcp code_templates_returns_migration_patterns` | ❌ W3 | ⬜ pending |
| 162-W4-01 | 10 | 4 | All | Full suite green: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | integration | (full suite command) | — | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Wave 0 is the test-stub generation that must happen before any production code lands. For this phase, Wave 0 is folded into Wave 1 because the test framework (`cargo test`) is already present and existing test files (`render/data.rs`, `render/form.rs`, `spec.rs`, `layout.rs`) are the homes for new tests.

- [ ] `ferro-json-ui/Cargo.toml` — add `strum = { version = "0.26", features = ["derive"] }` (Wave 1 — required by D-11 and unblocks variant test stubs)
- [ ] `ferro-mcp/Cargo.toml` — add `strsim = "0.11"` (Wave 3 — required by D-09)
- [ ] `ferro-json-ui/src/render/form.rs` — `render_checkbox_list` function + test stubs (Wave 1)
- [ ] `ferro-json-ui/src/component.rs` — `CheckboxListProps` struct + serde + schemars derives (Wave 1)
- [ ] `ferro-json-ui/src/spec.rs` — `SpecError::FooterMissing` variant + `validate_footer_ids` + test stubs (Wave 2)
- [ ] `ferro-mcp/src/tools/json_ui_verify_action.rs` — new file with stub returning `Err(NotFound)` (Wave 3)

---

## Manual-Only Verifications

| Behavior | Decision | Why Manual | Test Instructions |
|----------|----------|------------|-------------------|
| Gestiscilo per-row-actions confirmation | D-03/D-04 | Bidirectional adaptation gate — confirm with gestiscilo author that per-row actions belong on list pages before shipping interpolation | Open gestiscilo repo, grep `row_actions` usage in `cassa/` and `dashboard/pagine`; confirm with phase 138 author that the v2-native pattern (detail-page navigation) was rejected in favor of in-place actions |
| AuthLayout consumer audit | D-05 | Breaking change — ensure all gestiscilo specs using `"layout": "auth"` already declare `Card` as root | `grep -r '"layout": "auth"' /Users/alberto/repositories/gestiscilo-it/app/src/views/` and confirm each spec has `"root": "card_*"` or equivalent Card root |
| `migration-v1-to-v2.md` worked-example accuracy | D-20 | Doc page accuracy depends on consumer migration experience | Have the gestiscilo Phase 139+ author read the page and confirm each section maps to a real migration pain point they hit |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies declared
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (strum, strsim, render_checkbox_list, etc.)
- [ ] No watch-mode flags (cargo test is one-shot)
- [ ] Feedback latency < 90s for quick, < 360s for full wave gate
- [ ] `nyquist_compliant: true` set in frontmatter after Wave 1 planning is complete

**Approval:** pending
