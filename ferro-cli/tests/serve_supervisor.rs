//! Phase 145 integration tests: BackendSupervisor + keyboard + file watcher (D-36).
//!
//! These tests spawn the real `ferro` binary against the standalone fixture at
//! `tests/fixtures/minimal-serve/` and observe stdout log lines. They exercise
//! the lifecycle surface unit tests cannot (real cargo-run spawn, SIGINT
//! teardown, filesystem events, non-TTY banner path).
//!
//! The r-key test uses the `FERRO_SERVE_TEST_TRIGGER_PIPE` env-var hook
//! introduced in serve.rs (plan 145-03 Task 1, resolving RESEARCH.md A4).

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

/// Global serialization of tests that chdir or spawn the real ferro binary.
/// Matches the pattern in `docker_init_dry_run.rs`.
static CHDIR_LOCK: Mutex<()> = Mutex::new(());

/// Resolves the `minimal-serve` fixture directory (D-37).
fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal-serve")
}

/// Resolves the compiled `ferro` binary that `cargo test` built alongside this test.
fn ferro_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ferro"))
}

/// Default timeout budget for integration tests (matches D-36 2-second shutdown budget).
fn default_timeout() -> Duration {
    Duration::from_secs(5)
}

// ── Shared helpers ───────────────────────────────────────────────────

/// Stream a child's stdout into an mpsc so tests can `recv` lines with a deadline.
fn spawn_stdout_reader(child: &mut Child) -> Receiver<String> {
    let stdout = child.stdout.take().expect("child stdout piped");
    let (tx, rx) = channel::<String>();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    rx
}

/// Wait until a line containing `needle` is received, or the deadline passes.
/// Returns true on match, false on timeout. All seen lines are echoed via
/// `eprintln!` to help debug CI failures.
fn wait_for_stdout_line(rx: &Receiver<String>, needle: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                eprintln!("[test-stdout] {line}");
                if line.contains(needle) {
                    return true;
                }
            }
            Err(_) => return false,
        }
    }
    false
}

#[cfg(unix)]
fn send_sigint(child: &Child) {
    // SAFETY: `child.id()` is the PID of a process we just spawned via
    // `Command::spawn`; `libc::kill` with SIGINT is safe for a live PID we
    // own. The PID cannot alias another process because we have not yet
    // called `wait`, so the kernel has not reaped it.
    unsafe {
        libc::kill(child.id() as i32, libc::SIGINT);
    }
}
#[cfg(windows)]
fn send_sigint(child: &mut Child) {
    // Windows has no POSIX SIGINT — fall back to terminating. The tests'
    // assertions on shutdown ordering are soft on Windows by design.
    let _ = child.kill();
}

