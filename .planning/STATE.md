---
gsd_state_version: 1.0
milestone: v15.0
milestone_name: Agent-Operable App (Consumer MCP)
status: verifying
stopped_at: "Completed 221-03-PLAN.md — POST /mcp/chat endpoint + live-eval gate (SC#4). Phase 221 complete. v15.0 milestone complete."
last_updated: "2026-06-14T02:49:51.402Z"
last_activity: 2026-06-14
progress:
  total_phases: 102
  completed_phases: 94
  total_plans: 384
  completed_plans: 383
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md and .planning/VISION.md

**Current focus:** Phase 221 — inbound-nl-intent-loop

## Current Position

Milestone: v15.0 Agent-Operable App (Consumer MCP) — ACTIVE (roadmap created 2026-06-13). Scope: extend the projection/intent abstraction to a write-and-act MCP surface. Per-tenant API-key auth (Phase 217), `ActionDef`-derived write tools (Phase 218), server-side guard-enforced write dispatch (Phase 219), `ferro-ai` confirmation gating for destructive actions (Phase 220), inbound NL intent loop with replay/smoke CI path (Phase 221). All work in `ferro-mcp-server`. Validated via synthetic fixtures.

Phase: 221
Plan: Not started
Next: `/gsd-plan-phase 217`
Prior: v14.0 ✅ Channel Projection (215–216, `ferro-text::TextRenderer`); v13.x ✅ (207–214). Foundation: v12.6 consumer-MCP OAuth endpoint + `McpRenderer` read tools; v14.0 `BaseContext.evaluated_guards`; `ferro-ai`; v13.1 `TenantScoped` isolation.

Status: Phase complete — ready for verification

Progress: v15.0 — 0/5 phases. v14.0 ✅ shipped (215–216, 0.2.58).

Last activity: 2026-06-14
Workspace version: 0.2.58

> **Operator actions pending (from v14.0 / prior milestones):**
> - 0.2.56 (v13.1 CRUD proc macros + v13.3 scaffold parity) bumped locally, not yet published — push to trigger auto-publish.
> - 4 merged local branches safe to prune (backup/v12.0-…, feat/176-…, feat/180-…, v12.0/json-ui-v2).
> - Open consumer-side phase in gestiscilo-it to adopt STORAGE_* rename.

## Shipped Milestone: v14.0 Channel Projection — Non-Visual Rendering (Phases 215-216)

Shipped 2026-06-13. First production non-visual `Renderer` (`ferro-text::TextRenderer`) projecting the same `ServiceDef` as the visual/MCP renderers. Phase 215 extended the renderer-free `ferro-projections` surface (`BaseContext.evaluated_guards` + `verbosity`, `Intent::label()`, `Error::NoIntents`); Phase 216 added `FieldDef.render_hint` and the `ferro-text` output crate with per-intent text strategies, guard-filtered, verbosity-aware, `insta` snapshot-tested against the COMP-05 `approval_workflow` anchor.

| Phase | Status | Verification |
|-------|--------|--------------|
| 215. Non-Visual Rendering Context | ✅ shipped | 5/5 verified |
| 216. Conversational Text Renderer | ✅ shipped | verified — code review 0 critical |

Progress: [██████████] 100%

## Shipped Milestone: v12.7 Passwordless MCP Auth (Phases 202-203)

Shipped 2026-06-12. Passwordless and cross-device auth for the consumer-app MCP surface.

| Phase | Status | Verification |
|-------|--------|--------------|
| 202. Login-resume contract + magic-link sample app | ✅ shipped | passed 5/5 |
| 203. OAuth Device Authorization Grant (RFC 8628) | ✅ shipped | passed 5/5 (13-test SC-5 matrix) |

Progress: [██████████] 100%

## Shipped Milestone: v12.6 Consumer App MCP (Browser Login) (Phases 197-200)

Shipped 2026-06-11. Dogfood acceptance GO. Published to crates.io (ferro-mcp-server + ferro-mcp-oauth at 0.2.51).

| Phase | Status | Verification |
|-------|--------|--------------|
| 197. McpRenderer & ferro-mcp-server | ✅ shipped | passed 5/5 |
| 198. Streamable HTTP Endpoint + Unauthenticated Challenge | ✅ shipped | passed 4/4 |
| 199. OAuth Browser Login | ✅ shipped | passed 5/5 |
| 200. Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance | ✅ shipped | passed 4/4 (dogfood GO) |

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed (v14.0 close): 372
- Average duration: —
- Total execution time: —

**By Phase (v14.0 and recent):**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 215 | 2 | - | - |
| 216 | 3 | - | - |
| 214 | 2 | - | - |
| 212 | 3 | - | - |
| 217 | 4 | - | - |
| 218 | 3 | - | - |
| 219 | 3 | - | - |
| 220 | 3 | - | - |
| 221 | 3 | - | - |

