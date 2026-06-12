---
phase: 210
slug: comp-03-agent-success-rate-harness
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-13
---

# Phase 210 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Observable signals derived from RESEARCH.md §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust, `ferro-mcp/tests/agent_harness.rs`) |
| **Config file** | none — integration test target under `ferro-mcp/tests/` (greenfield) |
| **Quick run command** | `cargo test -p ferro-mcp --test agent_harness` (replay path, no LLM) |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-mcp --test agent_harness` |
| **Estimated runtime** | ~5–20 seconds (replay path; no network, no LLM) |

**Live eval (opt-in only, NOT in CI):** `FERRO_AGENT_EVAL=1 cargo test -p ferro-mcp --test agent_harness -- --ignored` — drives the live agent (~42 calls), refreshes baseline + transcripts. Requires `FERRO_AI_API_KEY`/`ANTHROPIC_API_KEY`.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp --test agent_harness` (replay path)
- **After every plan wave:** Run the full suite command above
- **Before `/gsd-verify-work`:** Full suite must be green WITHOUT `FERRO_AGENT_EVAL` set (the always-green, no-network invariant)
- **Max feedback latency:** ~20 seconds

---

## Per-Task Verification Map

> Populated by the planner. Every scorer/tier task must have an automated replay-path command that runs without an LLM.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 210-01-01 | 01 | 1 | COMP-03 | T-210-01 / — | API key never logged/committed; sourced from env only | unit | `cargo test -p ferro-mcp --test agent_harness` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp/tests/agent_harness.rs` — greenfield test target (currently empty `tests/` dir)
- [ ] Committed transcript fixtures — enable the deterministic replay path to run in default `cargo test`
- [ ] `ferro-mcp/Cargo.toml` `[dev-dependencies]` — rmcp `transport-async-rw` feature for in-process transport

*The replay path is what CI runs; it MUST be green without any LLM call.*

---

## Observable Signals (from RESEARCH.md §Validation Architecture)

| Signal | How it's proven | Tier |
|--------|-----------------|------|
| **Scorer determinism** | Same committed transcript → identical per-tier result on every replay run (assert in test) | all |
| **Tier independence** | Each tier (T1–T4) reported separately; never collapsed to a single boolean | all |
| **CI-green-without-LLM** | Replay path runs in default `cargo test` with no network/API key | infra |
| **Debug-panic safety** | T1 records invalid-spec failures via `global_catalog().validate()` / `catch_unwind` — never aborts the test process (Pitfall 3) | T1 |
| **Checkpoint materialization** | T4 materializes ServiceDef → `src/projections/<name>.rs` in a tempdir before `checkpoint_projection` (Pitfall 4) | T4 |
| **Contamination guard** | No corpus domain/entity/field noun appears in `catalog.rs`, ferro docs, or sample app (automated grep assertion) | corpus |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First committed baseline numbers are plausible | COMP-03 | Requires a live LLM run (gated, costs API calls); SC#5 "discovered weaknesses" finding is a human judgement | `FERRO_AGENT_EVAL=1 cargo test ... -- --ignored`, then review per-tier rates + name ≥1 real weakness |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify (replay path) or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (test target, transcripts, dev-dep feature)
- [ ] No watch-mode flags
- [ ] Feedback latency < 20s (replay path)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
