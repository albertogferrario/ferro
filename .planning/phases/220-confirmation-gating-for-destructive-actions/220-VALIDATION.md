---
phase: 220
slug: confirmation-gating-for-destructive-actions
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-14
---

# Phase 220 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[tokio::test]` + `#[test]` (tokio paused clock for TTL) |
| **Config file** | None (workspace `cargo test`) |
| **Quick run command** | `cargo test -p ferro-mcp-server --features confirmation -- confirmation` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-mcp-server --features confirmation`
- **After every plan wave:** `cargo test --all-features` + `cargo clippy --all --all-targets --all-features -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite + clippy green; the feature-off build assertion passes
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 220-00-01 | 00 | 0 | AMCP-05 | — | ferro-ai `default=["llm"]` feature refactor; confirmation feature scaffolding + RED tests compile | build/unit | `cargo build -p ferro-ai --no-default-features && cargo test -p ferro-mcp-server --features confirmation --no-run` | ❌ W0 | ⬜ pending |
| 220-01-01 | 01 | 1 | AMCP-05 | T-220-01 (unconfirmed destructive) | bare destructive call without token → `ConfirmationRequired` structured, not executed | unit | `cargo test -p ferro-mcp-server --features confirmation sc1_bare_destructive_without_token` | ❌ W0 | ⬜ pending |
| 220-01-02 | 01 | 1 | AMCP-05 | — | two-step request→confirm executes exactly once (single-use token) | unit | `cargo test -p ferro-mcp-server --features confirmation sc2_two_step_flow_executes_once` | ❌ W0 | ⬜ pending |
| 220-01-03 | 01 | 1 | AMCP-05 | T-220-02 (expired/mismatch bypass) | expired token rejected; action/record mismatch rejected; guard re-eval at confirm | unit | `cargo test -p ferro-mcp-server --features confirmation sc3_expired_token_rejected sc4_token_mismatch_action sc4_token_mismatch_record sc_guard_denied_at_confirm_time` | ❌ W0 | ⬜ pending |
| 220-01-04 | 01 | 1 | AMCP-05 | — | all confirmation result envelopes parse as `rmcp::model::CallToolResult` | unit | `cargo test -p ferro-mcp-server --features confirmation write_tool_result_parses_as_valid_mcp_content` | ❌ W0 | ⬜ pending |
| 220-02-01 | 02 | 2 | AMCP-05 | — | feature-OFF: `cargo build -p ferro-mcp-server` has no ferro-ai/reqwest; read tools + 219 write tests unaffected | build+test | `cargo build -p ferro-mcp-server && cargo tree -p ferro-mcp-server --edges normal \| grep -c ferro-ai` (expect 0) | ❌ W0 | ⬜ pending |
| 220-02-02 | 02 | 2 | AMCP-05 | — | full `--all-features` CI gate green (incl. confirmation tools + sample-app exercise if wired) | full | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-ai/Cargo.toml` — `[features] default = ["llm"]`; `reqwest`/`reqwest-eventsource`/`futures`/`async-stream`/`schemars` made `optional = true` under `llm`; a `confirmation` feature with only async-trait/tokio/serde
- [ ] `ferro-ai/src/lib.rs` + client/classifier/embed/tools modules — `#[cfg(feature = "llm")]` gates so `--no-default-features` compiles (confirmation only)
- [ ] `ferro-mcp-server/Cargo.toml` — `[features] confirmation = ["dep:ferro-ai"]`; `ferro-ai = { optional = true, default-features = false, features = ["confirmation"] }`
- [ ] `ferro-mcp-server/src/error.rs` — feature-gated `ConfirmationRequired(String)` variant
- [ ] RED confirmation tests in `write_dispatch.rs` (SC#1–#4 + guard-at-confirm) compile under `--features confirmation`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none) | — | All phase behaviors have automated coverage | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
