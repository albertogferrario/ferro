# Phase 225: Release Workflow rustls Migration and E2E CLI-from-Release Test - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-14
**Phase:** 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t
**Mode:** `--auto` (recommended defaults auto-selected)
**Areas discussed:** TLS backend scope, rustls crypto provider, aarch64-linux cross tooling, E2E placement/trigger, E2E test depth, relation to existing scaffold-smoke, sequencing risk

---

## TLS Backend Migration Scope

| Option | Description | Selected |
|--------|-------------|----------|
| ferro-cli transitive tree only | Migrate just what the release binary pulls | |
| ferro-cli tree + workspace-wide sea-orm/lettre swap | One TLS backend everywhere; no drift | ✓ |
| Defer / native-tls stays | No migration | |

**Auto-selected:** Workspace-wide. **Rationale:** single source of truth (framework coherence principle); split backends still drag openssl-sys into `--all-features` CI and create drift.

---

## rustls Crypto Provider

| Option | Description | Selected |
|--------|-------------|----------|
| `ring` | Pure-ish, no C compiler/cmake; portable cross-compile | ✓ |
| `aws-lc-rs` | Faster, but needs C compiler + cmake | |

**Auto-selected:** `ring`. **Rationale:** aws-lc-rs reintroduces the exact native-build-tooling barrier this migration removes; CLAUDE.md forbids external build tooling.

---

## aarch64-unknown-linux-gnu Cross Tooling

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `cross` (Docker) | Status quo | |
| Drop `cross`, native cross-linker | `rustup target add` + `gcc-aarch64-linux-gnu` | ✓ |

**Auto-selected:** Drop `cross`. **Rationale:** rustls removes the only C-cross barrier (openssl); native cross-link is faster and Docker-free. Fallback to per-target `cross` only if a residual C dep is found.

---

## E2E CLI-from-Release: What Gets Tested

| Option | Description | Selected |
|--------|-------------|----------|
| `cargo run -p ferro-cli` against path-deps | What `scaffold-smoke` already does | |
| Actual released binary + published library | Tests the real shipped artifact | ✓ |

**Auto-selected:** Released binary + published library. **Rationale:** this is the blind spot COMP-04 exposed (published 0.2.55 scaffold = 52 errors); the killer feature of the phase.

---

## E2E Trigger / Placement

| Option | Description | Selected |
|--------|-------------|----------|
| `#[ignore]` manual test | Like the existing benchmark | |
| `release.yml` job only | Tag-time only | |
| `release.yml` job + `workflow_dispatch` + schedule | Tag-time + continuous drift detection | ✓ |

**Auto-selected:** Job + dispatch + schedule. **Rationale:** generated app pins crates.io ferro, so library drift is continuous — catch it between releases too.

---

## E2E Test Surface Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal (`new` → `build`) | Fast, shallow | |
| Full COMP-04 sequence | `new`→auth→scaffold×N→job→build | ✓ |

**Auto-selected:** Full COMP-04 sequence, reusing `benchmark_new_project.rs` apparatus. **Rationale:** that exact surface produced the 52 errors; non-Docker in CI for speed.

---

## Relation to Existing `scaffold-smoke`

| Option | Description | Selected |
|--------|-------------|----------|
| Replace scaffold-smoke | Single gate | |
| Complement (two layers) | Workspace-smoke pre-merge + from-release post-tag | ✓ |

**Auto-selected:** Complement. **Rationale:** different failure classes — path-dep drift pre-merge vs published-artifact reality post-tag.

---

## Sequencing Risk (flagged to planner)

A from-release e2e built against the published library goes RED if published `ferro-rs` still
carries the COMP-04 drift. Auto-decision: **flag, do not resolve here** — the rustls half is
independent and lands regardless; the planner picks "verify published scaffold first" vs
"land e2e as `continue-on-error` until the alignment phase". Recorded as D-10.

## Claude's Discretion

- aarch64 cross-linker mechanism (`.cargo/config.toml` vs workflow env)
- optional `x86_64-unknown-linux-musl` static target
- scheduled-run cron cadence
- generated-app build profile (`--release` vs debug) in the e2e

## Deferred Ideas

- Scaffold-template ↔ published-library API alignment (the 52-error fix) — separate phase
- `ferro-wallet` openssl → rustls/ring — coherence follow-up
- musl static release target
