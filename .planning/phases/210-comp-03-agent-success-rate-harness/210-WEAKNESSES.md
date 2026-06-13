# Phase 210 — Discovered Weaknesses (COMP-03 SC#5)

**Baseline:** `claude-opus-4-8`, prompt `v1`, 14 tasks × 3 trials (42 attempted).
**Source:** `ferro-mcp/tests/fixtures/agent_harness/baseline.json` + per-task transcripts.

> Scope note: the live run exhausted its API credit budget partway through, so 19
> of 42 trials never reached the model (provider error, not agent failure) and are
> excluded from rates. Rates below are over the **23 genuinely measured trials**.
> Browse, Collect, Focus are fully measured (6/6 each); Process is 5/6; Summarize,
> Analyze, and Track are **unmeasured** (0/6 — credits exhausted) and are a known
> gap for a future funded run, not a measured-zero result.

## Measured baseline (23 trials)

| Tier | Rate | Pass / measured |
|------|------|-----------------|
| T1 structural validity | 0.83 | 19 / 23 |
| T2 intent coverage     | 0.30 | 7 / 23 |
| T3 functional completeness | 0.30 | 7 / 23 |
| T4 checkpoint pass     | 0.30 | 7 / 23 |

| Intent | measured | T1 | T2 | T3 | T4 |
|--------|----------|----|----|----|----|
| Browse   | 6 | 1.00 | 0.00 | 0.00 | 0.00 |
| Collect  | 6 | 1.00 | 0.00 | 0.00 | 0.00 |
| Focus    | 6 | 1.00 | 1.00 | 1.00 | 1.00 |
| Process  | 5 | 0.20 | 0.20 | 0.20 | 0.20 |
| Summarize | 0 | — unmeasured (6 errored) |
| Analyze   | 0 | — unmeasured (6 errored) |
| Track     | 0 | — unmeasured (6 errored) |

## Finding 1 — Browse and Collect: valid ServiceDef, wrong derived intent (T2 cliff)

**The strongest signal in the measured data.** For Browse and Collect (12 trials),
the agent produces a structurally valid ServiceDef every time (T1 = 100%) but the
**derived** intent never matches the target (T2 = 0/12), which cascades to T3/T4 = 0
under cumulative scoring.

So the agent can author a renderable, catalog-valid projection but cannot reliably
shape it so `derive_intents` classifies it as the intended structural intent. The
gap is not "can it produce a ServiceDef" — it is "does the ServiceDef carry the
structural signals (`has_many` + EntityName fields for Browse; many writable fields
+ `write_only` for Collect) that the derivation analyzers key on." Because the agent
is forbidden from emitting `intent_hints` (T2 anti-cheat), T2 measures genuine
structural derivation — and that is where it falls down for these two intents.

Hypothesis: the agent under-specifies the signals each intent's analyzer in
`ferro-projections/src/derive.rs` requires, so a Browse description renders as a
generic table whose top derived intent is something else. Evidence: every
`browse-*` and `collect-*` transcript trial carries a non-null `service_def` (T1
true) with `t2=false`.

## Finding 2 — `generation_context` gives no ServiceDef authoring guidance

The agent's toolset is `generation_context`, `json_ui_catalog`,
`checkpoint_projection`. `generation_context` returns handler/model/view
conventions — **not** projection/ServiceDef authoring guidance. The agent's only
source for the ServiceDef shape is the `schemars` schema injected into the harness
prompt. This is a real surface weakness: the introspection layer under test does
not teach an agent how to author the very artifact this evaluation asks for. It is
a candidate input to a future prompt/tooling iteration (out of scope for this
phase, which must not mutate a tool mid-measurement).

## Finding 3 — Harness defects the gated tests hid (process learnings)

The live run surfaced three real harness/dependency bugs that never appeared during
execution because the gated tests are `#[ignore]`'d and were never actually run:

1. **In-process rmcp transport drop** (`agent_harness.rs`): the spawned server
   bound `serve()`'s `RunningService` to `_`, dropping it right after the
   handshake — so the handshake succeeded but the first tool call failed with
   "Transport closed". Fixed by holding the handle and driving it with
   `.waiting().await`.
2. **ferro-ai multi-turn serialization** (`ferro-ai/src/client/anthropic.rs`):
   `complete_with_tools` returns the assistant turn as a JSON content-blocks array
   string, but `build_body` re-serialized it as a single text block, dropping the
   `tool_use` block — so the next `tool_result` was rejected by Anthropic
   ("no corresponding tool_use block"). ferro-ai's tool-use multi-turn round-trip
   had no prior consumer; this harness is the first. Fixed by spreading the array
   back as structured content.
3. **API errors scored as agent failures** (`agent_harness.rs`): the scorer
   counted provider-errored trials (credit exhaustion mid-run) as `T1=false`,
   which silently corrupted the baseline (a naive run showed T1=0.45 instead of
   the measured 0.83). Fixed: errored trials carry an `error` field and are
   EXCLUDED from rates; intents with zero measured trials are reported
   `unmeasured`, never `0.0`.

The standing guard against (3) recurring is `agent_eval_replay_matches_baseline`,
which recomputes the baseline from committed transcripts (excluding errored trials)
and asserts it matches `baseline.json` — deterministic, no LLM, runs in default CI.

## Known gap for the next run

Summarize, Analyze, and Track were never measured (credits exhausted). A future
funded run (`FERRO_AGENT_EVAL=1`) refreshes the transcripts + baseline via the
existing live test; the replay guard then re-pins the new numbers. The success-rate
floor / CI threshold remains deferred (ROADMAP open decision) until a complete
baseline exists.
