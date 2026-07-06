# Phase 229: Framework Benchmark — Harness Foundation (1A) - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning
**Source:** PRD Express Path (`docs/superpowers/plans/2026-06-15-benchmark-1a-harness-foundation.md`)

<domain>
## Phase Boundary

Build the reproducible benchmark harness and prove it end-to-end on the cheapest workload
(four micro-endpoints). Deliverable lives entirely under a new top-level `benchmark/` tree.

**In scope (1A):**
- The shared micro-endpoints contract (`/json`, `/db`, `/queries`, `/updates`).
- A pinned containerized toolbox (`oha` + `tokei` + python).
- Two harness axes: static compression (LoC/files/source-tokens via tokei) and raw perf
  (rps/p50/p99/success via oha), each as tested Python units.
- A reporting step turning raw result JSON into internal + public markdown tables.
- A minimal **Ferro** micro-endpoints app and a minimal **Laravel 11** micro-endpoints app,
  each containerized, both hitting one shared pinned Postgres.
- Contract conformance tests + compose orchestration.
- The first committed results run (`benchmark/results/2026-06-15/`) with hardware metadata.

**Out of scope (deferred):**
- Ferro Conduit (full RealWorld backend) → Phase 1B.
- Projection-driven static slice → Phase 1C.
- Rails/Django competitors → Phase 2.
- Agent-authoring efficiency experiment → Phase 3.
- Octane/Swoole-tuned Laravel (documented as a future variant; 1A uses stock `artisan serve`).
</domain>

<decisions>
## Implementation Decisions (locked — from the approved PRD)

### Resolved defaults
- **D-01 Database:** PostgreSQL `16.4`, one pinned container shared by both apps. (TechEmpower
  standard; fair for concurrency — SQLite write-locking would distort load tests.)
- **D-02 Laravel source:** for the trivial micro-endpoints, a minimal idiomatic Laravel 11 /
  PHP 8.3 app is authored (the "don't author the competitor" rule binds the realistic Conduit
  app in 1B, where `gothinkster/laravel-realworld-example-app` is vendored at a pinned commit).
- **D-03 Projection-slice resource:** `articles` — deferred to 1C, not built here.
- **D-04 Hardware/runner:** canonical perf numbers on a documented fixed local machine; each
  results run records `{cpu_model, physical_cores, ram_gb, os, kernel}`. CI (`ubuntu-latest`)
  runs conformance + a perf smoke only (shared runners too noisy for headline numbers). Tool
  versions pinned in the toolbox image → machine-independent.

### Tooling (pinned)
- **D-05:** `oha 1.4.7` (load gen, `--json`), `tokei 12.1.2` (static counts), built into
  `benchmark/harness/Dockerfile.toolbox` from `rust:1.88.0-slim-bookworm`; runtime image
  `debian:bookworm-slim` + `python3`, `jq`, `curl`.
- **D-06:** Ferro app built `--release` from `rust:1.88.0-slim-bookworm`; Laravel app from
  `php:8.3-cli-bookworm` with `pdo_pgsql` + opcache, `composer install --no-dev`.

### Harness contracts (interfaces other tasks depend on — keep stable)
- **D-07:** `parse_perf.parse_oha(raw)` → `{rps, p50_ms, p90_ms, p99_ms, success_rate}`;
  raises `ValueError` when `success_rate < 0.99`.
- **D-08:** `count_static.run(app_dir)` → `{code_lines, files, source_tokens}` (tokei excludes
  `vendor`/`target`).
- **D-09:** result filenames `perf-<framework>.json` and `static-<framework>.json`; the report
  builder (`build_tables.render_markdown`) consumes exactly those keys/filenames.

### Honesty + reproducibility guardrails (required, not optional)
- **D-10:** internal results report every number including where Ferro is slower/larger; the
  public table is a strict subset, never a contradiction. Rust-vs-interpreted throughput is
  labelled "expected, not a finding."
- **D-11:** pinned versions, containerized apps, committed raw JSON + derived tables + recorded
  hardware. A third party can re-run from `benchmark/README.md` alone.

### Claude's Discretion
- Exact wording of README/table prose; precise oha concurrency/duration (PRD defaults: 30s,
  c=256, 5s warmup) may be tuned if the target app saturates differently — record whatever is used.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### The approved design + task plan (authoritative)
- `docs/superpowers/specs/2026-06-15-ferro-framework-benchmark-design.md` — the approved
  benchmark design (purpose, axes, phasing, honesty guardrails).
- `docs/superpowers/plans/2026-06-15-benchmark-1a-harness-foundation.md` — the detailed 10-task
  1A plan: file structure, pinned tool versions, concrete TDD code for parse/count/report, the
  Dockerfiles, compose, conformance, and the results-run commands. **This is the implementation
  spine — GSD plans should mirror its task breakdown and acceptance gates, not re-derive them.**

### Ground-truth oracle for the Ferro micro-app (the one real unknown)
- `ferro-cli/src/commands/` — real CLI subcommands (`new`, `serve`, `db:migrate`, `db:seed`);
  the app is scaffolded with `target/debug/ferro new` then reduced to the four endpoints.
- `ferro-mcp` introspection (`list_routes`, `get_handler`, `code_templates`) + `docs/src/` —
  verify the **current** handler/routing/DB-extractor API before relying on the PRD's
  illustrative Rust handler code (Task 7). The conformance test is the real gate.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `target/debug/ferro` exists — scaffold the Ferro micro-app with it.
- Scaffold patterns: `ferro-cli/src/templates/files/backend/` shows the generated app shape
  (rustls/SQLite default) — the micro-app overrides DB to the shared Postgres via env.

### Established Patterns
- The repo is a cargo workspace; `benchmark/apps/ferro-micro` should be a standalone app built
  in its own Docker context (not added as a workspace member, to keep the benchmark isolated).

### Integration Points
- New top-level `benchmark/` dir — no changes to existing crates. Must NOT regress the
  workspace build (keep `benchmark/apps/*/Cargo.toml` out of the root workspace members, or the
  PHP/scratch apps will confuse `cargo`).
</code_context>

<specifics>
## Specific Ideas
- De-risk ordering: build the harness TDD units (perf parser, static counter, report builder)
  and the contract FIRST (pure Python, light), then the toolbox image, then the two apps, then
  the first results run. Heavy/thermal steps (toolbox compile of oha+tokei, Ferro `--release`
  build, Laravel composer install, load tests) come last and should be paced — never run a
  cargo release build and a load test concurrently.
</specifics>

<deferred>
## Deferred Ideas
- Conduit (1B), projection slice (1C), Rails/Django (Phase 2), agent-authoring (Phase 3),
  Octane-tuned Laravel variant. None are built in 229.

### Reviewed Todos (not folded)
None.
</deferred>

---

*Phase: 229-framework-benchmark-harness-foundation-1a*
*Context gathered: 2026-06-15 via PRD Express Path*
