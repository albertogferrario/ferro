---
gsd_state_version: 1.0
milestone: v13.1
milestone_name: CRUD Handler Proc Macros
status: verifying
stopped_at: Completed 212-crud-handler-proc-macros/212-03-PLAN.md
last_updated: "2026-06-13T06:06:22.543Z"
last_activity: 2026-06-13
progress:
  total_phases: 95
  completed_phases: 87
  total_plans: 361
  completed_plans: 360
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md and .planning/VISION.md

**Current focus:** Phase 212 — crud-handler-proc-macros

## Current Position

Milestone: v13.1 CRUD Handler Proc Macros — 🚧 ACTIVE (2026-06-13). [v13.0 Compressive Validation ✅ complete (207–211); v13.2 Render Completeness ✅ SHIPPED 0.2.55; v13.3 Scaffold↔Library Parity ✅ complete (214)]
Phase: 212
Plan: Not started
Next: /gsd-discuss-phase 212
Prior: Phase 214 ✅ COMPLETE 2026-06-13 (v13.3 — scaffold↔library parity via `ferro` facade exports + corrected templates + two-layer CI guard; 10/10 verified; `ci.yml`/`publish.yml` await a manual `workflow`-scope push); Phase 213 ✅ SHIPPED 0.2.55; Phase 211 ✅ COMP-04
Status: Phase complete — ready for verification

Progress: v13.0 ✅ 5/5 (207–211); v13.2 ✅ shipped (213); v13.3 ✅ (214); v13.1 0/1 (212 next)

Last activity: 2026-06-13
Workspace version: 0.2.55 (published); Phase 214 changes committed locally, not yet released

Phase 206 (ferro-storage provider-agnostic STORAGE_* env vars) ✅ COMPLETE 2026-06-12 — 5 phase commits on master, all 10 SCs PASS (`cargo check -p ferro-storage --features s3` 0; `cargo test -p ferro-storage --features s3 --lib config::tests` 7/7), workspace 0.2.53 → 0.2.54, AWS_* aliases deprecated with one-release cushion. Push pending bundles 0.2.53 (CDN quartet) + 0.2.54 (STORAGE_*) in one publish wave.

> **Operator actions pending:**
> - ✅ **Released 0.2.54** (2026-06-12) — pushed master via HTTPS (gh credential helper; SSH still denied); Publish workflow GREEN (Check Version → full `--all-features` Test 10m26s → Publish to crates.io 5m44s). Bundled Phase 204 (CDN quartet) + Phase 205 (MCP tools/call content-block fix, dogfood GO) + ferro-json-ui absolute-URL action fix + Phase 206 (STORAGE_* rename). A pre-push `cargo fmt --check` caught uncommitted Phase 206 fmt drift (config.rs/s3.rs) that would have failed CI — fixed in commit before push.
> - 4 fully-merged local branches safe to prune (backup/v12.0-…, feat/176-…, feat/180-…, v12.0/json-ui-v2).
> - Open consumer-side phase in gestiscilo-it/app to adopt STORAGE_* rename (mirrors gestiscilo Phase 205 shape).

## Shipped Milestone: v12.7 Passwordless MCP Auth (Phases 202-203)

Shipped 2026-06-12. Passwordless and cross-device auth for the consumer-app MCP surface. A magic-link ferro app completes the OAuth/MCP browser-login flow by resuming the in-flight authorize request (Phase 202 login-resume contract: `oauth_resume_redirect` / `take_oauth_return_to`), and `ferro-mcp-oauth` gains the OAuth 2.0 Device Authorization Grant (RFC 8628) for passwordless/cross-device/headless MCP clients (Phase 203). Both reuse the v12.6 consent + tenant-scoping surfaces and the single existing token issuer — the device-code Approved arm mints through the same `build_claims` + `mint_token` path as the auth-code arm (no second token path). Milestone audit `v12.7-MILESTONE-AUDIT.md`: tech_debt status, no blockers, cross-phase integration 8/8 wired. Deferred: WR-02/WR-03 resume edge cases.

| Phase | Status | Verification |
|-------|--------|--------------|
| 202. Login-resume contract + magic-link sample app | ✅ shipped | passed 5/5 |
| 203. OAuth Device Authorization Grant (RFC 8628) | ✅ shipped | passed 5/5 (13-test SC-5 matrix) |

Progress: [██████████] 100%

## Shipped Milestone: v12.6 Consumer App MCP (Browser Login) (Phases 197-200)

Shipped 2026-06-11. A deployed ferro application serves its own OAuth-protected MCP endpoint so a consumer agent authenticates through the browser and uses the application's projections as per-tenant tools. The MCP surface is a rendering target for the projection/intent system — the same `ServiceDef` that renders to JSON-UI also renders to MCP tool schema/output via `McpRenderer` (new `ferro-mcp-server` output crate; `ferro-projections` stays renderer-free). Tenant scoping + policy authorization are structural (existing multi-tenant middleware + policy layer reused). The non-negotiable dogfood gate **passed GO** — a real MCP client completed browser OAuth login end to end with confirmed per-tenant row isolation (`200-ACCEPTANCE.md`; NO-GO on first run from a `/authorize` redirect bug, fixed via `SessionMiddleware` mount, GO on re-run both tenant directions).

**Design spec:** `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md`

| Phase | Status | Verification |
|-------|--------|--------------|
| 197. McpRenderer & ferro-mcp-server | ✅ shipped | passed 5/5 |
| 198. Streamable HTTP Endpoint + Unauthenticated Challenge | ✅ shipped | passed 4/4 |
| 199. OAuth Browser Login | ✅ shipped | passed 5/5 |
| 200. Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance | ✅ shipped | passed 4/4 (dogfood GO) |

