# Phase 145: ferro serve manual reload key and watch supervisor - Context

**Gathered:** 2026-04-22
**Status:** Ready for planning
**Mode:** `--auto` (single-pass, recommended defaults selected for remaining gray areas)

<domain>
## Phase Boundary

Replace the external `cargo-watch` dependency in `ferro serve` with an in-process supervisor. Flip auto-watch to opt-in via `--watch` (off by default). Add a runtime `r` key that triggers a backend rebuild and types regeneration, cancelling any in-flight build. Use `notify-debouncer-mini` for trailing-edge debounce (500 ms fixed) so a burst of file-saves produces one rebuild rather than many.

**Scope target:** `ferro-cli/src/commands/serve.rs` and its dependency manifest (`ferro-cli/Cargo.toml`).
**Deletes:** `ensure_cargo_watch()` (serve.rs:148) and `start_type_watcher()` (serve.rs:425).
**Out of scope:** hot reload without process restart, frontend/Vite lifecycle changes, configurable debounce window, auto-respawn on compile failure.

</domain>

<decisions>
## Implementation Decisions

### CLI surface
- **D-01:** Auto-watch is OFF by default. `ferro serve` runs without a file watcher.
- **D-02:** New `--watch` flag opts into file-watch auto-reload (trailing-edge debounced).
- **D-03:** External `cargo-watch` install step is removed entirely (`ensure_cargo_watch()` deleted).
- **D-04:** All other flags (`--skip-types`, `--backend-only`, `--frontend-only`, `--port`, `--frontend-port`) unchanged.
- **D-05:** Startup banner documents the key legend (`r`, `q`/Ctrl+C) and watch status. Non-TTY stdin shows `r unavailable (non-TTY stdin)`. With `--watch`, last line reads `watch enabled (debounce 500ms)`.

### Runtime keys
- **D-06:** `r` key triggers `ReloadTrigger::Manual` — works in both `--watch` and no-watch modes.
- **D-07:** `q` or `Ctrl+C` → graceful shutdown.
- **D-08:** Lowercase `r` only for v1 (uppercase `R` ignored, treated as "any other key"). Keep surface minimal; revisit if field reports ask for it.

### Reload semantics
- **D-09:** A new trigger arriving while a build is in flight **cancels and restarts** (kill current backend child, then regen types, then respawn). Mental model: "I want my latest changes now" — queueing would subvert that.
- **D-10:** Scope of a reload = backend recompile + types regeneration, unified. Frontend/Vite is never restarted by `r` or file events.
- **D-11:** If a trigger arrives while no backend child is live (e.g. prior compile failed), skip kill, go straight to regen + spawn.
- **D-12:** No auto-respawn after non-zero exit. Backend child exits → supervisor logs exit code and waits for the next trigger (matches current `cargo-watch` behavior).

### Supervisor architecture
- **D-13:** A dedicated `BackendSupervisor` thread owns the backend `cargo run` child exclusively (not `ProcessManager`).
- **D-14:** `ProcessManager` continues to own the Vite child. `manager.shutdown_all()` still kills Vite at the end of the shutdown ordering.
- **D-15:** Reload producers (keyboard thread + file-watcher thread) feed a shared channel consumed by the supervisor. Both producers are optional: keyboard thread only if stdin is a TTY; file-watcher thread only if `--watch` is passed.
- **D-16:** **Channel primitive: `std::sync::mpsc`** (already idiomatic in `serve.rs`; avoids a new dep). The supervisor uses `recv_timeout` in its loop to interleave trigger handling with shutdown polling. No `crossbeam-channel` `select!` needed.
- **D-17:** **Trigger coalescing:** at the start of each supervisor cycle, drain any additional pending triggers from the channel (non-blocking `try_recv` loop) before starting the kill/regen/spawn sequence. Avoids stacking multiple reloads when `r` is mashed or a large burst gets debounced into multiple events close together.
- **D-18:** **Types regen is not interruptible.** Only the `cargo run` child is killable. Types regen runs to completion each cycle; any trigger arriving during regen is picked up on the next loop iteration (via D-17 drain).

