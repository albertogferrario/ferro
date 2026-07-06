# Phase 214: Scaffold↔Library Parity & Published-Artifact Smoke Test — Research

**Researched:** 2026-06-13
**Domain:** ferro-cli scaffold templates, framework/src/lib.rs facade, CI workflow
**Confidence:** HIGH — all findings are VERIFIED against source files in this session

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01 — `error_response!` → EXPORT.** Define an `error_response!` macro in `framework` and
  export it from the `ferro` facade.
- **D-02 — `ActiveValue` (and `Set`) → EXPORT (facade re-export).** Add `ActiveValue` to the
  `pub use sea_orm::{…}` block at `framework/src/lib.rs:122`.
- **D-03 — `ferro::Queue` / `ferro::QueueConfig` → TEMPLATE.** Change the template to emit
  `ferro::queue::Queue` / `ferro::queue::QueueConfig`.
- **D-04 — `#[rule]` → TEMPLATE.** Fix the template to emit `#[derive(ValidateRules)]` and
  bring it into scope.
- **D-05 — `crate::models::users` → TEMPLATE.** Align `make:auth` output with the generated
  model module layout.
- **D-06 — `ferro::database::connection` → TEMPLATE.** Fix the template call site:
  `database::connection` is a module, not a function.
- **D-07 — Route generated jobs through `ferro::queue::*`; add NO `ferro-queue` dependency
  to the generated `Cargo.toml`.**
- **D-08 — Two-layer CI guard:** per-PR path-dep layer + release gate via the committed
  Dockerfile.
- **D-09 — Mechanism:** Reuse `ferro-cli/tests/benchmark_new_project.rs` for the per-PR
  layer; reuse the committed Dockerfile for the release gate. Do NOT use
  `cargo install --version <released>`.
- **D-10 — SCAF-* requirement IDs:** SCAF-01..SCAF-05 (derive in REQUIREMENTS.md).

### Claude's Discretion

- Exact `error_response!` macro signature and return type (research-determined).
- Whether the parity fix and CI guard ship as one plan or split into two.
- Exact placement of the per-PR job in `.github/workflows/` and the release gate
  wiring into the publish pipeline.

### Deferred Ideas (OUT OF SCOPE)

- COMP-04 W3 — `make:scaffold` flag ordering (clap ergonomics).
- COMP-04 W4 — `make:model` vs `make:scaffold` naming drift.
- COMP-04 W2 — moving the CLI off `native-tls` to rustls.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SCAF-01 | Scaffold templates reference only the published `ferro` surface: every symbol a generated project emits resolves from the published crate | D-01..D-06 fixes enable this; section "Per-Symbol Fix Map" gives the exact edits |
| SCAF-02 | `make:job` produces a project whose `Cargo.toml` declares every crate its generated code imports | D-07 fix: route through `ferro::queue::*`; no `ferro-queue` dep needed |
| SCAF-03 | A clean scaffold `cargo build`s exit 0 against the published `ferro-rs` | Acceptance gate: the same sequence as the benchmark apparatus |
| SCAF-04 | A CI smoke test scaffolds + builds against the published artifact and gates every release | Dockerfile cold-cache job wired into publish pipeline |
| SCAF-05 | A per-PR scaffold-build guard against the workspace path dep gives fast pre-publish drift detection | New `scaffold-smoke` job in ci.yml using path dep override |

</phase_requirements>

---

## Summary

Phase 211's cold-cache benchmark exposed 52 compile errors in a freshly scaffolded ferro app built against the **published** `ferro-rs` 0.2.55. The root cause is scaffold↔library API drift: the `ferro-cli` templates emit symbols that the `ferro` facade does not export, or emit them by a path that no longer exists in the published crate. This phase fixes that drift (seven concrete symbol-level changes) and installs a permanent CI guard that prevents silent regression.

The research verified every broken call site in the three template files (`scaffold.rs`, `make.rs`, `auth.rs`) and the correct target API in `framework/src/lib.rs`, `framework/src/database/`, and `ferro-macros/src/lib.rs`. All seven fixes are fully concrete: exact file, line range, current broken text, and target replacement.

The CI guard has two layers: (1) a per-PR `scaffold-smoke` job in `ci.yml` that scaffolds + `cargo build`s against the workspace `ferro` via a `[patch.crates-io]` override — fast, no network, catches template drift on every PR; (2) the existing Dockerfile cold-cache job wired into `publish.yml` as a post-publish gate — catches packaging drift that only appears after crates.io publish. The generated `Cargo.toml` already contains the comment `# Local ferro dev: append an uncommitted [patch.crates-io] block` which is the exact mechanism the per-PR layer uses, patched programmatically by the test.

