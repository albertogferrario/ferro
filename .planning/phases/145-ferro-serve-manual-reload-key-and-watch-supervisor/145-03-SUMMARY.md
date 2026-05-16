---
plan: 145-03
phase: 145
title: integration tests + test-support deps + env-var reload hook
status: complete
completed: 2026-04-22
tasks: 2
---

# 145-03 Summary

Implemented the four integration tests scaffolded in Plan 01. Added test-support
dev-deps (`libc` for SIGINT delivery, `tempfile` confirmed present). Added the
`FERRO_SERVE_TEST_TRIGGER_PIPE` env-var hook in `serve.rs` resolving RESEARCH.md
assumption A4 (env-var pipe chosen over portable-pty).

## Tasks

1. **Task 1 — commit `9325a0f2`**: Added test-support deps in `ferro-cli/Cargo.toml`
   and the `FERRO_SERVE_TEST_TRIGGER_PIPE` reload trigger hook in `serve.rs`. The
   hook spawns a pipe reader thread when the env var is set, converting line-delimited
   writes into `TriggerSource::KeyR` messages over the same mpsc channel the real
   keyboard thread uses — integration tests can simulate an r-key press deterministically
   without a pseudo-TTY.

2. **Task 2 — commit `b91e67e7`**: Implemented four integration tests in
   `ferro-cli/tests/serve_supervisor.rs`:
   - `watch_mode_debounces_burst` — writes 10 .rs files in a tight burst inside a
     tempdir, spawns `ferro serve --watch` against the minimal-serve fixture, asserts
     exactly one "Rebuilding" log line arrives within 2s (D-19).
   - `r_key_in_no_watch_mode_triggers_one_rebuild` — spawns `ferro serve` with
     `FERRO_SERVE_TEST_TRIGGER_PIPE`, writes a line to the pipe, asserts a reload
     log line appears (D-06).
   - `non_tty_stdin_ignores_r_and_shows_banner` — spawns with redirected stdin,
     asserts banner renders without the keyboard thread being spawned (D-05, D-23).
   - `backend_only_shuts_down_cleanly` — spawns serve, sends SIGINT after the
     fixture boots, asserts graceful shutdown within 2s (D-29).

   Test helpers: `spawn_stdout_reader` (streams child stdout into mpsc for
   deadline-based `wait_for_stdout_line`), `send_sigint` (unix `libc::kill` with
   safety comment), `default_timeout` (5s).

## Gates

- `cargo build -p ferro-cli --tests` → exit 0
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` → exit 0
- `cargo test -p ferro-cli --test serve_supervisor` → **4 passed; 0 failed; 0 ignored**
- `cargo test -p ferro-cli --lib serve::tests` → **7 passed; 0 failed; 0 ignored** (unchanged)

## Decision Coverage

D-01 (no-watch default), D-05 (banner shape), D-06 (r-key reload), D-07 (keyboard
thread lifecycle), D-15 (mpsc triggers), D-19 (debouncer 500ms coalescing), D-20
(file-system watcher), D-21 (non-.rs filter), D-23 (is_tty gating), D-24 (raw mode),
D-26 (keyboard shutdown), D-29 (shutdown ordering within 2s budget), D-36
(integration-test surface).

## Key Files

- `ferro-cli/Cargo.toml` — dev-deps
- `ferro-cli/src/commands/serve.rs` — `FERRO_SERVE_TEST_TRIGGER_PIPE` hook
- `ferro-cli/tests/serve_supervisor.rs` — 4 integration tests + helpers (308 insertions)

## Notes

- Tests spawn the real `ferro` binary (release build via `assert_cmd::Command::cargo_bin`)
  so they exercise the actual supervisor wired in 02b.
- Fixture binary (`tests/fixtures/minimal-serve/`) prints "Backend server on …" then
  exits ~200ms later; after exit the supervisor sits idle (D-12, no auto-respawn).
  Tests observe this passive state; SIGINT exits the supervisor cleanly.
- Tests are serialized via a global `Mutex` guard (`TEST_LOCK`) matching the pattern
  in `docker_init_dry_run.rs` — they chdir and spawn real processes.
