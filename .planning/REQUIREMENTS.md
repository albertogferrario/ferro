# Requirements: v15.0 Agent-Operable App (Consumer MCP)

**Milestone goal:** A tenant operates a live ferro app through a per-tenant MCP endpoint whose tools are derived from the app's projections — reading and acting on real data through an agent rather than the dashboard. Extends the projection/intent abstraction to a `ServiceDef → MCP tools` renderer target and adds the inbound message → action loop. Validated against gestiscilo (synthetic fixtures in-framework; consumer migration is a separate follow-up).

**Grounding:** `.planning/research/SUMMARY.md` (+ STACK/FEATURES/ARCHITECTURE/PITFALLS). The read path already exists (Phase 197: `tools/list` + `tools/call` + fail-closed tenant-scoped dispatch); v15.0 is the write path + guard enforcement + per-tenant API-key auth + inbound NL loop. All work lands in `ferro-mcp-server` (no new crate, no rmcp upgrade); the only new dependency is a feature-flagged `ferro-ai`.

---

## v15 Requirements

### Tenant Context & Auth

- [x] **AMCP-01**: The MCP endpoint resolves the calling tenant and that tenant's evaluated guards into the render/call context, so every tool listing and tool call is tenant- and permission-scoped. (`McpContext` embeds `BaseContext` — `tenant_id` + `evaluated_guards`; today it is an empty struct and is the universal prerequisite for every other requirement.)
- [x] **AMCP-02**: A tenant authenticates to the MCP endpoint with a per-tenant API key (alongside the existing OAuth path), and the resolved principal scopes both the visible tool set and all data access to that tenant.

### Write Tools

- [x] **AMCP-03**: Each `ServiceDef`'s guarded actions are projected into MCP write tools (input schema derived from `ActionDef` inputs), exposed in `tools/list` only when the tenant's guards for that action pass, and annotated for the agent (read-only vs destructive). Tool definitions are derived purely from `ServiceDef` — no hand-authored per-tool surface.
- [x] **AMCP-04**: An agent can create, update, or state-transition a record by invoking a write tool; execution is tenant-scoped and **re-evaluates the action's guard server-side at call time** (the agent is never trusted), is idempotent against retries, and returns a spec-compliant typed result.

### Safety

- [x] **AMCP-05**: A destructive or irreversible action requires an explicit confirmation step before it executes — a two-tool confirm flow backed by the `ferro-ai` confirmation store with a TTL; an unconfirmed, mismatched, or expired attempt does not mutate data.

### Conversational Loop

- [ ] **AMCP-06**: A natural-language message is classified to a tool + arguments (`ferro-ai`), guard-checked and confirmation-gated, dispatched, and the result rendered back — the conversational turn. The loop ships with a gated replay/smoke path (env-flag, e.g. `FERRO_AI_LIVE_EVAL=1`) so it is CI-testable without live-LLM spend.

---

## Future Requirements (deferred)

- **DB-backed confirmation store** — `InMemoryConfirmationStore` loses pending confirmations on process restart; a persistent store is a production hardening item, acceptable to defer for the v15.0 walking skeleton.
- **Remaining channel renderers** — voice, structured-API, and mobile (`device_class` / chart-card) renderers remain a separate channel milestone.
- **gestiscilo full adoption** — migrating gestiscilo's own views/services to drive the endpoint is a consumer-repo follow-up; v15.0 delivers the framework capability + synthetic validation only.
- **MCP elicitation for missing parameters** — interactive parameter elicitation in the NL loop depends on MCP client support (June 2025 draft); the loop may require complete arguments up front for v15.0.

## Out of Scope

| Item | Reason |
|------|--------|
| New MCP output crate | `ferro-mcp-server` already hosts `McpRenderer`; a second crate would duplicate the control surface (v11.5 boundary rule already satisfied) |
| rmcp upgrade (≥1.5) | rmcp 0.12 supports runtime tool registration + typed `CallToolResult::structured`; upgrading is a breaking change across 3 crates and no v15.0 feature requires it |
| Editing gestiscilo's repo from ferro | Cross-repo validation splits along repo boundaries; ferro phases deliver capability + synthetic fixtures, never edit the consumer tree |
| Routing write dispatch through the app's HTTP stack (if a direct callback suffices) | Avoids re-implementing auth for the app's own routes; resolved in the write-dispatch phase |

---

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| AMCP-01 | Phase 217 | Complete |
| AMCP-02 | Phase 217 | Complete |
| AMCP-03 | Phase 218 | Complete |
| AMCP-04 | Phase 219 | Complete |
| AMCP-05 | Phase 220 | Complete |
| AMCP-06 | Phase 221 | Pending |

*Phase assignments filled by the roadmapper. Roadmap created 2026-06-13.*
