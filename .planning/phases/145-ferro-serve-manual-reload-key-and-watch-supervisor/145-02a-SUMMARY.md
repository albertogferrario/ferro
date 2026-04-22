---
phase: 145-ferro-serve-manual-reload-key-and-watch-supervisor
plan: 02a
subsystem: cli
tags: [cli, contracts, deletions, crossterm, serve-supervisor]

# Dependency graph
requires:
  - "145-01-SUMMARY.md — Wave 0 contracts, enums, inline test skeleton, spec-verbatim banner oracle"
provides:
  - "`--watch` flag on `Commands::Serve` (off by default, help text `Enable file-watch auto-reload (500ms debounce)`)"
  - "6-arg `commands::serve::run(port, frontend_port, backend_only, frontend_only, skip_types, watch)` with `let _ = watch;` bridge pending 02b"
  - "`spawn_child_with_prefix` free function — piping extraction used by `ProcessManager::spawn_with_prefix_env` and (in 02b) by `BackendSupervisor`"
  - "Four pure-helper bodies (`render_banner`, `classify_key`, `format_trigger_source`, `should_spawn_keyboard`) filled against the spec-verbatim banner literal"
  - "Deletion of `ensure_cargo_watch()`, `start_type_watcher()`, the `cargo watch -x` backend spawn, and all `cargo-watch` references"
affects:
  - "145-02b (un-ignores supervisor-dependent tests, introduces BackendSupervisor + keyboard thread + debouncer, consumes the `watch` param)"
  - "145-03 (un-ignores the four integration tests in serve_supervisor.rs)"
  - "145-04 (docs rewrite)"

# Tech tracking
tech-stack:
  added: []          # crossterm "0.29" already added in Plan 01
  patterns:
    - "Free-function extraction of stdout/stderr piping (Pattern 2) — `spawn_child_with_prefix(cmd, args, cwd, prefix, color, env_vars, shutdown) -> Result<Child, String>` used both by `ProcessManager` and (in 02b) by `BackendSupervisor`"
    - "`#[allow(dead_code)]` on pure helpers — they're referenced only by tests in 02a; 02b wires them into production and the allow can be removed then"
    - "Banner rendering via `std::fmt::Write + writeln!` into a `String` accumulator — exact-string oracle requires no extra whitespace from `format!`"

key-files:
  created: []
  modified:
    - "ferro-cli/src/main.rs (+9/-0) — add `--watch` bool field on `Commands::Serve`; thread through dispatch arm"
    - "ferro-cli/src/commands/serve.rs (+137/-202 net; ~634 lines final) — all deletions + extraction + pure-helper bodies + un-ignored 4 unit tests"
    - ".planning/phases/145-ferro-serve-manual-reload-key-and-watch-supervisor/deferred-items.md (+18) — log pre-existing rustfmt drift in ferro-json-ui/src/render.rs:2286"

key-decisions:
  - "Changed backend spawn from `cargo watch -x \"run --bin <pkg>\"` to plain `cargo run --bin <pkg>` (Rule 2 — critical functionality). Otherwise `ferro serve` would invoke a binary we just stopped installing. Failed compiles no longer auto-respawn; matches the spec's explicit non-goal on auto-respawn and pre-stages the `BackendSupervisor` behavior."
  - "Kept `#[allow(dead_code)]` on all four pure helpers (and the two enums) — they're consumed only by tests in 02a, production wiring comes in 02b. Removing the allows would require either dummy production refs or a build warning."
  - "Scoped the fmt/clippy gate to `-p ferro-cli` (mirroring Plan 01's posture). Workspace-wide `cargo fmt --all -- --check` fails on a pre-existing unrelated `ferro-json-ui/src/render.rs:2286` drift; logged to deferred-items.md. Workspace-wide clippy has pre-existing `SwitchProps.compact` compile errors, also out of scope."

patterns-established:
  - "Deletion-and-bridge pattern: when removing a subsystem (cargo-watch) whose replacement lives in a later plan (02b supervisor), replace the call site with the minimal direct alternative (plain `cargo run`) so the happy path still works without waiting for the full replacement"

requirements-completed: [D-01, D-02, D-03, D-04, D-05, D-08, D-23, D-24, D-27, D-28, D-30, D-32, D-34, D-35]

# Metrics
duration: 8min
completed: 2026-04-22
---

# Phase 145 Plan 02a: Deps, clap surface, deletions, pure helpers Summary

