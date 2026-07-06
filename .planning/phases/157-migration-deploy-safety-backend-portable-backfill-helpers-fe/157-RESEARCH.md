# Phase 157: Migration Deploy Safety — Research

**Researched:** 2026-05-14
**Domain:** Rust workspace crate scaffolding, SeaORM migration helpers, DigitalOcean App Platform spec, ferro-cli doctor checks
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** New `ferro-migration` crate — not inlined into `framework`. Public API is `ferro_migration::backfill_random_hex`. Add to Wave 1a of CI publish workflow.
- **D-02:** Backfill helper API: `backfill_random_hex(manager, "table", "col", 16).await?` — takes `&impl SchemaManager` + positional params. Initial set: `backfill_random_hex`, `backfill_random_uuid`, `backfill_current_timestamp`, and a general `backfill` closure escape hatch.
- **D-03:** Backend coverage: SQLite + Postgres only. MySQL is out of scope.
- **D-04:** PRE_DEPLOY job command: `{binary_name} db:migrate` where `{binary_name}` is the web bin detected by `detect_web_bin`.
- **D-05:** `migrate_gate` doctor check severity: **Error** (hard fail). Fails when: project has `migrations/` or registers a Migrator AND `.do/app.yaml` exists AND no job with `kind: PRE_DEPLOY` running a migrate command is present.
- **D-06:** Fix `framework/src/app.rs`'s `run_migrations_silent` to call `std::process::exit(1)` on failure. Also update `app/src/main.rs` and the `ferro-cli/src/templates/files/backend/main.rs.tpl` new-project template.

### Claude's Discretion

- Exact naming of `backfill_*` variants beyond the four listed.
- Whether `ferro-migration` re-exports from `framework` or is a standalone dep apps add to their migration crate.
- Internal implementation details of the `migrate_gate` check (YAML parser choice, etc.).

### Deferred Ideas (OUT OF SCOPE)

- Multi-region / blue-green migration coordination.
- Postgres extension management (e.g. auto-creating `pgcrypto`).
- Migration squashing / baseline snapshots.
- MySQL backend support for backfill helpers.
</user_constraints>

---

## Summary

Phase 157 closes three deploy-safety gaps discovered in a 2026-05-13 gestiscilo-it production breakage. The gaps are independent technically but related causally: (1) a SQLite-only backfill raw SQL was executed on a Postgres database, (2) the migration runner swallowed the error and let the server start with a stale schema, and (3) no PRE_DEPLOY gate in `.do/app.yaml` existed to block the deployment before traffic shifted.

The fix requires work in three areas: a new `ferro-migration` workspace crate that provides backend-portable backfill helpers using `SchemaManager::get_database_backend()` dispatch; a framework bug fix in `run_migrations_silent` to call `std::process::exit(1)` on failure (matching the existing pattern in `run_seeders` and `DB::init` failure paths); and two additions to `ferro-cli` — a `jobs_block` field in `AppYamlContext` that emits a PRE_DEPLOY migrate job by default, and a new `migrate_gate` doctor check under `CheckCategory::Deploy`.

All four codebase sites are well-understood from reading the actual source. The patterns are consistent and the integration points are minimal.

**Primary recommendation:** Implement in this order — (1) framework silent-failure fix (smallest, highest value), (2) `ferro-migration` crate, (3) `do:init` jobs block, (4) `migrate_gate` check. Each is independently shippable.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Backend-portable backfill helpers | Library crate (`ferro-migration`) | — | Consumed inside SeaORM `MigrationTrait::up` implementations; lives in the migration crate context |
| PRE_DEPLOY job scaffolding | CLI (`ferro-cli` / `do:init`) | Template (`app.yaml.tpl`) | `do:init` is already the entry point for `AppYamlContext` construction |
| `migrate_gate` doctor check | CLI (`ferro-cli` / `doctor/checks/`) | — | All doctor checks live in `ferro-cli`; check reads filesystem, not runtime |
| Runtime failure abort | Framework (`framework/src/app.rs`) | New-project template | `run_migrations_silent` is a framework method; template copies the same pattern |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sea-orm-migration` | 1.0 | Provides `SchemaManager`, `MigratorTrait`, `DbBackend` | Already the workspace standard; all migrations use it [VERIFIED: framework/Cargo.toml] |
| `thiserror` | 2 | Error enum derivation for `ferro-migration::Error` | Workspace-standard for error crates; used in `ferro-orm`, `ferro-audit` [VERIFIED: codebase grep] |
| `serde_yaml` or `serde_yaml`-free YAML scan | — | `migrate_gate` check reads `.do/app.yaml` | See Architecture Patterns — lightweight scan preferred |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `async-trait` | 0.1 | Required if implementing async traits in `ferro-migration` | Only if the `backfill` closure escape hatch takes an async closure |
| `tempfile` (dev) | 3 | TempDir for doctor check tests | Already a dev-dep in `ferro-cli` [VERIFIED: existing check tests] |

**Installation for `ferro-migration`:**
```bash
# Cargo.toml for the new crate
[dependencies]
sea-orm-migration = "1.0"
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```

---

## Architecture Patterns

### System Architecture Diagram

```
User writes migration
        │
        ▼
