# Phase 171: ferro ai:make & ferro ai:explain CLI Commands - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 171-ferro-ai-make-ferro-ai-explain-cli-commands
**Mode:** `--auto` (gray areas auto-selected, recommended option chosen per area)
**Areas discussed:** In-process introspection wiring, Context relevance filtering, ai:make output artifact, ServiceDef completion path & cost guard, ai:explain target resolution, Command gating & dry-run

---

## In-process introspection wiring (D-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Direct `tools::*::execute()` in-process | Call ferro-mcp tool fns as library calls; ferro-cli already depends on ferro-mcp | ✓ |
| Launch ferro-mcp server, call over JSON-RPC | Spin up `ferro mcp` and talk to it as a subprocess | |

**User's choice:** `[auto]` Direct in-process tool fns.
**Notes:** SC#1 mandates in-process; tool fns are the existing typed introspection surface; zero IPC overhead.

---

## Context relevance filtering (D-02)

| Option | Description | Selected |
|--------|-------------|----------|
| Deterministic lexical token-overlap filter | Rank items by name/description/field token overlap; always include generation_context | ✓ |
| Embedding cosine-similarity rerank | Use ferro_ai::embed + cosine_similarity (Phase 167) | |
| LLM relevance pre-pass | Extra LLM call to pick relevant items | |

**User's choice:** `[auto]` Lexical token-overlap.
**Notes:** Cost guard + determinism + filtering only needs overflow prevention. Embedding rerank deferred. SC#1 "semantically relevant" satisfied lexically in v1.

---

## ai:make output artifact (D-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Rust builder file in src/projections/ (+ new ServiceDef→source emitter) | Conforms to existing projection convention; discoverable by list_projections/inspect_projection | ✓ |
| Write a JSON file | Serialize ServiceDef to JSON on disk | |
| stdout-only | Print and let dev redirect | |

**User's choice:** `[auto]` Rust builder file + new emitter.
**Notes:** Keeps output consumable by Phase 173 and consistent with the only existing ServiceDef persistence convention. New ServiceDef→builder-source emitter is the central new unit. `--dry-run` prints pretty JSON, writes nothing.

---

## ServiceDef completion path & cost guard (D-04)

| Option | Description | Selected |
|--------|-------------|----------|
| `complete_with::<ServiceDef>()` options variant | Add configurable max_tokens/system; keep ServiceDef-aware normalizer | ✓ |
| Build raw CompletionRequest manually (Phase 170 style) | Bypass complete() wrapper | |
| Use `complete::<T>()` as-is (fixed 4096) | No cost-guard control | |

**User's choice:** `[auto]` `complete_with` options variant.
**Notes:** Keeps the typed killer path + ServiceDef-aware normalizer while honoring SC#5 cost guard. Minimal, justified SDK addition. `FERRO_AI_MAX_TOKENS_PER_COMMAND` maps onto request max_tokens.

---

## ai:explain target resolution (D-05)

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-detect route→model→service, optional `--type` | First-match precedence; projection-framed when ServiceDef found | ✓ |
| Require `--type` always | Explicit kind per invocation | |
| Prefixed target syntax | e.g. `route:/path` | |

**User's choice:** `[auto]` Auto-detect with `--type` override.
**Notes:** Lowest-friction agent/dev ergonomics; deterministic precedence; projection framing via derive_intents when a ServiceDef exists, prose fallback otherwise.

---

## Command gating & dry-run (D-06)

| Option | Description | Selected |
|--------|-------------|----------|
| AI-required fail-fast + default-on `projections` feature | No non-AI path; clear error naming FERRO_AI_* env vars | ✓ |
| Silent static fallback | Degrade like make:json-view | |
| Keep projections optional | Gate ai:* behind feature, error if absent | |

**User's choice:** `[auto]` AI-required, default-on projections.
**Notes:** AI-native commands; honest failure beats degraded output; projection contract is core CLI surface now. `--dry-run`: ai:make prints ServiceDef; ai:explain prints assembled context without the LLM call.

---

## Claude's Discretion

- Lexical-relevance scoring formula, top-N cutoff, input-token budget.
- Default per-command `max_tokens` constants.
- Prompt / system-prompt wording.
- Whether `complete_with` carries `model_override` (only `max_tokens` strictly required).
- CLI flags beyond `--dry-run` / `--type`.

## Deferred Ideas

- Embedding-based semantic relevance reranking (Phase 167 primitives).
- `ai_scaffold`/`ai_explain` MCP wrappers — Phase 172.
- `make:json-view` v2 renderer + AICLI-06 roundtrip — Phase 173.
- `temperature` on `CompletionRequest` (deterministic codegen) — deferred SDK enhancement.
