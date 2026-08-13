---
phase: 245-typed-result-handle-serializable-enforcement
verified: 2026-08-13T16:00:00Z
status: passed
score: 3/3
overrides_applied: 0
re_verification: null
---

# Phase 245: Typed Result Handle + Serializable Enforcement — Verification Report

**Phase Goal:** Make the offload call site ergonomic and the contract honest — return a typed handle
identifying where the result will land, and reject at compile time any method whose parameters or
return type are not serializable, with a diagnostic that names the offending type.

**Verified:** 2026-08-13T16:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Calling an offloaded method returns a typed result handle (not the bare value) | VERIFIED | `ferro-queue/src/offload.rs` L118: `async fn offload(self) -> Result<OffloadHandle<Self::Output>, Error>`; pass fixture `offload_handle.rs` L37–38 asserts `Offloadable<Output = Report>` and `OffloadHandle<Report>` at compile time |
| 2 | A `#[offload]` method with a non-serializable parameter or return type fails at compile time with a clear, type-naming diagnostic | VERIFIED | Both fail `.stderr` files contain the branded message naming the offending type: `RawHandle crosses the #[offload] isolation boundary and must be Serialize + DeserializeOwned` (L369 of param.stderr) and `RawReport crosses the #[offload] isolation boundary and must be Serialize + DeserializeOwned` (L288 of return.stderr); trybuild suite passes (per 245-02-SUMMARY.md) |
| 3 | The serializable boundary is documented as the module-isolation guarantee | VERIFIED | `docs/src/features/queues.md` contains `## Offloading Service Methods` section (L188); 4 occurrences of `isolation` (L255, L260, L273, L274); verbatim branded compiler error quoted (L273–274); SC#3 prose frames the contract as a structural isolation guarantee, not merely a restriction |

