# Phase 145: ferro serve manual reload key and watch supervisor — Pattern Map

**Mapped:** 2026-04-22
**Files analyzed:** 8 (3 modified + 5 created)
**Analogs found:** 8 / 8 — every file has a concrete in-tree template.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-cli/src/commands/serve.rs` (mod) | cli-command / process-supervisor | event-driven mpsc + child-process lifecycle | self + `ProcessManager::spawn_with_prefix` | exact |
| `ferro-cli/src/main.rs` (mod, clap) | cli-entrypoint | flag-parsing | `Commands::GenerateTypes { watch: bool }` at `main.rs:52-60` | exact |
| `ferro-cli/Cargo.toml` (mod) | manifest | — | self + existing `notify-debouncer-mini = "0.4"` at line 30 | exact |
| `docs/src/reference/cli.md` (mod) | docs | — | self, serve section at lines 110-136 | exact |
| `ferro-cli/src/commands/skills/serve.md` (mod) | docs / prompt-template | — | self (full rewrite) | exact |
| `ferro-cli/tests/serve_supervisor.rs` (new) | integration-test | child-process + fixture | `ferro-cli/tests/docker_init_dry_run.rs` (`CHDIR_LOCK`, `tempdir`); `tests/gestiscilo_fixture.rs:22-24` (`fixture_dir` via `CARGO_MANIFEST_DIR`) | role-match (our tests spawn the real binary, existing tests call lib entry points) |
| `ferro-cli/tests/fixtures/minimal-serve/{Cargo.toml,src/main.rs}` (new) | test-fixture | — | `docker_init_dry_run.rs:38-47` (inline writer) + `ferro-cli/tests/fixtures/gestiscilo/Cargo.toml` (layout convention) | exact |
| inline `#[cfg(test)] mod tests` in `serve.rs` (new) | unit-test | pure function + TempDir | `commands/docker_init.rs:142-162` (pure, substring asserts); `commands/make_theme.rs:138-308` (TempDir style) | exact |

---

## Pattern Assignments

### `ferro-cli/src/commands/serve.rs`

#### Pattern 1 — Imports (replace `notify::{Config,RecommendedWatcher,RecursiveMode,Watcher}` line 3)

Keep existing: `super::clean`, `console::style`, `std::io::{BufRead,BufReader}`, `std::net::TcpListener`, `std::path::Path`, `std::process::{Child,Command,Stdio}`, `std::sync::atomic::{AtomicBool,Ordering}`, `std::sync::mpsc::channel`, `std::sync::Arc`, `std::thread`, `std::time::Duration`.

Replace line 3 with:
```rust
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError};
```

#### Pattern 2 — Extract `spawn_child_with_prefix` free function from `serve.rs:27-96`

**Analog lines 47-94** (the stdout/stderr piping thread pair). Lift verbatim into a free function:
```rust
fn spawn_child_with_prefix(
    command: &str, args: &[&str], cwd: Option<&Path>,
    prefix: &str, color: console::Color, env_vars: &[(&str, &str)],
    shutdown: Arc<AtomicBool>,
) -> Result<Child, String>
```
Both `ProcessManager::spawn_with_prefix_env` (Vite) and `BackendSupervisor::spawn_backend()` call it. Eliminates duplication flagged in RESEARCH.md "Existing Patterns to Reuse" row 1.

#### Pattern 3 — Shutdown-polling wait loop (adapt `serve.rs:410-418`)

Drop the `any_exited()` trigger per D-12. Replacement shape:
```rust
while !shutdown.load(Ordering::SeqCst) { thread::sleep(Duration::from_millis(100)); }
if let Some(h) = keyboard_handle { let _ = h.join(); }
let _ = supervisor_handle.join();
manager.shutdown_all();
println!("{}", style("Servers stopped.").green());
```

