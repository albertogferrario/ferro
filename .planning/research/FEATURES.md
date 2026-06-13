# Feature Research

**Domain:** Agent-operable consumer MCP endpoint for a Rust web framework
**Researched:** 2026-06-13
**Confidence:** HIGH (read-path and guard exposure); MEDIUM (write-path safety, NL loop — active community convergence, patterns clear but exact confirmation UX varies by client)

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features that any agent-operable app endpoint must have. Missing these makes the endpoint unusable or unsafe.

| Feature | Why Expected | Complexity | Dependency on Existing Capability | Notes |
|---------|--------------|------------|-----------------------------------|-------|
| **tools/list returns projection-derived tools** | MCP protocol requires tools/list; tools must reflect what the tenant can actually do | LOW | `render_exposed_tools` + `ServiceDef.mcp_exposed` already in `ferro-mcp-server` (Phase 197) | Read tools exist; write tools are the v15.0 addition |
| **tools/call for read (list/filter with pagination)** | Core query path; agents must retrieve data to act on it | LOW | `dispatch()` in `ferro-mcp-server` is complete with tenant isolation and filter allowlisting | Already built; v15.0 extends to write |
| **tools/call for write — create record** | The primary agent action; without it the endpoint is read-only only | MEDIUM | `ActionDef` + `InputDef` in `ferro-projections`; `ActionDef.inputs` map directly to tool input schema | New in v15.0; requires `McpRenderer` to emit write-tool definitions |
| **tools/call for write — update record** | Agents must be able to modify existing records, not only create | MEDIUM | Same as create; `ActionDef` with transition_trigger maps to update/state-transition actions | New in v15.0 |
| **tools/call for write — state transition** | State machine actions (approve, reject, cancel) are the core of process workflows | MEDIUM | `StateMachine` + `Transition` in `ferro-projections`; `ActionDef.transition_trigger` links action to transition | New in v15.0 |
| **Guard-filtered tool exposure** | An agent must not be offered tools it cannot invoke; presenting them invites confusion and errors | MEDIUM | `BaseContext.evaluated_guards` (v14.0) + `ServiceDef.guards` + `ActionDef.preconditions`; filter logic analogous to `TextRenderer` guard path | Guards already evaluated for visual/text renderers; `McpContext` must carry `evaluated_guards` for write tools |
| **Tenant scoping: one API key = one tenant's toolset and data** | Cross-tenant access is a critical security property | LOW | `ServiceDef.tenant_column` + `dispatch()` fail-closed logic + `TenantScoped` trait (v13.1) + `ferro-mcp-oauth` bearer token carrying `(user, tenant)` | Already structurally enforced in read path; write path must inherit the same guarantee |
| **Per-tenant API-key auth** | Callers must authenticate to a specific tenant before any tool is visible | LOW-MEDIUM | `ferro-mcp-oauth` OAuth 2.1 crate (v12.6 walking skeleton): bearer validation + `McpTokenClaims` carrying `(user, tenant)` | Auth transport exists; v15.0 reuses it and adds API-key as an alternative simpler auth path |
| **Tool input schema derived from ServiceDef** | Agents need structured schemas to call tools correctly; hand-written schemas drift | LOW | `crate::schema::build_input_schema` in `ferro-mcp-server` already derives read-tool schemas from `ServiceDef.fields`; write tools add `ActionDef.inputs` as the input schema source | Already proven for read path; write path maps `InputDef` to JSON Schema properties |
| **MCP protocol error envelopes (code, message)** | MCP clients expect well-formed JSON-RPC error objects on failure | LOW | `ferro-mcp-server` `jsonrpc.rs` already returns `-32601`/`-32602`/`-32603` envelopes | Must extend for new error cases (action failed, guard denied, confirmation pending) |
| **Idempotency on write tools** | Agents retry on network failure; duplicate creates must not duplicate records | MEDIUM | `ferro-queue` has `idempotency_key()` hook; write dispatch needs an idempotency-key parameter in tool input schema | New in v15.0; map to a `UNIQUE` constraint or `INSERT OR IGNORE` pattern at dispatch time |
| **Policy denial returns clean error, no data disclosure** | Leaking unauthorized data in error messages is a confidentiality violation | LOW | Existing pattern in `dispatch()` and `jsonrpc.rs`; error content must say what was denied, not what exists | Extend the same discipline to write-path denials |
| **`readOnlyHint: false` + `destructiveHint` annotations on write tools** | MCP spec defines `ToolAnnotations`; clients use these to decide confirmation UI | LOW | `rmcp::model::ToolAnnotations` is already used for `readOnlyHint: true` on read tools (Phase 197); write tools must set `readOnlyHint: false`, state-transition and delete tools must set `destructiveHint: true` | Pure schema annotation; no new dependency |

