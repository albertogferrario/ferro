---
gsd_state_version: 1.0
milestone: v11.0
milestone_name: Framework Consolidation Audit
status: verifying
stopped_at: Phase 237 context gathered
last_updated: "2026-06-21T22:03:23.519Z"
last_activity: 2026-06-21
progress:
  total_phases: 11
  completed_phases: 10
  total_plans: 45
  completed_plans: 47
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md and .planning/VISION.md

**Current focus:** Phase 238 — inertia-first-load-html-shell

## Current Position

Milestone: v16.0 Write-Boundary AX — StateMachine-Derived Executor. Goal: derive a default write executor from the `ServiceDef` StateMachine (kill the "declare twice" `WriteDispatcher` duplication) with an override hook for the app-specific 20%. Last load-bearing gap in the projection/intent killer feature's write path. Verified 2026-06-16: `ferro-projections` has no executor-derivation machinery at 0.2.65.

Phase: 238
Plan: Not started
Next: `/gsd-plan-phase 231`. Phase 231 = derivation + guard re-eval + override hook + sync-by-construction in `ferro-projections` (EXEC-01/02/03/04). Phase 232 = wire the derived executor across the MCP + visual/form write surfaces, retire the hand-written `WriteDispatcher` (EXEC-05). All 5 v16 requirements mapped, no orphans.
Prior: v15.0 ✅ Agent-Operable App / Consumer MCP (217–221, shipped 2026-06-14, published 0.2.66). v14.0 ✅ Channel Projection (215–216, `ferro-text::TextRenderer`); v13.x ✅ (207–214). Phase 213 closed the projection render-content gaps (kanban/StatCard/actions data-bound).

Status: Phase complete — ready for verification

Last activity: 2026-06-21
Workspace version: 0.2.66 (published; local tree 0.2.65 + remote 0.2.66 version bump)

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
| 225 | 3 | - | - |
| 226 | 4 | - | - |
| 227 | 3 | - | - |
| 228 | 1 | - | - |
| 229 | 5 | - | - |
| 230 | 7 | - | - |
| 233 | 3 | - | - |
| 234 | 3 | - | - |
| 235 | 5 | - | - |
| 236 | 7 | - | - |
| 238 | 4 | - | - |

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
| Phase 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t P01 | 31540193 | 3 tasks | 18 files |
| Phase 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t P02 | 3 | 1 tasks | 1 files |
| Phase 225 P03 | 98 | 2 tasks | 1 files |
| Phase 226-homebrew-tap-distribution-for-ferro-cli P01 | 2 | 2 tasks | 3 files |
| Phase 226-homebrew-tap-distribution-for-ferro-cli P03 | 180 | 2 tasks | 2 files |
| Phase 226-homebrew-tap-distribution-for-ferro-cli P02 | 99 | 2 tasks | 3 files |
| Phase 227-documentation-audit-and-update-for-v0-2-61 P01 | 5 | 2 tasks | 1 files |
| Phase 227-documentation-audit-and-update-for-v0-2-61 P02 | 2 | 4 tasks | 4 files |
| Phase 227-documentation-audit-and-update-for-v0-2-61 P03 | 420 | 2 tasks | 1 files |
| Phase 228-readme-and-scaffold-doc-sweep P01 | 45 | 5 tasks | 5 files |
| Phase 229-framework-benchmark-harness-foundation-1a-build-the-reproduc P01 | 72 | 2 tasks | 4 files |
| Phase 229-framework-benchmark-harness-foundation-1a-build-the-reproduc P02 | 10 | 3 tasks | 6 files |
| Phase 229-framework-benchmark-harness-foundation-1a-build-the-reproduc P03 | 21 | 2 tasks | 2 files |
| Phase 229-framework-benchmark-harness-foundation-1a-build-the-reproduc P04 | 120 | 2 tasks | 34 files |
| Phase 229 P05 | 25 | 2 tasks | 18 files |
| Phase 230 P01 | 6m | 3 tasks | 14 files |
| Phase 230 P02 | 6 min | 2 tasks | 9 files |
| Phase 230 P03 | 40 min | 3 tasks | 13 files |
| Phase 230 P04 | 40 min | 2 tasks tasks | 4 files files |
| Phase 230 P05 | 25m | 1 tasks | 3 files |
| Phase 230 P06 | 35m | 2 tasks | 5 files |
| Phase 230 P07 | 75m | 3 tasks | 18 files |
| Phase 231 P01 | 12m | 2 tasks | 5 files |
| Phase 231 P02 | 28m | 3 tasks | 9 files |
| Phase 232 P01 | 16m | 3 tasks | 12 files |
| Phase 232 P02 | 25m | 2 tasks | 5 files |
| Phase 232 P03 | 30m | 2 tasks | 2 files |
| Phase 233 P01 | 2 | 2 tasks | 8 files |
| Phase 233 P02 | 4 | 2 tasks | 6 files |
| Phase 233 P03 | 8 | 1 tasks | 4 files |
| Phase 234-ferro-payments-billable-trait-loader-and-payment-service-cor P01 | 7 | 3 tasks | 3 files |
| Phase 234-ferro-payments-billable-trait-loader-and-payment-service-cor P02 | 8 | 3 tasks | 4 files |
| Phase 234-ferro-payments-billable-trait-loader-and-payment-service-cor P03 | 1399 | 3 tasks | 2 files |
| Phase 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto P01 | 104 | 1 tasks | 1 files |
| Phase 235 P02 | 150 | 1 tasks | 1 files |
| Phase 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto- P03 | 4 | 1 tasks | 1 files |
| Phase 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto- P04 | 227 | 2 tasks | 1 files |
| Phase 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto- P05 | 559 | 2 tasks | 5 files |
| Phase 236-ferro-payments-reapers-and-publish-0-1-0 P01 | 5 | 1 tasks | 1 files |
| Phase 236 P02 | 384 | 2 tasks | 3 files |
| Phase 236 P03 | 6 | 2 tasks | 1 files |
| Phase 236 P04 | 12 | 2 tasks | 3 files |
| Phase 236 P05 | 5 | 1 tasks | 5 files |
| Phase 236 P06 | 4 | 1 tasks | 3 files |
| Phase 238-inertia-first-load-html-shell P01 | 127 | 2 tasks | 1 files |
| Phase 238 P02 | 250 | 2 tasks | 1 files |
| Phase 238 P03 | 262 | 2 tasks | 4 files |
| Phase 238 P04 | 420 | 2 tasks | 1 files |

