---
phase: 160
slug: remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-05-17
---

# Phase 160 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Phase 160 is a deletion/cleanup phase; validation is dominated by **grep gates** and **cargo invocations**, not new unit tests.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust workspace; existing harness — no new framework introduced) |
| **Config file** | `Cargo.toml` workspace + per-crate `Cargo.toml` (none added by this phase) |
| **Quick run command** | `cargo test -p ferro-mcp --lib` (targeted at the only crate gaining code changes vs deletions) |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | quick ~30s; full suite ~5-10min (workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo build -p {crate_touched}` to confirm the touched crate still compiles.
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
- **Before `/gsd-verify-work`:** Full suite must be green, plus the D-10 grep gate must report zero matches.
- **Max feedback latency:** quick check < 60s; full wave gate < 10min.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Decision | Secure Behavior | Test Type | Automated Command | Status |
|---------|------|------|----------|-----------------|-----------|-------------------|--------|
| 160-01-1 | 01 | 1 | D-02 / D-03 / Pattern-1 | Doc-comment rewrites in render/mod.rs, atoms.rs, projection/builder.rs, layout.rs fixture — no new attack surface (T-160-01/02/03 accept) | grep + cargo | `cargo fmt --all -- --check && cargo clippy -p ferro-json-ui --all-targets -- -D warnings && cargo test -p ferro-json-ui --all-features && ! grep -rnE 'Port of v1\|Differences from v1\|ported verbatim from v1\|ported from v1\|Replaces v1\|Phase 116\|Per CONTEXT D-' ferro-json-ui/src/render/mod.rs ferro-json-ui/src/render/atoms.rs && ! grep -n 'Silence unused-import warnings until Plan 03' ferro-json-ui/src/projection/builder.rs && ! grep -n '"schema":"v1"' ferro-json-ui/src/layout.rs` | ⬜ pending |
| 160-01-2 | 01 | 1 | D-02 / D-03 / Pattern-1 | Doc-comment rewrites in render/containers.rs and render/form.rs — no new attack surface (T-160-01 accept) | grep + cargo | `cargo fmt --all -- --check && cargo clippy -p ferro-json-ui --all-targets -- -D warnings && cargo test -p ferro-json-ui --all-features && ! grep -nE 'Port of v1\|Differences from v1\|ported verbatim from v1\|ported from v1\|Replaces v1\|Phase 116\|Per CONTEXT D-\|render\.rs L[0-9]' ferro-json-ui/src/render/containers.rs ferro-json-ui/src/render/form.rs` | ⬜ pending |
| 160-01-3 | 01 | 1 | D-03 / Pattern-1 / Pattern-8 | Doc-comment rewrites in render/data.rs — no new attack surface (T-160-01 accept) | grep + cargo | `cargo fmt --all -- --check && cargo clippy -p ferro-json-ui --all-targets -- -D warnings && cargo test -p ferro-json-ui --all-features && ! grep -nE 'Port of v1\|Matches v1\|\(v1 verbatim\)\|\(v1 fallback\)\|Legacy \{row_key\}\|Phase 116\|Per CONTEXT D-\|render\.rs:[0-9]' ferro-json-ui/src/render/data.rs` | ⬜ pending |
| 160-02-1 | 02 | 1 | D-04 / Pattern-3 | `migration_v1_to_v2_templates` deletion — pure surface-area reduction, no I/O, no untrusted input (T-160-04 accept) | grep + cargo | `! grep -n 'migration_v1_to_v2' ferro-mcp/src/tools/code_templates.rs && cargo fmt --all -- --check && cargo clippy -p ferro-mcp --all-targets -- -D warnings && cargo test -p ferro-mcp --all-features --lib code_templates` | ⬜ pending |
| 160-03-1 | 03 | 1 | D-05 / Pattern-2 | `scan_json_ui_specs` rewrite preserves project-relative `views_dir_display = "src/views/"` (no absolute-path leakage; T-160-05 mitigate). `Path::join` traversal pattern unchanged from v1 scanner (T-160-06 accept). | unit + grep | `cargo fmt --all -- --check && cargo clippy -p ferro-mcp --all-targets -- -D warnings && cargo test -p ferro-mcp --all-features --lib application_info && ! grep -n 'Scans for legacy v1 patterns' ferro-mcp/src/tools/application_info.rs && ! grep -n 'TODO(Phase 120)' ferro-mcp/src/tools/application_info.rs` | ⬜ pending |
| 160-04-1 | 04 | 1 | D-06 / Pattern-4 | Test-fixture filename rename in json_ui_inspect.rs — pure string rename in test fixture, no production code change (T-160-07 accept) | unit + grep | `cargo fmt --all -- --check && cargo clippy -p ferro-mcp --all-targets -- -D warnings && cargo test -p ferro-mcp --all-features --lib json_ui_inspect::tests::test_ignores_non_json_files && ! grep -n 'old_view.rs\|old v1 file\|pub mod old' ferro-mcp/src/tools/json_ui_inspect.rs` | ⬜ pending |
| 160-05-1 | 05 | 1 | D-08 / Pattern-6 | ferro-json-ui/README.md rewrite — pure documentation, no production-code path touched (T-160-08 accept) | grep | `! grep -nE '\b(JsonUiView\|ComponentNode\|LayoutComponent)\b' ferro-json-ui/README.md && ! grep -n 'view\.into_response' ferro-json-ui/README.md && grep -q 'Spec::builder' ferro-json-ui/README.md && grep -q 'JsonUi::render_file' ferro-json-ui/README.md && grep -q '41 built-in components' ferro-json-ui/README.md` | ⬜ pending |
| 160-06-1 | 06 | 1 | D-07 / Pattern-5 | docs/protocol/src/terminology.md Renderer-definition rewrite — pure prose, no code path touched (T-160-09 accept) | grep | `! grep -n 'ferro-json-ui/v1' docs/protocol/src/terminology.md && grep -q 'ferro-json-ui/v2' docs/protocol/src/terminology.md && grep -q 'Spec.*conforming to' docs/protocol/src/terminology.md` | ⬜ pending |
| 160-06-2 | 06 | 1 | D-07 / Pattern-5 | docs/protocol/src/architecture.md JsonUiRenderer bullet rewrite — pure prose, no code path touched (T-160-09 accept) | grep | `! grep -n 'ferro-json-ui/v1' docs/protocol/src/architecture.md && grep -q 'ferro-json-ui/v2' docs/protocol/src/architecture.md && grep -q 'flat ID-keyed element map' docs/protocol/src/architecture.md` | ⬜ pending |
| 160-06-3 | 06 | 1 | D-07 / Pattern-5 | docs/protocol/src/rendering.md Output Format section rewrite — pure prose, no code path touched (T-160-09 accept) | grep | `! grep -n 'ferro-json-ui/v1' docs/protocol/src/rendering.md && grep -q 'ferro-json-ui/v2' docs/protocol/src/rendering.md && grep -q '`elements` map keyed by ID' docs/protocol/src/rendering.md && ! grep -nE 'schema.*version.*title.*body' docs/protocol/src/rendering.md` | ⬜ pending |
| 160-07-1 | 07 | 1 | D-07 / Pattern-5 | docs/src/features/projections.md rewrite to match projection/mod.rs rustdoc — documentation correctness fix, no new attack surface (T-160-10 accept) | grep | `! grep -n 'ferro-json-ui/v1' docs/src/features/projections.md && ! grep -n 'RenderContext::default' docs/src/features/projections.md && ! grep -n 'json\["components"\]' docs/src/features/projections.md && grep -q 'VisualContext::default' docs/src/features/projections.md && grep -q 'spec.elements' docs/src/features/projections.md && grep -q 'ferro-json-ui/v2' docs/src/features/projections.md` | ⬜ pending |
| 160-08-1 | 08 | 1 | D-08 / Pattern-7 | docs/src/reference/cli.md make:json-view Generated-file block rewrite — documentation correctness fix, aligns docs with actual CLI output, no new attack surface (T-160-11 accept) | grep | `! grep -nE '\b(JsonUiView\|ComponentNode\|TableProps\|TextElement\|TextProps\|Component::Text)\b' docs/src/reference/cli.md && grep -q 'src/views/user_index.json' docs/src/reference/cli.md && grep -q 'JsonUi::render_file' docs/src/reference/cli.md && grep -q '"ferro-json-ui/v2"' docs/src/reference/cli.md` | ⬜ pending |
| 160-09-1 | 09 | 2 | D-08 | D-08 sweep + audit report — pure documentation audit; any in-place fixes follow same accept profile as Plans 01-08 (T-160-12 accept) | grep | `test -f .planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-09-AUDIT-D08.md && grep -q 'FAIL count == 0\|FAIL: 0' .planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-09-AUDIT-D08.md` | ⬜ pending |
| 160-10-1 | 10 | 3 | D-09 / D-10 / D-11 | D-10 grep gates + ferro workspace checks + gestiscilo cross-repo build; ferro-code descope explicitly recorded (T-160-13/14/15 accept; no silent gap) | grep + cargo | `test -f .planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-VERIFICATION.md && grep -q 'Verdict.*PASS' .planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-VERIFICATION.md && ! grep -rnE '\b(JsonUiView\|ComponentNode\|PluginProps)\b' ferro-json-ui/src framework/src ferro-mcp/src && ! grep -rn 'ferro-json-ui/v1' ferro-json-ui/src framework/src ferro-mcp/src docs/src docs/protocol/src && ! grep -n 'migration_v1_to_v2_templates' ferro-mcp/src/tools/code_templates.rs && test ! -f docs/src/json-ui/migration-v1-to-v2.md && cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

None. Phase 160 introduces **no new code paths** that would need stub coverage; it deletes and rewrites existing surface. Existing `cargo test --all-features` covers the regression surface.

- [x] Existing infrastructure covers all phase verification needs.

---

## Manual-Only Verifications

| Behavior | Decision | Why Manual | Test Instructions |
|----------|----------|------------|-------------------|
| Doc-comment rewrites read as neutral prose (no historical narrative) | D-02 / D-03 | Style/tone judgment beyond what grep can catch (a comment could be free of `v1` keyword while still being a port narrative — e.g., "Differences from previous version") | Read each rewritten doc-comment block; confirm it describes function purpose + props + output, no historical/comparative framing |
| Public-doc rewrites read as neutral prose | D-07 | Same — `mdbook build` succeeding does not prove the prose is in the right voice | Read `docs/protocol/src/{terminology,architecture,rendering}.md` and `docs/src/features/projections.md` post-rewrite; apply the CLAUDE.md "trigger phrases for review" checklist |
| gestiscilo end-to-end smoke (post-cargo-test browser sanity) | D-09 | A green `cargo test` does not exercise the live JSON-UI render path | Start gestiscilo server, hit 2-3 key dashboard routes, confirm rendering matches Phase 159 verification expectations |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references — N/A, no Wave 0 needed
- [x] No watch-mode flags
- [x] Feedback latency < 60s for quick checks (grep gates resolve in ms; targeted `cargo test -p <crate> --lib <module>` runs complete in seconds)
- [x] `nyquist_compliant: true` set in frontmatter; per-task verification map populated with concrete IDs from Plans 01-10
- [ ] Neutral-prose manual review of rewritten doc comments (per Manual-Only Verifications table) — requires human sign-off
- [ ] gestiscilo end-to-end browser smoke after Plan 10 cross-repo build (per Manual-Only Verifications table) — requires human sign-off

**Approval:** pending
