use super::clean;
use console::style;
use crossterm::event::{KeyCode, KeyModifiers};
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

// Phase 145 — pure-function contracts and enums consumed by the
// BackendSupervisor that 145-02b will wire in. Bodies are filled by 145-02a
// against the inline test oracle below so later plans cannot drift.

/// Reload trigger dispatched to the BackendSupervisor over an mpsc channel (D-06, D-20).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // variants constructed by 145-02a/02b; referenced by tests.
pub(super) enum ReloadTrigger {
    Manual,
    FileChanged,
}

/// Result of classifying a keypress in the keyboard thread (D-06, D-07, D-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // variants constructed by 145-02a; referenced by tests.
pub(super) enum KbAction {
    Reload,
    Quit,
}

/// Renders the startup banner. Pure function — `is_tty` and `is_watch` are explicit
/// so tests do not depend on the real stdin state (D-05, D-24). Body emits the
/// spec-verbatim literal from
/// docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md §CLI surface.
#[allow(dead_code, clippy::too_many_arguments)] // consumed by 02b; referenced by tests.
pub(super) fn render_banner(
    is_watch: bool,
    is_tty: bool,
    backend_only: bool,
    frontend_only: bool,
    backend_host: &str,
    backend_port: u16,
    vite_port: u16,
) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    if !frontend_only {
        let _ = writeln!(s, "Backend server on http://{backend_host}:{backend_port}");
    }
    if !backend_only {
        let _ = writeln!(s, "Frontend server on http://127.0.0.1:{vite_port}");
    }
    if !frontend_only {
        let _ = writeln!(s);
        if is_tty {
            let _ = writeln!(s, "  r        rebuild backend + regenerate types");
        } else {
            let _ = writeln!(s, "  r        unavailable (non-TTY stdin)");
        }
        let _ = writeln!(s, "  q        quit    (or Ctrl+C)");
        if is_watch {
            let _ = writeln!(s, "  watch    enabled  (debounce 500ms)");
        } else {
            let _ = writeln!(
                s,
                "  watch    disabled  (pass --watch to auto-reload on file changes)"
            );
        }
    }
    s
}

/// Classifies a keypress. Lowercase `r` → Reload; `q` or Ctrl-C → Quit; else None (D-08).
/// Signature uses the final crossterm types directly — no placeholder, no Plan-02 rewrite.
#[allow(dead_code)] // consumed by 02b keyboard thread; referenced by tests.
pub(super) fn classify_key(code: KeyCode, modifiers: KeyModifiers) -> Option<KbAction> {
    match (code, modifiers) {
        (KeyCode::Char('r'), KeyModifiers::NONE) => Some(KbAction::Reload),
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            Some(KbAction::Quit)
        }
        _ => None,
    }
}

/// Formats a trigger source for the `[backend] reload triggered ({source})` log line (D-27, D-28).
#[allow(dead_code)] // consumed by 02b supervisor; referenced by tests.
pub(super) fn format_trigger_source(t: ReloadTrigger) -> &'static str {
    match t {
        ReloadTrigger::Manual => "manual",
        ReloadTrigger::FileChanged => "file change",
    }
}

/// Whether to spawn the keyboard thread. Equivalent to `is_tty`, isolated for testability (D-24).
#[allow(dead_code)] // consumed by 02b keyboard gate; referenced by tests.
pub(super) fn should_spawn_keyboard(is_tty: bool) -> bool {
    is_tty
}

/// Spawns a child process and streams its stdout/stderr to the terminal with a
/// colored prefix. The shutdown flag stops the reader threads when servers shut
/// down. Extracted from `ProcessManager::spawn_with_prefix_env` so 02b's
/// `BackendSupervisor` can reuse the same piping logic without duplicating it.
fn spawn_child_with_prefix(
    command: &str,
    args: &[&str],
    cwd: Option<&Path>,
    prefix: &str,
    color: console::Color,
    env_vars: &[(&str, &str)],
    shutdown: Arc<AtomicBool>,
) -> Result<Child, String> {
    let mut cmd = Command::new(command);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn {command}: {e}"))?;

    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");
    let prefix_out = prefix.to_string();
    let prefix_err = prefix.to_string();
    let sd_out = shutdown.clone();
    let sd_err = shutdown;

    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if sd_out.load(Ordering::SeqCst) {
                break;
            }
            if let Ok(line) = line {
                println!("{} {}", style(&prefix_out).fg(color).bold(), line);
            }
        }
    });

    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if sd_err.load(Ordering::SeqCst) {
                break;
            }
            if let Ok(line) = line {
                eprintln!("{} {}", style(&prefix_err).fg(color).bold(), line);
            }
        }
    });

    Ok(child)
}