MigrationTrait::up(manager: &SchemaManager)
        │
        ├─── ferro_migration::backfill_random_hex(manager, table, col, len)
        │         │
        │         ├── manager.get_database_backend() == Sqlite
        │         │       └── UPDATE … = lower(hex(randomblob(N/2)))
        │         │
        │         └── manager.get_database_backend() == Postgres
        │                 └── UPDATE … = encode(gen_random_bytes(N/2), 'hex')
        │
        └─── Result<(), DbErr> propagates to MigratorTrait::up
                  │
                  ▼ (on failure)
         run_migrations_silent (framework)
                  │
              [old] eprintln + continue → server starts stale
              [new] eprintln + process::exit(1) → deploy gate fires


Deploy path:
   do:init ──► AppYamlContext { jobs_block: render_jobs_block(web_bin) }
                      │
                      ▼
               .do/app.yaml with:
               jobs:
                 - name: migrate
                   kind: PRE_DEPLOY
                   run_command: /usr/local/bin/{web_bin} db:migrate


Doctor path (pre-push / CI):
   ferro doctor --deploy
        │
        ├── DockerTemplateDriftCheck  (existing)
        ├── CopyDirsDockerignoreCollisionCheck  (existing)
        └── MigrateGateCheck  (NEW)
                  │
                  ├── No migrations/ dir AND no Migrator usage → Ok (skip)
                  ├── No .do/app.yaml → Ok (skip — not a DO deploy project)
                  └── .do/app.yaml present, has migrations, NO PRE_DEPLOY migrate job
                            └── Error: "no PRE_DEPLOY migrate job found in .do/app.yaml"
```

### Recommended Project Structure for `ferro-migration`

```
ferro-migration/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs           # pub use; module declarations; crate doc
    ├── error.rs         # Error enum (thiserror)
    └── backfill.rs      # backfill_random_hex, backfill_random_uuid,
                         # backfill_current_timestamp, backfill (closure)
```

### Pattern 1: `SchemaManager::get_database_backend()` dispatch

**What:** `SchemaManager` exposes `get_database_backend()` which returns `DbBackend` (an enum with `Sqlite`, `Postgres`, `MySql`). This is the standard SeaORM mechanism for writing backend-conditional migration SQL. [VERIFIED: ferro-mcp/src/tools/list_migrations.rs, ferro-orm/tests/concurrent_decrement.rs]

**When to use:** Any time a migration contains raw SQL that differs between SQLite and Postgres.

```rust
// Source: verified from ferro-mcp/src/tools/database_schema.rs + ferro-orm pattern
use sea_orm_migration::prelude::*;

