# Phase 227: Documentation Audit and Update for v0.2.61 - Research

**Researched:** 2026-06-15
**Domain:** mdBook documentation (`docs/src/`) factual-accuracy audit
**Confidence:** HIGH — all findings verified against live source files in this session

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Audit every page in `docs/src/`, not only the flagged candidates. `getting-started/installation.md` is already known-good — confirm, don't re-touch.
- **D-02:** Fix threshold is factual accuracy only. Correct wrong commands, stale config snippets, dead version pins, outdated flow descriptions. Do not rewrite prose, reorganize pages, or add new sections.
- **D-03:** Verify every command/code example against ground truth: CLI commands → `ferro-cli/src/commands/`; scaffold config claims → `ferro-cli/src/templates/files/backend/Cargo.toml.tpl`.
- **D-04:** TLS/OpenSSL sweep: confirm no `native-tls`/`runtime-tokio-native-tls`/OpenSSL in config or example snippets. Current state: docs are already clean; the only match is the correct "no OpenSSL" line on the install page. Treat as a verification checkpoint.
- **D-05:** Make brew the lead install method consistently. `docs/src/reference/cli.md` currently leads with `cargo install ferro-cli` (lines 7-8) — reorder so brew is primary and cargo/curl are alternates, with the toolchain-free-CLI vs Rust-needed-to-build-app distinction stated.
- **D-06:** Replace stale hard version pins with version-neutral phrasing or a `<pinned>` placeholder. Known instance: `docs/src/cli/frontend-types.md:97` (`--ferro-version 0.2.33`).
- **D-07:** Verify `make:auth` / `make:scaffold` / `make:job` generator docs and the `ferro serve` walkthrough against live CLI behavior. Fix any command name / flag / output drift.

### Claude's Discretion
- Exact per-page edits determined during execution from verification results.
- Ordering of the audit sweep (alphabetical, by-staleness-risk, getting-started-first) is Claude's call.

### Deferred Ideas (OUT OF SCOPE)
- CHANGELOG for 0.2.60/0.2.61 — no CHANGELOG system exists today; introducing one is infrastructure, not a factual-accuracy fix.
- READMEs, scaffold-generated README, tap README, install-script messaging — explicitly Phase 228.
</user_constraints>

---

## Summary

This phase audits every `.md` file under `docs/src/` for factual accuracy after the v0.2.59→0.2.61 changes (Homebrew install added; MSRV 1.88; rustls/toolchain-free; scaffold now ships `runtime-tokio-rustls`). The research confirms the CONTEXT.md scout findings: the TLS/OpenSSL migration is already reflected correctly in the docs — the only live discrepancies are narrower than the roadmap feared.

The concrete work is: (1) brew-first reorder on `reference/cli.md`; (2) version-pin cleanup on `cli/frontend-types.md`; (3) `db:sync` flag correction in `reference/cli.md` (docs say `--migrate`, CLI has `--skip-migrations`); (4) `ferro make:model` phantom command cleanup in two pages; (5) stale MCP binary name (`ferro-mcp`) in `upgrading/migration-guide.md`; (6) stale MCP tool count discrepancy (`57` vs `80+`) between two pages; (7) introduction.md milestone string is stale. All other pages verified clean.

**Primary recommendation:** Execute the audit as a single wave, page-by-page, starting with `reference/cli.md` (highest density of issues), then the two pages with phantom commands (`working-with-agents.md`, `upgrading/migration-guide.md`), then `cli/frontend-types.md`, then introduction.md, then sweep the remaining pages as clean confirmations.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Doc content verification | CLI source (`ferro-cli/src/`) | Scaffold template (`Cargo.toml.tpl`) | Commands exist iff registered in `main.rs`; template is ground truth for scaffold dependencies |
| mdBook build validation | `mdbook build docs/` | SUMMARY.md link integrity | No CI step runs mdbook; intra-doc links must be maintained manually |
| Install-flow accuracy | `getting-started/installation.md` (canonical) | `reference/cli.md` (reconcile to) | All install snippets on other pages must align to installation.md's ordering |

---

## Ground-Truth Oracle

### Scaffold dependency truth
[VERIFIED: `ferro-cli/src/templates/files/backend/Cargo.toml.tpl`]

```toml
sea-orm-migration = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls"] }
sea-orm            = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "macros"] }
```

Confirmed: `runtime-tokio-rustls` throughout. No `native-tls`, no `runtime-tokio-native-tls`, no OpenSSL.