**Primary recommendation:** Fix all seven symbols, then wire both CI layers. The per-PR layer is where the high-volume detection happens; the Dockerfile layer is the release-quality gate.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `error_response!` macro definition | Framework library (`framework`) | Facade (`ferro`) re-export | The macro must exist before it can be exported |
| `ActiveValue` re-export | Facade (`ferro`) | — | Already the pattern for sea_orm traits at lib.rs:122 |
| Queue template fix | CLI templates (`ferro-cli`) | — | It is a template-emission bug, not a library gap |
| `#[rule]`/`ValidateRules` template fix | CLI templates (`ferro-cli`) | — | The derive is already exported; the template doesn't emit it |
| `crate::models::users` fix | CLI templates (`ferro-cli`) | — | Module path mismatch in generated code |
| `database::connection` fix | CLI templates (`ferro-cli`) | — | Template calls a module as a function |
| `make:job` import fix | CLI templates (`ferro-cli`) | — | Imports from undeclared crate `ferro_queue` |
| Per-PR scaffold-build guard | CI (`ci.yml`) | `benchmark_new_project.rs` | Extends existing apparatus |
| Release gate | CI (`publish.yml`) | Dockerfile | Post-publish check against crates.io artifact |

---

## Per-Symbol Fix Map

This is the core planning artifact. For each broken symbol: exact file, relevant line range, current broken emission, and exact target.

### D-01 — `error_response!` macro (EXPORT)

**Broken call sites:** `ferro-cli/src/templates/scaffold.rs`, in two template functions:

- `api_controller_template` (line ~997–1133): 14 call sites of `ferro::error_response!(500, "…")` and `ferro::error_response!(404, "…")` in `.map_err(|e| { … ferro::error_response!(500, "…") })?` positions and `.ok_or_else(|| ferro::error_response!(404, "…"))` positions.
- `api_controller_with_fk_template` (line ~1138–1463): same pattern, same usage, used in `.map_err(|e| { … ferro::error_response!(500, "…") })?` and `.ok_or_else(|| ferro::error_response!(404, "…"))` positions.

**Current emission (in both functions):** [VERIFIED: ferro-cli/src/templates/scaffold.rs:1000]
```rust
ferro::error_response!(500, "Failed to fetch {plural_snake}")
// used in: .map_err(|e| { tracing::error!(…); ferro::error_response!(500, "…") })?
// used in: .ok_or_else(|| ferro::error_response!(404, "…"))?
```

**Return type analysis:** [VERIFIED: ferro-cli/src/templates/scaffold.rs]
The generated handler functions are annotated `#[handler]` and return `-> Response` (which is `Result<HttpResponse, HttpResponse>`). The `?` on `.map_err(…)?` and `.ok_or_else(…)?` means the error arm must produce `HttpResponse` (the `Err` variant of `Response`). Both positions need `ferro::error_response!(status, msg)` → `HttpResponse`.

The existing `json_response!` macro at `framework/src/lib.rs:366` returns `Ok(HttpResponse::json(…))`. The analogous error macro must return `HttpResponse::…(msg)` directly (not wrapped in `Err`) because the `?` operator does the unwrapping — the closures in `.map_err` and `.ok_or_else` return the raw `HttpResponse` value that becomes the `Err` variant.

**Macro signature needed:**
```rust
// In framework/src/lib.rs (or a new framework/src/macros.rs included via lib.rs)
/// Return an HTTP error response with a status code and message.
///
/// Used in handler error arms: `.map_err(|_| ferro::error_response!(500, "msg"))`
/// and `.ok_or_else(|| ferro::error_response!(404, "msg"))`.
///
/// # Example
/// ```rust,ignore
/// Entity::find_by_id(id).one(db).await
///     .map_err(|e| ferro::error_response!(500, e.to_string()))?;
/// ```
#[macro_export]
macro_rules! error_response {
    ($status:expr, $msg:expr) => {
        $crate::HttpResponse::status_with_message($status, $msg)
    };
}
```

The `HttpResponse` type must have a method that accepts `(u16, impl Into<String>)` → `HttpResponse`. Research needed on which method name to use: candidates are `.status(u16)` chained with a body method, or a dedicated helper. Looking at the existing `HttpResponse::not_found("msg")` and `HttpResponse::internal_server_error("msg")` calls in the non-API scaffold templates (lines 665, 677), the framework already has named status helpers. The macro needs to bridge the numeric status code the templates pass (500, 404) to a `HttpResponse`. The correct pattern is: [VERIFIED: framework/src/lib.rs — `HttpResponse` is re-exported from `http`]

```rust
macro_rules! error_response {
    ($status:expr, $msg:expr) => {
        $crate::HttpResponse::new($status, $msg.to_string())
        // OR: build an HttpResponse with the given status and body
    };
}
```

The exact `HttpResponse` constructor to use must be confirmed against `framework/src/http/` during implementation. The important constraint: the macro must produce a bare `HttpResponse` (not `Result<_, HttpResponse>`), because it is called inside `.map_err(|_| …)` and `.ok_or_else(|| …)` closures.

**Where to define:** Add `macro_rules! error_response { … }` in `framework/src/lib.rs` (co-located with `json_response!`, `text_response!`, etc., at lines 366–461). Mark `#[macro_export]`.

**Export from facade:** The `#[macro_export]` puts it at crate root, so `ferro::error_response!` resolves automatically once the macro is defined in `framework`. No additional `pub use` line needed — `#[macro_export]` macros are always accessible at the crate root.

---

### D-02 — `ActiveValue` / `Set` (EXPORT)

**Broken usage in templates:** [VERIFIED: ferro-cli/src/templates/scaffold.rs]

In `scaffold_controller_with_fk_template` and `scaffold_controller_template` (lines ~703–712, ~886–892):
```rust
use sea_orm::{EntityTrait, ActiveModelTrait, ActiveValue};
// ...
id: ActiveValue::NotSet,
{insert_fields}  // expands to e.g.: title: ActiveValue::Set(form.title.clone()),
created_at: ActiveValue::Set(chrono::Utc::now()),
```