#### Pattern 4 — Ctrl+C handler (unchanged, reuse `serve.rs:334-340` verbatim)
```rust
ctrlc::set_handler(move || {
    println!();
    println!("{}", style("Shutting down servers...").yellow());
    shutdown.store(true, Ordering::SeqCst);
}).expect("Error setting Ctrl-C handler");
```

#### Pattern 5 — `notify-debouncer-mini` producer thread (NEW — source: RESEARCH.md §Pattern 1)
```rust
fn spawn_file_watcher(tx: Sender<ReloadTrigger>)
    -> Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>
{
    let src = Path::new("src");
    if !src.is_dir() {
        eprintln!("{} src/ missing, --watch disabled", style("Warning:").yellow());
        return None;
    }
    let mut debouncer = match new_debouncer(Duration::from_millis(500),
        move |res: notify_debouncer_mini::DebounceEventResult| {
            let Ok(events) = res else { return };
            let any_rs = events.iter().any(|e: &DebouncedEvent|
                e.path.extension().map(|x| x == "rs").unwrap_or(false));
            if any_rs { let _ = tx.send(ReloadTrigger::FileChanged); }
        }) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{} notify init failed: {e}", style("Warning:").yellow());
            return None;
        }
    };
    if let Err(e) = debouncer.watcher().watch(src, RecursiveMode::Recursive) {
        eprintln!("{} watch(src/) failed: {e}", style("Warning:").yellow());
        return None;
    }
    Some(debouncer)
}
```
Key invariants: 2-arg `new_debouncer` (mini, not full); `DebouncedEvent.path` singular; keep `Debouncer` alive through `run()`.

#### Pattern 6 — `crossterm` keyboard thread + RAII guard (NEW — source: RESEARCH.md §Pattern 2)
```rust
struct RawModeGuard;
impl Drop for RawModeGuard { fn drop(&mut self) { let _ = disable_raw_mode(); } }

fn spawn_keyboard_thread(tx: Sender<ReloadTrigger>, shutdown: Arc<AtomicBool>)
    -> Option<std::thread::JoinHandle<()>>
{
    if !std::io::stdin().is_terminal() { return None; }
    if let Err(e) = enable_raw_mode() {
        eprintln!("{} raw mode unavailable: {e}", style("Warning:").yellow());
        return None;
    }
    Some(std::thread::spawn(move || {
        let _guard = RawModeGuard;
        while !shutdown.load(Ordering::SeqCst) {
            match event::poll(Duration::from_millis(100)) { Ok(true) => {}, _ => continue }
            let Ok(Event::Key(k)) = event::read() else { continue };
            if k.kind != KeyEventKind::Press { continue; }
            match classify_key(k.code, k.modifiers) {
                Some(KbAction::Reload) => { let _ = tx.send(ReloadTrigger::Manual); }
                Some(KbAction::Quit)   => { shutdown.store(true, Ordering::SeqCst); break; }
                None => {}
            }
        }
    }))
}
```

#### Pattern 7 — Pure-function extractors (required by RESEARCH.md lines 591-592 for testability)
```rust
pub(super) fn render_banner(is_watch: bool, is_tty: bool, backend_only: bool,
    frontend_only: bool, backend_host: &str, backend_port: u16, vite_port: u16) -> String { /* */ }
pub(super) enum KbAction { Reload, Quit }
pub(super) fn classify_key(code: KeyCode, modifiers: KeyModifiers) -> Option<KbAction> {
    match (code, modifiers) {
        (KeyCode::Char('r'), KeyModifiers::NONE) => Some(KbAction::Reload),
        (KeyCode::Char('q'), KeyModifiers::NONE)
        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(KbAction::Quit),
        _ => None,
    }
}
pub(super) fn should_spawn_keyboard(is_tty: bool) -> bool { is_tty }
pub(super) fn format_trigger_source(t: ReloadTrigger) -> &'static str {
    match t { ReloadTrigger::Manual => "manual", ReloadTrigger::FileChanged => "file change" }
}
```