### Real CLI subcommand registry
[VERIFIED: `ferro-cli/src/main.rs` clap `#[command(name = ...)]` annotations]

| Source File | Command Name | Notes |
|-------------|-------------|-------|
| `new.rs` | `ferro new` | — |
| `serve.rs` | `ferro serve` | flags: `--port`, `--frontend-port`, `--backend-only`, `--frontend-only`, `--skip-types`, `--watch` |
| `generate_types.rs` | `ferro generate-types` | — |
| `generate_routes.rs` | `ferro generate-routes` | — |
| `make_auth.rs` | `ferro make:auth` | flags: `--force`/`-f` |
| `make_controller.rs` | `ferro make:controller` | — |
| `make_action.rs` | `ferro make:action` | — |
| `make_api.rs` | `ferro make:api` | flags: `--all`, `--yes`/`-y`, `--exclude`, `--include-all` |
| `make_api_key.rs` | `ferro make:api-key` | flags: `--env` |
| `make_scaffold.rs` | `ferro make:scaffold` | flags: `--with-tests`, `--with-factory`, `--auto-routes`, `--yes`/`-y`, `--api`, `--no-smart-defaults`, `--quiet`/`-q` |
| `make_job.rs` | `ferro make:job` | — |
| `make_event.rs` | `ferro make:event` | — |
| `make_listener.rs` | `ferro make:listener` | flags: `--event`/`-e` |
| `make_middleware.rs` | `ferro make:middleware` | — |
| `make_migration.rs` | `ferro make:migration` | — |
| `make_notification.rs` | `ferro make:notification` | — |
| `make_inertia.rs` | `ferro make:inertia` | — |
| `make_json_view.rs` | `ferro make:json-view` | flags: `--description`/`-d`, `--no-ai`, `--layout`/`-l`, `--from-service-json` |
| `make_factory.rs` | `ferro make:factory` | — |
| `make_error.rs` | `ferro make:error` | — |
| `make_resource.rs` | `ferro make:resource` | flags: `--model`/`-m` |
| `make_task.rs` | `ferro make:task` | — |
| `make_seeder.rs` | `ferro make:seeder` | — |
| `make_stripe.rs` | `ferro make:stripe` | flags: `--connect` |
| `make_theme.rs` | `ferro make:theme` | — |
| `make_whatsapp.rs` | `ferro make:whatsapp` | — |
| `make_lang.rs` | `ferro make:lang` | — |
| `make_module.rs` | `ferro make:module` | flags: `--with-migration`, `--no-views`, `--force`/`-f` |
| `make_policy.rs` | `ferro make:policy` | flags: `--model`/`-m` |
| `make_projection.rs` | `ferro make:projection` | flags: `--from-model` |
| `ai_make.rs` | `ferro ai:make` | flags: `--dry-run` (feature-gated: `projections`) |
| `ai_explain.rs` | `ferro ai:explain` | flags: `--type`, `--dry-run` (feature-gated: `projections`) |
| `projection_check.rs` | `ferro projection:check` | flags: `--name` (feature-gated: `projections`) |
| `db_migrate.rs` | `ferro db:migrate` | — |
| `db_rollback.rs` | `ferro db:rollback` | flags: `--step` (default: 1) |
| `db_status.rs` | `ferro db:status` | — |
| `db_fresh.rs` | `ferro db:fresh` | — |
| `db_seed.rs` | `ferro db:seed` | flags: `--class` |
| `db_sync.rs` | `ferro db:sync` | flags: `--skip-migrations` (inverts default behavior), `--regenerate-models` |
| `db_query.rs` | `ferro db:query` | — |
| `docker_init.rs` | `ferro docker:init` | flags: `--force`, `--ferro-version`, `--dry-run` |
| `docker_compose.rs` | `ferro docker:compose` | flags: `--with-mailpit`, `--with-minio` |
| `do_init.rs` | `ferro do:init` | flags: `--force`, `--dry-run` |
| `deploy_init.rs` | `ferro deploy:init` | flags: `--yes`, `--dry-run` |
| `ci_init.rs` | `ferro ci:init` | flags: `--force` |
| `doctor.rs` | `ferro doctor` | flags: `--json`, `--deploy` |
| `schedule_run.rs` | `ferro schedule:run` | — |
| `schedule_work.rs` | `ferro schedule:work` | — |
| `schedule_list.rs` | `ferro schedule:list` | — |
| `storage_link.rs` | `ferro storage:link` | flags: `--relative` |
| `api_check.rs` | `ferro api:check` | flags: `--url`, `--api-key`, `--spec-path` |
| `validate_contracts.rs` | `ferro validate:contracts` | flags: `--filter`/`-f`, `--json` |
| `mcp.rs` | `ferro mcp` | flags: `--cwd` |
| `boost_install.rs` | `ferro boost:install` | flags: `--editor` |
| `claude_install.rs` | `ferro claude:install` | flags: `--force`/`-f`, `--list`/`-l` |
| `clean.rs` | `ferro clean` | flags: `--sweep` |
| `auth_link.rs` | `ferro auth:link` | — |
| `json_ui_migrate_v1.rs` | `ferro json-ui:migrate-v1` | flags: `--dry-run` |
| `json_ui_schema.rs` | `ferro json-ui:schema` | flags: `--output`/`-o`, `--pretty`, `--component` |