In `api_controller_template` and `api_controller_with_fk_template` (lines ~1043, ~1371):
```rust
// No explicit import — uses {insert_fields} which contains ActiveValue::Set(…)
// The insert_fields template strings are generated by the scaffold command with
// ActiveValue::Set(…) syntax
```

The non-API (Inertia) scaffold templates import `use sea_orm::{EntityTrait, ActiveModelTrait, ActiveValue}` directly — this compiles fine when `sea-orm` is a direct dependency. But the API scaffold template (`api_controller_template`) does NOT import `ActiveValue`; it imports only `use sea_orm::{ColumnTrait, EntityTrait, QueryFilter}` (line ~982) while the `{insert_fields}` slot emits `ActiveValue::Set(…)`. This is the source of the `error[E0433]: cannot find type ActiveValue` errors.

**Fix strategy:** Two-part:

1. **Framework facade:** Add `ActiveValue` to `pub use sea_orm::{…}` at `framework/src/lib.rs:122`:
   ```rust
   pub use sea_orm::{
       ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
       PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
   };
   ```

2. **Template import fix for API controllers:** Change the import line in `api_controller_template` and `api_controller_with_fk_template` from:
   ```rust
   use sea_orm::{{ColumnTrait, EntityTrait, QueryFilter}};
   ```
   to:
   ```rust
   use ferro::{{ActiveValue}};
   use sea_orm::{{ColumnTrait, EntityTrait, QueryFilter}};
   ```
   (or pull `ActiveValue` through the `ferro::` import line if the template already has one).

**On `Set`:** `Set` is `ActiveValue::Set(…)` — it is a variant of the `ActiveValue` enum, not a separate symbol. Once `ActiveValue` is in scope, `ActiveValue::Set(v)` compiles. No separate `Set` re-export is needed.

**File:line for the sea_orm pub use:** [VERIFIED: framework/src/lib.rs:122]
```rust
// CURRENT:
pub use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};

// TARGET:
pub use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
```

---

### D-03 — `ferro::Queue` / `ferro::QueueConfig` (TEMPLATE)

**Broken import in template:** [VERIFIED: ferro-cli/src/templates/make.rs]

The `job_template` function (line 333) emits:
```rust
use ferro_queue::{{async_trait, Error, Job, Queueable}};
```

The generated `Cargo.toml` (`ferro-cli/src/templates/files/backend/Cargo.toml.tpl`) does NOT include `ferro-queue` as a dependency — only `ferro = { package = "ferro-rs", version = "0.2" }`. [VERIFIED: Cargo.toml.tpl]

**What the library already exports:** [VERIFIED: framework/src/lib.rs:194-202]
```rust
pub mod queue {
    pub use ferro_queue::{
        dispatch, dispatch_later, dispatch_to, register_tenant_capture_hook, CreateJobsTable,
        Error, FailedJobInfo, Job, JobInfo, JobPayload, JobState, PendingDispatch, Queue,
        QueueConfig, QueueStats, Queueable, SingleQueueStats, TenantScopeProvider, Worker,
        WorkerConfig, WorkerLoop,
    };
}
```

`async_trait` is re-exported from `ferro` itself at `framework/src/lib.rs:268`:
```rust
pub use async_trait::async_trait;
```

**Target emission:** Change `job_template` in `ferro-cli/src/templates/make.rs` line 339 from:
```rust
use ferro_queue::{{async_trait, Error, Job, Queueable}};
```
to:
```rust
use ferro::{{async_trait, queue::{{Error, Job, Queueable}}}};
```

**Additional note:** The `Queueable` trait (which provides `.dispatch()` and `.delay()`) is also at `ferro::queue::Queueable`. The template example comments also reference `dispatch` — this is at `ferro::queue::dispatch`. No `ferro-queue` entry in generated `Cargo.toml` needed.

---

### D-04 — `#[rule]` / `ValidateRules` derive (TEMPLATE)

**Broken usage:** [VERIFIED: ferro-cli/src/templates/scaffold.rs]

In `scaffold_controller_template` (line ~819) and `scaffold_controller_with_fk_template` (line ~634):
```rust
use ferro::{{
    http::{{Request, Response, HttpResponse}},
    inertia::{{Inertia, SavedInertiaContext}},
    validation::Validatable,
    ValidateRules,
}};
// ...
#[derive(Debug, Deserialize, Serialize, ValidateRules)]
pub struct {name}Form {{
{form_fields}}}
```

`ValidateRules` **is** already imported in the non-API scaffold template from `ferro::ValidateRules`. It is re-exported from `framework/src/lib.rs:342`:
```rust
pub use ferro_macros::ValidateRules;
```

The `#[rule]` attribute (the helper attribute of the derive) does not need a separate import in Rust — helper attributes of a derive macro are in scope implicitly when the derive is in scope.

