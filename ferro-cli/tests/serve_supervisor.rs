//! Phase 145 integration tests: BackendSupervisor + keyboard + file watcher (D-36).
//!
//! Each test is gated with `ignore` until 145-03-PLAN wires the real supervisor.
//! Run locally with: cargo test -p ferro-cli --test serve_supervisor -- --ignored

#![allow(dead_code)] // helpers below are referenced only when tests un-ignore in Plan 03

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

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

// ── Tests ────────────────────────────────────────────────────────────

/// D-01, D-07, D-29 — `ferro serve --backend-only` shuts down cleanly on SIGINT
/// within 2 seconds, leaving no zombie children.
#[test]
#[ignore = "implemented in 145-03-PLAN"]
fn backend_only_shuts_down_cleanly() {
    let _g = CHDIR_LOCK.lock().unwrap();
    let _ = (fixture_dir(), ferro_bin(), default_timeout());
    // Plan 03 body:
    //   1. Command::new(ferro_bin()).args(["serve","--backend-only"])
    //        .current_dir(fixture_dir()).stdin(Stdio::piped()).stdout(Stdio::piped())
    //        .stderr(Stdio::piped()).spawn()
    //   2. Wait for "Backend server on" on stdout (bounded).
    //   3. send_sigint(&child).
    //   4. Assert child.wait_timeout(2s) returned Some(_) and no zombie remains.
}

/// D-06 — `r` key in no-watch mode triggers exactly one `reload triggered (manual)`.
#[test]
#[ignore = "implemented in 145-03-PLAN (see RESEARCH.md Assumption A4 re pty vs test hook)"]
fn r_key_in_no_watch_mode_triggers_one_rebuild() {
    let _g = CHDIR_LOCK.lock().unwrap();
    let _ = (fixture_dir(), ferro_bin(), default_timeout());
    // Plan 03: use portable-pty (add dev-dep) OR a test-only env-var hook in serve.rs.
}

/// D-19, D-20 — `--watch` burst of 10 file writes in 100ms coalesces to one rebuild
/// after the debounce window (≥500ms, ≤2s).
#[test]
#[ignore = "implemented in 145-03-PLAN"]
fn watch_mode_debounces_burst() {
    let _g = CHDIR_LOCK.lock().unwrap();
    let _ = (fixture_dir(), ferro_bin(), default_timeout());
    // Plan 03 body:
    //   - spawn ferro serve --watch against a copy of the fixture in a tempdir
    //   - write src/f0.rs..src/f9.rs within 100ms
    //   - scan stdout for exactly one "reload triggered (file change)" within 500ms..2s
    //   - verify try_recv on the follow-up channel is empty
    //   - shutdown cleanly
}

/// D-05, D-24, D-26 — non-TTY stdin: banner shows `r unavailable`, no crash, clean shutdown.
#[test]
#[ignore = "implemented in 145-03-PLAN"]
fn non_tty_stdin_ignores_r_and_shows_banner() {
    let _g = CHDIR_LOCK.lock().unwrap();
    let _ = (fixture_dir(), ferro_bin(), default_timeout());
    // Plan 03 body:
    //   - spawn ferro serve with stdin = Stdio::piped() (cargo-test default is non-TTY)
    //   - assert stdout contains "unavailable"
    //   - write some bytes to stdin; assert no crash
    //   - SIGINT; assert clean exit
}
