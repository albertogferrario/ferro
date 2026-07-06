---
phase: 229-framework-benchmark-harness-foundation-1a-build-the-reproduc
plan: "01"
subsystem: benchmark
tags: [benchmark, scaffold, contract, reproducibility]
dependency_graph:
  requires: []
  provides:
    - benchmark/ directory tree (harness, contracts, apps, results)
    - benchmark/.gitignore (build artifacts excluded, results/ tracked)
    - benchmark/contracts/micro-endpoints.md (authoritative 4-endpoint contract)
  affects:
    - Plans 02-05 (harness units, apps, conformance) build against this tree and contract
tech_stack:
  added: []
  patterns:
    - benchmark/ tree isolated from cargo workspace (no workspace members added)
    - results/ committed for auditable raw data (gitignore explicitly keeps it)
    - contract-first: single markdown file is source of truth for both apps and conformance tests
key_files:
  created:
    - benchmark/README.md
    - benchmark/.gitignore
    - benchmark/results/.gitkeep
    - benchmark/contracts/micro-endpoints.md
  modified: []
decisions:
  - ".gitignore excludes apps/*/target/, vendor/, node_modules/, .env, __pycache__, *.pyc — but NOT results/ (intentionally committed)"
  - "Contract camelCase key is randomNumber in JSON responses matching the database column name — noted explicitly for Ferro's random_number → randomNumber serde rename"
  - "benchmark/ tree stays outside cargo workspace members per CONTEXT.md isolation constraint"
metrics:
  duration_seconds: 72
  completed_date: "2026-06-15"
  tasks_completed: 2
  tasks_total: 2
  files_created: 4
  files_modified: 0
---

# Phase 229 Plan 01: Benchmark Scaffold Tree and Contract Summary

**One-liner:** Benchmark directory skeleton with gitignore, honesty-first README, and authoritative four-endpoint JSON contract (clamp semantics, world schema) that both apps and conformance tests derive from.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Scaffold the benchmark tree (README, gitignore, results placeholder) | 82d81043 | benchmark/README.md, benchmark/.gitignore, benchmark/results/.gitkeep + 7 directories |
| 2 | Write the shared micro-endpoints contract | c06fe0ff | benchmark/contracts/micro-endpoints.md |

## What Was Built

**Task 1 — Directory skeleton and methodology files**

Created the complete `benchmark/` directory tree per the PRD file structure:
- `benchmark/harness/{perf,static,report}/` — homes for the three harness modules (Plans 02-03)
- `benchmark/contracts/conformance/` — home for the conformance test (Plan 05)
- `benchmark/apps/` — home for ferro-micro and laravel-micro (Plans 04, 07)
- `benchmark/results/` with `.gitkeep` so the directory is tracked before the first run

`benchmark/.gitignore` excludes all app build artifacts (`apps/*/target/`, `apps/*/vendor/`, `apps/*/node_modules/`, `apps/*/.env`, `**/__pycache__/`, `*.pyc`) and explicitly does not exclude `results/` — the comment makes the intent auditable.

`benchmark/README.md` contains the methodology skeleton with two required sections per CONTEXT.md decisions D-10 and D-11:
- **Reproducibility**: pinned containers/versions, canonical machine + hardware recording, CI conformance-only note
- **Honesty note**: internal results include every number (including where Ferro is slower/larger); public table is a strict subset; Rust-vs-interpreted throughput labeled "expected, not a finding"

**Task 2 — Shared contract**

`benchmark/contracts/micro-endpoints.md` defines all four endpoints:
- `GET /json` → `{"message":"Hello, World!"}`
- `GET /db` → one random row from `world` as `{"id","randomNumber"}`
- `GET /queries?n=K` → K random-row lookups, n clamped to [1,500]
- `GET /updates?n=K` → K read-then-update cycles, n clamped to [1,500]
- `world` schema: `id SERIAL PRIMARY KEY, randomNumber INT NOT NULL`, 10000 seed rows

The contract explicitly names `randomNumber` (camelCase) in JSON, flagging that Ferro's Rust model will use `random_number` with a serde rename — important for Plan 04 (app) and Plan 05 (conformance).

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. This plan creates no code, only structural files. The "Run it" section in README.md is explicitly placeholder, deferred to Task 9 per the PRD task plan.

## Threat Flags

Per the plan's threat model: this plan creates no network services, no secrets, no untrusted input. The `.env` exclusion in `.gitignore` prevents app secrets from landing in git. No new threat surface introduced.

## Self-Check: PASSED

- FOUND: benchmark/README.md
- FOUND: benchmark/.gitignore
- FOUND: benchmark/results/.gitkeep
- FOUND: benchmark/contracts/micro-endpoints.md
- FOUND commit: 82d81043 (Task 1 — scaffold tree)
- FOUND commit: c06fe0ff (Task 2 — contract)
