# Phase 229: Benchmark Harness Foundation (1A) - Research

**Researched:** 2026-06-15
**Domain:** Ferro app-authoring API, oha/tokei CLI flags and JSON schemas, pinned version validity
**Confidence:** HIGH (Ferro API verified against live source; oha/tokei schema verified against official docs/repo)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: PostgreSQL 16.4, one pinned container shared by both apps.
- D-02: Minimal idiomatic Laravel 11 / PHP 8.3 app authored for micro-endpoints.
- D-03: Projection-slice resource = `articles`, deferred to 1C.
- D-04: Canonical perf on a documented fixed local machine; CI runs conformance + perf smoke only.
- D-05: `oha 1.4.7`, `tokei 12.1.2`, built into toolbox from `rust:1.88.0-slim-bookworm`.
- D-06: Ferro app `--release` from `rust:1.88.0-slim-bookworm`; Laravel from `php:8.3-cli-bookworm`.
- D-07: `parse_perf.parse_oha(raw)` -> `{rps, p50_ms, p90_ms, p99_ms, success_rate}`.
- D-08: `count_static.run(app_dir)` -> `{code_lines, files, source_tokens}`.
- D-09: Result filenames `perf-<framework>.json` / `static-<framework>.json`.
- D-10: Internal report includes every number; public table is a strict subset.
- D-11: Pinned versions, containerised apps, committed raw JSON + derived tables + hardware record.

### Claude's Discretion
- Exact README/table prose.
- Precise oha concurrency/duration defaults (PRD: 30s, c=256, 5s warmup) may be tuned.

### Deferred Ideas (OUT OF SCOPE)
- Conduit (1B), projection slice (1C), Rails/Django (Phase 2), agent-authoring (Phase 3),
  Octane/Swoole-tuned Laravel variant.
</user_constraints>

---

## Summary

This research resolves the three real unknowns the PRD flagged: (1) the correct Ferro handler/routing/DB-extractor API for Task 7, (2) oha flag and JSON-key correctness, and (3) pinned-version validity for oha and tokei. The PRD's harness Python code and Dockerfiles are trusted and are not re-derived here.

**Primary recommendation:** Use the corrected handler pattern below (no `Db` parameter — use `DB::get()?` inside the body), fix the oha flag (`--output-format json` not `--json`), and note that tokei 12.1.2 is old but still installable with `--locked` — the PRD pins can be used but the executor must verify the cargo install succeeds when building the toolbox image.

---

## Ferro App-Authoring API (the one real unknown)

### Handler Signature and Return Type

`#[handler]` is a proc-macro from `ferro-macros`. It transforms a plain `async fn` into a handler that accepts a request. The macro auto-extracts parameters based on type:

| Param type | Extraction strategy |
|------------|---------------------|
| `Request` | Passed through by move |
| `i32`, `i64`, `String`, ... (primitives) | Extracted from path params via `FromParam` |
| `*::Model` | Route model binding via `AutoRouteBinding` |
| Any other type | `FromRequest::from_request(req).await?` |

**Critical finding: the macro has NO `Db` / `Database` / `DbConnection` extractor.** The PRD's handler code `pub async fn db(db: Db) -> Response` will not compile. The handler only accepts `Request`, primitives, `::Model`, or `FromRequest` types as parameters. The database must be obtained inside the body via `DB::get()?`. [VERIFIED: ferro-macros/src/utils.rs `classify_param_type`, ferro-macros/src/handler.rs]

Return type: `Response` = `Result<HttpResponse, HttpResponse>` (a type alias). The `?` operator converts `FrameworkError` to the `Err` variant. [VERIFIED: framework/src/http/mod.rs]

JSON responses: use `Ok(HttpResponse::json(serde_json::json!({ ... })))`. The `json_response!` macro is also available as a shorthand. [VERIFIED: framework/src/lib.rs lines 364-373, framework/src/http/response.rs]

### Database Access

`DB::get()?` returns a `DbConnection` which `Deref`s to `sea_orm::DatabaseConnection`. All SeaORM query methods work directly on `&*db` or `db.inner()`. The connection is a singleton stored in the App container, initialised by `DB::init().await` in `bootstrap.rs`. [VERIFIED: framework/src/database/mod.rs]

