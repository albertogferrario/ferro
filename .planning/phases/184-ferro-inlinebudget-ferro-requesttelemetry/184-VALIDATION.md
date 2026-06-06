---
phase: 184
slug: ferro-inlinebudget-ferro-requesttelemetry
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-06
---

# Phase 184 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (built-in) + `serial_test` for ordering of global-state tests |
| **Config file** | `framework/Cargo.toml` (test deps already present — no Wave 0 install) |
| **Quick run command** | `cargo test -p ferro-rs telemetry::` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~5s; full ~60s |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-rs telemetry::`
- **After every plan wave:** `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite green + `cargo fmt --all -- --check` + `cargo clippy --all --all-targets -- -D warnings` + `cargo publish -p ferro-rs --dry-run` + `cargo doc --no-deps`
- **Max feedback latency:** ~5s for quick run, ~60s for full suite

---

## Per-Task Verification Map

Phase 184 has no formal REQ-IDs (`phase_req_ids: null`). The contract is the 5 Success Criteria from ROADMAP.md, mapped here to test instances. Task IDs (`184-NN-XX`) are placeholders — the planner agent assigns final IDs during plan generation.

| Task ID | Plan | Wave | Success Criterion | Threat Ref | Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------------|------------|----------|-----------|-------------------|-------------|--------|
| 184-01-XX | 01 | 1 | (foundation) | — | Sample::now / Sample::at construct correctly | unit | `cargo test -p ferro-rs telemetry::request_telemetry::tests::sample_constructors` | ❌ W0 | ⬜ pending |
| 184-01-XX | 01 | 1 | SC-3a | — | record + snapshot round-trip | unit | `cargo test -p ferro-rs telemetry::request_telemetry::tests::record_and_snapshot_round_trip` | ❌ W0 | ⬜ pending |
| 184-01-XX | 01 | 1 | SC-3b | — | Thread-safe under concurrent record | unit | `cargo test -p ferro-rs telemetry::request_telemetry::tests::concurrent_record_no_deadlock` | ❌ W0 | ⬜ pending |
| 184-01-XX | 01 | 1 | SC-3c | — | Ring buffer caps at 128, drops oldest | unit | `cargo test -p ferro-rs telemetry::request_telemetry::tests::ring_buffer_caps_at_128` | ❌ W0 | ⬜ pending |
| 184-01-XX | 01 | 1 | (foundation) | — | AppConfig default = 102_400 (no env) | unit | `cargo test -p ferro-rs config::tests::inline_budget_threshold_default` | ❌ W0 | ⬜ pending |
| 184-01-XX | 01 | 1 | (foundation) | — | AppConfig env override (INLINE_BUDGET_BYTES) | unit | `cargo test -p ferro-rs config::tests::inline_budget_threshold_env` | ❌ W0 | ⬜ pending |
| 184-01-XX | 01 | 1 | (foundation) | — | AppConfigBuilder setter | unit | `cargo test -p ferro-rs config::tests::inline_budget_threshold_builder` | ❌ W0 | ⬜ pending |
| 184-02-XX | 02 | 2 | SC-1 | — | inline_budget returns Inline below threshold | unit | `cargo test -p ferro-rs telemetry::inline_budget::tests::decides_inline_below_threshold` | ❌ W0 | ⬜ pending |
| 184-02-XX | 02 | 2 | SC-1 | — | inline_budget returns Preload(url) at/above threshold | unit | `cargo test -p ferro-rs telemetry::inline_budget::tests::decides_preload_above_threshold` | ❌ W0 | ⬜ pending |
| 184-02-XX | 02 | 2 | SC-2 | — | Warning state-machine fires once per (key, request) | unit | `cargo test -p ferro-rs telemetry::inline_budget::tests::warn_fires_once_per_key` | ❌ W0 | ⬜ pending |
| 184-02-XX | 02 | 2 | SC-2 | — | Subsequent past-threshold calls do NOT re-warn | unit | `cargo test -p ferro-rs telemetry::inline_budget::tests::warn_silent_on_subsequent_crosses` | ❌ W0 | ⬜ pending |
| 184-02-XX | 02 | 2 | SC-2 | — | Different keys warn independently | unit | `cargo test -p ferro-rs telemetry::inline_budget::tests::warn_independent_per_key` | ❌ W0 | ⬜ pending |
| 184-03-XX | 03 | 3 | SC-1, SC-2, SC-3a | — | Real Request round-trip exercising both primitives | integration | `cargo test -p ferro-rs --test telemetry_smoke` | ❌ W0 | ⬜ pending |
| 184-03-XX | 03 | 3 | SC-5 | — | Workspace publish dry-run succeeds | manual/CI | `cargo publish -p ferro-rs --dry-run` | ✅ | ⬜ pending |
| 184-03-XX | 03 | 3 | (D-14) | — | Docs page renders + appears in SUMMARY | manual | open `book/the-basics/inline-budget-and-telemetry.html` after `mdbook build docs/` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Wave 0 creates the test scaffolding before any production code exists. All Wave 0 items are file creations, not test assertions yet.

