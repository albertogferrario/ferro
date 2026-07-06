# Phase 210: COMP-03 — Agent-Success-Rate Harness - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 210-comp-03-agent-success-rate-harness
**Mode:** `--auto` (recommended defaults auto-selected)
**Areas discussed:** Execution model, Agent output + toolset, Tier pass definitions, Contamination guard + corpus

---

## Execution model

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Live LLM, gated | Anthropic API call gated behind an env var; faithful but CI skips it | |
| (b) Deterministic replay | Replay recorded transcripts; CI-safe but measures the scorer, not a live agent | |
| (c) Hybrid | Live gated runs refresh baseline + transcripts; CI replays transcripts through the same scorer | ✓ |

**Selected:** (c) Hybrid — `FERRO_AGENT_EVAL=1` gates live runs; baseline model `claude-opus-4-8`; reuse ferro-ai completion client. Keeps the no-network green-`cargo test` invariant while still measuring a live agent on demand.

---

## Agent output + toolset

| Option | Description | Selected |
|--------|-------------|----------|
| ServiceDef (hand-authored) | Agent emits a ServiceDef guided by introspection tools; harness renders + checkpoints | ✓ |
| Rendered spec (JSON) | Agent emits the final spec directly | |
| generate_projection args | Agent calls generate_projection | (rejected — derives from a model, not NL) |

**Selected:** ServiceDef. Toolset = `generation_context` + `json_ui_catalog` + `checkpoint_projection`; `generate_projection` excluded (model-derivation, not NL authoring).

---

## Tier pass definitions

| Option | Description | Selected |
|--------|-------------|----------|
| Cumulative, render-chain based | T1 Catalog::validate 0 err → T2 derive_intents[0]==target → T3 213 content-binding bar → T4 checkpoint clean | ✓ |
| Snapshot equality | Compare against a frozen reference spec | (rejected — brittle to valid variation) |

**Selected:** Cumulative render-chain tiers (D-07/D-08). T3 enforces the Phase 213 content-binding bar per intent.

---

## Contamination guard + corpus

| Option | Description | Selected |
|--------|-------------|----------|
| Invented synthetic domains | Novel entity/field nouns not in ferro/gestiscilo/catalog; paraphrased phrasing | ✓ |
| Paraphrased real domains | Reword known apps | (weaker — closer to training data) |
| Reuse Phase 207 catalog | Use the canonical ServiceDefs as tasks | (rejected — in/near training data; contamination) |

**Selected:** Invented synthetic domains. 14 NL tasks, 2 per intent, each declaring its target intent for T2.

---

## Claude's Discretion

Exact prompt template + version string; transcript format; trial aggregation; the specific 14 invented domains; baseline-artifact layout.

## Deferred Ideas

- Success-rate floor / CI pass threshold — set after the first committed baseline (ROADMAP open decision).
- Corpus expansion / multi-step agent tasks — future hardening.