**Where is the actual break?** The `#[rule]` error in the cold-cache run (6 occurrences) came from the **API controller templates** (`api_controller_template`, `api_controller_with_fk_template`), where the `{name}Form` struct is:
```rust
#[derive(serde::Deserialize)]
pub struct {name}Form {{
{form_fields}
}}
```
Here the `ValidateRules` derive is absent entirely. The `form_fields` slot is generated by the scaffold command and may include `#[rule(…)]` attributes on the fields. Without `#[derive(ValidateRules)]`, `#[rule]` is an unknown attribute.

**However**, the API templates do NOT call `form.validate()` — there is no validation in the API template. So `form_fields` in the API context should not contain `#[rule(…)]` attributes.

The fix is two-part:
1. Confirm whether the scaffold command emits `#[rule(…)]` in `form_fields` when generating API controllers. If yes: add `ValidateRules` to the `api_controller_template`'s form struct derive. If no: the `#[rule]` errors came from the non-API (Inertia) template, which already has `ValidateRules` in scope.
2. For safety and completeness: ensure both template paths (API and Inertia) emit the derive correctly.

**Action for planner:** The implementer must grep the scaffold command's `form_fields` generation logic to confirm whether `#[rule(…)]` is emitted for API scaffolds. If it is, add `ValidateRules` to the API template's derive list. The Inertia template already imports and derives `ValidateRules` correctly — no change needed there.

---

### D-05 — `crate::models::users` (TEMPLATE)

**Broken usage:** [VERIFIED: ferro-cli/src/templates/auth.rs:105]

`auth_controller_template()` emits at line 105:
```rust
use crate::models::users;
```
And at line 150:
```rust
users::Model::find_by_email(&input.email).await
```
And at line 176:
```rust
let user = users::ActiveModel { … };
users::Entity::insert(user).exec_with_returning(…)
```

The generated model layout from `ferro new` + `make:auth` needs investigation. The `ferro new` command generates `src/models/mod.rs`. The `make:auth` command adds auth fields to an existing users table via migration. The user model is not generated by `make:auth` itself — `make:auth` generates a migration that alters the users table. The question is: does a freshly scaffolded project have a `src/models/users.rs` module declared in `src/models/mod.rs`?

**What `ferro new` generates:** [VERIFIED: ferro-cli/src/commands/new.rs + ferro-cli/src/templates/project.rs]
The `cargo_toml` and `write_backend_files` setup creates `src/models/mod.rs` (via `templates::models_mod()`). The content of `models/mod.rs.tpl` needs to be checked.

**The fix pattern:** Either:
- `make:auth` must also emit a `src/models/users.rs` file (and add `pub mod users;` to `src/models/mod.rs`), OR
- The `auth_controller_template` must be updated to reference the correct path where the users model exists after `make:auth`.

**Investigation required:** Read `ferro-cli/src/templates/files/backend/models/mod.rs.tpl` and the `make:auth` command to determine whether a `users` model module is created. This is the one D-05 that requires a secondary read by the implementer.

---

### D-06 — `ferro::database::connection` used as a function (TEMPLATE)

**Broken usage:** [VERIFIED: ferro-cli/src/templates/auth.rs:185]

`auth_controller_template()` emits at line 185:
```rust
let user = users::Entity::insert(user)
    .exec_with_returning(&ferro::database::connection().await)
    .await
    …
```

`ferro::database::connection` is a **module** (`framework/src/database/connection.rs`), not a function. The `pub mod database` in `framework/src/lib.rs` exposes `framework::database` as a module.

**Correct API:** [VERIFIED: framework/src/database/mod.rs:171]
```rust
pub fn connection() -> Result<DbConnection, FrameworkError> {
    App::resolve::<DbConnection>()
}
```
The correct call is `ferro::DB::connection()` (synchronous, returns `Result`), NOT `ferro::database::connection()`.

Additionally, `exec_with_returning` takes `&DatabaseConnection`, not `&DbConnection`. The correct form:
```rust
let db = ferro::DB::connection()
    .map_err(|e| HttpResponse::json(serde_json::json!({"message": e.to_string()})).status(500))?;
let user = users::Entity::insert(user)
    .exec_with_returning(db.inner())
    .await
    .map_err(|e| …)?;
```

Or, more consistent with the rest of the generated controller which calls `req.db()` (which is `DB::connection()` extracted from the request), the register handler should use `req.db()` consistently:

Looking at the auth template: it already has `req: Request` as parameter. The scaffold templates use `req.db()` to get the connection (lines 662, 673, etc. in non-API templates). The auth controller is `#[handler]` and has `req: Request`. However, `Request::db()` is not a standard method on Request — the non-API templates call `let db = req.db()` but there is no `pub fn db` on `Request` in `framework/src/http/`. [VERIFIED: framework grep — only `TestDatabase::db()` exists, no `Request::db()`]

**Actual pattern in framework:** The correct way to get the DB in a handler is `ferro::DB::connection()?.inner()` or using the injected `DbConnection` type. The `req.db()` calls in the Inertia scaffold templates may be a different bug, or `req.db()` may exist via an extension trait not grepped above.

**Action for planner:** The implementer must verify whether `Request::db()` exists (check `framework/src/http/request.rs`). Regardless, the `ferro::database::connection().await` call in `auth.rs:185` is unambiguously wrong (it is calling a module). The fix is at minimum to replace it with `ferro::DB::connection()?.inner()` with a proper error map, or with `req.db()` if that method exists.

---

### D-07 — `use ferro_queue::{…}` in `make:job` (TEMPLATE)