### Debounced file watcher (`--watch` only)
- **D-19:** Use `notify-debouncer-mini` (already in `ferro-cli/Cargo.toml`) with a fixed 500 ms window. Trailing-edge debouncing: one trigger fires after the burst settles.
- **D-20:** Watch target: `src/` recursive; filter to `*.rs` files; emit `ReloadTrigger::FileChanged`.
- **D-21:** Scope stays `src/` only for v1. `Cargo.toml`, migrations, and config files do NOT trigger reload. Revisit only on field-report signal.
- **D-22:** If `src/` is missing or `notify` init fails: log warning, skip file watcher, `--watch` becomes an effective no-op (serve continues without crashing).

### Keyboard thread
- **D-23:** Add `crossterm` dependency (latest stable — planner to pin exact version during implementation). Raw mode reads one key at a time.
- **D-24:** TTY detection via `std::io::stdin().is_terminal()`. If not a TTY, the thread is not spawned.
- **D-25:** RAII `Drop` guard ensures `disable_raw_mode()` runs even on panic / Ctrl+C so the terminal is never left in raw mode.
- **D-26:** If `enable_raw_mode()` fails → log warning, skip keyboard thread, serve continues.

### Output / logging
- **D-27:** On each trigger, log a single line: `[backend] reload triggered ({source})` where `{source}` is `manual` or `file change`. Startup banner is printed once at startup only — not re-rendered on each reload.
- **D-28:** Trigger source formatting is fixed at code level (not a user-facing option).

### Shutdown ordering (deterministic)
- **D-29:** Shutdown sequence:
  1. Ctrl+C handler or `q` key sets `shutdown = true`.
  2. Main thread breaks its wait loop.
  3. Supervisor's shutdown channel fires; supervisor kills its backend child and exits.
  4. Keyboard thread's `Drop` guard runs; raw mode disabled.
  5. `ProcessManager::shutdown_all()` kills Vite.
  6. "Servers stopped." printed.

### Dependencies
- **D-30:** Add `crossterm` to `ferro-cli/Cargo.toml`.
- **D-31:** `notify-debouncer-mini = "0.4"` is already present — keep the existing version; no bump.
- **D-32:** Remove all references to installing `cargo-watch` in docs and CLI.

### Docs
- **D-33:** Update `docs/src/` serve section: replace cargo-watch content with `--watch` + `r`-key model, including the key legend.
- **D-34:** Update clap annotations on the `serve` subcommand so `ferro serve --help` reflects the new `--watch` flag and default behavior.

### Testing
- **D-35:** Unit tests in `ferro-cli/src/commands/serve.rs`:
  - `BackendSupervisor::kill_current` is a no-op when `current = None`.
  - Debouncer coalesces N burst events into 1 trigger within the window (via debouncer test harness).
  - `render_banner(opts)` renders correct text for each `--watch` × TTY combination.
  - Trigger source formatting: `Manual` vs `FileChanged`.
- **D-36:** Integration tests in `ferro-cli/tests/`:
  - `ferro serve --backend-only` starts and shuts down cleanly on SIGINT within 2s with no zombie children.
  - `r` in no-watch mode triggers exactly one rebuild.
  - `--watch` burst of 10 writes in 100 ms → exactly one rebuild after the debounce window.
  - Non-TTY stdin: banner shows `r unavailable`, stdin bytes ignored, no crash.
- **D-37:** Minimal fixture project under `ferro-cli/tests/fixtures/minimal-serve/`, compiled once.
- **D-38:** Raw-mode restoration test (`stty` before/after) is optional; may be skipped in CI if it flakes on GitHub runners.

### Claude's Discretion
- Exact crossterm version pin (latest stable at implementation time).
- Internal struct layout of `BackendSupervisor` fields beyond those named in the spec (spec gives a skeleton — planner can extend as needed).
- Whether to split new supervisor/keyboard/watcher code across submodules of `serve.rs` or keep inline (prefer inline unless the file grows past ~800 lines).
- Exact error-message phrasing for log lines (keep neutral; match existing `serve.rs` voice).
- Fixture project contents (just enough to let `cargo run` complete in under a second).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec (primary)
- `docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md` — the approved design spec. Every decision above derives from this document. Planner and researcher MUST read it end-to-end before generating tasks.

