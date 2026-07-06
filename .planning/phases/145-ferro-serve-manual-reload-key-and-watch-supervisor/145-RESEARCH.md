# Phase 145: ferro serve manual reload key and watch supervisor - Research

**Researched:** 2026-04-22
**Domain:** Rust CLI process supervision, filesystem debouncing, cross-platform raw-mode TTY
**Confidence:** HIGH (locked decisions in CONTEXT.md; dependency APIs verified via docs.rs)

## Summary

This phase replaces the external `cargo-watch` dependency in `ferro serve` with an in-process supervisor thread that owns the backend `cargo run` child. Auto-watch becomes opt-in via a new `--watch` flag. A lowercase `r` key triggers a cancel-and-restart reload (kill child, regen types, respawn) via `crossterm` raw-mode stdin. The file watcher (only active under `--watch`) uses `notify-debouncer-mini` 0.4.1 with a fixed 500 ms trailing-edge debounce window.

The phase is constrained by 38 locked decisions in `145-CONTEXT.md` and a fully-approved design spec at `docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md`. Research effort therefore focuses on: (1) the exact API surface of the two new dependencies, (2) line-accurate delete ranges in `serve.rs`, (3) reuse of existing patterns, and (4) a validation architecture that maps each decision to a concrete unit/integration/manual test.

**Primary recommendation:** Model the new code as three threads over `std::sync::mpsc` — a `BackendSupervisor` loop (`recv_timeout` + shutdown poll + trigger drain), a `notify-debouncer-mini` producer (only under `--watch`), and a `crossterm` keyboard producer (only when stdin is a TTY). Keep the implementation inline in `serve.rs` unless the file exceeds ~800 lines. Reuse `ProcessManager::spawn_with_prefix`'s piping pattern for the backend child. Do not introduce `crossbeam-channel`, `tokio::sync::mpsc`, or `notify-debouncer-full`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

All 38 decisions (D-01 through D-38) in `.planning/phases/145-ferro-serve-manual-reload-key-and-watch-supervisor/145-CONTEXT.md` are locked. Research treats them as inputs, not alternatives. Summary by category:

**CLI surface** — D-01..D-05: auto-watch OFF by default; `--watch` opts in; `ensure_cargo_watch()` deleted; other flags unchanged; banner documents `r`, `q`/Ctrl+C, watch status, with `r unavailable (non-TTY stdin)` in non-TTY mode.

**Runtime keys** — D-06..D-08: lowercase `r` = `ReloadTrigger::Manual` in both modes; `q` or Ctrl+C = graceful shutdown; uppercase `R` ignored.

**Reload semantics** — D-09..D-12: new trigger cancels in-flight build; scope = backend + types together; if no child live, skip kill; no auto-respawn on non-zero exit.

**Supervisor architecture** — D-13..D-18: dedicated `BackendSupervisor` thread owns backend child; `ProcessManager` keeps Vite; producers feed shared channel; use `std::sync::mpsc` with `recv_timeout` (NOT `crossbeam-channel`); drain pending triggers at cycle start via `try_recv`; types regen is uninterruptible.

**Debounced file watcher** — D-19..D-22: `notify-debouncer-mini = "0.4"` (already present); fixed 500 ms window; watch `src/` recursive, `*.rs` filter; `src/` missing or init failure → skip watcher, serve continues.

**Keyboard thread** — D-23..D-26: add `crossterm` (latest stable); `std::io::stdin().is_terminal()` for TTY detection; RAII `Drop` guard restores raw mode on panic; `enable_raw_mode()` failure → skip keyboard thread.

**Output / logging** — D-27..D-28: each trigger logs one line `[backend] reload triggered ({source})` where source ∈ {`manual`, `file change`}; banner printed once at startup only.

**Shutdown ordering** — D-29: (1) handler sets shutdown; (2) main breaks wait loop; (3) supervisor kills child, exits; (4) keyboard `Drop` guard disables raw mode; (5) `ProcessManager::shutdown_all()` kills Vite; (6) "Servers stopped." printed.

**Dependencies** — D-30..D-32: add `crossterm`; keep `notify-debouncer-mini = "0.4"`; remove all cargo-watch references.

**Docs** — D-33..D-34: update `docs/src/` serve section; update clap annotations so `ferro serve --help` reflects `--watch` and key legend.

**Testing** — D-35..D-38: four unit tests (see Validation Architecture); four integration tests; minimal fixture under `ferro-cli/tests/fixtures/minimal-serve/`; raw-mode restoration test is optional (may be skipped in CI).

### Claude's Discretion

- Exact crossterm version pin (latest stable at implementation time — verified as **0.29.0**, published 2025-04-05, see Standard Stack).
- Internal struct layout of `BackendSupervisor` fields beyond the named minimum from the spec.
- Whether to split new supervisor/keyboard/watcher code across submodules or keep inline (prefer inline unless file exceeds ~800 lines — current file is 437 lines, so the added code is likely to keep it under the threshold).
- Exact error-message phrasing for log lines (keep neutral; match existing `serve.rs` voice).
- Fixture project contents (just enough to let `cargo run` complete in under a second).

### Deferred Ideas (OUT OF SCOPE)

- Auto-respawn on compile failure.
- Configurable debounce window (fixed at 500 ms).
- Hot reload without process restart.
- Watching `Cargo.toml`, migrations, or non-Rust files.
- Uppercase `R` or modifier-key bindings.
- Re-rendering the banner after each reload.
- Per-run debounce-window override via env var.

</user_constraints>

## Project Constraints (from CLAUDE.md)

Directly relevant to this phase:

- **Run the full pre-commit triad before every commit** — `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`. CI enforces `-D warnings`; any warning is a build failure. The `--all-targets` flag is required to pick up test-code issues.
- **No co-author lines in commits.**
- **Prefer editing existing files** over creating new ones. The supervisor/keyboard/watcher code stays in `serve.rs` unless it grows past ~800 lines.
- **Delete old code completely** — no deprecation, no versioned names. Delete `ensure_cargo_watch()` and `start_type_watcher()` fully; do not leave them behind commented out.
- **Document updates required** — `docs/src/` must reflect current features. Any cargo-watch reference must be removed or replaced.
- **Update ferro-mcp when needed** — the phase changes CLI surface (`--watch`, runtime `r` key), so if `ferro-mcp` advertises serve flags or commands, that advertisement must be checked for staleness.
- **Concrete types, not `interface{}` / trait objects** (applies to Go in the original rule; the Rust analogue is: prefer concrete structs over dyn-trait boxes when there is a single implementation — the `BackendSupervisor` should be a concrete struct, not a trait).
- **Channels over time.Sleep** — aligns with D-16 (`recv_timeout` in supervisor loop, not polling sleep).
- **Early returns to reduce nesting.**
- **`fmt::Error` chain preservation** — the Rust idiom is `.map_err(|e| format!("context: {e}"))` or `thiserror` — match existing `serve.rs` which uses `String` errors at the CLI surface.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Child process lifecycle (backend) | CLI orchestrator (`BackendSupervisor`) | — | Replaces external `cargo-watch`; ferro owns `std::process::Child` directly. |
| Child process lifecycle (Vite) | CLI orchestrator (`ProcessManager`) | — | Unchanged; Vite already owned in-process. Shutdown ordering step 5. |
| Filesystem watching | CLI orchestrator (`notify-debouncer-mini` producer thread) | — | Pure dev-tool capability; no API/DB/browser tier involvement. |
| TTY raw-mode keystroke capture | CLI orchestrator (`crossterm` producer thread) | — | Dev-time UX affordance; runs only when stdin is a TTY. |
| Type regeneration | CLI internal (`super::generate_types`) | — | Pre-existing; invoked synchronously by the supervisor on each reload cycle. |
| Inter-thread signaling | `std::sync::mpsc` + `Arc<AtomicBool>` | — | Existing idiom in `serve.rs`; do not introduce `crossbeam-channel` or `tokio::sync::mpsc`. |

