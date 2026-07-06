---
phase: 170-ferro-cli-migration
plan: 01
status: complete
requirements: [AISDK-06]
completed: 2026-06-08
---

# Plan 170-01 Summary — ferro-cli Migration

## Objective

Delete the blocking Anthropic-only `ferro-cli/src/ai.rs` client and route
`make:json-view`'s LLM calls through the `ferro-ai` SDK via `AiConfig::from_env()` +
`client.complete()`, preserving the two-pass generation behavior and static-template
fallback. Transport-swap plumbing only — no generation redesign.

## What Was Built

- **Deleted** `ferro-cli/src/ai.rs` (411 lines): the `reqwest::blocking` Anthropic client
  (`call_anthropic`, `call_anthropic_plain`, `call_anthropic_structured`) is gone. Removed
  `pub mod ai;` from `ferro-cli/src/lib.rs`.
- **Relocated** the four transport-agnostic helpers (`build_json_view_pass1`,
  `build_json_view_pass2`, `scan_models`, `scan_routes`) into `make_json_view.rs` — not
  deleted with the transport.
- **Rewired** `make_json_view.rs`:
  - Provider gating now branches on `AiConfig::from_env()`: `Ok(client)` → AI path,
    `Err(_)` → static template with the unchanged informational stderr message. `--no-ai`
    short-circuits before any client construction.
  - `generate_with_ai` bridges the sync CLI to the async SDK with one
    `tokio::runtime::Runtime::new()` reused across both passes (analog: `commands/mcp.rs`).
  - Pass 1: `client.complete(CompletionRequest { schema: None, max_tokens: 1024, .. })`
    (plain-text plan). Pass 2: `client.complete(CompletionRequest { schema: Some(
    global_catalog().json_schema()), max_tokens: 4096, .. })` (structured spec).
  - Preserved `Spec::from_json` + `catalog.validate` + yellow-warning / "Falling back to
    static template." fallback UX byte-for-byte.
- **Added** the `ferro-ai` dependency to `ferro-cli/Cargo.toml`; **kept** the `reqwest`
  `blocking` feature (still used by `commands/api_check.rs`).

## Key Decisions Honored (CONTEXT.md)

- **D-02 (load-bearing):** routed through low-level `client.complete()` with the catalog's
  runtime schema in `CompletionRequest.schema`, NOT `ferro_ai::complete::<T>()`. `Spec` was
  not forced to derive `JsonSchema`. `Message`/`Role` imported from `ferro_ai::client`.
- **D-01:** sync CLI surface preserved; runtime bridge at the call site only.
- **D-05:** `max_tokens` mapped per pass; `temperature` / `cache_control` intentionally not
  carried (deferred SDK enhancement).
- **D-06:** `reqwest` `blocking` feature and `api_check.rs` untouched.
- **SC#2 wording:** read as "through the ferro-ai SDK / `LlmClient::complete()`" — the
  literal `complete::<T>()` is not applicable here (see D-02). ROADMAP wording corrected.

## Key Files

### Modified
- `ferro-cli/Cargo.toml` — `ferro-ai` dep added; `reqwest` `blocking` retained
- `ferro-cli/src/lib.rs` — `pub mod ai;` removed
- `ferro-cli/src/commands/make_json_view.rs` — SDK-routed two-pass generation + bridge + relocated helpers

### Deleted
- `ferro-cli/src/ai.rs` — blocking Anthropic client

## Self-Check: PASSED

Verification gate (CLAUDE.md), all serial:
- `cargo fmt --all -- --check` → PASS
- `cargo clippy --all --all-targets -- -D warnings` → PASS (clean, no warnings)
- `cargo test --all-features` → **PASS — 3079 passed, 0 failed** (exit 0), including
  `ferro-cli` `make_json_view` unit tests and `static_fallback_produces_valid_spec`.

Code-level decision checks (grep-verified):
- `ai.rs` absent; no `reqwest::blocking` / `api.anthropic.com` in `make_json_view.rs`.
- `AiConfig::from_env`, `block_on(client.complete(...))`, `global_catalog().json_schema()` present.
- `api_check.rs` still uses `reqwest::blocking`; `Cargo.toml` keeps `blocking`.

## Requirements

- **AISDK-06** — Satisfied. Blocking client deleted; ferro-cli depends on ferro-ai; all
  LLM calls route through the SDK via `AiConfig::from_env()`.

## Notes

- Execution was interrupted once by a machine-level disk-full condition (100% volume) during
  the test gate — an environmental issue independent of the code (consistent with the repo's
  prior `disk-full` recovery). After space was freed, the full gate ran green on the
  already-committed code with no changes required.
