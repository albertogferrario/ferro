# Phase 245: Typed result handle + serializable-contract enforcement - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-08-13
**Phase:** 245-typed-result-handle-serializable-enforcement
**Areas discussed:** Handle call-site surface, Enforcement mechanism + diagnostic, Handle type & key identity, Return-type contract precision

---

## Handle call-site surface

| Option | Description | Selected |
|--------|-------------|----------|
| `.offload()` on the Job | Enqueue via the already-public derived struct; macro adds associated `type Output`; returns `OffloadHandle<T>`. Reuses 244's struct, minimal new surface. | ✓ |
| Companion method mirror | Generate `reports.build_monthly_offload(..)` via an extension trait; reads like calling the verb; more generated surface. | |
| Free `offload(job)` fn | Symmetric with existing `dispatch(job)`; simplest, least discoverable. | |

**User's choice:** `.offload()` on the derived Job.
**Notes:** The `#[offload]` trait method itself stays `-> T` in-process (244 D-01/D-03); `.offload()` is the enqueue entrypoint layered on top, preserving the "one trait" property.

---

## Enforcement mechanism + diagnostic

| Option | Description | Selected |
|--------|-------------|----------|
| Offloadable trait + `on_unimplemented` | `Offloadable { type Output: OffloadSerializable; offload() }` houses `.offload()`; `OffloadSerializable` marker with `#[diagnostic::on_unimplemented]`; params+return enforced by bounds, one branded message. | ✓ |
| Static assertion block | `const _` block asserting each param + return type; names the type but serde-default wording; `.offload()` on a plain inherent impl. | |
| Serde derive + minimal return check | Params via existing derive, minimal return assertion; least code, mixed message styles, weakest diagnostic. | |

**User's choice:** Offloadable trait + `#[diagnostic::on_unimplemented]`.
**Notes:** MSRV 1.94.1 confirmed — `#[diagnostic::on_unimplemented]` freely available (first use in tree). This single decision also settles where `.offload()` and `type Output` live.

---

## Handle type & key identity

| Option | Description | Selected |
|--------|-------------|----------|
| idempotency_key else UUID | Key = `idempotency_key()` when Some, else fresh UUID v4; coherent with queue dedup and 246 snapshot keying. | |
| Always random UUID | Fresh UUID v4 every `.offload()` call, decoupled from `idempotency_key`; simplest; handle→result many-to-one reconciled later. | ✓ |
| Always content-addressed | Key = hash(job_type + payload); identical payloads share a handle; conflicts with the non-idempotent "always run" semantic. | |

**User's choice:** Always random UUID v4.
**Notes:** The handle is the identity of *this offload call*, not the result content. Reconciling deduped-job → multiple handles is deferred to 246/247.

---

## Return-type contract precision

| Option | Description | Selected |
|--------|-------------|----------|
| Output = T, contract-only | For `Result<T,E>` the handle is `OffloadHandle<T>`; `E` stays stringified→job-failure (D-07), not Serialize-bound; worker still discards value in 245; persistence is 246. | ✓ (default) |
| Output = whole Result | `OffloadHandle<Result<T,E>>`; entire return must be Serialize; forces `E: Serialize`, reversing D-07; duplicates the queue failure path. | |
| Capture value now | 245's `handle()` starts persisting/returning the value — pulls Phase 246 scope into 245. | |

**User's choice:** Discussion skipped at this area; Claude locked the recommended default (Output = T, contract-only) as the coherent choice respecting D-07 and the 245/246 boundary.
**Notes:** Enforcement targets the success type `T` and the parameters, never `E`. 245 locks the typed contract + compile-time enforcement ahead of persistence by design.

## Claude's Discretion

- Module home for `OffloadHandle` / `Offloadable` / `OffloadSerializable` / `HandleKey`.
- Whether `OffloadHandle<T>` derives `Serialize, Deserialize, Clone, Debug` (recommended yes).
- Exact `#[diagnostic::on_unimplemented]` wording.
- Parameter-side bound expression (per-field bound vs generated `where` assertion).
- UUID crate/path for `HandleKey`.

## Deferred Ideas

- Result → projection snapshot + terminal error state (246); handle resolve/subscribe/streaming (247); deduped-job handle reconciliation (246/247); `#[offload(...)]` config surface (future additive).
