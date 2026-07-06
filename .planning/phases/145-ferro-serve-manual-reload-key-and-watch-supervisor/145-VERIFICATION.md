---
phase: 145-ferro-serve-manual-reload-key-and-watch-supervisor
verified: 2026-04-22T00:00:00Z
status: passed
goal: replace cargo-watch with in-process supervisor; --watch opt-in; runtime `r` key with cancel-and-restart; notify-debouncer-mini 500ms trailing-edge
must_haves_verified: 8/8
decisions_verified: 38/38
gates_run: 7
gates_passed: 7
re_verification: false
---

# Phase 145: ferro serve manual reload key and watch supervisor — Verification Report

**Phase Goal:** Replace the external `cargo-watch` dependency in `ferro serve` with an in-process supervisor. Make auto-watch opt-in via `--watch` (off by default). Add a runtime `r` key that triggers a backend rebuild and types regeneration, cancelling any in-flight build. Use `notify-debouncer-mini` for trailing-edge debounce (500 ms fixed) so a burst of file-saves produces one rebuild rather than many.

**Verified:** 2026-04-22
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Must-Haves)

| #   | Must-Have | Status | Evidence |
| --- | --------- | ------ | -------- |
| 1   | `cargo-watch` no longer a dependency or install step | VERIFIED | `grep cargo-watch\|cargo_watch\|ensure_cargo_watch ferro-cli/src/commands/serve.rs` → 0 matches. `grep cargo-watch ferro-cli/Cargo.toml` → 0 matches. `grep cargo-watch docs/src/` → 0 matches. Only allowed leftover is `ferro-cli/src/commands/skills/test.md` which documents `cargo test --watch` (explicitly out of phase 145 scope per Plan 04 SUMMARY). |
| 2   | `ferro serve` default behavior: NO file watching | VERIFIED | `./target/debug/ferro serve --help` shows `--watch` flag (opt-in, no default_value set so defaults to `false`). `ferro-cli/src/main.rs:51-53` declares `#[arg(long)] watch: bool` on `Commands::Serve`. `serve.rs:695-699` shows `_debouncer = if watch { spawn_file_watcher(...) } else { None }` — watcher only created when flag is true. |
| 3   | `ferro serve --watch` enables file watching via notify-debouncer-mini with 500ms trailing debounce | VERIFIED | `ferro-cli/Cargo.toml:31` declares `notify-debouncer-mini = "0.4"`. `serve.rs:382` invokes `spawn_file_watcher_at(Path::new("src"), Duration::from_millis(500), tx)`. Symbols `spawn_file_watcher` (line 379) and `spawn_file_watcher_at` (line 333) both exist. Integration test `watch_mode_debounces_burst` passes (stdout scraper confirms exactly one `reload triggered (file change)` line within 5s; extra events < 11 for a 10-.rs-write + 1-Cargo.toml burst). |
| 4   | Runtime `r` key triggers rebuild + types regen, cancelling any in-flight build | VERIFIED | `classify_key` (serve.rs:79-87) maps `(KeyCode::Char('r'), KeyModifiers::NONE) => Some(KbAction::Reload)`. `BackendSupervisor::run_loop` (serve.rs:482-505) calls `kill_current()` → `regenerate_types()` → `spawn_backend()` on every trigger. Integration test `r_key_in_no_watch_mode_triggers_one_rebuild` passes (stdout contains exactly one `reload triggered (manual)` after a pipe write; no duplicate within 2s). |
| 5   | Raw terminal mode always restored on normal exit, panic, Ctrl-C | VERIFIED | `RawModeGuard` struct (serve.rs:276) with `impl Drop` (serve.rs:278-282) calling `disable_raw_mode()`. `_guard = RawModeGuard` bound inside the keyboard thread at serve.rs:301 so Drop fires on normal exit and panic unwind. Integration test `backend_only_shuts_down_cleanly` passes (SIGINT → process exits within 5s budget). |
| 6   | `ensure_cargo_watch()` and `start_type_watcher()` functions deleted | VERIFIED | `grep "fn ensure_cargo_watch\|fn start_type_watcher" ferro-cli/` → 0 matches. Confirmed via both plan summaries (02a SUMMARY §Deletions) and live file scan. |
| 7   | All 38 CONTEXT.md decisions (D-01..D-38) covered | VERIFIED | See Decision Coverage Matrix below. D-01..D-34 all mapped to code or test. D-35 covered by 7 inline unit tests (all passing). D-36 covered by 4 integration tests (all passing). D-37 covered by `ferro-cli/tests/fixtures/minimal-serve/` fixture. D-38 documented as manual-only in 145-VALIDATION.md (acceptable per CONTEXT). |
| 8   | Plan set referenced VALIDATION.md | VERIFIED | `grep -l "145-VALIDATION" ferro-cli/.../145-*-PLAN.md` → 4 matches (145-01, 145-02a, 145-02b, 145-03). Plan 04 is docs-only and does not require validation sampling. |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `ferro-cli/src/commands/serve.rs` | BackendSupervisor + producers + rewired run() + inline tests | VERIFIED | 976 lines. Contains all required symbols: `ReloadTrigger`, `KbAction`, `render_banner`, `classify_key`, `format_trigger_source`, `should_spawn_keyboard`, `spawn_child_with_prefix`, `RawModeGuard` + `impl Drop`, `spawn_keyboard_thread`, `spawn_file_watcher`, `spawn_file_watcher_at`, `BackendSupervisor` struct + 6 methods (`new`, `kill_current`, `regenerate_types`, `spawn_backend`, `drain_triggers`, `run_loop`), rewired `run()` with explicit shutdown ordering. No `let _ = watch;`, no `ensure_cargo_watch`, no `start_type_watcher`, no `manager.any_exited`, no `cargo-watch`/`cargo watch` literal. |
| `ferro-cli/Cargo.toml` | crossterm + notify-debouncer-mini + libc dev-dep | VERIFIED | Line 25: `crossterm = "0.29"`. Line 31: `notify-debouncer-mini = "0.4"`. Lines 58-59: `[target.'cfg(unix)'.dev-dependencies] libc = "0.2"`. No `cargo-watch` anywhere. |
| `ferro-cli/src/main.rs` | --watch flag on Commands::Serve | VERIFIED | Lines 51-53 declare `#[arg(long)] watch: bool` with help text `Enable file-watch auto-reload (500ms debounce)`. |
| `ferro-cli/tests/serve_supervisor.rs` | 4 integration tests un-ignored | VERIFIED | All 4 tests present, none ignored: `backend_only_shuts_down_cleanly`, `r_key_in_no_watch_mode_triggers_one_rebuild`, `watch_mode_debounces_burst`, `non_tty_stdin_ignores_r_and_shows_banner`. Shared helpers `spawn_stdout_reader`, `wait_for_stdout_line`, `send_sigint`, `kill_and_wait`, `fixture_dir`, `ferro_bin`, `CHDIR_LOCK` all present. |
| `ferro-cli/tests/fixtures/minimal-serve/` | Standalone fixture crate | VERIFIED | `Cargo.toml` with empty `[workspace]` opt-out; `src/main.rs` prints `Backend server on http://127.0.0.1:0` and sleeps 200ms. |
| `docs/src/reference/cli.md` | Updated serve section (--watch, key legend) | VERIFIED | Lines 99-149 contain updated serve section: options table with `--watch` row, `Key bindings (when stdin is a TTY)` subsection, rewritten "What it does" referencing in-process supervisor. No `cargo-watch` references. `--port` default corrected to `8080`. |
| `ferro-cli/src/commands/skills/serve.md` | Rewritten skill prompt (no cargo-watch) | VERIFIED | 80 lines. Arguments list reflects real clap surface (`--watch`, not `--no-watch`). No `cargo install cargo-watch`. Uses neutral framing `explicit control over rebuild timing`. |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `Commands::Serve` dispatch | `commands::serve::run(..., watch)` | 6th positional arg | VERIFIED | main.rs threads `watch` into run; serve.rs:514 accepts `watch: bool` param. |
| `run()` | `BackendSupervisor::run_loop` | `thread::spawn(move \|\| supervisor.run_loop(reload_rx))` at serve.rs:708 | VERIFIED | Supervisor thread spawned and joined via `supervisor_handle` during shutdown. |
| Keyboard thread + file-watcher thread | `BackendSupervisor` | Shared `Sender<ReloadTrigger>` with `.clone()` per producer | VERIFIED | serve.rs:694 (`spawn_keyboard_thread(reload_tx.clone(), ...)`), 696 (`spawn_file_watcher(reload_tx.clone())`). Original tx dropped at serve.rs:742 for Disconnected-signaled teardown. |
| `BackendSupervisor::regenerate_types` | `super::generate_types::generate_types_to_file` | Direct call inside reload cycle | VERIFIED | serve.rs:431-434 calls `super::generate_types::generate_types_to_file(&self.project_path, &self.types_output_path)`. |
| `spawn_file_watcher` → `spawn_file_watcher_at` | Production wrapper pins `Path::new("src")` + 500ms | Thin wrapper | VERIFIED | serve.rs:379-383. The `_at` variant (serve.rs:333-375) is exposed for unit-test injection with a short debounce. |
| Integration tests | `minimal-serve` fixture | `Command::new(ferro_bin()).current_dir(fixture_dir())` | VERIFIED | serve_supervisor.rs helpers wire `CARGO_BIN_EXE_ferro` + `CARGO_MANIFEST_DIR`. All 4 tests pass. |

