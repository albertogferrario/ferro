# Phase 140: Core reshape - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 140-core-reshape
**Mode:** --auto with critical roadmap review

---

## Roadmap Critique

The original phase plan (140-143) was reviewed before discussion began. Key findings that shaped the context:

| Issue | Original | Resolution |
|-------|----------|------------|
| Phase 140 alone was an orphaned abstraction | `ProcessedEventLog` with no in-framework consumer until Phase 142 | Merged 140+141 into new Phase 140 |
| Phases 142+143 were artificially split | Typed events and new event types follow identical pattern | Merged into new Phase 141 |
| No ferro-mcp update in any phase | `stripe_webhook_events` scans for wrong patterns after reshape | Added new Phase 142 |
| `StripeEvent` vs `ferro_events::Event` dual-trait gap | Unresolved — double-fire risk if both wired | Option A selected (see below) |
| Version cadence | 4 crates.io releases (0.4→0.5→0.6→0.7) for one consumer | Reduced to 2 releases (0.4, 0.5) |

## Dispatch Architecture Decision

**Area:** Whether Stripe event structs implement `ferro_events::Event`

| Option | Description | Selected |
|--------|-------------|----------|
| Drop `ferro_events::Event` | `SyncDispatcher` is sole registry; `ProcessStripeWebhook` holds `Arc<SyncDispatcher>` | ✓ |
| Keep `ferro_events::Event` | Both dispatch paths remain; double-fire risk if consumer wires both | |

**User's choice:** Option A — drop `ferro_events::Event` from Stripe events

**Rationale:** Payment-correctness events require exactly-once semantics. The `ferro_events` broadcast bus has no ordering or exactly-once contract — it was the wrong primitive for Stripe events from the start. `SyncDispatcher` as the sole handler registry eliminates double-fire risk by design. `ProcessStripeWebhook` sharing `Arc<SyncDispatcher>` means handlers are registered once and work across both dispatch paths.

## Auto-selected Decisions

All remaining gray areas auto-selected (recommended defaults):

- **Builder typestate vs runtime check for idempotency_key:** Runtime check — simpler, sufficient
- **DashMap acquisition:** Add as direct dep to ferro-stripe/Cargo.toml (already transitive in workspace)
- **webhook/events.rs in Phase 140:** No reshape — event structs keep current shape; Phase 141 owns the event typing work
- **webhook/mod.rs fate:** Implementer discretion (re-export shim or explicit pub mod declarations)

## Deferred Ideas

- Webhook secret rotation — deliberate non-goal, revisit pre-1.0 if consumer needs it
- Typestate builder for CheckoutBuilder — complexity not warranted at this stage
