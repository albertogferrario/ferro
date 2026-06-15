# Ferro Framework Benchmark — Design

**Date:** 2026-06-15
**Status:** Approved design — Phase 1 ready for implementation planning
**Scope of this doc:** the full benchmark design, with Phase 1 specified for build.

## Purpose

A reproducible benchmark that compares Ferro against mature batteries-included web
frameworks (Laravel, Rails, Django) along the axes that matter for how Ferro is built and
used, not only raw throughput. The same harness serves three audiences from one source of
truth:

- **Internal validation** — measure Ferro's authoring-efficiency and performance against a
  mature baseline, including the cases where Ferro is weaker.
- **Public comparison** — a fair, re-runnable artifact derived from the internal results.
  The public view is always a subset of the internal data, never a contradiction of it.
- **Performance scorecard** — standard throughput/latency numbers.

## Design principles

1. **One harness, three views.** Internal results are the source of truth; the public
   artifact and the performance table are projections of it.
2. **Do not author the competition.** Competitor implementations come from the community
   RealWorld backends (Laravel/Rails/Django), not hand-written by this project. Only the
   Ferro implementation is authored here. This removes implementation bias against the
   competitors.
3. **Report losses.** Where Ferro is weaker (ecosystem maturity, language familiarity, some
   developer-experience axes), the internal results say so explicitly.
4. **Reproducibility is a requirement, not a nice-to-have.** Pinned versions, containerized
   apps, a fixed hardware spec, and committed raw data. Any third party can re-run it.
5. **No foregone conclusions framed as insights.** Rust outperforms interpreted languages on
   raw throughput; that is reported plainly as expected, not as a finding.

## Workloads

Three workloads exercise different axes:

1. **Conduit (RealWorld spec)** — the credibility spine. A non-trivial application with a
   fixed, documented API contract: JWT auth, article CRUD, comments, follows, tag feeds, and
   pagination. The shared contract makes implementations directly comparable. Community
   reference backends exist for Laravel, Rails, and Django.
2. **Micro-endpoints (TechEmpower categories)** — JSON serialization, single-query,
   multiple-queries, and updates. Drives the raw-performance numbers.
3. **Authoring task** — "add a specified feature end-to-end," given to an AI coding agent
   once per framework, with each framework's own introspection tooling available (Ferro's
   `ferro-mcp` and Laravel's `laravel/mcp`). Measures authoring efficiency directly rather
   than by proxy.

## Measurement axes

| Axis | Metrics | Notes |
|------|---------|-------|
| Compression (static) | Lines of code, file count, distinct framework concepts touched, source tokens per Conduit feature | Objective and diffable; computed from the committed implementations |
| Authoring efficiency | Tokens, turns, tool-calls, wall-clock, and success rate to build a specified feature; multiple trials per framework | Multi-trial to account for LLM run-to-run variance; success = produced a working feature that passes the contract tests |
| Raw performance | Requests/sec, p50/p99 latency, peak memory, cold-start time | Containerized, identical hardware, load generated with `oha` and/or `k6` |

### Compression sub-measurement: projection slice

Conduit is hand-written CRUD on every framework, which under-represents Ferro's
projection/intent surface. To capture that, Phase 1 adds a small **projection-driven slice**
— a single resource exposed end-to-end (list / detail / create / edit / summary) — measured
on the static compression axis only. This is where Ferro's input-to-output ratio is expected
to differ most from imperative CRUD.

## Reproducibility

- Each app runs in a container with pinned framework and language-runtime versions.
- A documented hardware/runner spec; all runs on the same spec.
- Load tests: fixed concurrency levels, warm-up period, and duration; N repetitions; report
  median and spread, not single runs.
- Raw output (latency histograms, agent transcripts/token counts, LoC reports) committed to
  the benchmark directory so results are auditable and re-derivable.

## Architecture

```
benchmark/
  apps/
    ferro-conduit/            # authored here
    laravel-conduit/          # vendored community RealWorld backend (pinned)
    rails-conduit/            # Phase 2
    django-conduit/           # Phase 2
    micro-endpoints/          # per-framework minimal apps for TechEmpower categories
    projection-slice/         # Ferro vs Laravel, static-only
  harness/
    perf/                     # load-test runner + result parser (oha/k6)
    static/                   # LoC / file / concept / token counters
    authoring/                # agent-task runner + transcript/token capture (Phase 3)
  contracts/
    conduit-openapi.yaml      # shared API contract + conformance tests
  results/
    <date>/                   # committed raw outputs + derived tables
  README.md                   # how to run each axis, hardware spec, methodology
```

Each unit has one purpose and a defined interface: the harness runners consume containerized
apps and the shared contract, and emit raw result files; reporting is a separate step that
reads `results/` and produces the public and internal tables. Implementations and measurement
are decoupled so a new framework can be added without touching the runners.

## Phasing

- **Phase 1 (this spec → plan → build):** Ferro Conduit + the harness (`perf` + `static`) +
  the projection slice, compared against the vendored **Laravel** Conduit only. Delivers a
  complete, reproducible internal/public/performance result for one matchup.
- **Phase 2:** add Rails and Django (static + performance breadth). No harness rework.
- **Phase 3:** the authoring-efficiency experiment (Ferro + `ferro-mcp` vs Laravel +
  `laravel/mcp`), controlled multi-trial, with committed transcripts and token accounting.

## Phase 1 deliverables

1. `benchmark/apps/ferro-conduit/` — a Ferro implementation conforming to the Conduit API
   contract, passing the contract conformance tests.
2. `benchmark/apps/laravel-conduit/` — a pinned, vendored community RealWorld Laravel backend.
3. `benchmark/apps/micro-endpoints/` — minimal Ferro and Laravel apps for the JSON,
   single-query, multi-query, and update categories.
4. `benchmark/apps/projection-slice/` — one resource, end-to-end, in Ferro and Laravel
   (static-comparison only).
5. `benchmark/harness/perf/` and `benchmark/harness/static/` — runners + parsers.
6. `benchmark/contracts/` — the shared contract and conformance tests.
7. `benchmark/results/<date>/` — committed raw outputs and the derived internal/public tables.
8. `benchmark/README.md` — methodology, hardware spec, and run instructions.

## Non-goals

- Not a claim of general superiority; the benchmark measures specified workloads under
  specified conditions.
- Phase 1 does not run the authoring-efficiency experiment (Phase 3).
- No micro-optimization of any implementation to win a category; each app uses idiomatic,
  documented patterns for its framework.

## Open questions for Phase 1 planning

- Database choice for the comparison (SQLite vs Postgres) and whether to run both.
- Which community Laravel RealWorld backend to vendor, and its pinned commit.
- The exact feature used for the projection slice.
- The hardware/runner spec (local fixed machine vs a pinned CI runner class).