## Phase Requirements

This phase has **no REQ-IDs in REQUIREMENTS.md** (the file was not found in the repo). The **design spec** `docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md` is the source of truth, and every locked decision in `145-CONTEXT.md` derives from it. The Validation Architecture below maps decisions (D-01..D-38) to concrete tests since no REQ-IDs exist to map against.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `notify-debouncer-mini` | `0.4.1` (already in `Cargo.lock`) | Trailing-edge debouncer on top of `notify` | The "mini" crate in the `notify-rs` family; lightweight debouncing without rename-correlation overhead (the `-full` variant adds rename handling Ferro does not need). [VERIFIED: Cargo.lock line "notify-debouncer-mini\nversion = \"0.4.1\""] |
| `notify` | `6.x` (already in `Cargo.toml`) | Underlying cross-platform FS watcher | Paired with debouncer-mini; no version bump needed. [VERIFIED: ferro-cli/Cargo.toml line 29] |
| `crossterm` | `0.29.0` (latest stable, published 2025-04-05) | Cross-platform raw-mode stdin + key events | The de-facto Rust library for TUI-adjacent raw-mode I/O; pure Rust, MIT-licensed. [VERIFIED: crates.io API — newest_version=0.29.0, updated_at=2025-04-05T15:21:48.500601Z] |

### Supporting (already present, no changes)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `console` | `0.15` | Colored stdout prefixing | Match the existing `style(...).fg(color).bold()` idiom for any new log lines. |
| `ctrlc` | `3.5` | Ctrl+C signal handler | Keep the existing handler wiring; supervisor just reads the same `Arc<AtomicBool>` shutdown flag. |
| `std::sync::mpsc` | stdlib | Channel primitive | Already imported at `serve.rs:9`. Use `recv_timeout` + `try_recv` per D-16/D-17. |
| `std::io::IsTerminal` | stdlib (stable since 1.70) | TTY detection | `std::io::stdin().is_terminal()` per D-24. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff | Verdict |
|------------|-----------|----------|---------|
| `notify-debouncer-mini` | `notify-debouncer-full` | `-full` merges rename pairs and suppresses duplicate create/modify events; heavier; pulls in file-ID cache. | **REJECTED** by D-19. Ferro only needs trailing-edge coalescing of `*.rs` writes; rename correlation is not required. |
| `std::sync::mpsc` | `crossbeam-channel` with `select!` | `select!` macro is ergonomic for multi-channel wait; adds a dep. | **REJECTED** by D-16. The same behavior is achievable with `recv_timeout` on the trigger channel and polling `Arc<AtomicBool>` for shutdown — matches the existing idiom. |
| `crossterm` | `termion` (Unix-only) | Smaller, but Unix-only; ferro-cli ships on Windows. | **REJECTED** implicitly by cross-platform requirement; D-23 names crossterm. |
| Inline modules (single file) | Split into `serve/supervisor.rs`, `serve/keyboard.rs`, `serve/watcher.rs` | More files; clearer isolation. | **PREFER INLINE** until file exceeds ~800 lines (current: 437; expected after phase: ~650). Split is a later refactor if size grows. |

### Version Verification

```
Cargo.lock:  notify-debouncer-mini 0.4.1 (matches Cargo.toml "0.4" — no bump)
crates.io:   crossterm newest_version = 0.29.0 (2025-04-05)
```

**Planner decision point:** Pin `crossterm = "0.29"` (recommended). The minor version is stable — 0.28 ships since 2024 — but 0.29 adds the `KeyEventKind::Press` discrimination already present on Windows that's needed to ignore key-release events on that platform (see Common Pitfalls).

## Architecture Patterns

### System Architecture Diagram

```
                     ferro serve [--watch] [--backend-only] [--frontend-only]
                                          │
                                          ▼
                           validate_ferro_project / env setup
                                          │
          ┌───────────────────────────────┼────────────────────────────────┐
          ▼                               ▼                                ▼
   types regen (once)        ProcessManager (Vite)             BackendSupervisor thread
   (if !skip_types &&          │                                │      owns Child
    !frontend_only)            │ spawn_with_prefix               │      drain+coalesce triggers
                               │   npm run dev                   │      on trigger:
                               │                                 │        kill_current()
                               │                                 │        regen types (uninterruptible)
                               │                                 │        spawn cargo run
                               │                                 ▲
                               │                                 │ reload_rx (mpsc)
                               │                                 │
                               │                  ┌──────────────┴───────────────┐
                               │                  │                              │
                               │    Keyboard thread (if TTY)     File watcher thread (if --watch)
                               │      crossterm raw mode           notify-debouncer-mini
                               │      Drop guard restores raw      500 ms trailing edge
                               │      r → Manual                   *.rs in src/ → FileChanged
                               │      q → shutdown flag            init fail → warn, skip
                               │
                               └─ shutdown_all() kills Vite (step 5 of shutdown ordering)
                                          ▲
                                          │
                                  ctrlc handler / q key
                                  sets Arc<AtomicBool> shutdown = true
```

### Component Responsibilities

| Component | File / lines | Responsibility |
|-----------|-------------|----------------|
| `ProcessManager` | `serve.rs:14–114` (kept) | Owns Vite child only after this phase. `shutdown_all()` still called during shutdown step 5. |
| `BackendSupervisor` (new) | `serve.rs:~425–?` (after deletions, replaces `start_type_watcher`) | Owns backend child. Consumes `reload_rx`, performs kill→regen→spawn cycle. |
| Keyboard thread (new) | inline in `serve.rs` | Drops a RAII guard that disables raw mode; reads `Event::Key`, emits `ReloadTrigger::Manual` on `r`, sets shutdown on `q`. |
| File watcher thread (new) | inline in `serve.rs` | Wraps `notify_debouncer_mini::new_debouncer`, filters `*.rs`, emits `ReloadTrigger::FileChanged`. Only spawned under `--watch`. |
| Banner renderer (new) | `render_banner(opts) -> String` | Pure function of (is_watch, is_tty, frontend_only, backend_only). Unit-testable. |
| `get_package_name`, `ensure_npm_dependencies`, `validate_ferro_project`, `find_available_port` | `serve.rs:116–205` (kept) | Unchanged. |

### Recommended File Structure (inline)