#### Pattern 8 — Supervisor loop (NEW — source: RESEARCH.md §Pattern 3)
```rust
enum ReloadTrigger { Manual, FileChanged }

struct BackendSupervisor {
    package_name: String, skip_types: bool,
    project_path: PathBuf, types_output_path: PathBuf,
    current: Option<Child>, shutdown: Arc<AtomicBool>,
}

impl BackendSupervisor {
    fn run_loop(&mut self, reload_rx: Receiver<ReloadTrigger>) {
        self.spawn_backend();
        loop {
            if self.shutdown.load(Ordering::SeqCst) { self.kill_current(); break; }
            match reload_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(mut src) => {
                    // D-17 drain — factor out into `fn drain_triggers(&mut self, rx) -> ReloadTrigger`
                    loop {
                        match reload_rx.try_recv() {
                            Ok(next) => src = next,
                            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
                        }
                    }
                    println!("{} reload triggered ({})",
                        style("[backend]").magenta().bold(), format_trigger_source(src));
                    self.kill_current();
                    self.regenerate_types();
                    self.spawn_backend();
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }
    fn kill_current(&mut self) {
        if let Some(mut child) = self.current.take() {
            let _ = child.kill(); let _ = child.wait();
        }
    }
    fn regenerate_types(&self) {
        if self.skip_types { return; }
        match super::generate_types::generate_types_to_file(&self.project_path, &self.types_output_path) {
            Ok(count) if count > 0 =>
                println!("{} Regenerated {} type(s)", style("[types]").blue(), count),
            Ok(_) => {}
            Err(e) => eprintln!("{} Failed to regenerate: {}", style("[types]").yellow(), e),
        }
    }
    fn spawn_backend(&mut self) {
        let args = ["run", "--bin", &self.package_name];
        match spawn_child_with_prefix("cargo", &args, None, "[backend]",
            console::Color::Magenta, &[], self.shutdown.clone()) {
            Ok(child) => self.current = Some(child),
            Err(e) => { eprintln!("{} {}", style("Error:").red().bold(), e); self.current = None; }
        }
    }
}
```

#### Deletions in `serve.rs` (exact ranges from RESEARCH.md §"Current Code to Delete")

| Lines | What | Replacement |
|---|---|---|
| 3 | `use notify::{Config,RecommendedWatcher,RecursiveMode,Watcher};` | keep only `use notify::RecursiveMode;` + new imports |
| 148–171 | `fn ensure_cargo_watch()` | removed entirely |
| 315–321 | `if !frontend_only { ensure_cargo_watch() ... }` | removed |
| 342–370 | `manager.spawn_with_prefix("cargo", &["watch","-x",&run_cmd], ...)` | supervisor construction + optional producers |
| 397–403 | `thread::spawn(move || start_type_watcher(...))` | folded into supervisor |
| 410–418 | `any_exited()` branch | pure shutdown-flag poll |
| 425–504 | `fn start_type_watcher()` | folded into `BackendSupervisor::regenerate_types` |

---

### `ferro-cli/src/main.rs`

**Analog:** `Commands::GenerateTypes { watch: bool, ... }` at `main.rs:52-60` (identical bool-flag shape). Add to `Commands::Serve` (currently `main.rs:29-50`):
```rust
/// Enable file-watch auto-reload (500ms debounce)
#[arg(long)]
watch: bool,
```
Do NOT add `short = 'w'` — collides semantically with `generate-types -w`.

**Dispatch update** (`main.rs:484-492`):
```rust
Commands::Serve { port, frontend_port, backend_only, frontend_only, skip_types, watch } => {
    commands::serve::run(port, frontend_port, backend_only, frontend_only, skip_types, watch);
}
```

**Corresponding `serve::run` signature** (`serve.rs:207-213`): add `watch: bool` as 6th param.

---

### `ferro-cli/Cargo.toml`

**Analog:** existing `notify-debouncer-mini = "0.4"` at line 30. Add one line under `[dependencies]`:
```toml
crossterm = "0.29"
```
Keep unchanged: `notify = "6"`, `notify-debouncer-mini = "0.4"` (D-31), `tempfile = "3.24.0"` under `[dev-dependencies]`.

