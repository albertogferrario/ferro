---
phase: 221
slug: inbound-nl-intent-loop
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-14
---

# Phase 221 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust, tokio async tests) |
| **Config file** | none — workspace `Cargo.toml` + per-crate test modules |
| **Quick run command** | `cargo test -p ferro-mcp-server --features ai` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60–180 seconds (cold build longer; see disk-full gate note) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp-server --features ai`
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings` + `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green; feature-off build must also pass (`cargo build -p ferro-mcp-server` with no `ai`/`ai-live`)
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

> Filled per-plan by the planner. Each row maps a task to its automated check.
> The spine of this phase is the deterministic no-LLM replay path (SC#3): it MUST be a
> non-ignored test running under the `ai` feature only (no `llm`/reqwest).

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | AMCP-06 | T-221-* | Classified args pass 219 validation + guard re-eval + tenant scoping; classifier never bypasses auth | unit | `cargo test -p ferro-mcp-server --features ai` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Transcript-fixture directory + committed fixtures for the intent loop (mirror `ferro-mcp/tests/fixtures/agent_harness/`), keyed `ToolSelection` records — no API keys committed
- [ ] Replay `ClassificationProvider` (reqwest-free, compiles under `ai` feature only)
- [ ] Deterministic replay test stub asserting all branches: classify → guard-check → confirmation-gate → dispatch → clarify (SC#3)
- [ ] Gated live-eval test stub (`#[ignore]` + `FERRO_AI_LIVE_EVAL=1`), cost-announced before first call (SC#4)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live LLM classification accuracy + cost-announcement string | AMCP-06 (SC#4) | Costs real LLM spend; opt-in only, never in CI | `FERRO_AI_LIVE_EVAL=1 cargo test -p ferro-mcp-server --features ai-live -- --ignored <live_test>` — confirm cost is announced before the first call and the result matches/updates the fixture |

*All other phase behaviors (SC#1, SC#2, SC#3, SC#5) have automated no-LLM replay verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (replay fixtures + provider + branch-coverage test)
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
