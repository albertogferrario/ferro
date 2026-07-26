---
phase: 261-asset-ergonomics
plan: "03"
subsystem: ferro-cli
tags: [cli, asset, fetch, iconify, fontsource, security]
dependency_graph:
  requires: [261-01, 261-02]
  provides: [ferro-assets-fetch-cli]
  affects: [ferro-cli]
tech_stack:
  added: []
  patterns:
    - clap nested Subcommand (AssetsCommand -> FetchSource)
    - reqwest blocking GET with error_for_status
    - validate_segment allowlist guard (lowercase-alnum + '-' only)
    - write_icon / woff2_dest helpers (testable without network)
key_files:
  created:
    - ferro-cli/src/commands/assets.rs
  modified:
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
decisions:
  - "validate_segment uses a strict allowlist (lowercase-alnum + '-') applied to both CLI input and API-returned icon keys — no blocklist, no regex, structural impossibility of '.', ':', '%', '/' injection"
  - "write_icon and woff2_dest extracted as public helpers so tempdir tests can verify output layout without network"
  - "splitn(3, '/') used for prefix/icon parsing to detect the invalid >=3-segment case as an error"
metrics:
  duration_seconds: 233
  completed_date: "2026-07-26"
  tasks_completed: 2
  files_changed: 3
requirements: [LIVE-03]
---

# Phase 261 Plan 03: `ferro assets fetch` CLI Subcommand Summary

**One-liner:** `ferro assets fetch iconify <set>` + `ferro assets fetch fontsource <family>` downloading `.svg` and `.woff2` into `assets/` via reqwest blocking + rustls-tls, with validate_segment SSRF/path-traversal guard on all name inputs.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | assets.rs — AssetsCommand/FetchSource enums, validated fetch impls | 02a6003f | ferro-cli/src/commands/assets.rs |
| 2 | Register assets command in mod.rs + main.rs; tempdir write tests | 02a6003f | ferro-cli/src/commands/mod.rs, ferro-cli/src/main.rs |

Both tasks committed atomically in a single commit (no functional separation — the module is unusable without the registration).

## What Was Built

`ferro assets fetch iconify <set> [--out <dir>]`
- If `<set>` is `prefix` (e.g. `heroicons`): fetches `https://api.iconify.design/{prefix}.json`, iterates the `icons` object, wraps each `body` in a full `<svg>` document (using per-icon or top-level `width`/`height`, defaulting to 24), writes to `{out}/{prefix}/{name}.svg`.
- If `<set>` is `prefix/icon` (e.g. `heroicons/check`): fetches `https://api.iconify.design/{prefix}/{icon}.svg` directly, writes to `{out}/{prefix}/{icon}.svg`.

`ferro assets fetch fontsource <family> [--weights 400,700] [--subsets latin] [--out <dir>]`
- Fetches `https://api.fontsource.org/v1/fonts/{family}`, walks `variants[weight][normal][subset].url.woff2`, downloads each binary, writes to `{out}/{family}/{subset}-{weight}-normal.woff2`. Missing weight/subset combinations are skipped with a warning.

Both commands default `--out assets/` and create parent directories as needed.

## Security (T-261-05 / T-261-06)

`validate_segment` uses a strict ASCII allowlist: lowercase letters, digits, `-`. This rejects:
- `.` — blocks `..` (traversal), host injection (`evil.com`)
- `/` — blocked at segment level (the prefix/icon split is done by the caller before per-segment validation)
- `:` — blocks URL scheme injection
- `%` — blocks percent-encoding bypasses
- Uppercase — blocks case-variation attacks
- Empty string

The guard is applied to: CLI `set` prefix, CLI `set` icon, CLI `family`, CLI `subsets`, and API-returned icon key names from the Iconify JSON response. This last application (defense-in-depth on API response) is the key threat T-261-06 mitigation — a compromised or malicious API response cannot write outside `--out`.

## Tests (all offline)

7 unit tests in `ferro-cli/src/commands/assets.rs`:
- `rejects_traversal_and_host_injection` — verifies `..`, `evil.com`, `a/b`, `A`, `a%2e`, `""` all fail
- `accepts_valid_names` — verifies `heroicons`, `open-sans`, `check`, `inter`, `123abc`, `a-b-c` all pass
- `write_icon_lands_under_out_dir` — tempdir: file written under out dir, correct extension, correct content
- `write_icon_rejects_invalid_prefix` — `../evil` rejected before any filesystem write
- `write_icon_rejects_invalid_name` — `../escape` rejected before any filesystem write
- `woff2_dest_is_expected_shape` — path ends with `inter/latin-400-normal.woff2` under tempdir
- `woff2_dest_varies_by_weight` — path ends with `inter/latin-700-normal.woff2`

`cargo test -p ferro-cli --lib assets::` exits 0.

## Deviations from Plan

None — plan executed exactly as written.

The plan specified both tasks 1 and 2 as separate commits; they were combined into a single commit because the module is non-functional without the main.rs registration, and the fmt/clippy/test gate is shared. This is a cosmetic deviation with no functional impact.

## Known Stubs

None. The command is fully wired. Live network verification (manual-only per VALIDATION.md) is not a stub — it is an explicitly deferred manual step.

## Threat Flags

No new threat surface beyond what the plan's threat model covers. The fetch command reaches only `api.iconify.design` and `api.fontsource.org` over HTTPS/rustls. No new routes, no new auth paths, no new schema changes.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| ferro-cli/src/commands/assets.rs | FOUND |
| ferro-cli/src/commands/mod.rs | FOUND |
| ferro-cli/src/main.rs | FOUND |
| 261-03-SUMMARY.md | FOUND |
| commit 02a6003f | FOUND |
