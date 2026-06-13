# Project Research Summary

**Project:** ferro v15.0 — Agent-Operable App (Consumer MCP)
**Domain:** Projection-derived multi-tenant MCP endpoint with write/act capabilities and inbound NL intent loop
**Researched:** 2026-06-13
**Confidence:** HIGH

## Executive Summary

v15.0 extends an already-functional read-only MCP endpoint (shipped in v12.6/Phase 197) into a full write-and-act surface. The read path — `tools/list` returning projection-derived read tools, `tools/call` dispatching tenant-scoped SQL queries, bearer-token OAuth auth, and the Phase 205 `CallToolResult::structured` content-block fix — is complete and requires no rework. v15.0 adds five capabilities on top of this foundation: (1) write/action tools derived from `ActionDef` in `ServiceDef`, (2) server-side guard enforcement at execution time, (3) confirmation gating for destructive actions via `ferro-ai::ConfirmationStore`, (4) per-tenant API-key auth as an alternative to OAuth JWT, and (5) an inbound natural-language intent loop using `ferro-ai::Classifier<ToolSelection>`.

The architectural foundation is solid and mostly already built. `ferro-projections` provides `ServiceDef`, `ActionDef`, `GuardDef`, and `BaseContext.evaluated_guards` (v14.0/Phase 215). `ferro-mcp-server` has `McpRenderer` (already the correct output crate per the v11.5 boundary rule), `dispatch()` with fail-closed tenant scoping, and a JSON-RPC layer. `ferro-ai` provides `Classifier<T>` and `ConfirmationStore`. `ferro-mcp-oauth` provides the full bearer-token validation stack. The only new Cargo dependency is `ferro-ai` added to `ferro-mcp-server`, feature-flagged so read-only consumers do not pull LLM HTTP clients. The primary structural gap is `McpContext`, which is currently an empty struct with a "Phase 200 will extend" comment — it must embed `tenant_id` and `evaluated_guards` before any other phase can proceed.

The primary risks are security in nature rather than architectural: cross-tenant data leaks if the write dispatch path bypasses `TenantScoped`; guard bypass if guards are enforced only at `tools/list` and not re-evaluated at `tools/call` execution; and destructive actions executing without a confirmation round-trip. All three are non-negotiable — each must be resolved in the same phase that introduces the relevant capability, never deferred. The cost-discipline risk (live-LLM calls in the request path without a replay/smoke mode) is structural: the inbound intent loop must ship with `FERRO_AI_LIVE_EVAL=1` gating and replay fixtures in the same phase as the live path.

## Key Findings

### Recommended Stack

All v15.0 work lands in `ferro-mcp-server` (the existing MCP output crate). No new crates are needed. The only new Cargo.toml change is adding `ferro-ai = { path = "../ferro-ai", version = "0.2" }` to `ferro-mcp-server`, feature-flagged. `rmcp` stays pinned at `"0.12"` — no upgrade is warranted or safe (would break `ferro-mcp`, `ferro-mcp-server`, `ferro-api-mcp` simultaneously with no required feature benefit). All other required libraries (`sha2`, `subtle`, `sea-orm`, `schemars`, `serde_json`, `jsonwebtoken`, `tokio`, `tracing`) are already present in the relevant crates.

**Core technologies:**
- `rmcp 0.12` (pinned, no upgrade): MCP protocol types (`Tool`, `ToolAnnotations`, `CallToolResult`) — already integrated; `CallToolResult::structured()` is the mandatory result constructor for all tools
- `ferro-projections` (workspace): `ServiceDef`, `ActionDef`, `GuardDef`, `BaseContext.evaluated_guards` — single source of truth for all tool definitions; write tools derived from `ActionDef` via the same derivation pattern as read tools
- `ferro-ai` (workspace, new dep in `ferro-mcp-server`): `Classifier<T>` for NL→tool-selection classification, `ConfirmationStore` for destructive-action gating; provider-agnostic with Anthropic `claude-sonnet-4-6` default
- `ferro-mcp-oauth` (workspace): Bearer-token validation for OAuth JWT path; API-key branch (SHA-256 lookup) added in Phase 1 — same `BearerCheck::Authenticated(principal)` outcome regardless of path
- `sea-orm 1.0`: Write dispatch (INSERT, UPDATE, state transition) — already in `ferro-mcp-server`; action execution needs DB writes

