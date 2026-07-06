---
phase: 228-readme-and-scaffold-doc-sweep
plan: 01
subsystem: documentation
tags: [docs, readme, scaffold, install, brew]
dependency_graph:
  requires: []
  provides: [accurate-install-instructions, brew-first-scaffold-readme, tap-readme-draft]
  affects: [ferro-cli/src/templates/files/root/README.md.tpl, README.md, scripts/install.sh, scripts/create-app.sh]
tech_stack:
  added: []
  patterns: [brew-first install ordering, toolchain-free CLI distinction, neutral version-pinless status]
key_files:
  created:
    - .planning/phases/228-readme-and-scaffold-doc-sweep/tap-README-draft.md
  modified:
    - ferro-cli/src/templates/files/root/README.md.tpl
    - README.md
    - scripts/install.sh
    - scripts/create-app.sh
decisions:
  - "Brew-first ordering applied everywhere; cargo install kept as acknowledged alternate (not removed)"
  - "Status line drops version and milestone names entirely — both rot on every release"
  - "Scaffold README 'cargo run' troubleshooting replaced by 'ferro serve' — aligns with actual toolchain-free flow"
metrics:
  duration: "~45 minutes"
  completed: "2026-06-15"
  tasks_completed: 5
  files_modified: 4
  files_created: 1
---

# Phase 228 Plan 01: README and Scaffold Doc Sweep Summary

One-liner: aligned four user-facing docs to the brew-first / Rust-1.88+ / toolchain-free-CLI flow established in v0.2.60–v0.2.61, replacing stale version pins and phantom commands.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Fix scaffold README template | d6a410e0 | ferro-cli/src/templates/files/root/README.md.tpl |
| 2 | Fix root README stale Status line | 6d17a597 | README.md |
| 3 | Fix installer scripts | 16e687bf | scripts/install.sh, scripts/create-app.sh |
| 4 | Create tap-repo README draft | ff60a31e | .planning/phases/228-readme-and-scaffold-doc-sweep/tap-README-draft.md |
| 5 | Goal-backward consistency sweep | (no commit — verification only) | — |

## Decisions Made

1. **Brew leads everywhere, cargo is an acknowledged alternate.** Both the scaffold README template and create-app.sh now show `brew install albertogferrario/ferro/ferro` first, with `cargo install ferro-cli` retained as a parenthetical alternate. README.md already had this ordering — no change needed there.

2. **Status line drops all version and milestone identifiers.** The old line named `v0.2.0` and `v12.0 spec-driven rendering` — both immediately stale. Replaced with "Pre-1.0. Breaking changes are allowed between minor versions until 1.0." which will remain accurate until v1.0 ships.

3. **`ferro serve` replaces `cargo run` in scaffold README troubleshooting.** The original `cargo run` wording was factually correct but inconsistent with the brew-installed CLI flow. The trailing context ("types are regenerated automatically on each start") was preserved.

## Task 5 Sweep Output (verbatim)

### Sweep 1 — Phantom commands (all must return 0)

```
ferro migrate in scripts/ + tpl:        0  PASS
cargo run -- migrate in scripts/:       0  PASS
cargo run -- serve in scripts/:         0  PASS
cargo run in tpl:                       0  PASS
```

### Sweep 2 — Stale version/milestone pins (all must return 0)

```
v0.2.0 in README.md:                    0  PASS
v12.0 spec-driven in README.md:         0  PASS
1.75 in tpl:                            0  PASS
```

### Sweep 3 — Brew leads permanent-install locations

```
brew count in tpl (expect >= 2):        2  PASS
brew count in create-app.sh (expect >= 1): 1  PASS
cargo install ferro-cli in create-app.sh (alternate retained): 1  PASS
cargo install ferro-cli in README.md (inline alternate retained): 1  PASS
```

### Sweep 4 — Toolchain-free distinction consistency

```
1.88+ in tpl (expect >= 1):             1  PASS
1.88+ in tap draft (expect >= 1):       1  PASS
Node.js** 18+ in tpl (expect >= 1):     1  PASS
```

**Final automated sweep:** `SWEEP_PASS`

**Rust test gate:** `cargo test -p ferro-cli -- test_readme_substitution` — 1 passed, 0 failed.

## Deviations from Plan

None — plan executed exactly as written. All five edits in Task 1 applied verbatim. Task 3 three-line replacement (create-app.sh lines 141-142 → three lines) applied as specified. All acceptance criteria passed on first attempt.

## Known Stubs

None. All edits are documentation/messaging corrections with no placeholder content.

## Threat Flags

None. Only echo/printf message strings and documented command names were changed. No download URLs, platform detection, archive extraction, exec paths, or control flow were modified (per T-228-01 disposition: accept).

## Self-Check: PASSED
