# Phase 252: Design module + lint + CLI - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-03
**Phase:** 252-design-module-lint-cli
**Mode:** `--auto` (recommended defaults selected without interactive questioning)
**Areas discussed:** Intent plumbing & wire shape, Rule engine architecture, Lint semantics (inference/allow), CLI behavior, Rule specifics & Phase-251 handoffs, Tests & sample-app gate

---

## Intent plumbing & wire shape

| Option | Description | Selected |
|--------|-------------|----------|
| String-typed wire field + drift test | `DesignMeta { intent: Option<String>, allow: Vec<String> }`; seven labels canonical in `design/`, drift-tested against `ferro_projections::Intent::label()` | ✓ |
| Gate `design` behind `projections` feature | Use `ferro_projections::Intent` directly; module unavailable in default builds | |
| Make ferro-projections a required dependency | Heaviest option; reverses a deliberate optional-dep decision | |

**Choice rationale:** `ferro-projections` is a non-default optional dep of ferro-json-ui and ferro-cli consumes default features — the field and module must be feature-independent. A string intent makes "invalid values are findings, never errors" structural. The drift test keeps "archetypes ARE the projection intents" guarded (CI runs `--all-features`).

---

## Rule engine architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Static registry with public metadata | `DesignRule { id, title, rationale, intents, check }` + `design::rules()` iterator; Phase 253 derives docs/MCP guidance from it | ✓ |
| Trait objects per rule | `Box<dyn Rule>` collection; more indirection, same capability | |
| Hardcoded match in `lint()` | Simplest, but metadata not introspectable for Phase 253 | |

**Choice rationale:** Phase 253's pattern-catalog docs and MCP design guidance must come from the same source as the checks — single source of truth, matching Phase 251's drift-guard philosophy. `Finding`/`Severity` derive `Serialize` so CLI `--json` and the future MCP tool share one serialization.

---

## Lint semantics (inference / allow)

| Option | Description | Selected |
|--------|-------------|----------|
| Inference = info finding with stable rule id | `declare-intent` id, allowable like any rule; unknown intent → warning + fall back to inference; unknown `allow` id → warning | ✓ |
| Inference as unallowable meta-notice | Info always emitted; `allow` cannot suppress it | |
| Unknown `allow` ids as info | Softer, but a typo'd escape hatch would pass `--deny` CI silently | |

**Choice rationale:** every finding, including the inference nudge, gets a rule id so the `allow` escape hatch is uniform. Unknown `allow` ids must be warnings — under `--deny` a typo that silently disables nothing should fail CI, not pass it.

---

## CLI behavior

| Option | Description | Selected |
|--------|-------------|----------|
| `design:lint` colon-namespaced command | Follows `db:migrate` / `json-ui:schema` clap pattern; `commands/design_lint.rs`; default path `src/views`; skip non-spec JSON ($schema marker); `--deny` fails on warnings only | ✓ |
| `lint` top-level command | Shorter but breaks the established namespace convention | |
| Fail `--deny` on info too | Stricter, but contradicts the spec's "info for inferences" intent | |

**Choice rationale:** anchor spec §4 fixes the surface; only discovery edge-semantics needed a call. JSON files without `"$schema": "ferro-json-ui/v2"` are skipped silently; marker-bearing files that fail parse are warning-level file diagnostics (visible, and gating under `--deny`).

---

## Rule specifics & Phase-251 handoffs

| Option | Description | Selected |
|--------|-------------|----------|
| Fold stale-prop rule; dashboard-family = {dashboard, app} | Add migration-hygiene rule (all intents) sourced from the Phase 251 migration table; research WR-01 lint first for single-home placement; `auth` layout exempt from page-header | ✓ |
| Anchor-spec 10 rules only | Defer all Phase 251 handoffs to the friction loop | |
| Fold all three handoffs (incl. dot_colors + CSS hygiene) | Max absorption; CSS hygiene is a different artifact class and risks scope creep | |

**Choice rationale:** Phase 251 named this lint the "natural home" for stale-prop diagnostics three times (251-01/251-03 SUMMARYs, 251-VERIFICATION finding 1) and gestiscilo's migration directly benefits; "~10 rules" is approximate by spec wording. The WR-01 research directive prevents a duplicate control surface. `dot_colors` stays discretionary; CSS-hygiene deferred.

---

## Tests & sample-app gate

| Option | Description | Selected |
|--------|-------------|----------|
| App views declare intent; gate asserts zero findings | Dogfoods `design.intent` on the 3 sample specs; app-crate test walks `app/src/views/*.json` | ✓ |
| Gate tolerates info findings | Views stay undeclared; inference noise accepted | |

**Choice rationale:** success criterion 1 wants the field exercised; declaring intent on the sample views makes the lint-clean gate strict (zero findings) and gives Phase 253 a working example to document.

---

## Claude's Discretion

- Rule engine internals (fn pointers vs traits, file layout, traversal helpers)
- Inference heuristic details (tie-breaking, cluster thresholds, no-match fallback)
- Human-readable output formatting; exact `--json` envelope (documented + stable)
- OQ-3 `dot_colors` bonus rule if cheap

## Deferred Ideas

- Phase 253: MCP tool, catalog/generation-context extensions, docs chapter, publish
- CSS-hygiene lint (generated-CSS artifact class, not spec lint)
- gestiscilo Phase 232 consumer adoption (separate repo, gated on 253 publish)
