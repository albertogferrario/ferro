# Phase 211: COMP-04 — Time-to-Working-App Benchmark - Context

**Gathered:** 2026-06-13 (--auto; recommended defaults selected and logged)
**Status:** Ready for planning

<domain>
## Phase Boundary

A benchmark at `ferro-cli/tests/benchmark_new_project.rs` (criterion 0.8.2
`iter_custom`, `FERRO_BENCH=1`-gated) that measures the time from `ferro new` to a
running service with auth, 3 entity types, and 1 background job — recording each
step's wall-clock individually — plus at least one committed **cold-cache** number
from a clean Docker container, and a committed Markdown result doc with a full
environment specification.

**Locked by COMP-04 success criteria (not open for discussion):**
- File path `ferro-cli/tests/benchmark_new_project.rs`; criterion 0.8.2 `iter_custom`
  with `default-features = false, features = ["cargo_bench_support"]`.
- `FERRO_BENCH=1` gate (skipped in default CI; no second target dir on CI disk).
- Five measured steps: `ferro new <tmpdir>` → `ferro make:auth` →
  `ferro make:model <X>` ×3 → `ferro make:job <Y>` → `cargo build` in tmpdir; each
  step timed individually; the build step asserts exit code 0.
- ≥1 cold-cache Docker run committed (no pre-installed toolchain, no Cargo cache);
  the cold number is the one any external doc reports.
- Committed Markdown result doc with: Rust toolchain version, cache state, host
  machine class, agent-assistance level, per-step breakdown, total time.
- A non-empty "discovered weaknesses" finding (empty fails the phase close).

This phase builds the apparatus and produces the first committed result. It does
NOT set a CI wall-clock threshold (deferred to after the first cold run) and does
NOT modify the CLI commands it measures.
</domain>

<decisions>
## Implementation Decisions

### Execution model
- **D-01:** Hybrid, mirroring COMP-03/COMP-04's gate pattern. The criterion
  `iter_custom` benchmark runs **warm/local**, gated behind `FERRO_BENCH=1` — it
  produces the per-step wall-clock breakdown. The **cold-cache** number comes from
  a separate Docker run (clean container, no toolchain, no cache). Default
  `cargo test`/CI never runs either (no second target dir, no Docker), preserving
  the always-green, bounded-disk invariant.
- **D-02:** The cold-cache Docker run is a **human-action** step (`autonomous:
  false`): the autonomous executor has no Docker daemon and the run is disk/CPU/
  time-heavy. The phase commits a `Dockerfile` (clean rust base, no cache priming)
  + a run command; the developer executes it and commits the produced number into
  the result doc. The autonomous executor builds and verifies everything else
  (the benchmark scaffold, the warm path wiring, the result-doc template).

