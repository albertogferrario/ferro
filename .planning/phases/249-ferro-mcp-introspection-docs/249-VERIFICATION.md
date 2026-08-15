---
phase: 249-ferro-mcp-introspection-docs
verified: 2026-08-15T12:00:00Z
status: passed
score: 8/8
overrides_applied: 0
---

# Phase 249: `ferro-mcp` Introspection + Docs — Verification Report

**Phase Goal:** Close the single-source loop — surface offloadable methods through `ferro-mcp` so an agent reads the same trait as the in-process contract, the wire payload, and the offload spec; and document the authoring surface, result path, scaling model, and non-goals.
**Verified:** 2026-08-15
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC1 | `list_services` marks offloadable methods and exposes their derived payload schema (queue + typed param list) | VERIFIED | `OffloadableMethod` + `OffloadParam` structs in `list_services.rs:32–57`; `scan_offload_methods_from_files` wired on both `execute()` branches (grep count == 2); `skip_serializing_if = "Vec::is_empty"` guards additive field; six unit tests pass |
| SC2 | Docs cover: authoring `#[offload]`, the result handle + streaming pattern, the deployable worker / scaling model, and the deferred elastic direction | VERIFIED | `docs/src/features/offload.md` exists with 4 top-level `##` sections: Authoring, Result path, Scaling model, Non-goals (2.0 direction); contains `enqueue_and_mark_pending`, `resolve`, `read_result_redacted`, `CreateProjectionSnapshotsTable`, and the `OffloadHandle<T>` typed-handle explanation |
| SC3 | The "many-user" scaling answer (stateless tier + replicable workers + cache + queue) is documented as the framework's capacity story | VERIFIED | `offload.md §Scaling model` states the architecture explicitly; `serve --no-worker` and `worker --queue <class>` deploy recipe present; `### Honest limitations` covers DB connection ceiling, no OTel/metrics, latency bound; neutral-voice grep count == 0 |

**Score: 3/3 roadmap truths verified**

### Plan 01 Must-Have Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| T1 | An agent calling `list_services` sees each offloadable method with its declared queue and typed param list | VERIFIED | `OffloadableMethod { name, queue, params }` serialized in `ServiceItem.methods`; `scan_offload_methods` test asserts 2 methods found with correct queues; `extract_method_params` populates typed `OffloadParam` list |
| T2 | A plain (non-offload) service serializes to exactly `{name, binding_type}` — no bytes added (D-02) | VERIFIED | `plain_service_unchanged` test asserts `serde_json::to_string` of a zero-methods `ServiceItem` produces no `"methods"` key; `skip_serializing_if = "Vec::is_empty"` on the field |
| T3 | The `list_services` tool description states offloadable methods and their payload are surfaced (D-03) | VERIFIED | `service.rs:606` — "discovering which service methods are offloadable"; `service.rs:610` — "Plain services omit the `methods` field" |

**Score: 3/3 plan 01 truths verified**

### Plan 02 Must-Have Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| T4 | A developer reading the docs finds one canonical page (`offload.md`) covering authoring, result-handle/streaming pattern, deployable-worker scaling recipe, honest limitations, and the deferred elastic 2.0 non-goals | VERIFIED | `offload.md` exists with all required sections; PgBouncer mentioned; KEDA/WASM/Nomad in Non-goals section |
| T5 | `queues.md` no longer duplicates the offload prose; it points to `offload.md` | VERIFIED | `enqueue_and_mark_pending` count in `queues.md` == 0; `## Subscribe and await an offloaded result` section removed; pointer paragraph at queues.md:188–192 links to `offload.md` |
| T6 | `deployments.md` cross-links the scaling recipe; the mdBook nav registers `offload.md` | VERIFIED | `deployments.md:9` — blockquote callout `(offload.md#scaling-model)`; `SUMMARY.md:25` — `[Work Distribution (Offload)](features/offload.md)` between queues and notifications |

**Score: 3/3 plan 02 truths verified**

---

## Required Artifacts

| Artifact | Status | Evidence |
|----------|--------|----------|
| `ferro-mcp/src/tools/list_services.rs` | VERIFIED | Contains `OffloadableMethod`, `OffloadParam`, `ServiceItem.methods` (additive), `detect_offload_attr`, `extract_method_params`, `scan_offload_methods_from_files`, 6-test inline module; all three levels pass |
| `ferro-mcp/src/service.rs` | VERIFIED | `list_services` tool description contains "offloadable" and "Plain services omit the `methods` field" |
| `ferro-mcp/src/tools/generation_context.rs` | VERIFIED | `pub offload: &'static str` field at line 23; points to `docs/src/features/offload.md` in the populated value at line 602; no authoring template added (`code_templates` count == 0) |
| `docs/src/features/offload.md` | VERIFIED | Exists; 4 top-level `##` sections; `## Honest limitations` subsection; PgBouncer, OpenTelemetry/Prometheus, worker-scheduling-bound latency, KEDA/WASM non-goals all present |
| `docs/src/SUMMARY.md` | VERIFIED | `features/offload.md` registered at line 25, between queues (24) and notifications (26) |
| `docs/src/features/queues.md` | VERIFIED | Pointer paragraph present; `enqueue_and_mark_pending` count == 0; `## WorkerLoop Configuration` and `## MCP Tools` sections intact |
| `docs/src/features/deployments.md` | VERIFIED | `(offload.md#scaling-model)` deep cross-link at line 9; `## Artifact Storage` and `## Atomic Promote Model` sections intact |