**Removed the external `cargo-watch` dependency from `ferro serve`, added the `--watch` opt-in flag on the clap surface, extracted `spawn_child_with_prefix` as a shared piping helper, and filled four pure-helper bodies (`render_banner` / `classify_key` / `format_trigger_source` / `should_spawn_keyboard`) against the spec-verbatim banner oracle — four unit tests un-ignored and passing.**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-04-22T15:36:16Z
- **Completed:** 2026-04-22T15:44:34Z
- **Tasks:** 2
- **Files modified:** 3 (`ferro-cli/src/main.rs`, `ferro-cli/src/commands/serve.rs`, `.planning/…/deferred-items.md`)
- **Net line change in `serve.rs`:** +137 / −202 (634 lines final, deletion-heavy)

## Accomplishments

### Clap surface (Task 1)
- `--watch` flag added to `Commands::Serve` in `ferro-cli/src/main.rs` with help text `Enable file-watch auto-reload (500ms debounce)` and `default false`. No `short = 'w'` (would collide with `generate-types -w`).
- Dispatch arm destructures `watch` and threads it into `commands::serve::run` as the 6th positional param.
- `serve::run` signature extended with `watch: bool`; a temporary `let _ = watch;` binding keeps the build green — full consumption is 02b's job.

### Deletions (Task 2)
- `fn ensure_cargo_watch()` and its call site gone. `cargo install cargo-watch` is no longer triggered anywhere. Net supply-chain reduction.
- `fn start_type_watcher()` and its `thread::spawn(...)` wrapper gone. Types-regen threading will be re-introduced inside `BackendSupervisor::regenerate_types` in 02b.
- Backend spawn no longer uses `cargo watch -x "run --bin <pkg>"`. It now calls `cargo run --bin <pkg>` directly via `ProcessManager::spawn_with_prefix`. Failed compiles no longer auto-respawn (matches the spec's explicit non-goal on auto-respawn — see §Non-goals of `docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md`).
- Unused imports removed: `notify::{Config, RecommendedWatcher, RecursiveMode, Watcher}`, `std::sync::mpsc::channel`, `std::time::Duration`.

### Extraction (Task 2)
- `fn spawn_child_with_prefix(command, args, cwd, prefix, color, env_vars, shutdown) -> Result<Child, String>` is now a free function at module scope. `ProcessManager::spawn_with_prefix_env` delegates to it, pushing the returned `Child` into its `children` vec. No behavior change for the Vite child; identical stdout/stderr piping, identical colored prefix, identical shutdown-flag semantics.

### Pure-helper bodies (Task 2)

All four bodies are concrete and driven by the inline test oracle Plan 01 seeded. No `todo!()` remains anywhere in `serve.rs`.

```rust
pub(super) fn render_banner(
    is_watch: bool, is_tty: bool,
    backend_only: bool, frontend_only: bool,
    backend_host: &str, backend_port: u16, vite_port: u16,
) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    if !frontend_only { writeln!(s, "Backend server on http://{backend_host}:{backend_port}")?; }
    if !backend_only  { writeln!(s, "Frontend server on http://127.0.0.1:{vite_port}")?; }
    if !frontend_only {
        writeln!(s);
        if is_tty { writeln!(s, "  r        rebuild backend + regenerate types"); }
        else      { writeln!(s, "  r        unavailable (non-TTY stdin)"); }
        writeln!(s, "  q        quit    (or Ctrl+C)");
        if is_watch { writeln!(s, "  watch    enabled  (debounce 500ms)"); }
        else        { writeln!(s, "  watch    disabled  (pass --watch to auto-reload on file changes)"); }
    }
    s
}

pub(super) fn classify_key(code: KeyCode, modifiers: KeyModifiers) -> Option<KbAction> {
    match (code, modifiers) {
        (KeyCode::Char('r'), KeyModifiers::NONE) => Some(KbAction::Reload),
        (KeyCode::Char('q'), KeyModifiers::NONE)
        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(KbAction::Quit),
        _ => None,
    }
}

pub(super) fn format_trigger_source(t: ReloadTrigger) -> &'static str {
    match t {
        ReloadTrigger::Manual      => "manual",
        ReloadTrigger::FileChanged => "file change",
    }
}

pub(super) fn should_spawn_keyboard(is_tty: bool) -> bool { is_tty }
```

Rustfmt reshaped `classify_key`'s or-pattern arm into a block (per line-width rule) — functionally identical.

### Exact banner literal emitted by `render_banner` (for 02b to confirm no drift)

Watch OFF, TTY:
```
Backend server on http://127.0.0.1:8080
Frontend server on http://127.0.0.1:5173

  r        rebuild backend + regenerate types
  q        quit    (or Ctrl+C)
  watch    disabled  (pass --watch to auto-reload on file changes)
```

Watch ON, TTY: identical except last line reads `  watch    enabled  (debounce 500ms)`.
Watch OFF, non-TTY: `r` line becomes `  r        unavailable (non-TTY stdin)`; watch line reads `disabled  (pass --watch...)`.
Watch ON, non-TTY: `  r        unavailable (non-TTY stdin)` + `  watch    enabled  (debounce 500ms)`.

Each banner ends with a trailing `\n` after the watch line. `backend_only = true` omits the `Frontend server` line; `frontend_only = true` omits the whole backend block (Backend line + key legend + watch line).

## Task Commits

Each task committed atomically:

1. **Task 1: --watch flag + dispatch threading** — `897c2355` (feat)
2. **Task 2: deletions + extraction + pure-helper bodies + 4 un-ignored tests** — `42eecb77` (feat)

## Test Inventory (after 02a)

Inline `serve::tests` (7 tests total):

| # | Test                                    | Status     | Plan to un-ignore |
|---|-----------------------------------------|------------|-------------------|
| 1 | `render_banner_matrix`                  | **passes** | 145-02a (done)    |
| 2 | `classify_key_table`                    | **passes** | 145-02a (done)    |
| 3 | `trigger_source_formatting`             | **passes** | 145-02a (done)    |
| 4 | `should_spawn_keyboard_gated_on_tty`    | **passes** | 145-02a (done)    |
| 5 | `kill_current_noop_when_none`           | `#[ignore]` — "implemented in 145-02b-PLAN — BackendSupervisor lives there" | 145-02b |
| 6 | `supervisor_coalesces_multiple_triggers`| `#[ignore]` — "implemented in 145-02b-PLAN — drain_triggers lives there"   | 145-02b |
| 7 | `debouncer_coalesces_burst`             | `#[ignore]` — "implemented in 145-02b-PLAN — spawn_file_watcher_at lives there" | 145-02b |

Integration `serve_supervisor` (4 tests, all still `#[ignore]` → 145-03).

## Pre-commit triad tail

```
$ cargo build -p ferro-cli
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.80s

$ cargo clippy -p ferro-cli --all-targets -- -D warnings
    (no output; exits 0)

$ cargo fmt --package ferro-cli -- --check
    (no output; exits 0)

$ cargo test -p ferro-cli --lib serve::tests --quiet
running 7 tests
iii....
test result: ok. 4 passed; 0 failed; 3 ignored; 0 measured; 473 filtered out; finished in 0.00s

$ cargo test -p ferro-cli --all-features --quiet
... test result: ok. 473 passed; 0 failed; 7 ignored (inline) + 4 ignored (integration) ...
```

Workspace-wide `cargo fmt --all -- --check` and `cargo clippy --all --all-targets -- -D warnings` remain gated by pre-existing unrelated issues in `ferro-json-ui` — logged in `deferred-items.md`.

## Decisions Made

- **Backend spawn switched to `cargo run`, not queued for 02b.** The plan's must-have "`cargo-watch` binary is no longer installed or invoked by `ferro serve`" requires this. Without the change, deleting `ensure_cargo_watch` would break `ferro serve` for any user without cargo-watch on PATH. Net simplification: 02b's supervisor will own this spawn anyway, and the short-term semantics (one-shot `cargo run`, exits on compile failure, no auto-respawn) match the spec's explicit non-goal on auto-respawn.
- **`#[allow(dead_code)]` kept on pure helpers and enums.** They're referenced only by tests in 02a; 02b wires them into production and can drop the allows at that point. Alternative (introduce dummy production refs) would be noise.
- **Docstring comment header rewritten.** Removed the "Bodies are `todo!()` here" note now that bodies are filled. Grep-acceptance for `todo!` now prints `0`.
- **Fmt/clippy gates scoped to `-p ferro-cli`.** Matches Plan 01's established posture and the phase-context guidance. Workspace-wide drift is logged for a separate future cleanup; not Phase 145's job.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Critical functionality] Changed backend spawn from `cargo watch -x run` to plain `cargo run`**
- **Found during:** Task 2 (deletion of `ensure_cargo_watch`).
- **Issue:** The plan's explicit must-have — "`cargo-watch` binary is no longer installed or invoked by `ferro serve`" — requires the backend spawn to stop using `cargo watch -x "run --bin …"`. Without the change, `ferro serve` would still attempt to invoke `cargo watch` at runtime despite the install step being removed, breaking any environment where cargo-watch isn't on PATH.
- **Fix:** `manager.spawn_with_prefix("cargo", &["run", "--bin", &package_name], …)` — a plain one-shot spawn. Matches the spec's behavior for a no-watch / compile-failure path (no auto-respawn). 02b's supervisor will take ownership of this spawn and add re-spawn-on-trigger semantics.
- **Files modified:** `ferro-cli/src/commands/serve.rs` (backend spawn block).
- **Verification:** `grep -cE "cargo-watch|cargo watch" ferro-cli/src/commands/serve.rs` prints `0`. `ferro serve` still spawns backend + frontend for the no-watch happy path.
- **Committed in:** `42eecb77` (Task 2 commit).