**Score:** 3/3 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-queue/src/offload.rs` | Four offload primitives: `OffloadSerializable`, `HandleKey`, `OffloadHandle<T>`, `Offloadable` | VERIFIED | 152 lines; all four types present with correct implementations (L27–31 `OffloadSerializable` + `#[diagnostic::on_unimplemented]`; L41–59 `HandleKey` with `Uuid::new_v4()`; L71–96 `OffloadHandle<T>` with `PhantomData<fn() -> T>` + `#[serde(skip)]`; L108–123 `Offloadable` async trait) |
| `ferro-queue/src/lib.rs` | Module declaration + re-export of four types | VERIFIED | L54: `mod offload;`; L72: `pub use offload::{HandleKey, OffloadHandle, OffloadSerializable, Offloadable};` |
| `framework/src/lib.rs` | `::ferro::queue` re-export of all four types | VERIFIED | L227–228: `HandleKey`, `OffloadHandle`, `OffloadSerializable`, `Offloadable` present in `pub mod queue { pub use ferro_queue::{ ... } }` |
| `ferro-macros/src/offload.rs` | `output_type` capture + `impl Offloadable` emission + param where-clause | VERIFIED | L65 `pub output_type: TokenStream2` in `OffloadMethodInfo`; L174–202 `collect_info` returns-type extraction (Result<T,E>→T, bare→T, default→()); L290–298 struct where-clause bounding field types + output_type to `OffloadSerializable`; L322–327 `impl ::ferro::queue::Offloadable for #job_ident { type Output = #output_type; }` |
| `ferro-macros/tests/ui/offload/pass/offload_handle.rs` | Compile-pass proof: `.offload()` returns `OffloadHandle<Output>` | VERIFIED | L37–38: `fn assert_output_is_report::<J: Offloadable<Output = Report>>()` + `assert_output_is_report::<ReportsServiceBuildMonthlyJob>();`; type-equality check without runtime dispatch |
| `ferro-macros/tests/ui/offload/fail/non_serializable_param.rs` | Compile-fail: non-serializable param type | VERIFIED | `pub struct RawHandle;` (no derive); `async fn process(&self, handle: RawHandle);` |
| `ferro-macros/tests/ui/offload/fail/non_serializable_param.stderr` | Contains "isolation boundary" and "RawHandle" | VERIFIED | "RawHandle crosses the #[offload] isolation boundary" at L369; "isolation boundary" note at L380; "RawHandle" named explicitly in both branded error lines |
| `ferro-macros/tests/ui/offload/fail/non_serializable_return.rs` | Compile-fail: non-serializable return type | VERIFIED | `pub struct RawReport;` (no derive); `async fn build(&self, id: i64) -> RawReport;` |
| `ferro-macros/tests/ui/offload/fail/non_serializable_return.stderr` | Contains "isolation boundary" and "RawReport" | VERIFIED | "RawReport crosses the #[offload] isolation boundary" at L288 and L321; "isolation boundary" note at L299 and L332 |
| `docs/src/features/queues.md` | `#[offload]` section documenting isolation boundary (SC#3) | VERIFIED | `## Offloading Service Methods` present; `OffloadHandle`, `.offload()`, `Serialize + DeserializeOwned`, `isolation` all confirmed; verbatim branded error block at L272–275; neutral voice confirmed (no trigger phrases found) |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-queue/src/offload.rs` | `uuid::Uuid::new_v4` | `HandleKey::new()` | VERIFIED | L46: `Self(Uuid::new_v4().to_string())` |
| `ferro-queue/src/offload.rs` | `PendingDispatch::dispatch` | `Offloadable::offload()` provided default | VERIFIED | L120: `crate::PendingDispatch::new(self).dispatch().await?;` |
| `framework/src/lib.rs` | `ferro_queue::{OffloadHandle, Offloadable, OffloadSerializable, HandleKey}` | `pub mod queue` re-export | VERIFIED | L225–231: all four names in the `pub use ferro_queue::{ ... }` block |
| `ferro-macros/src/offload.rs collect_info` | `OffloadMethodInfo.output_type` | return-type extraction | VERIFIED | L174–213: match over `ReturnType`; `Result<T,E>→T` via `find_map(GenericArgument::Type)`; bare `→T` verbatim; default `→()` |
| `ferro-macros/src/offload.rs emit_job_items` | `::ferro::queue::Offloadable` | emitted impl block | VERIFIED | L322: `impl ::ferro::queue::Offloadable for #job_ident` |
| `ferro-macros/src/offload.rs emit_job_items` | `::ferro::queue::OffloadSerializable` | struct where-clause over field types + output_type | VERIFIED | L294–295: `#( #field_types: ::ferro::queue::OffloadSerializable, )*` and `#output_type: ::ferro::queue::OffloadSerializable,` |
| `docs/src/features/queues.md` | `OffloadSerializable isolation boundary` | SC#3 prose | VERIFIED | L255–275: section "Serializable contract as the isolation boundary"; isolation framing explicit |

---

### Data-Flow Trace (Level 4)

Not applicable. This phase produces Rust types, a proc-macro, and documentation — no components that render dynamic data from a data source.

---

### Behavioral Spot-Checks

The orchestrator reports `cargo test -p ferro-queue -p ferro-macros -p ferro-rs` all green, including the trybuild suite. Disk is at 12 GiB (ENOSPC risk); heavy re-runs are not repeated. Structural evidence is conclusive.

| Behavior | Evidence | Status |
|----------|----------|--------|
| `handle_key_is_uuid_v4` unit test | offload.rs L130–135; 245-01-SUMMARY.md: "2/2 passed" | PASS |
| `handle_round_trips_with_non_serializable_t` unit test | offload.rs L141–150; 245-01-SUMMARY.md: "2/2 passed" | PASS |
| trybuild pass fixture `offload_handle.rs` compiles | 245-02-SUMMARY.md: "1 passed, 0 failed"; fixture verified on disk | PASS |
| trybuild fail fixtures match `.stderr` snapshots | 245-02-SUMMARY.md: all 3 fail fixtures match; `.stderr` files verified on disk | PASS |

---

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| OFFLOAD-02 | 245-01, 245-02, 245-03 | Typed result handle; non-serializable param/return fails at compile time with type-naming diagnostic (isolation boundary) | SATISFIED | SC#1: `OffloadHandle<T>` returned by `.offload()`; SC#2: two fail trybuild fixtures with branded diagnostic naming `RawHandle`/`RawReport`; SC#3: docs section with isolation framing |

OFFLOAD-02 status in REQUIREMENTS.md (L72) shows "Not started" — this reflects the state before Phase 245 executed and has not been updated post-completion. The traceability table maps OFFLOAD-02 to Phase 245 (L72), which is this phase. The requirement itself (L35–37) matches all three success criteria exactly. The table entry is a documentation artifact, not a code gap.

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `ferro-queue/src/lib.rs` L71 | Doc comment reads "Re-export async_trait for convenience" but the re-export is the offload surface | INFO | Inaccurate label; no runtime impact. Flagged as IN-01 in 245-REVIEW.md |
| `ferro-queue/src/offload.rs` L71 | `#[derive(...)]` on `OffloadHandle<T>` binds `T: Clone/Debug/PartialEq/Eq` through std derives (WR-01 in 245-REVIEW.md) | WARNING | Does not affect Phase 245 success criteria; the serde round-trip goal (OFFLOAD-02e) is satisfied by `#[serde(skip)]`. A success type that is `Serialize + DeserializeOwned` but not `Clone`/`Eq`/`Debug` cannot be used as `OffloadHandle<T>.clone()` / comparison — the module docstring's "regardless of T" claim is overstated for std traits. Not a goal-blocking gap. |
| `ferro-queue/src/offload.rs` L118–121 | `HandleKey` minted locally and not persisted with the enqueue payload (WR-02 in 245-REVIEW.md) | WARNING | Inert seam — per design decision D-08 the handle is explicitly inert in Phase 245; Phase 246 is where the result path (key persistence + correlation) is built. The docstring marks the handle as inert. Not a Phase 245 goal-blocking gap. |

No blocker anti-patterns found. Both warnings are pre-identified advisory items in 245-REVIEW.md and are non-blocking for Phase 245 goal achievement.

---

### Human Verification Required

None. All three success criteria are verifiable programmatically and the trybuild evidence is conclusive.

---

### Observations (Advisory — Non-Blocking)

1. **SC#2 branded message primacy.** The `OffloadSerializable` branded diagnostic (`isolation boundary`) is not the first `error[E0277]` in either fail `.stderr` — serde's own supertrait bound errors from `Offloadable: Serialize + DeserializeOwned` fire first. The PLAN explicitly acknowledges this (Open Question 1 in 245-02-PLAN.md) and the SUMMARY documents the mitigation applied (`#[serde(bound = "")]` + `output_type` added to struct where-clause). SC#2 requires "fails at compile time with a clear, type-naming diagnostic"; this holds — the branded message naming the offending type is present in the error output. Message ordering within a multi-error compilation is an advisory quality concern, not a goal failure.

2. **`OffloadHandle<T>` std-derive over-constraint (WR-01).** The docstring states the handle works "regardless of T" but `Clone`/`Debug`/`PartialEq`/`Eq` still require those traits on `T`. The discrepancy is a doc/derive gap, not a stated must-have. The serde round-trip goal (OFFLOAD-02e) is correctly satisfied.

3. **`HandleKey` not yet persisted with the job (WR-02).** Inert by design for Phase 245; Phase 246 is the result-path phase that addresses this. The docstring and the 245-REVIEW.md both flag the seam explicitly.

---

### Gaps Summary

No gaps. All three success criteria are verified against the actual codebase.

---

_Verified: 2026-08-13T16:00:00Z_
_Verifier: Claude (gsd-verifier)_