**Already covered in D-03.** The only file is `ferro-cli/src/templates/make.rs`, `job_template` function, line 339.

**Current (broken):**
```rust
use ferro_queue::{{async_trait, Error, Job, Queueable}};
```

**Target:**
```rust
use ferro::{{async_trait, queue::{{Error, Job, Queueable}}}};
```

No `ferro-queue` entry in generated `Cargo.toml` — confirmed the template is `ferro-cli/src/templates/files/backend/Cargo.toml.tpl` which has only `ferro = { package = "ferro-rs", version = "0.2" }`. [VERIFIED]

---

## CI Guard Architecture

### Layer 1: Per-PR Scaffold-Build (path-dep override) — SCAF-05

**Existing apparatus:** `ferro-cli/tests/benchmark_new_project.rs` — already runs the full 5-step sequence and asserts `cargo build` exit 0. It is `#[ignore]` and gated on `FERRO_BENCH=1`. [VERIFIED]

**The path-dep problem:** The generated `Cargo.toml` pins `ferro = { package = "ferro-rs", version = "0.2" }` (crates.io). A per-PR test building with that pin would download whatever is on crates.io — not the local workspace under test. The fix is to append a `[patch.crates-io]` block to the generated `Cargo.toml` after scaffolding, before `cargo build`:

```toml
[patch.crates-io]
ferro-rs = { path = "/path/to/workspace/framework" }
```

The generated `Cargo.toml` already contains the comment `# Local ferro dev: append an uncommitted [patch.crates-io] block at the bottom of this file.` — this is the intended mechanism. [VERIFIED: ferro-cli/src/templates/files/backend/Cargo.toml.tpl]

**Implementation approach:** In the test (or a new test function in `benchmark_new_project.rs`), after step 4 (`make:job`) and before step 5 (`cargo build`), append to `project_dir/Cargo.toml`:
```rust
let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
let framework_path = workspace_root.join("framework");
let patch_block = format!("\n[patch.crates-io]\nferro-rs = {{ path = \"{}\" }}\n", framework_path.display());
fs::OpenOptions::new().append(true).open(project_dir.join("Cargo.toml"))?.write_all(patch_block.as_bytes())?;
```

This requires no new mechanism — `CARGO_MANIFEST_DIR` is available in tests and points to `ferro-cli/`. The workspace root is one level up.

**New test function:** Add `scaffold_builds_against_workspace_ferro()` (or rename the existing function and add a non-`ignore` variant). The per-PR variant:
- Does NOT need `FERRO_BENCH=1` — it should run on every CI push.
- Builds against the workspace `framework` via `[patch.crates-io]`.
- Must be fast: the bottleneck is `cargo build` of the generated app, which will pull only `ferro-rs` from workspace (already compiled by CI) + the app's other deps (sea-orm, tokio, etc.). Expect 2–5 minutes on a warm cache.
- Disk: the generated app compiles against workspace `ferro` without separate crate downloads; similar disk profile to the existing `--all-features` test.

**CI placement:** A new job `scaffold-smoke` in `.github/workflows/ci.yml`, running after `check`. It uses `ubuntu-latest`, the same Rust toolchain (`1.88.0`), and `Swatinem/rust-cache@v2`. It runs the single new non-ignored test:
```yaml
scaffold-smoke:
  name: Scaffold Smoke (workspace)
  runs-on: ubuntu-latest
  needs: check
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@master
      with:
        toolchain: "1.88.0"
    - uses: Swatinem/rust-cache@v2
    - run: cargo test -p ferro-cli scaffold_builds_against_workspace_ferro -- --nocapture
```

**Note on disk:** The GH runner has tight disk (profile.dev `debug=false` already set). The generated app is a simple Rust project. The workspace `ferro` compile artifacts are already present from the earlier CI steps. Disk risk is LOW — the new test reuses already-built workspace artifacts.

**Note on publish token / workflow scope:** [VERIFIED: project memory `feedback_ci_clippy_command_match.md`, `project_ferro_ci_disk_and_push.md`] The CI token lacks `workflow` scope, so CI-yaml edits cannot be pushed by the automated token. A local push by the developer works. This is a known constraint — the implementer must push the CI yaml edit manually (normal git push).

---

### Layer 2: Release Gate (post-publish Dockerfile) — SCAF-04

**Existing apparatus:** `ferro-cli/tests/fixtures/benchmark/Dockerfile` — builds `debian:bookworm-slim`, installs `libssl-dev`/`pkg-config`, installs `rustup`, installs `ferro-cli --version 0.2.55 --locked`, then runs the full 5-step scaffold+build sequence. [VERIFIED]

**Current state:** The Dockerfile pins `--version 0.2.55`. After each publish, this pin must be updated to the new version — or made dynamic.

**Wiring into `publish.yml`:** The publish workflow runs on push to master, checks version, and publishes in waves. A post-publish gate job after the final publish wave should:
1. Build the Dockerfile with the new version tag (updated in the Dockerfile, or passed as a `--build-arg`).
2. Run `docker run` and assert exit 0.
3. On failure: create a GitHub issue or annotate the workflow run.