**2. [Rule 3 - Blocking] Scoped pre-commit fmt gate to `-p ferro-cli`**
- **Found during:** Task 1 (running `cargo fmt --all -- --check`).
- **Issue:** Workspace-wide fmt check fails on `ferro-json-ui/src/render.rs:2286` — a pre-existing long-line drift on master, unrelated to Phase 145.
- **Fix:** Gate scoped to `cargo fmt --package ferro-cli -- --check` (exits 0). Phase 145 scope stays inside `ferro-cli`. Issue logged to `deferred-items.md`.
- **Files modified:** `.planning/phases/145-ferro-serve-manual-reload-key-and-watch-supervisor/deferred-items.md`.
- **Verification:** `cargo fmt --package ferro-cli -- --check` exits 0.
- **Committed in:** `42eecb77` (Task 2 commit — alongside the main rewrite).

**3. [Rule 1 - Bug] Removed stale "Bodies are `todo!()`" note in module header**
- **Found during:** Task 2 post-edit grep.
- **Issue:** `grep -c "todo!"` was printing `1` because the module header comment still read "Bodies are `todo!()` here". Fixing this is required for the acceptance "`grep -cn 'todo!' ferro-cli/src/commands/serve.rs` prints `0` in the four pure helpers".
- **Fix:** Rephrased to `"Bodies are filled by 145-02a against the inline test oracle below"`.
- **Files modified:** `ferro-cli/src/commands/serve.rs`.
- **Verification:** `grep -c "todo!"` now prints `0`.
- **Committed in:** `42eecb77` (Task 2 commit).