Pattern inside a handler body:
```rust
let db = DB::get()?;
let row = Entity::find_by_id(id).one(&*db).await
    .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;
```

Alternatively, using Ferro's `Model` trait (avoids raw `&*db`):
```rust
// Requires `impl ferro::Model for Entity {}` in the model file
let row = Entity::find_by_pk(id).await?; // calls DB::connection() internally
```

`Entity::find_by_pk(id).await?` is the idiomatic Ferro path — it internally calls `DB::connection()` and maps the SeaORM error to `FrameworkError`. [VERIFIED: framework/src/database/model.rs lines 93-102]

### Routes

Routes are registered via macros in `src/routes.rs`:
```rust
use ferro::{get, routes};
use crate::controllers;

routes! {
    get!("/json",    controllers::bench::json_handler),
    get!("/db",      controllers::bench::db_handler),
    get!("/queries", controllers::bench::queries),
    get!("/updates", controllers::bench::updates),
}
```
[VERIFIED: ferro-cli/src/templates/files/backend/routes.rs.tpl]

`Server::from_config(router).run().await` consumes the `routes!` output directly. [VERIFIED: ferro-cli/src/templates/files/backend/main.rs.tpl, framework/src/server.rs]

### Query String Access

`req.query("n")` returns `Option<String>` (URL-decoded). `req.query_as::<i32>("n")` returns `Option<i32>`. [VERIFIED: framework/src/http/request.rs lines 201-232]

### Model and Migration

SeaORM `DeriveEntityModel` + `DeriveRelation` + `ActiveModelBehavior`. Ferro's `Model` and `ModelMut` traits add `find_by_pk`, `insert_one`, `update_one`, etc. [VERIFIED: framework/src/database/model.rs, ferro-cli/src/templates/files/backend/models/user.rs.tpl]

Migration: `sea_orm_migration::prelude::*`, `#[derive(DeriveMigrationName)]`, `async_trait::async_trait`, `impl MigrationTrait for Migration`. [VERIFIED: ferro-cli/src/templates/files/backend/migrations/create_users_table.rs.tpl]

### CLI Commands (confirmed names)

| Command | What it does |
|---------|-------------|
| `./ferro-micro serve` | Starts the server (runs migrations by default) |
| `./ferro-micro serve --no-migrate` | Starts without running migrations |
| `./ferro-micro db:migrate` | Runs pending migrations |
| `./ferro-micro db:fresh` | Drops all tables and re-runs migrations |
| `./ferro-micro db:seed` | Runs all registered seeders |
| `./ferro-micro db:seed --class WorldSeeder` | Runs a specific seeder by struct name |

[VERIFIED: ferro-cli/src/templates/files/backend/main.rs.tpl, Commands enum]

### Server Host/Port

Configured via environment variables — NOT CLI flags on the binary:
- `SERVER_HOST` (default: `127.0.0.1`)
- `SERVER_PORT` (default: `8080`)
- `DATABASE_URL` (default: `sqlite://./database.db`)

[VERIFIED: framework/src/config/providers/server.rs, framework/src/database/config.rs, ferro-cli/src/templates/files/root/env.tpl]

**The binary's `serve` subcommand does NOT accept `--host`/`--port` CLI flags.** The PRD Dockerfile CMD `["app", "serve", "--host", "0.0.0.0", "--port", "3000"]` will fail because the binary only accepts `--no-migrate` on `serve`. The correct Dockerfile CMD is env-var based (see Corrected Dockerfile below).

[VERIFIED: ferro-cli/src/templates/files/backend/main.rs.tpl — `Commands::Serve { no_migrate: bool }` only]

### Scaffolding the micro-app

`ferro new ferro-micro --no-interaction` creates a full-stack project (with React/Inertia frontend). The benchmark app needs only the backend. The scaffolded `Dockerfile` should use `cargo build --release --bin ferro-micro`. The binary is at `target/release/ferro-micro`.

The app must NOT be added to the root workspace `members`. Built in its own Docker context with no `[patch.crates-io]` block needed (it pins `ferro = { package = "ferro-rs", version = "0.2" }` against crates.io). [VERIFIED: ferro-cli/src/templates/files/backend/Cargo.toml.tpl]

### Seeder Interface

