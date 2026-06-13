---
phase: 210
slug: comp-03-agent-success-rate-harness
status: ready
nyquist_compliant: true
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
| 210-01-01 | 01 | 1 | COMP-03 | — | dev-deps test-only, no always-on dep added to ferro-mcp default build | build | `cargo build -p ferro-mcp --tests 2>&1 \| tail -5` | ❌ W0 | ⬜ pending |
| 210-01-02 | 01 | 1 | COMP-03 | — | corpus uses invented domains, no app identity | unit | `python3 -c "import json; d=json.load(open('ferro-mcp/tests/fixtures/agent_harness/corpus.json')); assert len(d)==14; ..."` | ❌ W0 | ⬜ pending |
| 210-01-03 | 01 | 1 | COMP-03 | T-210-13 / — | no corpus noun appears in catalog.rs/docs (contamination guard) | unit | `cargo test -p ferro-mcp --test agent_harness corpus_contamination_guard 2>&1 \| tail -10` | ❌ W0 | ⬜ pending |
| 210-02-01 | 02 | 2 | COMP-03 | — | T1 scorer never aborts test run (catch_unwind / global_catalog().validate) | unit | `cargo test -p ferro-mcp --test agent_harness 2>&1 \| tail -15` | ❌ W0 | ⬜ pending |
| 210-02-02 | 02 | 2 | COMP-03 | T-210-09 / — | T4 materializes ServiceDef into tempdir; no writes outside tempdir | unit | `cargo test -p ferro-mcp --test agent_harness 2>&1 \| tail -15` | ❌ W0 | ⬜ pending |
| 210-02-03 | 02 | 2 | COMP-03 | — | replay path deterministic; tiers reported separately, never collapsed | unit | `cargo test -p ferro-mcp --test agent_harness 2>&1 \| tail -20` | ❌ W0 | ⬜ pending |
| 210-03-01 | 03 | 3 | COMP-03 | — | in-process rmcp duplex; no `generate_projection` in agent toolset | integration | `cargo test -p ferro-mcp --test agent_harness 2>&1 \| tail -12 && cargo clippy -p ferro-mcp --all-targets -- -D warnings` | ❌ W0 | ⬜ pending |
| 210-03-02 | 03 | 3 | COMP-03 | T-210-01 / — | API key from env only, never logged; live test `#[ignore]`+`FERRO_AGENT_EVAL` gated | integration | `cargo test -p ferro-mcp --test agent_harness 2>&1 \| tail -15 && cargo clippy -p ferro-mcp --all-targets -- -D warnings` | ❌ W0 | ⬜ pending |
| 210-04-01 | 04 | 4 | COMP-03 | T-210-01 / — | gated live run; secrets never committed to baseline/transcripts | **manual** | MISSING — human gated live run (no API key in autonomous env). **Blocks 210-04-02** until baseline.json + transcripts committed. | ❌ W0 | ⬜ pending |
| 210-04-02 | 04 | 4 | COMP-03 | — | replay equals committed baseline (determinism guard) | unit | `cargo test -p ferro-mcp --test agent_harness 2>&1 \| tail -20` | ❌ W0 | ⬜ pending |
| 210-04-03 | 04 | 4 | COMP-03 | — | SC#5 weakness finding is non-empty, no TBD/placeholder | doc | `test -s .planning/phases/210-comp-03-agent-success-rate-harness/210-WEAKNESSES.md && wc -l .planning/phases/210-comp-03-agent-success-rate-harness/210-WEAKNESSES.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

> **210-04-01 is a `checkpoint:human-action`** (live gated run, no API key in autonomous env) — exempt from automated verify. Its automated coverage is indirect: **210-04-02's `agent_eval_replay_matches_baseline` cannot pass until 210-04-01 commits `baseline.json` + per-task transcripts.** The human gate necessarily precedes its automated verification.

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

- [x] All tasks have `<automated>` verify (replay path) or Wave 0 dependencies — 10/11 auto; 210-04-01 is a human checkpoint covered indirectly by 210-04-02
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (test target, transcripts, dev-dep feature) — built in Plan 01 T1 + Plan 02 T3
- [x] No watch-mode flags
- [x] Feedback latency < 20s (replay path)
- [x] `nyquist_compliant: true` set in frontmatter
- [ ] `wave_0_complete: true` — flipped by the executor once the test target + fixtures exist (false at plan time)

**Approval:** approved 2026-06-13 (planning-stage contract complete; wave_0_complete flips at execution)