### Differentiators (Competitive Advantage)

Features that are not table stakes but define the value of this endpoint over hand-written MCP tool servers.

| Feature | Value Proposition | Complexity | Dependency on Existing Capability | Notes |
|---------|-------------------|------------|-----------------------------------|-------|
| **Tools auto-derived from ONE ServiceDef (single source of truth)** | The same definition the visual renderer (JsonUiRenderer) and text renderer (TextRenderer) consume produces the MCP tools. A field added to the ServiceDef appears immediately in the visual UI, the conversational text output, and the MCP tool schema without any additional authoring. Zero parallel maintenance. | MEDIUM | `Renderer` trait + `ServiceDef` in `ferro-projections`; `McpRenderer` implements `Renderer<Output=Tool, Context=McpContext>` mirroring `JsonUiRenderer` and `TextRenderer` | This is the architectural killer feature. Common baseline (Laravel MCP, generic MCP servers, `ferro-api-mcp`) all hand-write tools or map OpenAPI routes — none derive from a shared projection schema that also drives visual rendering. |
| **Guard-filtered write tools via evaluated_guards** | An agent only sees (and can invoke) the actions its tenant's policy permits. The guard evaluation that filters the visual UI and conversational text output also filters which MCP write tools are emitted in `tools/list`. This is structural, not a per-tool permission check. | MEDIUM | `BaseContext.evaluated_guards` (v14.0); `ServiceDef.guards` + `ActionDef.preconditions`; `McpContext` must embed a guard map (Phase 200 stub identified in code as `// Phase 200 will extend with tenant/policy context`) | `McpContext` currently carries no state. This is the Phase 200 work. Guards already evaluated for visual and text renderers; MCP inherits the same evaluation path, not a new system. |
| **Inbound NL intent loop (message → classify → elicit → confirm → execute)** | A tenant can issue a natural-language command and the endpoint maps it to an action, elicits missing parameters, optionally confirms, and executes — completing the listen/act half deferred from v14.0. | HIGH | `ferro-ai::Classifier` (structured classification) + `ConfirmationStore` (TTL-gated pending actions) + `ferro-ai::ToolRegistry` (tool dispatch) — all already built | The `Classifier` maps NL to a `ferro_projections` intent/action. The `ConfirmationStore` gates destructive actions. `ToolRegistry` dispatches. These are all built; the wiring into the MCP message path is new. Most complex v15.0 feature. |
| **Dry-run/preview tool variant for destructive actions** | Before a delete or irreversible state transition executes, an agent can call a `preview_<action>` tool that returns what would change without committing. Closes the loop on human-in-the-loop without requiring a round-trip outside the MCP session. | MEDIUM-HIGH | `ActionDef.effects` lists the effects; preview needs a read-only shadow of the write dispatch path | Not in common baseline. Reduces the confirmation burden on the human by making the consequence visible to the agent before execution. Defer to a later phase if write-path confirmation via `ConfirmationStore` proves sufficient in practice. |
| **Write-path confirmation via `ConfirmationStore` with TTL expiry** | When a write action is classified as destructive or money-moving, the tool call parks the payload behind a key instead of executing immediately. A second tool call with the key within the TTL window executes the action. Expiry dispatches `ConfirmationExpired` via `ferro-events`. | MEDIUM | `ferro-ai::ConfirmationStore` + `InMemoryConfirmationStore` + `ConfirmationExpired` event — all already built | The confirmation primitive is complete. v15.0 wires it to write-tool dispatch. The two-step interaction (propose then confirm) maps naturally to a conversational agent turn. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Separate MCP permission model (MCP-specific scopes beyond existing policies)** | Seems like fine-grained control per MCP tool | Dual control surface: existing policy layer plus MCP layer diverge over time, creating confusion about what is actually permitted. The v12.6 design spec explicitly rules this out. | Reuse the existing multi-tenant middleware and policy layer. An agent's reach equals the authenticated user's reach by construction. `ServiceDef.mcp_ability` provides an opt-in capability check without a second permission system. |
| **Auto-exposure of all projections** | Convenient — "expose everything" | A projection authored for internal tooling or admin UI is silently public to any token holder. A data migration projection could be exposed accidentally. | `mcp_exposed: bool` explicit opt-in per `ServiceDef`. Only marked projections appear in `tools/list`. This field already exists in the codebase. |
| **MCP-specific handler code per action** | Seems like flexibility | Defeats the single-source-of-truth property: the MCP tool and the HTTP handler diverge. Authorization logic may be duplicated or skipped. | Write tool dispatch runs the same action logic as the HTTP surface, gated by the same guards. `ActionDef` in the `ServiceDef` is the contract; the MCP endpoint is a rendering target, not a parallel implementation. |
| **Streaming/SSE write results** | Low-latency feedback for long writes | Write actions in a projected service are expected to be fast (validation then DB write then response); streaming adds transport complexity without benefit for this case. SSE streaming exists for LLM token output, not action results. | Synchronous response with structured result envelope. If a write is genuinely long-running, route it through `ferro-queue` as a background job and return a job ID. |
| **Refresh token rotation and long-session ergonomics in v15.0** | Convenient for persistent agent sessions | The auth transport (`ferro-mcp-oauth`, v12.6 walking skeleton) is not the focus of v15.0; spending design effort on token rotation before write-path action dispatch is proven correct is premature. | Short-lived access tokens (v12.6 baseline). Token refresh ergonomics are explicitly deferred per the v12.6 design spec non-goals. |
| **Per-tool rate limiting beyond existing middleware** | Prevents abuse | A second rate limiting surface that diverges from the per-user/per-tenant limits already applied by the application middleware. Two surfaces that can contradict each other. | Reuse the existing `RateLimiter` middleware. It applies to the `/mcp` route group as it does to any other. |

