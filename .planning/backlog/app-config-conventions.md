# Application Config Conventions

Surfaced 2026-05-07 while raising the HTTP body limit in a downstream app (gestiscilo-it/mkmenu). The override worked, but the convention to do it cleanly is undocumented.

## Planning Note

This document is a sketch from a downstream-app perspective, not an inside-Ferro design. When this item is promoted from backlog to a phase, the Ferro planning agent must first open a discussion with the maintainer and revise the proposal to fit Ferro's vision and existing conventions before drafting `PLAN.md`. Specifically:

- Reconcile the desired pattern against `.planning/VISION.md` and Ferro's existing provider/registry primitives.
- The auto-registration mechanism (declarative macro vs proc-macro vs `build.rs` scanning) is a Ferro architectural decision and may diverge from what is sketched below.
- Treat the Scope items as starting points, not commitments — drop, merge, or reframe them as needed to match how Ferro already organizes scaffolding and DX.

## Context

Ferro already exposes the primitives for app-side configuration overrides:

- `Config::register::<T>(value)` writes a typed provider into the global registry.
- `Config::get::<T>()` reads it back. Framework code uses `unwrap_or_else(T::from_env)` as the fallback path (e.g. `Server::from_config`).
- Built-in providers (`ServerConfig`, `AppConfig`, `LangConfig`, `DatabaseConfig`) all expose `from_env()` and a `builder()`.

In practice, an app wires this manually:

```rust
// src/config/mod.rs
pub fn register_all() {
    Config::register(FerroDatabaseConfig::from_env());
    Config::register(MailConfig::from_env());
    Config::register(server::build()); // app-side override
}
```

```rust
// src/config/server.rs
use ferro::config::ServerConfig;

pub fn build() -> ServerConfig {
    ServerConfig::builder()
        .max_body_size(100 * 1024 * 1024)
        .build()
}
```

The pattern works, but each app reinvents the folder layout and the `register_all()` glue. There is no scaffold, no convention doc, and no auto-discovery.

## Desired Pattern

Mirror Laravel's `config/` folder: each file is a typed config provider, registered uniformly.

```
src/config/
├── mod.rs        # registers every sibling
├── server.rs     # pub fn build() -> ServerConfig
├── mail.rs       # pub fn build() -> MailConfig
├── database.rs   # pub fn build() -> DatabaseConfig
└── ...
```

A new app gets `src/config/` scaffolded with the common providers stubbed; adding a new override is one file plus one line in `mod.rs` (or zero if auto-discovery is implemented).

## Scope

1. **Scaffolder** — `ferro make:config <name>` generates `src/config/<name>.rs` with a `build()` stub for the named provider type.
2. **Auto-registration** — either a declarative macro (`ferro::config::register_all!()`) that expands to the per-file calls, or a `build.rs`/proc-macro that walks `src/config/*.rs` and emits them.
3. **Convention doc** — `docs/configuration.md` covering: where overrides live, when env-only is enough, when to register a typed provider, how to author a new framework-side provider.
4. **Optional polish** — typed accessor sugar (e.g. `config!(Server.max_body_size)`) to avoid `Config::get::<ServerConfig>().unwrap().max_body_size` at use-sites.

## Out of Scope

- Runtime config reload.
- Multi-environment merging (env vars already cover this).
- Encrypted secrets at rest.

## Effort

Medium. Item 1 is small (scaffolder template). Item 2 is the core work and needs proc-macro or build-script scanning. Items 3 and 4 are documentation and DX polish.

## Related

- `framework/src/config/providers/server.rs` — `ServerConfigBuilder` already exposes the override knobs the app needs.
- `framework/src/server.rs` — `Server::from_config` is the consumer; pattern generalizes to other framework subsystems that read their config the same way.