---

### `docs/src/reference/cli.md`

**Analog:** self (serve section lines 110-136).

1. Add to options table (after line 128):
   ```
   | `--watch` | `false` | Enable file-watch auto-reload (500ms debounce) |
   ```
2. Replace line 132 (`1. Starts the Rust backend with cargo watch for hot reloading`) with:
   ```
   1. Starts the Rust backend via an in-process supervisor. Auto-reload is opt-in via `--watch`; without it, press `r` to rebuild on demand.
   ```
3. Add a key-legend subsection (r/q/Ctrl+C) matching spec lines 76-86.
4. Grep the whole `docs/src/` for other `cargo-watch` references before commit.

---

### `ferro-cli/src/commands/skills/serve.md`

**Analog:** self (full rewrite).

Problematic spans to replace:
- Lines 14-18: arguments mention `--watch` as default and `--no-watch` (latter does not exist)
- Lines 28-38: `check_deps` step invokes `cargo install cargo-watch` — delete (D-03/D-32)
- Lines 41-73: every shell command uses `cargo watch -x ...` — rewrite to `ferro serve` / `ferro serve --watch`

Replacement scaffold:
```markdown
<arguments>
Optional:
- `--port=PORT` — backend port (default 8080)
- `--frontend-port=PORT` — frontend port (default 5173)
- `--backend-only` / `--frontend-only`
- `--skip-types`
- `--watch` — enable file-watch auto-reload (500ms debounce, off by default)
</arguments>

<process>
<step name="start_server">
Default (manual):  `ferro serve`  — press `r` to rebuild, `q`/Ctrl+C to quit.
With auto-reload:  `ferro serve --watch`
</step>
</process>
```

---

### `ferro-cli/tests/serve_supervisor.rs` (NEW)

**Primary analog:** `ferro-cli/tests/docker_init_dry_run.rs:1-26` (header + `CHDIR_LOCK` + `tempdir`).
**Secondary analog:** `ferro-cli/tests/gestiscilo_fixture.rs:22-24` (`fixture_dir` via `CARGO_MANIFEST_DIR`).

**Critical difference from existing tests:** must spawn the real `ferro` binary via `env!("CARGO_BIN_EXE_ferro")` because testing child-process lifecycle and SIGINT. Existing tests call library funcs synchronously.

**Header template:**
```rust
//! Phase 145 integration tests: BackendSupervisor + keyboard + file watcher.
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

static CHDIR_LOCK: Mutex<()> = Mutex::new(());
fn fixture_dir() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal-serve") }
fn ferro_bin() -> PathBuf { PathBuf::from(env!("CARGO_BIN_EXE_ferro")) }
```

**Child-spawn pattern** (model on `serve.rs:47-60`):
```rust
let mut child = Command::new(ferro_bin())
    .args(["serve", "--backend-only"])
    .current_dir(fixture_dir())
    .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
    .spawn().expect("spawn ferro serve");
```

**Four tests** (D-36):
- `backend_only_shuts_down_cleanly` — SIGINT, assert exit within 2s
- `r_key_in_no_watch_mode_triggers_one_rebuild` — see Assumption A4 (may need `portable-pty` OR a test-only env var hook)
- `watch_mode_debounces_burst` — 10 writes in 100ms, expect one `reload triggered (file change)` within 500ms–2s
- `non_tty_stdin_ignores_r_and_shows_banner` — assert banner contains `r unavailable`, no panic on SIGINT

**SIGINT helper** (no analog — `libc` not a dev-dep; planner choice):
```rust
#[cfg(unix)]
fn send_sigint(child: &std::process::Child) {
    unsafe { libc::kill(child.id() as i32, libc::SIGINT); }
}
#[cfg(windows)]
fn send_sigint(child: &std::process::Child) { let _ = child.kill(); }
```