### Expected Features

**Must have (table stakes):**
- Write tools for Collect-intent actions (create record) derived from `ActionDef`
- Write tools for Process-intent state-transition actions derived from `ActionDef.transition_trigger`
- Guard-filtered write-tool exposure via `evaluated_guards` in `McpContext` — guards re-enforced at `tools/call` execution, not only at `tools/list` listing time
- Two-step confirmation gate for destructive/money-moving actions via `ferro-ai::ConfirmationStore` (stable `confirm_<action>` tool, not per-invocation dynamic tool names)
- Idempotency-key parameter on all write tools — required for safe agent retry loops
- `destructiveHint` / `idempotentHint` `ToolAnnotations` on write tools (derived from `ActionDef` attributes, not hand-authored)
- Tenant scoping on write dispatch, fail-closed (mirrors existing read `dispatch()` pattern; `tenant_id` always from auth token, never from call payload)
- Per-tenant API-key auth with scope field (`read` vs. `read_write`) on the key model from issuance; OAuth JWT path unchanged
- Inbound NL classification via `ferro-ai::Classifier<ToolSelection>` wired to a `natural_language_query` tool, with replay/smoke mode gated on `FERRO_AI_LIVE_EVAL=1`

**Should have (differentiators):**
- Single `ServiceDef` source of truth: the same definition `JsonUiRenderer` and `TextRenderer` consume produces the MCP tools — zero parallel maintenance when a field or action is added
- Confirm-before-write in the NL loop for all action intents: classify → present proposed action → execute only on affirmative reply (confidence score is not a substitute for a confirmation round-trip)
- Structured tool results (`CallToolResult::structured`) with sensitive-field exclusion (omit `FieldMeaning::Password`, `Token`, `Secret`) to reduce prompt-injection surface
- Audit log entry per write tool call (tool name, tenant, action, param IDs) to support post-incident investigation

**Defer (v15.x or later):**
- Single-record Focus tool (`get_<entity>`) — useful but not required for write-path validation
- Update-record tool (modify without state transition) — defer if gestiscilo validation only exercises create and transition
- MCP elicitation for missing parameters — spec still in June 2025 draft
- Dry-run/preview tool variant — defer until confirmation UX is validated in practice
- Refresh-token rotation and long-session ergonomics
- Per-tool rate limiting beyond existing middleware

### Architecture Approach

The architecture is a pure extension of the existing MCP output crate. `McpRenderer` in `ferro-mcp-server/src/renderer.rs` already implements `Renderer<Output = Tool, Context = McpContext>` and is the correct home for all projection→MCP-tool rendering per the v11.5 boundary rule. Read tools stay as `list_<service_name>` with `readOnlyHint: true`. Write/action tools are added as one additional `Tool` per guard-passing `ActionDef`, with name `<action.name>` (or `<action.name>_on_<service.name>` on collision), `readOnlyHint: false`, and `destructiveHint` derived from `action.transition_trigger.is_some()`. The inbound NL intent loop lives in the consumer application's handler layer, not in `ferro-mcp-server` itself — the server exposes `dispatch_write()` and a `render_tool_descriptions()` helper; the loop wiring is app-layer.

**Major components:**
1. `McpContext { tenant_id: Option<i64>, evaluated_guards: HashMap<String, bool> }` — foundational context struct (currently empty, extended in Phase 1); threaded from auth middleware through every tool rendering and dispatch call
2. `ferro-mcp-server/src/auth.rs` — unified tenant resolution: OAuth JWT branch + API-key SHA-256 branch; both produce `tenant_id: i64` via the same `BearerCheck` outcome type
3. `ferro-mcp-server/src/renderer.rs` + `schema.rs` — extended `McpRenderer`: read tools (existing) + `render_action_tool()` emitting guard-filtered write tools with `build_action_input_schema(action, service)`
4. `ferro-mcp-server/src/write_dispatch.rs` (new file) — `dispatch_write()`: input validation against `ActionDef.inputs`, guard re-evaluation at execution, confirmation gating, app callback invocation
5. `ferro-mcp-server/src/jsonrpc.rs` — extended `handle_write_call()` routing non-`list_*` tool names to `dispatch_write`
6. Consumer app `src/handlers/mcp_chat.rs` — NL intent loop: `Classifier<ToolSelection>` → guard check → `ConfirmationStore` gate → `dispatch_write`