---

## Feature Dependencies

```
ServiceDef.mcp_exposed + ServiceDef.actions + ServiceDef.guards
    └──required by──> McpRenderer (write tool generation)
                          └──required by──> tools/list (write tools visible)
                                               └──required by──> tools/call (write execution)

BaseContext.evaluated_guards (v14.0)
    └──required by──> McpContext with guard map (Phase 200 stub)
                          └──required by──> Guard-filtered tool exposure
                                               └──required by──> Write-path guard denial

ferro-ai::ConfirmationStore (already built)
    └──required by──> Write-path confirmation gate
                          └──required by──> Inbound NL loop (confirm step)
                                               └──required by──> NL to execute end-to-end

ferro-ai::Classifier (already built)
    └──required by──> Inbound NL intent loop
                          └──required by──> NL message to action dispatch

ferro-mcp-oauth bearer token carrying (user, tenant) (v12.6)
    └──required by──> Per-tenant API-key auth
                          └──required by──> All tool calls (read and write)

dispatch() fail-closed tenant predicate (already built, Phase 197)
    └──extends to──> Write dispatch (create/update/transition)
                         └──required by──> Tenant scoping on write path

ActionDef.preconditions + GuardDef (already in ServiceDef)
    └──required by──> Guard-filtered write tool exposure
                          └──enhances──> Write-path guard denial

rmcp::model::ToolAnnotations (already used for readOnlyHint on read tools)
    └──extends to──> destructiveHint on write/delete tools
```

### Dependency Notes

- **Guard-filtered tool exposure requires McpContext to carry evaluated_guards:** Phase 197 left `McpContext` as an empty struct with the comment "Phase 200 will extend with tenant/policy context." v15.0 is that work. The pattern is established in `TextRenderer` (uses `BaseContext.evaluated_guards`); `McpContext` adopts the same map, evaluated once per request by the HTTP middleware before `handle_tools_list`.
- **Write dispatch requires ActionDef inputs to map to JSON Schema:** `InputDef` already has `name`, `data_type`, `required`, `description`. `build_input_schema` in `ferro-mcp-server` must be extended to handle `ActionDef.inputs` the same way it handles `FieldDef` filter parameters. Medium complexity; the mapping pattern exists in the read path.
- **Inbound NL loop requires Classifier before ConfirmationStore:** Classification (NL to intent/action name) must succeed before the confirmation gate is reached. Low-confidence classification should return an error or elicitation prompt, not proceed to confirming the wrong action.
- **ConfirmationStore TTL must match agent session expectations:** The default TTL in `InMemoryConfirmationStore` (60s in examples) may be too short for an interactive agent conversation. Configurable TTL via `McpServerConfig`. This is a configuration question, not a new API.

