---
phase: 245-typed-result-handle-serializable-enforcement
plan: "03"
subsystem: docs
tags: [offload, typed-handle, serializable-contract, isolation-boundary, documentation]
dependency_graph:
  requires: [245-02]
  provides: [offload-doc-isolation-boundary, offload-doc-authoring-surface]
  affects: [docs/src/features/queues.md]
tech_stack:
  added: []
  patterns:
    - "Serializable contract documented as module-isolation guarantee (SC#3)"
    - "Verbatim branded compiler diagnostic quoted from 245-02 .stderr fixtures"
key_files:
  created: []
  modified:
    - docs/src/features/queues.md
decisions:
  - "Inserted section after ## Dispatching Jobs, before ## WorkerLoop Configuration — reads as the ergonomic alternative to hand-writing a Job"
  - "Message primacy documented honestly: serde supertrait E0277s fire before the branded OffloadSerializable diagnostic; section notes this ordering explicitly"
  - "Resolve/subscribe described as 'a later result-path capability' — no internal phase numbers or codenames (neutral-voice rule)"
  - "Param and return branded messages quoted from 245-02-SUMMARY.md; return case used as the in-doc example (RawReport) since it demonstrates the full isolation framing"
metrics:
  duration_seconds: 71
  completed_date: "2026-08-13T15:20:45Z"
  tasks_completed: 1
  files_changed: 1
requirements: [OFFLOAD-02]
---

# Phase 245 Plan 03: Documentation — Offload Isolation Boundary — Summary

`## Offloading Service Methods` section added to `docs/src/features/queues.md`, documenting the `#[offload]` authoring surface, the typed `OffloadHandle<T>`, the success-type contract, and — the SC#3 central point — the serializable contract as the module-isolation guarantee, with the verbatim branded compiler diagnostic from the 245-02 trybuild fixtures.

## One-liner

`docs/src/features/queues.md` extended with `## Offloading Service Methods`: `#[offload]` macro derivation (`<Trait><Method>Job` naming), `.offload()` enqueue entrypoint, inert `OffloadHandle<T>` (`.key()/.id()` only), `Result<T,E>`→`OffloadHandle<T>` success-type contract, and the serializable-as-isolation-boundary guarantee with verbatim SC#3 compiler diagnostic.

## Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add `## Offloading Service Methods` section to queues.md | 52441b0a | docs/src/features/queues.md |

## Section Coverage

The new section (`## Offloading Service Methods`, inserted after `## Dispatching Jobs`, before `## WorkerLoop Configuration`) covers:

**Authoring surface (D-01):** `#[offload]` on a `#[service]` trait method derives a `ferro-queue` Job; the trait method keeps its in-process signature. Derived Job name pattern `<TraitPascalCase><MethodPascalCase>Job` confirmed (e.g. `ReportsServiceBuildMonthlyJob`). Call-site example using `ReportsService` trait matches the interfaces block from the plan.

**Typed handle (D-02, D-08):** `.offload()` returns `Result<OffloadHandle<T>, Error>`. Handle is inert — `.key()` / `.id()` only. Resolve/subscribe described as "a later result-path capability" with no internal phase references.

**Success-type contract (D-09):** Table maps `-> T` → `OffloadHandle<T>`, `-> Result<T,E>` → `OffloadHandle<T>` (E stringified, not required Serialize), `-> ()` → `OffloadHandle<()>`.

**Serializable contract as isolation boundary (SC#3):** Framed as a structural guarantee: the payload must be fully described by serializable data to cross the boundary. Verbatim branded diagnostic from `non_serializable_return.stderr` quoted with the message primacy note (serde supertrait E0277s fire first; branded line follows).

## Acceptance Criteria Verification

- `grep offload docs/src/features/queues.md` — 14 matches
- `grep isolation docs/src/features/queues.md` — 4 matches (heading + prose + 2 branded error lines)
- `## Offloading Service Methods` heading — present
- `#[offload]` — present
- `OffloadHandle` — 8 occurrences
- `.offload()` — present
- `Serialize + DeserializeOwned` — 4 occurrences
- Fenced block with `isolation boundary` substring — present (verbatim .stderr quote)
- Neutral-voice check (`killer feature`, `the bet`, `competitor`, `load-bearing`) — 0 matches

## Deviations from Plan

None. The plan specified exactly one task: add the section. The section was added as specified, using the verbatim diagnostic text from 245-02-SUMMARY.md.

## Phase 245 Closure

All three Success Criteria are now TRUE:

| SC | Description | Status |
|----|-------------|--------|
| SC#1 | `.offload()` returns a typed `OffloadHandle<T>` carrying the success type | TRUE (245-01/02) |
| SC#2 | Non-serializable param or return type fails trybuild with a type-naming message | TRUE (245-02) |
| SC#3 | Serializable boundary documented as module-isolation guarantee | TRUE (245-03, this plan) |

Phase 245 is complete and ready for `/gsd-verify-work`.

## Known Stubs

None. The documentation covers the shipped behavior exactly. The inert-handle property is documented as a current-phase constraint, not a stub — resolve/subscribe is a planned future capability documented as such.

## Threat Flags

None. The change is a Markdown documentation file only; no code, endpoints, or auth paths introduced. T-245-07 (public-docs voice hygiene) is satisfied — no strategy framing, no competitor language, no unreleased-feature codenames in the added text.

## Self-Check: PASSED