**DOES NOT EXIST (no source file):**
- `ferro make:model` — no `make_model.rs`, not registered in `main.rs`. Command doesn't exist.

### Known-good install page
[VERIFIED: `docs/src/getting-started/installation.md`]

Order: Homebrew (recommended, no Rust needed) → curl installer → cargo → source. Mentions MSRV Rust 1.88+, no OpenSSL needed, rustls. This is the consistency target.

### CHANGELOG status
[VERIFIED: file exists at `/ferro/CHANGELOG.md`]

`CHANGELOG.md` exists at the repo root (not under `docs/src/`). It is not linked from SUMMARY.md. The CONTEXT.md decision to defer CHANGELOG work stands — this is out of scope for Phase 227. No action needed on the file's existence.

### mdBook build mechanics
[VERIFIED: `docs/book.toml`, `.github/workflows/`]

- `mdbook` is installed at `~/.cargo/bin/mdbook`.
- No CI step invokes `mdbook build` — the workflows cover only cargo fmt/clippy/test and publish/release. The mdBook build is not a CI gate.
- `book.toml` sets `create-missing = false` — missing pages linked from SUMMARY.md would error.
- SUMMARY.md is the canonical TOC. Intra-doc links (e.g. `[do-init](do-init.md)`) must resolve. No new pages are needed for Phase 227 fixes.

---

## Full Page Inventory

[VERIFIED: `find docs/src -name '*.md'`]

67 pages total (including SUMMARY.md):

```
docs/src/
├── SUMMARY.md
├── introduction.md
├── agents/
│   └── checkpoint-projection.md
├── cli/
│   ├── ci-init.md
│   ├── do-init.md
│   ├── doctor.md
│   ├── frontend-types.md           ← DISCREPANCIES
│   └── routes-json-schema.md
├── database/
│   ├── atomic-updates.md
│   ├── audit-log.md
│   └── reservations.md
├── features/
│   ├── agent-operable-app.md
│   ├── ai.md                       ← PHANTOM COMMAND (ferro ai:make reference in prose)
│   ├── api-mcp.md
│   ├── api-resources.md
│   ├── api.md
│   ├── authentication.md
│   ├── broadcasting.md
│   ├── caching.md
│   ├── database.md
│   ├── deployments.md
│   ├── derive-macros.md
│   ├── events.md
│   ├── ferro-assets.md
│   ├── inertia.md
│   ├── json-ui.md
│   ├── live-read-models.md
│   ├── localization.md
│   ├── mcp-api-key-auth.md
│   ├── mcp-oauth.md
│   ├── multi-tenancy.md
│   ├── notifications.md
│   ├── projections.md
│   ├── queues.md
│   ├── rate-limiting.md
│   ├── static-files.md
│   ├── storage.md
│   ├── stripe.md
│   ├── testing.md
│   ├── themes.md
│   ├── validation.md
│   └── whatsapp.md
├── getting-started/
│   ├── directory-structure.md
│   ├── installation.md             ← KNOWN GOOD (verify, don't touch)
│   ├── quickstart.md
│   └── working-with-agents.md      ← PHANTOM COMMAND
├── introduction.md                  ← STALE MILESTONE STRING
├── json-ui/
│   ├── actions.md
│   ├── components.md
│   ├── data-binding.md
│   ├── expressions.md
│   ├── forms.md
│   ├── getting-started.md
│   ├── json-schema.md
│   ├── layouts.md
│   ├── plugins.md
│   ├── runtime-primitives.md
│   └── spec-construction.md
├── reference/
│   └── cli.md                      ← DISCREPANCIES (brew-first, db:sync flag)
├── the-basics/
│   ├── action-handlers.md
│   ├── controllers.md
│   ├── inline-budget-and-telemetry.md
│   ├── middleware.md
│   ├── request-response.md
│   └── routing.md
└── upgrading/
    └── migration-guide.md          ← STALE MCP BINARY NAME
```

