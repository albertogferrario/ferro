# Phase 214: Scaffold↔Library Parity & Published-Artifact Smoke Test - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 214-scaffold-library-parity
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** Per-symbol resolution, Queue exposure, CI cadence, Test mechanism, Requirement labels

---

## Per-symbol resolution (export vs template-change)

| Symbol | Option: EXPORT from `ferro` | Option: TEMPLATE change | Selected |
|--------|-----------------------------|-------------------------|----------|
| `error_response!` | Define macro + facade-export | Change controllers to a different error pattern | EXPORT ✓ |
| `ActiveValue` | Add to `sea_orm` facade re-export | Add `use sea_orm::ActiveValue` in template | EXPORT ✓ |
| `ferro::Queue`/`QueueConfig` | Add top-level re-export | Emit `ferro::queue::*` | TEMPLATE ✓ |
| `ferro_queue` dep | Add `ferro-queue` to generated Cargo.toml | Route via `ferro::queue::*`, no dep | TEMPLATE ✓ |
| `#[rule]` | Re-export attribute standalone | Emit `#[derive(ValidateRules)]` | TEMPLATE ✓ |
| `crate::models::users` | n/a | Align make:auth with model layout | TEMPLATE ✓ |
| `ferro::database::connection` | n/a | Fix call site (module, not fn) | TEMPLATE ✓ |

**Decision rule:** EXPORT when the symbol is genuine public API the templates are the intended
consumer of AND it coheres with the facade design; TEMPLATE-change when the published crate
already exposes an equivalent (namespaced) or it's a plain template bug.

**Grounding (scout evidence):**
- `framework/src/lib.rs:122` re-exports sea_orm traits but **not** `ActiveValue`; `lib.rs:78`
  documents the facade intent ("saves users from `use sea_orm::*`") → `ActiveValue` belongs there.
- `framework/src/lib.rs:194-202` already namespaces the queue under `ferro::queue::*` (v12.3) →
  templates target the namespace; a top-level re-export would duplicate the control surface.
- No `error_response!` equivalent exists in `framework/src/` → genuine missing public API.
- `ferro-macros/src/lib.rs:542` — `rule` is the `ValidateRules` derive helper attribute.

**Notes:** Exact `error_response!` signature/return type deferred to research (used in both
`.map_err` and `.ok_or_else` positions in `scaffold.rs`).

---

## Queue exposure in generated projects

| Option | Description | Selected |
|--------|-------------|----------|
| Add `ferro-queue` dep | Generated Cargo.toml gains a second ferro dependency | |
| Route via `ferro::queue::*` | Generated jobs import from the existing facade re-export; no extra dep | ✓ |

**User's choice (auto):** Route via `ferro::queue::*`. Coherent with v12.3 namespacing; keeps
generated projects single-dependency.

---

## CI guard cadence

| Option | Description | Selected |
|--------|-------------|----------|
| Per-PR only | Fast, but cannot test the published artifact (not yet published) | |
| Per-release only | Tests published artifact, but drift lands on master with no early signal | |
| Two-layer | Per-PR path-dep build (fast) + release-time published-artifact gate | ✓ |

**User's choice (auto):** Two-layer. **Rationale:** the published-artifact check is inherently
post-publish (cannot build against an unpublished version), so it cannot gate a PR; the per-PR
path-dep layer closes the chicken-and-egg gap with fast pre-publish drift detection.

---

## Published-artifact test mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Committed Dockerfile (release) + benchmark apparatus (per-PR) | Reuse evidence apparatus; cold-cache realism | ✓ |
| `cargo install --version <released>` job | Slow, flaky | |
| Workspace test with pinned published dep | Mixes published + workspace; weaker realism | |

**User's choice (auto):** Reuse `ferro-cli/tests/fixtures/benchmark/Dockerfile` for the release
gate and `ferro-cli/tests/benchmark_new_project.rs` for the per-PR layer.

---

## Requirement labels

**User's choice (auto):** Derive SCAF-01..SCAF-05 in REQUIREMENTS.md from 211-WEAKNESSES W1
(matching the v13.1 TBD-then-derive pattern). See CONTEXT.md D-10.

---

## Claude's Discretion

- Exact `error_response!` macro signature/return type (research-determined).
- Whether parity fix + CI guard ship as one plan or split (planner's call).
- Placement of CI jobs in `.github/workflows/` and release-pipeline wiring.

## Deferred Ideas

- COMP-04 W3 — `make:scaffold` flag ordering (clap ergonomics) — future CLI phase.
- COMP-04 W4 — `make:model` vs `make:scaffold` naming/doc drift — separate doc-alignment.
- COMP-04 W2 — move CLI off `native-tls` → rustls to drop OpenSSL build prereq — hardening idea.
