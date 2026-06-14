# Phase 221: Inbound NL Intent Loop - Discussion Log

> **Audit trail only.** Decisions are in CONTEXT.md; this preserves alternatives considered.

**Date:** 2026-06-14
**Phase:** 221-inbound-nl-intent-loop
**Mode:** `--auto` (gray areas auto-selected; recommended defaults logged)
**Areas:** Loop home, Classification→routing, Clarification, Write→confirmation, Replay/live-eval, Feature gating, Result envelopes

---

| Area | Selected default | Alternatives rejected |
|------|------------------|-----------------------|
| Loop home | Testable core fn in ferro-mcp-server + `/mcp/chat` endpoint in sample app; `ToolSelection` in ferro-mcp-server | Loop entirely in app (SC#3/#4 not unit-testable without HTTP); ToolSelection in ferro-ai (it's projection-specific) |
| Classification → routing | `Classifier<ToolSelection>` (system=`render_tool_descriptions`, schema=ToolSelection) → existing `dispatch()`/`dispatch_write()`; no new dispatch logic (SC#1) | A classification-specific dispatch path (violates SC#1) |
| Clarification | Reuse Classifier `Error::LowConfidence` → `needs_clarification` structured response (SC#5) | New confidence/threshold logic in ferro-mcp-server (ferro-ai already does it) |
| Write → confirmation | Classified write → 220 confirmation gate before execute (SC#2); never direct dispatch_write for destructive | Bypass confirmation for "trusted" classification (unsafe) |
| Replay / live-eval | Reuse Phase 210 COMP-03 transcript-fixture + deterministic-replay-guard; FERRO_AI_LIVE_EVAL gate; cost-announce before first live call (SC#3/#4) | Invent a new replay mechanism (210 already solved no-key fixtures + determinism) |
| Feature gating | `intent` feature enabling ferro-ai(+llm live) + loop; replay provider reqwest-free so CI SC#3 runs llm-free (extends 220 D-06) | ferro-ai unconditional (drags reqwest); live in CI (spend) |
| Result envelopes | Reuse 219/220 `CallToolResult::structured`/`write_tool_error_result`; classified args UNTRUSTED → full 219 validation+guard+tenant (PITFALLS §3) | Trust classifier args / new envelope |

**Claude's Discretion:** render_tool_descriptions text format; ToolSelection schema field names; cost-estimate string; replay provider location (ferro-ai vs ferro-mcp-server tests).

**Deferred:** parameter-elicitation state machine; multi-turn memory; live-eval in CI; gestiscilo /mcp/chat adoption.