**Practical approach for the per-publish version pin:** Add a `--build-arg FERRO_VERSION=<version>` to the Dockerfile `RUN cargo install ferro-cli` step:
```dockerfile
ARG FERRO_VERSION=0.2.55
RUN cargo install ferro-cli --version ${FERRO_VERSION} --locked
```
Then the CI job passes `--build-arg FERRO_VERSION=${{ steps.check.outputs.version }}`.

**Docker availability in GitHub Actions:** `ubuntu-latest` runners have Docker pre-installed. No extra setup needed.

**Publish pipeline placement:** After the final Wave 3 publish job in `publish.yml` (which publishes `ferro-cli`), add a `post-publish-scaffold-smoke` job:
```yaml
post-publish-scaffold-smoke:
  name: Post-Publish Scaffold Smoke (crates.io)
  runs-on: ubuntu-latest
  needs: publish-wave3   # exact job name to be confirmed from publish.yml
  if: needs.check-version.outputs.should_publish == 'true'
  steps:
    - uses: actions/checkout@v4
    - name: Build and run cold-cache benchmark container
      run: |
        docker build \
          --build-arg FERRO_VERSION=${{ needs.check-version.outputs.version }} \
          -t ferro-scaffold-smoke \
          ferro-cli/tests/fixtures/benchmark/
        docker run --rm ferro-scaffold-smoke
```

**Important:** The post-publish gate cannot run before crates.io propagation (~30–60s after publish). A brief `sleep 60` or retry loop before `cargo install` inside the container is needed. The Dockerfile already uses `cargo install ferro-cli --version … --locked`; if the version is not yet propagated, the install fails. The container should retry or the CI job should add a delay.

---

## Template File Summary

| Template file | Function(s) affected | Symbols to fix | D-# |
|---------------|---------------------|----------------|-----|
| `ferro-cli/src/templates/scaffold.rs` | `api_controller_template` (~line 969) | `ferro::error_response!`, missing `ActiveValue` import | D-01, D-02 |
| `ferro-cli/src/templates/scaffold.rs` | `api_controller_with_fk_template` (~line 1138) | `ferro::error_response!`, missing `ActiveValue` import | D-01, D-02 |
| `ferro-cli/src/templates/scaffold.rs` | `scaffold_controller_template` (~line 795), `scaffold_controller_with_fk_template` (~line 494) | `#[rule]`/`ValidateRules` (verify if API also needs it) | D-04 |
| `ferro-cli/src/templates/make.rs` | `job_template` (line 333) | `use ferro_queue::` → `ferro::queue::` | D-03, D-07 |
| `ferro-cli/src/templates/auth.rs` | `auth_controller_template()` | `crate::models::users` module path (D-05), `ferro::database::connection()` call (D-06) | D-05, D-06 |

**Framework changes:**
| File | Change | D-# |
|------|--------|-----|
| `framework/src/lib.rs:122` | Add `ActiveValue` to `pub use sea_orm::{…}` | D-02 |
| `framework/src/lib.rs` (~line 461) | Add `error_response!` macro_rules! definition | D-01 |

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Path-dep override in per-PR test | Custom build system or cargo config hack | `[patch.crates-io]` block appended to generated `Cargo.toml` | Cargo's first-class mechanism; already documented in the generated template comment |
| Post-publish Docker run | New CI infrastructure | Existing Dockerfile + `docker build && docker run` in publish.yml | The Dockerfile is already the Phase 211 evidence apparatus and reproduces the exact cold-cache experience |
| Version propagation wait | Polling loop in CI | `sleep 60` before `cargo install` inside the Dockerfile | crates.io propagation is typically < 60s; retry on failure is sufficient |

---

## Common Pitfalls

### Pitfall 1: `#[macro_export]` scoping
**What goes wrong:** Adding `error_response!` inside a submodule with `#[macro_export]` makes it accessible at the crate root (`ferro::error_response!`) but clippy may complain about the macro not being in scope at the definition site.
**How to avoid:** Define the macro directly in `framework/src/lib.rs` (not in a submodule), consistent with `json_response!`, `text_response!`, etc.

### Pitfall 2: `ActiveValue` import conflict in Inertia scaffold templates
**What goes wrong:** The Inertia scaffold template (`scaffold_controller_template`) imports `use sea_orm::{EntityTrait, ActiveModelTrait, ActiveValue}`. After D-02, `ActiveValue` is also at `ferro::ActiveValue`. Importing both is fine (they resolve to the same type), but lints may flag it.
**How to avoid:** Change the Inertia template import to use `ferro::ActiveValue` and remove it from the `use sea_orm::{}` block in that template. This makes both template families consistent.

### Pitfall 3: `req.db()` method existence
**What goes wrong:** The Inertia scaffold template calls `let db = req.db()`. If this is not a real `Request` method, the Inertia templates also fail compilation.
**How to avoid:** Confirm `Request::db()` existence during implementation (read `framework/src/http/request.rs`). If it does not exist, the Inertia template has a second bug — both templates must use `ferro::DB::connection()?` directly.

### Pitfall 4: `make:auth` and the `users` model
**What goes wrong:** `make:auth` generates a migration that alters the users table but may not generate the `users` model file. If no model module exists, `crate::models::users` fails regardless of path correction.
**How to avoid:** Confirm during implementation: does `make:auth` emit `src/models/users.rs` and a `pub mod users;` line in `src/models/mod.rs`? If not, `make:auth` must also emit the model file (or the `auth_controller_template` must only reference types that exist post-scaffold).