pub async fn backfill_random_hex(
    manager: &SchemaManager<'_>,
    table: &str,
    column: &str,
    hex_len: u32,
) -> Result<(), DbErr> {
    let byte_len = hex_len / 2;
    let sql = match manager.get_database_backend() {
        DbBackend::Sqlite => format!(
            "UPDATE \"{table}\" SET \"{column}\" = lower(hex(randomblob({byte_len}))) \
             WHERE \"{column}\" IS NULL OR \"{column}\" = ''"
        ),
        DbBackend::Postgres => format!(
            "UPDATE \"{table}\" SET \"{column}\" = encode(gen_random_bytes({byte_len}), 'hex') \
             WHERE \"{column}\" IS NULL OR \"{column}\" = ''"
        ),
        DbBackend::MySql => {
            return Err(DbErr::Custom(
                "backfill_random_hex: MySQL not supported".into(),
            ))
        }
    };
    manager
        .get_connection()
        .execute(Statement::from_string(manager.get_database_backend(), sql))
        .await
        .map(|_| ())
}
```

**Note on `gen_random_bytes`:** This is a Postgres built-in available since Postgres 9.x as part of `pgcrypto`. However, in Postgres 13+ the `gen_random_bytes` function is available in the **core** distribution without `pgcrypto` via `pgcrypto`-equivalent built-ins... actually, `gen_random_bytes` specifically still requires `pgcrypto` in standard Postgres. DO App Platform's managed Postgres has `pgcrypto` enabled by default. Alternative without `pgcrypto`: use `md5(random()::text || clock_timestamp()::text)` (not cryptographically strong) or `encode(gen_random_bytes(N), 'hex')` with pgcrypto. [ASSUMED: pgcrypto availability on DO managed Postgres]

**Safer Postgres alternative (no extension required):** `substring(md5(random()::text), 1, {hex_len})` — cryptographically weak but universally available without extensions. For a token meant as a checkin code this may be acceptable. The planner should flag this for user decision.

### Pattern 2: DigitalOcean PRE_DEPLOY job block format

**What:** The DO App Platform app spec `jobs:` array accepts entries with `kind: PRE_DEPLOY`. The job shares the same Dockerfile as the service. The `run_command` is an absolute path inside the container. [VERIFIED: docs.digitalocean.com/products/app-platform/reference/app-spec/]

```yaml
# Source: docs.digitalocean.com/products/app-platform/reference/app-spec/
jobs:
  - name: migrate
    kind: PRE_DEPLOY
    dockerfile_path: Dockerfile
    source_dir: /
    github:
      repo: {{REPO}}
      branch: {{GITHUB_BRANCH}}
      deploy_on_push: true
    run_command: /usr/local/bin/{{WEB_BIN}} db:migrate
    instance_size_slug: apps-s-1vcpu-0.5gb
    instance_count: 1
```

**Rendering approach:** The `{{JOBS_BLOCK}}` token is already in `app.yaml.tpl` at line 20. The `render_app_yaml` function currently does **not** replace `{{JOBS_BLOCK}}`; it only replaces `{{WORKERS_BLOCK}}`. The `debug_assert!(!rendered.contains("{{"))` would fire in debug builds. [VERIFIED: ferro-cli/src/templates/do.rs] This means the `jobs_block` field and replacement must be added to `render_app_yaml` as part of this phase.

**What `render_jobs_block` should produce:**
- Always emits the migrate job (not optional, not commented out). The design decision (D-04) is that `do:init` scaffolds this by default.
- The `run_command` uses the absolute path pattern already established for workers: `/usr/local/bin/{web_bin} db:migrate`.
- The `github:` section must be present in the job (DO requires a source); it mirrors the web service's `{{REPO}}`/`{{GITHUB_BRANCH}}`.

### Pattern 3: Doctor check — `MigrateGateCheck`

**What:** Follows the `DoctorCheck` trait pattern exactly as `DockerTemplateDriftCheck`. Category is `CheckCategory::Deploy`. [VERIFIED: ferro-cli/src/doctor/checks/docker_template_drift.rs]

**Detection logic for "project has migrations":**
The existing `MigrationsCheck` uses `root.join("src/migrations").exists()`. For `migrate_gate`, the CONTEXT.md specifies: project has a `migrations/` directory **or** registers a `Migrator`. The filesystem check `root.join("migrations").is_dir() || root.join("src/migrations").is_dir()` covers both standard ferro project layouts. [VERIFIED: ferro-cli/src/project.rs line 281 uses `root.join("migrations").is_dir()`]

**Detection logic for "PRE_DEPLOY migrate job in app.yaml":**

The `migrate_gate` check must parse `.do/app.yaml` and verify a job entry with `kind: PRE_DEPLOY` whose `run_command` contains `db:migrate`. Parser choice (Claude's Discretion): a full YAML parser (`serde_yaml`) or a line-scan heuristic. The line-scan approach is simpler and avoids adding a dep to `ferro-cli`:

```rust
// Line-scan approach (no extra dep):
fn has_predeploy_migrate_job(yaml: &str) -> bool {
    let mut in_jobs = false;
    let mut saw_predeploy = false;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed == "jobs:" { in_jobs = true; }
        if in_jobs {
            if trimmed.contains("PRE_DEPLOY") { saw_predeploy = true; }
            if saw_predeploy && trimmed.contains("db:migrate") { return true; }
        }
    }
    false
}
```

A full YAML parse with `serde_yaml` is more robust but `ferro-cli` already does line-based parsing elsewhere (see `deploy/app_yaml_existing.rs`). Either is valid — recommend line-scan to avoid a new dep.

**Check flow:**
1. No `migrations/` dir (neither `migrations/` nor `src/migrations/`) → `Ok("no migrations directory — skipped")`
2. No `.do/app.yaml` → `Ok("skipped — not a DO deploy project")`
3. `.do/app.yaml` present, `has_predeploy_migrate_job` returns true → `Ok("PRE_DEPLOY migrate job present")`
4. `.do/app.yaml` present, no PRE_DEPLOY migrate job → `Error("no PRE_DEPLOY migrate job in .do/app.yaml").with_details("Run \`ferro do:init --force\` to scaffold a migrate job, then commit.")`

### Pattern 4: `run_migrations_silent` fix

**What:** Replace `eprintln!("Warning:…")` + continue with `eprintln!` + `std::process::exit(1)`. [VERIFIED: framework/src/app.rs:386-391]

**Existing precedent in the codebase:**
```rust
// framework/src/app.rs:329-331 (run_seeders — already uses exit(1))
if let Err(e) = result {
    eprintln!("Seeding failed: {e}");
    std::process::exit(1);
}

