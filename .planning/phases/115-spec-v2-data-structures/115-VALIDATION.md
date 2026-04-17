---
phase: 115
slug: spec-v2-data-structures
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-18
---

# Phase 115 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` (unit + integration tests) |
| **Config file** | `ferro-json-ui/Cargo.toml` (existing) |
| **Quick run command** | `cargo test -p ferro-json-ui --lib` |
| **Full suite command** | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings && cargo fmt --all -- --check` |
| **Estimated runtime** | ~30 seconds (quick), ~90 seconds (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui --lib`
- **After every plan wave:** Run `cargo test --all-features` + clippy
- **Before `/gsd-verify-work`:** Full suite must be green — fmt + clippy + test all features
- **Max feedback latency:** 30 seconds for unit tests; 90 seconds for full suite

---

## Per-Task Verification Map

Planner filled this table 2026-04-18 based on the 5-plan / 4-wave decomposition. Wave 0 artifacts (test fixtures under `ferro-json-ui/tests/fixtures/`, `tests/round_trip.rs`, `tests/reject.rs`) are produced in Plan 01 Task 2 — no separate wave is needed.

Updated 2026-04-18 (post plan-checker review): added row 115-02-03 for the new schema-smoke test task (Plan 02 Task 3) that satisfies D-32's runtime contract per plan-checker Blocker 1.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 115-01-01 | 01 | 1 | SPEC-01, SPEC-02, SPEC-03 | T-115-01, T-115-02, T-115-03, T-115-04 | Spec and Element types round-trip; structural validation rejects deeply nested / cyclic / duplicate-ID / malformed input without panicking | unit | `cargo test -p ferro-json-ui --lib spec::` | ❌ W0 (creates spec.rs) | ⬜ pending |
| 115-01-02 | 01 | 1 | SPEC-03 | T-115-01, T-115-02, T-115-03 | Fixture round-trip + variant-specific rejection contract | integration | `cargo test -p ferro-json-ui --test round_trip && cargo test -p ferro-json-ui --test reject` | ❌ W0 (creates fixtures + tests) | ⬜ pending |
| 115-02-01 | 02 | 2 | SPEC-04 | — | v1 types (`JsonUiView`, `Component`, `ComponentNode`, `PluginProps`) removed from ferro-json-ui source; surviving Props derive JsonSchema | build + grep | `cargo build -p ferro-json-ui --lib && ! grep -rE "JsonUiView\|ComponentNode" ferro-json-ui/src/` | ✅ | ⬜ pending |
| 115-02-02 | 02 | 2 | SPEC-04 | T-115-06 | Placeholder renderer escapes HTML; resolve walks flat element map; projection Output = Spec | unit + build | `cargo test -p ferro-json-ui && cargo build -p ferro-json-ui --all-targets --all-features` | ✅ | ⬜ pending |
| 115-02-03 | 02 | 2 | SPEC-04 (SPEC-04b — D-32 runtime contract) | — | Every surviving `*Props` struct has a runtime `schema_for!` smoke test that asserts the generated JSON Schema is a non-empty JSON object. Satisfies D-32 at runtime — compile-time `#[derive(JsonSchema)]` alone does not prove the generated code executes without panic (plan-checker Blocker 1). | unit (runtime) | `cargo test -p ferro-json-ui --lib schema_for_ 2>&1 \| grep -qE "^test result: ok\. [0-9]+ passed"` (and `! grep -q "FAILED"`) | ✅ (added by Plan 02 Task 3 as inline `mod schema_smoke_tests` in component.rs per D-26) | ⬜ pending |
| 115-03-01 | 03 | 3 | SPEC-04 | T-115-09 | Framework `JsonUi::render(&Spec, ...)` signature migrated; re-exports updated | build | `cargo build -p framework --lib` | ✅ | ⬜ pending |
| 115-03-02 | 03 | 3 | SPEC-04 | — | Framework inline tests ported to Spec::builder; plugin tests `#[ignore]`d with Phase 116 TODO. Task action now instructs incremental `cargo check -p framework --tests` sampling after every ~10 ported tests (plan-checker Warning 2). | unit + build | `cargo test -p framework json_ui && cargo clippy -p framework --all-targets --all-features -- -D warnings` | ✅ | ⬜ pending |
| 115-04-01 | 04 | 3 | SPEC-04 | — | ferro-mcp live-code type signatures migrated; render_projection wraps Spec to Value; v1-scanner regex preserved in json_ui_inspect.rs with literal `TODO(Phase 120)` in both json_ui_inspect.rs and application_info.rs (plan-checker Warning 3). | build + test + grep | `cargo build -p ferro-mcp --all-targets --all-features && cargo test -p ferro-mcp --lib && grep -q 'JsonUiView' ferro-mcp/src/tools/json_ui_inspect.rs && grep -q 'TODO(Phase 120)' ferro-mcp/src/tools/json_ui_inspect.rs && grep -q 'TODO(Phase 120)' ferro-mcp/src/tools/application_info.rs` | ✅ | ⬜ pending |
| 115-04-02 | 04 | 3 | SPEC-04 | T-115-11 | ferro-mcp + ferro-cli template strings emit v2 syntax; workspace-wide build green | build + test | `cargo build --all-targets --all-features && cargo test -p ferro-cli` | ✅ | ⬜ pending |
| 115-05-01 | 05 | 4 | SPEC-01..04 | T-115-13, T-115-15 | Full workspace fmt + clippy + test green; all 7 ROADMAP success criteria verified. SC-6 upgraded from grep-count to runtime test invocation (plan-checker Blocker 1). | lint + test + build | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features && cargo test -p ferro-json-ui --lib schema_for_` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Sampling continuity:** every task has an `<automated>` verify command. No 3 consecutive tasks without automated verification. Wave 0 (fixtures + integration tests) is produced inside Plan 01 Task 2. The new row 115-02-03 (schema-smoke tests) is ~35 runtime tests enforcing D-32's runtime contract — one `#[test]` per surviving Props struct, all invoked in a single `cargo test` call for <5s feedback.

