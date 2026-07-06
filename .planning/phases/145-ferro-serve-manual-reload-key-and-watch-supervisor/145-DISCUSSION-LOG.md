# Phase 145: ferro serve manual reload key and watch supervisor - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-22
**Phase:** 145-ferro-serve-manual-reload-key-and-watch-supervisor
**Mode:** `--auto` (Claude selected recommended defaults without interactive prompting)
**Areas discussed:** Channel primitive, Trigger coalescing, Types regen interruption, Watch target breadth, Keyboard key surface, Output/logging on reload

---

## Channel primitive (producer → supervisor)

| Option | Description | Selected |
|--------|-------------|----------|
| `std::sync::mpsc` with `recv_timeout` | Already idiomatic in `serve.rs`. No new dep. Supervisor interleaves trigger recv with shutdown polling via timeout. | ✓ |
| `crossbeam-channel` with `select!` | Richer select semantics; matches spec pseudocode. Adds a dep. | |
| `tokio::sync::mpsc` | Async-native; but supervisor is explicitly a blocking thread. Forces unneeded runtime integration. | |

**Auto-selected:** `std::sync::mpsc` with `recv_timeout` — minimize dependencies, match existing patterns in `serve.rs` (imports at line 9).

---

## Trigger coalescing during a build

| Option | Description | Selected |
|--------|-------------|----------|
| Drain all pending triggers at cycle start | After the supervisor wakes on one trigger, `try_recv` in a loop to drain any extras before starting kill/regen/spawn. Avoids stacked reloads. | ✓ |
| Process one trigger at a time strictly FIFO | Each trigger produces one full cycle, even if several arrive during one build. | |
| Only count the last trigger | Explicitly discard intermediate triggers. | |

**Auto-selected:** Drain-all — the observable behavior is the same as "only count the last" (one kill-regen-spawn cycle follows the drain), and the implementation is cleaner.

---

## Types regen interruptibility

| Option | Description | Selected |
|--------|-------------|----------|
| Types regen runs to completion | Only the `cargo run` child is killable. A new trigger during regen is picked up at the next loop iteration. | ✓ |
| Types regen is also cancellable | Requires making the types step a spawned killable process. More code, more failure modes, regen is already fast. | |

**Auto-selected:** Regen uncancellable — keeps the supervisor state machine simple. Regen is fast enough that "cancel during regen" is a non-issue in practice.

---

## Watch target breadth (`--watch` mode)

| Option | Description | Selected |
|--------|-------------|----------|
| `src/` recursive, `*.rs` only | Matches spec. Smallest surface, lowest noise. | ✓ |
| `src/` + `Cargo.toml` | Cargo edits would auto-reload. Risks storms when cargo-edit tools rewrite the manifest. | |
| `src/` + migrations + config | Broadest coverage. Much higher event volume; high false-positive risk. | |

**Auto-selected:** `src/` recursive, `*.rs` only per spec. Revisit only on field signal.

---

## Keyboard key surface

| Option | Description | Selected |
|--------|-------------|----------|
| Lowercase `r` + `q` only | Minimal. Any other key is ignored. | ✓ |
| Lowercase and uppercase `r`/`R` | Tolerant of shift-held. | |
| Add `h` for help, `c` for clear screen, etc. | Richer TUI. Scope creep for v1. | |

**Auto-selected:** lowercase only — minimal surface per spec. Revisit if users ask.

---

## Output / logging on reload

| Option | Description | Selected |
|--------|-------------|----------|
| One log line per trigger, banner printed once at startup | `[backend] reload triggered (manual|file change)`. Clean. | ✓ |
| Re-render full banner on each reload | Always shows current key legend and watch status. Noisy. | |
| Silent reload (no log line) | Minimalist; but user loses feedback that `r` was received. | |

**Auto-selected:** one line per trigger, startup-only banner.

---

## Claude's Discretion

The following fell into Claude's discretion and are noted in CONTEXT.md `Claude's Discretion` section:

- Exact `crossterm` version pin (latest stable at implementation time).
- Internal `BackendSupervisor` field layout beyond what the spec skeleton names.
- Whether new supervisor/keyboard/watcher code stays inline in `serve.rs` or splits into submodules (prefer inline unless file grows past ~800 lines).
- Exact log-line phrasing (keep neutral, match existing `serve.rs` voice).
- Minimal-serve fixture contents for integration tests.

## Deferred Ideas

Captured in CONTEXT.md `<deferred>` section. Summary:

- Auto-respawn on compile failure
- Configurable debounce window
- Hot reload without process restart
- Watching `Cargo.toml`, migrations, non-Rust files
- Uppercase `R` / modifier-key bindings
- Banner re-render on each reload
- Env-var debounce override
