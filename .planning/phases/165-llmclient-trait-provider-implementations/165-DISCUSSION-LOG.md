# Phase 165: LlmClient Trait & Provider Implementations - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 165-llmclient-trait-provider-implementations
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** Streaming scope, Old-provider convergence, Default models, Error typing

---

## Pre-discussion: milestone-tracking reconciliation

Before the phase could be discussed, GSD tooling could not resolve Phase 165 because STATE.md
`milestone:` was stale (`v11.0`) and the ROADMAP v12.4 heading still contained the literal
"was v12.1", which shadowed the real `### v12.1 AI` heading in the parser's milestone-scoping regex.

**Fix applied (user-approved: "Repoint to v12.1, keep reminders"):**
- STATE.md `milestone:` `v11.0` → `v12.1`; `milestone_name` → "AI — ferro-ai SDK & AI as Projection Consumer"; `status` `verifying` → `planning`. Pending release reminders (master push, manual `cargo publish` of ferro-bundle/deployments/assets, Phase 189) left intact in STATE "Next".
- ROADMAP.md line 475 heading reworded to drop the literal "v12.1" (rename note retained in the body and overview bullet).

Also noted: several 📋/🚧 milestone headings (v11.3, v11.5, v11.9, v11.11, v11.12, v12.3) are stale —
the work shipped (SUMMARY+VERIFICATION on disk) but the headings were never flipped to ✅. Flagged to
the user; cleanup deferred (not blocking).

---

## Streaming scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full streaming, all providers | Real `complete_stream` for Anthropic/OpenAI (SSE) + Ollama (NDJSON), returns `TokenStream` | ✓ |
| Anthropic streaming only | Others return `Unsupported` for now | |
| Signature + type only | Define `TokenStream`, defer all real streaming to Phase 168/169 | |

**Selected:** Full streaming, all providers.
**Notes:** SC#1 mandates the `complete_stream` method and SC#6 explicitly declares `reqwest-eventsource`
in provider modules — a signature-only stub would contradict both. (CONTEXT D-08/D-09)

---

## Old-provider convergence

| Option | Description | Selected |
|--------|-------------|----------|
| Reimplement as adapter over AnthropicClient | Old `classify_raw` delegates HTTP to new client; delete duplicated HTTP | ✓ |
| Keep parallel this phase | Two HTTP paths coexist; converge later | |

**Selected:** Reimplement as thin adapter over `AnthropicClient`.
**Notes:** Single HTTP source of truth — aligns with no-duplicate-control-surface convention and
STATE.md "Classifier<T> delegates HTTP to AnthropicClient internally". Requires the client request
struct to carry an optional structured-output schema field from day one (D-11) so the bridge works
before the Phase 166 normalizer exists. Public API preserved (D-12). (CONTEXT D-10/D-11/D-12)

---

## Default models

| Option | Description | Selected |
|--------|-------------|----------|
| sonnet / gpt-4o / llama3.1 | Anthropic `claude-sonnet-4-6`, OpenAI `gpt-4o`, Ollama `llama3.1` | ✓ |
| opus default for Anthropic | `claude-opus-*` as Anthropic default | |

**Selected:** Anthropic `claude-sonnet-4-6`, OpenAI `gpt-4o`, Ollama `llama3.1` — all overridable via
`FERRO_AI_MODEL`, resolved through `LlmClient::default_model()`.
**Notes:** Preserves the existing classifier default value; sonnet is the fast/cheap/capable default for
classification workloads. (CONTEXT D-05)

---

## Error typing

| Option | Description | Selected |
|--------|-------------|----------|
| Typed status-carrying Provider error + Unsupported | `Error::Provider { status, message }`, status-based `is_retryable()`; add `Error::Unsupported` | ✓ |
| Keep `Provider(String)` + string matching | Only add mandatory `Unsupported` | |

**Selected:** Typed status-carrying error + `Unsupported`.
**Notes:** Removes the `is_permanent_provider_error` string-sniffing audit smell. Breaking change to the
`Error` enum — permitted in v12.1 AI. (CONTEXT D-13/D-14)

## Claude's Discretion

- Exact method signatures under `client/`; internal module layout; `AiConfig` dispatch shape (enum vs `Box<dyn>`).

## Deferred Ideas

- Typed `complete::<T>()` + schema normalizer + tool calling → Phase 166.
- Embeddings + cosine + pgvector → Phase 167 (with the Anthropic-`embed()`-`Unsupported` reconciliation note).
- Framework SSE primitives → Phase 168.
