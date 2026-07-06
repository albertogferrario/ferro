---
phase: 228-readme-and-scaffold-doc-sweep
verified: 2026-06-15T10:00:00Z
status: passed
score: 6/6
overrides_applied: 0
---

# Phase 228: README and Scaffold Doc Sweep — Verification Report

**Phase Goal:** Make every README + generated-app docs consistent and current — root README brew-first install + quickstart matching the real flow; the scaffold's generated README template reflecting the rustls/SQLite-default app + `ferro serve` flow; `scripts/install.sh`/`create-app.sh` user-facing messaging; the toolchain-free-CLI vs Rust-needed-to-build-app distinction stated consistently. Docs/scripts-only phase. Tap-repo README is a DRAFT artifact in-repo only.

**Verified:** 2026-06-15T10:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Root README Status line no longer pins a stale version (v0.2.0) or milestone (v12.0 spec-driven) | VERIFIED | `grep "v0.2.0\|v12.0 spec-driven" README.md` → 0 matches. Line 186 reads "Pre-1.0. Breaking changes are allowed between minor versions until 1.0." |
| 2 | Scaffold-generated README leads install with brew, states Rust 1.88+ MSRV and Node 18+ | VERIFIED | tpl line 9: `1.88+`; line 10: `brew install albertogferrario/ferro/ferro` (toolchain-free); line 11: `Node.js 18+`. Brew count in tpl = 2 (line 10 + line 81). |
| 3 | scripts/install.sh prints the real migrate command (ferro db:migrate), not the phantom 'ferro migrate' | VERIFIED | `grep "ferro db:migrate" scripts/install.sh` → line 184. `grep "ferro migrate" scripts/install.sh` → 0 matches. |
| 4 | scripts/create-app.sh next-steps use ferro db:migrate / ferro serve and lead permanent install with brew (install before use) | VERIFIED | After review-fix commit dec2ef2a: brew install at line 140, ferro db:migrate at line 143, ferro serve at line 144. Install step appears BEFORE the run commands. `cargo install ferro-cli` retained as alternate (line 141). npm install frontend step retained (line 137). |
| 5 | A ready-to-paste tap README draft exists inside the ferro repo (no push to the separate tap repo) | VERIFIED | File exists at `.planning/phases/228-readme-and-scaffold-doc-sweep/tap-README-draft.md`. Contains `brew install albertogferrario/ferro/ferro` and the cross-repo boundary comment. `git remote -v \| grep homebrew-ferro` → 0 results. |
| 6 | The toolchain-free-CLI vs Rust-1.88+-to-build distinction reads consistently with installation.md across all edited files | VERIFIED | Oracle (installation.md): "toolchain-free via Homebrew" + "Rust 1.88+... to build the app". Scaffold tpl: `brew install ... (toolchain-free; no Rust needed)` + `Rust (stable, 1.88+)`. Tap draft: "No Rust toolchain required to install" + "Rust 1.88+ to build and run a scaffolded app". create-app.sh: "toolchain-free — required for the commands below". |