---

## Concrete Discrepancy List

All items below were verified against source files in this session.

### DISC-01: `reference/cli.md` — Install section leads with `cargo install` [HIGH PRIORITY]
[VERIFIED: `docs/src/reference/cli.md` lines 7-8, D-05]

**File:Line:** `docs/src/reference/cli.md:7-17`

**Current text:**
```
## Installation

```bash
cargo install ferro-cli
```

Or build from source:

```bash
git clone https://github.com/albertogferrario/ferro
cd ferro/ferro-cli
cargo install --path .
```
```

**Fix:** Replace the Installation section with brew-first ordering matching `installation.md`: Homebrew (recommended, no Rust needed) → curl → cargo. Include the toolchain-free-CLI vs Rust-needed-to-build-app distinction.

---

### DISC-02: `reference/cli.md` — `db:sync` documents `--migrate` flag that doesn't exist
[VERIFIED: `ferro-cli/src/main.rs` DbSync registration; `ferro-cli/src/commands/db_sync.rs`]

**File:Line:** `docs/src/reference/cli.md:1024-1031`

**Current text:**
```bash
ferro db:sync --migrate
```
Table says: `| --migrate | Run pending migrations before syncing |`

**Reality:** The real flag is `--skip-migrations` (default behavior is to run migrations; the flag suppresses them). The docs invert the polarity.

**Fix:** Replace `--migrate` with `--skip-migrations` in the example and options table. Change description to: "Skip running migrations before syncing (migrations run by default)."

---

### DISC-03: `cli/frontend-types.md` — Stale `0.2.33` version pin
[VERIFIED: `docs/src/cli/frontend-types.md` lines 79 and 97, D-06]

**File:Line 1:** `docs/src/cli/frontend-types.md:79`

```dockerfile
RUN cargo install ferro-cli --version <pinned> --locked
```
This line is already using the placeholder `<pinned>` — it is CLEAN. No fix needed here.

**File:Line 2:** `docs/src/cli/frontend-types.md:97`

```bash
ferro docker:init --ferro-version 0.2.33 --force
```
This is a live stale version pin.

**Fix:** Replace `0.2.33` with a version-neutral placeholder, e.g. `<current-version>` or `0.2.xx`, and add a note that the value comes from `Cargo.lock` (the `ferro-rs` package version). The preceding prose already explains this: "The pinned `ferro-cli` version is read from your project's `Cargo.lock`."

**File:Line 3:** `docs/src/cli/frontend-types.md:115` — cross-link to `do-init.md` with label "ferro docker:init"

```
- [`ferro docker:init`](do-init.md) — regenerates the Dockerfile
```

`do-init.md` documents `ferro do:init` (DigitalOcean), NOT `ferro docker:init`. The link target is wrong — `do-init.md` is the DigitalOcean spec page; there is no separate `docker-init.md`. The label says `docker:init` but points to the DigitalOcean page.

**Fix:** Either (a) correct the label to `ferro do:init` if the intent was DigitalOcean, or (b) remove the link if no dedicated docker:init page exists. Given the context (Dockerfile regeneration), the linked command is genuinely `ferro docker:init`, not `ferro do:init` — the link target (`do-init.md`) is the error. Since no `docker-init.md` exists, the fix is to drop the broken cross-link or link to `reference/cli.md#ferro-dockerinit` instead.

---

### DISC-04: `getting-started/working-with-agents.md` — Phantom `ferro make:model` command
[VERIFIED: no `make_model.rs` in `ferro-cli/src/commands/`; not registered in `main.rs`]

**File:Lines:** `docs/src/getting-started/working-with-agents.md:105, 108, 112`

```
generation_hint: "Use `ferro make:model <ModelName>` to scaffold a new model with migration"
```
```
the CLI command: `ferro make:model`.
```
```bash
ferro make:model Post
```