**Log-scraping helper** (model on `serve.rs:70-80` `BufReader::lines()`):
```rust
fn read_until(reader: impl std::io::BufRead, needle: &str, timeout: Duration) -> Option<String> {
    let deadline = std::time::Instant::now() + timeout;
    let mut out = String::new();
    for line in reader.lines().flatten() {
        out.push_str(&line); out.push('\n');
        if line.contains(needle) { return Some(out); }
        if std::time::Instant::now() > deadline { break; }
    }
    None
}
```

---

### `ferro-cli/tests/fixtures/minimal-serve/{Cargo.toml,src/main.rs}` (NEW)

**Analog (inline template):** `docker_init_dry_run.rs:38-47` (`write_fixture_project`).
**Analog (on-disk layout):** `ferro-cli/tests/fixtures/gestiscilo/Cargo.toml` (repo convention: fixtures under `tests/fixtures/<name>/`).

**Target Cargo.toml:**
```toml
[package]
name = "minimal-serve"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "minimal-serve"
path = "src/main.rs"
```

**Target src/main.rs** (sub-second `cargo run` per D-37):
```rust
fn main() {
    println!("Backend server on http://127.0.0.1:0");
    std::thread::sleep(std::time::Duration::from_millis(200));
}
```

---

### inline `#[cfg(test)] mod tests` in `serve.rs` (NEW)

**Analog (pure functions + substring asserts):** `commands/docker_init.rs:142-162`.
**Analog (TempDir + filesystem):** `commands/make_theme.rs:138-308`.

