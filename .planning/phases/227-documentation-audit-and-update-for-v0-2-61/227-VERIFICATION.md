---
phase: 227-documentation-audit-and-update-for-v0-2-61
verified: 2026-06-15T00:00:00Z
status: passed
score: 10/10
overrides_applied: 0
re_verification: false
---

# Phase 227: Documentation Audit and Update for v0.2.61 — Verification Report

**Phase Goal:** Comprehensive sweep of `docs/src/` for accuracy after the v0.2.59→0.2.61 changes. Audit every page for stale content (TLS/OpenSSL→rustls, old install flow, version pins, scaffold structure, generators + `ferro serve` flow). Verify code/command examples against the live CLI. Surface and fix discrepancies, don't silently work around. Focus on factual accuracy, not a rewrite.
**Verified:** 2026-06-15
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `reference/cli.md` install section leads with Homebrew, matching `installation.md` | VERIFIED | Line 9: `### Homebrew (macOS and Linux — recommended)` appears before `### Cargo` (line 21). `brew install albertogferrario/ferro/ferro` at line 12. |
| 2 | `reference/cli.md` documents the real `db:sync` flag (`--skip-migrations`), not the phantom `--migrate` | VERIFIED | Line 1040: `ferro db:sync --skip-migrations`. Line 1047: table row `--skip-migrations`. `db:sync --migrate` absent from file. Verified against `ferro-cli/src/commands/db_sync.rs` (fn signature: `run(skip_migrations: bool, regenerate_models: bool)`). |
| 3 | `working-with-agents.md` references only real CLI commands (no phantom `ferro make:model`) | VERIFIED | `grep -rn "make:model" docs/src/` returns 0 matches. `ferro make:scaffold Post` present at lines 108, 112. `make:handler` also absent (WR-01 fixed via commit faecc891). |
| 4 | `working-with-agents.md` and `introduction.md` agree on MCP tool count (version-neutral, no contradiction) | VERIFIED | Both pages use `a full suite of introspection tools` / `a full suite of tools`. `grep "(57\|80+) (introspection )?tools" docs/src/` returns 0 matches. |
| 5 | `frontend-types.md` has no stale hard version pin and no broken cross-link | VERIFIED | `0.2.33` absent. `ferro docker:init --ferro-version <pinned> --force` at line 97. `(do-init.md)` absent; link at line 115 points to `../reference/cli.md#ferro-dockerinit`. |
| 6 | `migration-guide.md` MCP config uses the real `ferro mcp` invocation form | VERIFIED | "After" block at lines 90-98: `"command": "/absolute/path/to/target/debug/ferro"`, `"args": ["mcp"]`. `"command": "ferro-mcp"` absent. "Before" block (`cancer-mcp`) unchanged. |
| 7 | `introduction.md` milestone string is current (not stale v12.0) | VERIFIED | `v12.0 spec-driven rendering` absent. `Ferro is pre-1.0. Breaking changes are allowed between minor versions until 1.0.` retained at line 59. `80+ tools` absent. |
| 8 | No stale TLS/OpenSSL reference exists anywhere except the single correct install-page line | VERIFIED | `grep -rniE "native-tls\|runtime-tokio-native-tls\|openssl" docs/src/` returns exactly 1 match: `installation.md:8: ... no OpenSSL needed; the scaffold uses rustls`. |
| 9 | No stale hard version pin (`0.2.NN`) remains anywhere in `docs/src/` | VERIFIED | `grep -rnE "0\.2\.[0-9]+" docs/src/` returns 0 matches. |
| 10 | All intra-doc links resolve (`mdbook build` exits 0) | VERIFIED | `~/.cargo/bin/mdbook build docs/` exits 0. Both corrected cross-links in `cli.md` (`../features/authentication.md` at line 328, `../features/api-resources.md` at line 707) point to existing files. |

**Score:** 10/10 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `docs/src/reference/cli.md` | Brew-first install + `--skip-migrations` | VERIFIED | Modified by commits fa23302d (install) and 5f3455b9 (db:sync), plus faecc891 (cross-link fix) |
| `docs/src/getting-started/working-with-agents.md` | Real `make:scaffold` command, version-neutral tool count | VERIFIED | Modified by commit 5f5513b8 + faecc891 (make:handler fix) |
| `docs/src/cli/frontend-types.md` | No `0.2.33` pin, working cross-link | VERIFIED | Modified by commit b6503c90 |
| `docs/src/upgrading/migration-guide.md` | Real `ferro mcp` invocation, no `ferro make:model` | VERIFIED | Modified by commits 4d68fd78 (MCP config) and 36078905 (phantom command) |
| `docs/src/introduction.md` | No stale v12.0 string, version-neutral tool count | VERIFIED | Modified by commit facf7383 |
| `.planning/phases/227-.../227-03-SUMMARY.md` | Audit evidence (grep outputs + mdbook result) | VERIFIED | File exists with verbatim sweep outputs confirming all four staleness classes |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `docs/src/reference/cli.md` | `ferro-cli/src/commands/db_sync.rs` | `--skip-migrations` flag name matches `#[arg]` | WIRED | `db_sync.rs` line 13: `pub fn run(skip_migrations: bool, ...)` |
| `docs/src/reference/cli.md` | `docs/src/getting-started/installation.md` | Brew-first ordering reconciled | WIRED | Both pages lead with `brew install albertogferrario/ferro/ferro` |
| `docs/src/getting-started/working-with-agents.md` | `ferro-cli/src/commands/make_scaffold.rs` | `ferro make:scaffold` maps to real source module | WIRED | `make_scaffold.rs` exists; `make_model.rs` does not exist |
| `docs/src/upgrading/migration-guide.md` | `ferro-cli/src/commands/mcp.rs` | `ferro mcp` subcommand | WIRED | `"args": ["mcp"]` in "After" block |
| `docs/src/cli/frontend-types.md` | `docs/src/reference/cli.md#ferro-dockerinit` | Cross-link to docker:init section | WIRED | `grep -i "ferro docker:init" docs/src/reference/cli.md` confirms heading at line 1128; mdbook build succeeds |
| `docs/src/reference/cli.md` | `docs/src/features/authentication.md` | `../features/authentication.md` | WIRED | File exists; mdbook build exits 0 |
| `docs/src/reference/cli.md` | `docs/src/features/api-resources.md` | `../features/api-resources.md` | WIRED | File exists; mdbook build exits 0 |