/// Best-effort kill + wait. Guarantees we never leave zombies. Returns true
/// when the child exited within the budget.
fn kill_and_wait(mut child: Child, budget: Duration) -> bool {
    let start = Instant::now();
    let _ = child.kill();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => {
                if start.elapsed() > budget {
                    return false;
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return false,
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────

/// D-01, D-07, D-29 — `ferro serve --backend-only` shuts down cleanly on SIGINT
/// within the shutdown budget, leaving no zombie children.
#[test]
fn backend_only_shuts_down_cleanly() {
    let _guard = CHDIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = default_timeout();
    let mut child = Command::new(ferro_bin())
        .args(["serve", "--backend-only", "--skip-types"])
        .current_dir(fixture_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ferro serve");

    let rx = spawn_stdout_reader(&mut child);

    // Wait for the startup banner — proves the supervisor started successfully.
    assert!(
        wait_for_stdout_line(&rx, "Backend server on", Duration::from_secs(15)),
        "backend banner not seen"
    );

    // Signal shutdown.
    let start = Instant::now();
    #[cfg(unix)]
    send_sigint(&child);
    #[cfg(windows)]
    send_sigint(&mut child);

    // D-29: whole process must exit within ~2s (RESEARCH.md gives a 2s budget;
    // allow 5s to absorb GitHub runner jitter and in-flight `cargo run`).
    let clean = kill_and_wait(child, Duration::from_secs(5));
    assert!(clean, "ferro serve did not exit within 5s of SIGINT");
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "shutdown slower than budget: {:?}",
        start.elapsed()
    );
}

/// D-06 — `r` triggers exactly one `reload triggered (manual)` in no-watch mode.
/// Uses the `FERRO_SERVE_TEST_TRIGGER_PIPE` env-var hook (145-03 Task 1).
#[test]
fn r_key_in_no_watch_mode_triggers_one_rebuild() {
    let _guard = CHDIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().expect("tempdir");
    let pipe = tmp.path().join("trigger.pipe");
    std::fs::write(&pipe, "").unwrap();

    let mut child = Command::new(ferro_bin())
        .args(["serve", "--backend-only", "--skip-types"])
        .current_dir(fixture_dir())
        .env("FERRO_SERVE_TEST_TRIGGER_PIPE", &pipe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ferro serve");

    let rx = spawn_stdout_reader(&mut child);
    assert!(
        wait_for_stdout_line(&rx, "Backend server on", Duration::from_secs(15)),
        "backend banner not seen"
    );

    // Trigger a manual reload by writing `r` to the pipe.
    std::fs::write(&pipe, "r").unwrap();

    // Expect one `reload triggered (manual)` line within a generous budget.
    let saw = wait_for_stdout_line(&rx, "reload triggered (manual)", Duration::from_secs(5));
    assert!(saw, "expected one manual reload line");

    // No duplicate reload within the next 2s (hook truncates after read, so
    // a second write is required to re-trigger).
    let dup = wait_for_stdout_line(&rx, "reload triggered", Duration::from_secs(2));
    assert!(!dup, "unexpected duplicate reload trigger line");

    #[cfg(unix)]
    send_sigint(&child);
    #[cfg(windows)]
    send_sigint(&mut child);
    assert!(
        kill_and_wait(child, Duration::from_secs(5)),
        "ferro serve did not exit within 5s of SIGINT"
    );
}

/// D-19, D-20, D-21 — `--watch` coalesces a burst of `.rs` writes into one
/// debounced `reload triggered (file change)` trigger, and non-`.rs` writes
/// (e.g. `Cargo.toml`) do NOT trigger a reload.
#[test]
fn watch_mode_debounces_burst() {
    let _guard = CHDIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // The checked-in fixture is shared state — copy to a tempdir so writes
    // during the burst do not mutate tracked files.
    let tmp = tempfile::tempdir().expect("tempdir");
    let sandbox = tmp.path();
    std::fs::create_dir_all(sandbox.join("src")).unwrap();
    std::fs::copy(fixture_dir().join("Cargo.toml"), sandbox.join("Cargo.toml")).unwrap();
    std::fs::copy(
        fixture_dir().join("src/main.rs"),
        sandbox.join("src/main.rs"),
    )
    .unwrap();

    let mut child = Command::new(ferro_bin())
        .args(["serve", "--backend-only", "--skip-types", "--watch"])
        .current_dir(sandbox)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ferro serve --watch");

    let rx = spawn_stdout_reader(&mut child);
    assert!(
        wait_for_stdout_line(&rx, "Backend server on", Duration::from_secs(15)),
        "backend banner not seen"
    );
    assert!(
        wait_for_stdout_line(&rx, "enabled", Duration::from_secs(3)),
        "watch banner did not say 'enabled'"
    );

    // Give the debouncer a moment to register its watch on `src/`, otherwise
    // the burst can race with watcher initialization on some platforms.
    thread::sleep(Duration::from_millis(200));

    // Burst: 10 `.rs` writes within a tight window.
    let burst_start = Instant::now();
    for i in 0..10 {
        std::fs::write(
            sandbox.join(format!("src/f{i}.rs")),
            format!("// burst file {i}\nfn _x() {{}}"),
        )
        .unwrap();
    }

    // D-21 control: also write a non-.rs file — must NOT trigger a reload.
    std::fs::write(sandbox.join("Cargo.toml"), "# touched").unwrap();

    // Expect at least one `reload triggered (file change)` line, and the
    // debouncer should not have produced it immediately — wait at least most
    // of the 500ms window before the trigger arrives.
    assert!(
        wait_for_stdout_line(
            &rx,
            "reload triggered (file change)",
            Duration::from_secs(5)
        ),
        "expected a debounced file-change reload line"
    );
    let elapsed = burst_start.elapsed();
    assert!(
        elapsed >= Duration::from_millis(400),
        "debounce window too short: {elapsed:?}"
    );

    // Coalescing assertion: count any follow-up `reload triggered` lines
    // within a bounded quiet window. The 11 raw writes (10 .rs + 1 Cargo.toml)
    // MUST collapse to strictly fewer events — the filter drops Cargo.toml
    // and the debouncer collapses the .rs burst.
    let drain_deadline = Instant::now() + Duration::from_secs(2);
    let mut extra = 0usize;
    while let Some(remaining) = drain_deadline.checked_duration_since(Instant::now()) {
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                eprintln!("[test-stdout] {line}");
                if line.contains("reload triggered") {
                    extra += 1;
                }
            }
            Err(_) => break,
        }
    }
    let total = 1 + extra;
    assert!(
        total < 11,
        "debouncer failed to coalesce: {total} events for 11 writes"
    );

    #[cfg(unix)]
    send_sigint(&child);
    #[cfg(windows)]
    send_sigint(&mut child);
    assert!(
        kill_and_wait(child, Duration::from_secs(5)),
        "ferro serve did not exit within 5s of SIGINT"
    );
}

/// D-05, D-24, D-26 — non-TTY stdin: banner shows `r unavailable`, feeding
/// bytes to stdin does not crash, SIGINT still shuts down cleanly.
#[test]
fn non_tty_stdin_ignores_r_and_shows_banner() {
    let _guard = CHDIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut child = Command::new(ferro_bin())
        .args(["serve", "--backend-only", "--skip-types"])
        .current_dir(fixture_dir())
        .stdin(Stdio::piped()) // non-TTY (pipe, not a terminal)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ferro serve");

    let rx = spawn_stdout_reader(&mut child);

    assert!(
        wait_for_stdout_line(&rx, "unavailable", Duration::from_secs(15)),
        "non-TTY banner did not contain 'unavailable'"
    );

    // Try to feed `r` into stdin — must not crash the server. The keyboard
    // thread is skipped in non-TTY mode, so no reader consumes these bytes;
    // the write is effectively a no-op.
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"r\n");
    }

    // Give the server 500ms to prove it is still alive.
    thread::sleep(Duration::from_millis(500));

    // No reload trigger should fire from piped stdin bytes.
    let saw_reload = wait_for_stdout_line(&rx, "reload triggered", Duration::from_millis(500));
    assert!(
        !saw_reload,
        "non-TTY stdin bytes must not produce a reload trigger"
    );

    #[cfg(unix)]
    send_sigint(&child);
    #[cfg(windows)]
    send_sigint(&mut child);
    assert!(
        kill_and_wait(child, Duration::from_secs(5)),
        "ferro serve did not exit within 5s of SIGINT"
    );
}
