# Phase 142: ferro-mcp Parity - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 142-ferro-mcp-parity
**Mode:** --auto (all choices auto-selected)
**Areas discussed:** stripe_webhook_events scan pattern, stripe_config_status scaffold detection, stripe_subscription_info fate, MCP tool descriptions, WebhookEventInfo response shape

---

## stripe_webhook_events — Scan Pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Scan `src/stripe/listeners.rs` only | Keep existing behavior, update regex | |
| Scan all `.rs` under `src/` | Broader scan, finds handlers anywhere in app | ✓ |
| Scan `src/providers/` only | Narrower, assumes provider pattern | |

**User's choice:** Scan all `.rs` under `src/` with closure regex `\.on\(\s*\|[a-zA-Z_]+:\s*(\w+)\s*\|` plus turbofish secondary.
**Notes:** [auto] No listener struct name in closure-based API — drop `listener` field, add `line: u32`.

---

## stripe_config_status — Scaffold Detection

| Option | Description | Selected |
|--------|-------------|----------|
| List files as-is | No change to logic, app files reflect new names naturally | |
| Add capability-axis boolean fields | Four booleans for checkout.rs, refund.rs, account.rs, webhook/ | ✓ |
| Replace scaffold_files with structured report | Breaking change, more structured | |

**User's choice:** Add four capability-axis boolean fields alongside existing `scaffold_exists` and `scaffold_files`.
**Notes:** [auto] Additive — no breaking change to existing tool output shape.

---

## stripe_subscription_info — Fate

| Option | Description | Selected |
|--------|-------------|----------|
| Retire the tool | Remove it; subscription axis gone from framework | |
| Keep with description update | Tool still useful for apps with tenant_billing tables | ✓ |
| Update to scan for new subscription patterns | Scan for different table name | |

**User's choice:** Keep as-is (behavior unchanged), description update only.
**Notes:** [auto] Apps that use billing still create tenant_billing migrations; tool remains useful.

---

## MCP Tool Descriptions

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal updates | Change only the listener reference | |
| Full description rewrite | Reflect SyncDispatcher model throughout | ✓ |

**User's choice:** Full description update for all three tools.
**Notes:** [auto] All three descriptions need updating for accuracy.

---

## WebhookEventInfo Response Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `listener` as `Option<String>` | Backward compat shim | |
| Hard-remove `listener`, add `line: u32` | Clean break, feature branch | ✓ |
| Keep `listener` with empty string | Minimal diff | |

**User's choice:** Hard-remove `listener` field, add `line: u32` line number.
**Notes:** [auto] Feature branch — no backward compat needed. Line number is more useful than empty listener name.

---

## Claude's Discretion

- File walk implementation (walkdir vs recursive read_dir)
- Whether `stripe_config_status` `scaffold_files` listing becomes recursive
- Exact wording of updated tool descriptions

## Deferred Ideas

- SyncDispatcher provider registration scanning
- Per-file handler coverage metrics
- stripe_subscription_info retirement review at v1.0
