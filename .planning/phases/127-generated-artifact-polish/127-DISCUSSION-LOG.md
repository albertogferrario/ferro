# Phase 127: Generated artifact polish — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-08
**Phase:** 127-generated-artifact-polish
**Mode:** `--auto` (Claude auto-selected the recommended option for every gray area)
**Areas discussed:** Dockerfile entrypoint, DO web service, do:init envs, build dedupe, dep table ordering, "Next steps" footer, --dry-run, .dockerignore README warning

---

## Dockerfile entrypoint (REPORT item 18)

| Option | Description | Selected |
|--------|-------------|----------|
| ENTRYPOINT + CMD with shared bin detection | Reuse `web_bin` heuristic from `do:init` so Dockerfile and DO stay in sync; emit `ENTRYPOINT ["/usr/local/bin/<bin>"]` and `CMD ["serve"]` | ✓ |
| ENTRYPOINT only, no CMD | Force user to pass subcommand explicitly | |
| Require explicit `web_bin` in metadata, no fallback | Stricter, but breaks single-bin convenience | |

**User's choice:** Auto-selected option 1 (recommended). Single source of truth via shared bin detection; matches the do:init heuristic that already exists.

---

## DO web service entrypoint (REPORT item 18 corollary)

| Option | Description | Selected |
|--------|-------------|----------|
| Rely on Dockerfile ENTRYPOINT, no `run_command:` on web | Single source of truth; matches existing worker model where `run_command` is opt-in | ✓ |
| Emit `run_command:` on web service AND Dockerfile ENTRYPOINT | Belt-and-suspenders, but duplicates the bin name in two files | |

**User's choice:** Auto-selected option 1 (recommended). Avoids duplication.

---

## `do:init` env entries (REPORT item 16)

| Option | Description | Selected |
|--------|-------------|----------|
| Real entries with empty values, secret typing via heuristic | `doctl apps update`-ready; users still set values but structure is correct | ✓ |
| Real entries, all `type: SECRET` | Safer default, but noisy and miscategorizes obvious non-secrets | |
| Keep comment-only block | Status quo; no improvement | |

**User's choice:** Auto-selected option 1. Heuristic on key name (substring match: secret/password/token/key/dsn/private/credential, with `_URL` keys non-secret unless they also match).

---

## Build dedupe (REPORT item 6)

| Option | Description | Selected |
|--------|-------------|----------|
| Drop per-bin builds, keep single `cargo build --release` | Plain build already builds every `[[bin]]`; per-bin lines are cache no-ops | ✓ |
| Keep per-bin builds for explicitness | More verbose but no functional benefit | |

**User's choice:** Auto-selected option 1.

---

## Dep table ordering (REPORT item 5)

| Option | Description | Selected |
|--------|-------------|----------|
| Switch `rewrite_ferro_version.rs` to `toml_edit` | Preserves source order, minimal diff | ✓ |
| Stay on `toml` crate, accept alphabetization | Smaller change, but produces noisy diffs in code review | |

**User's choice:** Auto-selected option 1. Add `preserves_dep_table_order` regression test alongside existing rewriter tests.

---

## "Next steps" footer (REPORT item 7)

| Option | Description | Selected |
|--------|-------------|----------|
| 3-5 line cargo-style footer, suppressed in `--dry-run` | Concise, no emoji, points at the next concrete command | ✓ |
| Long help block with multiple sections | More info but breaks cargo-style brevity | |
| No footer | Status quo | |

**User's choice:** Auto-selected option 1. `docker:init` footer suggests `docker build` + `docker run`; `do:init` footer suggests `doctl apps create --spec`.

---

## `--dry-run` flag (REPORT item 9)

| Option | Description | Selected |
|--------|-------------|----------|
| Render to stdout with per-file headers, no filesystem writes | Standard CLI dry-run pattern; CI-friendly | ✓ |
| Write to a temporary directory and print the path | More inspectable but more side-effects | |
| Dry-run only the diff against existing files | More useful but more complex | |

**User's choice:** Auto-selected option 1. `--dry-run` short-circuits before any persistence; rendering errors are still hard errors (not soft warnings).

---

## `.dockerignore` README warning (REPORT item 10)

| Option | Description | Selected |
|--------|-------------|----------|
| Whitelist `README.md` via `!README.md` after the `*.md` exclusion | Smallest change; silences cargo's `readme = "README.md"` warning | ✓ |
| Drop the `*.md` exclusion entirely | Includes all docs in the image (size cost) | |
| Document the warning, change nothing | Lowest effort but the warning is noise | |

**User's choice:** Auto-selected option 1. Add a one-line comment in the generated `.dockerignore` explaining why README.md is whitelisted.

---

## Claude's Discretion

- Exact wording of the "Next steps" footer (within cargo-style, no-emoji constraint).
- Template token name(s) for the new ENTRYPOINT block.
- Whether the secret heuristic is extracted into a shared helper module now (recommended, since Phase 128 preflight will reuse it) or inlined.
- Whether new tests live in existing template test files or a new integration test file (recommended: extend existing).

## Deferred Ideas

- Preflight checks (items 3, 4, 12, 13, 17) → Phase 128.
- Interactive deploy:init metadata scaffolder (item 15) → Phase 128.
- Publish workflow gating + per-crate version notes (items 8, 14) → Phase 129.
- gsd-tools phase-numbering collision bug (item 11) → file against `gsd-tools`, not Ferro.