```
ferro-cli/src/commands/serve.rs
├── use imports           (add: crossterm, notify_debouncer_mini, std::io::IsTerminal)
├── ProcessManager        (unchanged — Vite only after this phase)
├── helpers: get_package_name / validate / find_available_port / ensure_npm_dependencies
├── render_banner(opts) -> String          (NEW, pure)
├── enum ReloadTrigger { Manual, FileChanged }    (NEW)
├── struct BackendSupervisor { ... }       (NEW)
│   ├── fn spawn_backend()                 (reuses piping pattern from spawn_with_prefix)
│   ├── fn kill_current()                  (no-op when current = None — D-35)
│   └── fn run_loop(reload_rx, shutdown)
├── fn spawn_keyboard_thread(tx, shutdown) -> Option<JoinHandle>   (NEW)
│   └── RawModeGuard { impl Drop }         (NEW — RAII restore)
├── fn spawn_file_watcher(tx, shutdown) -> Option<Debouncer>       (NEW)
└── fn run(...)                            (rewired: no cargo-watch, supervisor-driven)
```

```
ferro-cli/tests/
├── serve_integration.rs                  (NEW — four integration tests, D-36)
└── fixtures/minimal-serve/               (NEW — compiled once, D-37)
    ├── Cargo.toml
    └── src/main.rs                        (single println! + immediate exit, sub-second build)
```

### Pattern 1: `notify-debouncer-mini` producer thread

```rust
// Source: https://docs.rs/notify-debouncer-mini/0.4.1 [VERIFIED via WebFetch]
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, DebouncedEventKind};
use notify::RecursiveMode;
use std::sync::mpsc::Sender;
use std::time::Duration;

fn spawn_file_watcher(
    tx: Sender<ReloadTrigger>,
) -> Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>> {
    let src = std::path::Path::new("src");
    if !src.is_dir() {
        eprintln!("{} src/ missing, --watch disabled", style("Warning:").yellow());
        return None;
    }
    let mut debouncer = match new_debouncer(
        Duration::from_millis(500), // D-19 fixed window
        move |res: notify_debouncer_mini::DebounceEventResult| {
            let Ok(events) = res else { return };
            // Filter to *.rs under src/
            let any_rs = events.iter().any(|e: &DebouncedEvent| {
                e.path.extension().map(|x| x == "rs").unwrap_or(false)
            });
            if any_rs {
                let _ = tx.send(ReloadTrigger::FileChanged); // D-20
            }
        },
    ) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{} notify init failed: {e}", style("Warning:").yellow()); // D-22
            return None;
        }
    };
    if let Err(e) = debouncer.watcher().watch(src, RecursiveMode::Recursive) {
        eprintln!("{} watch(src/) failed: {e}", style("Warning:").yellow());
        return None;
    }
    Some(debouncer) // dropped on shutdown → ends thread (Drop semantics verified)
}
```

Key API notes (verified against docs.rs/notify-debouncer-mini/0.4.1):

- `new_debouncer(timeout: Duration, handler: F) -> Result<Debouncer<F>, Error>` — two args only (no tick rate in `-mini`; that parameter is exclusive to `-full`).
- `DebouncedEvent` has **two fields**: `path: PathBuf` (singular, not `paths`) and `kind: DebouncedEventKind`.
- `DebouncedEventKind` has two non-exhaustive variants: `Any`, `AnyContinuous`. Filter on extension, not on kind.
- Accessing the watcher: `debouncer.watcher().watch(path, mode)` (not `debouncer.watch(...)`).
- **Drop ends the debouncer** — hold the `Debouncer` in the supervising scope so it lives for the whole `run()` call; drop on shutdown terminates its internal thread.

### Pattern 2: `crossterm` raw-mode keyboard thread with RAII guard

```rust
// Source: https://docs.rs/crossterm/0.29.0 [VERIFIED via WebFetch]
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use std::io::IsTerminal;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;

struct RawModeGuard; // D-25 RAII restore
impl Drop for RawModeGuard {
    fn drop(&mut self) { let _ = disable_raw_mode(); }
}

fn spawn_keyboard_thread(
    tx: Sender<ReloadTrigger>,
    shutdown: Arc<AtomicBool>,
) -> Option<std::thread::JoinHandle<()>> {
    if !std::io::stdin().is_terminal() { return None; } // D-24
    if let Err(e) = enable_raw_mode() {
        eprintln!("{} raw mode unavailable: {e}", style("Warning:").yellow()); // D-26
        return None;
    }
    Some(std::thread::spawn(move || {
        let _guard = RawModeGuard; // restored on panic / normal exit
        while !shutdown.load(Ordering::SeqCst) {
            // Poll with timeout so we can observe the shutdown flag.
            match event::poll(Duration::from_millis(100)) {
                Ok(true) => {}
                _ => continue,
            }
            let Ok(Event::Key(k)) = event::read() else { continue };
            // Windows key-release filter (crossterm 0.26+)
            if k.kind != KeyEventKind::Press { continue; }
            match (k.code, k.modifiers) {
                (KeyCode::Char('r'), KeyModifiers::NONE) => { // D-08 lowercase only
                    let _ = tx.send(ReloadTrigger::Manual);
                }
                (KeyCode::Char('q'), KeyModifiers::NONE)
                | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    shutdown.store(true, Ordering::SeqCst);
                    break;
                }
                _ => { /* ignore */ }
            }
        }
    }))
}
```

Key API notes (verified):

- `event::poll(Duration)` before `event::read()` — blocks `read()` otherwise, and we need to observe the shutdown flag.
- `KeyEventKind::Press` filter is required on Windows (crossterm 0.26+). Without it, `r` would fire twice (press + release).
- `disable_raw_mode()` is best-effort; ignoring its error is standard practice.

### Pattern 3: Supervisor loop with `recv_timeout` and trigger drain

```rust
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::sync::atomic::Ordering;
use std::time::Duration;

impl BackendSupervisor {
    fn run_loop(&mut self, reload_rx: Receiver<ReloadTrigger>) {
        self.spawn_backend(); // initial
        loop {
            if self.shutdown.load(Ordering::SeqCst) {
                self.kill_current(); // D-29 step 3
                break;
            }
            match reload_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(mut src) => {
                    // D-17: drain any additional pending triggers before action
                    loop {
                        match reload_rx.try_recv() {
                            Ok(next) => src = next, // keep most recent source label
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => break,
                        }
                    }
                    println!(
                        "{} reload triggered ({})",
                        style("[backend]").magenta().bold(),
                        match src {
                            ReloadTrigger::Manual => "manual",
                            ReloadTrigger::FileChanged => "file change",
                        }
                    );
                    self.kill_current();    // D-09/D-11 no-op when None
                    self.regenerate_types(); // D-10/D-18 uninterruptible
                    self.spawn_backend();   // D-12 no auto-respawn handled elsewhere
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }
}
```

### Anti-Patterns to Avoid