---

## Write-Path Safety and Confirmation Behavior

This is the highest-risk feature area and the most likely to need its own requirements.

### What counts as "destructive or money-moving"

The `ActionDef` on a `ServiceDef` has `effects: Vec<String>`. Effects like `"delete_record"`, `"charge_payment"`, `"send_email"`, `"cancel_subscription"` are irreversible or financially consequential. The framework cannot enumerate these exhaustively; instead, the classification is declarative:

1. **State-transition actions** (`ActionDef.transition_trigger` is `Some`) that lead to a terminal `StateDef` (no outgoing transitions) are treated as destructive by default.
2. **Any action whose `effects` list is non-empty** is treated as non-idempotent (set `idempotentHint: false` in `ToolAnnotations`).
3. **Delete-category actions** (effect name matches `"delete"`, `"remove"`, `"cancel"`, `"terminate"` by convention, or tagged via a `ActionDef.destructive: bool` field if needed) set `destructiveHint: true` in `ToolAnnotations`.
4. **Money-moving actions** (effect includes `"charge"`, `"refund"`, `"transfer"`) are treated as destructive for confirmation purposes.

### The two-step confirmation protocol via ConfirmationStore

When a write tool is called and its action is classified as requiring confirmation:

**Step 1 — first tools/call:** The handler generates a `confirmation_key` (UUID), stores the action payload in `InMemoryConfirmationStore` with a TTL, and returns an MCP tool result with `isError: false` containing:

```json
{
  "confirmation_required": true,
  "confirmation_key": "<uuid>",
  "action": "<action_name>",
  "payload_summary": "<human-readable description of what will happen>",
  "expires_in_seconds": 120
}
```

**Step 2 — confirm tool call:** A second tool — `confirm_<action>` — accepts the `confirmation_key`. If the key is valid and not expired, the action executes and returns the result. If the key has expired, the caller receives a `ConfirmationExpired` error. If the key is not found, the caller receives `"confirmation_expired_or_invalid"` — never a silent proceed.

The `ConfirmationExpired` event fires via `ferro-events` when TTL lapses without confirmation, enabling the application to clean up any pre-reserved state (for example, `ferro-reservation` holds).

**Why two tools, not a `confirmed: bool` argument:** A single `tools/call` with a `confirmed: bool` argument would let an agent auto-confirm by setting `confirmed: true` without human oversight. The two-tool pattern forces a distinct conversational turn, making it structurally harder for an agent to skip the confirmation step in a single chain-of-thought execution.

### Guard-denied vs confirmation-required: distinct failure modes

- **Guard denied:** the action's preconditions are not met for the current tenant/state (`evaluated_guards` returns `false` for a guard named in `ActionDef.preconditions`). The tool is not emitted in `tools/list` in the first place (guard-filtered exposure). If called directly via a replayed request, `tools/call` returns `isError: true` with a clear message. No data disclosure.
- **Confirmation required:** the action's preconditions pass, but the action is classified as destructive or money-moving. The tool is visible and callable; the first call parks the payload and returns the `confirmation_required` result; the confirm tool executes.

These two must not be conflated. A tool that is both guard-denied and confirmation-required never reaches the confirmation step — the guard check is earlier in the chain.

### Idempotency key

Write tools include an optional `idempotency_key: String` parameter in their `inputSchema`. If provided and a write with the same key has already succeeded within a configurable window, the call returns the previous result without re-executing. Implementation: a `UNIQUE` constraint on an `idempotency_keys` table, analogous to `ferro-queue`'s idempotency hook. Required for safe agent retry loops on network failure.

### Fail-closed defaults

Matching the read-path discipline in `dispatch()`:
- A write tool whose action's guard set cannot be evaluated (no `evaluated_guards` map in context) is denied, not permitted.
- A write tool for a tenant-scoped service called without a `tenant_id` in context is denied, not permitted.
- A write tool call with an unrecognized `confirmation_key` returns `isError: true` — never silently proceeds.

---

## Read Path — Behavior Specification

### tools/list for read tools

`tools/list` returns one tool per `mcp_exposed` projection:
- Tool name: `list_<service.name>` (existing behavior, Phase 197)
- `inputSchema`: filter fields from `ServiceDef.fields` where `is_filter_field(field)` returns true, plus `limit` and `offset` (existing behavior)
- `ToolAnnotations.readOnlyHint: true` (existing)
- Guards are not evaluated on read-tool exposure (read tools have no `preconditions`); guard filtering applies to write tools only

