# Phase 170: ferro-cli Migration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 170-ferro-cli-migration
**Mode:** `--auto` (all gray areas auto-selected, recommended option chosen per area)
**Areas discussed:** Async bridge, SDK entry point, Generation shape, Provider gating, Request knobs, Feature cleanup

---

## Async→Sync Bridge

| Option | Description | Selected |
|--------|-------------|----------|
| Local `Runtime::new().block_on()` | Bridge at the LLM-call boundary; keep sync `run()`/`main()` | ✓ |
| Convert command to async | Make `run()`/`main()` async, thread a runtime from the top | |

**Choice:** Local runtime at the call site. ferro-cli `main()` is sync; tokio "full" already a dep; smallest change.

---

## SDK Entry Point

| Option | Description | Selected |
|--------|-------------|----------|
| Low-level `client.complete()` + catalog schema | Send `global_catalog().json_schema()` via the raw `CompletionRequest.schema` field | ✓ |
| Generic `complete::<T>()` | Requires `Spec: JsonSchema`; derives schema from schemars, diverging from the catalog validator | |

**Choice:** `client.complete()`. The catalog's runtime-built schema is the validation source of truth; `complete::<T>()` can't reproduce it. Surfaces a discrepancy with ROADMAP SC#2's literal wording — flagged in CONTEXT for the planner to reword to "through the ferro-ai SDK".

---

## Generation Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve two-pass | Plan (plain text) → structure (catalog schema) → validate → static fallback | ✓ |
| Collapse to single structured call | One `complete` with schema | |

**Choice:** Preserve two-pass. SC#3 requires existing behavior preserved; v2 redesign is Phase 173.

---

## Provider Gating & Env Vars

| Option | Description | Selected |
|--------|-------------|----------|
| Gate on `AiConfig::from_env()` | `Ok` → AI path, `Err` → static template; `FERRO_AI_*` controls provider | ✓ |
| Keep `ANTHROPIC_API_KEY` check | Anthropic-only gating | |

**Choice:** `AiConfig::from_env()`. SC#4 requires `FERRO_AI_*` vars to control the provider. `--no-ai` preserved.

---

## Request Knob Parity

| Option | Description | Selected |
|--------|-------------|----------|
| Accept SDK defaults; map max_tokens + system | No temperature / cache_control (not in `CompletionRequest`) | ✓ |
| Extend `CompletionRequest` with temperature now | Cross-provider trait change inside the migration phase | |

**Choice:** Accept SDK defaults. Avoid scope creep; loss of `temperature: 0.2` noted as a deferred SDK enhancement.

---

## reqwest `blocking` Feature

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `blocking` feature | `api_check.rs` still uses `reqwest::blocking` | ✓ |
| Remove `blocking` feature | Would break `api_check` | |

**Choice:** Keep. Only `ai.rs`'s blocking usage is removed; SC#1 scoped to the deleted AI client.

---

## Claude's Discretion

- Module placement of relocated prompt-builder / scan helpers.
- One runtime per command vs per-call (prefer one, reused across both passes).
- Optional regression test for the static-fallback path under `AiConfig::from_env()` error.

## Deferred Ideas

- `temperature: Option<f32>` on `CompletionRequest` (future ferro-ai SDK phase).
- Provider-agnostic prompt `cache_control` knob (future SDK phase).
- `make:json-view` v2 schema-driven redesign (Phase 173).