**Test module skeleton (5 tests — D-35 four plus D-17 coalesce):**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    #[test] fn render_banner_matrix() {
        let b1 = render_banner(false, true,  false, false, "127.0.0.1", 8080, 5173);
        assert!(b1.contains("watch") && b1.contains("disabled"));
        let b2 = render_banner(true,  true,  false, false, "127.0.0.1", 8080, 5173);
        assert!(b2.contains("enabled") && b2.contains("500"));
        let b3 = render_banner(false, false, false, false, "127.0.0.1", 8080, 5173);
        assert!(b3.contains("r ") && b3.contains("unavailable"));
        let b4 = render_banner(true,  false, false, false, "127.0.0.1", 8080, 5173);
        assert!(b4.contains("unavailable") && b4.contains("enabled"));
    }

    #[test] fn kill_current_noop_when_none() {
        let mut sup = BackendSupervisor {
            package_name: "x".into(), skip_types: true,
            project_path: ".".into(), types_output_path: ".".into(),
            current: None,
            shutdown: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };
        sup.kill_current();
        assert!(sup.current.is_none());
    }

    #[test] fn classify_key_table() {
        use crossterm::event::{KeyCode, KeyModifiers};
        assert!(matches!(classify_key(KeyCode::Char('r'), KeyModifiers::NONE),    Some(KbAction::Reload)));
        assert!(matches!(classify_key(KeyCode::Char('R'), KeyModifiers::SHIFT),   None));
        assert!(matches!(classify_key(KeyCode::Char('q'), KeyModifiers::NONE),    Some(KbAction::Quit)));
        assert!(matches!(classify_key(KeyCode::Char('c'), KeyModifiers::CONTROL), Some(KbAction::Quit)));
        assert!(matches!(classify_key(KeyCode::Char('x'), KeyModifiers::NONE),    None));
    }

    #[test] fn trigger_source_formatting() {
        assert_eq!(format_trigger_source(ReloadTrigger::Manual),      "manual");
        assert_eq!(format_trigger_source(ReloadTrigger::FileChanged), "file change");
    }

    #[test] fn debouncer_coalesces_burst() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src"); std::fs::create_dir(&src).unwrap();
        let (tx, rx) = channel::<ReloadTrigger>();
        let _debouncer = spawn_file_watcher_at(&src, tx).expect("debouncer init");
        let start = Instant::now();
        for i in 0..10 { std::fs::write(src.join(format!("f{i}.rs")), "fn main(){}").unwrap(); }
        let evt = rx.recv_timeout(Duration::from_secs(2)).expect("one trigger");
        assert!(matches!(evt, ReloadTrigger::FileChanged));
        assert!(start.elapsed() >= Duration::from_millis(400));
        assert!(rx.try_recv().is_err());
    }

    #[test] fn supervisor_coalesces_multiple_triggers() {
        // Requires factoring the `try_recv` drain (Pattern 8) into
        // `fn drain_triggers(&mut self, rx: &Receiver<_>) -> ReloadTrigger`.
        // Pre-populate channel with 3 Manual triggers; assert drain sees 2 extras.
        todo!("factor drain helper; implementation in 145-03-PLAN");
    }
}
```
`debouncer_coalesces_burst` requires factoring `spawn_file_watcher` so it accepts an explicit path (else it locks in `"src"` relative to CWD) — `fn spawn_file_watcher_at(dir: &Path, tx: ...) -> Option<Debouncer<...>>`.

---

## Shared Patterns

### `Arc<AtomicBool>` shutdown flag
**Source:** `serve.rs:8, 16, 99, 332, 338`. Single flag, cloned to supervisor/keyboard/watcher/ctrlc. Writers: ctrlc handler + keyboard `q` path. All readers use `shutdown.load(Ordering::SeqCst)`. Do not introduce `Mutex`, `RwLock`, or `tokio::sync::Notify`.

### Colored prefixed logging
**Source:** `serve.rs:247` (cyan `[frontend]`), `serve.rs:353-356` (magenta `[backend]`), `serve.rs:441-445` (yellow `[types]` warn). Established palette: `[backend]` magenta, `[frontend]` cyan, `[types]` blue, `Warning:` yellow, `Error:` red, `✓` green. Apply verbatim; do not introduce new colors.

### `std::sync::mpsc` only
**Source:** `serve.rs:9, 426`. Forbidden by D-16: `crossbeam-channel`, `tokio::sync::mpsc`. Clone `reload_tx` once per producer (keyboard, watcher); consumer uses `recv_timeout` + `try_recv`.

### Child-kill swallow idiom
**Source:** `serve.rs:101-102` — `let _ = child.kill(); let _ = child.wait();`. Apply verbatim in `BackendSupervisor::kill_current`. Addresses RESEARCH.md risk "Child-kill race".

### Soft-failure warning voice
**Source:** `serve.rs:306-309` (`"continuing anyway"` tone), `serve.rs:441-455` (watcher init failures). Apply to every new soft-failure branch (notify init, raw mode, missing `src/`). Serve never aborts on subsystem failure past startup validation.

---

## Key Analog Files (single-line refs)

- `ferro-cli/src/commands/serve.rs:27-96` — `spawn_with_prefix_env` (template for extraction)
- `ferro-cli/src/commands/serve.rs:98-114` — `shutdown_all` / `any_exited` (Vite keeps; backend-side replace)
- `ferro-cli/src/commands/serve.rs:282-313` — initial types regen (keep unchanged)
- `ferro-cli/src/main.rs:52-60` — `Commands::GenerateTypes { watch: bool, ... }`
- `ferro-cli/src/main.rs:484-492` — `Commands::Serve` dispatch arm (thread `watch` through)
- `ferro-cli/tests/docker_init_dry_run.rs` — `CHDIR_LOCK`, tempdir, inline fixture writer
- `ferro-cli/tests/gestiscilo_fixture.rs:22-24` — `fixture_dir()` via `CARGO_MANIFEST_DIR`
- `ferro-cli/tests/fixtures/gestiscilo/Cargo.toml` — fixture-on-disk convention
- `ferro-cli/src/commands/docker_init.rs:142-162` — minimal pure-function test module
- `ferro-cli/src/commands/make_theme.rs:138-308` — TempDir-style test module
- `ferro-cli/src/commands/generate_types.rs:838` — `generate_types_to_file(project_path, output_path)` (supervisor's regen call)

---

## PATTERN MAPPING COMPLETE
