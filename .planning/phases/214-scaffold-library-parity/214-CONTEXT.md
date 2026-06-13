# Phase 214: Scaffold↔Library Parity & Published-Artifact Smoke Test - Context

**Gathered:** 2026-06-13
**Status:** Ready for planning
**Depends on:** Phase 211 (COMP-04 — the cold-cache benchmark that found the defect)

<domain>
## Phase Boundary

Make a freshly scaffolded ferro app compile against the **published** `ferro` crate, and add a
CI guard that keeps it that way. Two deliverables:

1. **Parity fix** — align the `ferro-cli` scaffold templates with the published `ferro` surface
   so `ferro new → make:auth → make:scaffold ×3 → make:job → cargo build` exits 0. For each
   symbol the generated code references, either export it from the `ferro` facade (when it is
   intended public API) or change the template to use what the published crate already exposes.
2. **Permanent CI guard** — a smoke test that scaffolds and builds against the published
   artifact and fails the pipeline on regression, so a non-compiling release can never ship
   silently again.

**Out of scope:** re-authoring the projection render or the CLI's UX; this is strictly
scaffold↔library parity + the guard. COMP-04 W2 (`libssl-dev`/`pkg-config` prereq) and W3
(`make:scaffold` flag ordering) are already fixed in the committed benchmark apparatus; W3's
clap flag-ordering ergonomics is a separate concern, out of scope unless trivially co-located.

</domain>

<decisions>
## Implementation Decisions

### Per-symbol resolution (export vs template-change)

The arbiter for every symbol: does the clean scaffold sequence `cargo build` exit 0 against the
published `ferro-rs`? The decision rule — **export** when the symbol is genuinely public API the
templates are the intended consumer of AND it coheres with the existing facade design;
**template-change** when the published crate already exposes an equivalent (often namespaced) or
when it is a plain template bug.

