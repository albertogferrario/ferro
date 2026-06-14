<!-- DRAFT — paste target: albertogferrario/homebrew-ferro README. Do NOT commit from the ferro repo session (cross-repo boundary, Phase 228 D-05). -->

# homebrew-ferro

Homebrew tap for the [Ferro](https://github.com/albertogferrario/ferro) CLI.

## Install

```sh
brew install albertogferrario/ferro/ferro
```

No Rust toolchain required to install or use the CLI.

## What this installs

The `ferro` binary — the Ferro framework CLI. You can use it to scaffold new projects, run migrations, start the development server, and manage your application.

## After install

```sh
ferro new my-app
cd my-app
ferro serve
```

Run `ferro --help` for the full command list.

## Building a scaffolded app

The CLI itself is toolchain-free. To **build and run** a scaffolded Ferro application, you need:

- **Rust 1.88+** (with Cargo) — install via [rustup](https://rustup.rs). No OpenSSL required; the scaffold uses rustls.
- **Node.js 18+** — for the frontend dev server (Vite + Inertia).
- **SQLite** (default), PostgreSQL, or MySQL — SQLite requires no extra setup.

## How the tap works

The formula is updated automatically on each [ferro GitHub release](https://github.com/albertogferrario/ferro/releases) via a `workflow_run` CI job (token-free). Manual formula updates are not needed.

## Links

- Framework repo: <https://github.com/albertogferrario/ferro>
- Documentation: <https://docs.ferro-rs.dev/>