*Updated after each plan completion*
| Phase 217-tenant-context-per-tenant-api-key-auth P00 | 35 | 3 tasks | 11 files |
| Phase 217-tenant-context-per-tenant-api-key-auth P01 | 8 | 2 tasks | 3 files |
| Phase 217-tenant-context-per-tenant-api-key-auth P02 | 5 | 2 tasks | 1 files |
| Phase 217-tenant-context-per-tenant-api-key-auth P03 | 15 | 3 tasks | 6 files |
| Phase 218-write-tool-rendering-from-actiondef P00 | 171 | 3 tasks | 3 files |
| Phase 218-write-tool-rendering-from-actiondef P01 | 97 | 1 tasks | 1 files |
| Phase 218-write-tool-rendering-from-actiondef P02 | 1187 | 3 tasks | 3 files |
| Phase 219-write-dispatch P00 | 15 | 3 tasks | 6 files |
| Phase 219-write-dispatch P01 | 20 | 3 tasks | 4 files |
| Phase 219-write-dispatch P02 | 120 | 3 tasks | 11 files |
| Phase 220-confirmation-gating-for-destructive-actions P00 | 609 | 3 tasks | 5 files |
| Phase 220-confirmation-gating-for-destructive-actions P01 | cross-session | 3 tasks | 4 files |
| Phase 220-confirmation-gating-for-destructive-actions P02 | 900 | 2 tasks | 4 files |
| Phase 221-inbound-nl-intent-loop P01 | 1025 | 3 tasks | 13 files |
| Phase 221 P02 | 12 | 2 tasks | 3 files |
| Phase 221-inbound-nl-intent-loop P03 | 27 | 2 tasks | 7 files |

## Accumulated Context

### Key Decisions

See PROJECT.md Key Decisions table for full history.

Recent decisions affecting current work:

- [v15.0 roadmap] `McpContext` must embed `BaseContext` (`tenant_id` + `evaluated_guards`) before any write-path work can proceed — universal prerequisite. Phase 217 is the hard gate.
- [v15.0 roadmap] Per-tenant API key carries an explicit scope field (`read` / `read_write`) at issuance — not retrofittable; must be in the key model from Phase 217.
- [v15.0 roadmap] Write tools derived purely from `ActionDef` — no hand-authored tool definitions in `McpRenderer` (per `feedback_no_duplicate_control_surface`).
- [v15.0 roadmap] Guards re-evaluated server-side at `tools/call` execution time — not advisory from `tools/list` listing. Violation = privilege escalation.
- [v15.0 roadmap] `ferro-ai` added to `ferro-mcp-server` behind a feature flag — read-only consumers do not pull LLM HTTP clients.
- [v15.0 roadmap] NL loop replay/smoke path (`FERRO_AI_LIVE_EVAL=1` gate) ships in the same phase as the live path — no live-only NL loop.
- [v12.6] `ferro-mcp-server` is a new output crate (Wave 2 — depends on `ferro-projections`).
- [v12.6] `ferro-projections` stays renderer-free; `McpRenderer` lives in `ferro-mcp-server`.
- [v14.0] `BaseContext.evaluated_guards`: absent key = render, explicit `false` = filter.

### Research Flags for Phase Planning

Before planning each phase, verify these open questions:

- **Phase 217:** Does `ferro make:api-key` (v8.1) already provide a reusable `api_keys` table in the framework or app? One `grep`/`find` resolves this; determines whether Phase 217 needs a new migration.
- **Phase 219:** Write dispatch callback contract — HTTP POST to the app's own route vs. registered Rust `async fn` callback. The choice affects the registration API surface in `ferro-mcp-server`. Decide before Phase 219 planning.
- **Phase 220:** Is `destructive` an explicit `bool` field on `ActionDef` or inferred from `transition_trigger.is_some()`? Heuristic may be insufficient for delete-category actions without a state machine. Decide before Phase 220 planning; may require a `ferro-projections` API change.

### Pending Todos

- Operator: push 0.2.56 commits to trigger auto-publish (bundles v13.1 + v13.3 work).
- Ferro doctor `db_connection` and `migrations_pending` checks should auto-resolve `--bin <pkg>` for multi-bin projects without `default-run`.

### Blockers/Concerns

None active. Research flags above are pre-phase checks, not blockers.

### Roadmap Evolution

- v15.0 roadmap created 2026-06-13: Phases 217-221. Phase numbering continues from v14.0 (last phase 216). AMCP-01..06 mapped: 217 (AMCP-01, AMCP-02), 218 (AMCP-03), 219 (AMCP-04), 220 (AMCP-05), 221 (AMCP-06). All 6 requirements covered.

## Session Continuity

Last session: 2026-06-14T02:15:29.886Z
Stopped at: Completed 221-03-PLAN.md — POST /mcp/chat endpoint + live-eval gate (SC#4). Phase 221 complete. v15.0 milestone complete.
Resume file: None
Next action: `/gsd-plan-phase 217` — Tenant Context + Per-Tenant API-Key Auth
