---
phase: 163
slug: json-ui-improvements-batch-2-cassa-and-calendario-field-test
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-16
---

# Phase 163 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust workspace) |
| **Config file** | `Cargo.toml` (workspace root); per-crate `Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-json-ui --lib` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~90 seconds (quick), ~6 minutes (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p {affected_crate} --lib`
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green (fmt + clippy + tests)
- **Max feedback latency:** 90 seconds for quick, 360 seconds for full

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Decision | Test Type | Automated Command | File Exists | Status |
|---------|------|------|----------|-----------|-------------------|-------------|--------|
| 163-01-* | 01 | 1 | D-01 ($each struct + serde) | unit | `cargo test -p ferro-json-ui --lib spec::tests::each_directive` | ❌ W0 | ⬜ pending |
| 163-02-* | 02 | 2 | D-03, D-04 ($if struct + accepts full Visibility) | unit | `cargo test -p ferro-json-ui --lib spec::tests::if_directive` | ❌ W0 | ⬜ pending |
| 163-03-* | 03 | 2 | D-01/D-02/D-03/D-04 resolve-time expansion | unit | `cargo test -p ferro-json-ui --lib resolve::tests::expand_directives` | ❌ W0 | ⬜ pending |
| 163-04-* | 04 | 2 | D-12 (validator gates for malformed directives) | unit | `cargo test -p ferro-json-ui --lib spec::tests::validate_directives` | ❌ W0 | ⬜ pending |
| 163-05-* | 05 | 3 | D-06/D-07 SpecBuilder ergonomic layer | unit | `cargo test -p ferro-json-ui --lib builder::tests::ergonomic` | ❌ W0 | ⬜ pending |
| 163-06-* | 06 | 3 | D-13 MCP catalog reflects directives | unit | `cargo test -p ferro-mcp --lib tools::json_ui_catalog::tests::reflects_directives` | ❌ W0 | ⬜ pending |
| 163-07-* | 07 | 4 | D-09/D-10/D-11 ferro-cli codemod | integration | `cargo test -p ferro-cli --test json_ui_migrate_v1` | ❌ W0 | ⬜ pending |
| 163-08-* | 08 | 4 | D-01/D-02/D-03/D-05 E2E directive integration tests | integration | `cargo test -p ferro-json-ui --test directives_e2e` | ❌ W0 | ⬜ pending |
| 163-09-* | 09 | 4 | D-05/D-08 decision rubric docs | docs | `cargo doc --no-deps -p ferro-json-ui && test -f docs/src/json-ui/spec-construction.md` | ❌ W0 | ⬜ pending |
| 163-10-* | 10 | 5 | CHANGELOG entry (no decision; release-prep) | presence | `grep -E '^- (Added\|Changed\|Fixed)' CHANGELOG.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Task IDs are placeholders — the planner assigns real IDs in PLAN.md files.*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/spec.rs` — add `#[cfg(test)] mod tests` for `EachDirective` / `IfDirective` deserialization round-trips
- [ ] `ferro-json-ui/src/resolve.rs` — add `#[cfg(test)] mod tests` covering `expand_directives` pass
- [ ] `ferro-json-ui/tests/directives_e2e.rs` — new integration test file with at least one fixture mirroring a cassa friction site (orders kanban list with `$each`)
- [ ] `ferro-cli/tests/json_ui_migrate_v1.rs` — new integration test file with a real input controller fixture and expected JSON/Rust output snapshots
- [ ] `ferro-mcp/src/tools/json_ui_catalog.rs` — extend test module to assert `$each` and `$if` appear in catalog output
- [ ] Shared fixtures directory if needed: `ferro-json-ui/tests/fixtures/directives/` with one JSON spec per directive variant

---

## Manual-Only Verifications

| Behavior | Decision | Why Manual | Test Instructions |
|----------|----------|------------|-------------------|
| Codemod handles an unrecognized v1 pattern by emitting a `// TODO: codemod could not auto-translate` marker | D-09 | Requires running codemod against a real-world controller with mixed patterns; coverage of every possible v1 pattern is impractical in automated tests | Run `ferro json-ui:migrate-v1 src/controllers/<known-edge-case>.rs --dry-run`; verify the dry-run output includes `// TODO: codemod could not auto-translate` for the non-translatable section, and the rest of the file is rewritten correctly |
| MCP `json_ui_catalog` tool output is agent-readable for `$each` / `$if` directive discovery | D-13 | Requires exercising the MCP server via `claude mcp` or equivalent client; automated test verifies the data, manual test verifies an agent can discover the directives from catalog output | Start `ferro mcp`, call `json_ui_catalog`, verify the response payload includes top-level directive entries for `$each` and `$if` with a docstring and example |
| `docs/src/json-ui/spec-construction.md` decision rubric reads cleanly for an external developer | D-08 | Documentation quality is subjective; automated checks only catch presence and build cleanliness | Read the rendered doc; confirm the four-quadrant rubric (Static / `$each` / `$if` / SpecBuilder) is unambiguous and each quadrant has a worked example |

---

## Coverage by Decision

| Decision | Coverage |
|----------|----------|
| D-01 `$each` directive | Plan 01 (struct) + Plan 03 (resolve expansion) + Plan 08 (e2e) — unit + integration |
| D-02 (covers 3 cassa sites) | Plan 08 fixture mirrors orders kanban list — integration |
| D-03 `$if` conditional emission | Plan 02 (struct) + Plan 03 (resolve deletion) + Plan 08 (e2e) — unit + integration |
| D-04 visibility evaluator reuse | Plan 02 (`$if` accepts full Visibility enum) + Plan 03 (resolver invokes `Visibility::evaluate`, no parallel evaluator) — unit |
| D-05 no `$template` element | Plans contain no `$template` work — coverage is structural; Plan 08 e2e fixtures exercise the post-expansion shape |
| D-06 / D-07 SpecBuilder ergonomic layer | Plan 05 — unit |
| D-08 docs rubric | Plan 09 — docs build + presence check |
| D-09 / D-10 / D-11 codemod | Plan 07 — integration with fixture inputs/outputs |
| D-12 validator errors | Plan 04 — unit per error variant |
| D-13 MCP catalog reflects directives | Plan 06 — unit + manual MCP smoke |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (test modules in spec.rs / resolve.rs, new test files in ferro-json-ui/tests and ferro-cli/tests)
- [ ] No watch-mode flags (all commands one-shot)
- [ ] Feedback latency < 360s (full suite estimate)
- [ ] `nyquist_compliant: true` set in frontmatter (set by planner / executor when all tasks have automated coverage)

**Approval:** pending