**Reality:** `ferro make:model` does not exist. The correct command to scaffold a model + migration is `ferro make:scaffold <Name>` (generates model, migration, controller, and Inertia pages) or `ferro make:migration create_<name>_table` + `ferro db:sync`.

**Fix:** Replace `ferro make:model Post` with the correct command. Given the context (scaffolding "a new model with migration"), the closest real command is `ferro make:scaffold Post` (or split into `ferro make:migration` + `ferro db:sync` if the intent is model-only without controller). The example should also update the generated output path from `app/models/post.rs` to `src/models/post.rs` (real scaffold path).

---

### DISC-05: `upgrading/migration-guide.md` — Stale `ferro-mcp` binary name in MCP config
[VERIFIED: `docs/src/upgrading/migration-guide.md` lines 93-96; real command is `ferro mcp`]

**File:Lines:** `docs/src/upgrading/migration-guide.md:93-96`

```json
{
  "mcpServers": {
    "ferro-mcp": {
      "command": "ferro-mcp",
      "args": ["serve"]
    }
  }
}
```

**Reality:** The MCP server is not a separate `ferro-mcp` binary. It is invoked as `ferro mcp` (subcommand of the main `ferro` binary). The correct config is:
```json
{
  "mcpServers": {
    "ferro": {
      "command": "/path/to/target/debug/ferro",
      "args": ["mcp"]
    }
  }
}
```

The working-with-agents.md page already has the correct form. The migration guide is outdated.

**Fix:** Update the "After" block in the MCP config example to match the working-with-agents.md form. This is a high-visibility page (upgrade path) so accuracy matters.

---

### DISC-06: `introduction.md` — Stale milestone string
[VERIFIED: `docs/src/introduction.md` line 59; STATE.md shows current milestone is v15.0 / v11.0]

**File:Line:** `docs/src/introduction.md:59`

```
Current milestone work targets v12.0 spec-driven rendering.
```

The project is well past v12.0 (now at v15.0 shipped). This sentence is stale.

**Fix (D-02 boundary):** The factual correction is to remove or update the milestone reference. Since prose rewriting is out of scope, the minimal fix is to change the milestone to current or drop the milestone-specific sentence and leave "Ferro is pre-1.0. Breaking changes are allowed between minor versions until 1.0."

---

### DISC-07: MCP tool count discrepancy (57 vs 80+) between two pages
[VERIFIED: `ferro-mcp/src/tools/` has 64 files (minus `mod.rs` = 63 tool modules); `introduction.md` says "80+"; `working-with-agents.md` says "57"]

**File:Line A:** `docs/src/getting-started/working-with-agents.md:7`
```
exposes 57 introspection tools
```

**File:Line B:** `docs/src/introduction.md:13`
```
80+ tools via `ferro-mcp`
```

**Reality:** `ferro-mcp/src/tools/` has 64 `.rs` files including `mod.rs`, `relevance.rs` (internal), `ai_scaffold.rs` (internal) — so the user-visible tool count is somewhere in the 55-65 range. Neither number may be precisely correct, and both will rot as tools are added.

**Fix (minimal, D-02):** The safest fix is to remove the specific count entirely from both pages, or use version-neutral phrasing ("dozens of introspection tools" / "a full suite of introspection tools"). Do not introduce a new hard count that will rot again. This is a D-02 (factual accuracy) fix: two contradictory numbers on two pages is a fact error even if neither is precisely wrong by much.

---

### DISC-08: `features/ai.md` — `ferro ai:make` CLI reference in prose
[VERIFIED: `ferro ai:make` IS registered in `main.rs` under `#[cfg(feature = "projections")]`]

**File:Lines:** `docs/src/features/ai.md:352-353`

```
the same shape `ferro ai:make` produces.
Does NOT write files; use the `ferro ai:make` CLI command to write `src/projections/<name>.rs`.
```

`ferro ai:make` is a real command — it exists in `main.rs` as `AiMake` gated on the `projections` feature. The prose in `ai.md` is accurate. **No fix needed.** (False positive — confirming clean.)

---

## Verified Clean Pages (no fixes needed)

[VERIFIED via grep sweeps and spot-reads in this session]

The following pages were checked for TLS/OpenSSL references, hard version pins, install-method ordering, and phantom command names — all clean:

| Page | Checked For | Result |
|------|-------------|--------|
| `getting-started/installation.md` | Everything (known-good) | Clean — canonical reference |
| `getting-started/quickstart.md` | Commands, flow, TLS | Clean — all commands real, flow accurate |
| `features/authentication.md` | `make:auth` command, flags | Clean — `ferro make:auth` is real |
| `features/queues.md` | `make:job` command | Clean — `ferro make:job ProcessPayment` is real |
| `features/api.md` | `make:api`, `make:api-key` | Clean |
| `features/api-mcp.md` | `make:api`, `make:api-key` | Clean |
| `features/database.md` | `make:migration`, db: commands | Clean (db:sync --migrate not referenced here) |
| `features/events.md` | `make:event`, `make:listener` | Clean |
| `features/notifications.md` | `make:notification` | Clean |
| `features/localization.md` | `make:lang` | Clean |
| `features/inertia.md` | `ferro serve`, types | Clean |
| `features/stripe.md` | `make:stripe` | Clean |
| `features/whatsapp.md` | `make:whatsapp` | Clean |
| `features/themes.md` | `make:theme` | Clean |
| `features/json-ui.md` | `make:json-view` | Clean |
| `features/api-resources.md` | `make:resource` | Clean |
| `features/static-files.md` | `ferro serve` | Clean |
| `features/deployments.md` | sea-orm imports | Clean (no TLS features in prose) |
| `cli/do-init.md` | `ferro do:init` command | Clean |
| `cli/ci-init.md` | `ferro ci:init` | Clean |
| `cli/doctor.md` | `ferro doctor` | Clean |
| `cli/routes-json-schema.md` | `ferro generate-routes` | Clean |
| `json-ui/*.md` (all 10 pages) | Commands, TLS | Clean |
| `the-basics/*.md` (all 6 pages) | Commands, middleware | Clean |
| `database/*.md` (3 pages) | sea-orm, TLS | Clean |
| `agents/checkpoint-projection.md` | — | Clean |
| `features/projections.md` | — | Clean |
| `features/live-read-models.md` | — | Clean |
| `features/testing.md` | — | Clean |
| `features/broadcasting.md` | — | Clean |
| `features/caching.md` | — | Clean |
| `features/storage.md` | — | Clean |
| `features/rate-limiting.md` | — | Clean |
| `features/multi-tenancy.md` | — | Clean |
| `features/mcp-oauth.md` | — | Clean |
| `features/mcp-api-key-auth.md` | — | Clean |
| `features/agent-operable-app.md` | — | Clean |
| `features/derive-macros.md` | — | Clean |
| `features/validation.md` | — | Clean |
| `features/ferro-assets.md` | — | Clean |
| `getting-started/directory-structure.md` | — | Clean |
| `getting-started/working-with-agents.md` | MCP config, tool count | Phantom command + tool count (DISC-04, DISC-07) |

---

## TLS/OpenSSL Sweep Result

[VERIFIED: `grep -rn "native-tls|runtime-tokio-native-tls|openssl|OpenSSL" docs/src/`]

**Single match only:**
```
docs/src/getting-started/installation.md:8:
  - Rust 1.88+ (with Cargo) — to build the app (no OpenSSL needed; the scaffold uses rustls)
```

This is the correct, accurate statement. No stale TLS references anywhere in the docs.

**D-04 verdict: CLEAN.** This was a verification checkpoint as expected; no fix needed.

---

## Verification Mechanism for Executor

### Command existence check
For any CLI command in a doc page, verify against `ferro-cli/src/main.rs` (clap `#[command(name = ...)]` annotation) or `ferro-cli/src/commands/mod.rs` (module list). If no corresponding source module exists, the command is phantom.

### Flag accuracy check
Compare flags documented on a page against the `#[arg(long = ...)]` declarations in `ferro-cli/src/main.rs` for the matching `Commands::` variant.

### Scaffold dependency check
For any `[dependencies]` snippet in docs, compare against `ferro-cli/src/templates/files/backend/Cargo.toml.tpl` directly.

### mdBook link integrity
`mdbook build docs/` will error on broken intra-doc links (since `create-missing = false`). Run `mdbook build docs/` in the repo root to verify after edits. This is the only automated doc check available (no CI step currently runs it, but it can be run locally).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Verifying command existence | Manual reading | `grep -n 'command(name' ferro-cli/src/main.rs` |
| Verifying scaffold TLS features | Manual reading | `cat ferro-cli/src/templates/files/backend/Cargo.toml.tpl` |
| Checking intra-doc link validity | Manual scanning | `mdbook build docs/` (exits non-zero on broken links) |

