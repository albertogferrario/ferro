# Phase 220: Confirmation Gating for Destructive Actions - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-14
**Phase:** 220-confirmation-gating-for-destructive-actions
**Mode:** `--auto` (gray areas auto-selected; recommended defaults chosen and logged)
**Areas discussed:** Tool surface, Token binding, TTL/expiry, Store wiring, Destructive detection, Feature flag + dependency hygiene, Result envelopes, Seam insertion

---

## Tool surface

| Option | Description | Selected |
|--------|-------------|----------|
| `request_confirm_<action>` + `confirm_<action>` two tools; bare destructive call → confirmation-required | matches SC#2 named flow + SC#1 | ✓ |
| Single confirm tool; the write tool returns pending_confirmation (ARCHITECTURE Model A) | SC#2 explicitly names request_confirm_ + confirm_ | |

**Auto-selected:** two synthesized tools per destructive action; bare destructive `<action>` call gated at the D-08 seam.

## Token binding

| Option | Description | Selected |
|--------|-------------|----------|
| Token bound to (tenant_id, action_name, record_id); single-use; server-generated | SC#4 mismatch + SC#2 exactly-once | ✓ |
| Agent-supplied / unbound token | bypassable; fails SC#4 | |

**Auto-selected:** server-generated, bound, single-use (consumed by confirm()).

## TTL / expiry

| Option | Description | Selected |
|--------|-------------|----------|
| Configurable TTL on McpServerConfig (default 300s) passed to request_confirmation; store timer handles expiry | SC#3; reuses store's internal timer | ✓ |

**Auto-selected.** `confirm()` after TTL → None → expired error.

## Store wiring

| Option | Description | Selected |
|--------|-------------|----------|
| `InMemoryConfirmationStore` (v15.0 skeleton); registered alongside the dispatcher | DB-backed deferred per REQUIREMENTS | ✓ |

**Auto-selected.** Placement (dispatcher field vs param) is research/discretion; must not leak into the non-confirmation build.

## Destructive detection

| Option | Description | Selected |
|--------|-------------|----------|
| `transition_trigger.is_some()` (reuse 218/219 signal) | no projection change | ✓ |
| New `requires_confirmation`/`irreversible` ActionDef flag | deferred | |

**Auto-selected:** transition_trigger.

## Feature flag + dependency hygiene (central decision)

| Option | Description | Selected |
|--------|-------------|----------|
| `confirmation` Cargo feature on ferro-mcp-server gating an OPTIONAL ferro-ai dep; feature-gate ferro-ai so confirmation excludes reqwest/llm | ARCHITECTURE Phase 4 note; SC#5; toolchain-only build | ✓ |
| Add ferro-ai unconditionally | drags reqwest/reqwest-eventsource into ferro-mcp-server | |
| Extract ConfirmationStore into a new ferro-confirmation crate | heavier; fallback only | |

**Auto-selected:** feature-gate. **RESEARCH-CRITICAL:** `ferro-ai/Cargo.toml` has NON-optional `reqwest`/`reqwest-eventsource`; research must make them optional behind a default `llm`/`classification` feature and confirm `src/confirmation/` is transitively reqwest-free, so `ferro-ai` can be depended on with `default-features=false, features=["confirmation"]`. Fallback: extract to `ferro-confirmation`.

## Result envelopes

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse 219 `CallToolResult::structured` / `write_tool_error_result`; strict-deser guard extended | SC consistency; single envelope source | ✓ |

**Auto-selected.**

## Seam insertion

| Option | Description | Selected |
|--------|-------------|----------|
| Intercept at the 219 D-08 seam in dispatch_write; confirm tools wrap the same dispatch machinery | single source of truth | ✓ |

**Auto-selected.**

## Claude's Discretion

- Token format/field name; store as dispatcher-field vs param; McpServerConfig TTL field name; whether confirm_<action> re-runs guard re-eval at execute (recommended yes).

## Deferred Ideas

- NL loop (221); DB-backed confirmation store; explicit requires_confirmation ActionDef flag; gestiscilo adoption.