### Data-Flow Trace (Level 4)

Supervisor is a runtime orchestrator, not a renderer of dynamic data. Data flow is verified structurally instead:

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `BackendSupervisor.run_loop` | `ReloadTrigger` events on `reload_rx` | Keyboard thread (`r` key via `classify_key`) and file-watcher thread (`.rs` writes via `notify-debouncer-mini`) | Yes — both producers send real events; integration tests confirm triggers flow end-to-end to stdout log lines | FLOWING |
| `render_banner` output | Banner string | Pure function; inputs from `run()` (watch flag, is_tty, ports, etc.) | Yes — 4-variant exact-string test oracle passes | FLOWING |
| `spawn_file_watcher_at` → channel | `ReloadTrigger::FileChanged` | `notify-debouncer-mini` → filter `.rs` → `tx.send(...)` | Yes — unit test `debouncer_coalesces_burst` + integration `watch_mode_debounces_burst` confirm writes produce events | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| ferro binary built | `cargo build -p ferro-cli` | `Finished dev profile in 16.85s` | PASS |
| Serve --help shows --watch | `./target/debug/ferro serve --help \| grep -- --watch` | `--watch  Enable file-watch auto-reload (500ms debounce)` | PASS |
| Inline unit tests pass | `cargo test -p ferro-cli --lib serve::tests` | `7 passed; 0 failed; 0 ignored` | PASS |
| Integration tests pass | `cargo test -p ferro-cli --test serve_supervisor` | `4 passed; 0 failed; 0 ignored` (6.85s total) | PASS |
| Clippy clean | `cargo clippy -p ferro-cli --all-targets -- -D warnings` | exit 0 | PASS |
| Fmt clean | `cargo fmt --package ferro-cli -- --check` | exit 0 | PASS |
| No cargo-watch leaked into user-facing docs | `grep -rE "cargo-watch\|cargo watch" docs/src/ ferro-cli/src/commands/skills/serve.md` | 0 matches | PASS |