### Critical Pitfalls

1. **Cross-tenant tool leak** — write dispatch calls a bare SeaORM query without injecting the tenant predicate; an agent authenticated as tenant A receives records belonging to tenant B. Prevention: route every write through `TenantScoped` contract; the cross-tenant isolation test fixture (`mcp_tenant_isolation.rs`, Phase 200) must be extended to cover every new write tool in the same commit that adds it.

2. **Server-side guard bypass** — guards enforced only at `tools/list` (filtering which tools appear) but not re-evaluated at `tools/call` execution; a crafted MCP request calls a guarded action directly. Prevention: `tools/call` dispatch re-evaluates the guard using live DB state before any write; a test must call a guarded tool directly via `tools/call` and assert an MCP error response.

3. **Destructive write without confirmation** — a state-transition or delete action executes immediately on a single `tools/call`; an agent retry or misclassification executes it twice or on the wrong record. Prevention: idempotency keys on every write tool; two-step `confirm_<action>` protocol for destructive actions; confirm-before-write in the NL loop for all action intents.

4. **MCP protocol drift (content-block regression)** — a new write-path tool result constructs a bare `content[]` array without `type` field, reproducing the Phase 205 bug. Prevention: all `tools/call` responses use `CallToolResult::structured()`; Phase 205 rmcp strict-deserialization regression test extended to cover every new tool in the same phase.

5. **Live-LLM cost without replay path** — the inbound NL loop calls the LLM on every run including CI; credit exhaustion mid-test leaves results incomplete (COMP-03: ~$21 wasted). Prevention: replay/smoke mode shipping in the same phase as the live path; `FERRO_AI_LIVE_EVAL=1` gating all live calls.

## Implications for Roadmap

All four research files converge on a strictly ordered five-phase build sequence. The dependency graph is tight: each phase is blocked by the previous. Phases 1–3 are the write-path backbone; Phase 4 adds the safety layer; Phase 5 adds the NL entry point that consumes all previous layers.

### Phase 1: Auth Foundation + McpContext Extension

**Rationale:** `McpContext` is currently an empty struct and `BearerOutcome` in `auth.rs` is a stub. Every downstream phase needs a real `tenant_id` threaded through context and real guard evaluation. Nothing else can be built on stubs.
**Delivers:** Real tenant resolution from OAuth JWT (`validate_bearer`) and API key (SHA-256 DB lookup); `McpContext { tenant_id: Option<i64>, evaluated_guards: HashMap<String, bool> }`; API-key scope field (`read` vs. `read_write`) on the key model.
**Addresses:** Per-tenant API-key auth, guard-filtering substrate.
**Avoids:** API-key scope creep pitfall (scope on key model at issuance, not retrofitted); cross-tenant pitfall (tenant_id from auth token, never from call payload).
**Open question:** Does `ferro make:api-key` (v8.1) already provide an `api_keys` table to reuse? Verify before designing the schema.

### Phase 2: Write-Tool Rendering (ActionDef → MCP Tool)

**Rationale:** Write tools must appear in `tools/list` before they can be called. This phase extends `McpRenderer` and `schema.rs` to derive write tools from `ActionDef`, applying guard filtering from `McpContext.evaluated_guards`. Depends on Phase 1 for the context struct.
**Delivers:** `render_action_tool(service, action, ctx) -> Tool`; `build_action_input_schema(action, service)`; extended `render_exposed_tools()` emitting both read and write tools; `destructiveHint`/`idempotentHint` annotations derived from `ActionDef` attributes.
**Implements:** McpRenderer extension, schema.rs action-input schema derivation.
**Avoids:** MCP protocol drift (Phase 205 regression test extended); McpRenderer scope-creep (all tool definitions traced to `ServiceDef` attributes, no hand-authored branches).

