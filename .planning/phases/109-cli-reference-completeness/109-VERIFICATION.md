---
phase: 109-cli-reference-completeness
verified: 2026-03-26T00:00:00Z
status: passed
score: 6/6 must-haves verified
---

# Phase 109: CLI Reference Completeness Verification Report

**Phase Goal:** Close documentation gaps for all CLI commands — every ferro-cli command must have a corresponding reference entry in docs/src/reference/cli.md
**Verified:** 2026-03-26
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Every CLI command in ferro-cli has a reference entry in docs/src/reference/cli.md | VERIFIED | 49 `### \`ferro` headings in cli.md; all Commands enum variants in main.rs (lines 18-408) have corresponding sections |
| 2 | Each new entry follows the same format (synopsis, flags, description, example) | VERIFIED | All 12 new standalone sections contain bash fenced examples, Options tables (where applicable), and "What it does" or "Generated files" lists |
| 3 | The Command Summary table includes all 13 previously missing commands | VERIFIED | Summary table now has 49 command rows; api:check, clean, make:api, make:api-key, make:lang, make:projection, make:stripe, make:theme, make:whatsapp, projection:check, validate:contracts all present (make:policy was already a row) |
| 4 | generate-routes is documented as an internal note under generate-types, not as a standalone command | VERIFIED | Line 148 of cli.md: "> **Includes route generation:** `generate-types` also runs route generation internally, producing TypeScript route helpers alongside the type definitions." No standalone `### \`ferro generate-routes\`` heading exists. No Command Summary row for generate-routes. |
| 5 | projection:check entry notes the projections feature gate requirement | VERIFIED | Line 1198 of cli.md: "> **Requires the `projections` feature.** Build with `cargo build --features projections` or add `projections` to the `default` features in `Cargo.toml` before running this command." |
| 6 | make:policy has a full body section (previously only had a summary table row) | VERIFIED | Lines 796-834 contain the full body section: synopsis, options table (name + --model/-m), generated file path, and Rust code example. Only one summary table row exists (line 1339). |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `docs/src/reference/cli.md` | Complete CLI reference with all commands documented | VERIFIED | File exists at 1377 lines. Contains 49 `### \`ferro` command headings. Pattern match `api:check` found (5 occurrences). New `## Validation & Diagnostics` section added at line 1161. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `docs/src/reference/cli.md` | `ferro-cli/src/main.rs` | flag names and defaults match source code definitions | VERIFIED | All flags verified against main.rs Commands enum: `--sweep` (Clean variant, line 383), `--url`/`--api-key`/`--spec-path` (ApiCheck variant, lines 400-408), `--yes`/`--all`/`--exclude`/`--include-all` (MakeApi variant, lines 97-108), `--env` (MakeApiKey, line 88), `--model`/`-m` (MakePolicy, lines 210-211), `--from-model` (MakeProjection, line 219), `--connect` (MakeStripe, line 121), `--name` (ProjectionCheck, line 228), `--filter`/`-f`/`--json` (ValidateContracts, lines 389-395). All defaults and flag names match. |
| `docs/src/reference/cli.md` | `ferro-cli/src/main.rs` | 13 command names present | VERIFIED | 12 of 13 names found explicitly in cli.md (generate-routes documented via "route generation" phrase at line 148, not by its hyphenated name — correct per plan). All 12 standalone commands appear as headings and in the Command Summary table. |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| CLIMCP-01 | 109-01-PLAN.md | All 13 undocumented CLI commands added to reference/cli.md | SATISFIED | 12 new standalone `### \`ferro\`` sections added plus 1 internal note. 11 new Command Summary rows added (make:policy already had a row). REQUIREMENTS.md line 84 marks this as Complete. |

No orphaned requirements found. Only CLIMCP-01 maps to Phase 109 in the traceability table (REQUIREMENTS.md lines 78-100). CLIMCP-02 and CLIMCP-03 map to Phase 110, not Phase 109.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `docs/src/reference/cli.md` | 625 | `# Basic resource (generates struct with placeholder fields)` | Info | Comment in a bash example for `make:resource` — pre-existing, describes behavior of the generated stub file, not a documentation gap |
| `docs/src/reference/cli.md` | 900 | `theme.json` described as "a placeholder for overriding..." | Info | Accurate description of the generated file's purpose — this is factual, not a stub indicator |

No blockers. The word "placeholder" at line 900 accurately describes what `theme.json` is (an empty overrides file). The comment at line 625 is in a pre-existing section (`make:resource`), not added in this phase.

### Human Verification Required

None. The documentation content (flag names, defaults, descriptions) has been fully verified against the authoritative source (`ferro-cli/src/main.rs` Commands enum). All observable truths are mechanically verifiable.

### Gaps Summary

No gaps. All six must-have truths are verified against the actual codebase:

1. The 49-heading count matches: 37 pre-existing + 12 new standalone sections.
2. The Command Summary table has 49 command rows (51 total `| \`` rows including 2 environment variable rows at the end).
3. All flag names and defaults in the new sections match the `#[arg(...)]` definitions in `ferro-cli/src/main.rs`.
4. `generate-routes` is handled exactly as specified — internal note only, no standalone heading, no summary row.
5. `projection:check` includes the `projections` feature gate callout with the exact `cargo build --features projections` command.
6. `make:policy` has a complete body section with no duplicate summary row.
7. Commit `fb74d34d` exists and documents the work.
8. REQUIREMENTS.md traceability table marks CLIMCP-01 as Complete for Phase 109.

---

_Verified: 2026-03-26_
_Verifier: Claude (gsd-verifier)_
