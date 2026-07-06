# Phase 218: Write-Tool Rendering from ActionDef - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 218-write-tool-rendering-from-actiondef
**Mode:** `--auto` (gray areas auto-selected; recommended defaults chosen and logged)
**Areas discussed:** Tool naming, Input schema derivation, Guard filtering, Annotations, List assembly, Renderer placement, SC#5 coverage

---

## Tool naming

| Option | Description | Selected |
|--------|-------------|----------|
| `action.name` verbatim; `_on_<service>` only on cross-service collision | SC#1 (no hand-authored overrides); ARCHITECTURE | ✓ |
| `<action.name>_<service.name>` always | Verbose; unnecessary when names are unique | |

**Auto-selected:** `action.name`, collision-suffixed. Names never start with `list_` → 217 scope gate stays correct.

## Input schema derivation

| Option | Description | Selected |
|--------|-------------|----------|
| `build_action_input_schema(action, service)` mirroring `build_input_schema`, reuse `data_type_to_json_schema`, inject Identifier param | SC#2; ARCHITECTURE mapping | ✓ |
| Hand-author per-action schemas | Violates AMCP-03 | |

**Auto-selected:** derived builder. Sensitive `FieldMeaning` inputs excluded (PITFALLS §3, reuse `is_filter_field` precedent).

## Guard filtering

| Option | Description | Selected |
|--------|-------------|----------|
| Omit tool if any precondition `== Some(false)` in `ctx.evaluated_guards`; absent = show | SC#3; v14.0 semantics | ✓ |
| Always show, filter at call time | Violates SC#3 (must be absent from list) | |

**Auto-selected:** list-time filter on `evaluated_guards`. Explicitly a visibility mechanism, NOT the auth gate (PITFALLS §2 — enforcement is 219). Runtime guard population flagged for researcher.

## Annotations

| Option | Description | Selected |
|--------|-------------|----------|
| `read_only(false).destructive(transition_trigger.is_some())`; no idempotentHint | SC#4, existing attribute, no new ActionDef field | ✓ |
| Add explicit `destructive`/`irreversible` ActionDef flag now | Belongs to 220 confirmation work | |

**Auto-selected:** derive destructiveHint from `transition_trigger`. `idempotentHint` deferred (no attribute; 219 concern).

## List assembly

| Option | Description | Selected |
|--------|-------------|----------|
| `render_exposed_tools` emits `list_<service>` then write tools per action | Single-source; minimal change | ✓ |
| Separate write-tool renderer entry point | Two assembly sites, drift risk | |

**Auto-selected:** extend `render_exposed_tools`; read tool first, then actions in declaration order.

## Renderer placement

| Option | Description | Selected |
|--------|-------------|----------|
| All in `ferro-mcp-server` (renderer.rs + schema.rs); no projection change | v11.5 boundary; ActionDef already in projections | ✓ |
| Add rendering to ferro-projections | Violates renderer-location rule | |

**Auto-selected:** ferro-mcp-server only.

## SC#5 coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Extend the Phase 205 strict-deser regression test to cover every write tool | SC#5 verbatim | ✓ |

**Auto-selected:** extend 205 test (researcher pins exact file/test name).

## Claude's Discretion

- `build_action_input_schema` signature; whether `data_type_to_json_schema` is promoted to `pub(crate)` or wrapped.
- Test fixture shape (service with 1 read + ≥2 actions, one guarded, one with `transition_trigger`).

## Deferred Ideas

- Write dispatch + server-side guard re-eval + idempotency + audit (219); confirmation gating + confirm_<action> (220); idempotentHint/explicit destructive flag (219/220); NL loop (221).