---

## Wave 0 Requirements

Produced inside Plan 01 Task 2 (not a separate wave):

- [ ] `ferro-json-ui/tests/fixtures/ok/` — 7 valid spec fixtures (minimal, three-level nested, actions, visibility, plugin-named type, data payload, omitted optionals)
- [ ] `ferro-json-ui/tests/fixtures/reject/` — 11 invalid spec fixtures (missing root, dangling child, A→B→A cycle, A→A self-cycle, 4-level nesting, invalid ID with space/empty/digit-start/too-long/child-ref-format, duplicate IDs in raw JSON)
- [ ] `ferro-json-ui/tests/round_trip.rs` — integration test that walks `fixtures/ok/` and asserts parse → serialize → reparse equality (includes 1 builder-parity test for D-31)
- [ ] `ferro-json-ui/tests/reject.rs` — integration test that walks `fixtures/reject/` and asserts the expected `SpecError` variant for each

No new testing framework needed — `cargo test` is the workspace standard.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Placeholder renderer produces valid HTML at runtime | D-17 | Visual sanity check; full render pipeline arrives in Phase 116 | Run the sample `app` crate, open the rendered placeholder page in a browser, confirm `<pre><code>` block shows Spec JSON and the `<!-- ferro-json-ui v2 render pipeline arrives in Phase 116 -->` comment is present in DOM. Note: the sample `app` crate currently uses Inertia not JSON-UI, so this check requires wiring a temporary JSON-UI route — deferred until Phase 116. |

All other phase behaviors (type shape, structural validation, builder, JsonSchema generation — both compile-time AND runtime per row 115-02-03, v1-deletion, workspace build green) have automated verification via `cargo test`, `cargo build --all-targets --all-features`, `cargo clippy --all --all-targets -- -D warnings`, and grep-based acceptance criteria.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (fixtures/ok + fixtures/reject + round_trip.rs + reject.rs) — produced in Plan 01 Task 2
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s (quick) / 90s (full)
- [ ] `nyquist_compliant: true` set in frontmatter after plans are reviewed by plan-checker
- [ ] Plan 05 Task 1 confirms fmt + clippy + test all green before phase close
- [ ] Plan-checker review addressed: Blocker 1 (row 115-02-03 added for runtime D-32), Warning 1 (Plan 02 Task 1 action includes incremental `cargo check` guidance), Warning 2 (Plan 03 Task 2 action includes incremental sampling after every ~10 ports), Warning 3 (Plan 04 Task 1 acceptance criteria include `JsonUiView` regex preservation + `TODO(Phase 120)` literal presence)

**Approval:** pending — planner filled the Per-Task Verification Map, checker to sign off.