---

## Data-Flow Trace (Level 4)

Not applicable. This is a documentation-only phase. No dynamic data rendering exists in static Markdown files.

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All intra-doc links resolve | `~/.cargo/bin/mdbook build docs/` | EXIT 0 | PASS |
| TLS sweep returns exactly 1 match | `grep -rniE "native-tls\|runtime-tokio-native-tls\|openssl" docs/src/` | 1 line: `installation.md:8` | PASS |
| No version pins remain | `grep -rnE "0\.2\.[0-9]+" docs/src/` | 0 matches | PASS |
| No phantom `make:model` | `grep -rn "make:model" docs/src/` | 0 matches | PASS |
| No contradictory tool counts | `grep -rnE "(57\|80\+) (introspection )?tools" docs/src/` | 0 matches | PASS |
| `--skip-migrations` present, `--migrate` absent | `grep "skip-migrations\|db:sync --migrate" docs/src/reference/cli.md` | `--skip-migrations` found, `--migrate` absent | PASS |
| Docs-only scope — no compiled source modified | `git log --since=2026-06-14 -- '*.rs' Cargo.toml` | No phase-227 commits touch `.rs` or `Cargo.toml` | PASS |

---

## Requirements Coverage

No requirement IDs are mapped to this phase. `requirements: []` is intentional per the phase plan — this is a documentation-only audit phase with no REQUIREMENTS.md entries.

---

## Anti-Patterns Found

| File | Pattern | Severity | Impact | Status |
|------|---------|----------|--------|--------|
| `docs/src/upgrading/migration-guide.md` | Internal pre-publication codename "cancer" used throughout as the h1 title and page narrative | Info | Public repo — new users may find the term unexpected; page otherwise accurate | Open — see Follow-up below |
| `docs/src/reference/cli.md` (line 536) | `"$schema": "ferro-json-ui/v2"` in a generated code sample | Info | Uses "v2" label in a code sample; memory note says public docs must not use "v2" framing | Open — see Follow-up below |

Neither item was within this phase's "factual accuracy, no rewrite" boundary and neither affects functional correctness of any documented command. Both are surfaced as follow-up decisions, not as blocking gaps.

---

## Human Verification Required

None. All must-haves are programmatically verifiable via grep and mdbook build. No visual, real-time, or external-service behavior is involved.

---

## Gaps Summary

No gaps. All 10 must-haves are verified. The whole `docs/src/` tree was swept by four grep oracles and mdbook build.

---

## Follow-up Items (Out of Scope — Not Blocking)

These items were surfaced during the phase (per REVIEW.md and verification) but fall outside the "factual accuracy, no rewrite" boundary defined for Phase 227. They require a developer decision before action.

### FU-01: `migration-guide.md` exposes internal codename "cancer"

`docs/src/upgrading/migration-guide.md` uses "cancer" (the pre-publication internal codename) in its h1, opening paragraph, and throughout the before/after examples as a public page in a public repository. This is an escalation-gate item: the options are (a) neutralize to `v1.x → v2.0` framing, (b) remove the page if no v1 users exist who need migration instructions, or (c) leave as-is with a note that the term is historical. A full rewrite is outside this phase's scope.

### FU-02: `"$schema": "ferro-json-ui/v2"` in CLI reference code sample

`docs/src/reference/cli.md` line 536 shows `"$schema": "ferro-json-ui/v2"` in the generated file sample for `make:json-view`. The project memory (`feedback_json_ui_naming.md`) specifies that public docs must not use the "v2" label. The correct action depends on what the live `make:json-view` command actually emits — verify with `ferro make:json-view <name>` and align the sample, or add a clarifying note if the schema identifier itself is intentional.

---

## Code Commits (Phase 227)

| Commit | Description | Files |
|--------|-------------|-------|
| fa23302d | Brew-first install ordering in reference/cli.md (DISC-01) | `docs/src/reference/cli.md` |
| 5f3455b9 | Correct db:sync flag from `--migrate` to `--skip-migrations` (DISC-02) | `docs/src/reference/cli.md` |
| 5f5513b8 | Fix phantom make:model + stale tool count in working-with-agents.md | `docs/src/getting-started/working-with-agents.md` |
| b6503c90 | Fix stale version pin and broken cross-link in frontend-types.md | `docs/src/cli/frontend-types.md` |
| 4d68fd78 | Fix stale ferro-mcp binary name in migration-guide.md MCP config | `docs/src/upgrading/migration-guide.md` |
| facf7383 | Remove stale v12.0 milestone string and make tool count version-neutral in introduction.md | `docs/src/introduction.md` |
| 36078905 | Remove phantom ferro make:model from migration guide (wave-2 sweep catch) | `docs/src/upgrading/migration-guide.md` |
| faecc891 | Fix code-review findings — phantom make:handler + broken feature-guide cross-links | `docs/src/getting-started/working-with-agents.md`, `docs/src/reference/cli.md` |

---

_Verified: 2026-06-15_
_Verifier: Claude (gsd-verifier)_
