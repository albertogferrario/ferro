---
gsd_state_version: 1.0
milestone: v12.6
milestone_name: Consumer App MCP (Browser Login)
status: executing
stopped_at: Completed 197-01-PLAN.md
last_updated: "2026-06-10T12:43:59.871Z"
last_activity: 2026-06-10
progress:
  total_phases: 82
  completed_phases: 73
  total_plans: 316
  completed_plans: 314
  percent: 99
---

# Project State

## Project Reference

See: .planning/PROJECT.md and .planning/VISION.md

**Current focus:** Phase 197 — mcprenderer-ferro-mcp-server

## Current Position

Phase: 197 (mcprenderer-ferro-mcp-server) — EXECUTING
Plan: 2 of 3
Next: `/gsd-plan-phase 197`
Status: Ready to execute

Progress: [████████████████████░░░░░░░░░░░░░░░░] 89% (73/82 phases)

Last activity: 2026-06-10
Workspace version: 0.2.49

> **Operator actions pending:**
> - `git push` — master is ahead of origin; the push triggers GH Actions auto-publish of **ferro-stripe 0.7.0** (completes v11.6.2's publish, unblocks gestiscilo Phase 99).
> - 4 fully-merged local branches safe to prune (backup/v12.0-…, feat/176-…, feat/180-…, v12.0/json-ui-v2).

## Active Milestone: v12.6 Consumer App MCP (Browser Login) (Phases 197-200)

**Design center:** The MCP surface is a rendering target for the projection / intent system — the same `ServiceDef` that renders to JSON-UI (visual) also renders to MCP tool schema and tool output (agent-consumable) via `McpRenderer`. One source of truth; no parallel hand-maintained tool contract.

**Design spec:** `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md`

**Key design constraints locked:**

- `ferro-mcp-server` is a new output crate (mirrors `ferro-json-ui`); `ferro-projections` stays renderer-free.
- Tenant scoping and authorization are structural — reused from existing multi-tenant middleware and policy layer; no second permission system.
- Walking skeleton: read-only, one opt-in projection, end to end. Write intents deferred.
- Dogfood GO/NO-GO in Phase 200 is non-negotiable: a real MCP client completing browser login is the acceptance gate.

| Phase | Status |
|-------|--------|
| 197. McpRenderer & ferro-mcp-server | Not started |
| 198. Streamable HTTP Endpoint + Unauthenticated Challenge | Not started |
| 199. OAuth Browser Login | Not started |
| 200. Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance | Not started |

Progress: [░░░░░░░░░░] 0%

## Shipped Milestone: v12.5 Projection Checkpoint (Phases 194-196)

Shipped 2026-06-10. Close the agent write→verify loop: `checkpoint_projection` MCP tool walks the intent-slice spine, owns the field→column seam (the only silent gap no existing validator covers), delegates the remaining seams to existing validators, and returns a single structured verdict with ranked next steps. Closes by default after generation; ambient status in `application_info`/`projection_coverage`. Killer feature: a dangling projection field (no backing migration column) surfaces statically in one call rather than at runtime.

Progress: [██████████] 100%

## Shipped Milestone: v12.4 Form Validation DX (Phases 190-192)

Shipped 2026-06-09. Async DB-backed `unique` rule with exclude-self (edit-form safety) + `ConstraintMap` opt-in DB constraint→field-level error mapping + ferro-mcp template and docs. Source: gestiscilo-it field test (slug-uniqueness violations surfacing as raw SQL errors). All 3 phases verified.

Progress: [██████████] 100%

## Shipped Milestone: v12.3 Deployment Platform Primitives (Phases 185-188)

Shipped 2026-06-07, sourced from gestiscilo-it v7.1 Tenant Frontend Platform. Four phases, all complete: 185 `ferro::queue` DB-backed job queue, 186 `ferro-deployments` new crate, 187 `ferro-assets` new crate, 188 `ferro-storage` CDN extension. Released to crates.io at 0.2.48.

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed: 313
- Average duration: —
- Total execution time: —

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 140 | 5 | - | - |
| 141 | 4 | - | - |
| 143 | 4 | - | - |
| 144 | 5 | - | - |
| 145 | 5 | - | - |
| 146 | 3 | - | - |
| 148 | 3 | - | - |
| 149 | 7 | - | - |
| 150 | 5 | - | - |
| 155 | 7 | - | - |
| 156 | 6 | - | - |
| 157 | 4 | - | - |
| 158 | 2 | - | - |
| 120 | 5 | - | - |
| 121 | 6 | - | - |
| 159 | 3 | - | - |
| 162 | 10 | - | - |
| 163 | 10 | - | - |
| 164 | 12 | - | - |
| 160 | 10 | - | - |
| 175 | 6 | - | - |
| 176 | 2 | - | - |
| 177 | 3 | - | - |
| 180 | 6 | - | - |
| 181 | 8 | - | - |
| 184 | 3 | - | - |
| 189 | 4 | - | - |
| 185 | 5 | - | - |
| 186 | 4 | - | - |
| 187 | 4 | - | - |
| 188 | 3 | - | - |
| 165 | 4 | - | - |
| 166 | 5 | - | - |
| 167 | 2 | - | - |
| 168 | 2 | - | - |
| 169 | 3 | - | - |
| 170 | 1 | - | - |
| 171 | 4 | - | - |
| 172 | 4 | - | - |
| 190 | 4 | - | - |
| 173 | 2 | - | - |
| 191 | 2 | - | - |
| 192 | 2 | - | - |
| 193 | 1 | - | - |
| 194 | 3 | - | - |
| 195 | 4 | - | - |
| 196 | 4 | - | - |

*Updated after each plan completion*
| Phase 108-p0-accuracy-fixes P01 | 3 | 1 tasks | 3 files |
| Phase 108-p0-accuracy-fixes P02 | 12min | 2 tasks | 3 files |
| Phase 109-cli-reference-completeness P01 | 148s | 2 tasks | 1 files |
| Phase 110-mcp-tool-accuracy P02 | 8min | 1 tasks | 1 files |
| Phase 110-mcp-tool-accuracy P01 | 15min | 2 tasks | 2 files |
| Phase 111-documentation-coverage P01 | 106s | 2 tasks | 2 files |
| Phase 111-documentation-coverage P02 | 2min | 2 tasks | 2 files |
| Phase 112-agent-first-philosophy P01 | 2min | 2 tasks | 3 files |
| Phase 112-agent-first-philosophy PP02 | 248s | 2 tasks | 19 files |
| Phase 113-pattern-coherence P02 | 12min | 2 tasks | 5 files |
| Phase 113-pattern-coherence P01 | 85 | 2 tasks | 22 files |
| Phase 114.1-template-renderer P01 | 10min | 2 tasks | 3 files |
| Phase 122-deploy-scaffold-core-rewrite P01 | 7min | 1 tasks | 2 files |
| Phase 122-deploy-scaffold-core-rewrite P02 | 6min | 2 tasks | 5 files |
| Phase 122-deploy-scaffold-core-rewrite P03 | 5min | 2 tasks | 2 files |
| Phase 122-deploy-scaffold-core-rewrite P04 | 5min | 2 tasks | 5 files |
| Phase 122-deploy-scaffold-core-rewrite P05 | ~6min | 2 tasks | 5 files |
| Phase 122 P06 | 3m | 1 tasks | 1 files |
| Phase 123-deploy-mcp-tools P02 | 8min | 2 tasks | 8 files |
| Phase 123-deploy-mcp-tools P05 | 6min | 2 tasks | 3 files |
| Phase 124-doctor-introspection-and-ci-scaffold P02 | 15min | 2 tasks | 4 files |
| Phase 124 P03 | 25min | 2 tasks | 8 files |
| Phase 124 P05 | 5min | 1 tasks | 2 files |
| Phase 122.1 P02 | 6min | 2 tasks | 2 files |
| Phase 122.1 P04 | ~8min | 2 tasks | 7 files |
| Phase 122.2 P01 | 3min | 2 tasks | 6 files |
| Phase 122.2 P03 | 12min | 3 tasks | 18 files |
| Phase 122.2 P07 | 8min | 2 tasks | 4 files |
| Phase 122.2 P08 | 14m | 2 tasks | 10 files |
| Phase 127 P01 | 25min | 3 tasks | 7 files |
| Phase 127-generated-artifact-polish P02 | 10min | 2 tasks | 4 files |
| Phase 127-generated-artifact-polish P03 | 8min | 1 tasks | 3 files |
| Phase 127-generated-artifact-polish P04 | 15min | 3 tasks | 5 files |
| Phase 128-deploy-preflight P01 | 5min | 2 tasks | 4 files |
| Phase 128-deploy-preflight P03 | 2min | 2 tasks | 3 files |
| Phase 128-deploy-preflight P02 | 5min | 3 tasks | 8 files |
| Phase 128-deploy-preflight P04 | 4min | 2 tasks | 4 files |
| Phase 129-publish-workflow-refinement P01 | 2 | 2 tasks | 1 files |
| Phase 129-publish-workflow-refinement P02 | 2min | 3 tasks | 2 files |
| Phase 129 P03 | 2min | 2 tasks | 1 files |
| Phase 131 P01 | 20min | 2 tasks | 7 files |
| Phase 131-scaffolder-multibin-copydirs-runtime-apt P02 | 9min | 2 tasks | 11 files |
| Phase 131 P03 | 8min | 1 tasks | 6 files |
| Phase 132 P01 | 11min | 2 tasks | 4 files |
| Phase 133-generalize-renderer-trait P01 | 3.5min | 1 tasks | 5 files |
| Phase 133-generalize-renderer-trait P02 | 5min | 1 tasks | 4 files |
| Phase 134-relocate-renderers-to-output-crates P01 | 15min | 1 tasks | 6 files |
| Phase 134-relocate-renderers-to-output-crates P02 | 4min | 2 tasks | 10 files |
| Phase 135-servicedef-derivation-bridge P01 | 8min | 2 tasks | 3 files |
| Phase 135-servicedef-derivation-bridge P02 | 6min | 2 tasks | 3 files |
| Phase 141 P02 | 15min | 2 tasks | 5 files |
| Phase 145 P01 | 11min | 3 tasks | 7 files |
| Phase 145 P02a | 8min | 2 tasks | 3 files |
| Phase 145 P02b | 21min | 2 tasks | 1 files |
| Phase 148 P01 | 221s | 2 tasks | 2 files |
| Phase 149 P01 | 9min | 3 tasks | 6 files |
| Phase 149 P02 | 4m 12s | 3 tasks | 4 files |
| Phase 149 P03 | 5m 1s | 2 tasks | 3 files |
| Phase 149 P04 | 4m 41s | 3 tasks | 2 files |
| Phase 149 P05 | 4m 7s | 2 tasks | 1 files |
| Phase 149 P06 | 9m 17s | 3 tasks | 2 files |
| Phase 149 P07 | 8m 2s | 7 tasks | 8 files |
| Phase 151 P02 | 3min | 2 tasks | 2 files |
| Phase 151 P03 | 4m 15s | 2 tasks | 2 files |
| Phase 151-ferro-wallet-crate P04 | 2m 46s | 2 tasks | 2 files |
| Phase 151-ferro-wallet-crate P05 | 4m 34s | 4 tasks | 5 files |
| Phase 151 P07 | 10min | 3 tasks | 4 files |
| Phase 151-ferro-wallet-crate P06 | 4min | 1 tasks | 1 files |
| Phase 151-ferro-wallet-crate P151-08 | 94s | 1 tasks | 1 files |
| Phase 151 P09 | 2min | 2 tasks | 3 files |
| Phase 153 P02 | 5 | 4 tasks | 4 files |
| Phase 153 P03 | 176s | 2 tasks | 2 files |
| Phase 153 P04 | 2min | 1 tasks | 1 files |
| Phase 153 P05 | 3m3s | 4 tasks | 4 files |
| Phase 153 P06 | 448 | 5 tasks | 4 files |
| Phase 154 P01 | 5min | 5 tasks | 13 files |
| Phase 154 P02 | 5 | 4 tasks | 4 files |
| Phase 154 P03 | 170 | 3 tasks | 3 files |
| Phase 154 P04 | 18 | 4 tasks | 4 files |
| Phase 154 P05 | 35 | 1 tasks | 2 files |
| Phase 154 P06 | 11 | 4 tasks | 6 files |
| Phase 154 P07 | 10 | 4 tasks | 3 files |
| Phase 156 P06 | 286s | 2 tasks | 2 files |
| Phase 162-json-ui-improvements-batch-1-components-expressions-and-spec P01 | 595 | 3 tasks | 5 files |
| Phase 162 P02 | 138s | 1 tasks | 1 files |
| Phase 162 P03 | 15 | 2 tasks | 4 files |
| Phase 162 P04 | 25 | 3 tasks | 5 files |
| Phase 162 P05 | 7 | 2 tasks | 1 files |
| Phase 162-json-ui-improvements-batch-1-components-expressions-and-spec P08 | 10 | 3 tasks | 3 files |
| Phase 162-json-ui-improvements-batch-1-components-expressions-and-spec P09 | 2 | 3 tasks | 5 files |
| Phase 162-json-ui-improvements-batch-1-components-expressions-and-spec P10 | 7 | 4 tasks | 6 files |
| Phase 163 P03 | pre-committed | 1 tasks | 3 files |
| Phase 163 P04 | 158 | 1 tasks | 1 files |
| Phase 163 P05 | 10min | 1 tasks | 1 files |
| Phase 163 P06 | 8min | 1 tasks | 1 files |
| Phase 163 P08 | 98s | 1 tasks | 1 files |
| Phase 163 P10 | 3min | 1 tasks | 1 files |
| Phase 163.1 P01 | 183s | 5 tasks | 7 files |
| Phase 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp P01 | 42min | 3 tasks | 7 files |
| Phase 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp P02 | 3min | 1 tasks | 1 files |
| Phase 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp P03 | 4min | 1 tasks | 1 files |
| Phase 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp P04 | 75s | 1 tasks | 1 files |
| Phase 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp PP05 | 4min | 1 tasks tasks | 1 files files |
| Phase 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp P160-06 | 5min | 3 tasks | 3 files |
| Phase 160 P07 | 53s | 1 tasks | 1 files |
| Phase 160 P08 | 1m 2s | 1 tasks | 1 files |
| Phase 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp P09 | 8min | 1 tasks | 1 files |
| Phase 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp P10 | 7min | 1 tasks | 2 files |
| Phase 175 P01 | 13min | 2 tasks | 4 files |
| Phase 175 P02 | 6min | 2 tasks | 2 files |
| Phase 175 P03 | 7min | 2 tasks | 1 files |
| Phase 175 P04 | 17min | 3 tasks | 6 files |
| Phase 175 P06 | 740s | 2 tasks | 4 files |
| Phase 175 P05 | 7min | 2 tasks | 2 files |
| Phase 176 P01 | pre-executed | 5 tasks | 4 files |
| Phase 176 P02 | 6 | 2 tasks | 2 files |
| Phase 177 P01 | 8 | 2 tasks | 3 files |
| Phase 177 P02 | 6min | 2 tasks | 2 files |
| Phase 177 P03 | 5 minutes | 1 tasks | 1 files |
| Phase 180 P01 | 5 min | 2 tasks | 3 files |
| Phase 180 P02 | 8 min | 2 tasks | 2 files |
| Phase 180 P03 | 8 min | 2 tasks | 4 files |
| Phase 180 P01 | 25 | 2 tasks | 5 files |
| Phase 180 P03 | 15 min | 1 tasks | 5 files |
| Phase 180 P05 | 15 | 1 tasks | 3 files |
| Phase 180 P06 | 15 | 1 tasks | 1 files |
| Phase 181 P04 | 10 | 2 tasks | 1 files |
| Phase 181 P05 | 8 | 2 tasks | 1 files |
| Phase 181 P06 | 8m | 2 tasks | 1 files |
| Phase 181 P07 | 15min | 3 tasks | 6 files |
| Phase 181 P08 | 10 | 3 tasks | 2 files |
| Phase 184 P01 | 17min | 3 tasks | 5 files |
| Phase 184 P02 | 15min | 3 tasks | 3 files |
| Phase 184-ferro-inlinebudget-ferro-requesttelemetry P184-03 | 12min | 3 tasks | 5 files |
| Phase 189 P01 | 4min | 2 tasks | 2 files |
| Phase 189 P02 | 169s | 2 tasks | 2 files |
| Phase 189 P03 | 143s | 2 tasks | 5 files |
| Phase 189 P04 | 6 minutes | 1 tasks | 1 files |
| Phase 185 P01 | 325s | 3 tasks | 7 files |
| Phase 185 P02 | 295s | 2 tasks | 4 files |
| Phase 185 P03 | 325s | 2 tasks | 5 files |
| Phase 185 P04 | 352 | 2 tasks | 6 files |
| Phase 185 P05 | 495s | 2 tasks | 4 files |
| Phase 186 P01 | 274s | 3 tasks | 8 files |
| Phase 186 P02 | 382s | 2 tasks | 5 files |
| Phase 186 P03 | 148s | 2 tasks | 2 files |
| Phase 186 P04 | 471s | 3 tasks | 4 files |
| Phase 187 P01 | 525s | 3 tasks | 10 files |
| Phase 187 P02 | 769s | 3 tasks | 11 files |
| Phase 187 P03 | 540s | 2 tasks | 4 files |
| Phase 187 P04 | 1252s | 2 tasks | 5 files |
| Phase 188 P01 | 720 | 2 tasks | 4 files |
| Phase 188 P02 | 294 | 2 tasks | 3 files |
| Phase 188 P03 | 848 | 2 tasks | 6 files |
| Phase 165 P01 | 286s | 3 tasks | 9 files |
| Phase 165 P02 | 450s | 2 tasks | 2 files |
| Phase 165 P03 | 10 | 1 tasks | 1 files |
| Phase 165 P04 | 24min | 3 tasks | 6 files |
| Phase 166 P01 | 5min | 3 tasks | 4 files |
| Phase 166 P02 | 300 | 2 tasks | 2 files |
| Phase 166 P03 | 525s | 3 tasks | 4 files |
| Phase 166 P04 | 453 | 3 tasks | 8 files |
| Phase 166 P05 | 598 | 2 tasks | 1 files |
| Phase 169 P01 | 275s | 3 tasks | 2 files |
| Phase 169-streamtext-component P02 | 1688s | 4 tasks | 4 files |
| Phase 169 P03 | 65s | 1 tasks | 1 files |
| Phase 171 P01 | 161s | 2 tasks | 2 files |
| Phase 171-ferro-ai-make-ferro-ai-explain-cli-commands P02 | 90 | 3 tasks | 7 files |
| Phase 171 P03 | 315s | 2 tasks | 3 files |
| Phase 171 P04 | 967 | 1 tasks | 4 files |
| Phase 171 P04 | continuation | 2 tasks | 1 files |
| Phase 172 P01 | 187s | 2 tasks | 3 files |
| Phase 172 P02 | 450 | 2 tasks | 4 files |
| Phase 172 P03 | 105s | 1 tasks | 1 files |
| Phase 172 P04 | 1100s | 3 tasks | 8 files |
| Phase 190-async-rule-infrastructure-unique-rule P01 | 214s | 2 tasks | 2 files |
| Phase 190-async-rule-infrastructure-unique-rule P02 | 327s | 2 tasks | 2 files |
| Phase 190 P03 | 236s | 2 tasks | 2 files |
| Phase 190 P04 | 1255s | 2 tasks | 5 files |
| Phase 173 P01 | 618 | 3 tasks | 2 files |
| Phase 173 P02 | 663s | 4 tasks | 3 files |
| Phase 191 P01 | 176s | 3 tasks | 3 files |
| Phase 191 P02 | 565 | 3 tasks | 4 files |
| Phase 192 P01 | 430s | 2 tasks | 1 files |
| Phase 192 P02 | 426s | 2 tasks | 1 files |
| Phase 193 P01 | ~20 minutes | 2 tasks | 6 files |
| Phase 194-core-checkpoint-tool P01 | 235s | 2 tasks | 3 files |
| Phase 194-core-checkpoint-tool P02 | 420 | 2 tasks | 1 files |
| Phase 194-core-checkpoint-tool P03 | 309s | 3 tasks | 4 files |
| Phase 195 P01 | 377s | 3 tasks | 3 files |
| Phase 195 P02 | 7 | 3 tasks | 1 files |
| Phase 195 P03 | 20min | 3 tasks | 3 files |
| Phase 195-close-the-loop-by-default P04 | 25 | 3 tasks | 4 files |
| Phase 196 P01 | 5 min | 3 tasks | 2 files |
| Phase 196 P02 | 100s | 1 tasks | 1 files |
| Phase 196 P03 | 10min | 3 tasks | 2 files |
| Phase 196-dogfood-acceptance-hardening P04 | 1149s | 3 tasks | 3 files |
| Phase 197 P01 | 322s | 3 tasks | 8 files |

## Accumulated Context

### Key Decisions

See PROJECT.md Key Decisions table for full history.

Recent decisions affecting current work:

- [v12.6] `ferro-mcp-server` is a new output crate (Wave 2 — depends on `ferro-projections`); add to `.github/workflows/publish.yml`.
- [v12.6] `ferro-projections` stays renderer-free; `McpRenderer` lives in `ferro-mcp-server` (mirrors `JsonUiRenderer` in `ferro-json-ui`).
- [v12.6] Tenant scoping and policy authorization are structural — no parallel permission system; existing multi-tenant middleware and policy layer reused.
- [v12.6] Walking skeleton: read-only, one opt-in projection (`mcp_exposed` marker). Write intents, auto-exposure, and MCP App UI are deferred.
- [v12.6] OAuth transport is necessary supporting infrastructure implemented to spec-standard (MCP authorization spec: discovery metadata, DCR, PKCE S256) — keep lean; disproportionate investment belongs to the `McpRenderer`.
- [v12.6] Dogfood GO/NO-GO in Phase 200 is non-negotiable: a real MCP client completing browser login is the acceptance gate (per Phase 196 discipline).
- [v12.5] Seam cascade rule: seam 1 fail → seams 4+5 `not_checked`; seam 4 fail → seam 5 `not_checked`; seams 2 and 3 run independently.
- [v12.5] `next_steps` capped at 5 (`MAX_NEXT_STEPS`).
- [v12.5] `props_to_contract` seam demoted to `not_checked`-by-default (source `validate_contracts`, reason `unproven_against_real_inputs`).
- Research established strict ordering: P0 accuracy → CLI/MCP → completeness → philosophy → metadata
- COMPONENT_CATALOG duplication requires a design decision before implementation (Phase 113)
- ferro-stripe phantom stubs: classify as incomplete, add callout — do not implement in v11.0
- `#![warn(missing_docs)]` on framework crate only — not workspace-wide (avoids mass failures)
- [110-01] All ferro imports use explicit crate-root exports — no ferro::prelude or ferro::validation:: module paths
- [110-01] Status codes use .status(u16) pattern — StatusCode enum not re-exported from ferro crate
- [110-01] Validation rule functions imported at crate root: ferro::{Validator, required, email, min, ...}
- [112-01] introduction.md leads with "agent-first" in sentence 1 — MCP mentioned before any framework comparison or Laravel reference
- [v12.1] LlmClient single trait with Err(Error::Unsupported) for missing capabilities — preserves ergonomic dispatch without lowest-common-denominator collapse
- [160-02] MCP code_templates category deletion pattern: drop registration+comment, producer fn, and integration test in one diff — no orphaned comment, no green-test artifact

### Pending Todos

- Ferro doctor `db_connection` and `migrations_pending` checks should auto-resolve `--bin <pkg>` for multi-bin projects without `default-run`. Tracked in `.planning/phases/122.2-deploy-simplification/122.2-VERIFICATION.md`.
- (Optional) Yank `ferro-assets 0.2.47` (requires `nasm`; superseded by pure-Rust 0.2.48). Needs a yank-scoped crates.io token or the web UI.

### Blockers/Concerns

- [Research flag] Phase 113: COMPONENT_CATALOG resolution needs design decision evaluation (shared data file vs build script vs new crate) — evaluate options before scoping

### Roadmap Evolution

- Phases 197-200 added (2026-06-10): v12.6 Consumer App MCP (Browser Login) milestone. Four-phase structure: 197 (ferro-mcp-server + McpRenderer — projection→tool rendering, proven in-process), 198 (Streamable HTTP endpoint + 401 challenge), 199 (OAuth browser login — discovery, DCR, PKCE, consent, token validation), 200 (per-tenant scoping + policy + dogfood acceptance gate). Design spec: `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md`.
- Phases 194-196 added (2026-06-09): v12.5 Projection Checkpoint milestone.

## Session Continuity

Last session: 2026-06-10T12:43:59.863Z
Stopped at: Completed 197-01-PLAN.md
Resume file: None
Next action: `/gsd-plan-phase 197`