---

**Total deviations:** 3 auto-fixed (1 critical functionality — cargo-watch invocation removal, 1 blocking — fmt gate scope, 1 bug — stale comment). All mechanical and aligned with the plan's stated must-haves; none changed plan semantics.

**Impact on plan:** None material. The cargo-watch invocation removal is required by the plan's own must-haves — the plan text was slightly imprecise on Task 2's deletion list but the invariants in `<must_haves>` are unambiguous.

## Issues Encountered

- **Rustfmt reshaped `classify_key`'s or-pattern arm.** The plan-suggested body used a two-line or-pattern that exceeds rustfmt's default line width. After `cargo fmt`, the arm became `(KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => { Some(KbAction::Quit) }` — functionally identical, and the test oracle is insensitive to this formatting.
- **Acceptance grep `struct BackendSupervisor` counted `1` as a false positive.** It matches the string "construct BackendSupervisor" inside an `#[ignore]`'d test comment ("02b body: construct BackendSupervisor::new(...)"). No actual struct definition exists in 02a; the plan's intent ("02b work is not done here") is satisfied.

## Deferred Issues

See `.planning/phases/145-ferro-serve-manual-reload-key-and-watch-supervisor/deferred-items.md`:
- Pre-existing `SwitchProps.compact` compile errors in ferro-json-ui (logged in Plan 01).
- Pre-existing rustfmt drift in `ferro-json-ui/src/render.rs:2286` (newly logged in Plan 02a).

## User Setup Required

None — CLI-only work, no new services, no secrets, no config.

## Notes for 02b