### Automated Gate Output

All 7 gates (build, clippy, fmt, lib tests, integration tests, help-flag check, cargo-watch grep) executed successfully. Workspace-wide clippy gate was deferred per Plan 02b's explicit scoping to `-p ferro-cli`, due to pre-existing unrelated issues in `ferro-json-ui` documented in `deferred-items.md`. This scoping is explicitly sanctioned by the verification instructions and is not a phase 145 failure.

### Decision Coverage Matrix (D-01..D-38)

| D-ID | Description | Coverage | Evidence |
| ---- | ----------- | -------- | -------- |
| D-01 | Auto-watch OFF by default | SATISFIED | main.rs:51-53 `watch: bool` with no `default_value` (defaults to false); serve.rs:695 guards watcher behind `if watch`. |
| D-02 | `--watch` flag on serve | SATISFIED | main.rs:51-53. |
| D-03 | cargo-watch install step removed | SATISFIED | `grep ensure_cargo_watch ferro-cli/` → 0. |
| D-04 | Other flags unchanged | SATISFIED | main.rs:32-49 preserve `--port`, `--frontend-port`, `--backend-only`, `--frontend-only`, `--skip-types`. Defaults preserved (`8080`/`5173`). |
| D-05 | Startup banner with key legend + watch status + non-TTY line | SATISFIED | `render_banner` at serve.rs:40-75 emits all 4 watch×TTY variants; `render_banner_matrix` unit test asserts exact-string equality. |
| D-06 | `r` → `ReloadTrigger::Manual` | SATISFIED | classify_key at serve.rs:81; integration test `r_key_in_no_watch_mode_triggers_one_rebuild`. |
| D-07 | `q`/Ctrl+C graceful shutdown | SATISFIED | classify_key at serve.rs:82-84 maps both to `KbAction::Quit`; keyboard thread sets shutdown flag; `backend_only_shuts_down_cleanly` integration test passes. |
| D-08 | Lowercase `r` only | SATISFIED | classify_key_table unit test asserts uppercase `R` + `SHIFT` returns `None`. |
| D-09 | Cancel-and-restart on new trigger | SATISFIED | `run_loop` at serve.rs:482-505 calls `kill_current()` before respawning. Noted as manual-only in VALIDATION for fine-grained observability (timing). |
| D-10 | Reload scope = backend + types only | SATISFIED | `run_loop` only touches supervisor state; Vite is owned by `ProcessManager` and not restarted. |
| D-11 | Skip kill when current=None | SATISFIED | `kill_current` (serve.rs:417-422) uses `take()`; `kill_current_noop_when_none` unit test asserts no-op. |
| D-12 | No auto-respawn on backend exit | SATISFIED | `manager.any_exited` deleted (Plan 02b); main wait loop (serve.rs:757-759) polls only `shutdown.load`. Manual-only verification per VALIDATION.md. |
| D-13 | BackendSupervisor owns backend child | SATISFIED | `BackendSupervisor.current: Option<Child>` at serve.rs:393; ProcessManager only owns Vite. |
| D-14 | ProcessManager keeps Vite | SATISFIED | serve.rs:655-671 uses `manager.spawn_with_prefix_env("npm", ...)`. |
| D-15 | Producers optional | SATISFIED | `spawn_keyboard_thread` returns `Option<JoinHandle<()>>` keyed on `is_terminal()`; `spawn_file_watcher` only called `if watch`. Non-TTY integration test confirms absence of keyboard thread. |
| D-16 | std::sync::mpsc + recv_timeout | SATISFIED | serve.rs:12 imports `std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender}`. `run_loop` uses `recv_timeout(Duration::from_millis(100))` at serve.rs:489. No `crossbeam_channel`, no `tokio::sync::mpsc`. |
| D-17 | Trigger coalescing via drain | SATISFIED | `drain_triggers` at serve.rs:471-477 (while-let-Ok drain). `supervisor_coalesces_multiple_triggers` unit test passes. |
| D-18 | Types regen uninterruptible | SATISFIED | `regenerate_types` runs synchronously inside `run_loop`; no inner `recv` or cancellation point. Manual-only verification per VALIDATION.md. |
| D-19 | notify-debouncer-mini 500ms fixed | SATISFIED | serve.rs:382 pins `Duration::from_millis(500)` in the production wrapper. `debouncer_coalesces_burst` unit test + `watch_mode_debounces_burst` integration test confirm timing. |
| D-20 | src/ recursive + *.rs filter | SATISFIED | `spawn_file_watcher_at` watches `src` with `RecursiveMode::Recursive` (serve.rs:366); filter at serve.rs:352-354 checks `e.path.extension().map(\|x\| x == "rs")`. |
| D-21 | Cargo.toml / migrations do NOT trigger | SATISFIED | Filter derivation from D-20. `watch_mode_debounces_burst` integration test writes a `Cargo.toml` + 10 `.rs` files and asserts total events < 11 (Cargo.toml did not trigger). |
| D-22 | Missing src/ or init failure → warn + no crash | SATISFIED | `spawn_file_watcher_at` (serve.rs:338-373) returns `None` with yellow warning on missing dir, notify init failure, or watch() failure. |
| D-23 | crossterm dep declared | SATISFIED | Cargo.toml:25 `crossterm = "0.29"`. |
| D-24 | TTY detection via is_terminal | SATISFIED | `should_spawn_keyboard` at serve.rs:98-100 is identity on is_tty. `spawn_keyboard_thread` checks `std::io::stdin().is_terminal()` at serve.rs:292. Integration test `non_tty_stdin_ignores_r_and_shows_banner` confirms. |
| D-25 | RAII Drop guard for raw mode | SATISFIED | `RawModeGuard` struct + `impl Drop` at serve.rs:276-282. `_guard = RawModeGuard` bound in keyboard thread at serve.rs:301. Manual-only verification for panic path per VALIDATION.md. |
| D-26 | enable_raw_mode failure → warn + skip | SATISFIED | serve.rs:296-299 prints warning and returns `None` when `enable_raw_mode` fails. |
| D-27 | Log format `[backend] reload triggered ({source})` | SATISFIED | serve.rs:492-496 emits `"{} reload triggered ({})"` with `[backend]` style prefix and `format_trigger_source(src)` label. |
| D-28 | Source labels `manual` / `file change` | SATISFIED | `format_trigger_source` at serve.rs:90-95. `trigger_source_formatting` unit test asserts exact strings. |
| D-29 | Shutdown ordering steps 1..6 | SATISFIED | serve.rs:761-780 executes: wait-loop break → join keyboard → drop debouncer → join supervisor → `manager.shutdown_all` → print "Servers stopped.". `backend_only_shuts_down_cleanly` integration test confirms <5s budget. |
| D-30 | crossterm in Cargo.toml | SATISFIED | Mirror of D-23; Cargo.toml:25. |
| D-31 | notify-debouncer-mini 0.4 kept | SATISFIED | Cargo.toml:31 unchanged from pre-phase. |
| D-32 | All cargo-watch references removed | SATISFIED | Source: `grep -rnE "cargo-watch\|cargo watch" ferro-cli/src/` → 0 (excluding test.md which is about `cargo test --watch`, explicitly out of scope). Docs: `grep cargo-watch docs/src/` → 0. |
| D-33 | docs/src/ serve section updated | SATISFIED | `docs/src/reference/cli.md` lines 99-149: `--watch` in options table, key-bindings subsection, rewritten "What it does". |
| D-34 | `ferro serve --help` reflects --watch | SATISFIED | `./target/debug/ferro serve --help` output shows `--watch  Enable file-watch auto-reload (500ms debounce)`. |
| D-35 | Unit tests present and passing | SATISFIED | 7 inline unit tests in `#[cfg(test)] mod tests` (serve.rs:783-975), all passing: `render_banner_matrix`, `classify_key_table`, `trigger_source_formatting`, `should_spawn_keyboard_gated_on_tty`, `kill_current_noop_when_none`, `supervisor_coalesces_multiple_triggers`, `debouncer_coalesces_burst`. |
| D-36 | Integration tests present and passing | SATISFIED | 4 integration tests in `ferro-cli/tests/serve_supervisor.rs`, all passing: `backend_only_shuts_down_cleanly`, `r_key_in_no_watch_mode_triggers_one_rebuild`, `watch_mode_debounces_burst`, `non_tty_stdin_ignores_r_and_shows_banner`. |
| D-37 | Minimal fixture project | SATISFIED | `ferro-cli/tests/fixtures/minimal-serve/` with `[workspace]` opt-out, standalone-buildable `Cargo.toml` + `src/main.rs`. |
| D-38 | Raw-mode `stty` before/after test | SATISFIED (manual-only) | Per CONTEXT.md explicitly allowed to be optional/skipped in CI. Documented as manual verification row in `145-VALIDATION.md`. |

