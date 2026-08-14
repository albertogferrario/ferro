# Phase 247: Read-model delta → broadcast streaming - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-08-14
**Phase:** 247-read-model-delta-broadcast-streaming
**Areas discussed:** Broadcast seam, Delta payload + error redaction, Subscribe/completion race, Channel privacy, Client read-back, Test scope

---

## Broadcast seam

| Option | Description | Selected |
|--------|-------------|----------|
| Framework offload layer | Emit from the framework offload hook; keep snapshot_write write-only. Honors D-11 + 246 decoupling. | ✓ |
| ferro-projection direct API | Give snapshot_write an optional Broadcaster (symmetric with fold path). | |
| Snapshot-changed callback | Generic post-write hook the framework subscribes to. | |

**User's choice:** Framework offload layer.
**Notes:** Exact plumbing (hook arg vs worker-context slot) left to planning (CONTEXT D-03).

---

## Delta payload + error redaction

| Option | Description | Selected |
|--------|-------------|----------|
| Full value; redact failure | Completed delta carries value; failed delta carries a non-sensitive marker; raw error stays in snapshot + logs. | ✓ |
| Full envelope incl. raw error | Delta mirrors snapshot exactly (raw error to clients). | |
| Signal-only, client reads back | Delta is just 'ready'; client fetches over HTTP. | |

**User's choice:** Full value; redact failure.
**Notes:** Resolves the 246 client-exposure security flag; server-side fidelity preserved via snapshot read-back (CONTEXT D-05/D-06).

---

## Subscribe/completion race

| Option | Description | Selected |
|--------|-------------|----------|
| Pending marker + helper | Write {status:"pending"} at enqueue; OffloadHandle gains a race-safe resolve helper; document the pattern. | ✓ |
| Race-safe helper, no pending row | Helper + docs, but no pending snapshot. | |
| Document pattern only | Just document subscribe-then-read; no helper, no pending row. | |

**User's choice:** Pending marker + helper.
**Notes:** 246 D-08 deferred the pending marker here; delivers the 245 D-08 resolve methods. Pending write must route through the framework layer, not ferro-queue (D-11) — mechanism is planning (CONTEXT D-07/D-08/D-09).

---

## Channel privacy

| Option | Description | Selected |
|--------|-------------|----------|
| Public + unguessable handle | Capability model: public channel keyed by UUID v4 handle; zero-config. | ✓ |
| Public now, harden later | Public this phase; private-authorized noted as deferred. | |
| Private authorized channel | private-* channel gated by the broadcast authorizer; needs handle→owner metadata + authorizer. | |

**User's choice:** Public + unguessable handle.
**Notes:** Handle is a capability token (unguessable UUID, server-minted). Private-authorized channel deferred (CONTEXT D-11 + Deferred Ideas).

---

## Client read-back

| Option | Description | Selected |
|--------|-------------|----------|
| Redaction-aware read helper + docs | Ship read_result_redacted; framework stays route-agnostic, app wires the route. | ✓ |
| Built-in read-back route | 247 also registers a framework route returning the redacted result. | |
| Document only | No helper or route; consumers re-implement redaction. | |

**User's choice:** Redaction-aware read helper + docs.
**Notes:** Existing read_result (full envelope) retained for authorized/server-side use. Built-in route deferred (CONTEXT D-10 + Deferred Ideas).

---

## Test scope

| Option | Description | Selected |
|--------|-------------|----------|
| In-memory cross-process loop | Worker on Broadcaster A → subscriber on Broadcaster B (in-memory transport) receives redacted delta; + non-blocking assertion. | |
| Single-process E2E only | Local subscriber + non-blocking; rely on 246.1 for cross-process. | |
| Add env-gated redis cross-process | In-memory cross-process loop PLUS an env-gated live-redis variant (246.1 style). | ✓ |

**User's choice:** Add env-gated redis cross-process (most thorough).
**Notes:** Single-process is the degenerate case of the cross-process harness (CONTEXT D-12).

---

## Claude's Discretion

- Exact delta event-name string (recommend `"offload.result"`).
- Resolve-helper signature + timeout semantics.
- Module home for `read_result_redacted` and the resolve helper.
- Precise plumbing: Broadcaster → result hook (D-03); pending-write enqueue seam (D-08).

## Deferred Ideas

- Private-authorized result channel (needs handle→owner metadata + authorizer).
- Built-in framework read-back route.