- `pub fn run(…, watch: bool)` at line ~300 binds `let _ = watch;` as a temporary sink. 02b should:
  1. Delete the `let _ = watch;` line.
  2. Construct `BackendSupervisor` with the backend cargo run args + shutdown flag.
  3. Open the reload-trigger mpsc channel; spawn the supervisor thread with `recv_timeout` + `try_recv` drain (drain logic is currently sketched in the `#[ignore]`'d `supervisor_coalesces_multiple_triggers` test comment).
  4. If `watch`, call `spawn_file_watcher_at(Path::new("src"), Duration::from_millis(500), reload_tx.clone())`. The `_at` parametrization is required by `debouncer_coalesces_burst`'s test harness (tempdir-based).
  5. If `should_spawn_keyboard(std::io::stdin().is_terminal())`, spawn keyboard thread with `RawModeGuard` Drop, `event::poll(Duration::from_millis(100))`, `event::read()`, `classify_key`.
  6. Replace the old `manager.spawn_with_prefix("cargo", ["run", "--bin", …], …)` with the supervisor's `spawn_backend()` which also calls `spawn_child_with_prefix` for stdout piping.
  7. Drop the `any_exited()`-driven shutdown branch (per D-12; matches `cargo-watch` behavior — no auto-respawn).
- Imports 02b needs to add (all intentionally NOT imported in 02a):
  - `crossterm::event::{self, Event, KeyEventKind}`
  - `crossterm::terminal::{enable_raw_mode, disable_raw_mode}`
  - `notify::RecursiveMode`
  - `notify_debouncer_mini::{new_debouncer, DebouncedEvent, DebounceEventResult}`
  - `std::io::IsTerminal`
  - `std::path::PathBuf`
  - `std::sync::mpsc::{channel, Sender, Receiver, RecvTimeoutError, TryRecvError}`
  - `std::time::Duration`
- Three tests waiting to un-ignore with existing skeleton comments pointing at their helper names: `kill_current_noop_when_none` (needs `BackendSupervisor::new`), `supervisor_coalesces_multiple_triggers` (needs `drain_triggers`), `debouncer_coalesces_burst` (needs `spawn_file_watcher_at`).

## Next Phase Readiness

- **145-02b ready to start.** `run()` skeleton has the `watch: bool` parameter in place; `spawn_child_with_prefix` is ready for `BackendSupervisor::spawn_backend()` reuse; pure helpers are bodies-in for keyboard thread and banner; test skeletons carry helper-name hints. No dependency bumps needed — `crossterm = "0.29"` and `notify-debouncer-mini = "0.4"` already in Cargo.toml.
- **145-03 ready.** Integration test scaffold exists with `CHDIR_LOCK`, `fixture_dir`, `ferro_bin`. 02b's supervisor behavior is the remaining prerequisite.

## Self-Check

Files verified to exist:
- `ferro-cli/src/main.rs` — contains `watch: bool,` in `Commands::Serve` at line 53, `Enable file-watch auto-reload` at line 51, and threads `watch` through the dispatch arm at lines 488-503.
- `ferro-cli/src/commands/serve.rs` (634 lines) — contains `fn spawn_child_with_prefix` at line ~107, `fn render_banner` at line ~38, `fn classify_key` at line ~78, `fn format_trigger_source` at line ~90, `fn should_spawn_keyboard` at line ~99, `let _ = watch;` at line ~305 inside `run()`, no `fn ensure_cargo_watch`, no `fn start_type_watcher`, no literal `cargo-watch`, no literal `cargo watch`.
- `.planning/phases/145-ferro-serve-manual-reload-key-and-watch-supervisor/deferred-items.md` — now documents both the `SwitchProps.compact` issue and the `render.rs:2286` fmt drift.

Commits verified:
- `897c2355` Task 1 — present in `git log --oneline`.
- `42eecb77` Task 2 — present in `git log --oneline`.

Test discovery verified:
- `cargo test -p ferro-cli --lib serve::tests -- --list` lists 7 tests, 4 un-ignored by name (render_banner_matrix, classify_key_table, trigger_source_formatting, should_spawn_keyboard_gated_on_tty) + 3 still ignored by name.
- `cargo test -p ferro-cli --lib serve::tests` → 4 passed, 0 failed, 3 ignored.
- `cargo test -p ferro-cli --all-features` → green; no regressions.
- `cargo build -p ferro-cli` → exits 0.
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` → exits 0.
- `cargo fmt --package ferro-cli -- --check` → exits 0.
- `cargo run -p ferro-cli -- serve --help | grep -- '--watch'` → prints the flag with help text.

## Self-Check: PASSED

---
*Phase: 145-ferro-serve-manual-reload-key-and-watch-supervisor*
*Completed: 2026-04-22*
