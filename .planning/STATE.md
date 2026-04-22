---
gsd_state_version: 1.0
milestone: v11.0
milestone_name: Framework Consolidation Audit
status: executing
stopped_at: Phase 147 context gathered
last_updated: "2026-04-22T22:10:56.145Z"
last_activity: 2026-04-22
progress:
  total_phases: 147
  completed_phases: 132
  total_plans: 334
  completed_plans: 313
  percent: 94
---

# Project State

## Project Reference

See: .planning/PROJECT.md and .planning/VISION.md

**Core value:** Ferro is a Rust web framework optimized for AI-assisted authoring, with projection / intent (`ferro-projections`) as its core abstraction.
**Current focus:** Phase 146 — add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va

## Current Position

Phase: 146
Plan: Not started
Workspace version: 0.2.5
Status: Ready to execute
Last activity: 2026-04-22
Next milestone: v12.0 JSON-UI v2 (Phase 115 — Spec v2 Data Structures)

Progress: [██████████] 96%

## Performance Metrics

**Velocity:**

- Total plans completed: 30
- Average duration: —
- Total execution time: —

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 140 | 5 | - | - |
| 141 | 4 | - | - |
| 143 | 4 | - | - |
| 144 | 5 | - | - |
| 146 | 3 | - | - |
| 145 | 5 | - | - |

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

## Accumulated Context

### Key Decisions

See PROJECT.md Key Decisions table for full history.

Recent decisions affecting current work:

- Research established strict ordering: P0 accuracy → CLI/MCP → completeness → philosophy → metadata
- COMPONENT_CATALOG duplication requires a design decision before implementation (Phase 113)
- ferro-stripe phantom stubs: classify as incomplete, add callout — do not implement in v11.0
- `#![warn(missing_docs)]` on framework crate only — not workspace-wide (avoids mass failures)
- [110-01] All ferro imports use explicit crate-root exports — no ferro::prelude or ferro::validation:: module paths
- [110-01] Status codes use .status(u16) pattern — StatusCode enum not re-exported from ferro crate
- [110-01] Validation rule functions imported at crate root: ferro::{Validator, required, email, min, ...}
- [112-01] introduction.md leads with "agent-first" in sentence 1 — MCP mentioned before any framework comparison or Laravel reference
- [112-01] Working with Agents guide covers ferro-mcp only — ferro-api-mcp remains on its dedicated api-mcp.md page
- [112-01] Agent-to-CLI workflow documented within working-with-agents.md as a section, not a separate page
- [112-01] MCP config command is `ferro mcp` — not a standalone ferro-mcp binary
- [145-01] Test-fixture crates under workspace root need an empty [workspace] table in their Cargo.toml to opt out of the enclosing workspace and build standalone
- [145-01] classify_key signature declared with final crossterm types (KeyCode, KeyModifiers) at Wave 0 — no Plan-02 signature rewrite needed
- [145-02b] BackendSupervisor owns backend child in its own thread; main thread holds JoinHandle for deterministic shutdown ordering per D-29
- [145-02b] drop(reload_tx) after cloning to producers lets the supervisor's recv_timeout see Disconnected — belt-and-braces termination path in addition to the AtomicBool shutdown flag
- [145-02b] debouncer_coalesces_burst uses 500ms production window and "strictly fewer events than raw writes" invariant; plan's 50ms + "exactly one" was flaky under macOS FSEvents + parallel-test CPU contention
- [145-02b] ProcessManager::any_exited deleted entirely (D-12: backend child exits are not grounds for shutdown); also deleted spawn_with_prefix convenience wrapper since only spawn_with_prefix_env is still called

### Pending Todos

- Push workspace to origin/master to publish v0.2.0 (627 commits ahead).
- Ferro doctor `db_connection` and `migrations_pending` checks should auto-resolve `--bin <pkg>` for multi-bin projects without `default-run`. Tracked in `.planning/phases/122.2-deploy-simplification/122.2-VERIFICATION.md`.

### Blockers/Concerns

