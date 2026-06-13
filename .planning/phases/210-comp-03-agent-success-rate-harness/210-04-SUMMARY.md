---
phase: 210-comp-03-agent-success-rate-harness
plan: "04"
subsystem: ferro-mcp
tags: [agent-harness, comp-03, baseline, replay-guard, weaknesses, bugfix]
dependency_graph:
  requires:
    - ferro-mcp/tests/agent_harness.rs (Wave 1-3 harness + live loop)
    - ferro-ai/src/client/anthropic.rs (complete_with_tools multi-turn)
  provides:
    - ferro-mcp/tests/fixtures/agent_harness/baseline.json (first committed baseline, partial)
    - ferro-mcp/tests/fixtures/agent_harness/transcripts/*.json (14 per-task transcripts, error-annotated)
    - recompute_baseline_doc(): shared offline scorer/aggregator (excludes errored trials)
    - agent_eval_replay_matches_baseline: CI determinism guard (no LLM)
    - regen_baseline_from_transcripts: gated offline baseline regen (FERRO_AGENT_REGEN=1)
    - 210-WEAKNESSES.md (SC#5 discovered-weaknesses finding)
  affects:
    - ferro-ai build_body assistant-message serialization (multi-turn tool-use fix)
tech_stack:
  added: []
  patterns:
    - errored-trial exclusion (API error != agent failure)
    - single recompute fn shared by live write / regen / replay assertion
key_files:
  created:
    - ferro-mcp/tests/fixtures/agent_harness/baseline.json
    - .planning/phases/210-comp-03-agent-success-rate-harness/210-WEAKNESSES.md
  modified:
    - ferro-mcp/tests/agent_harness.rs
    - ferro-ai/src/client/anthropic.rs
metrics:
  measured_trials: 23
  errored_trials: 19
  tier_rates_measured: { t1: 0.83, t2: 0.30, t3: 0.30, t4: 0.30 }
---

# Plan 210-04 Summary — First Baseline + Weaknesses (COMP-03 close)

## What was built

The first committed baseline, the deterministic replay guard, and the SC#5
discovered-weaknesses finding — plus three real harness/dependency bug fixes the
live run surfaced.

The Task 1 human-action live run (`FERRO_AGENT_EVAL=1`, `claude-opus-4-8`) was
executed with a developer-supplied API key. It exhausted its credit budget
partway through, so 19 of 42 trials never reached the model. The baseline is
therefore a genuine **partial**: Browse/Collect/Focus fully measured, Process 5/6,
and Summarize/Analyze/Track unmeasured (credits) — recorded honestly as
`status: unmeasured`, not `0.0`.

## Measured baseline (23 trials, n excludes 19 provider-errored)

| Tier | Rate | Per-intent highlights |
|------|------|------------------------|
| T1 | 0.83 (19/23) | Browse/Collect/Focus 1.0; Process 0.2 |
| T2 | 0.30 (7/23)  | Focus 1.0; **Browse/Collect 0.0** |
| T3 | 0.30 (7/23)  | cumulative on T2 |
| T4 | 0.30 (7/23)  | cumulative on T2 |

## Bugs found and fixed (live run exposed; gated tests had hidden them)

1. **rmcp transport drop** (`agent_harness.rs`): server `RunningService` dropped
   after handshake → "Transport closed" on first tool call. Now held +
   `.waiting().await`.
2. **ferro-ai multi-turn serialization** (`ferro-ai/src/client/anthropic.rs`):
   assistant tool-use content array re-serialized as a text string → Anthropic 400
   on the next tool_result. Now spread back as structured content. (ferro-ai's
   tool-use multi-turn had no prior consumer.)
3. **API errors scored as agent failures** (`agent_harness.rs`): errored trials
   counted as `T1=false`, corrupting the baseline. Now excluded; trials carry an
   `error` field; unmeasured intents reported as such.

## Verification

- `cargo test -p ferro-mcp --test agent_harness` → 5 passed, 3 ignored (gated),
  NO env vars, no network. `agent_eval_replay_matches_baseline` reproduces the
  committed baseline from transcripts (determinism guard).
- `cargo fmt --all -- --check` clean; `cargo clippy -p ferro-ai -p ferro-mcp
  --all-targets -- -D warnings` clean.
- No API key (real or placeholder) in any committed transcript/baseline.

## Deviations

- Baseline is partial (credit exhaustion) — a deviation from "all 14 tasks
  measured." Documented in 210-WEAKNESSES.md; a future funded run refreshes
  Summarize/Analyze/Track via the existing live test, re-pinned by the replay
  guard. Success-rate floor remains deferred (ROADMAP open decision).

## Self-Check: PASSED