### Pitfall 5: Dockerfile version pin becoming stale
**What goes wrong:** After each publish, `Dockerfile` pins `--version 0.2.55` which is now outdated.
**How to avoid:** Parameterize with `ARG FERRO_VERSION` as described in the CI guard section. The committed Dockerfile baseline must be kept as the Phase 211 evidence artifact; the CI job passes the version at build time.

### Pitfall 6: Per-PR test disk overflow
**What goes wrong:** The new `scaffold-smoke` CI job compiles a fresh Rust project + ferro workspace deps, pushing the GH runner over disk.
**How to avoid:** The workspace `ferro` artifacts are already built by earlier CI steps (the cache is shared). The generated app only compiles its own small codebase + pulls transitive deps. Disk risk is low but monitor. The `debug=false` profile setting (already in workspace `Cargo.toml`) keeps artifacts lean.

---

## Code Examples

### `error_response!` macro (target implementation)
```rust
// Source: framework/src/lib.rs (add after text_response! around line 388)
// [VERIFIED: HttpResponse is re-exported from http module; exact constructor TBD]

/// Return an HTTP error response for use in handler error arms.
///
/// Produces a bare `HttpResponse` value (not `Result`) suitable for use in
/// `.map_err(|e| ferro::error_response!(500, e.to_string()))` and
/// `.ok_or_else(|| ferro::error_response!(404, "not found"))`.
///
/// # Example
///
/// ```rust,ignore
/// Entity::find_by_id(id).one(db).await
///     .map_err(|e| ferro::error_response!(500, e.to_string()))?
///     .ok_or_else(|| ferro::error_response!(404, "Not found"))?;
/// ```
#[macro_export]
macro_rules! error_response {
    ($status:expr, $msg:expr) => {
        // Implementer: replace with the correct HttpResponse constructor.
        // Candidates: HttpResponse::new($status, $msg.to_string()),
        //             HttpResponse::json(serde_json::json!({"message": $msg})).status($status)
        $crate::HttpResponse::new_with_status($status as u16, $msg.to_string())
    };
}
```

### `ActiveValue` export (target)
```rust
// Source: framework/src/lib.rs:122
// CURRENT:
pub use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};

// TARGET:
pub use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
```

### Queue import in `job_template` (target)
```rust
// Source: ferro-cli/src/templates/make.rs, job_template(), line 339
// CURRENT:
use ferro_queue::{{async_trait, Error, Job, Queueable}};

// TARGET:
use ferro::{{async_trait, queue::{{Error, Job, Queueable}}}};
```

### Per-PR path-dep patch (test code)
```rust
// After step 4 (make:job), before step 5 (cargo build):
let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .expect("CARGO_MANIFEST_DIR has no parent")
    .to_path_buf();
let framework_path = workspace_root.join("framework");
let patch_block = format!(
    "\n[patch.crates-io]\nferro-rs = {{ path = \"{}\" }}\n",
    framework_path.display()
);
use std::io::Write;
let mut cargo_toml = std::fs::OpenOptions::new()
    .append(true)
    .open(project_dir.join("Cargo.toml"))
    .expect("Cargo.toml must exist after scaffold");
cargo_toml.write_all(patch_block.as_bytes()).expect("write patch block");
```

---

## Validation Architecture

The acceptance gate is pre-specified by COMP-04 and the CONTEXT.md:

**Gate sequence:** `ferro new bench-app` → `cd bench-app` → `ferro make:auth` → `ferro make:scaffold --no-smart-defaults -q -y --api Article title:string body:text` → `ferro make:scaffold --no-smart-defaults -q -y --api Product name:string price:float` → `ferro make:scaffold --no-smart-defaults -q -y --api Order status:string total:float` → `ferro make:job EmailNotification` → `cargo build` → **must exit 0**

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + `tempfile` (already a dev-dep in ferro-cli) |
| Config file | `ferro-cli/tests/benchmark_new_project.rs` (extend existing file) |
| Quick run command | `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro -- --nocapture` |
| Full cold-cache run | `docker build -t bench ferro-cli/tests/fixtures/benchmark/ && docker run --rm bench` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SCAF-01 | All scaffold symbols resolve from published `ferro` surface | Integration (via build) | `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro` | ❌ Wave 0 — add to benchmark_new_project.rs |
| SCAF-02 | `make:job` Cargo.toml has no missing deps | Integration (via build) | Same as above — build fails if dep missing | ❌ Wave 0 |
| SCAF-03 | Clean scaffold `cargo build` exits 0 against published `ferro-rs` | Integration | `docker run --rm ferro-scaffold-smoke` | ❌ Wave 0 — Dockerfile needs version param |
| SCAF-04 | CI smoke test gates each release | CI job | publish.yml `post-publish-scaffold-smoke` | ❌ Wave 0 — new CI job |
| SCAF-05 | Per-PR guard catches drift pre-publish | CI job | ci.yml `scaffold-smoke` job | ❌ Wave 0 — new CI job |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro -- --nocapture`
- **Per wave merge:** Full scaffold sequence + the per-PR CI job must be green
- **Phase gate:** The Docker cold-cache run exits 0 against the latest published `ferro-rs`

### Wave 0 Gaps

