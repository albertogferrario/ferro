---
phase: 148
slug: htmlembed-component-ferro-json-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-24
---

# Phase 148 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

Derived from `148-RESEARCH.md` §Validation Architecture (L1101+). Every requirement ID in
EMBED-01..EMBED-05 maps to at least one automated check below.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` (no external framework) |
| **Config file** | `ferro-json-ui/Cargo.toml`, `ferro-mcp/Cargo.toml` (workspace — no test-specific config) |
| **Quick run command** | `cargo test -p ferro-json-ui --lib` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30-60s quick; ~3-5min full (incremental) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui --lib`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green; ferro-mcp crate tests must be green (catalog exhaustiveness enforced there)
- **Max feedback latency:** ~60s quick; ~300s full

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 148-01-01 | 01 | 0 | EMBED-01 (D-01) | T-148-01 | `HtmlEmbedProps` derives compile; serde tagged round-trip works | unit | `cargo test -p ferro-json-ui html_embed_round_trips` | ❌ W0 | ⬜ pending |
| 148-01-02 | 01 | 0 | EMBED-01 (D-03) | — | `HtmlEmbedProps::new(impl Into<String>)` compiles for `&str` / `String` | unit | `cargo test -p ferro-json-ui html_embed_props_new_constructor` | ❌ W0 | ⬜ pending |
| 148-01-03 | 01 | 0 | EMBED-01 (D-04) | — | `ComponentNode::html_embed(key, props)` factory shape | unit | `cargo test -p ferro-json-ui component_node_html_embed_factory_shape` | ❌ W0 | ⬜ pending |
| 148-01-04 | 01 | 0 | EMBED-04 (fixture) | — | `all_known_types_round_trip` suite covers HtmlEmbed JSON fixture | unit | `cargo test -p ferro-json-ui all_known_types_round_trip` | ❌ W0 | ⬜ pending |
| 148-01-05 | 01 | 0 | EMBED-02 (D-06) | T-148-02 | `<div>{html}</div>` wraps `html` verbatim, no class/id/attrs | unit | `cargo test -p ferro-json-ui render_html_embed_wraps_in_div` | ❌ W0 | ⬜ pending |
| 148-01-06 | 01 | 0 | EMBED-02 (D-06, load-bearing) | T-148-02 | `<script>alert('xss')</script>` emitted unescaped — XSS passthrough contract | unit | `cargo test -p ferro-json-ui render_html_embed_emits_html_verbatim_without_escaping` | ❌ W0 | ⬜ pending |
| 148-01-07 | 01 | 0 | EMBED-02 (D-06) | T-148-02 | HTML entities (`&amp;`) pass through verbatim without double-escape | unit | `cargo test -p ferro-json-ui render_html_embed_preserves_entities_verbatim` | ❌ W0 | ⬜ pending |
| 148-01-08 | 01 | 0 | EMBED-02 (D-06) | T-148-02 | Angle brackets pass through verbatim | unit | `cargo test -p ferro-json-ui render_html_embed_preserves_angle_brackets_verbatim` | ❌ W0 | ⬜ pending |
| 148-01-09 | 01 | 0 | EMBED-02 (D-06, edge case) | — | Empty string renders as `<div></div>` | unit | `cargo test -p ferro-json-ui render_html_embed_empty_string_produces_empty_div` | ❌ W0 | ⬜ pending |
| 148-01-10 | 01 | 0 | EMBED-03 (D-10, pass 1) | — | `resolve_component_node` does not mutate an `HtmlEmbed` node | unit | `cargo test -p ferro-json-ui resolve_component_skips_html_embed` | ❌ W0 | ⬜ pending |
| 148-01-11 | 01 | 0 | EMBED-03 (D-10, pass 2) | — | `collect_unresolved_node` does not add `HtmlEmbed` to unresolved set | unit | `cargo test -p ferro-json-ui collect_unresolved_skips_html_embed` | ❌ W0 | ⬜ pending |
| 148-01-12 | 01 | 0 | EMBED-03 (D-10, pass 3) | — | `resolve_errors_node` does not mutate an `HtmlEmbed` node | unit | `cargo test -p ferro-json-ui resolve_errors_skips_html_embed` | ❌ W0 | ⬜ pending |
| 148-01-13 | 01 | 0 | EMBED-04 (D-19) | — | Exhaustive-list assertion bumped 41→42; `"HtmlEmbed"` in expected array | unit | `cargo test -p ferro-mcp json_ui_catalog::tests::test_all_components_present` | ❌ W0 | ⬜ pending |
| 148-02-01 | 02 | 1 | EMBED-01 (D-01, D-02) | — | `HtmlEmbedProps` + `Component::HtmlEmbed` variant + serde arms land in `component.rs` | build | `cargo build -p ferro-json-ui` | ❌ W0 | ⬜ pending |
| 148-02-02 | 02 | 1 | EMBED-01 (D-03, D-04) | — | `HtmlEmbedProps::new` and `ComponentNode::html_embed` factory land | unit | Tests from Wave 0 now pass | ❌ W0 | ⬜ pending |
| 148-03-01 | 03 | 1 | EMBED-02 (D-05..D-08) | T-148-02 | `render_html_embed` + dispatch arm + leaf arm in `collect_plugin_types_node` | unit | Wave-0 render tests now pass | ❌ W0 | ⬜ pending |
| 148-04-01 | 04 | 1 | EMBED-03 (D-10, D-11) | — | `Component::HtmlEmbed(_)` joins leaf OR-chain in all three resolver passes | build + unit | `cargo build -p ferro-json-ui && cargo test -p ferro-json-ui resolve` | ❌ W0 | ⬜ pending |
| 148-05-01 | 05 | 1 | EMBED-04 (D-17..D-20) | — | MCP `CatalogComponent` entry + `test_components_have_props` still passes (has 1 required prop) | unit | `cargo test -p ferro-mcp json_ui_catalog::tests` | ❌ W0 | ⬜ pending |
| 148-05-02 | 05 | 1 | EMBED-04 (D-16) | — | `COMPONENT_CATALOG` const contains `### HtmlEmbed` section with safety language | unit | `cargo test -p ferro-json-ui component_catalog_lists_html_embed` (new, trivial) | ❌ W0 | ⬜ pending |
| 148-05-03 | 05 | 1 | EMBED-04 (D-21) | — | `docs/src/json-ui/components.md` has `### HtmlEmbed` section with safety callout | manual + grep | `grep -q '^### HtmlEmbed' docs/src/json-ui/components.md` | ❌ W1 | ⬜ pending |
| 148-05-04 | 05 | 1 | EMBED-05 (CI gate) | — | Full CI gate green | integration | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | N/A (gate) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Task ID convention (pending planner finalization):** `148-{plan}-{task}` where plan numbers match the wave decomposition in CONTEXT D-23 (`148-01` = Wave 0 RED, `148-02..05` = Wave 1 impl split by file area). The planner may split or merge tasks; IDs above are indicative.

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/component.rs` — add `html_embed_round_trips`, `html_embed_props_new_constructor`, `component_node_html_embed_factory_shape` tests under the existing `mod tests`; append `("HtmlEmbed", r#"{"type":"HtmlEmbed","html":"<svg/>"}"#)` fixture to `all_known_types_round_trip`.
- [ ] `ferro-json-ui/src/render.rs` — add 5 `render_html_embed_*` tests (wraps_in_div, empty_string, preserves_entities, preserves_angle_brackets, emits_html_verbatim_without_escaping) following existing substring-assertion convention.
- [ ] `ferro-json-ui/src/resolve.rs` — add 3 resolver no-op tests (`resolve_component_skips_html_embed`, `collect_unresolved_skips_html_embed`, `resolve_errors_skips_html_embed`).
- [ ] `ferro-mcp/src/tools/json_ui_catalog.rs` — bump exhaustive-list assertion L1212 from `41` to `42` and append `"HtmlEmbed"` to the `expected` array at L1218-1260 (lands RED in Wave 0; CatalogComponent entry in Wave 1 turns it GREEN).
- [ ] `ferro-json-ui/src/lib.rs` — optional `component_catalog_lists_html_embed` assertion if an existing `mod tests` exists (else skip — grep-verified in code review).

No framework installation needed — `cargo test` and the CI gate command are already the workspace standard.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `### HtmlEmbed` docs section has visible safety callout (not just any prose) | EMBED-04 (D-21) | No automated doc-test infrastructure for `docs/src/`; callout styling matches sibling docs | Open `docs/src/json-ui/components.md`, confirm `### HtmlEmbed` section exists, contains a blockquote or `> ⚠` warning block, explicitly names XSS / caller-responsibility; `grep -E '^### HtmlEmbed' docs/src/json-ui/components.md` returns 1 hit. |
| Rustdoc on `HtmlEmbedProps` renders with prominent safety paragraph at the top | EMBED-01 (D-15) | `cargo doc --no-deps` is not in CI; prose quality is a code-review gate | `cargo doc --no-deps -p ferro-json-ui --open` → navigate to `HtmlEmbedProps` → confirm first paragraph is the safety contract (verbatim, XSS, caller-owned-safety). |
| Inline comment in `render_html_embed` body flags deliberate `html_escape` omission | EMBED-02 / maintainability dimension 8 | No automated check for inline comments; future "fix" guard | Read `ferro-json-ui/src/render.rs` `fn render_html_embed`; confirm a `// ` line explicitly notes "deliberate — do not add html_escape" or equivalent. |

---

## 8-Dimension Validation Map

> See `148-RESEARCH.md` §Validation Architecture L1141-1155 for full dimension-by-dimension rationale.

| Dimension | Coverage | Evidence |
|-----------|----------|----------|
| 1. Structural correctness | Compile-time | `cargo build -p ferro-json-ui` — three `match` statements in resolve.rs become non-exhaustive if HtmlEmbed variant is added without leaf-chain updates |
| 2. Behavioral correctness | 5 render tests | `render_html_embed_*` suite in render.rs |
| 3. Semantic correctness (unescaped intent) | 1 load-bearing XSS passthrough test | `render_html_embed_emits_html_verbatim_without_escaping` |
| 4. Integration correctness | serde + MCP catalog + all_known_types_round_trip | `html_embed_round_trips`, catalog exhaustive-list test, fixture append |
| 5. Security correctness (XSS intent documented) | Same test as dim 3 + 4 reinforcing artifacts | XSS test doubles as security contract; `HtmlEmbedProps` rustdoc, `COMPONENT_CATALOG`, MCP catalog, docs safety callout |
| 6. Performance correctness | N/A | Single `format!` call; O(n) in string length; no benchmarks |
| 7. Observability correctness | N/A | Pure function, no side effects |
| 8. Maintainability correctness | 3 reinforcing artifacts | Inline comment in render body + rustdoc + XSS test's reversed assertion direction |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (render tests, resolver tests, serde tests, catalog exhaustive-list bump)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s (quick) / < 300s (full)
- [ ] `nyquist_compliant: true` set in frontmatter once Wave 0 lands

**Approval:** pending (draft — finalize after planner produces PLAN.md files)