```rust
use ferro::{async_trait, Seeder, FrameworkError};
use sea_orm::DatabaseConnection;

pub struct WorldSeeder;

#[async_trait]
impl Seeder for WorldSeeder {
    async fn run(&self, db: &DatabaseConnection) -> Result<(), FrameworkError> {
        // insert 10000 rows using db
        Ok(())
    }
}
```

Register in `src/seeders/mod.rs`:
```rust
pub fn register() -> ferro::SeederRegistry {
    ferro::SeederRegistry::new().add::<WorldSeeder>()
}
```
[VERIFIED: framework/src/seeder/mod.rs, ferro-cli/src/commands/make_seeder.rs]

---

## Corrected Ferro Handler Code (Task 7)

Replace the PRD's illustrative code with the following verified pattern:

```rust
// src/controllers/bench.rs
use ferro::{handler, Request, Response, DB};
use ferro::http::HttpResponse;
use sea_orm::{EntityTrait, ActiveModelTrait, Set};
use rand::Rng;
use serde_json::json;

use crate::models::world;

#[handler]
pub async fn json_handler() -> Response {
    Ok(HttpResponse::json(json!({ "message": "Hello, World!" })))
}

#[handler]
pub async fn db_handler() -> Response {
    let db = DB::get()?;
    let id = rand::thread_rng().gen_range(1i32..=10_000);
    let row = world::Entity::find_by_id(id)
        .one(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
        .ok_or_else(|| ferro::FrameworkError::database("world row not found".to_string()))?;
    Ok(HttpResponse::json(json!({ "id": row.id, "randomNumber": row.random_number })))
}

fn clamp_n(n: Option<String>) -> i32 {
    n.and_then(|s| s.parse::<i32>().ok()).unwrap_or(1).clamp(1, 500)
}

#[handler]
pub async fn queries(req: Request) -> Response {
    let k = clamp_n(req.query("n"));
    let db = DB::get()?;
    let mut out = Vec::with_capacity(k as usize);
    for _ in 0..k {
        let id = rand::thread_rng().gen_range(1i32..=10_000);
        let row = world::Entity::find_by_id(id)
            .one(&*db)
            .await
            .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
            .ok_or_else(|| ferro::FrameworkError::database("world row not found".to_string()))?;
        out.push(json!({ "id": row.id, "randomNumber": row.random_number }));
    }
    Ok(HttpResponse::json(json!(out)))
}

#[handler]
pub async fn updates(req: Request) -> Response {
    let k = clamp_n(req.query("n"));
    let db = DB::get()?;
    let mut out = Vec::with_capacity(k as usize);
    for _ in 0..k {
        let id = rand::thread_rng().gen_range(1i32..=10_000);
        let mut row: world::ActiveModel = world::Entity::find_by_id(id)
            .one(&*db)
            .await
            .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
            .ok_or_else(|| ferro::FrameworkError::database("world row not found".to_string()))?
            .into();
        let new_n = rand::thread_rng().gen_range(1i32..=10_000);
        row.random_number = Set(new_n);
        let saved = row.update(&*db)
            .await
            .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;
        out.push(json!({ "id": saved.id, "randomNumber": saved.random_number }));
    }
    Ok(HttpResponse::json(json!(out)))
}
```

Key differences from the PRD's illustrative code:
- `DB::get()?` called inside handler body — NOT as a function parameter
- `req.query("n")` returns `Option<String>` — needs `.and_then(|s| s.parse().ok())`
- `FrameworkError::database(string)` used for not-found errors (safe fallback)
- No `Db` type in function signature

### World model (`src/models/world.rs`)

```rust
use ferro::database::{Model as DatabaseModel, ModelMut};
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "world")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub random_number: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
impl DatabaseModel for Entity {}
impl ModelMut for Entity {}
```
[VERIFIED pattern from ferro-cli/src/templates/files/backend/models/user.rs.tpl]

