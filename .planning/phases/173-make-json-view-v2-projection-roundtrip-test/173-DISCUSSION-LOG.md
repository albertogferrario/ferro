# Phase 173: make:json-view v2 + projection-roundtrip test - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-09
**Phase:** 173-make-json-view-v2-projection-roundtrip-test
**Mode:** `--auto` (recommended defaults auto-selected, grounded in codebase scout)
**Areas discussed:** Rendering path, NL-path unification, component_schema tension, roundtrip-test determinism, v1-absence

---

## Rendering path

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse `Spec::from_service_def` (builder.rs:54) | The deterministic ServiceDef→Spec renderer already exists; wire make:json-view to it | ✓ |
| Build a new Renderer in make:json-view | Duplicates shipped machinery; violates reuse | |
| Keep LLM picking components | Contradicts SC3 (Intent/FieldMeaning-driven selection) | |

**Selected:** reuse existing renderer (D-01).
**Notes:** Scout found `Spec::from_service_def` already implements "first concrete Renderer over a ServiceDef."

## NL-path unification

| Option | Description | Selected |
|--------|-------------|----------|
| NL → ServiceDef (reuse ai:make) → Spec::from_service_def | Single intermediary; embodies "AI as projection consumer"; deletes old direct two-pass | ✓ |
| Keep direct NL→spec two-pass alongside | Parallel path; contradicts feature-branch delete-old-code convention | |

**Selected:** unify through ServiceDef (D-03).
**Notes:** The superseded `generate_with_ai` direct-to-spec path is removed, not kept.

## component_schema vs deterministic selection

| Option | Description | Selected |
|--------|-------------|----------|
| Deterministic builder selects; json_schema validates; component_schema only if residual LLM prop-fill | Honors SC3; doesn't invent an LLM pass just to use component_schema (SC1) | ✓ |
| Add an LLM component-fill pass to exercise component_schema | Adds nondeterminism for no functional gain | |

**Selected:** deterministic-first, SC1 satisfied vacuously if no per-component LLM call (D-05).
**Notes:** Flagged as Claude's-discretion for the planner to resolve against actual builder capabilities; documented in VERIFICATION rather than forcing an LLM pass.

## Roundtrip-test determinism

| Option | Description | Selected |
|--------|-------------|----------|
| Offline fixture/mock-LlmClient ServiceDef → Spec::from_service_def → json_schema validate; assert path | Deterministic, key-free CI; mirrors projection_schema.rs; pins the ServiceDef-aware path (SC5) | ✓ |
| Live LLM call in the test | Nondeterministic; needs a provider key in CI | |

**Selected:** offline deterministic roundtrip (D-06); live NL quality is a separate manual gate (D-07).
**Notes:** Mirrors Phase 171's human-verify-live-quality precedent.

## v1-absence

| Option | Description | Selected |
|--------|-------------|----------|
| Assert no v1 JsonUiView types in pipeline/output | SC4 verification; expected already true | ✓ |

**Selected:** verification-only (D-02).

## Claude's Discretion

- `make:json-view` flag spelling for rendering an existing project ServiceDef (D-04).
- Whether any residual LLM refinement pass survives (default: no) (D-05).
- Fixture vs mock-LlmClient for the roundtrip test (D-06).

## Deferred Ideas

- Non-visual ServiceDef renderers (conversational/voice/API) — v14.0 Channel Projection.
- Live-LLM roundtrip in CI — kept as a manual gate; CI-secrets decision.