## Accumulated Context

### Key Decisions

See PROJECT.md Key Decisions table for full history.

Recent decisions affecting current work:

- [v16.0 Phase 232 Plan 03] Single-source PROVEN (EXEC-05 / Phase 232 complete): `single_source_both_channels` drives ONE declared `submit` transition through BOTH the MCP framing (`handle_tools_call` → `dispatch_write(.., "mcp")`) and the visual handler (`dispatch_write(.., "web")`) and asserts the IDENTICAL persisted derived `to_state`, with the audit channel (`mcp.action.submit` vs `web.action.submit`) the ONLY divergence; `single_source_guard_rejects_both` proves the guard re-eval is the same kernel gate on both channels. SC4 structural grep confirms exactly one `dispatch_write` definition (`framework/src/write/mod.rs`) and no `match action_name`/transition-target match on the write path (the `match find_action`/`match action.execute` hits are action resolution / `Result` matching, not transition re-encoding). The WriteDispatcher envelope is intact (relocated, not deleted). Full workspace gate green (fmt + clippy `--all --all-targets -D warnings` + test `--all-features`, exit 0). v16.0 write-boundary milestone closed.
- [v16.0 Phase 232 Plan 01] The transition-execution kernel now lives in exactly ONE location: `framework::write` (relocated from `ferro-mcp-server`, behavior identical — guard re-eval, idempotency, confirm seam, persist, audit, override; envelope `WriteDispatcher`/`ExecutorFn`/`GuardEvaluatorFn`/`OverrideFn` preserved). The kernel owns a self-contained `WriteError`/`WriteResult` (facade-re-exported as `ferro::write::WriteError`); each channel maps it via `From` at the framing boundary. The audit prefix is parameterized: `format!("{channel}.action.{name}")` — MCP framing passes the literal `"mcp"` at every call site so `mcp.action.{name}` stays regression-pinned. `ferro-mcp-server` is now framing that depends on `ferro-rs` (acyclic) and calls into the kernel. The app `make_write_dispatcher()` closure constructs `ferro::write::WriteError`. framework `confirmation` is a pure feature flag (no ferro-ai). This is the EXEC-05 foundation; Wave 2 (Plans 02/03) builds the visual write surface on it.
- [v16.0 Phase 231] EXEC-02/03 wired into the consumer write path (Plan 02): `dispatch_write` re-evaluates `preconditions ∪ transition-guard` deduped-by-name through the single live `GuardEvaluatorFn` loop (never `ctx.evaluated_guards`); `WriteDispatcher` carries a post-persist `OverrideFn` registry (`new()`/`with_override()`) that cannot suppress the base guard/transition. EXEC-01 end-to-end: the app executor derives `to_state` from `ferro::derive_transition_plan(...).to_state` (facade only) and the hand-written `match action_name => new_status` is deleted across `app/src`. EXEC-05 (cross-surface wiring, retire the WriteDispatcher) remains Phase 232.
- [v16.0 Phase 231] EXEC-01/04 derivation core shipped in `ferro-projections` (Plan 01): `TransitionPlan` + pure `derive_transition_plan()` source `to_state` only from `Transition.to` (fan-out is a hard `AmbiguousTransition` error, never a silent first-pick); `ServiceDef::validate()` round-trips against the derivation so executor/StateMachine drift is impossible by construction. Schema-only — no sea-orm/tokio/closures. Re-exported via the `ferro::` facade for Phase 232.
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

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260614-nd3 | make replay_deterministic test assert single-execution via idempotency key and exec_count | 2026-06-14 | f14a4421 | [260614-nd3-make-replay-deterministic-test-assert-si](./quick/260614-nd3-make-replay-deterministic-test-assert-si/) |