### World migration (`src/migrations/create_world_table.rs`)

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(World::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(World::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(World::RandomNumber).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(World::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum World {
    Table,
    Id,
    RandomNumber,
}
```
[VERIFIED pattern from ferro-cli/src/templates/files/backend/migrations/create_users_table.rs.tpl]

### Corrected Dockerfile for the Ferro app

```dockerfile
# benchmark/apps/ferro-micro/Dockerfile
FROM rust:1.88.0-slim-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libpq5 \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/ferro-micro /usr/local/bin/app
# Server host/port are env vars, not CLI flags — the binary does not accept --host/--port
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000
EXPOSE 3000
CMD ["/usr/local/bin/app", "serve"]
```

### World seeder (`src/seeders/world_seeder.rs`)

```rust
use ferro::{async_trait, FrameworkError, Seeder};
use sea_orm::{DatabaseConnection, EntityTrait, Set};

use crate::models::world;

pub struct WorldSeeder;

#[async_trait]
impl Seeder for WorldSeeder {
    async fn run(&self, db: &DatabaseConnection) -> Result<(), FrameworkError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let rows: Vec<world::ActiveModel> = (1..=10_000)
            .map(|_| world::ActiveModel {
                random_number: Set(rng.gen_range(1..=10_000)),
                ..Default::default()
            })
            .collect();
        world::Entity::insert_many(rows)
            .exec(db)
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;
        Ok(())
    }
}
```

Note: `insert_many` is a standard SeaORM method on `EntityTrait`, not a Ferro method. It accepts `&DatabaseConnection` directly. [ASSUMED — SeaORM `insert_many` API shape; executor should verify against sea-orm docs]

---

## oha Tool: Critical Flag Correction

**The PRD uses `--json`. The correct flag is `--output-format json`.**

oha does not have a `--json` short flag. The correct flag is `--output-format json` (two tokens: the flag name and its value). This affects `run_perf.py`.

### Confirmed flags:
| Flag | Status |
|------|--------|
| `-z <DURATION>` | Confirmed (e.g. `-z 30s`) |
| `-c <N>` | Confirmed (concurrent connections) |
| `--no-tui` | Confirmed |
| `--output-format json` | Correct — two arguments: `"--output-format"` and `"json"` |
| `--json` | Does NOT exist |

[VERIFIED: github.com/hatoo/oha README via WebFetch]

### oha JSON schema keys (confirmed match with PRD's parse_oha):

```
summary.requestsPerSec   -> float (req/s)
summary.successRate      -> float 0.0-1.0
latencyPercentiles.p50   -> float (seconds, multiply by 1000 for ms)
latencyPercentiles.p90   -> float (seconds)
latencyPercentiles.p99   -> float (seconds)
```

**All key names used in the PRD's `parse_oha` function are correct.** The latency-to-millisecond conversion (multiply by 1000) is correct. [VERIFIED: github.com/hatoo/oha schema.json via WebFetch]

### Corrected subprocess call in `run_perf.py`:

```python
# warmup (discard output)
subprocess.run(
    ["oha", "-z", warmup, "-c", concurrency, "--no-tui", url],
    capture_output=True, text=True,
)
# timed run
raw = subprocess.run(
    ["oha", "-z", duration, "-c", concurrency, "--no-tui",
     "--output-format", "json", url],   # <-- two args, not "--json"
    capture_output=True, text=True, check=True,
).stdout
```

---

## tokei Tool: Version and JSON Schema

### Version status

- tokei 12.1.2: released 2021-01-12, still available on crates.io, not yanked. [MEDIUM confidence]
- Latest tokei: 14.0.0 (released 2025-12-30). The PRD pin of 12.1.2 is 3 major versions behind.
- `cargo install tokei --version 12.1.2 --locked` should succeed but executor must verify during toolbox image build.

### tokei JSON output structure (12.1.2 schema)

`tokei --output json <dir>` produces a map from language name to stats:
```json
{
  "Rust": {
    "code": 120,
    "comments": 10,
    "blanks": 5,
    "reports": [
      { "name": "/abs/path/to/file.rs", "stats": { ... } },
      ...
    ]
  },
  "Total": { "code": 120, ... }
}
```

`tokei --files --output json <dir>` — same structure, `reports` array is populated per-language.

The `count_static.py` reads `v.get("code", 0)` and `v.get("reports", [])` with `r["name"]` for file paths. These key names are correct for tokei 12.1.2. [MEDIUM confidence — cross-referenced against tokei issue #311 and multiple usage examples]

---

## Pinned Version Reality Check

| Component | PRD Pin | Available | Notes |
|-----------|---------|-----------|-------|
| `oha` | 1.4.7 | Yes (not yanked) | Latest is 1.14.0; 1.4.7 usable with `--locked`. Verify `--output-format json` exists in that version at toolbox build time. |
| `tokei` | 12.1.2 | Yes (not yanked) | Latest is 14.0.0; 12.1.2 usable with `--locked`. JSON schema stable across 12.x. |
| `postgres` | 16.4 | Yes | Valid Docker Hub tag. |
| `php` | 8.3-cli-bookworm | Yes | Valid Docker Hub tag. |
| `rust` | 1.88.0-slim-bookworm | Yes | Current ferro workspace MSRV. |
| Laravel | 11.* | Yes | Current stable. |

---

## Workspace Isolation

The Ferro micro-app must not appear in the root workspace `members`. The scaffolded `Cargo.toml` has no `[workspace]` declaration, so cargo will look up to find the root workspace when run locally. Inside the Docker build context, the root workspace file is absent — the build is self-contained. This is safe. Do not add `benchmark/apps/ferro-micro` to the root `Cargo.toml` members. [VERIFIED: scaffold Cargo.toml.tpl has no workspace section]

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | SeaORM `integer().auto_increment().primary_key()` maps to Postgres `SERIAL` DDL | World migration | Migration may fail or use wrong column type; check SeaORM Postgres DDL output or use `big_integer()` |
| A2 | `rand` crate must be added to the scaffold's `Cargo.toml` (it is not in the template) | Handler code | Compile error; fix: add `rand = "0.8"` to `Cargo.toml` |
| A3 | oha 1.4.7 supports `--output-format json` (not a different flag name) | oha flags | If 1.4.7 uses `--json` instead, update `run_perf.py` and bump pin |
| A4 | tokei 12.1.2's JSON `reports[i]["name"]` is a flat string (not a nested object) | count_static.py | JSON parse error; test by running `tokei 12.1.2 --output json` on a sample dir |
| A5 | `world::Entity::insert_many(rows).exec(db)` compiles correctly with SeaORM 1.0 | Seeder | Compile error; alternative: loop with `ActiveModel.insert(db)` per row |

---

## Open Questions

1. **Does oha 1.4.7 use `--output-format json` or `--json`?**
   - Recommendation: After building the toolbox image, run `docker run --rm ferro-bench-toolbox oha --help | grep -i output` to verify the flag name before writing `run_perf.py`.

2. **What is the exact `FrameworkError` variant for not-found?**
   - `FrameworkError::database(msg.to_string())` is always safe. Check `framework/src/error.rs` for any `not_found` or `model_not_found` constructors if a cleaner HTTP 404 is desired.

---

## Sources

### Primary (HIGH confidence)
- `ferro-macros/src/utils.rs` — `classify_param_type` verifies no Db extractor in `#[handler]`
- `ferro-macros/src/handler.rs` — handler macro shape and parameter dispatch
- `framework/src/database/mod.rs` — `DB::get()`, `DB::connection()` API
- `framework/src/database/model.rs` — `Model::find_by_pk`, `ModelMut::update_one`
- `framework/src/config/providers/server.rs` — `SERVER_HOST`, `SERVER_PORT` env vars
- `framework/src/database/config.rs` — `DATABASE_URL` env var
- `framework/src/http/request.rs` — `req.query(name)` returns `Option<String>`
- `ferro-cli/src/templates/files/backend/main.rs.tpl` — `serve` accepts only `--no-migrate`
- `ferro-cli/src/templates/files/backend/routes.rs.tpl` — `routes!`, `get!` macro shape
- `framework/src/seeder/mod.rs` — `Seeder` trait with `db: &DatabaseConnection`
- github.com/hatoo/oha README — `--output-format json` flag confirmed
- github.com/hatoo/oha schema.json — JSON key names `requestsPerSec`, `successRate`, `latencyPercentiles.p50/p90/p99`

### Secondary (MEDIUM confidence)
- docs.rs/crate/tokei/latest — tokei 12.1.2 available and not yanked; latest is 14.0.0
- tokei GitHub issues and usage docs — JSON `reports` array structure with `name` field

### Tertiary (LOW confidence)
- SeaORM Postgres `SERIAL` mapping via `integer().auto_increment()` — inferred from migration template pattern, not verified against Postgres DDL output
- oha 1.4.7 specific flag set — `--output-format json` confirmed for current oha; specific availability in 1.4.7 not verified