- **Handmade leading-edge debounce** (what `start_type_watcher` does today — `last_regen.elapsed() > debounce_duration`). Fires on the first event of a burst, ignores the rest. Opposite of the desired trailing-edge semantics. The whole point of pulling in `notify-debouncer-mini` is to delete this pattern.
- **`notify::RecommendedWatcher` directly** (current code). Removes the debouncing layer; we'd re-invent it badly.
- **Blocking `event::read()` without `event::poll()`** — would prevent the keyboard thread from observing the shutdown flag in a timely manner.
- **Forgetting `KeyEventKind::Press`** — causes `r` to fire twice on Windows.
- **Holding a `MutexGuard` across a `Child::kill()` / `wait()`** — not a problem here since the supervisor owns `current: Option<Child>` with no mutex, but worth avoiding if refactoring.
- **Spawning the child without `.stdout(Stdio::piped()).stderr(Stdio::piped())`** — breaks the prefixed-log UX. Reuse the existing pattern from `ProcessManager::spawn_with_prefix`.
- **Re-rendering the banner on every reload** — explicitly deferred (D-27).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cross-platform raw-mode stdin | Custom termios / ReadConsoleInput glue | `crossterm` | Platform quirks (Windows key-release events, macOS terminfo, Linux terminal-attribute restore on panic) are non-trivial. |
| Trailing-edge FS-event coalescing | Custom `Instant::now()` throttle | `notify-debouncer-mini` | The `-mini` crate explicitly handles the "continuous writes" case via `DebouncedEventKind::AnyContinuous`, plus coalesces cross-platform event bursts. |
| Cross-platform child-process prefixed piping | New thread-per-stream + BufReader | The existing `ProcessManager::spawn_with_prefix` pattern | Already implemented at `serve.rs:27–96`; extract or reuse. |
| Cross-platform TTY detection | Custom `isatty(3)` ffi | `std::io::IsTerminal` (stable since 1.70) | No new dep needed. |
| Ctrl+C handling | Custom signal-handling | Existing `ctrlc::set_handler` wiring | Already wired; supervisor piggybacks on the same `Arc<AtomicBool>`. |

**Key insight:** The phase is essentially about **removing** hand-rolled code (`ensure_cargo_watch`'s external-binary install, `start_type_watcher`'s broken leading-edge throttle) and replacing it with **library-provided** equivalents (`notify-debouncer-mini`'s real debouncer, `crossterm`'s real raw-mode). Resist the temptation to wrap each library too thinly — call the library directly in inline helpers.

## Runtime State Inventory

