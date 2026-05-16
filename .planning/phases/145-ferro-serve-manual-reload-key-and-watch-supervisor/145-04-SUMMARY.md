---
plan: 145-04
phase: 145
title: docs refresh + phase-wide cargo-watch grep gate
status: complete
completed: 2026-04-22
tasks: 3
---

# 145-04 Summary

Refreshed the `ferro serve` documentation surface to match the 02a + 02b
implementation. Verified phase-wide removal of `cargo-watch` references (outside
the unrelated `test.md` which documents `cargo test --watch`).

## Tasks

1. **Task 1 — commit `3d408041`**: Rewrote `docs/src/reference/cli.md` for the
   new `--watch` + r-key model. Removed all cargo-watch installation language.
   Documents default behavior (no watching, manual `r` reload), `--watch` opt-in
   (500 ms debounced file watcher), and `q`/Ctrl-C shutdown.

2. **Task 2 — commit `82492119`**: Rewrote `ferro-cli/src/commands/skills/serve.md`
   for the same model. Default-mode framing uses the neutral "use when you want
   explicit control over rebuild timing" (no environment-specific phrasing such
   as "thermally-constrained machines" — grep-verified at zero occurrences).

3. **Task 3 — pure verification, no edits**:
   - `grep -rn "cargo-watch\|cargo watch" docs/src/ ferro-cli/src/ | grep -v "test.md"`
     → empty (clean)
   - `./target/debug/ferro serve --help | grep -- "--watch"` → matches
     `--watch  Enable file-watch auto-reload (500ms debounce)`
   - `grep -cE "thermally-constrained|slow laptop" ferro-cli/src/commands/skills/serve.md`
     → `0`
   - `cargo clippy -p ferro-cli --all-targets -- -D warnings` → exit 0

## Gates

- Grep gate: clean (cargo-watch fully absent from `docs/src/` and `ferro-cli/src/`,
  except the allowed `test.md` reference for `cargo test --watch`)
- `--watch` flag discoverable in `ferro serve --help`
- No environment-specific framing leaked into public-facing skill prompt
- Clippy clean

## Decision Coverage

D-32 (remove cargo-watch from docs), D-33 (docs/src/ updated), D-34 (clap help
text asserts `--watch` visibility).

## Key Files

- `docs/src/reference/cli.md` — updated for --watch + r-key model
- `ferro-cli/src/commands/skills/serve.md` — updated with neutral voice

## Notes

- Task 3 was executed inline after the parallel worktree agent was interrupted;
  verification output is captured above rather than in a separate commit since
  it modifies zero files.
- `docs/book/` generated output is ignored by the grep gate (it's mdbook build
  output that will regenerate from the updated `docs/src/` content).
- `ferro-cli/src/commands/skills/test.md` retains references to `cargo test --watch`
  (a cargo nightly subcommand feature, unrelated to cargo-watch binary) — this is
  intentional and allowed by the grep gate's test.md exclusion.
