---
phase: 226
slug: homebrew-tap-distribution-for-ferro-cli
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-14
---

# Phase 226 — Validation Strategy

> Per-phase validation contract. Derived from 226-RESEARCH.md "## Validation Architecture".
> This phase produces a Homebrew formula + a release.yml bump job + an in-repo bump script +
> docs. Much of the live validation is operator-manual (the tap repo and secret are external).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `ruby -c` (formula syntax) · `brew audit`/`brew test-bot` (tap CI) · `bash` (bump script) · GitHub Actions |
| **Config file** | none — formula template + bump script live in this repo; audit/test run in the tap repo CI |
| **Quick run command** | `ruby -c homebrew/Formula/ferro.rb` (or the rendered template) |
| **Full suite command** | `bash scripts/bump-homebrew-formula.sh <tag>` (dry) + `brew audit --strict <formula>` |
| **Estimated runtime** | seconds (syntax/script); tap CI minutes |

---

## Sampling Rate

- **After every task commit:** `ruby -c` the formula/template; `bash -n scripts/bump-homebrew-formula.sh` (syntax) for the script task; YAML-parse `release.yml` for the bump-job task.
- **After the wave:** dry-run `scripts/bump-homebrew-formula.sh` against a known tag and confirm it renders a formula whose `ruby -c` passes; `brew audit --strict` the rendered formula if `brew` is available on the runner.
- **Before `/gsd-verify-work`:** formula syntax green; bump script renders valid formula; release.yml parses; docs updated.
- **Max feedback latency:** < 10s for syntax/render checks.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Decision | Test Type | Automated Command | File Exists | Status |
|---------|------|------|----------|-----------|-------------------|-------------|--------|
| 226-xx | 01 | 1 | D-02 | syntax | `ruby -c homebrew/Formula/ferro.rb` (seed/template) → exit 0 | ❌ W0 | ⬜ pending |
| 226-xx | 01 | 1 | D-02 | structure | `grep -q 'on_macos' && grep -q 'on_linux' && grep -q 'bin.install "ferro"'` on the formula | ❌ W0 | ⬜ pending |
| 226-xx | 02 | 1 | D-03 | script syntax | `bash -n scripts/bump-homebrew-formula.sh` → exit 0 | ❌ W0 | ⬜ pending |
| 226-xx | 02 | 1 | D-03 | render | dry-run script with a sample tag → emits a formula whose `ruby -c` passes; 4 distinct sha256 lines | ❌ W0 | ⬜ pending |
| 226-xx | 03 | 2 | D-03/D-04 | yaml + wiring | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"`; bump job has `needs: release`, `if: github.event_name == 'push'` (non-prerelease), uses `secrets.HOMEBREW_TAP_TOKEN` | ❌ W0 | ⬜ pending |
| 226-xx | 03 | 2 | D-05 | tap CI artifact | tap-repo `tests.yml` snippet present in repo (staged for operator) — `grep test-bot` | ❌ W0 | ⬜ pending |
| 226-xx | 04 | 2 | D-06 | docs | install docs/README contain `brew install albertogferrario/ferro/ferro` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red. Task IDs finalized by the planner.*

---

## Wave 0 Requirements

- No new test framework. `ruby` (formula syntax), `bash -n` (script), `python3 -c yaml` (workflow), `grep` (structure) are all present.
- `brew audit`/`brew test-bot` run in the **tap repo CI** (operator-created), not this repo — staged as a snippet.

---

## Manual-Only Verifications

| Behavior | Decision | Why Manual | Test Instructions |
|----------|----------|------------|-------------------|
| Tap repo + `Formula/` exist | D-01 | External repo; executor cannot create it | Operator creates `albertogferrario/homebrew-ferro`, copies the seed formula + tap `tests.yml` |
| `HOMEBREW_TAP_TOKEN` secret set | D-04 | Secret creation is operator-only | Operator: fine-grained PAT, Contents:write on `homebrew-ferro` only, add as repo secret |
| End-to-end `brew install` | D-01/D-02 | Needs the live tap + a published release | After first post-wiring release: `brew install albertogferrario/ferro/ferro && ferro --version` on macOS and Linux |
| `ferro new myapp` on a clean (no-Rust) Mac | killer feature | Needs a real toolchain-free machine | Run `ferro new myapp`, verify scaffold |
| Bump job green on a real tag | D-03 | Needs an actual release run | Watch the `release.yml` bump job in Actions after the next tag |

---

## Validation Sign-Off

- [ ] Each task has an automated syntax/structure/render check or a Manual-Only entry
- [ ] No 3 consecutive tasks without automated verify
- [ ] Wave 0 covers MISSING references (none — existing tooling)
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s for in-repo checks
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
