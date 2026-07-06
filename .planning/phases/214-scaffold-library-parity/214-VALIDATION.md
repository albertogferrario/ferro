---
phase: 214
slug: scaffold-library-parity
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-13
---

# Phase 214 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + `tempfile` (already a dev-dep in ferro-cli) |
| **Config file** | `ferro-cli/tests/benchmark_new_project.rs` (extend existing file) |
| **Quick run command** | `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro -- --nocapture` |
| **Full suite command** | `docker build --build-arg FERRO_VERSION=<latest> -t ferro-scaffold-smoke ferro-cli/tests/fixtures/benchmark/ && docker run --rm ferro-scaffold-smoke` |
| **Estimated runtime** | ~120 s (workspace path-dep build, warm cache); ~10 min (Docker cold-cache) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro -- --nocapture`
- **After every plan wave:** Full scaffold sequence + the per-PR CI job must be green
- **Before `/gsd-verify-work`:** The Docker cold-cache run exits 0 against the latest published `ferro-rs`
- **Max feedback latency:** ~120 seconds (per-PR path-dep layer)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 214-01-xx | 01 | 1 | SCAF-01 | — / — | N/A | integration (build) | `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro` | ❌ W0 | ⬜ pending |
| 214-01-xx | 01 | 1 | SCAF-02 | — / — | N/A | integration (build) | `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro` | ❌ W0 | ⬜ pending |
| 214-02-xx | 02 | 2 | SCAF-03 | — / — | N/A | integration | `docker run --rm ferro-scaffold-smoke` | ❌ W0 | ⬜ pending |
| 214-02-xx | 02 | 2 | SCAF-04 | — / — | N/A | CI job | publish.yml `post-publish-scaffold-smoke` | ❌ W0 | ⬜ pending |
| 214-02-xx | 02 | 2 | SCAF-05 | — / — | N/A | CI job | ci.yml `scaffold-smoke` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*Task IDs are placeholders — the planner assigns final IDs; map each SCAF-* to the task that satisfies it.*

---

## Wave 0 Requirements

- [ ] `ferro-cli/tests/benchmark_new_project.rs` — add `scaffold_builds_against_workspace_ferro` (non-ignored; appends `[patch.crates-io]` to point `ferro-rs` at the workspace path)
- [ ] `ferro-cli/tests/fixtures/benchmark/Dockerfile` — parameterize `FERRO_VERSION` via `ARG`
- [ ] `.github/workflows/ci.yml` — add `scaffold-smoke` job (per-PR, path-dep, no network)
- [ ] `.github/workflows/publish.yml` — add `post-publish-scaffold-smoke` job (release gate, published artifact)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Release gate fails the pipeline on a genuinely broken published scaffold | SCAF-04 | Requires an actual published-artifact regression to fire; cannot be triggered pre-publish in a unit test | After the parity fix is published, intentionally break one template locally, confirm the per-PR `scaffold-smoke` job goes red; the release gate is verified by its first green run post-publish |

*All other phase behaviors have automated verification via the scaffold→build exit-0 assertion.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