### Phase 3: Write Dispatch (Action Execution)

**Rationale:** Write tools registered in Phase 2 must be callable. This phase adds `dispatch_write()` and extends `handle_tools_call()`. Guards are re-evaluated at execution time in this phase — not deferred.
**Delivers:** `ferro-mcp-server/src/write_dispatch.rs` with `dispatch_write()`; `handle_write_call()` in `jsonrpc.rs`; idempotency-key parameter and enforcement; audit log entry per write tool call; `GuardFailed`/`ActionNotFound` error variants.
**Uses:** SeaORM for write operations; `TenantScoped` contract for fail-closed tenant scoping.
**Avoids:** Server-side guard bypass (re-evaluated at execution, live DB state); cross-tenant tool leak; hallucinated-parameter pitfall (audit log, tool descriptions require explicit ID retrieval).
**Open question:** Callback contract for write dispatch — HTTP POST to app's own route vs. registered Rust async callback. Decide before implementation.

### Phase 4: Confirmation Gating

**Rationale:** Destructive actions must not be single-step. This phase wraps `dispatch_write()` with `ConfirmationStore` gating and synthesizes stable `confirm_<action>` tools. Depends on Phase 3.
**Delivers:** `ConfirmationStore` integration in `write_dispatch.rs`; stable `confirm_<action>` tool per destructive action; `ferro-ai` dep added to `ferro-mcp-server` with feature flag; configurable TTL (5–10 min) via `McpServerConfig`.
**Uses:** `ferro-ai::ConfirmationStore` + `InMemoryConfirmationStore`.
**Avoids:** Destructive write without confirmation; double-execution on retry (idempotency + confirmation gate together).
**Open question:** Is `destructive` an explicit `bool` field on `ActionDef` or inferred from `transition_trigger.is_some()`? Heuristic may be insufficient for delete-category actions that do not use the state machine. Decide before the phase begins. Also: assess whether `InMemoryConfirmationStore` is sufficient for v15.0 or whether a DB-backed store is needed for rolling-restart deployments.

### Phase 5: Inbound NL Intent Loop

**Rationale:** The NL entry point consumes all previous capabilities. It is the most integration-sensitive piece and must be last so any failure is isolated to the loop wiring, not the underlying tool machinery.
**Delivers:** Consumer app `src/handlers/mcp_chat.rs` — `POST /mcp/chat` endpoint; `Classifier<ToolSelection>` → guard check → `ConfirmationStore` gate → `dispatch_write`; replay/smoke path from transcript fixtures gated on `FERRO_AI_LIVE_EVAL=1`; `render_tool_descriptions()` helper in `ferro-mcp-server/src/lib.rs`.
**Uses:** `ferro-ai::Classifier<T>` with `ToolSelection { tool_name, confidence, arguments }` defined in `ferro-mcp-server`; provider-agnostic via `AiConfig::from_env()`.
**Avoids:** Live-LLM cost pitfall (replay mode ships in same phase); NL misclassification executing wrong action (confirm-before-write for all action intents; clarification returned on low confidence, not direct dispatch); prompt injection (structured results + sensitive-field exclusion from Phase 2).
**Note:** Classify directly to `ToolSelection` (tool name + arguments), not via an intermediate Intent step. Direct classification is more robust and simpler.

### Phase Ordering Rationale

- Phase 1 is a hard prerequisite for all others: `McpContext` and real auth are the only structural dependencies shared by every subsequent phase.
- Phase 2 before Phase 3: write tools must exist in `tools/list` before `handle_write_call` can look them up.
- Phase 3 before Phase 4: `dispatch_write()` must exist before `ConfirmationStore` can gate it.
- Phase 4 before Phase 5: the NL loop must reach the confirmation gate for any action intent; shipping the loop before Phase 4 is complete means the loop ships without its safety layer.
- Phase 5 last: it is the most failure-prone piece to isolate; keeping it last means a Phase 5 failure is diagnosable without questioning whether Phases 1–4 are working.

### Research Flags

Phases needing a targeted verification before implementation begins:

- **Phase 1:** Whether `ferro make:api-key` (v8.1) produced a reusable `api_keys` table in the framework or app. One `grep`/`find` command resolves this.
- **Phase 3:** Callback contract design decision (HTTP POST vs. registered Rust async fn). The architectural choice affects the registration API surface exposed by `ferro-mcp-server`.
- **Phase 4:** `ActionDef.destructive` field vs. heuristic inference. This is a public `ferro-projections` API change and should be validated against how existing `ServiceDef` definitions in the sample app are written.

Phases with well-established patterns (no per-phase research sprint needed):

- **Phase 2:** Write-tool rendering directly mirrors the existing read-tool rendering pattern in `ferro-mcp-server/src/renderer.rs` + `schema.rs`. Pattern is verified from source.
- **Phase 5:** `ferro-ai::Classifier<T>` API verified from direct code read. Replay/smoke mode design is fully specified in PITFALLS research.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All technology choices verified from direct codebase reads; rmcp 0.12 API confirmed via Context7 |
| Features | HIGH (read path), MEDIUM (write safety UX) | Read path: working code. Write-path confirmation interaction varies by MCP client; patterns clear but exact UX not fully standardized |
| Architecture | HIGH | Component boundaries verified from source files; v11.5 boundary rule confirms `ferro-mcp-server` as the correct output crate; no new crates needed |
| Pitfalls | HIGH | Grounded in live incidents (Phase 205 content-block bug, Phase 200 tenant isolation, COMP-03 cost exhaustion) and OWASP BOLA/guard-bypass classifications |

**Overall confidence:** HIGH

### Gaps to Address

- **`api_keys` table existence:** Unverified whether `ferro make:api-key` (v8.1) created a reusable table. Check before Phase 1 schema design. Determines whether Phase 1 includes a new migration.
- **Write dispatch callback contract:** HTTP POST to app's own route vs. registered Rust async callback. Decision needed before Phase 3. Affects registration API surface.
- **`ActionDef.destructive` vs. heuristic:** Whether `destructive` becomes an explicit `bool` on `ActionDef` needs a design decision before Phase 4. Impacts `ferro-projections` public API.
- **Confirmation TTL:** 60s default in `InMemoryConfirmationStore` examples is too short for multi-turn agent conversations. Correct default (5–10 min suggested) should be validated against real interaction latency before Phase 4 locks `McpServerConfig`.
- **`ConfirmationStore` persistence:** `InMemoryConfirmationStore` is lost on restart. Assess in Phase 4 whether a DB-backed store is required for v15.0 production deployments or whether in-memory is acceptable.

## Sources

### Primary (HIGH confidence)

- `ferro-mcp-server/src/renderer.rs`, `dispatch.rs`, `jsonrpc.rs`, `schema.rs`, `auth.rs` — existing MCP server implementation, tenant scoping, Phase 205 content-block fix
- `ferro-projections/src/render/mod.rs`, `service.rs`, `action.rs` — `Renderer` trait, `BaseContext.evaluated_guards`, `ServiceDef`, `ActionDef`, `GuardDef`
- `ferro-mcp-oauth/src/validate.rs` — `validate_bearer`, `BearerCheck`, `McpTokenClaims`
- `ferro-ai/src/classifier/mod.rs`, `config.rs`, `confirmation/mod.rs` — `Classifier<T>`, `AiConfig`, `ConfirmationStore`
- Context7 `/websites/rs_rmcp` — `Tool::new`, `ToolAnnotations`, `CallToolResult::structured` API surface
- `.planning/PROJECT.md` — v15.0 milestone scope and capability list
- `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md` — v12.6 OAuth MCP design spec

### Secondary (MEDIUM confidence)

- MCP Tool Annotations specification (2026-03-16) — `readOnlyHint`, `destructiveHint`, `idempotentHint` semantics
- MCP elicitation draft (June 2025) — parameter elicitation protocol (deferred to v15.x; spec still in draft)
- COMP-03 baseline (Phase 210, partial) — live-LLM cost pattern and replay/smoke path requirement

### Tertiary (LOW confidence)

- NL misclassification failure modes — based on general LLM agent reliability research; ferro-specific misclassification rates not yet measured

---
*Research completed: 2026-06-13*
*Ready for roadmap: yes*