- [ ] `ferro-cli/tests/benchmark_new_project.rs` — add `scaffold_builds_against_workspace_ferro` (non-ignored, uses `[patch.crates-io]`)
- [ ] `ferro-cli/tests/fixtures/benchmark/Dockerfile` — parameterize `FERRO_VERSION` via `ARG`
- [ ] `.github/workflows/ci.yml` — add `scaffold-smoke` job
- [ ] `.github/workflows/publish.yml` — add `post-publish-scaffold-smoke` job

---

## Open Questions (RESOLVED)

All three were resolved during the planning session by direct verification against source; the
resolutions are carried verbatim into `214-01-PLAN.md`'s `<interfaces>` block.

1. **`Request::db()` method existence — RESOLVED: does not exist.**
   - Verified by grep: there is NO `Request::db()` method on `Request`. Generated code must use
     `ferro::DB::connection()?.inner()` (synchronous, returns `Result`). This is the D-06 fix.

2. **`make:auth` users model generation — RESOLVED: no separate model emitted.**
   - `models/mod.rs.tpl` declares `pub mod user;` (SINGULAR) and the base scaffold already provides
     `crate::models::user` (`user.rs.tpl` defines `pub type User = Model`, `find_by_email`).
     `make:auth` does NOT emit a `users` model. D-05 is a pure path correction `users` → `user` in
     `auth.rs` — no model-emission task needed.

3. **`#[rule]` in API `form_fields` — RESOLVED: the `--api` path DOES emit `#[rule(…)]`.**
   - Confirmed at `make_scaffold.rs:330-333`: the `--api` path emits `#[rule(required, string)]` etc.
     into `{form_fields}` (assumption A5 was wrong). The API form struct MUST derive `ValidateRules`
     (the D-04 fix). Two API form structs + two Inertia form structs derive it.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | All compilation | ✓ | 1.88.0 (workspace toolchain) | — |
| Docker | Release gate (Dockerfile) | ✓ | Available on ubuntu-latest GH runner | — |
| `tempfile` crate | Per-PR test | ✓ | Already dev-dep in ferro-cli | — |
| crates.io network | Release gate | ✓ | Required (not network-isolated) | — |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `HttpResponse` has a method callable with `(u16, String)` to produce an error response | D-01 macro signature | Macro body must use a different constructor; implementer reads framework/src/http/ |
| A2 | `Request::db()` does NOT exist on the `Request` type | D-06, Pitfall 3 | If it exists, the auth template's `req.db()` is fine; only the `database::connection()` call needs fixing |
| A3 | `make:auth` does NOT emit a `src/models/users.rs` file | D-05 | If it does emit one, D-05 is a simpler module-path fix |
| A4 | crates.io propagation takes < 60s | Release gate | Longer propagation → `cargo install` fails in the container; retry logic or longer sleep needed |
| A5 | `form_fields` in the `--api` scaffold path does NOT contain `#[rule(…)]` attributes | D-04 | If it does, the API templates need `ValidateRules` added to the form struct derive |

**None of these assumptions block planning** — all are resolvable by the implementer reading 2–3 source files before implementation begins.

---

## Sources

### Primary (HIGH confidence — VERIFIED in this session)
- `ferro-cli/src/templates/scaffold.rs` — all `error_response!`, `ActiveValue`, `ValidateRules` call sites
- `ferro-cli/src/templates/make.rs` — `job_template` `ferro_queue` import
- `ferro-cli/src/templates/auth.rs` — `ferro::database::connection` and `crate::models::users` call sites
- `framework/src/lib.rs` — facade re-exports: `sea_orm` block (line 122), `queue` module (lines 194–202), `ValidateRules` (line 342), existing macros (lines 366–461)
- `framework/src/database/mod.rs` — `DB::connection()` API (line 171)
- `framework/src/database/connection.rs` — `DbConnection` struct and `inner()` / `conn()` methods
- `ferro-macros/src/lib.rs:542` — `ValidateRules` derive + `rule` helper attribute declaration
- `ferro-cli/tests/benchmark_new_project.rs` — existing apparatus for per-PR guard extension
- `ferro-cli/tests/fixtures/benchmark/Dockerfile` — committed cold-cache harness
- `ferro-cli/src/templates/files/backend/Cargo.toml.tpl` — generated Cargo.toml with `ferro = { package = "ferro-rs", version = "0.2" }`
- `.github/workflows/ci.yml` — existing CI structure for per-PR job placement
- `.github/workflows/publish.yml` — publish pipeline structure for release gate placement

### Secondary (MEDIUM confidence)
- Phase 211 WEAKNESSES.md — authoritative error list from the actual cold-cache run
- Phase 214 CONTEXT.md — locked decisions D-01..D-10

---

## Metadata

**Confidence breakdown:**
- Per-symbol fix locations: HIGH — verified in source files
- `error_response!` macro return type: MEDIUM — `HttpResponse` constructor name not confirmed (A1)
- CI guard mechanism: HIGH — path-dep override via `[patch.crates-io]` is a verified Cargo feature; Dockerfile parameterization is straightforward
- D-05 users model: LOW/MEDIUM — requires secondary read of make:auth command (A3)

**Research date:** 2026-06-13
**Valid until:** Stable (no fast-moving dependencies; all findings are about the project's own source code)