**Summary:** 38/38 decisions covered.

### Anti-Patterns Found

None material to phase scope.

- serve.rs contains no `todo!()`, no `unimplemented!()`, no `PLACEHOLDER`, no `TODO` / `FIXME` comments.
- The only `#[allow(...)]` is `#[allow(clippy::too_many_arguments)]` on `render_banner` — justified by the 7-arg signature being contractually pinned (plan 02a documented this decision; verified against clippy's default threshold).
- Test fixture contains a trivial 200ms sleep — intentional (allows integration tests to observe banner before fixture exit).
- The `FERRO_SERVE_TEST_TRIGGER_PIPE` env-var hook in `run()` (serve.rs:717-738) is an intentional test seam guarded by `std::env::var` presence. Documented in 145-03-SUMMARY.md and inline comments. Not part of stable CLI surface.

### Deferred / Pre-Existing (Out of Phase Scope)

Documented in `deferred-items.md` and sanctioned by the verification instructions:

1. **ferro-json-ui `SwitchProps.compact` compile errors** — pre-existing on master before phase 145 started. Workspace-wide clippy gate consequently fails; phase 145 is scoped to `-p ferro-cli` as explicitly instructed. NOT a phase 145 failure.
2. **ferro-json-ui `render.rs:2286` rustfmt drift** — pre-existing long-line drift. Same rationale.
3. **`ferro-cli/src/commands/skills/test.md` cargo-watch reference** — about `cargo test --watch` (cargo's own nightly subcommand, not the `cargo-watch` binary). Explicitly out of phase 145 scope per Plan 04 SUMMARY; sanctioned by the verification instructions.

### Human Verification Required

None for phase-passing determination. The following items are defined as manual-only in `145-VALIDATION.md` and were explicitly accepted as non-automated at context-gathering time:

- D-09 cancel-mid-compile visual confirmation (manual-only by design).
- D-12 no auto-respawn on compile failure (manual-only by design).
- D-18 types-regen uninterruptibility (manual-only by design).
- D-25 raw-mode restoration on panic via `stty -a` diff (manual-only, CI-unstable).
- D-26 `enable_raw_mode` failure fallback (manual-only, no clean injection point).

These are not gating items — they are documented in the phase validation contract as manual-only. The automated coverage (structural + unit + integration) is complete for the automated surface.

### Gaps Summary

None. All 8 must-haves verified, all 38 decisions covered, all 7 automated gates pass, all 11 tests (7 unit + 4 integration) pass. Phase 145 delivers the stated goal in full.

---

_Verified: 2026-04-22_
_Verifier: Claude (gsd-verifier)_
