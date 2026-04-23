# ferro serve: manual reload key and unified watch supervisor

Status: Design approved 2026-04-22
Scope: `ferro-cli/src/commands/serve.rs` and its dependencies
Target workspace version: next minor after 0.2.5

## Problem

Running `ferro serve` with the types watcher alongside auto-reload (via `cargo-watch`) produces compounding rebuilds during rapid file changes. A burst of N writes triggers N recompiles in rapid succession, each of which is expensive and most of which are stale the moment they start. On thermally-constrained hardware (MacBook, fanless laptops), this behavior is actively harmful.

Two missing capabilities make the problem worse:

1. There is no way to trigger a reload on demand. The user is always at the mercy of the watcher.
2. There is no way to disable auto-reload and drive rebuilds manually. The only alternative today is to not use `ferro serve` at all, which also loses the frontend dev server, types generation, and port management.

## Goals

- Coalesce burst file-save events into exactly one rebuild per debounce window.
- Provide a runtime `r` key that triggers a rebuild immediately, cancelling any in-flight build.
- Make auto-reload opt-in via `--watch`. The default invocation serves without a file watcher.
- Unify backend recompile and types regeneration under a single reload path.

## Non-goals

- Hot reload without a process restart. Out of scope for a Rust framework.
- Changing Vite / frontend lifecycle. Frontend HMR is already good.
- Configurable debounce window. Fixed at 500 ms. Can be revisited if a real use case appears.
- Automatic respawn on compile failure. Failed builds wait for the next manual or file-triggered reload (matches current `cargo-watch` behavior).

## Decisions

- **Auto-watch is off by default.** `ferro serve` does not watch files; the user presses `r` to rebuild. `ferro serve --watch` opts into auto-reload.
- **`r` while a build is in flight cancels and restarts.** The mental model of a manual reload key is "I want my latest changes now"; queueing or ignoring would subvert that.
- **Scope of `r` and the debounced watcher: backend + types together.** One trigger rebuilds the backend and regenerates types. Frontend is not involved.
- **Replace `cargo-watch` with an in-process supervisor.** Ferro owns child-process lifecycle directly via `std::process::Child` and watches the filesystem via `notify-debouncer-mini`.

## Architecture

One supervisor thread owns the backend `cargo run` child. Two optional producer threads feed reload triggers into it through a shared channel:

```
Keyboard thread (crossterm raw mode)       File watcher thread (notify-debouncer-mini)
       │                                            │
       │ 'r' → Trigger::Manual                      │ debounced fs event →
       │                                            │ Trigger::FileChanged
       └──────────────────┬─────────────────────────┘
                          ▼
                reload_rx (mpsc channel)
                          │
                          ▼
            ┌──────────────────────────────┐
            │ BackendSupervisor (thread)   │
            │                              │
            │ on Trigger:                  │
            │   1. kill current child      │
            │   2. regenerate types        │
            │   3. spawn fresh cargo run   │
            │                              │
            │ on shutdown:                 │
            │   kill child, exit           │
            └──────────────────────────────┘
```

The Vite child remains managed by the existing `ProcessManager`. The supervisor owns only the backend child.

The keyboard thread is spawned only when stdin is a TTY. The file-watcher thread is spawned only when `--watch` is passed. Both are optional; the supervisor runs regardless.

## CLI surface

| Flag | Today | New |
|------|-------|-----|
| `--watch` | — | new; enables file-watch auto-reload |
| implicit auto-watch via cargo-watch | always on | removed |
| `--skip-types`, `--backend-only`, `--frontend-only`, `--port`, `--frontend-port` | unchanged | unchanged |

Startup banner (backend-enabled mode, TTY):

```
Backend server on http://127.0.0.1:8080
Frontend server on http://127.0.0.1:5173

  r        rebuild backend + regenerate types
  q        quit    (or Ctrl+C)
  watch    disabled  (pass --watch to auto-reload on file changes)
```

With `--watch`, the last line reads `watch    enabled  (debounce 500ms)`.

Non-TTY mode: keyboard thread is skipped, banner reads `r  unavailable (non-TTY stdin)`. Ctrl+C still works via signal handler.

## Runtime keys

| Key | Action |
|-----|--------|
| `r` | Trigger reload (`Trigger::Manual`). Works in watch and no-watch modes. |
| `q` or `Ctrl+C` | Graceful shutdown. |

## Reload supervisor

Core types:

```rust
enum ReloadTrigger {
    Manual,
    FileChanged,
}

struct BackendSupervisor {
    package_name: String,
    skip_types: bool,
    project_path: PathBuf,
    types_output_path: PathBuf,
    current: Option<Child>,
    shutdown: Arc<AtomicBool>,
}
```

Supervisor loop pseudocode:

```
spawn_backend()                        // initial cargo run
loop {
    select! {
        trigger = reload_rx.recv() => {
            log "[backend] reload triggered ({source})"
            kill_current()             // Child::kill() + wait(), best-effort
            regenerate_types()         // skipped if --skip-types; errors logged as warnings
            spawn_backend()            // fresh cargo run
        }
        _ = shutdown_rx.recv() => {
            kill_current()
            break
        }
    }
}
```

`spawn_backend()` reuses the stdout/stderr piping pattern already present in `ProcessManager::spawn_with_prefix`.

Exit detection: if the backend child exits on its own (compile error, panic, port bind failure) the supervisor logs the exit code and waits for the next reload trigger. No auto-respawn. Matches today's behavior.

## Debounced file watcher (`--watch` mode only)

Uses `notify-debouncer-mini`, a thin wrapper over `notify` that performs trailing-edge debouncing: events are collected for a window and one trigger fires after the burst settles. This replaces the current handmade leading-edge throttle (`last_regen.elapsed() > debounce_duration`) in `start_type_watcher`, which fires on the first event and ignores the rest of the burst — the opposite of what we want here.

Watch target: `src/` recursive. Filter to `*.rs` files. Emit `ReloadTrigger::FileChanged` to the shared reload channel.

## Keyboard thread

`crossterm` raw mode. Reads keys one at a time:

- `r` → send `ReloadTrigger::Manual`
- `q` → set shutdown flag
- Any other key → ignored

RAII guard ensures `disable_raw_mode()` runs on Drop so panics and Ctrl+C do not leave the terminal in raw mode.

TTY detection via `std::io::stdin().is_terminal()`. If stdin is not a TTY, the thread is not spawned and the banner reflects this.

## Error handling

| Scenario | Behavior |
|----------|----------|
| `Child::kill()` fails (already exited) | swallow; proceed to `wait()` then respawn |
| `cargo run` fails to spawn | log error, `current = None`, wait for next trigger |
| `cargo run` exits non-zero | log exit code, wait for next trigger |
| Reload triggered while no child is live | skip kill, go straight to regen + spawn |
| `enable_raw_mode()` fails | log warning, skip keyboard thread, serve continues |
| `notify` init fails | log warning, skip file watcher, `--watch` becomes effective no-op |
| `src/` missing | log warning, skip file watcher |

Raw-mode hygiene is enforced by a `Drop` guard in the keyboard thread.

Shutdown ordering:

1. Ctrl+C handler sets `shutdown = true`.
2. Main thread breaks its wait loop.
3. Supervisor's shutdown channel fires; supervisor kills its child and exits.
4. Keyboard thread's `Drop` guard runs; raw mode disabled.
5. Vite child killed via existing `ProcessManager::shutdown_all()`.
6. "Servers stopped." printed.

## Deletions

- `ensure_cargo_watch()` (serve.rs:148–171). No longer needed.
- `start_type_watcher()` (serve.rs:425–504). Folded into the supervisor.
- All references to installing `cargo-watch` in docs.

## Dependencies

- Add: `notify-debouncer-mini` (pairs with existing `notify` dep).
- Add: `crossterm` (lightweight, cross-platform raw-mode stdin).
- Remove: `cargo-watch` external binary assumption.

## Testing

### Unit tests (ferro-cli/src/commands/serve.rs)

- `BackendSupervisor::kill_current` is a no-op when `current = None`.
- Debouncer coalesces N burst events into 1 trigger within the window (using the debouncer's test harness).
- `render_banner(opts)` renders correct text for each `--watch` × TTY combination.
- Trigger source formatting: `Manual` vs `FileChanged`.

### Integration tests (ferro-cli/tests/)

- `ferro serve --backend-only` starts and shuts down cleanly on SIGINT within 2 s with no zombie children.
- `r` in no-watch mode triggers exactly one rebuild.
- `--watch` mode: a burst of 10 file writes within 100 ms produces exactly one rebuild after the debounce window.
- Non-TTY stdin: banner shows `r unavailable`, stdin bytes are ignored, no crash.
- Raw mode is restored on exit (verified via `stty` before/after; may be skipped in CI).

Fixture: minimal Ferro project under `ferro-cli/tests/fixtures/minimal-serve/` compiled once, rebuilds tested against its `target/`.

### Manual validation checklist

1. `ferro serve` — banner shows `watch disabled`; saving a `.rs` file does nothing; `r` triggers rebuild.
2. `ferro serve --watch` — banner shows `watch enabled`; 5 rapid saves → exactly one rebuild after ~500 ms.
3. `r` mid-compile → in-flight build killed, fresh build starts.
4. Ctrl+C during compile → backend and Vite exit within 2 s; terminal is not stuck in raw mode.
5. Introduce a compile error → rebuild fails, serve waits; fix and press `r` → backend returns.
6. `ferro serve --frontend-only` → no supervisor, no `r` prompt; Ctrl+C works as before.

## Docs to update

- `docs/src/` — replace the cargo-watch section with `--watch` + `r`-key model, including the key legend.
- `ferro serve --help` output (clap annotations).