struct ProcessManager {
    children: Vec<Child>,
    shutdown: Arc<AtomicBool>,
}

impl ProcessManager {
    fn new() -> Self {
        Self {
            children: Vec::new(),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    fn spawn_with_prefix(
        &mut self,
        command: &str,
        args: &[&str],
        cwd: Option<&Path>,
        prefix: &str,
        color: console::Color,
    ) -> Result<(), String> {
        self.spawn_with_prefix_env(command, args, cwd, prefix, color, &[])
    }

    fn spawn_with_prefix_env(
        &mut self,
        command: &str,
        args: &[&str],
        cwd: Option<&Path>,
        prefix: &str,
        color: console::Color,
        env_vars: &[(&str, &str)],
    ) -> Result<(), String> {
        let child = spawn_child_with_prefix(
            command,
            args,
            cwd,
            prefix,
            color,
            env_vars,
            self.shutdown.clone(),
        )?;
        self.children.push(child);
        Ok(())
    }

    fn shutdown_all(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        for child in &mut self.children {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn any_exited(&mut self) -> bool {
        for child in &mut self.children {
            if let Ok(Some(_)) = child.try_wait() {
                return true;
            }
        }
        false
    }
}

fn get_package_name() -> Result<String, String> {
    let cargo_toml = Path::new("Cargo.toml");
    let content = std::fs::read_to_string(cargo_toml)
        .map_err(|e| format!("Failed to read Cargo.toml: {e}"))?;

    let parsed: toml::Value = content
        .parse()
        .map_err(|e| format!("Failed to parse Cargo.toml: {e}"))?;

    parsed
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Could not find package name in Cargo.toml".to_string())
}

fn validate_ferro_project(backend_only: bool, frontend_only: bool) -> Result<(), String> {
    let cargo_toml = Path::new("Cargo.toml");
    let frontend_dir = Path::new("frontend");

    if !frontend_only && !cargo_toml.exists() {
        return Err("No Cargo.toml found. Are you in a Ferro project directory?".into());
    }

    if !backend_only && !frontend_dir.exists() {
        return Err("No frontend directory found. Are you in a Ferro project directory?".into());
    }

    Ok(())
}

fn ensure_npm_dependencies() -> Result<(), String> {
    let frontend_path = Path::new("frontend");
    let node_modules = frontend_path.join("node_modules");

    if !node_modules.exists() {
        println!("{}", style("Installing frontend dependencies...").yellow());
        let npm_install = Command::new("npm")
            .args(["install"])
            .current_dir(frontend_path)
            .status()
            .map_err(|e| format!("Failed to run npm install: {e}"))?;

        if !npm_install.success() {
            return Err("Failed to install npm dependencies".into());
        }
        println!(
            "{}",
            style("Frontend dependencies installed successfully.").green()
        );
    }

    Ok(())
}

fn find_available_port(start: u16, max_attempts: u16) -> u16 {
    for offset in 0..max_attempts {
        let port = start + offset;
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    start
}

pub fn run(
    port: u16,
    frontend_port: u16,
    backend_only: bool,
    frontend_only: bool,
    skip_types: bool,
    watch: bool,
) {
    // 02b wires the file watcher / supervisor. Bind here to keep the build green.
    let _ = watch;

    // Load .env file from current directory
    let _ = dotenvy::dotenv();

    // Resolve backend host and port from env vars (matching ServerConfig defaults)
    let backend_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    // Resolve ports: CLI args take precedence, then env vars, then defaults
    let backend_port = if port != 8080 {
        // CLI argument was explicitly provided (different from default)
        port
    } else {
        // Use env var or default (8080)
        std::env::var("SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080)
    };

    let requested_vite_port = if frontend_port != 5173 {
        // CLI argument was explicitly provided
        frontend_port
    } else {
        // Use env var or default
        std::env::var("VITE_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(frontend_port)
    };

    let vite_port = find_available_port(requested_vite_port, 10);
    if vite_port != requested_vite_port {
        println!(
            "{} Port {} in use, using {} instead",
            style("[frontend]").cyan().bold(),
            requested_vite_port,
            vite_port
        );
    }

    // Set VITE_DEV_SERVER so InertiaConfig picks up the resolved port
    std::env::set_var("VITE_DEV_SERVER", format!("http://localhost:{vite_port}"));

    // Auto-cleanup old build artifacts (silent, non-blocking)
    // Configurable via CARGO_SWEEP_DAYS (default: 7, set to 0 to disable)
    let sweep_days: u32 = std::env::var("CARGO_SWEEP_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);

    if sweep_days > 0 {
        if let Some(cleaned) = clean::run_silent(sweep_days) {
            println!("{} {}", style("♻").cyan(), cleaned);
        }
    }

    println!();
    println!(
        "{}",
        style("Starting Ferro development servers...").cyan().bold()
    );
    println!();

    // Validate project
    if let Err(e) = validate_ferro_project(backend_only, frontend_only) {
        eprintln!("{} {}", style("Error:").red().bold(), e);
        std::process::exit(1);
    }

    // Generate TypeScript types on startup (unless skipped or frontend-only)
    if !skip_types && !frontend_only {
        let project_path = Path::new(".");
        let output_path = project_path.join("frontend/src/types/inertia-props.ts");

        println!("{}", style("Generating TypeScript types...").cyan());
        match super::generate_types::generate_types_to_file(project_path, &output_path) {
            Ok(0) => {
                println!(
                    "{}",
                    style("No InertiaProps structs found (skipping type generation)").dim()
                );
            }
            Ok(count) => {
                println!(
                    "{} Generated {} type(s) to {}",
                    style("✓").green(),
                    count,
                    output_path.display()
                );
            }
            Err(e) => {
                // Don't fail, just warn - types are a nice-to-have
                eprintln!(
                    "{} Failed to generate types: {} (continuing anyway)",
                    style("Warning:").yellow(),
                    e
                );
            }
        }
        println!();
    }

    // Ensure npm dependencies are installed (only if running frontend)
    if !backend_only {
        if let Err(e) = ensure_npm_dependencies() {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }

    let mut manager = ProcessManager::new();
    let shutdown = manager.shutdown.clone();

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        println!();
        println!("{}", style("Shutting down servers...").yellow());
        shutdown.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Start backend via a plain `cargo run`. In 02b the BackendSupervisor will
    // own this child and react to `r` / file-change triggers; for 02a we keep
    // the no-watch happy path working with a one-shot spawn, matching
    // pre-phase behavior except that failed compiles no longer auto-respawn.
    if !frontend_only {
        let package_name = match get_package_name() {
            Ok(name) => name,
            Err(e) => {
                eprintln!("{} {}", style("Error:").red().bold(), e);
                std::process::exit(1);
            }
        };

        println!(
            "{} Backend server on http://{}:{}",
            style("[backend]").magenta().bold(),
            backend_host,
            backend_port
        );

        if let Err(e) = manager.spawn_with_prefix(
            "cargo",
            &["run", "--bin", &package_name],
            None,
            "[backend] ",
            console::Color::Magenta,
        ) {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }

    // Start frontend with npm/vite
    if !backend_only {
        println!(
            "{} Frontend server on http://127.0.0.1:{}",
            style("[frontend]").cyan().bold(),
            vite_port
        );

        let frontend_path = Path::new("frontend");
        let vite_port_str = vite_port.to_string();

        if let Err(e) = manager.spawn_with_prefix_env(
            "npm",
            &["run", "dev", "--", "--port", &vite_port_str, "--strictPort"],
            Some(frontend_path),
            "[frontend]",
            console::Color::Cyan,
            &[],
        ) {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            manager.shutdown_all();
            std::process::exit(1);
        }
    }

    // File-watch + types-regen threading moves into BackendSupervisor in 02b.
    // 02a leaves the initial startup types-regen (above) in place and does not
    // start any file watcher here.

    println!();
    println!("{}", style("Press Ctrl+C to stop all servers").dim());
    println!();

    // Wait for shutdown signal or process exit
    while !manager.shutdown.load(Ordering::SeqCst) {
        thread::sleep(std::time::Duration::from_millis(100));

        // Check if any child process has exited
        if manager.any_exited() {
            manager.shutdown.store(true, Ordering::SeqCst);
            break;
        }
    }

    manager.shutdown_all();
    println!("{}", style("Servers stopped.").green());
}

#[cfg(test)]
mod tests {
    use super::*;

    // D-05, D-24 — banner renders correctly for all four (watch × tty) combinations,
    // EXACT STRING match against the spec banner literal from
    // docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md §CLI surface.
    //
    // These literals are the test oracle for 02a's `render_banner` body.
    // If 02a emits anything different, these assertions fail.
    #[test]
    fn render_banner_matrix() {
        let b_watch_off_tty = "Backend server on http://127.0.0.1:8080\n\
                               Frontend server on http://127.0.0.1:5173\n\
                               \n\
                               \x20\x20r        rebuild backend + regenerate types\n\
                               \x20\x20q        quit    (or Ctrl+C)\n\
                               \x20\x20watch    disabled  (pass --watch to auto-reload on file changes)\n";
        let b_watch_on_tty = "Backend server on http://127.0.0.1:8080\n\
                              Frontend server on http://127.0.0.1:5173\n\
                              \n\
                              \x20\x20r        rebuild backend + regenerate types\n\
                              \x20\x20q        quit    (or Ctrl+C)\n\
                              \x20\x20watch    enabled  (debounce 500ms)\n";
        let b_watch_off_non_tty = "Backend server on http://127.0.0.1:8080\n\
                                   Frontend server on http://127.0.0.1:5173\n\
                                   \n\
                                   \x20\x20r        unavailable (non-TTY stdin)\n\
                                   \x20\x20q        quit    (or Ctrl+C)\n\
                                   \x20\x20watch    disabled  (pass --watch to auto-reload on file changes)\n";
        let b_watch_on_non_tty = "Backend server on http://127.0.0.1:8080\n\
                                  Frontend server on http://127.0.0.1:5173\n\
                                  \n\
                                  \x20\x20r        unavailable (non-TTY stdin)\n\
                                  \x20\x20q        quit    (or Ctrl+C)\n\
                                  \x20\x20watch    enabled  (debounce 500ms)\n";

        assert_eq!(
            render_banner(false, true, false, false, "127.0.0.1", 8080, 5173),
            b_watch_off_tty,
        );
        assert_eq!(
            render_banner(true, true, false, false, "127.0.0.1", 8080, 5173),
            b_watch_on_tty,
        );
        assert_eq!(
            render_banner(false, false, false, false, "127.0.0.1", 8080, 5173),
            b_watch_off_non_tty,
        );
        assert_eq!(
            render_banner(true, false, false, false, "127.0.0.1", 8080, 5173),
            b_watch_on_non_tty,
        );
    }

    // D-08 — lowercase `r` only; uppercase R / unrelated keys return None.
    #[test]
    fn classify_key_table() {
        assert_eq!(
            classify_key(KeyCode::Char('r'), KeyModifiers::NONE),
            Some(KbAction::Reload)
        );
        assert_eq!(classify_key(KeyCode::Char('R'), KeyModifiers::SHIFT), None);
        assert_eq!(
            classify_key(KeyCode::Char('q'), KeyModifiers::NONE),
            Some(KbAction::Quit)
        );
        assert_eq!(
            classify_key(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Some(KbAction::Quit)
        );
        assert_eq!(classify_key(KeyCode::Char('x'), KeyModifiers::NONE), None);
    }

    // D-27, D-28 — source label mapping.
    #[test]
    fn trigger_source_formatting() {
        assert_eq!(format_trigger_source(ReloadTrigger::Manual), "manual");
        assert_eq!(
            format_trigger_source(ReloadTrigger::FileChanged),
            "file change"
        );
    }

    // D-24 — should_spawn_keyboard is equivalent to the is_tty input.
    #[test]
    fn should_spawn_keyboard_gated_on_tty() {
        assert!(should_spawn_keyboard(true));
        assert!(!should_spawn_keyboard(false));
    }

    // D-11 — kill_current is a no-op when current = None.
    #[test]
    #[ignore = "implemented in 145-02b-PLAN — BackendSupervisor lives there"]
    fn kill_current_noop_when_none() {
        // 02b body: construct BackendSupervisor::new(...), assert current.is_none(),
        // call kill_current(), assert current is still None and no panic.
    }

    // D-17 — multiple pending triggers coalesce into one cycle.
    #[test]
    #[ignore = "implemented in 145-02b-PLAN — drain_triggers lives there"]
    fn supervisor_coalesces_multiple_triggers() {
        // 02b body: build mpsc<ReloadTrigger>, send 3 triggers, drop tx, call
        // BackendSupervisor::drain_triggers(&rx, first), assert latest matches,
        // assert rx.try_recv().is_err().
    }

    // D-19 — debouncer coalesces a burst of *.rs writes into exactly one FileChanged.
    #[test]
    #[ignore = "implemented in 145-02b-PLAN — spawn_file_watcher_at lives there"]
    fn debouncer_coalesces_burst() {
        // 02b body (per PATTERNS.md §inline tests):
        //   let tmp = tempfile::TempDir::new().unwrap();
        //   let src = tmp.path().join("src");
        //   std::fs::create_dir(&src).unwrap();
        //   let (tx, rx) = std::sync::mpsc::channel::<ReloadTrigger>();
        //   let _debouncer = spawn_file_watcher_at(&src, Duration::from_millis(50), tx)
        //       .expect("debouncer init");
        //   let start = Instant::now();
        //   for i in 0..10 {
        //       std::fs::write(src.join(format!("f{i}.rs")), "fn main(){}").unwrap();
        //   }
        //   let evt = rx.recv_timeout(Duration::from_secs(2)).expect("one trigger");
        //   assert!(matches!(evt, ReloadTrigger::FileChanged));
        //   assert!(start.elapsed() >= Duration::from_millis(40));
        //   assert!(rx.recv_timeout(Duration::from_millis(300)).is_err());
    }
}