- **D-01 — `error_response!` → EXPORT.** Define an `error_response!` macro in `framework` and
  export it from the `ferro` facade. Rationale: every generated CRUD handler calls
  `ferro::error_response!(status, "msg")` (see `ferro-cli/src/templates/scaffold.rs`, ~14
  call sites); no equivalent helper exists anywhere in `framework/src/`. It is genuine public
  ergonomic API, not a template bug. Research must confirm the exact return type the templates
  rely on (used in both `.map_err(|_| …)` and `.ok_or_else(|| …)` positions → must produce the
  controller's error type / `HttpResponse`).
- **D-02 — `ActiveValue` (and `Set`) → EXPORT (facade re-export).** Add `ActiveValue` to the
  `pub use sea_orm::{…}` block at `framework/src/lib.rs:122` and point the scaffold controller
  template at `ferro::ActiveValue`. Rationale: the facade comment at `lib.rs:78` documents the
  intent — "saves users from having to add `use sea_orm::*` imports". `ActiveValue` is essential
  for ActiveModel mutation and belongs in that re-export set alongside the already-exported
  `ActiveModelTrait`/`IntoActiveModel`.
- **D-03 — `ferro::Queue` / `ferro::QueueConfig` → TEMPLATE.** Change the template to emit
  `ferro::queue::Queue` / `ferro::queue::QueueConfig`. The library already re-exports these under
  the `ferro::queue` module (`framework/src/lib.rs:194-202`, the v12.3 queue-namespacing
  decision). Adding a top-level `ferro::Queue` re-export would duplicate the control surface and
  contradict the shipped namespacing — rejected per the no-duplicate-control-surface principle.
- **D-04 — `#[rule]` → TEMPLATE.** The `rule` attribute is the helper attribute of the
  `ValidateRules` derive (`ferro-macros/src/lib.rs:542`,
  `#[proc_macro_derive(ValidateRules, attributes(rule))]`). The generated request structs use
  `#[rule(...)]` without `#[derive(ValidateRules)]` (or without it in scope). Fix the template to
  emit the derive and bring it into scope.
- **D-05 — `crate::models::users` → TEMPLATE.** Align `make:auth` output with the generated
  model module layout so the `users` model module resolves.
- **D-06 — `ferro::database::connection` → TEMPLATE.** Fix the template call site:
  `database::connection` is a module, not a function; the template uses it as a function.

### Queue exposure in generated projects

- **D-07 — Route generated jobs through `ferro::queue::*`; add NO `ferro-queue` dependency to
  the generated `Cargo.toml`.** `make:job` currently emits `use ferro_queue::{…}` while the
  generated `Cargo.toml` declares no `ferro-queue` dep (COMP-04 W1, verified). Since `ferro`
  already re-exports the full queue surface under `ferro::queue`, the template should import from
  there. This keeps generated projects single-dependency (`ferro` only) and coherent with the
  v12.3 namespacing decision. Touches `ferro-cli/src/templates/make.rs`.

### CI guard cadence and shape

- **D-08 — Two-layer guard.**
  - **Per-PR fast layer** — scaffold + `cargo build` against the **workspace** `ferro` via a
    path dependency. No network, fast feedback, catches all template↔library API drift (the
    export/template bugs above) on every PR. Built on the existing
    `ferro-cli/tests/benchmark_new_project.rs` apparatus.
  - **Release gate (post-publish)** — scaffold + `cargo build` against the **published**
    `ferro-rs` using the committed cold-cache Dockerfile. Catches packaging / `Cargo.toml` drift
    that only manifests after publish (e.g. a missing published dependency). Fails the release
    pipeline on regression.
  - **Rationale (record under SC#5):** the published-artifact check is inherently post-publish —
    you cannot build against a crate version that is not yet on crates.io — so it cannot gate a
    PR. The per-PR path-dep layer closes that chicken-and-egg gap with fast pre-publish feedback;
    the release gate is the guarantee that a non-compiling scaffold never ships silently.

- **D-09 — Mechanism.** Reuse the committed `ferro-cli/tests/fixtures/benchmark/Dockerfile`
  cold-cache job for the release gate (it is already the evidence apparatus and reproduces the
  real first-time cold experience). Reuse/extend `ferro-cli/tests/benchmark_new_project.rs` for
  the per-PR path-dep layer. Do **not** use `cargo install --version <released>` (slow, flaky).

### Requirement labels (derive in REQUIREMENTS.md from 211-WEAKNESSES W1)

- **D-10 — SCAF-* IDs:**
  - **SCAF-01** — Scaffold templates reference only the published `ferro` surface: every symbol a
    generated project emits resolves from the published crate (exports aligned per D-01..D-06).
  - **SCAF-02** — `make:job` produces a project whose `Cargo.toml` declares every crate its
    generated code imports (queue routed through `ferro::queue`, no missing `ferro-queue` dep).
  - **SCAF-03** — A clean scaffold (`ferro new → make:auth → make:scaffold ×3 → make:job`)
    `cargo build`s exit 0 against the published `ferro-rs` (COMP-04's failing assertion now
    passes).
  - **SCAF-04** — A CI smoke test scaffolds + builds against the published artifact and gates
    every release on it.
  - **SCAF-05** — A per-PR scaffold-build guard against the workspace path dep gives fast
    pre-publish drift detection.

### Claude's Discretion
- Exact `error_response!` macro signature and return type (research-determined from the
  template call sites in `scaffold.rs`).
- Whether the parity fix and the CI guard ship as one plan or split into a parity-fix plan + a
  CI-guard plan (planner's call; the ROADMAP anticipates a possible split).
- Exact placement of the per-PR job in `.github/workflows/` and the release gate's wiring into
  the publish pipeline.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements source (the defect)
- `.planning/phases/211-comp-04-time-to-working-app-benchmark/211-WEAKNESSES.md` — Finding W1 is
  the authoritative list of the 52 errors and their categories. SCAF-01..05 derive from it.
- `.planning/ROADMAP.md` § "Phase 214: Scaffold↔Library Parity & Published-Artifact Smoke Test"
  — goal + draft success criteria SC#1-4.

### Smoke-test apparatus (the guard basis)
- `ferro-cli/tests/benchmark_new_project.rs` — existing scaffold+build apparatus; basis for the
  per-PR path-dep layer (D-08, D-09).
- `ferro-cli/tests/fixtures/benchmark/Dockerfile` — committed cold-cache harness against the
  published artifact; basis for the release gate (D-08, D-09).
- `ferro-cli/tests/fixtures/benchmark/RESULTS.md` — benchmark results / methodology.

### Library surface to align to
- `framework/src/lib.rs:122` — `pub use sea_orm::{…}` facade re-export (add `ActiveValue`, D-02);
  `lib.rs:78` documents the facade intent.
- `framework/src/lib.rs:194-202` — `pub mod queue { pub use ferro_queue::{…} }` (the
  `ferro::queue::*` namespace templates must target, D-03/D-07).
- `ferro-macros/src/lib.rs:542` — `ValidateRules` derive + `rule` helper attribute (D-04).

### Templates to fix
- `ferro-cli/src/templates/scaffold.rs` — `error_response!` call sites (D-01), `ActiveValue` use
  (D-02), `ferro::database::connection` call site (D-06).
- `ferro-cli/src/templates/make.rs` — `make:job` `use ferro_queue::{…}` → `ferro::queue::*`
  (D-03/D-07).
- `ferro-cli/src/templates/auth.rs` — `make:auth` `crate::models::users` resolution (D-05).

### Coherence anchor
- v12.3 queue-namespacing decision (`ferro::queue::Job`, `ferro::queue::dispatch`) — referenced
  in `.planning/STATE.md` / `.planning/PROJECT.md`. D-03/D-07 must not reintroduce a top-level
  queue re-export that contradicts it.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Committed benchmark apparatus** (`ferro-cli/tests/benchmark_new_project.rs` +
  `ferro-cli/tests/fixtures/benchmark/{Dockerfile,RESULTS.md}`) — already asserts the scaffold
  sequence + `cargo build` exit 0. Both CI layers (D-08) extend this rather than build new.
- **`ferro` facade re-export pattern** — `framework/src/lib.rs` already re-exports sea_orm
  traits and the queue surface; D-01/D-02 extend the same pattern.

### Established Patterns
- **Facade-first imports** (`lib.rs:78`): generated code should import from `ferro::*`, not from
  transitive crates directly. This is the through-line for D-02 (`ferro::ActiveValue`),
  D-03/D-07 (`ferro::queue::*`).
- **Namespaced subsystems** (v12.3): the queue lives at `ferro::queue::*`, not top-level. Respect
  the namespace; do not flatten.

### Integration Points
- `framework/src/lib.rs` — facade re-exports (D-01 macro export, D-02 `ActiveValue`).
- `framework/src/` (new macro module) — `error_response!` definition (D-01).
- `ferro-cli/src/templates/{scaffold,make,auth}.rs` — template fixes (D-03..D-07).
- `.github/workflows/` — per-PR job + release gate (D-08/D-09); release wiring must respect the
  publish-wave ordering noted in project memory.

</code_context>

<specifics>
## Specific Ideas

- The acceptance gate is concrete and pre-written by COMP-04:
  `ferro new → make:auth → make:scaffold ×3 → make:job → cargo build` must exit 0 against the
  **published** `ferro-rs`. Plans should assert exactly this sequence.
- The published `Cargo.toml` pins `ferro = { package = "ferro-rs", version = "0.2" }` (crates.io,
  not a path dep), so the local workspace binary reproduces the failure — the published library,
  not the scaffolding binary, is the constraint. The release gate must build against the
  published crate to be meaningful.

### The 52 errors, by root cause (from 211-WEAKNESSES W1)

| Generated code references | Problem | Locked resolution |
|---------------------------|---------|-------------------|
| `ferro::error_response!(…)` (every API controller) | macro not exported by published `ferro` | D-01 EXPORT (define + facade-export) |
| `#[rule]` on request structs | `ValidateRules` derive/attr not in scope | D-04 TEMPLATE (emit derive) |
| `ferro::Queue`, `ferro::QueueConfig` | live at `ferro::queue::*`, not top-level | D-03 TEMPLATE (`ferro::queue::*`) |
| `use ferro_queue::{…}` in `make:job` | `ferro-queue` absent from generated `Cargo.toml` | D-07 TEMPLATE (route via `ferro::queue`) |
| `ActiveValue::Set(…)` in controllers | `ActiveValue` not re-exported / imported | D-02 EXPORT (facade re-export) |
| `crate::models::users` (make:auth) | module layout mismatch | D-05 TEMPLATE (align layout) |
| `ferro::database::connection` as fn | it is a module | D-06 TEMPLATE (fix call site) |

</specifics>

<deferred>
## Deferred Ideas

- **COMP-04 W3 — `make:scaffold` flag ordering** (the greedy `[FIELDS]...` positional swallows
  trailing flags). A clap ergonomics fix; out of scope here unless trivially co-located with a
  template touch. Candidate for a future CLI-ergonomics phase.
- **COMP-04 W4 — `make:model` vs `make:scaffold` naming drift** between spec/docs and the shipped
  CLI. Documentation alignment, not parity; track separately.
- **COMP-04 W2 — `libssl-dev`/`pkg-config` CLI install prereq** is already fixed in the committed
  Dockerfile; the deeper option (move the CLI off `native-tls` to rustls to drop the OpenSSL
  build-time dependency) is a separate hardening idea, not this phase.

</deferred>

---

*Phase: 214-scaffold-library-parity*
*Context gathered: 2026-06-13*