### Roadmap Evolution

- Phase 238 added (2026-06-21): v16.2 `ferro-inertia` first-load HTML shell. Promotes [backlog/2026-06-21-inertia-first-load-shell.md](backlog/2026-06-21-inertia-first-load-shell.md) — `ferro-inertia` owns the `X-Inertia` JSON contract but has no server-rendered first-load HTML document (embedded `data-page` + Vite asset tags). Content-negotiated render: full HTML when not `X-Inertia`, JSON when it is; dev (Vite module tags off `vite_dev_server`) / prod (hashed tags from Vite `manifest.json`) asset modes; configurable root template; same-origin `server.proxy` docs. Field-reported by downstream app `u` (Phase 5 OQ-4 deferral). **Reconcile against `ferro-assets` SSR-manifest substrate before planning — likely wiring, not from-scratch.** Numbered 238 to avoid collision with the pre-existing Phase 237 (ActionGroup, roadmap-only reservation that `phase add` could not see — no directory). Next: `/gsd-plan-phase 238`.
- Phases 233-236 added (2026-06-17): v16.1 ferro-payments milestone [CONSUMER-PAIRED with gestiscilo 218-223]. New workspace crate `ferro-payments` shipping a polymorphic `PaymentIntent` entity + `Billable` trait, decomposed into four phases: **233** crate scaffold + PaymentIntent entity + portable migration `m20260617_create_payment_intents` (partial unique on (billable_kind, billable_id) WHERE status IN ('reserved','paid')), **234** Billable trait + BillableLoader trait + PaymentService core (start_checkout + request_refund, mocked-Stripe unit tests), **235** webhook SyncDispatcher integration (wire_dispatcher helper + three typed handlers + auto-refund fallback for loader-None/already-released races), **236** ReleaseExpiredPaymentIntents + ReconcileRefundsInFlight reapers + workspace test bin + publish `ferro-payments 0.1.0`. Reuses existing ferro-stripe SyncDispatcher + ProcessedEventLog + CheckoutBuilder + Connect destination-charge support — no new ferro-stripe surface. First consumer: gestiscilo Phases 218-223 (tenant booking upfront payment), blocked on Phase 236 publication. Spec: `docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md`. Next: `/gsd-plan-phase 233`.
- v15.0 roadmap created 2026-06-13: Phases 217-221. Phase numbering continues from v14.0 (last phase 216). AMCP-01..06 mapped: 217 (AMCP-01, AMCP-02), 218 (AMCP-03), 219 (AMCP-04), 220 (AMCP-05), 221 (AMCP-06). All 6 requirements covered.

## Session Continuity

Last session: 2026-06-21T22:03:23.506Z
Stopped at: Phase 237 context gathered
Resume file: .planning/phases/237-actiongroup-component-dropdownmenu-replacement/237-CONTEXT.md
Next action: `/gsd-plan-phase 217` — Tenant Context + Per-Tenant API-Key Auth