**Score:** 6/6 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `README.md` | Neutral, version-pinless Status section containing "Pre-1.0" | VERIFIED | Line 186: "Pre-1.0. Breaking changes are allowed between minor versions until 1.0." Quickstart includes `ferro db:migrate` (added in review-fix commit dec2ef2a). |
| `ferro-cli/src/templates/files/root/README.md.tpl` | Brew-first scaffold README with 1.88+/Node 18+ | VERIFIED | 88 lines, substantive content. Contains `brew install albertogferrario/ferro/ferro` (2 occurrences), `1.88+`, `18+`, `ferro db:migrate`, `ferro serve`. No phantom `ferro routes` row (removed in review-fix dec2ef2a). |
| `scripts/install.sh` | Correct migrate command in next-steps | VERIFIED | Line 184: `ferro db:migrate`. No phantom `ferro migrate` present. |
| `scripts/create-app.sh` | Correct commands + brew-first permanent install | VERIFIED | brew install at line 140 (before run commands), `ferro db:migrate` at 143, `ferro serve` at 144. npm install retained at 137. |
| `.planning/phases/228-readme-and-scaffold-doc-sweep/tap-README-draft.md` | Tap repo README draft with brew one-liner | VERIFIED | File exists. Contains `brew install albertogferrario/ferro/ferro`, `1.88+`, boundary comment on line 1, all 6 required facts (brew one-liner, toolchain-free, auto-bump via workflow_run, post-install commands, Rust build requirement, links). |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-cli/src/templates/files/root/README.md.tpl` | `ferro-cli/src/templates/mod.rs::test_readme_substitution` | `include_str!` in `project.rs:222` | VERIFIED | `project.rs:221–226`: `readme()` function uses `include_str!("files/root/README.md.tpl")` with 3 `.replace()` calls. Test at `mod.rs:582–589` asserts all 5 required strings (`# My App`, `A test description`, `cd my-app`, `ferro serve`, `ferro db:migrate`). All 5 strings present in current template. SUMMARY reports test passed green. |

---

## Data-Flow Trace (Level 4)

Not applicable — docs/scripts phase. No components rendering dynamic data. All artifacts are static text files (markdown templates, shell scripts).

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Phantom commands absent | `grep -rn "ferro routes\|ferro migrate\|cargo run -- migrate" README.md ferro-cli/src/templates/files/root/README.md.tpl scripts/` | 0 matches | PASS |
| Stale version pins absent | `grep -n "v0.2.0\|v12.0 spec-driven" README.md` | 0 matches | PASS |
| Brew leads in scaffold tpl (2+ occurrences) | `grep -c "brew install albertogferrario/ferro/ferro" ferro-cli/src/templates/files/root/README.md.tpl` | 2 | PASS |
| MSRV 1.88+ consistent | `grep -c "1.88+" ferro-cli/src/templates/files/root/README.md.tpl` | 1 | PASS |
| No .rs or Cargo.toml files modified | `git show d6a410e0 6d17a597 16e687bf ff60a31e dec2ef2a --name-only --format="" \| grep -E "\.rs$\|Cargo\.toml"` | 0 matches | PASS |
| No homebrew-ferro remote added | `git remote -v \| grep -c "homebrew-ferro"` | 0 | PASS |

---

## Requirements Coverage

No requirement IDs scoped to this phase (`requirements: []` in PLAN frontmatter — intentional, per plan note).

---

## Anti-Patterns Found

No anti-patterns found. All edited files are documentation and shell scripts (messaging only). No placeholder content, no phantom commands remaining, no stale pins.

---

## Human Verification Required

None. All observable truths are verifiable via grep and file inspection. The tap draft is a planning artifact for manual paste — its content accuracy is verifiable by reading it against the documented 6 facts (all present).

---

## Additional Context: Review Findings Addressed

The code review (228-REVIEW.md) raised 3 findings. All were addressed in a follow-up commit `dec2ef2a` (after the 4 task commits):

- **WR-01** (phantom `ferro routes` in scaffold template): Removed. The command table row is gone from the current template.
- **IN-01** (root README quickstart missing `ferro db:migrate`): Added. Line 25 of README.md now shows `ferro db:migrate` between `cd myapp` and `ferro serve`.
- **IN-02** (create-app.sh install step ordering): Fixed. Brew/cargo install hint now appears BEFORE the `ferro db:migrate`/`ferro serve` commands, with an inline comment "required for the commands below".

These fixes were within scope (same files, same docs/messaging category) and bring the phase to full goal achievement.

---

## Gaps Summary

None. All 6 must-have truths verified. All artifacts exist, are substantive, and are correctly wired. Phase goal achieved.

---

_Verified: 2026-06-15T10:00:00Z_
_Verifier: Claude (gsd-verifier)_