This is a refactor phase (replace cargo-watch → in-process supervisor). Explicit inventory:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | **None** — verified by grep for persisted config/db rows referencing "cargo-watch". | None. |
| Live service config | **None** — `ferro serve` is an ephemeral dev-tool command, no external service has a `cargo-watch` reference. | None. |
| OS-registered state | **External `cargo-watch` binary** may or may not be installed at `~/.cargo/bin/cargo-watch` on user machines. | **No action required** — we stop *installing* and *invoking* it, but leaving an existing installed copy is harmless. Document the removal in `docs/src/` (D-33). Optionally mention in changelog that `cargo install cargo-watch` is no longer needed. |
| Secrets / env vars | **None** — no env var references `cargo-watch`. `SERVER_HOST`, `SERVER_PORT`, `VITE_PORT`, `CARGO_SWEEP_DAYS`, `VITE_DEV_SERVER` are all unrelated. | None. |
| Build artifacts | **None** — `cargo-watch` is a global binary, not a project artifact. No `Cargo.lock` entry for it (it's an external binary, not a library dep). | None. |

**Derived plan tasks:** (1) delete `ensure_cargo_watch()` source, (2) remove docs references, (3) remove the `ferro serve` help-text line that mentions auto-reload via cargo-watch, (4) replace `cargo watch -x "run --bin ..."` spawn with direct `cargo run --bin ...` via supervisor.

## Current Code to Delete / Rewrite

Line ranges are from `ferro-cli/src/commands/serve.rs` (437 lines as of 2026-04-22).

| Kind | Lines | What it is | Replacement |
|------|-------|------------|-------------|
| Delete | 148–171 | `fn ensure_cargo_watch()` | Nothing — remove entirely (D-03). |
| Delete call site | 315–321 | `if !frontend_only { if let Err(e) = ensure_cargo_watch() ... }` in `run()` | Nothing — this block goes away. |
| Rewrite | 342–370 | Backend spawn via `manager.spawn_with_prefix("cargo", &["watch", "-x", &run_cmd], ...)` | Construct `BackendSupervisor`, spawn producer threads if applicable, hand off `reload_rx`. |
| Delete | 397–403 | `if !skip_types && !frontend_only { thread::spawn(move || start_type_watcher(...)) }` | Replaced by `BackendSupervisor` which folds type regen into each reload cycle. |
| Delete | 425–504 | `fn start_type_watcher(shutdown: Arc<AtomicBool>)` | Folded into `BackendSupervisor::regenerate_types()` call within the reload cycle (D-10). |
| Rewrite | 405–418 | Main wait loop (`while !manager.shutdown ... thread::sleep(100ms) ... any_exited()`) | Keep the shape but remove `any_exited()` triggered shutdown (that was the cargo-watch child dying); replace with join on supervisor thread handle. D-12 says backend child exits are not auto-respawned but also not grounds for shutdown — only Ctrl+C / `q` are. |

Surrounding context affected (no delete, but must understand):

- Lines 3, 9–10: `use notify::...`, `use std::sync::mpsc::channel;`, `use std::thread;` — imports stay but `notify::{Config, RecommendedWatcher, RecursiveMode, Watcher}` is replaced with `notify::RecursiveMode` only (+ `use notify_debouncer_mini::...`).
- Lines 14–114: `ProcessManager` unchanged in structure. Consider whether `shutdown_all()` and `any_exited()` still make sense; both remain relevant for Vite.
- Lines 283–313: initial type generation on startup stays as-is (runs once before supervisor starts).

Also to change outside `serve.rs`:

| File | Change |
|------|--------|
| `ferro-cli/src/main.rs` lines 29–50 | Add `#[arg(long)] watch: bool,` to `Commands::Serve` struct variant (D-02, D-34). |
| `ferro-cli/src/main.rs` lines 484–492 | Thread new `watch` through into `commands::serve::run(...)`. |
| `ferro-cli/Cargo.toml` | Add `crossterm = "0.29"` to `[dependencies]` (D-30). |
| `docs/src/**/*serve*` | Replace any cargo-watch reference with `--watch` + `r`-key model; add key legend (D-33). |
| `ferro-cli/src/commands/skills/serve.md` | Same (D-33 spirit — check for cargo-watch mention). |

## Existing Patterns to Reuse

| Pattern | Source | How to reuse |
|---------|--------|-------------|
| Stdout/stderr piping with colored prefix | `ProcessManager::spawn_with_prefix` (`serve.rs:27–96`) | Extract into a free function or a helper on `BackendSupervisor`. Both the backend supervisor and the existing Vite path should share it. Avoid duplicating the two `thread::spawn` blocks. |
| `Arc<AtomicBool>` shutdown flag | `serve.rs:8, 16, 99, 332` | Keep one flag; `ctrlc` handler, keyboard thread (`q`), supervisor's main-loop poll, and `ProcessManager` all read from it. |
| `ctrlc::set_handler` at `serve.rs:335` | — | Unchanged. Still the canonical Ctrl+C entry point. |
| `console::style(...).fg(color).bold()` logging | Throughout `serve.rs` | New `[backend] reload triggered (...)` log matches (D-27). |
| Initial type generation block (`serve.rs:283–313`) | — | Runs once at startup, unchanged. The supervisor's `regenerate_types()` calls the same `super::generate_types::generate_types_to_file`. |

## Risk Areas

| Risk | Why it matters | Mitigation |
|------|---------------|-----------|
| **Raw-mode terminal left broken on panic** | #1 failure mode for TUI-adjacent tools (named in CONTEXT.md specifics). User has to `stty sane` to recover. | RAII `RawModeGuard` with `impl Drop` (D-25). Integration test optional (D-38). Also: `ctrlc` handler DOES NOT call `disable_raw_mode()` directly — it only sets the shutdown flag. The keyboard thread's `Drop` runs when the thread exits after observing the flag. |
| **Child-kill race: child exits on its own between `try_wait` and `kill`** | `Child::kill()` can return `ErrorKind::InvalidInput` ("no such process") on already-dead child. | Swallow the error, call `wait()` to reap, then proceed to respawn (spec Error handling table). Documented in spec. |
| **Debouncer thread lifecycle** | `Debouncer` owns a background thread. If it's dropped early (e.g., end of a scope), filesystem events silently stop. | Hold `Debouncer` in the same scope as `run()` (as a local, not in a sub-block). Drop explicitly at shutdown; `Debouncer::Drop` joins its thread. |
| **Two producers racing on same `tx`** | Both keyboard and file-watcher threads send on the same `Sender<ReloadTrigger>`. | `std::sync::mpsc::Sender` is `Clone + Send`; clone one per producer. Standard pattern; no races. |
| **`cargo run` spawn consumes stdout/stderr before `child.stdout.take()`** | Calling `spawn()` with `Stdio::piped()` followed by `take()` only works if `take()` is called immediately. | Reuse the existing pattern verbatim; it already does this correctly. |
| **Windows key-event duplication** | `event::read()` returns both press and release events on Windows from crossterm 0.26+. Without `KeyEventKind::Press` filter, `r` fires twice per keypress on Windows. | Always filter `if k.kind != KeyEventKind::Press { continue; }` (shown in Pattern 2 above). |
| **Test harness for debouncer timing** | Tests asserting "10 events in 100ms → 1 trigger after 500ms" are time-sensitive and can flake under CI load. | Use `std::time::Instant` and assert a **range** (e.g., `>= 500ms && <= 1500ms`). Or use `notify-debouncer-mini`'s test-only `Debouncer::new_with_event_fn` with manual event injection — but `-mini` does not expose such a harness publicly, so the realistic test is filesystem-driven with a generous timeout upper bound. |
| **`is_terminal()` behavior under `cargo test`** | `cargo test` captures stdout, making `std::io::stdin().is_terminal()` return `false`. | Unit tests for `render_banner` do NOT depend on `is_terminal()` — the function takes `is_tty: bool` as an argument. Integration tests pass `--backend-only` and pipe stdin explicitly to control TTY state. |
| **`manager.any_exited()` triggering unwanted shutdown** | Today, if `cargo-watch` exits the whole process shuts down. In the new design, the backend child exiting is not grounds for shutdown (D-12). | Remove the `any_exited()` branch from the main wait loop, OR scope it to Vite only. |
| **Race between keyboard thread reading a key and shutdown flag going true** | `event::poll` with 100 ms timeout means worst case 100 ms delay between `q`/Ctrl+C and the thread exiting its loop. | Acceptable — still well within the 2 s shutdown budget from integration test D-36. |
| **`notify-debouncer-full` vs `-mini` confusion** | The official docs snippets from `notify-rs/notify` heavily feature `-full` (which has `new_debouncer(timeout, tick, tx)` — **three args**). `-mini` is `new_debouncer(timeout, handler)` — **two args**. | Use `-mini` per D-19. Pattern 1 above uses the correct 2-arg signature. Verified against docs.rs/notify-debouncer-mini/0.4.1. |

## Testing Approach

### Unit tests (in `ferro-cli/src/commands/serve.rs` under `#[cfg(test)] mod tests { ... }`)

Four unit tests from D-35:

1. **`render_banner_matrix`** — pure function `render_banner(is_watch: bool, is_tty: bool, backend_only: bool, frontend_only: bool) -> String`. Table-test four combinations: `(watch=false, tty=true)` shows `watch disabled` and `r  rebuild ...`; `(watch=true, tty=true)` shows `watch enabled (debounce 500ms)`; `(watch=false, tty=false)` shows `r unavailable (non-TTY stdin)`; `(watch=true, tty=false)` shows both `r unavailable` and `watch enabled`. Assert substrings. [covers D-05, D-24]
2. **`kill_current_noop_when_none`** — construct `BackendSupervisor` with `current = None`, call `kill_current()`, assert no panic. Supervisor still in valid state (no-op completed). [covers D-11, D-35]
3. **`trigger_source_formatting`** — map `ReloadTrigger::Manual` → `"manual"`, `ReloadTrigger::FileChanged` → `"file change"`. Pure mapping, 2 assertions. [covers D-27, D-28, D-35]
4. **`debouncer_coalesces_burst`** — write 10 files to a `tempfile::TempDir`/src/*.rs within <100ms, assert `reload_rx.recv()` yields exactly one `FileChanged` within <2s, then `try_recv()` returns Empty. Uses real `notify-debouncer-mini` against temp filesystem — the test is integration-ish but lives in the unit module because it exercises the debouncer wrapper directly, not the full serve loop. [covers D-19, D-20]

### Integration tests (in `ferro-cli/tests/serve_integration.rs`, following `docker_init_dry_run.rs` patterns)

Four integration tests from D-36:

1. **`backend_only_shuts_down_cleanly`** — spawn `ferro serve --backend-only` against `tests/fixtures/minimal-serve/`, wait for "Backend server on" banner, send SIGINT, assert exit within 2 s and no zombie children (poll `child.try_wait()`). [covers D-01, D-07, D-29]
2. **`r_key_in_no_watch_mode_triggers_one_rebuild`** — spawn without `--watch`, pipe `r\n` to stdin (in a mode that keeps stdin a TTY-like stream — see note below), assert exactly one `reload triggered (manual)` appears within 2 s. Due to TTY requirement (D-24), this test may need to use a `pty` crate (e.g., `portable-pty`) to simulate a TTY, OR skip the TTY dependency by triggering reload via an internal hook. **Recommendation:** expose a test-only path (e.g., a function `trigger_reload()` that sends `Manual` on the channel) rather than wiring a pty into the test suite. Fall back to manual verification (validation checklist #1) if the pty path is too heavy. [covers D-06]
3. **`watch_mode_debounces_burst`** — spawn with `--watch`, write 10 files into the fixture's `src/` in quick succession, assert exactly one `reload triggered (file change)` appears after ~500ms and before 2s. [covers D-19, D-20]
4. **`non_tty_stdin_banner_and_no_crash`** — spawn with default stdin (cargo test's captured stdin — non-TTY), assert banner contains `r unavailable` and the process does not panic. Send SIGINT, assert clean exit. [covers D-05, D-24, D-26]

### Optional integration test (D-38)

- **`raw_mode_restored_on_exit`** — shell out to `stty -g`, spawn `ferro serve`, send SIGINT, shell out to `stty -g` again, assert the state string matches. Flaky on GitHub runners (they may not even have a TTY). Mark `#[ignore]` by default; run manually.

### Manual validation checklist (from spec, reprinted for completeness)

1. `ferro serve` — banner `watch disabled`; saving `.rs` does nothing; `r` triggers rebuild.
2. `ferro serve --watch` — banner `watch enabled`; 5 rapid saves → one rebuild after ~500 ms.
3. `r` mid-compile → kill, fresh build.
4. Ctrl+C during compile → backend + Vite exit within 2 s; terminal not stuck in raw mode (test with `stty -a | grep -i raw` after).
5. Introduce compile error → rebuild fails, serve waits; fix + `r` → backend returns.
6. `ferro serve --frontend-only` → no supervisor, no `r` prompt; Ctrl+C works as before.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `tempfile 3.24` (already dev-dep) |
| Config file | none — `cargo test` via `Cargo.toml` workspace |
| Quick run command | `cargo test -p ferro-cli` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | supervisor's `cargo run` spawn | ✓ (assumed; required to build ferro itself) | — | — |
| `npm` | `ensure_npm_dependencies` (unchanged) | Project-level, unchanged by this phase | — | — |
| `notify-debouncer-mini` 0.4.x | file watcher | ✓ (Cargo.lock pinned to 0.4.1) | 0.4.1 | — |
| `crossterm` 0.29.x | keyboard thread | ✗ (needs adding to Cargo.toml) | — | Add to deps (D-30). |
| A TTY for manual `r` testing | manual validation | Depends on user terminal | — | Non-TTY path exists (banner shows `r unavailable`). |

**Missing dependencies with no fallback:** None — `crossterm` is trivially addable.
**Missing dependencies with fallback:** None.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| External `cargo-watch` binary | In-process `notify-debouncer-mini` + `std::process::Child` | this phase | Removes install step, removes external process, gives ferro control over lifecycle. |
| Leading-edge throttle (`last_regen.elapsed() > 500ms`) | Trailing-edge debounce (`notify-debouncer-mini`, 500ms window) | this phase | Correct semantics: coalesce bursts into one rebuild *after* the burst settles. |
| Auto-reload always on | Opt-in via `--watch` | this phase | Manual-first workflow better suited to thermally-constrained hardware. |
| No runtime key interaction | `r` to reload, `q` to quit | this phase | User can drive rebuilds on demand; matches Vite's HMR affordance mental model. |

**Deprecated / outdated (remove):**

- `cargo-watch` install step and all references.
- Handmade 500ms leading-edge throttle in `start_type_watcher`.
- `notify::RecommendedWatcher` direct usage in `start_type_watcher` (replaced by debouncer's internal watcher).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `#[test]` + `tempfile` 3.24 (already dev-dep) |
| Config file | none (workspace `Cargo.toml`) |
| Quick run command | `cargo test -p ferro-cli` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Decisions → Test Map

Because this phase has no REQ-IDs, the validation matrix maps **decisions (D-XX)** to tests. Every observable decision must have a test or an explicit "manual only" justification.

| ID | Behavior | Test Type | Automated Command | File Exists? |
|----|----------|-----------|-------------------|--------------|
| D-01 | Auto-watch OFF by default — no file watcher thread when `--watch` absent | integration | `cargo test -p ferro-cli --test serve_integration backend_only_shuts_down_cleanly` | ❌ Wave 0 |
| D-02 | `--watch` flag accepted by clap | unit (clap-driven) | `cargo test -p ferro-cli clap_watch_flag_parses` | ❌ Wave 0 |
| D-03 | `ensure_cargo_watch()` source removed | build-time | `cargo build -p ferro-cli` fails if function is absent from call sites that still reference it (redundant — deletion is verified by grep in `gsd-verify-work` or equivalent) | N/A (build) |
| D-04 | Other flags unchanged | existing tests | `cargo test -p ferro-cli` — no regressions in existing flag coverage (none today for serve, but main.rs clap parses still) | ❌ Wave 0 if adding |
| D-05 | Banner text for each (watch × TTY) combination | **unit** | `cargo test -p ferro-cli render_banner_matrix` | ❌ Wave 0 |
| D-06 | `r` key triggers `Manual` in no-watch mode | integration (requires pty OR test-hook) | `cargo test -p ferro-cli --test serve_integration r_key_in_no_watch_mode_triggers_one_rebuild` | ❌ Wave 0 |
| D-07 | `q` / Ctrl+C = graceful shutdown | integration | `cargo test -p ferro-cli --test serve_integration backend_only_shuts_down_cleanly` | ❌ Wave 0 |
| D-08 | Lowercase `r` only; `R` ignored | **unit** | `cargo test -p ferro-cli keyboard_thread_ignores_uppercase_r` (pure mapper test — refactor `match (code, modifiers)` into a free function `classify_key(...) -> Option<KbAction>` for testability) | ❌ Wave 0 |
| D-09 | New trigger mid-build cancels in-flight | manual | Validation checklist #3 | Manual |
| D-10 | Reload = backend + types together, never frontend | integration | Implicit in D-06 and D-19 tests (assert Vite is not touched) | ❌ Wave 0 |
| D-11 | `kill_current` no-op when current = None | **unit** | `cargo test -p ferro-cli kill_current_noop_when_none` | ❌ Wave 0 |
| D-12 | No auto-respawn on non-zero exit | manual | Validation checklist #5 | Manual |
| D-13 | `BackendSupervisor` owns backend child exclusively | structural | Code review + type system (no `ProcessManager::spawn(cargo run)` call anywhere) | Review |
| D-14 | `ProcessManager` keeps Vite | structural | Code review — `manager.spawn_with_prefix("npm", ...)` still present | Review |
| D-15 | Producers optional: keyboard only if TTY, watcher only if `--watch` | integration | `non_tty_stdin_banner_and_no_crash` covers keyboard; `backend_only_shuts_down_cleanly` covers watcher-off | ❌ Wave 0 |
| D-16 | `std::sync::mpsc` + `recv_timeout` | structural | Code review — no `crossbeam_channel` or `tokio::sync::mpsc` imports in serve.rs | Review |
| D-17 | Trigger coalescing via `try_recv` drain | **unit** | `cargo test -p ferro-cli supervisor_coalesces_multiple_triggers` — construct supervisor with a channel, send 3 `Manual` triggers, assert only one kill/regen/spawn cycle runs (use a mock "cycle count" incremented in a wrapped `regenerate_types`). | ❌ Wave 0 |
| D-18 | Types regen uninterruptible | manual + structural | Validation: press `r` during regen and observe second trigger is picked up only after the first completes | Manual |
| D-19 | Debouncer 500 ms fixed window | **unit** | `cargo test -p ferro-cli debouncer_coalesces_burst` — assert single event after 500–2000 ms for a 10-file burst within 100 ms | ❌ Wave 0 |
| D-20 | Watch target = `src/` recursive, `*.rs` filter | integration | `cargo test -p ferro-cli --test serve_integration watch_mode_debounces_burst` — write `src/foo.rs` and `src/other.txt`, assert only `.rs` triggers | ❌ Wave 0 |
| D-21 | `Cargo.toml` / migrations do NOT trigger | integration | Part of D-20 test: write `fixture/Cargo.toml`, assert no reload within 2s | ❌ Wave 0 |
| D-22 | Missing `src/` or init failure → warn + no crash | **unit** | `cargo test -p ferro-cli spawn_file_watcher_missing_src_returns_none` — `spawn_file_watcher` in a tempdir with no `src/` returns None and logs warning | ❌ Wave 0 |
| D-23 | `crossterm` dep present | structural | `grep crossterm ferro-cli/Cargo.toml` in CI | Review |
| D-24 | `is_terminal()` drives keyboard-thread spawn | **unit** | `cargo test -p ferro-cli spawn_keyboard_thread_skipped_when_not_tty` — factor the decision into a pure function `should_spawn_keyboard(is_tty: bool) -> bool`; banner renderer already covers user-visible side | ❌ Wave 0 |
| D-25 | RAII Drop guard restores raw mode | manual | Validation checklist #4 — `stty -a` after Ctrl+C shows no raw. Optional automated via D-38. | Manual |
| D-26 | `enable_raw_mode()` failure → skip, no crash | manual | Hard to simulate portably; manual verification on a non-TTY-capable environment | Manual |
| D-27 | Reload log line format `[backend] reload triggered ({source})` | **unit** | Part of `trigger_source_formatting` test — assert exact format string | ❌ Wave 0 |
| D-28 | Source labels are `manual` / `file change` | **unit** | Part of `trigger_source_formatting` test | ❌ Wave 0 |
| D-29 | Shutdown ordering steps 1–6 | integration | `backend_only_shuts_down_cleanly` asserts clean exit in 2s — the ordering is observed via logs ("Servers stopped." last) | ❌ Wave 0 |
| D-30 | `crossterm` added to `Cargo.toml` | build-time | `cargo build` fails otherwise | N/A |
| D-31 | `notify-debouncer-mini = "0.4"` kept | structural | Cargo.toml unchanged for this dep | Review |
| D-32 | All cargo-watch refs removed | CI grep | `! grep -r "cargo-watch" ferro-cli/ docs/src/` | ❌ Wave 0 script |
| D-33 | `docs/src/` updated | docs | Manual review + grep for cargo-watch | Manual + grep |
| D-34 | `ferro serve --help` reflects `--watch` | integration | `cargo run -p ferro-cli -- serve --help | grep -- '--watch'` | Manual smoke |
| D-35 | All four unit tests exist and pass | self-referential | `cargo test -p ferro-cli` | ❌ Wave 0 |
| D-36 | All four integration tests exist and pass | self-referential | `cargo test -p ferro-cli --tests` | ❌ Wave 0 |
| D-37 | Fixture project at `ferro-cli/tests/fixtures/minimal-serve/` | structural | File existence check in CI | ❌ Wave 0 |
| D-38 | Raw-mode restoration test (optional) | manual or `#[ignore]`d | `cargo test -p ferro-cli --test serve_integration raw_mode_restored_on_exit -- --ignored` | ❌ Wave 0 (optional) |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-cli` (<30 s for unit tests; integration tests <15 s each).
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` (per CLAUDE.md).
- **Phase gate:** All of the above green, plus manual validation checklist items 1–6 run by the author before `/gsd-verify-work`.

### Wave 0 Gaps

- [ ] `ferro-cli/tests/serve_integration.rs` — integration tests (D-36)
- [ ] `ferro-cli/tests/fixtures/minimal-serve/Cargo.toml` — fixture (D-37)
- [ ] `ferro-cli/tests/fixtures/minimal-serve/src/main.rs` — fixture (D-37)
- [ ] Unit test module at the bottom of `ferro-cli/src/commands/serve.rs` with five `#[test]` functions (D-35 plus `supervisor_coalesces_multiple_triggers` for D-17)
- [ ] Factor `should_spawn_keyboard`, `classify_key`, and `render_banner` into pure testable functions before wiring them into `run()` (required for unit tests to avoid spawning real threads)

## Security Domain

`security_enforcement` is not explicitly `false` in `.planning/config.json`, so the section is required.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V1 Architecture | yes | Replacing external binary invocation (`cargo watch`) with in-process supervision reduces attack surface (no PATH-hijack risk on `cargo-watch` binary). |
| V2 Authentication | no | Dev-time local tool; no authentication surface. |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | partial | Keyboard input is trusted (local user); file paths from `notify` events are filtered to `*.rs` before any action, avoiding spurious execution on arbitrary filenames. |
| V6 Cryptography | no | — |
| V7 Error Handling | yes | All error paths are logged, never swallowed silently except `disable_raw_mode()` (safe to ignore on teardown). |
| V10 Malicious Code | partial | Removing external `cargo-watch` install (`cargo install cargo-watch`) eliminates a supply-chain vector where a compromised cargo-watch crate could run arbitrary code during ferro setup. |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious `cargo install cargo-watch` hijack | Tampering / Elevation | **Removed entirely** by this phase (D-03, D-32). No third-party binary is auto-installed anymore. |
| Raw-mode terminal not restored → user session wedged | Denial of Service (developer) | RAII `Drop` guard (D-25). |
| FS events trigger unbounded rebuilds (resource exhaustion) | Denial of Service | Trailing-edge 500 ms debounce (D-19); cancel-in-flight behavior prevents build stacking (D-09). |
| Attacker-controlled file path in `notify` events triggers non-deterministic behavior | Tampering | Filter to `*.rs` in `src/` (D-20, D-21); rebuilds run `cargo run` with fixed args regardless of which path changed. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Pinning `crossterm = "0.29"` is acceptable and matches "latest stable at implementation time". [VERIFIED: crates.io 2026-04-22] | Standard Stack | If a newer stable appears before implementation, planner may choose it; 0.29 is a floor. Low risk. |
| A2 | `notify-debouncer-mini` 0.4.1's `Debouncer::Drop` cleanly joins its thread. [CITED: docs.rs "Dropping the debouncer also ends the debouncer"] | Patterns / Risk Areas | If Drop is not a clean join, test thread lifetimes may expose issues. Mitigation: hold `Debouncer` in `run()` scope, drop explicitly before return. |
| A3 | `event::poll(Duration::from_millis(100))` is low-overhead enough to run continuously. [ASSUMED — standard TUI pattern, not benchmarked for ferro] | Patterns | If CPU usage is noticeable, increase poll interval to 250 ms (still well within 2 s shutdown budget). |
| A4 | Integration test for `r` key can be written without a pty crate by exposing a test-only reload-trigger path. [ASSUMED — actual test authorship may find a pty unavoidable] | Testing | If true pty is required, add `portable-pty` as dev-dep. This is a planner choice, not a blocker. |
| A5 | `docs/src/` contains a cargo-watch reference that needs updating. [ASSUMED — `grep "cargo-watch" docs/src/` not run in research] | Current code to delete | If no reference exists, task is a no-op; run the grep during planning to confirm. |
| A6 | Current `serve.rs` line ranges (437 total) are correct as of the research date; deletion ranges `148–171` and `425–504` assume no further edits between research and implementation. | Current code to delete | Re-verify line numbers at implementation time; structure, not line numbers, is what matters. |

## Open Questions

1. **Should `render_banner` print anything when both `--backend-only` and `--frontend-only` are accidentally set?**
   - What we know: existing `validate_ferro_project` doesn't explicitly reject this combination.
   - What's unclear: does clap reject it? Current main.rs shows no conflict attribute.
   - Recommendation: planner can add a `conflicts_with` clap attribute as an unrelated quality-of-life fix, or leave as-is (it's not in phase scope).

2. **Is there a planning decision on whether the keyboard thread joins on shutdown or is detached?**
   - What we know: D-29 step 4 says "Keyboard thread's `Drop` guard runs", which implies the thread is joined (otherwise `Drop` on a thread-local doesn't run deterministically).
   - What's unclear: whether `run()` should keep the `JoinHandle` and call `.join()` explicitly.
   - Recommendation: **keep the `JoinHandle`** and `.join()` it after the shutdown flag is set but before `manager.shutdown_all()`. This makes step 4 deterministic. If the keyboard thread is stuck in `event::poll`, the 100 ms poll interval guarantees termination within 100 ms.

3. **How does `ctrlc` interact with `crossterm` raw mode?**
   - What we know: `ctrlc` installs a signal handler that runs in a signal-safety context. On Unix, raw mode just changes tty termios — a signal handler that only sets an `AtomicBool` is safe.
   - What's unclear: edge cases on Windows where Ctrl+C might be consumed by raw-mode event loop instead of the signal handler.
   - Recommendation: test manually on Windows if targeted; the keyboard thread's Ctrl+C path (D-07) provides a second channel regardless.

## Sources

### Primary (HIGH confidence)

- `.planning/phases/145-ferro-serve-manual-reload-key-and-watch-supervisor/145-CONTEXT.md` — 38 locked decisions.
- `docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md` — approved design spec (source of every decision).
- `docs.rs/notify-debouncer-mini/0.4.1/` — `new_debouncer`, `DebouncedEvent`, `DebouncedEventKind`, `Debouncer::watcher()` (verified via WebFetch).
- `docs.rs/crossterm/0.29.0/` — `Event`, `KeyEvent`, `KeyCode`, `KeyEventKind::Press`, `enable_raw_mode`, `disable_raw_mode`, `event::poll`, `event::read` (verified via WebFetch + Context7 CLI fallback).
- `crates.io/api/v1/crates/crossterm` — latest stable version 0.29.0, published 2025-04-05.
- `Cargo.lock` — `notify-debouncer-mini 0.4.1` pinned (verified via grep).
- `ferro-cli/src/commands/serve.rs` — 437 lines, current implementation read end-to-end.
- `ferro-cli/src/main.rs` — clap definitions for `Commands::Serve` (lines 29–50, 484–492) and the existing flag surface.
- `ferro-cli/Cargo.toml` — dependency manifest (`notify-debouncer-mini = "0.4"` at line 30).
- `ferro-cli/tests/docker_init_dry_run.rs` — existing integration-test pattern (CHDIR_LOCK, library-entry-point invocation).

### Secondary (MEDIUM confidence)

- `github.com/notify-rs/notify/blob/main/notify-debouncer-mini/README.md` — "The Notify debouncer is a utility designed to filter incoming file system events. It ensures that only one event is emitted per specified timeframe for each file" (via Context7 CLI).
- Context7 CLI (`ctx7@latest docs /crossterm-rs/crossterm ...`) — crossterm raw-mode, `event::poll` pattern, `event::read` patterns.

### Tertiary (LOW confidence)

- None — all API claims tied to official docs or direct source.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — versions verified against crates.io and Cargo.lock.
- Architecture: HIGH — spec and CONTEXT.md prescribe structure; research confirmed it is implementable with the locked dep pins.
- Pitfalls: HIGH — key pitfalls (raw-mode hygiene, Windows key-release, debouncer thread lifecycle) are documented in official sources or the design spec.
- Testing strategy: MEDIUM — the `r` key integration test realistically requires a pty or a test-only trigger hook; both paths are acceptable but the specific choice is a planner decision (A4).

**Research date:** 2026-04-22
**Valid until:** 2026-05-22 (stable dependencies; only risk is crossterm minor bump, which is additive).

## RESEARCH COMPLETE

**Phase:** 145 - ferro-serve-manual-reload-key-and-watch-supervisor
**Confidence:** HIGH

### Key Findings

- `notify-debouncer-mini` 0.4.1 API is a **two-arg** `new_debouncer(timeout, handler)` — the three-arg signature shown in most online examples belongs to `-full`. `DebouncedEvent` has a single `path`, not `paths`. `DebouncedEventKind` has variants `Any` and `AnyContinuous`.
- `crossterm` latest stable is **0.29.0** (published 2025-04-05). Key pattern is `event::poll(Duration)` → `event::read()` → match on `Event::Key(k)` with `k.kind == KeyEventKind::Press` (required on Windows since 0.26+).
- Delete ranges in `serve.rs`: `148–171` (`ensure_cargo_watch`), `315–321` (its call site), `342–370` (cargo-watch spawn), `397–403` (type-watcher thread spawn), `425–504` (`start_type_watcher`).
- The existing `ProcessManager::spawn_with_prefix` piping pattern (lines 27–96) should be extracted into a shared helper so `BackendSupervisor::spawn_backend` reuses it without duplication.
- All 38 decisions have concrete tests in the Validation Architecture matrix; a small number (D-25, D-26, D-09, D-12, D-18) are intentionally manual-only and justified.

### File Created

`/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/145-ferro-serve-manual-reload-key-and-watch-supervisor/145-RESEARCH.md`

### Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Standard Stack | HIGH | All versions verified against crates.io / Cargo.lock. |
| Architecture | HIGH | Locked decisions + spec; three-thread mpsc pattern is standard Rust idiom. |
| Pitfalls | HIGH | All key pitfalls have documented mitigations tied to official sources. |
| Testing | MEDIUM | `r` key integration test requires a pty-simulation choice not yet made; flagged in Open Questions. |

### Open Questions

1. Should `render_banner` reject `--backend-only && --frontend-only`? Out of scope; planner may add as polish.
2. Keyboard thread: join explicitly or rely on Drop? Recommend explicit `.join()` for D-29 determinism.
3. `ctrlc` × raw-mode interaction on Windows — manual verification suggested.

### Ready for Planning

Research complete. Planner can now create PLAN.md files with confidence that the dependency API surface, delete ranges, reuse patterns, and validation matrix are all concrete.