Progress: [██████████] 100%

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

- Total plans completed: 355
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
| 197 | 3 | - | - |
| 198 | 2 | - | - |
| 199 | 5 | - | - |
| 200 | 7 | - | - |
| 203 | 5 | - | - |
| 204 | 3 | - | - |
| 205 | 3 | - | - |
| 207 | 1 | - | - |
| 208 | 2 | - | - |
| 210 | 4 | - | - |
| 211 | 2 | - | - |
| 214 | 2 | - | - |
| 212 | 3 | - | - |

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
| Phase 197 P02 | 183s | 2 tasks | 3 files |
| Phase 197 P03 | 662s | 3 tasks | 3 files |
| Phase 198 P01 | 517s | 3 tasks | 7 files |
| Phase 198 P02 | 1502 | 2 tasks | 5 files |
| Phase 199 P01 | 494 | 3 tasks | 20 files |
| Phase 199 P02 | 420 | 2 tasks | 5 files |
| Phase 199 P03 | 311s | 3 tasks | 4 files |
| Phase 199 P05 | 672 | 2 tasks | 7 files |
| Phase 200 P01 | 4min | 1 tasks | 19 files |
| Phase 200 P02 | 268s | 2 tasks | 4 files |
| Phase 200 P03 | 144s | 2 tasks | 11 files |
| Phase 200 P04 | 481s | 3 tasks | 10 files |
| Phase 200 P05 | 380 | 1 tasks | 1 files |
| Phase 200 P06 | 8min | 1 tasks | 4 files |
| Phase 202 P01 | 15min | 3 tasks | 7 files |
| Phase 202 P02 | 8min | 2 tasks | 7 files |
| Phase 202-login-resume-contract-magic-link-sample-app P03 | 8 | 2 tasks | 2 files |
| Phase 202 P04 | 10 | 1 tasks | 2 files |
| Phase 202 P05 | 20 | 2 tasks | 6 files |
| Phase 203-oauth-device-authorization-grant-rfc-8628 P01 | 188s | 2 tasks | 2 files |
| Phase 203-oauth-device-authorization-grant-rfc-8628 P02 | 120s | 1 tasks | 1 files |
| Phase 203-oauth-device-authorization-grant-rfc-8628 P03 | 430s | 2 tasks | 2 files |
| Phase 203-oauth-device-authorization-grant-rfc-8628 P04 | 384s | 2 tasks | 1 files |
| Phase 203 P05 | 673s | 3 tasks | 3 files |
| Phase 204 P01 | 4m41s | 3 tasks | 3 files |
| Phase 204 P02 | ~8m | 2 tasks | 1 files |
| Phase 204 P03 | ~45m | 3 tasks | 5 files |
| Phase 205-fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p P01 | 3min | 2 tasks | 2 files |
| Phase 205-fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p P02 | 5min | 2 tasks | 1 files |
| Phase 207 P01 | 566 | 3 tasks | 10 files |
| Phase 209 P01 | 202s | 2 tasks | 4 files |
| Phase 213 P01 | 354 | 2 tasks | 1 files |
| Phase 213 P02 | 15 | 1 tasks | 1 files |
| Phase 213 P03 | 20 | 2 tasks | 3 files |
| Phase 213-projection-render-completeness P04 | 15 | 2 tasks | 4 files |
| Phase 213 P05 | 12 | 2 tasks | 2 files |
| Phase 210 P01 | 332 | 3 tasks | 3 files |
| Phase 210 P02 | 876 | 3 tasks | 3 files |
| Phase 210-comp-03-agent-success-rate-harness P03 | 349 | 2 tasks | 1 files |
| Phase 211 P01 | ~5 minutes | 3 tasks | 4 files |
| Phase 214-scaffold-library-parity P01 | 416 | 3 tasks | 7 files |
| Phase 214 P02 | 90m | 3 tasks | 12 files |
| Phase 212 P01 | 276s | 2 tasks | 4 files |
| Phase 212-crud-handler-proc-macros P02 | 17 | 3 tasks | 13 files |
| Phase 212-crud-handler-proc-macros P03 | 25 | 3 tasks | 5 files |

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

- Phases 202-203 added (2026-06-11): v12.7 Passwordless MCP Auth milestone (planned). 202 = login-resume contract in `ferro-mcp-oauth` + magic-link sample app exemplar (consumer pairing: gestiscilo `verify_magic_link` honors `oauth_return_to`); 203 = OAuth Device Authorization Grant (RFC 8628) for passwordless/cross-device/headless MCP clients. Source: v12.6 field finding — the OAuth browser flow assumed synchronous password login; magic-link breaks continuation + cross-device delivery.
- Phases 197-200 added (2026-06-10): v12.6 Consumer App MCP (Browser Login) milestone. Four-phase structure: 197 (ferro-mcp-server + McpRenderer — projection→tool rendering, proven in-process), 198 (Streamable HTTP endpoint + 401 challenge), 199 (OAuth browser login — discovery, DCR, PKCE, consent, token validation), 200 (per-tenant scoping + policy + dogfood acceptance gate). Design spec: `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md`.
- Phases 194-196 added (2026-06-09): v12.5 Projection Checkpoint milestone.

## Session Continuity

Last session: 2026-06-13T05:41:38.096Z
Stopped at: Completed 212-crud-handler-proc-macros/212-03-PLAN.md
Resume file: None
Next action: `/gsd-discuss-phase 212` to begin v13.1 CRUD Handler Proc Macros (the active milestone). Alternatively `/gsd-complete-milestone` to formally archive the completed v13.0/v13.2/v13.3 work first. Operator: push the local Phase 214 commits (incl. `ci.yml`/`publish.yml`, which need a manual `workflow`-scope push) before the next release.