---

## Common Pitfalls

### Pitfall 1: `db:sync --migrate` vs `--skip-migrations` polarity inversion
**What goes wrong:** The docs say `--migrate` (run migrations); the CLI has `--skip-migrations` (skip them). The default behavior already runs migrations; the flag suppresses. An executor correcting the docs must invert the description, not just rename the flag.

### Pitfall 2: `do-init.md` link target in `frontend-types.md`
**What goes wrong:** `do-init.md` is the DigitalOcean (`do:init`) page, not the Docker page. The link says `ferro docker:init` but points to `do-init.md`. No separate `docker-init.md` exists. Fixing the label only doesn't fix the fact that the link destination is wrong.

### Pitfall 3: `ferro make:model` — command does not exist
**What goes wrong:** Three spots in `working-with-agents.md` reference `ferro make:model`. A naive audit might assume this is a real command not yet documented in `reference/cli.md`. Verification shows it doesn't exist at all.

### Pitfall 4: Tool count claims will rot again
**What goes wrong:** Replacing the stale `57` with the current count from the source tree produces a number that will be wrong after the next tool is added. The fix should use version-neutral language, not a new hard count.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `ferro make:scaffold` is the closest real equivalent to the phantom `ferro make:model` | DISC-04 | Low — scaffold is documented and real; executor should confirm intended command before writing |
| A2 | The `features/ai.md` `ferro ai:make` references are accurate because the command is feature-gated on `projections` | DISC-08 / Clean | Low — verified in main.rs registration |

**All other claims in this document were verified against live source files in this session.**

---

## Open Questions

1. **`working-with-agents.md` phantom command: what's the correct replacement?**
   - What we know: `ferro make:model` doesn't exist; `ferro make:scaffold` generates model + migration + controller + pages.
   - What's unclear: Was the intended command `ferro make:scaffold` (full scaffold) or `ferro make:migration` + `ferro db:sync` (model-only path)?
   - Recommendation: Use `ferro make:scaffold Post` as the replacement since the example context is "scaffolding a new model" end-to-end. If model-only is intended, use `ferro make:migration create_posts_table` + `ferro db:sync`.

2. **`reference/cli.md` `db:sync` fix: update the quickstart.md reference too?**
   - What we know: `quickstart.md` uses `ferro db:sync` without any flags (no flag = run migrations, which is correct default behavior). It's clean.
   - What's unclear: The quickstart step 2 says just `ferro db:sync` with no flag — this is accurate. No fix needed there.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|---------|
| `mdbook` | Link integrity check | ✓ | installed at `~/.cargo/bin/mdbook` | Manual SUMMARY.md review |

---

## Sources

### Primary (HIGH confidence)
- `ferro-cli/src/main.rs` — authoritative clap command registry, verified in session
- `ferro-cli/src/commands/mod.rs` — authoritative module list, verified in session
- `ferro-cli/src/commands/db_sync.rs` — `--skip-migrations` flag verified in session
- `ferro-cli/src/templates/files/backend/Cargo.toml.tpl` — scaffold dependencies verified in session
- `docs/src/getting-started/installation.md` — known-good canonical install page, verified
- `docs/src/reference/cli.md` — audited, discrepancies found at lines 7-17 and 1024-1031
- `docs/src/cli/frontend-types.md` — audited, discrepancies at lines 97 and 115
- `docs/src/getting-started/working-with-agents.md` — audited, phantom command at lines 105-112
- `docs/src/upgrading/migration-guide.md` — audited, stale MCP config at lines 93-96
- `docs/src/introduction.md` — audited, stale milestone at line 59
- `ferro-mcp/src/tools/` directory listing — tool count baseline, verified in session

### Secondary (MEDIUM confidence)
- `docs/src/SUMMARY.md` — TOC structure and all page paths, verified in session

---

## Metadata

**Confidence breakdown:**
- Discrepancy list: HIGH — every item verified against live source, not assumed
- Verified-clean pages: HIGH — grep sweeps cover all staleness markers
- TLS/OpenSSL sweep: HIGH — single grep confirmed, nothing to fix
- `ferro make:model` non-existence: HIGH — no source file, not registered in main.rs

**Research date:** 2026-06-15
**Valid until:** This research is file-content dependent; valid until the next CLI source change. Re-verify `ferro-cli/src/main.rs` before Phase 228 if significant time passes.
