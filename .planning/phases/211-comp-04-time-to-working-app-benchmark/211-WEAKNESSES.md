# Phase 211 — Discovered Weaknesses (COMP-04 SC#5)

**Run:** cold-cache Docker benchmark of the committed `Dockerfile`, `debian:bookworm-slim`.
**Toolchain:** rustc/cargo 1.96.0 (stable). **Published artifacts:** `ferro-cli` 0.2.55,
`ferro-rs` 0.2.55 (resolved from `ferro = { package = "ferro-rs", version = "0.2" }`).
**Host:** Apple M1 Pro, 8 cores, 16 GB. **Date:** 2026-06-13.
**Source:** `ferro-cli/tests/fixtures/benchmark/RESULTS.md` + the cold run output.

The five-step sequence is: `ferro new` → `make:auth` → `make:scaffold` ×3 → `make:job` →
`cargo build`. Steps 1–4 each complete in under a second; the result below concerns step 5.

## Finding 1 — The generated app does not compile against published `ferro-rs` 0.2.55 (dominant)

`cargo build` of the scaffolded project fails with **52 compile errors**. The published
`ferro-cli` 0.2.55 scaffold templates emit code that does not match the published `ferro`
crate surface. Observed errors (cold run, verbatim categories):

- `error[E0433]: cannot find error_response in ferro` — the generated API controllers call
  `ferro::error_response!(...)` for every CRUD handler; the macro is not exported by the
  published crate. This single missing export accounts for the majority of the 52 errors.
- `error: cannot find attribute rule in this scope` — the generated request structs use a
  `#[rule]` validation attribute that is not in scope (6 occurrences).
- `error[E0432]: unresolved imports ferro::Queue, ferro::QueueConfig`.
- `error[E0432]: unresolved import ferro_queue` — `make:job` emits `use ferro_queue::{...}`,
  but `ferro-queue` is **not** a dependency in the generated `Cargo.toml` (verified: the
  generated `[dependencies]` block has no `ferro-queue`/`ferro_queue` entry).
- `error[E0433]: cannot find type ActiveValue` — scaffold controllers write
  `ActiveValue::Set(...)` without importing `ActiveValue` (sea-orm).
- `error[E0432]: unresolved import crate::models::users` — `make:auth` output references a
  `users` model module that is not resolvable as generated.
- `error[E0423]: expected function, found module ferro::database::connection`.

**Implication:** "time to working app" is unbounded on the published toolchain — the cost is
not compile time but a scaffold↔library API drift that blocks compilation. Because the
generated `Cargo.toml` pins `ferro-rs` from crates.io (not a path dependency), scaffolding
with the local workspace binary reproduces the same failure: the published library, not the
scaffolding binary, is the constraint. This is the central first-time-experience defect and
the strongest argument for a published-artifact smoke test (scaffold → `cargo build`) in CI.

## Finding 2 — Cold CLI install needs `libssl-dev` + `pkg-config` (undocumented prerequisite)

On a clean `debian:bookworm-slim`, `cargo install ferro-cli` aborts:
`openssl-sys` (pulled transitively via `native-tls`) reports "Could not find directory of
OpenSSL installation". Installing the CLI requires `libssl-dev` and `pkg-config`, which are
not mentioned in any install instruction. The committed Dockerfile now installs them.
**Implication:** the documented "install the CLI" step silently fails on a minimal Linux base;
either document the system packages or move the CLI off `native-tls` (e.g. rustls) to remove
the OpenSSL build-time dependency.

## Finding 3 — `make:scaffold` flag ordering swallows flags as field names

`make:scaffold Name field:type --no-smart-defaults -q -y --api` fails with
`Error parsing fields: Invalid field name: '--no-smart-defaults'`. The `[FIELDS]...`
positional is greedy, so flags placed after the fields (the order most users would type) are
consumed as field names. Flags must precede the fields: `make:scaffold [OPTIONS] <NAME>
[FIELDS]...`. **Implication:** the natural argument order produces a confusing error rather
than working or printing usage; the parser should accept interspersed flags or fail with a
flag-specific hint.

## Finding 4 — Spec/implementation naming mismatch (`make:model` vs `make:scaffold`)

ROADMAP SC#2 and CONTEXT.md specify `ferro make:model <X>`, but no `make:model` subcommand
exists; entity generation is `ferro make:scaffold`. The benchmark satisfies SC#2's intent
(three entity types, each with migration + controller) using `make:scaffold`. **Implication:**
the planning vocabulary and the shipped CLI surface have drifted; align the docs/spec wording
with the actual subcommand name.