- [ ] `framework/src/telemetry/mod.rs` — new module with `pub mod inline_budget; pub mod request_telemetry;` and module-level docs
- [ ] `framework/src/telemetry/inline_budget.rs` — new file with `Decision` enum, `InlineBudgetState` struct (private), unit-test scaffold `#[cfg(test)] mod tests`
- [ ] `framework/src/telemetry/request_telemetry.rs` — new file with `Sample`, `RequestTelemetry`, global `OnceLock<DashMap>`, `#[cfg(test)] pub(crate) fn reset()`, unit-test scaffold
- [ ] `framework/tests/telemetry_smoke.rs` — integration test scaffold using `hyper-util` + `http-body-util` patterns from `framework/tests/action_handler.rs`
- [ ] `docs/src/the-basics/inline-budget-and-telemetry.md` — docs page scaffold (sections for InlineBudget, RequestTelemetry, end-to-end example, scope conventions)
- [ ] `framework/Cargo.toml` `[dev-dependencies]` — add `tracing-test = "0.2"` (optional — adopted per OQ3 recommendation for direct warning-emission assertion; falls back to state-machine assertion if rejected)

No external framework install needed — `cargo test` is built-in and `serial_test` is already a workspace dep.

---

## Manual-Only Verifications

| Behavior | Success Criterion | Why Manual | Test Instructions |
|----------|-------------------|------------|-------------------|
| Real `cargo publish` to crates.io | SC-5 | Phase ships via the existing WAVE2 GH Actions workflow on master merge; gestiscilo Phase 187 bumps after | Watch the GH Actions run after merge; confirm `ferro-rs@0.2.44` appears on crates.io |
| Crate location decision recorded with rationale | SC-4 | Documentary — already satisfied in `184-CONTEXT.md` D-01 | Re-read `184-CONTEXT.md` D-01 during code review |
| Docs page is operator-discoverable | D-14 | The page is built into mdbook output; mdbook lint catches structural issues but not content quality | After `mdbook build`, open `book/the-basics/inline-budget-and-telemetry.html` and confirm the two-primitives walkthrough is coherent |

---

## 8 Nyquist Dimensions

| # | Dimension | Command | Pass Criterion |
|---|-----------|---------|----------------|
| 1 | Compile | `cargo build -p ferro-rs` | Exit 0, no errors |
| 2 | Lint | `cargo clippy --all --all-targets -- -D warnings` | Exit 0, no warnings (CI gate matches this exact command) |
| 3 | Unit tests | `cargo test -p ferro-rs telemetry::` | All inline `#[test]` pass |
| 4 | Integration tests | `cargo test -p ferro-rs --test telemetry_smoke` | `telemetry_smoke.rs` passes after Wave 0 creates it |
| 5 | Docs build | `cargo doc --no-deps -p ferro-rs` | Exit 0; new module visible in `target/doc/ferro_rs/telemetry/` |
| 6 | Format | `cargo fmt --all -- --check` | Exit 0, no diff |
| 7 | Publish dry-run | `cargo publish -p ferro-rs --dry-run` | Exit 0, package builds tarball |
| 8 | Observability | Inline unit tests assert warning state-machine: first cross sets `warned[key]=true`, subsequent crosses do NOT re-flip (no double-warn). Direct emission assertion via `tracing-test = "0.2"` is optional reinforcement. | State-machine assertion + (optional) `tracing-test` emission match |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s (full suite)
- [ ] `nyquist_compliant: true` set in frontmatter (after planner assigns final task IDs)

**Approval:** pending (will be approved after planner produces PLAN.md files with final task IDs and the verification map can be locked)