### Current implementation target
- `ferro-cli/src/commands/serve.rs` — file being modified. Key regions: `ProcessManager` struct (lines 14–146), `ensure_cargo_watch()` (148–171, to be deleted), main `run()` function with `--watch` implicit behavior (roughly 317–420), `start_type_watcher()` (425–504, to be folded into supervisor).
- `ferro-cli/Cargo.toml` — dependency manifest. `notify-debouncer-mini = "0.4"` already present; `crossterm` must be added.

### Dependency docs (fetch during research, not hand-typed)
- `notify-debouncer-mini` crate docs (trailing-edge debouncer, `new_debouncer` API). Use context7 `resolve-library-id` + `query-docs` during research rather than guessing from memory.
- `crossterm` raw-mode docs (`terminal::enable_raw_mode`, `event::read`, `event::KeyCode`, `is_terminal` via `std::io::IsTerminal`). Same — fetch from context7.

### Project conventions
- `CLAUDE.md` — workspace conventions. Relevant: "Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` before every commit." Every test-writing plan must respect this. No co-author lines.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ProcessManager` / `ProcessManager::spawn_with_prefix` (`ferro-cli/src/commands/serve.rs:27–96`) — the stdout/stderr piping with colored prefix is exactly the pattern the new `BackendSupervisor::spawn_backend()` should reuse. Extract the piping logic into a shared helper rather than duplicating it.
- `ProcessManager::shutdown_all` (line 98) — continues to manage Vite; shutdown ordering step 5 already present.
- `notify-debouncer-mini` crate (Cargo.toml line 30) — dep already present; no vendoring work required.
- `ctrlc` (line 25) — existing Ctrl+C handler wiring can remain; supervisor just reads the same shutdown flag.

### Established Patterns
- `std::sync::mpsc::channel` (imported at line 9) + `Arc<AtomicBool>` shutdown flag (line 8) is already the concurrency idiom in `serve.rs`. Stay consistent — do NOT introduce `crossbeam-channel` or `tokio::sync::mpsc`.
- Thread-per-producer pattern (`thread::spawn` line 11) is idiomatic here. Keyboard thread + file-watcher thread follow the same pattern.
- Colored stdout prefixing via `console::style(...).fg(color).bold()` — new log lines should match.

### Integration Points
- `run()` in `serve.rs` orchestrates the whole flow. The new `BackendSupervisor` replaces the cargo-watch spawn + type-watcher thread pair; it sits between the existing Vite management and the existing Ctrl+C / shutdown wiring.
- The CLI enum in `ferro-cli/src/commands/` or the clap definition file (planner to locate) needs a new `--watch` flag on the `serve` subcommand.

</code_context>

<specifics>
## Specific Ideas

- Motivation is a real field report: thermally-constrained hardware (MacBook) suffers from compounding rebuilds. The 500 ms trailing-edge debounce and the cancel-and-restart semantic on `r` directly address that.
- The key legend format in the banner is explicit in the spec — use it verbatim rather than reinventing.
- Raw-mode hygiene is load-bearing: a broken terminal after a crash is the #1 failure mode for TUI-adjacent tools. The `Drop` guard is not optional.

</specifics>

<deferred>
## Deferred Ideas

- **Auto-respawn on compile failure.** Explicitly named as a non-goal in the spec. If demand appears later, add as a separate phase with a configurable retry policy.
- **Configurable debounce window.** Fixed at 500 ms. Revisit only if a real use case appears.
- **Hot reload without process restart.** Out of scope for a Rust framework.
- **Watching `Cargo.toml`, migrations, or non-Rust files.** Stays out of scope; planner should NOT expand `src/` recursive to cover these.
- **Uppercase `R` or modifier-key bindings.** Deferred; lowercase `r` only for v1.
- **Re-rendering the banner after each reload.** Deferred; single startup print only.
- **Per-run debounce-window override via env var.** Deferred; not adding configuration surface without demand.

</deferred>

---

*Phase: 145-ferro-serve-manual-reload-key-and-watch-supervisor*
*Context gathered: 2026-04-22*
