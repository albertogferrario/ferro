---
phase: 210-comp-03-agent-success-rate-harness
plan: "03"
subsystem: ferro-mcp
tags: [agent-harness, comp-03, live-eval, rmcp-duplex, tool-use-loop, gated-test]
dependency_graph:
  requires:
    - ferro-mcp/tests/agent_harness.rs (Wave 1+2 skeleton + scorer)
    - ferro-mcp/tests/fixtures/agent_harness/corpus.json (Wave 1 corpus)
    - ferro-mcp/tests/fixtures/agent_harness/transcripts/ (Wave 2 fixtures)
    - ferro-mcp/Cargo.toml (rmcp client+transport-async-rw dev-dep, already added Wave 2)
  provides:
    - spawn_in_process_client(): FerroMcpService over tokio::io::duplex
    - call_dev_tool(): in-process tool dispatch via rmcp peer().call_tool()
    - build_agent_tools(): 3 D-06 tool definitions (no generate_projection)
    - build_system_prompt(): schemars::schema_for!(ServiceDef) + intent_hints prohibition
    - run_agent_trial(): multi-turn complete_with_tools loop (MAX_ITERATIONS=8)
    - agent_eval_live_refresh_baseline: gated live test (#[ignore] + FERRO_AGENT_EVAL=1)
    - smoke_in_process_rmcp_duplex: gated duplex transport smoke test (no LLM)
  affects:
    - ferro-mcp test surface (adds Wave 3 live apparatus to agent_harness)
tech_stack:
  added: []
  patterns:
    - tokio::io::duplex with TransportAdapterAsyncCombinedRW (A-rmcp resolved MEDIUM→HIGH)
    - ferro_ai::AnthropicClient::complete_with_tools multi-turn loop
    - #[ignore] + FERRO_AGENT_EVAL=1 belt-and-suspenders gate (D-01)
    - schemars::schema_for!(ServiceDef) injected into prompt (no domain examples)
    - intent_hints prohibition stated-before-runs (T2 anti-cheat, D-08)
key_files:
  created: []
  modified:
    - ferro-mcp/tests/agent_harness.rs
decisions:
  - "A-rmcp resolved HIGH: tokio::io::DuplexStream implements AsyncRead+AsyncWrite+Send+'static, satisfying TransportAdapterAsyncCombinedRW in rmcp 0.12.0 transport/async_rw.rs. Confirmed from rmcp source in cargo registry — single DuplexStream passed to serve() works without splitting."
  - "PROMPT_VERSION=v1 — incremented when system/user prompt changes to invalidate committed baselines."
  - "Transcript stores integer pass counts alongside fractions to enable exact-equality replay assertions rather than float comparisons."
  - "tool_calls audit log truncated to 200 chars per entry (result_summary) — never logs API key (T-210-08)."
  - "spawn_in_process_client uses tokio::spawn for the server half; JoinHandle is dropped (server lives until client cancels). This matches the established pattern from ferro-api-mcp/tests/e2e.rs."
metrics:
  duration_seconds: 349
  completed_date: "2026-06-13"
  tasks_completed: 2
  files_modified: 1
---

# Phase 210 Plan 03: In-Process rmcp Transport + Agent Tool-Use Loop Summary

In-process rmcp duplex transport wiring + `complete_with_tools` agent loop: the live apparatus that feeds the Wave-2 scorer with freshly-authored `ServiceDef`s. Compile-verified; gated behind `#[ignore]` + `FERRO_AGENT_EVAL=1`; default `cargo test` stays green with no network/API key.

## What Was Built

**Task 1 — In-process rmcp transport + tool dispatch:**

- `spawn_in_process_client(project_root)`: creates a `tokio::io::duplex(64 * 1024)` pair, spawns `FerroMcpService` as the server half via `tokio::spawn`, and completes the MCP initialize handshake on the client half. Returns `RunningService<RoleClient, ()>`.
- `call_dev_tool(client, name, args)`: dispatches a tool call via `client.peer().call_tool(CallToolRequestParam { name: name.to_owned().into(), arguments })` and extracts text content from `result.content.iter().filter_map(|c| c.raw.as_text())`.
- `smoke_in_process_rmcp_duplex`: `#[ignore]` + `FERRO_AGENT_EVAL=1`-gated smoke test that proves the transport without any LLM call — calls `json_ui_catalog` and asserts a non-empty result.

**A-rmcp resolution (MEDIUM → HIGH):** `tokio::io::DuplexStream` implements `AsyncRead + AsyncWrite + Send + 'static`, which satisfies `IntoTransport<Role, std::io::Error, TransportAdapterAsyncCombinedRW>` in rmcp 0.12.0 `transport/async_rw.rs` lines 32–43. Confirmed from live source in cargo registry. A single `DuplexStream` can be passed directly to `serve()`.

**Task 2 — Agent tool-use loop + gated live eval test:**

- `PROMPT_VERSION = "v1"` constant for baseline invalidation on prompt changes.
- `build_agent_tools()`: defines exactly 3 `LlmToolRequest` entries (`generation_context`, `json_ui_catalog`, `checkpoint_projection`). D-06 enforced — `generate_projection` count = 0.
- `build_system_prompt()`: injects `schemars::schema_for!(ServiceDef)` as the shape reference, instructs the agent to use tools then emit a JSON code block, explicitly forbids `intent_hints` (T2 anti-cheat, stated before runs per D-08), and does not name the target intent or use ferro intent vocabulary (contamination discipline).
- `run_agent_trial(llm, rmcp_client, task_description, project_root)`: multi-turn `complete_with_tools` loop (cap `MAX_ITERATIONS=8`). On `ToolUse`: pushes `Assistant{assistant_content}` first, then dispatches each block via `call_dev_tool` and pushes `Tool{result, tool_call_id=block.id}` (history reconstruction per `ferro_ai::client::mod.rs` doc-comment). On `Text`: extracts the final `ServiceDef` JSON (strips code fences via `extract_service_def_json`).
- `agent_eval_live_refresh_baseline`: `#[ignore]` + `FERRO_AGENT_EVAL=1`-gated live test. For each corpus task × 3 trials: runs `run_agent_trial`, scores via `score()`, writes `transcripts/<task_id>.json`, writes `baseline.json`. API key sourced from `FERRO_AI_API_KEY` / `ANTHROPIC_API_KEY` only — never logged, never written to any file (T-210-08 mitigation).

## Verification Results

```
cargo test -p ferro-mcp --test agent_harness
running 6 tests
test agent_eval_live_refresh_baseline ... ignored, live LLM eval; run with FERRO_AGENT_EVAL=1 and FERRO_AI_API_KEY set
test smoke_in_process_rmcp_duplex ... ignored, in-process rmcp roundtrip; run with FERRO_AGENT_EVAL=1 (no API key needed)
test t1_invalid_spec_scores_fail_without_panic ... ok
test corpus_contamination_guard ... ok
test tier_results_never_collapse_to_boolean ... ok
test agent_eval_replay_scores_are_deterministic ... ok

test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.42s
```

```
cargo clippy -p ferro-mcp --all-targets -- -D warnings
Finished `dev` profile — no warnings, no errors
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `generate_projection` in doc comments would fail D-06 acceptance criterion**
- **Found during:** Task 1 verification
- **Issue:** Doc comment on `call_dev_tool` and a comment in the smoke test body both mentioned `generate_projection` by name (documenting the D-06 exclusion). `grep -c 'generate_projection'` returned 3, failing the acceptance criterion requiring 0.
- **Fix:** Rewrote the doc comment to describe the constraint without naming the excluded tool; removed the comment block referencing it.
- **Files modified:** `ferro-mcp/tests/agent_harness.rs`
- **Commit:** 633bb5d7

**2. [Rule 1 - Bug] `ToolUseBlock` unused import**
- **Found during:** Task 2 compilation
- **Issue:** `ToolUseBlock` was listed in the `use ferro_ai::client::` import but not referenced in any code (the loop accesses blocks from `CompletionResponse::ToolUse { blocks, .. }` directly without naming the type).
- **Fix:** Removed `ToolUseBlock` from the import list.
- **Files modified:** `ferro-mcp/tests/agent_harness.rs`
- **Commit:** 5ba4c1d6

**3. [Rule 1 - Bug] `eprintln!` format string failed `uninlined_format_args` clippy lint**
- **Found during:** Task 2 clippy run
- **Issue:** `eprintln!("\n=== Baseline ===\n...", t1_rate, t2_rate, ...)` triggered `-D clippy::uninlined_format_args`.
- **Fix:** Inlined format arguments using `{t1_rate:.2}` style.
- **Files modified:** `ferro-mcp/tests/agent_harness.rs`
- **Commit:** 5ba4c1d6

**4. [Rule 1 - Bug] `CallToolRequestParam.name` requires `'static` lifetime**
- **Found during:** Task 1 compilation
- **Issue:** `name.into()` for a `&str` parameter produced a lifetime error — `Cow<'static, str>` requires `'static`.
- **Fix:** Used `name.to_owned().into()` to convert to `String` (which is `'static`).
- **Files modified:** `ferro-mcp/tests/agent_harness.rs`
- **Commit:** 633bb5d7

## Known Stubs

None. The live apparatus is fully wired: `spawn_in_process_client` uses the real `FerroMcpService`, `run_agent_trial` calls the real `AnthropicClient::complete_with_tools`, and the gated test writes real transcript + baseline files. The actual baseline-producing run is Wave 4 (autonomous: false) — not a stub, a deliberate Wave boundary.

## Threat Flags

No new network endpoints, auth paths, or schema changes introduced. T-210-08 (API key never logged) is mitigated: the key is bound to a local `api_key` variable and passed directly to `AnthropicClient::new()` with no intermediate logging.

## Self-Check

Files created/modified:
- `ferro-mcp/tests/agent_harness.rs` — FOUND

Commits:
- `633bb5d7` — FOUND (feat: in-process rmcp duplex transport)
- `5ba4c1d6` — FOUND (feat: agent tool-use loop + gated live eval test)

## Self-Check: PASSED
