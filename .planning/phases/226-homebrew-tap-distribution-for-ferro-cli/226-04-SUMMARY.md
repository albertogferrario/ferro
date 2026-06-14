---
phase: 226-homebrew-tap-distribution-for-ferro-cli
plan: 04
status: complete
completed: 2026-06-14
autonomous: false
deviation: token-free pivot (no PAT) — see 226-CONTEXT.md <deviation>
---

# Plan 226-04 Summary — Tap setup (token-free)

Originally the operator runbook (create tap, mint PAT, push, verify). The operator chose to
eliminate the token, so the work was done directly by the agent under the token-free design.

## Done

- **Created** the public tap repo `albertogferrario/homebrew-ferro` (default branch `master`).
- **Seeded** it via the GitHub Contents API (no SSH push needed):
  - `Formula/ferro.rb` — seed formula (real structure, placeholder checksums).
  - `Formula/ferro.rb.tpl` — render template.
  - `bin/update-formula.sh` — token-free self-update: reads ferro's public `releases/latest`,
    computes the 4 SHA256s, renders the formula, commits to its own repo. Robust to no-release
    (gh `--jq` dumps the 404 body to stdout → guarded by exit status + tag-shape check) and to
    partially-published releases (missing asset → clean skip).
  - `.github/workflows/update-formula.yml` — `schedule` (every 6h) + `workflow_dispatch`,
    `permissions: contents: write`, runs the script with the built-in `GITHUB_TOKEN`.
  - `.github/workflows/tests.yml` — `ruby -c` + `brew audit --strict` (audit `--online` on PRs).
- **Verified** the self-update workflow runs **green** via `workflow_dispatch`:
  exits cleanly with "No published albertogferrario/ferro release yet — nothing to do."
- **Removed** the now-obsolete ferro-side artifacts (relocated into the self-contained tap):
  the `bump-homebrew-formula` job in `release.yml`, `scripts/bump-homebrew-formula.sh`, and the
  `homebrew/` staging dir. Replaced the job with an explanatory comment. `release.yml` YAML valid;
  no `HOMEBREW_TAP_TOKEN` references remain.

## Eliminated (vs original plan)

- The fine-grained PAT and the `HOMEBREW_TAP_TOKEN` secret — not needed under the self-poll design.
- The `workflow`-scoped cross-repo push from ferro — the tap is independent and polls public releases.

## Remaining (one item, normal release action — not tap work)

- ferro must publish its **first release** (push + tag `vX.Y.Z` → `release.yml` builds the 4 tarballs).
  The tap then auto-bumps to real checksums (within 6h, or instantly via the tap's "Run workflow"),
  after which `brew install albertogferrario/ferro/ferro && ferro new` works end-to-end.
- Live verification of that install is tracked in `226-HUMAN-UAT.md`.

## Decisions satisfied

D-01 (own tap) ✅ · D-02 (4-arch binary formula) ✅ (lives in tap) · D-03 (auto-bump) ✅ token-free
self-poll · D-04 (no PAT) ✅ eliminated · D-05 (`test do` + audit CI) ✅ in tap · D-06 (docs) ✅.

## Self-Check: PASSED
- Tap repo exists, public, seeded (5 files).
- Self-update workflow ran green (token-free, exited correctly on no-release).
- ferro `release.yml` clean; obsolete artifacts removed; YAML valid.