- [Research flag] Phase 113: COMPONENT_CATALOG resolution needs design decision evaluation (shared data file vs build script vs new crate) — evaluate options before scoping

### Roadmap Evolution

- Phase 147 added: DetailForm component for inline edit — ferro-json-ui
- Phase 146 added: Add KeyValueEditor component to ferro-json-ui
- Phase 122 added: Deploy scaffold core rewrite (docker_init/do_init/templates rewrite, path→git ferro dep handling, multi-bin + worker support) — driven by gestiscilo deployment work
- Phase 123 added: Deploy MCP tools (deploy_check, deploy_diff_env, runtime_requirements) — read-only deploy diagnostics surfaced via ferro-mcp
- Phase 124 added: Doctor, introspection, CI scaffold (ferro doctor, routes --json, ci.yml generation, ignore_patterns sync)
- Phase 125 added: Module scaffolder + ferro-json-ui runtime split (make:module convention, runtime IIFE refactor)
- [CLI bug] `gsd-tools phase add` assigned 115 four times in one batch — does not detect previously added phases when computing next integer; also collided with an unrelated active milestone (JSON-UI v2 already at 115-121). Manually renumbered to 122-125. File against gsd-tools.
- Phase 126 added (2026-04-08): Deploy experience feedback triage — analysis-only phase pointing the next agent at `phases/126-deploy-experience-feedback/REPORT.md` (field notes from first end-to-end gestiscilo deploy: 2 fixed bugs already shipped in 0.2.1, 9 sharp edges still present, 6 DX improvements). Agent must produce `PROPOSAL.md` classifying every item before any new ferro work is scoped.
- Phase 131 added (2026-04-09): Scaffolder multi-bin, copy_dirs, runtime_apt, DO app.yaml robustness, drift detection — promoted from `.planning/backlog/gestiscilo-scaffolder-multibin-gap.md` (gestiscilo-it Phase 75 field test gap). CLI bug recurred again (returned phase 1); manually renumbered.
- Phase 130 added (2026-04-09): Invert dep convention (simple) — retire `Cargo.docker.toml` and `cargo_docker_toml_staleness` doctor check; Docker builds use `Cargo.toml` directly; local ferro dev via uncommitted `[patch.crates-io]`. Source: `.planning/proposals/dep-override-convention.md` (simplified per user direction — no new CLI verbs, no new doctor check). CLI bug recurred: `gsd-tools phase add` returned phase 1 instead of 130; manually renumbered.
- Phase 143 inserted (2026-04-20): Tailwind static CSS pipeline (URGENT) — opened new milestone v11.7. Source: gestiscilo-it production field report — `@tailwindcss/browser@4` runtime JIT fails on Safari, renders login page as unstyled HTML. Replace with pre-built static CSS. Manually scaffolded (gsd-tools phase insert rejected because STATE.md milestone field still says v11.0 but v11.6 and earlier have shipped — STATE drift is a separate cleanup). Context: `.planning/phases/143-tailwind-static-css-pipeline/143-CONTEXT.md`.
- Phase 144 added (2026-04-21): Fix root path routing in group routes — `get!("/", ...)` inside a group does not match the trailing-slash URL. Source: gestiscilo-it field test — `/s/{slug}/` returns 404; `/s/{slug}/index.html` works. The `serve_root` handler is unreachable via the canonical URL.
- Phase 145 added (2026-04-22): ferro serve manual reload key and watch supervisor — replace external `cargo-watch` with in-process supervisor, flip auto-watch to opt-in via `--watch`, add runtime `r` key for cancel-and-restart rebuilds, unify backend recompile + types regen under one debounced loop. Source: field report — rapid file saves produce compounding stale rebuilds; thermal cost on MacBook. Spec: `docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md`.

## Session Continuity

Last session: 2026-04-22T22:10:56.107Z
Stopped at: Phase 147 context gathered
Resume file: .planning/phases/147-detailform-component-for-inline-edit-ferro-json-ui/147-CONTEXT.md
Next action: `/gsd-complete-milestone v11.7`