### tools/call for read (list with filters)

`tools/call` on `list_<entity>` applies the filter allowlist, injects the tenant predicate (fail-closed), and returns `{ rows, total, limit, offset }`. Already built and tested in Phase 197.

A single-record fetch (`get_<entity>`) derived from a Focus-intent projection is a natural extension following the same dispatch pattern. It is a P2 feature for v15.x.

---

## Inbound NL Intent Loop — Behavior Specification

The conversational turn sequence:

1. **Receive message:** via a `natural_language_query` tool call carrying a free-text string.
2. **Classify intent and action:** `ferro-ai::Classifier` maps the message to a `(ServiceDef name, ActionDef name)` pair using the projection catalog as the classification schema. Returns a `ClassificationResult` with a `confidence` score.
3. **Low confidence:** if `confidence < threshold` (default `0.7` per existing `ClassifierConfig`), return a clarification prompt to the agent instead of proceeding.
4. **Elicit missing parameters:** if the matched `ActionDef.inputs` have required fields not inferable from the message, return an elicitation request listing the missing fields and their expected types. MCP elicitation primitive (June 2025 spec draft) is the transport for this.
5. **Confirm if destructive:** if the classified action is in the destructive category, proceed to the two-step `ConfirmationStore` protocol described above.
6. **Execute:** call the write dispatch path with the collected parameters and the tenant context from the bearer token.
7. **Render result:** serialize the action result using `TextRenderer` on the updated `ServiceDef` state, returning a human-readable summary as MCP tool content.

**Dependency chain:** `ferro-ai::Classifier` (classify) → guard check (evaluate_guards) → `ferro-ai::ConfirmationStore` (if destructive) → write dispatch → `ferro-text::TextRenderer` (render result).

---

## Tenant Scoping — Behavior Specification

One API key authenticates exactly one `(user, tenant)` pair:

- The bearer token (from `ferro-mcp-oauth`) carries `McpTokenClaims { user_id, tenant_id, ... }`.
- The `/mcp` middleware extracts `tenant_id` and injects it into the request context, exactly as the multi-tenant HTTP middleware does for web routes.
- `tools/list` is evaluated in the context of that tenant: `ServiceDef.mcp_exposed` projections are filtered, guard maps are evaluated for that tenant's user, only permitted write tools are emitted.
- `tools/call` executes with `tenant_id` as a bound parameter on every data query (fail-closed via existing `dispatch()` logic).
- No tool in the session can return data belonging to a different tenant. This is structural, not a per-tool check.

What the agent sees: a tool surface scoped to one operator's data and permissions. A second API key for a second tenant produces a second, fully independent session with its own guard evaluation and data scope.

---

## MVP Definition (v15.0 Scope)

### Launch With (v15.0)

- [ ] Write tools for Collect-intent actions (create record) — derived from `ActionDef` in `ServiceDef`
- [ ] Write tools for Process-intent state-transition actions — derived from `ActionDef.transition_trigger`
- [ ] Guard-filtered write-tool exposure via `evaluated_guards` in `McpContext`
- [ ] Two-step confirmation gate for destructive/money-moving actions via `ConfirmationStore`
- [ ] Idempotency-key parameter on all write tools
- [ ] `destructiveHint` / `idempotentHint` annotations on write tools (annotation only, no new API)
- [ ] Tenant scoping on write dispatch (fail-closed, mirrors read path)
- [ ] Inbound NL classification via `ferro-ai::Classifier` wired to a `natural_language_query` tool
- [ ] Per-tenant API-key auth (reusing `ferro-mcp-oauth` bearer validation)

### Add After Validation (v15.x)

- [ ] Single-record Focus tool (`get_<entity>`) — useful but not required for write-path validation
- [ ] Update-record tool (modify existing record without state transition) — distinct from create and transition; defer if gestiscilo validation only needs create and transition
- [ ] Elicitation flow for missing parameters — MCP elicitation spec is June 2025 draft; validate core write path first
- [ ] Dry-run/preview tool variant — adds observability but increases API surface; defer until confirmation UX is validated in practice

### Future Consideration (v2+)

- [ ] Configurable MCP-specific capability scopes beyond existing policies
- [ ] Streaming write results for long-running operations
- [ ] Refresh-token rotation and persistent agent sessions
- [ ] Cross-tenant aggregate views for multi-tenant admin agents
- [ ] Voice/audio rendering of action results (separate channel milestone)