// app/src/bootstrap.rs:44-57 (DB::init failure — already uses exit(1))
DB::init().await.unwrap_or_else(|e| {
    eprintln!("Error: Failed to connect to database");
    std::process::exit(1);
});
```

**Three files to update:**
1. `framework/src/app.rs:386-391` — `run_migrations_silent` method (the framework's `App` struct method)
2. `app/src/main.rs:140-144` — the sample app's free-fn copy of `run_migrations_silent`
3. `ferro-cli/src/templates/files/backend/main.rs.tpl:140-144` — the new-project template's copy

All three contain the identical pattern. The template fix prevents future projects from being scaffolded with the silent-failure bug.

### Anti-Patterns to Avoid

- **Using `lower(hex(randomblob(N)))` without backend dispatch:** The exact SQL that caused the production failure. Never call `Statement::from_string(DbBackend::Sqlite, ...)` in a migration that will run on multiple backends.
- **Adding `serde_yaml` to `ferro-migration`:** The backfill crate is a migration helper; it has no business parsing YAML. YAML parsing (if needed) belongs only in `ferro-cli`.
- **Adding `migrate_gate` as `CheckCategory::General`:** The gate is deploy-specific. Using `General` would run it in every `ferro doctor` invocation including local dev, producing false positives for projects that don't use DO.
- **Making `migrate_gate` a `Warn`:** The CONTEXT.md decision is `Error`. A `Warn` allows silent bypass by the CI exit-code check.
- **Rendering `{{JOBS_BLOCK}}` as an empty string when no migrations exist:** This leaves a blank line in the YAML. Render a commented-out example block (following the `render_workers_block` empty-case pattern).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Backend detection in migration SQL | Custom `env::var("DATABASE_URL")` parsing | `manager.get_database_backend()` | `SchemaManager` already knows the backend; string parsing of DATABASE_URL is fragile |
| YAML validation for `migrate_gate` | Full structural YAML parse | Line-scan for `PRE_DEPLOY` + `db:migrate` | Avoids a new dep; the check only needs to detect presence, not validate schema |
| Random byte generation | `rand` crate | Native DB functions (`randomblob`, `gen_random_bytes`) | Stays in the database transaction; no application-layer randomness needed |

**Key insight:** SeaORM's `SchemaManager` API was explicitly designed for backend-conditional migration SQL. Using anything else (env var inspection, runtime config) creates a coupling that `SchemaManager` is purpose-built to eliminate.

---

## Common Pitfalls

### Pitfall 1: `gen_random_bytes` requiring pgcrypto extension

**What goes wrong:** `UPDATE … SET col = encode(gen_random_bytes(8), 'hex')` fails on a Postgres instance without the `pgcrypto` extension enabled.
**Why it happens:** `gen_random_bytes` is not a core Postgres built-in; it lives in `pgcrypto`.
**How to avoid:** Either (a) assume DO managed Postgres has `pgcrypto` (it does by default, but not guaranteed for self-hosted), or (b) use `md5(random()::text)` truncated to `hex_len` characters — universally available without extensions. The `backfill_random_hex` implementation should use option (b) unless the phase explicitly decides to depend on `pgcrypto`. [ASSUMED: pgcrypto behavior on non-DO Postgres]
**Warning signs:** `ERROR: function gen_random_bytes(integer) does not exist` in migration logs.

### Pitfall 2: `{{JOBS_BLOCK}}` currently unresolved in rendered YAML

**What goes wrong:** The `render_app_yaml` function today replaces `{{WORKERS_BLOCK}}` and `{{ENVS_BLOCK}}` but **not** `{{JOBS_BLOCK}}`. The `debug_assert!(!rendered.contains("{{"))` fires in debug builds, but release builds silently emit a literal `{{JOBS_BLOCK}}` string into the YAML file.
**Why it happens:** The token was added to the template in anticipation of this phase but the renderer wasn't updated.
**How to avoid:** Add `jobs_block` to `AppYamlContext`, add `.replace("{{JOBS_BLOCK}}", &jobs_block)` in `render_app_yaml`, and add `render_jobs_block` function. [VERIFIED: ferro-cli/src/templates/do.rs — no `JOBS_BLOCK` replace present]

### Pitfall 3: `migrate_gate` false-positives for projects without DO deploy

**What goes wrong:** A project using SeaORM migrations but not deploying to DO App Platform gets an `Error` from `migrate_gate` because it has `migrations/` but no `.do/app.yaml`.
**Why it happens:** Check logic doesn't short-circuit on absent `.do/app.yaml`.
**How to avoid:** Check order is `migrations/ present?` then `app.yaml present?` then `PRE_DEPLOY job present?`. The second gate (`app.yaml` absent → skip) prevents this.

### Pitfall 4: Registry test count mismatch

**What goes wrong:** Adding a 12th check but the existing test `default_checks_returns_eleven_in_declared_order` still asserts `len() == 11`.
**Why it happens:** The test is a named constant, not derived from `default_checks().len()`.
**How to avoid:** Update the registry test: change `11` to `12`, add `"migrate_gate"` to the `names` vector in the correct position, and add `"migrate_gate"` to the `deploy_category_filter_returns_two` → `returns_three` test. [VERIFIED: ferro-cli/src/doctor/registry.rs:38]

### Pitfall 5: Template `main.rs.tpl` uses `{}` not `{e}` formatting

**What goes wrong:** The template at line 143 uses `eprintln!("Warning: Migration failed: {}", e)` (old-style). After editing, new format should be `{e}` (Rust 2021 edition standard in the workspace).
**Why it happens:** Template predates the workspace standardization.
**How to avoid:** Use `eprintln!("Migration failed: {e}");` + `std::process::exit(1);` in all three files being updated.

### Pitfall 6: `backfill` closure escape hatch async boundary

**What goes wrong:** If the `backfill` closure escape hatch takes an `async` closure, the trait object needs `async-trait` or the signature becomes complex.
**Why it happens:** Rust async closures are not stable as trait objects without `async-trait`.
**How to avoid:** For the closure-based `backfill`, accept a `Pin<Box<dyn Future<Output = Result<(), DbErr>> + Send>>` producer, or use a concrete `async fn` pointer. Simplest: make `backfill` accept `F: Future<Output = Result<(), DbErr>>` (caller wraps in `async move { ... }`).

---

## Code Examples

### `backfill_random_uuid` (Postgres: `gen_random_uuid()`, SQLite: manual hex)

```rust
// Pattern derived from: ferro-mcp/src/tools/database_schema.rs get_database_backend dispatch
pub async fn backfill_random_uuid(
    manager: &SchemaManager<'_>,
    table: &str,
    column: &str,
) -> Result<(), DbErr> {
    let sql = match manager.get_database_backend() {
        DbBackend::Sqlite => {
            // SQLite has no UUID function; produce a UUID-shaped hex string
            format!(
                "UPDATE \"{table}\" SET \"{column}\" = \
                 lower(hex(randomblob(4))) || '-' || \
                 lower(hex(randomblob(2))) || '-4' || \
                 substr(lower(hex(randomblob(2))), 2) || '-' || \
                 substr('89ab', abs(random()) % 4 + 1, 1) || \
                 substr(lower(hex(randomblob(2))), 2) || '-' || \
                 lower(hex(randomblob(6))) \
                 WHERE \"{column}\" IS NULL OR \"{column}\" = ''"
            )
        }
        DbBackend::Postgres => format!(
            "UPDATE \"{table}\" SET \"{column}\" = gen_random_uuid()::text \
             WHERE \"{column}\" IS NULL OR \"{column}\" = ''"
        ),
        DbBackend::MySql => {
            return Err(DbErr::Custom("backfill_random_uuid: MySQL not supported".into()))
        }
    };
    manager
        .get_connection()
        .execute(Statement::from_string(manager.get_database_backend(), sql))
        .await
        .map(|_| ())
}
```

### `render_jobs_block` for `do:init`

```rust
// Pattern: follows render_workers_block from ferro-cli/src/templates/do.rs
fn render_jobs_block(web_bin: &str, repo: &str, branch: &str) -> String {
    format!(
        "jobs:\n\
           - name: migrate\n\
             kind: PRE_DEPLOY\n\
             dockerfile_path: Dockerfile\n\
             source_dir: /\n\
             github:\n\
               repo: {repo}\n\
               branch: {branch}\n\
               deploy_on_push: true\n\
             run_command: /usr/local/bin/{web_bin} db:migrate\n\
             instance_size_slug: apps-s-1vcpu-0.5gb\n\
             instance_count: 1\n"
    )
}
```

Note: `render_jobs_block` needs `repo` and `branch` because the DO spec requires a source on the job. Since these are already resolved in `AppYamlContext` (`preserved_github_repo`/`preserved_github_branch`), they should be passed through or the renderer should use the already-resolved values.

**Simplest approach:** Add `jobs_block: String` to `AppYamlContext`. Caller in `do_init.rs` calls `render_jobs_block(&web_bin, &repo, &branch)` and sets the field. The renderer does `.replace("{{JOBS_BLOCK}}", &ctx.jobs_block)`.

### `run_migrations_silent` fix (three-file pattern)

```rust
// framework/src/app.rs — replace existing eprintln + continue
async fn run_migrations_silent<Migrator: MigratorTrait>() {
    let db = Self::get_database_connection().await;
    if let Err(e) = Migrator::up(&db, None).await {
        eprintln!("Migration failed: {e}");
        std::process::exit(1);
    }
}
```

---

## Integration Points

### New `ferro-migration` crate

1. Add `ferro-migration` to `[workspace]` members in root `Cargo.toml`
2. Add to `WAVE1A_CRATES` in `.github/workflows/publish.yml` (Wave 1a: leaf crates with zero internal deps) [VERIFIED: publish.yml line 201]
3. Add to `ferro-cli/src/doctor/checks/mod.rs` export list (if migrating the check needs it — no, it doesn't)
4. Document in `CLAUDE.md` workspace table (the crate description pattern)

### `doctor/checks/migrate_gate.rs`

1. New file at `ferro-cli/src/doctor/checks/migrate_gate.rs`
2. Add `pub mod migrate_gate;` to `ferro-cli/src/doctor/checks/mod.rs`
3. Add `pub use migrate_gate::MigrateGateCheck;` to the pub re-exports in `mod.rs`
4. Add `Box::new(MigrateGateCheck)` to `default_checks()` in `registry.rs`
5. Update the count assertion in `registry.rs` tests from `11` to `12`
6. Update `deploy_category_filter_returns_two` → `returns_three` test

### `AppYamlContext` / `render_app_yaml`

1. Add `pub jobs_block: String` field to `AppYamlContext`
2. Add `render_jobs_block` function (private) to `do.rs`
3. Add `.replace("{{JOBS_BLOCK}}", &ctx.jobs_block)` in `render_app_yaml`
4. Update `do_init.rs` to construct `jobs_block` via `render_jobs_block`
5. Update all `AppYamlContext { … }` construction sites in tests (add `jobs_block` field)

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `run_migrations_silent` continues on error | `process::exit(1)` on migration failure | Phase 157 | Server no longer starts with stale schema |
| No PRE_DEPLOY gate in scaffolded app.yaml | `do:init` always emits PRE_DEPLOY migrate job | Phase 157 | New apps get deploy safety by default |
| Raw backend-specific SQL in migrations | `ferro_migration::backfill_*` portable helpers | Phase 157 | Backend mismatch caught at test time, not deploy time |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | DO managed Postgres has `pgcrypto` enabled by default (enabling `gen_random_bytes`) | Code Examples | `backfill_random_hex` Postgres branch fails at migration time; must use `md5(random()::text)` instead |
| A2 | `gen_random_uuid()` is available in Postgres without pgcrypto (it is in Postgres 13+ core) | Code Examples | `backfill_random_uuid` Postgres branch fails; must use `encode(gen_random_bytes(16), 'hex')` with pgcrypto or UUID extension |

**Note on A2:** `gen_random_uuid()` was added to Postgres core in version 13 (released 2020-09-24). DO App Platform uses Postgres 14+ for managed databases. This claim is HIGH confidence. [ASSUMED based on training knowledge; not verified via DO docs in this session]

---

## Open Questions (RESOLVED)

1. **`gen_random_bytes` vs `md5(random())` for Postgres backfill_random_hex**
   - What we know: `gen_random_bytes` requires `pgcrypto`; DO managed Postgres has it enabled; self-hosted Postgres may not.
   - What's unclear: Whether `ferro-migration` should depend on `pgcrypto` availability or use the weaker `md5(random())` approach.
   - Recommendation: Use `encode(gen_random_bytes(N), 'hex')` for now (DO App Platform is the target); document the `pgcrypto` requirement in the crate's README.
   - RESOLVED: Use `encode(gen_random_bytes(N), 'hex')` with pgcrypto; document requirement in README.

2. **`jobs_block` as a pre-rendered String vs computed inside `render_app_yaml`**
   - What we know: `workers_block` is computed inside `render_app_yaml` from `ctx.workers: Vec<String>`. `jobs_block` is similar.
   - What's unclear: Whether to mirror the workers pattern (compute internally) or pre-compute at the caller.
   - Recommendation: Compute internally in `render_app_yaml` using `ctx.web_bin`, `ctx.repo`/`ctx.preserved_github_repo`, and `ctx.preserved_github_branch`. This keeps the context struct clean and parallels the workers pattern.
   - RESOLVED: Compute internally inside `render_app_yaml` — no new `AppYamlContext` field needed.

3. **`migrate_gate` detection of "registers a Migrator"**
   - What we know: The CONTEXT.md condition includes "registers a Migrator" in addition to `migrations/` dir presence.
   - What's unclear: How to detect Migrator registration without shelling out to `cargo`.
   - Recommendation: Use filesystem presence (`migrations/` or `src/migrations/`) as the sole signal. Registering a Migrator always requires a migrations directory. This is the same approach used by `MigrationsCheck`.
   - RESOLVED: Filesystem presence check only (`migrations/` or `src/migrations/`).

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code-only changes in Rust source files. No external tools, services, or CLIs beyond `cargo` (already required for all ferro development).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `tokio::test` |
| Config file | None (cargo test) |
| Quick run command | `cargo test -p ferro-cli -p ferro-migration --all-features` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | `ferro-migration` crate compiles and exports `backfill_random_hex` | unit | `cargo test -p ferro-migration` | ❌ Wave 0 |
| D-02 | `backfill_random_hex` dispatches correct SQL per backend | unit | `cargo test -p ferro-migration::backfill::tests` | ❌ Wave 0 |
| D-03 | `backfill_*` returns `DbErr::Custom` for MySql backend | unit | `cargo test -p ferro-migration` | ❌ Wave 0 |
| D-04 | `render_app_yaml` emits PRE_DEPLOY migrate job with correct `run_command` | unit | `cargo test -p ferro-cli templates::do_::tests` | ❌ (existing test suite, new test needed) |
| D-05 | `migrate_gate` returns `Error` when `.do/app.yaml` has no PRE_DEPLOY job | unit | `cargo test -p ferro-cli doctor::checks::migrate_gate` | ❌ Wave 0 |
| D-05 | `migrate_gate` returns `Ok` when no `.do/app.yaml` exists | unit | `cargo test -p ferro-cli doctor::checks::migrate_gate` | ❌ Wave 0 |
| D-05 | `migrate_gate` is `CheckCategory::Deploy` | unit | `cargo test -p ferro-cli doctor::registry` | ❌ Wave 0 |
| D-06 | `run_migrations_silent` calls `process::exit(1)` on failure | unit (process spawn) | manual / inspect | ❌ Wave 0 |

### Wave 0 Gaps

- [ ] `ferro-migration/src/lib.rs` — new crate skeleton
- [ ] `ferro-migration/src/backfill.rs` — backfill functions
- [ ] `ferro-cli/src/doctor/checks/migrate_gate.rs` — new check + unit tests
- [ ] New test in `ferro-cli/src/templates/do.rs` — `render_app_yaml_emits_predeploy_migrate_job`
- [ ] Registry count update in `ferro-cli/src/doctor/registry.rs`

---

## Security Domain

The backfill helpers generate random values for database tokens. Relevant considerations:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V6 Cryptography | Yes — `backfill_random_hex` generates tokens | Use `randomblob` (SQLite CSPRNG) and `gen_random_bytes` (Postgres CSPRNG) — both are cryptographically secure |
| V5 Input Validation | No | Table/column names are developer-controlled strings in migration code, not user input |

**Note:** `md5(random()::text)` (the extension-free Postgres fallback) is NOT cryptographically secure. If `backfill_random_hex` is used for security-sensitive tokens, the `gen_random_bytes` path must be used. The crate documentation should clearly state this distinction.

---

## Sources

### Primary (HIGH confidence)

- `ferro-cli/src/templates/do.rs` — `AppYamlContext`, `render_app_yaml`, `render_workers_block` patterns [VERIFIED: read in session]
- `ferro-cli/src/templates/files/do/app.yaml.tpl` — template token positions [VERIFIED: read in session]
- `ferro-cli/src/doctor/checks/generated_artifacts.rs` — canonical `DoctorCheck` pattern [VERIFIED: read in session]
- `ferro-cli/src/doctor/checks/docker_template_drift.rs` — `CheckCategory::Deploy` pattern [VERIFIED: read in session]
- `ferro-cli/src/doctor/registry.rs` — current 11 checks, wave structure [VERIFIED: read in session]
- `ferro-cli/src/doctor/check.rs` — `DoctorCheck` trait, `CheckResult`, `CheckCategory` types [VERIFIED: read in session]
- `framework/src/app.rs:386-391` — `run_migrations_silent` bug [VERIFIED: read in session]
- `app/src/main.rs:112-115` — sample app usage [VERIFIED: read in session]
- `ferro-cli/src/templates/files/backend/main.rs.tpl:140-144` — template usage [VERIFIED: read in session]
- `ferro-cli/src/commands/do_init.rs` — caller pattern, `AppYamlContext` construction [VERIFIED: read in session]
- `ferro-orm/Cargo.toml` — Wave 1a crate pattern (dep set, Cargo.toml shape) [VERIFIED: read in session]
- `.github/workflows/publish.yml:201` — `WAVE1A_CRATES` string [VERIFIED: read in session]

### Secondary (MEDIUM confidence)

- `docs.digitalocean.com/products/app-platform/reference/app-spec/` — PRE_DEPLOY job YAML format [CITED: fetched in session]
- `docs.digitalocean.com/products/app-platform/how-to/manage-jobs/` — job configuration details [CITED: fetched in session]

### Tertiary (LOW confidence)

- `gen_random_bytes` pgcrypto dependency, `gen_random_uuid` Postgres 13+ availability — training knowledge, not fetched from official Postgres docs [ASSUMED]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps verified in workspace
- Architecture: HIGH — all patterns verified from actual source files
- Pitfalls: HIGH — three of six pitfalls verified from actual code reading; two are standard Rust/SeaORM knowledge
- DigitalOcean spec: MEDIUM — verified from official docs fetch

**Research date:** 2026-05-14
**Valid until:** 2026-07-14 (stable domain; DO spec and SeaORM 1.0 are stable)
