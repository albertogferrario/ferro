---
phase: 247
slug: read-model-delta-broadcast-streaming
status: verified
threats_open: 0
asvs_level: 1
created: 2026-08-14
---

# Phase 247 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| snapshot store → client-facing read-back | The full envelope (including the raw `Display` error) is written server-side by `persist_error`; `read_result_redacted` is the only permitted path for shaping data toward a browser client. | `OffloadResult<T>` — raw error string on the server side, redacted `"terminal error"` marker on the client side |
| worker → broadcast bus → subscribed client | The worker publishes a delta that crosses the 246.1 broadcast transport (in-memory or Redis) to any client subscribed on the capability-keyed channel. | Redacted delta payload: `{"status":"completed","value":<v>}` or `{"status":"failed"}` with no error field |
| resolve() subscriber → broadcast channel | The resolve helper subscribes to the public capability-keyed channel and treats the delta as an untrusted wakeup, reconciling against the authoritative snapshot. | Wakeup signal only; result is read from `read_result`, not from the delta payload |
| documented client → redacted delta / redacted read-back | The documented browser path consumes only redacted surfaces (`read_result_redacted` and the redacted delta), never the raw error. | Redacted `OffloadResult<T>` |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-247-info-disclosure | Information Disclosure | `read_result_redacted` + result hook broadcast payload | mitigate | (a) `read_result_redacted` replaces the Failed arm's raw error with `"terminal error"` and never returns the raw value (offload.rs:225–235). (b) The failed delta arm is `json!({ "status": "failed" })` with no `error` key (offload.rs:344). (c) `persist_error` still writes the full raw error to the snapshot only (offload.rs:151). (d) Integration test `offload_failed_delta_is_redacted` asserts `data.get("error").is_none()` and that `"sensitive-secret-value"` is absent from the delta while present in the `read_result` snapshot (offload_delta_broadcast.rs:304–332). | closed |
| T-247-input-validation | Tampering / Input Validation (V5) | `OffloadResult<T>` envelope deserialization | mitigate | The `Pending` variant is fieldless (offload.rs:100 — bare `Pending,` with no fields) and carries no smuggleable data. The enum retains `#[serde(tag = "status", rename_all = "snake_case")]` strict internally-tagged parsing (offload.rs:84–85). No new deserialization surface beyond the added tag; existing `serde_json::from_value` path unchanged (offload.rs:205). | closed |
| T-247-hook-failfail | Denial of Service | result hook / job retry | mitigate | The result hook logs `tracing::warn!` and returns `()` on persist failure, returning early before broadcasting so no delta is emitted without a backing snapshot (offload.rs:286–290, 352–358). `broadcast_delta` logs `tracing::warn!` and swallows send errors without propagating them to the hook (offload.rs:313–317). A broadcast failure cannot mark the job failed or trigger a retry storm. | closed |
| T-247-hostile-payload | Tampering | cross-replica bus payload (Redis / 246.1 transport) | mitigate | Inherited from Phase 246.1 T-246.1-03: the bus envelope is parsed with strict `serde_json` and dropped on error. Phase 247 adds no new bus-parsing surface — the delta rides the existing `ServerMessage::Event` fan-out (offload.rs:474). The `redis_cross_replica` test uses the shipped `RedisTransport` without any new parse layer (offload_delta_broadcast.rs:407–469). | closed |
| T-247-resolve-wakeup | Spoofing | `resolve()` delta-triggered read-back | mitigate | `resolve()` subscribes before reading the snapshot (offload.rs:447–451), short-circuits a terminal handle via `read_result` (offload.rs:459–465), and on receiving any delta treats it only as a wakeup: the result is obtained by calling `read_result::<T>(key, db)` keyed on the handle the caller already holds, never from the delta payload (offload.rs:477–480). A forged or spurious `offload.result` event can trigger only a DB read; it cannot inject a fabricated value. | closed |
| T-247-handle-enum | Spoofing | public channel `projection.offload.result.{handle}` | accept | Accepted — see Accepted Risks Log. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-247-01 | T-247-handle-enum | The handle is a UUID v4 (122 bits of entropy) minted server-side in Phase 245 and returned only to the enqueuing caller. It functions as a capability token: a client that holds a handle can read or subscribe to exactly one result, and guessing a valid handle is computationally infeasible. A leaked handle exposes one result for one job. No additional access control is layered on the channel this phase; the risk is accepted as bounded and documented. The capability-token model and the single-result exposure caveat are restated in `docs/src/features/queues.md` (line 303). | gsd-security-auditor | 2026-08-14 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-08-14 | 6 | 6 | 0 | gsd-security-auditor |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-08-14