---

## Key Link Verification

| From | To | Via | Status | Evidence |
|------|----|-----|--------|---------|
| `list_services.rs::execute()` | `scan_offload_methods_from_files` | Both runtime and static branches | WIRED | `grep -c` of `scan_offload_methods_from_files(project_root, &mut services)` == 2 |
| `ServiceItem.methods` | JSON output | `skip_serializing_if = "Vec::is_empty"` | WIRED | `grep -c 'skip_serializing_if = "Vec::is_empty"'` == 2 (one on `params`, one on `methods`) |
| `docs/src/SUMMARY.md` | `docs/src/features/offload.md` | mdBook nav entry | WIRED | `[Work Distribution (Offload)](features/offload.md)` at line 25 |
| `docs/src/features/queues.md` | `offload.md` | Pointer paragraph | WIRED | `grep -q "(offload.md)"` succeeds |
| `docs/src/features/deployments.md` | `offload.md#scaling-model` | Blockquote callout | WIRED | `grep -c "(offload.md#scaling-model)"` == 1; `## Scaling model` heading exists in `offload.md` as anchor target |

---

## Data-Flow Trace (Level 4)

Not applicable to this phase. The deliverables are static-analysis tooling and documentation files. No dynamic data rendering paths exist.

---

## Behavioral Spot-Checks

Step 7b: SKIPPED (no running server required; the deliverable is a static source parser invoked inside the MCP tool, plus documentation files). Evidence from the pre-existing test suite (`cargo test -p ferro-mcp` — 320+ passed, 0 failed, recorded in 249-01-SUMMARY.md) confirms the parser logic under all six unit scenarios.

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| OFFLOAD-06 | 249-01-PLAN, 249-02-PLAN | Offloadable methods introspectable via `ferro-mcp`; docs cover authoring surface, result path, scaling model, non-goals / deferred elastic direction | SATISFIED | SC1–SC3 verified above; both plan waves complete |

**Note on traceability table:** `REQUIREMENTS.md` line 76 shows OFFLOAD-06 as "Not started" in the traceability table — this is a known tooling gap (see `project_gsd_phase_complete_traceability_table_gap.md` in project memory): the `gsd phase complete` command flips the checkbox (`[x]` at line 55) but leaves the table row stale. The checkbox at line 55 correctly shows `[x]`. The status should be updated manually to "Complete".

---

## Anti-Patterns Found

No blockers. The two REVIEW warnings (WR-01, WR-02) were addressed:

- **WR-01 (named `#[service(impl = X)]` form):** Fixed before the REVIEW was written. `extract_service_impl_name` is present in `list_services.rs:263` and called at both extraction sites (lines 154 and 390). A test `extract_service_impl_name_positional_and_named` at line 617 covers positional, named (`impl = X`), named with `fake = Y`, reverse key order, and the "ImplRegistry" non-keyword prefix case.
- **WR-02 (multiline `#[service(...)]` attribute):** Left as-is per REVIEW guidance — the review classified this as lower-frequency and acceptable for a best-effort read surface. No blocker.

The four INFO findings (IN-01 through IN-04) are all classified as non-critical: a stale macro-comment disagreement outside this phase's file scope (IN-01), spacing-dependent queue-name matching that holds for rustfmt-formatted source (IN-02), a state-machine edge case in malformed source that causes no panic and mis-attributes at most one method (IN-03), and a double WalkDir traversal with no correctness impact (IN-04).

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `REQUIREMENTS.md` | 76 | Traceability table row stale ("Not started" vs actual "Complete") | Info | No runtime impact; cosmetic only; known tooling gap |

---

## Human Verification Required

None. All must-haves are verifiable programmatically via grep and file existence checks. The docs voice check (neutral-voice grep == 0) and the structural checks (section presence, cross-link anchors, nav ordering) are all confirmable without running the application.

---

## Gaps Summary

No gaps. All eight observable truths are VERIFIED, all seven artifacts are substantive and wired, all key links are confirmed, and no blocker anti-patterns were found.

The REQUIREMENTS.md traceability table row for OFFLOAD-06 remains stale but is not a code gap — it is a cosmetic documentation maintenance item arising from a known tooling limitation.

---

_Verified: 2026-08-15T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
