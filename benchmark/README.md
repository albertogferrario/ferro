# Ferro Framework Benchmark

A reproducible comparison of Ferro against mature batteries-included frameworks.
Design: `docs/superpowers/specs/2026-06-15-ferro-framework-benchmark-design.md`.

## What 1A measures
Four micro-endpoints (`/json`, `/db`, `/queries`, `/updates`) in Ferro and Laravel, on two
axes: raw performance (requests/sec, p50/p99 latency, memory) and static compression (LoC,
files, source tokens).

## Reproducibility
- Apps and tooling run in pinned containers (`compose.yaml`, `harness/Dockerfile.toolbox`).
- PostgreSQL 16.4, shared by both apps.
- Perf tool: `oha` (pinned in the toolbox). Static tool: `tokei` (pinned).
- Canonical perf numbers come from a fixed local machine; each results run records the
  hardware. CI runs conformance + a perf smoke only.

## Honesty note
Internal results (`results/<date>/internal.md`) report every number, including where Ferro is
slower or larger. The public table is a subset, never a contradiction, of the internal data.
Rust outperforming interpreted languages on raw throughput is expected and is reported as
such, not as a finding.

## Run it
See "Running the benchmark" below (added in Task 9).
