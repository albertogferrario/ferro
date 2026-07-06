# Phase 122.1 — Deploy scaffold UAT fixes

## Context
Gestiscilo and mkmenu UAT (2026-04-07) found four parser/heuristic bugs in the
Phase 122 scaffolders that block the "zero hand edits after regeneration"
promise. None are architectural — all localise to two files:
`ferro-cli/src/deploy/env_example.rs` and `ferro-cli/src/templates/do.rs`.

## Goal
Fix the four bugs discovered during Phase 122 UAT so `ferro docker:init` +
`ferro do:init` produce a Dockerfile and `.do/app.yaml` that deploy gestiscilo
and mkmenu without any hand edits.

## Scope

### 1. `.env.example` inline-comment stripping
`parse_env_example` currently captures trailing `# comment` text into the
value. Real-world `.env.example` files commonly annotate defaults:
```
APP_ENV=local          # local, staging, production, testing
APP_DEBUG=true         # Set false in production
```
Expected behavior: value is `local` / `true`, not `"local          # ..."`.

- Strip trailing `\s+#.*$` from unquoted values.
- Preserve `#` characters inside quoted values (`"foo#bar"`).
- Preserve literal `#` as first non-whitespace char after `=` when the line
  has no leading whitespace before the `#` (rare but valid).
- Add unit tests covering: comment after unquoted value, `#` inside quotes,
  `#` as only content after `=`, `#` with no leading space.

### 2. DO app name hyphenation
`do_init` currently uses the Cargo package name verbatim for `name:` in
`app.yaml`. mkmenu's package name is `mkmenu_ferro` which becomes
`name: mkmenu_ferro` — but DO App Platform convention is hyphens:
`mkmenu-ferro`.

- New helper `sanitize_do_app_name(pkg: &str) -> String` in
  `ferro-cli/src/templates/do.rs`: lowercase, replace `_` with `-`, strip any
  char that isn't `[a-z0-9-]`, collapse consecutive dashes.
- Wire into `AppYamlContext` construction in `do_init.rs`.
- Unit tests: `mkmenu_ferro` → `mkmenu-ferro`, `MyApp_v2` → `myapp-v2`,
  `foo__bar` → `foo-bar`, `app.name` → `appname`.

### 3. Dev-default placeholder substitution in envs block
Currently every `.env.example` entry becomes a literal `value:` in `app.yaml`.
That's wrong for dev defaults: `APP_URL=http://localhost:8080` gets shipped
to production. For secrets (classified via the existing secret heuristic),
we already reference `${KEY}`. Extend that to dev-default values.

Heuristic: replace literal value with `${KEY}` placeholder when any of:
- Value contains `localhost` or `127.0.0.1` or `0.0.0.0`.
- Value starts with `file:` or is a relative path like `./foo`, `data/foo`.
- Value is empty string or whitespace.
- Value matches a dev-only port literal (`3000`, `5173`, `8080`) AND key
  ends with `_PORT` → keep literal (ports are fine).

When substituted, also emit a comment above the entry:
`# Override in DO App Platform secrets (was: <original value>)`.

- Unit tests covering each heuristic branch.
- Do NOT touch `DATABASE_URL` handling — it's already special-cased via the
  databases block.

### 4. Worker filtering for test/dev binaries
`do_init` currently emits one `workers:` entry per `[[bin]]`. mkmenu has a
`test_parser` binary used for local parser debugging, not a worker. It
should not become a DO worker component.

Exclusion heuristic:
- Bin name starts with `test_`, `test-`, `dev_`, `dev-`, `debug_`, `debug-`.
- Bin has `required-features` in `Cargo.toml` `[[bin]]` entry (not typical
  for production workers).
- `src/bin/<name>.rs` path matches a test-like name.

Preserve the main server bin (first `[[bin]]` matching package name) as the
web service, all other non-excluded bins become workers.

- Unit tests with mock `[[bin]]` listings: `gestiscilo` + `screenshot-worker`
  → one service + one worker; `mkmenu_ferro` + `test_parser` → one service,
  zero workers.

## Verification
- Regenerate `Dockerfile` and `.do/app.yaml` for gestiscilo/app and mkmenu
  with `ferro docker:init --force` + `ferro do:init --force`.
- gestiscilo `.do/app.yaml`:
  - `name: gestiscilo` (no underscores).
  - `workers:` has exactly one entry: `screenshot-worker`.
  - `envs:` block uses `${APP_URL}` not `http://localhost:8080`.
  - No `#` characters embedded in `value:` fields.
- mkmenu `.do/app.yaml`:
  - `name: mkmenu-ferro` (hyphen, not underscore).
  - `workers:` block absent OR does not contain `test_parser`.
  - `envs:` block uses `${APP_URL}` not literal localhost.

## Out of scope
- New deploy features — only bug fixes for Phase 122 UAT findings.
- Changing the Dockerfile template (all 4 bugs are in do.rs + env_example.rs).
- Reference-app verification on machines without gestiscilo-it checked out.
