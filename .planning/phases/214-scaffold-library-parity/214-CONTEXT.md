# Phase 214 Context — Scaffold↔Library Parity & Published-Artifact Smoke Test

**Milestone:** v13.3 Scaffold↔Library Parity (📋 planned, scoped 2026-06-13)
**Status:** Scoped — needs `/gsd-discuss-phase 214` before planning.
**Depends on:** Phase 211 (COMP-04 — found the defect).

## Why this phase exists

COMP-04 (Phase 211) ran the first cold-cache time-to-working-app benchmark and produced the
honest first-time-experience number: a developer who installs the published `ferro-cli` and
scaffolds an app **cannot build it**. `cargo build` of the generated project fails with **52
compile errors**. The CLI steps themselves are sub-second; the entire "time to working app"
cost is a scaffold↔library API drift.

Evidence (committed): `phases/211-comp-04-time-to-working-app-benchmark/211-WEAKNESSES.md`
(Finding W1) and the benchmark apparatus at `ferro-cli/tests/benchmark_new_project.rs` +
`ferro-cli/tests/fixtures/benchmark/{Dockerfile,RESULTS.md}`.

Crucial fact: the generated `Cargo.toml` pins `ferro = { package = "ferro-rs", version = "0.2" }`
from crates.io (not a path dependency). So the local workspace binary reproduces the same
failure — **the published library, not the scaffolding binary, is the constraint.** This cannot
be hidden by testing against the local tree.

## The 52 errors, by root cause (from 211-WEAKNESSES W1)

| Generated code references | Problem | Likely fix direction |
|---------------------------|---------|----------------------|
| `ferro::error_response!(...)` (every API controller) | macro not exported by published `ferro` | export it from `ferro`, or change the controller template |
| `#[rule]` on request structs | validation attribute not in scope | export/`use` the attribute, or change the template |
| `ferro::Queue`, `ferro::QueueConfig` | unresolved | re-export from `ferro`, or change the job template |
| `use ferro_queue::{…}` in `make:job` output | `ferro-queue` absent from generated `Cargo.toml` | add the dep to the template's `Cargo.toml`, or route through a `ferro` re-export |
| `ActiveValue::Set(...)` in scaffold controllers | `ActiveValue` never imported | add the sea-orm import to the controller template |
| `crate::models::users` (make:auth output) | unresolved module | align make:auth output with the generated model layout |
| `ferro::database::connection` used as fn | it is a module | fix the template call site |

## Two deliverables

1. **Parity fix** — make a clean scaffold compile against the published `ferro` surface. For
   each symbol above, either export it from the `ferro` facade (preferred when the API is meant
   to be public) or change the scaffold template to use what the published crate actually
   exposes. The arbiter is: does `ferro new → make:auth → make:scaffold ×3 → make:job →
   cargo build` exit 0?

2. **Permanent CI guard** — a smoke test that scaffolds and builds **against the published
   artifact** and fails the pipeline on regression. COMP-04's apparatus is the basis; a
   published-crate variant (or a release-time job using the committed Dockerfile) is the
   natural shape. The Phase 211 benchmark already wires the `cargo build` exit-0 assertion —
   this phase makes it a release gate.

## Open questions to lock in discuss-phase

1. **Export vs template-change per symbol.** Which of the seven symbols should become public
   `ferro` API (the templates are the intended consumer) vs which are template bugs? This is a
   surface-design decision, not mechanical — it touches `framework/src/lib.rs` re-exports.
2. **`ferro-queue` exposure.** Add `ferro-queue` to the generated `Cargo.toml`, or re-export
   `Job`/`Queueable`/etc. under `ferro` so generated jobs need no extra dependency? (Check
   coherence with the v12.3 `ferro::queue` namespacing decision.)
3. **CI cadence.** Per-PR (catches drift early, but adds a full scaffold+build to every PR and
   needs network to crates.io) vs per-release (cheaper, but drift can land on master). Record
   the rationale (SC#5).
4. **Published-artifact testing mechanism.** Reuse the committed Dockerfile in CI, a
   `cargo install --version <released>` job, or a workspace test that builds a scaffold with the
   `ferro` dep pinned to the last published version?
5. **Requirement labels.** Derive `SCAF-*` IDs in REQUIREMENTS.md from W1 (this milestone left
   them TBD, matching the v13.1 pattern).

## Non-goals

- Re-authoring the projection render or the CLI's UX — this is strictly scaffold↔library parity
  + the guard.
- The other COMP-04 findings (W2 `libssl-dev`/`pkg-config` install prereq, W3 `make:scaffold`
  flag ordering) are already fixed in the committed benchmark apparatus; W3's flag-ordering may
  warrant a separate clap ergonomics fix but is out of scope here unless trivially co-located.