---

## Feature Prioritization Matrix

| Feature | Agent/User Value | Implementation Cost | v15.0 Priority |
|---------|-----------------|---------------------|----------------|
| Write tools (create + transition) derived from ServiceDef | HIGH | MEDIUM | P1 |
| Guard-filtered tool exposure via evaluated_guards | HIGH | MEDIUM | P1 |
| Two-step confirmation for destructive actions | HIGH | MEDIUM | P1 — highest-risk area, must be correct before shipping |
| Tenant scoping on write dispatch (fail-closed) | HIGH | LOW (extend existing pattern) | P1 |
| destructiveHint + idempotentHint ToolAnnotations | MEDIUM | LOW | P1 (annotation only, low effort) |
| Idempotency key on write tools | MEDIUM | MEDIUM | P1 |
| Inbound NL intent loop (walking skeleton) | HIGH | HIGH | P1 (core differentiator) |
| Single-record Focus tool (get) | MEDIUM | LOW | P2 |
| Update-record tool | MEDIUM | MEDIUM | P2 |
| MCP elicitation for missing parameters | LOW-MEDIUM | HIGH | P3 (spec still draft) |
| Dry-run/preview tool | MEDIUM | MEDIUM-HIGH | P3 (defer to validation results) |

**Priority key:** P1 = must have for v15.0 validation; P2 = add after core confirmed working; P3 = future consideration.

---

## Common Baseline (Industry Reference)

The following patterns are observed across other frameworks exposing apps to agents and define the minimum bar ferro must meet or exceed:

- **Laravel MCP (September 2025):** Tools are hand-written as PHP `Tool` classes and registered on a server. Tools can be given per-route dynamic resolvers. No projection-derived schemas. No single source of truth shared with the visual renderer. Read and write tools are both supported.
- **Generic OpenAPI-to-MCP bridges (e.g. `ferro-api-mcp`, `openapi-mcp`):** Map OpenAPI operation names to MCP tool names. Schemas come from OpenAPI parameter/request-body definitions. Tool schemas drift when the OpenAPI spec diverges from implementation.
- **Common patterns all implementations share:** `tools/list` must return valid JSON Schema per input; `tools/call` must return MCP-spec content envelopes; auth via bearer token; tenant isolation via token claims; `readOnlyHint` on GET-equivalent tools; `destructiveHint` on delete-equivalent tools; confirmation flows for destructive operations (implementation varies — some use out-of-band approval, some a second tool call, some rely on the client's own confirmation UI); idempotency keys for retry safety.

Ferro's differentiator versus the common baseline is the single `ServiceDef` source of truth. The visual renderer, conversational text renderer, and MCP tool renderer all consume the same definition. Changes to the projection propagate to all three surfaces simultaneously. This is not achievable in a hand-written tool approach.

---

## Sources

- MCP specification tool annotations (`readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`): [Tool Annotations as Risk Vocabulary](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/) — HIGH confidence
- MCP elicitation (human-in-the-loop, June 2025 draft): [How Elicitation in MCP Brings Human-in-the-Loop to AI Tools](https://thenewstack.io/how-elicitation-in-mcp-brings-human-in-the-loop-to-ai-tools/) — MEDIUM confidence (spec still in draft)
- Laravel MCP (consumer tool exposure, September 2025): [Laravel MCP blog](https://laravel.com/blog/introducing-laravel-mcp-build-with-the-universal-ai-standard) — HIGH confidence
- MCP security confirmation requirements: [Securing MCP: A Control Plane for Agent Tool Execution](https://developer.microsoft.com/blog/securing-mcp-a-control-plane-for-agent-tool-execution) — MEDIUM confidence
- `ferro-mcp-server/src/dispatch.rs`, `renderer.rs`, `jsonrpc.rs` — direct code inspection, HIGH confidence
- `ferro-ai/src/confirmation/mod.rs`, `store.rs`, `events.rs` — direct code inspection, HIGH confidence
- `ferro-projections` `ActionDef`, `GuardDef`, `ServiceDef`, `BaseContext.evaluated_guards` (v14.0) — direct code inspection, HIGH confidence
- v12.6 consumer-MCP design spec: `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md` — direct read, HIGH confidence

---

*Feature research for: v15.0 Agent-Operable App (Consumer MCP)*
*Researched: 2026-06-13*