### Benchmark composition
- **D-03:** The five steps invoke the `ferro` CLI binary (bin name `ferro`,
  package `ferro-cli`) against a fresh `tempfile::tempdir()`: `new`, `make:auth`,
  `make:model` (×3 distinct entities), `make:job`, then `cargo build` in the
  generated project. Each step's wall-clock is recorded separately; the `cargo
  build` step asserts exit code 0 (SC#2). The exact `make:model` subcommand name
  is a RESEARCH item (commands present: `new.rs`, `make_auth.rs`, `make_job.rs`;
  also `make_scaffold.rs`, `make_projection.rs` — confirm the model generator).
- **D-04:** criterion 0.8.2 added to `ferro-cli` `[dev-dependencies]` with
  `default-features = false, features = ["cargo_bench_support"]` (SC#1) — no
  always-on dependency on the default build, no external build tooling.

### Result document
- **D-05:** A committed Markdown result doc carries the SC#4 environment-spec table
  (Rust toolchain version, cache state cold/warm, host machine class,
  agent-assistance level, per-step breakdown, total). Recommended location:
  `ferro-cli/tests/fixtures/benchmark/RESULTS.md` (alongside the benchmark). A
  result without the env spec is not accepted.
- **D-06:** Agent-assistance level for the committed baseline = **manual commands**
  (deterministic, reproducible). Agent-driven scaffolding is a future variant, not
  this baseline.

### Calibration (deferred, not this phase)
- **D-07:** No CI wall-clock threshold is asserted now (ROADMAP calibration note —
  decided after the first cold-cache run). The benchmark asserts only the build
  step's exit code. If CI disk makes the gate infeasible, the benchmark stays a
  committed manual artifact.

### Claude's Discretion
- Exact criterion `iter_custom` structure; the 3 entity names + 1 job name for the
  measured app; the Dockerfile base image tag; RESULTS.md table layout; how per-step
  timings are surfaced (criterion measurement vs explicit `Instant` capture inside
  `iter_custom`).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirement + roadmap
- `.planning/REQUIREMENTS.md` §COMP-04 — binding requirement (cold-cache run,
  committed apparatus doc, `FERRO_BENCH=1` gate).
- `.planning/ROADMAP.md` §"Phase 211" — the 5 success criteria + the calibration
  note (CI threshold deferred).

### CLI commands under measurement (confirm exact subcommand names)
- `ferro-cli/src/commands/new.rs` — `ferro new` project scaffold.
- `ferro-cli/src/commands/make_auth.rs` — `ferro make:auth`.
- `ferro-cli/src/commands/make_job.rs` — `ferro make:job`.
- `ferro-cli/src/commands/make_scaffold.rs`, `make_projection.rs` — candidate model
  generators; confirm which produces `ferro make:model <X>` (entity + migration).
- `ferro-cli/Cargo.toml` — bin name `ferro`; add criterion dev-dep here.

### Gate pattern reference
- `.planning/phases/210-comp-03-agent-success-rate-harness/210-CONTEXT.md` D-01 +
  `ferro-mcp/tests/agent_harness.rs` — the `FERRO_AGENT_EVAL=1` + `#[ignore]` gate
  idiom to mirror as `FERRO_BENCH=1`.

### Project constraints
- `./CLAUDE.md` — no external build tooling; always-green no-network `cargo test`;
  disk-full risk on heavy builds (check `df`, clean `target/` before the gated run).
- `.planning/phases/120-cli-and-mcp-updates/` — prior CLI command work (existing
  `ferro-cli/tests/` patterns: `docker_init_dry_run.rs`, `gestiscilo_fixture.rs`).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`ferro-cli/tests/`** is established (not greenfield) — reuse its test harness
  conventions (`docker_init_dry_run.rs`, `gestiscilo_fixture.rs`, `fixtures/`).
- **`ferro new` / `make:auth` / `make:job`** commands exist; the benchmark drives
  the real CLI, not stubs.
- **COMP-03 gate idiom** (`FERRO_AGENT_EVAL=1` + `#[ignore]`) — copy the shape for
  `FERRO_BENCH=1`.

### Established Patterns
- Gated heavy test behind an env-var early-return + `#[ignore]`, kept out of CI.
- `tempfile::tempdir()` for isolated scaffolding (also used by the COMP-03 harness).

### Integration Points
- New `ferro-cli/tests/benchmark_new_project.rs`; new `ferro-cli` dev-dep
  (criterion 0.8.2); committed `RESULTS.md` + `Dockerfile` under the test/fixtures
  or phase dir.
</code_context>

<specifics>
## Specific Ideas

- The cold-cache Docker number is the honest "first-time experience" figure and the
  only one external docs should quote (SC#3). Warm/local is for the per-step
  breakdown, not the headline.
- Disk is tight on this host (~12 GB free); the gated benchmark builds a full app
  in a tmpdir (~GBs) and the Docker run more — `df` + clean `target/` before
  running, per the disk-full gotcha.
</specifics>

<deferred>
## Deferred Ideas

- **CI wall-clock threshold** — set after the first cold-cache run (ROADMAP
  calibration note); a follow-up, not this phase.
- **Agent-driven scaffolding variant** of the benchmark (vs manual commands) —
  future hardening; this phase establishes the manual-command baseline.
- **Asserting the gate in CI** — only if disk budget allows; otherwise the
  benchmark remains a committed manual artifact (D-07).

### Reviewed Todos (not folded)
None — no pending todos matched Phase 211.
</deferred>

---

*Phase: 211-comp-04-time-to-working-app-benchmark*
*Context gathered: 2026-06-13 (--auto)*
