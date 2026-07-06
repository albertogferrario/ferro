# Phase 156: frontend/src/types/ — Generator-Owned Convention Cleanup - Pattern Map

**Mapped:** 2026-05-14
**Files analyzed:** 10 new/modified files
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-cli/src/doctor/checks/frontend_types_convention.rs` | check (new file) | file-I/O | `ferro-cli/src/doctor/checks/generated_artifacts.rs` | exact |
| `ferro-cli/src/doctor/checks/mod.rs` | module registry (modify) | — | self (current state) | exact |
| `ferro-cli/src/doctor/registry.rs` | registry (modify) | — | self (current state) | exact |
| `ferro-cli/src/doctor/check.rs` | framework (modify) | — | self (current state) | exact |
| `ferro-cli/src/templates/docker.rs` | renderer (modify) | transform | self (current state) | exact |
| `ferro-cli/src/commands/docker_init.rs` | command (modify) | file-I/O | self (current state) | exact |
| `ferro-cli/src/commands/generate_types.rs` | command (modify — comment only) | — | self (current state) | exact |
| `ferro-cli/src/templates/files/root/gitignore.tpl` | template (modify — comment only) | — | self (current state) | exact |
| `docs/src/cli/frontend-types.md` | documentation (new file) | — | `docs/src/cli/doctor.md`, `docs/src/cli/do-init.md` | role-match |
| `docs/src/SUMMARY.md` | documentation index (modify) | — | self (current state) | exact |
| `ferro-cli/src/templates/files/root/README.md.tpl` | scaffold template (modify) | — | self (current state) | exact |

---

## Pattern Assignments

### `ferro-cli/src/doctor/checks/frontend_types_convention.rs` (check, file-I/O)

**Analog:** `ferro-cli/src/doctor/checks/generated_artifacts.rs`

**Imports pattern** (lines 1–4):
```rust
use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;
```

**Struct + NAME constant pattern** (lines 7–10):
```rust
pub struct GeneratedArtifactsCheck;

const NAME: &str = "generated_artifacts";
const ARTIFACTS: &[&str] = &["Dockerfile", ".dockerignore", ".do/app.yaml"];
```
For this check, mirror exactly:
```rust
pub struct FrontendTypesConventionCheck;

const NAME: &str = "frontend_types_convention";
// Files the generator writes (confirmed: generate_types.rs lines 882, 922).
// If the generator is extended, keep this list in sync.
const GENERATED_ALLOWLIST: &[&str] = &["inertia-props.ts", "routes.ts"];
```

**DoctorCheck impl pattern** (lines 12–19) — copy verbatim, no `category()` override (defaults to `General`):
```rust
impl DoctorCheck for GeneratedArtifactsCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}
```

**Core check_impl pattern** (lines 21–33) — the `generated_artifacts` check scans for missing files; the new check scans for unexpected files:
```rust
pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let missing: Vec<&str> = ARTIFACTS
        .iter()
        .filter(|f| !root.join(f).exists())
        .copied()
        .collect();
    if missing.is_empty() {
        CheckResult::ok(NAME, "Dockerfile, .dockerignore, .do/app.yaml present")
    } else {
        CheckResult::warn(NAME, format!("{} artifact(s) missing", missing.len()))
            .with_details(format!("missing: {}", missing.join(", ")))
    }
}
```
For the new check, the logic is inverted — collect files NOT in the allowlist:
```rust
pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let types_dir = root.join("frontend/src/types");
    if !types_dir.is_dir() {
        return CheckResult::ok(NAME, "frontend/src/types absent (clean)");
    }
    let hand_written: Vec<String> = std::fs::read_dir(&types_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if GENERATED_ALLOWLIST.contains(&name.as_str()) { None } else { Some(name) }
        })
        .collect();
    if hand_written.is_empty() {
        CheckResult::ok(NAME, "frontend/src/types contains only generator-owned files")
    } else {
        CheckResult::warn(NAME, format!("{} hand-written file(s) in frontend/src/types/", hand_written.len()))
            .with_details(format!(
                "move to frontend/src/lib/types/: {}",
                hand_written.join(", ")
            ))
    }
}
```

**Test pattern** (lines 36–63 in generated_artifacts.rs) — use `TempDir`, call `check_impl` directly:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn name_is_generated_artifacts() {
        assert_eq!(GeneratedArtifactsCheck.name(), "generated_artifacts");
    }

    #[test]
    fn all_present_returns_ok() {
        let tmp = TempDir::new().unwrap();
        // create expected files ...
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }

    #[test]
    fn missing_warns_never_errors() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Warn);
    }
}
```
Required test cases for the new check: `absent_directory_is_ok`, `only_generated_files_is_ok`, `hand_written_file_warns`, `mixed_generated_and_hand_written_warns_on_hand_written_only`. See RESEARCH.md Code Examples §1 for full implementations.

---

### `ferro-cli/src/doctor/checks/mod.rs` (module registry, modify)

**Analog:** self — `ferro-cli/src/doctor/checks/mod.rs`

**Current pattern** (lines 1–24):
```rust
pub mod copy_dirs_dockerignore_collision;
// ... (10 pub mod declarations, alphabetical by module name)

pub use copy_dirs_dockerignore_collision::CopyDirsDockerignoreCollisionCheck;
// ... (10 pub use re-exports, alphabetical)
```

**Addition pattern** — insert one `pub mod` and one `pub use` line following alphabetical order:
```rust
pub mod frontend_types_convention;          // NEW — between generated_artifacts and local_env_parity
pub use frontend_types_convention::FrontendTypesConventionCheck;  // NEW
```

---

### `ferro-cli/src/doctor/registry.rs` (registry, modify)

**Analog:** self — `ferro-cli/src/doctor/registry.rs`

**Current import block** (lines 4–8):
```rust
use super::checks::{
    CopyDirsDockerignoreCollisionCheck, DatabaseUrlSqliteInProdCheck, DbConnectionCheck,
    DeployEnvParityCheck, DirtyGitTreeCheck, DockerTemplateDriftCheck, GeneratedArtifactsCheck,
    LocalEnvParityCheck, MigrationsCheck, ToolchainCheck,
};
```
Add `FrontendTypesConventionCheck` to this import list.

**default_checks() pattern** (lines 15–28) — append `Box::new(FrontendTypesConventionCheck)` at the end:
```rust
pub fn default_checks() -> Vec<Box<dyn DoctorCheck>> {
    vec![
        Box::new(ToolchainCheck),
        // ... existing 10 ...
        Box::new(DirtyGitTreeCheck),
        Box::new(FrontendTypesConventionCheck),  // NEW — check #11
    ]
}
```

**Test update** (lines 35–54) — update count assertion and names list:
```rust
#[test]
fn default_checks_returns_ten_in_declared_order() {  // rename to _eleven_
    let checks = default_checks();
    assert_eq!(checks.len(), 11);  // was 10
    let names: Vec<&'static str> = checks.iter().map(|c| c.name()).collect();
    assert_eq!(
        names,
        vec![
            "toolchain_match",
            // ... existing 10 ...
            "git_clean_and_pushed",
            "frontend_types_convention",  // NEW
        ]
    );
}
```

---

### `ferro-cli/src/doctor/check.rs` (framework, modify)

**Analog:** self — `ferro-cli/src/doctor/check.rs`

**Test that lists general_names** (lines 225–234) — add `"frontend_types_convention"` to the `general_names` slice:
```rust
let general_names = &[
    "toolchain_match",
    "db_connection",
    "migrations_pending",
    "local_env_parity",
    "deploy_env_parity",
    "generated_artifacts",
    "database_url_sqlite_in_prod",
    "git_clean_and_pushed",
    "frontend_types_convention",  // NEW
];
```

---

### `ferro-cli/src/templates/docker.rs` (renderer, modify)

**Analog:** self — `ferro-cli/src/templates/docker.rs`

**DockerContext struct extension** (lines 33–53) — add one field:
```rust
pub struct DockerContext {
    pub rust_channel: String,
    pub has_frontend: bool,
    pub bins: Vec<String>,
    pub web_bin: String,
    pub copy_dirs_present: Vec<String>,
    pub runtime_apt: Vec<String>,
    /// Resolved ferro-cli version for the types-gen stage `cargo install` pin.
    /// Resolved by the caller from Cargo.lock (`ferro-rs` package) or
    /// `env!("CARGO_PKG_VERSION")` as fallback. Never empty.
    pub ferro_version: String,
}
```

**New stage constant pattern** — mirrors `FRONTEND_STAGE_BODY` (lines 117–124):
```rust
const FRONTEND_STAGE_BODY: &str = r#"
FROM node:20-bookworm-slim AS frontend-builder
WORKDIR /frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci || npm install
COPY frontend/ ./
RUN npm run build
"#;
```
New constants following the same pattern:
```rust
const TYPES_GEN_STAGE_BODY: &str = r#"
FROM rust:{{RUST_IMAGE_TAG}} AS types-gen
WORKDIR /app
RUN cargo install ferro-cli --version {{FERRO_VERSION}} --locked
COPY . .
RUN ferro generate-types
"#;

// Replaces FRONTEND_STAGE_BODY when has_frontend == true.
// The COPY --from=types-gen line appears immediately before RUN npm run build.
const FRONTEND_STAGE_WITH_TYPES_COPY_BODY: &str = r#"
FROM node:20-bookworm-slim AS frontend-builder
WORKDIR /frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci || npm install
COPY frontend/ ./
COPY --from=types-gen /app/frontend/src/types ./src/types
RUN npm run build
"#;
```

**render_dockerfile token replacement chain** (lines 102–114) — add `{{FERRO_VERSION}}` to the chained `.replace()` calls:
```rust
let rendered = DOCKERFILE_TPL
    .replace("{{FRONTEND_STAGE}}", &frontend_stage)
    .replace("{{RUST_IMAGE_TAG}}", &rust_image_tag)
    .replace("{{FERRO_VERSION}}", &ctx.ferro_version)   // NEW
    .replace("{{ENTRYPOINT}}", &entrypoint_block)
    .replace("{{BIN_COPIES}}", &bin_copies)
    .replace("{{COPY_DIRS}}", &copy_dirs)
    .replace("{{RUNTIME_APT}}", &runtime_apt);

debug_assert!(
    !rendered.contains("{{"),
    "unresolved template token in rendered Dockerfile:\n{rendered}"
);
```

**frontend_stage composition** (lines 57–61) — replace the simple string with concatenated stages:
```rust
let frontend_stage = if ctx.has_frontend {
    format!("{TYPES_GEN_STAGE_BODY}{FRONTEND_STAGE_WITH_TYPES_COPY_BODY}")
} else {
    String::new()
};
```

**Test additions** — mirror the existing test pattern (lines 197–203):
```rust
#[test]
fn frontend_stage_present_only_when_has_frontend() {
    let mut c = ctx();
    c.has_frontend = false;
    assert!(!render_dockerfile(&c).contains("frontend-builder"));
    c.has_frontend = true;
    assert!(render_dockerfile(&c).contains("frontend-builder"));
}
```
New tests required:
- `types_gen_stage_present_when_has_frontend` — `has_frontend=true` → output contains `AS types-gen`
- `types_gen_stage_absent_when_no_frontend` — `has_frontend=false` → output does NOT contain `AS types-gen`
- `copy_from_types_gen_before_npm_build` — `has_frontend=true` → `COPY --from=types-gen` appears before `RUN npm run build`
- `no_unresolved_tokens_with_frontend` — extend existing `no_unresolved_tokens_in_dockerfile` with `has_frontend=true` variant

**ctx() helper update** — add `ferro_version` field to the test helper:
```rust
fn ctx() -> DockerContext {
    DockerContext {
        rust_channel: "stable".to_string(),
        has_frontend: false,
        bins: vec!["app".to_string()],
        web_bin: "app".to_string(),
        copy_dirs_present: vec![],
        runtime_apt: vec![],
        ferro_version: "0.2.33".to_string(),  // NEW
    }
}
```

---

### `ferro-cli/src/commands/docker_init.rs` (command, modify)

**Analog:** self — `ferro-cli/src/commands/docker_init.rs`

**Current DockerContext construction** (lines 73–80):
```rust
let ctx = DockerContext {
    rust_channel,
    has_frontend,
    bins,
    web_bin,
    copy_dirs_present,
    runtime_apt: metadata.runtime_apt.clone(),
};
```
After change:
```rust
let ferro_version = resolve_ferro_version(&root);
let ctx = DockerContext {
    rust_channel,
    has_frontend,
    bins,
    web_bin,
    copy_dirs_present,
    runtime_apt: metadata.runtime_apt.clone(),
    ferro_version,  // NEW
};
```

**resolve_ferro_version helper** — new private function using the `toml` crate (already in `ferro-cli/Cargo.toml`):
```rust
fn resolve_ferro_version(root: &Path) -> String {
    // 1. Try Cargo.lock — matches what the project compiles against.
    if let Ok(lock) = std::fs::read_to_string(root.join("Cargo.lock")) {
        if let Ok(parsed) = lock.parse::<toml::Value>() {
            let pkgs = parsed.get("package").and_then(|v| v.as_array());
            if let Some(pkgs) = pkgs {
                for pkg in pkgs {
                    let name = pkg.get("name").and_then(|n| n.as_str());
                    let ver = pkg.get("version").and_then(|v| v.as_str());
                    if name == Some("ferro-rs") {
                        if let Some(v) = ver {
                            return v.to_string();
                        }
                    }
                }
            }
        }
    }
    // 2. Fallback: the running binary's own version.
    // Used when Cargo.lock is absent or has no ferro-rs entry (rare).
    env!("CARGO_PKG_VERSION").to_string()
}
```
Note: `_ferro_version_flag: Option<&str>` parameter already exists at line 55 (currently unused, underscore-prefixed). The `resolve_ferro_version` replaces this stub — the parameter can be kept for forward-compat or removed.

---

### `ferro-cli/src/doctor/checks/docker_template_drift.rs` (check, modify)

**Analog:** self — current file, specifically lines 64–71 (DockerContext construction)

**Current DockerContext construction** (lines 64–71):
```rust
let ctx = DockerContext {
    rust_channel: read_rust_channel(root),
    has_frontend: root.join("frontend/package.json").is_file(),
    bins,
    web_bin,
    copy_dirs_present,
    runtime_apt: metadata.runtime_apt,
};
```
After adding `ferro_version` field to `DockerContext`, this struct literal becomes a compile error. Must add the field at the same time as docker.rs changes:
```rust
let ctx = DockerContext {
    rust_channel: read_rust_channel(root),
    has_frontend: root.join("frontend/package.json").is_file(),
    bins,
    web_bin,
    copy_dirs_present,
    runtime_apt: metadata.runtime_apt,
    ferro_version: resolve_ferro_version(root),  // NEW — same helper as docker_init.rs
};
```
The `resolve_ferro_version` function should be placed in a shared location (e.g., `ferro-cli/src/templates/docker.rs` as a `pub(crate)` function) so both `docker_init.rs` and `docker_template_drift.rs` can call it without duplication.

**Test fixtures update** (lines 131–139 and 163–171) — all `DockerContext { ... }` struct literals in tests must also gain `ferro_version`:
```rust
let ctx = DockerContext {
    rust_channel: "stable".to_string(),
    has_frontend: false,
    bins: vec!["sample".to_string()],
    web_bin: "sample".to_string(),
    copy_dirs_present: vec![],
    runtime_apt: vec![],
    ferro_version: "0.0.0".to_string(),  // NEW — value irrelevant for drift tests
};
```

---

### `ferro-cli/src/commands/generate_types.rs` (command, comment fix)

**Analog:** self — lines 710–711

**Current text** (lines 710–711):
```rust
output.push_str("// For custom types not generated here, create manual type files in:\n");
output.push_str("// frontend/src/types/\n");
```

**Fixed text** (D-18):
```rust
output.push_str("// For custom types not generated here, create manual type files in:\n");
output.push_str("// frontend/src/lib/types/\n");
```

---

### `ferro-cli/src/templates/files/root/gitignore.tpl` (template, comment fix)

**Analog:** self — lines 14–15

**Current text** (lines 14–15):
```
# generated_types
frontend/src/types/
```

**Fixed text** (D-06) — strengthen comment to indicate load-bearing status:
```
# generated_types — load-bearing: frontend/src/types/ is owned by `ferro generate-types`.
# Removing this rule breaks the generator-owned convention (see docs/src/cli/frontend-types.md).
frontend/src/types/
```

---

### `docs/src/cli/frontend-types.md` (documentation, new file)

**Analog:** `docs/src/cli/do-init.md` and `docs/src/cli/doctor.md`

**Document structure pattern** (from do-init.md and doctor.md):
- H1 title matching command/feature name
- Short one-sentence summary after H1
- `## Usage` with fenced bash block
- `## What it produces` / `## Checks` explaining the core content
- `## Status semantics` or similar reference tables
- `## Examples` with fenced bash blocks
- `## Related commands` with links to sibling docs

**Content** (per D-08):
- What `ferro generate-types` produces (two files: `inertia-props.ts`, `routes.ts`)
- Why `frontend/src/types/` is gitignored (generator-owned convention)
- Where hand-written types belong (`frontend/src/lib/types/`)
- Bootstrap sequence on fresh clone: "run `cargo run` once before `npm run dev`"
- Troubleshooting "missing types" errors (TypeScript TS2307 on fresh clone)
- Docker production builds: consumers of older scaffolds must run `ferro docker:init --force`

---

### `docs/src/SUMMARY.md` (documentation index, modify)

**Analog:** self — lines 62–66 (Reference / CLI section)

**Current CLI sub-entries** (lines 63–66):
```markdown
  - [do:init](cli/do-init.md)
  - [ci:init](cli/ci-init.md)
  - [doctor](cli/doctor.md)
  - [routes:json-schema](cli/routes-json-schema.md)
```

**Addition** — insert after `[doctor]`:
```markdown
  - [frontend-types](cli/frontend-types.md)
```

---

### `ferro-cli/src/templates/files/root/README.md.tpl` (scaffold template, modify)

**Analog:** self — lines 82–86 (Troubleshooting section)

**Current troubleshooting entries** (lines 82–86):
```markdown
- **`ferro: command not found`** — install with `cargo install ferro-cli`.
- **Migrations fail** — delete `database.db` and run `ferro db:fresh`.
- **Frontend assets missing** — run `npm install` inside `frontend/`, then restart `ferro serve`.
- **Port 8080 in use** — change `SERVER_PORT` in `.env`.
```

**Addition** — insert a new bullet for types bootstrap (D-11):
```markdown
- **TypeScript errors about `Cannot find module './types/inertia-props'`** — run `cargo run` once to generate types before running `npm run dev`. Types are regenerated automatically on each server start.
```

---

### `docs/src/cli/doctor.md` (documentation, modify)

**Analog:** self — line 3 and the checks table (lines 23–33)

**Line 3 current:**
```
Single-command project health diagnostics. Runs nine checks in declared
```

**Line 3 after:**
```
Single-command project health diagnostics. Runs ten checks in declared
```

**Checks table addition** — append row #10 (current last is #9 `git_clean_and_pushed`):

| # | Name | Category | Purpose |
|---|---|---|---|
| 10 | `docker_template_drift` | Deploy | Dockerfile matches scaffolder output |
| 11 | `frontend_types_convention` | General | No hand-written files in `frontend/src/types/` |

Note: the table must also be checked for the current row numbering since the research notes the table shows 9 but registry has 10 checks — verify and reconcile during implementation.

**`checks[].name` reference list** (line 94) — add `"frontend_types_convention"` to the list of nine stable identifiers.

---

## Shared Patterns

### CheckResult construction
**Source:** `ferro-cli/src/doctor/check.rs` lines 36–68
**Apply to:** `frontend_types_convention.rs`
```rust
// ok — no details needed
CheckResult::ok(NAME, "short message")

// warn with actionable details
CheckResult::warn(NAME, "short message")
    .with_details("actionable suggestion with file names")
```

### TempDir test setup
**Source:** `ferro-cli/src/doctor/checks/generated_artifacts.rs` lines 35–63
**Apply to:** `frontend_types_convention.rs` tests and updated `docker_template_drift.rs` tests
```rust
use std::fs;
use tempfile::TempDir;

let tmp = TempDir::new().unwrap();
fs::create_dir_all(tmp.path().join("some/nested/dir")).unwrap();
fs::write(tmp.path().join("some/file.ts"), "").unwrap();
let r = check_impl(tmp.path());
assert_eq!(r.status, crate::doctor::check::CheckStatus::Warn);
```

### Template token replacement chain
**Source:** `ferro-cli/src/templates/docker.rs` lines 102–114
**Apply to:** the extended `render_dockerfile` function
```rust
let rendered = DOCKERFILE_TPL
    .replace("{{TOKEN1}}", &value1)
    .replace("{{TOKEN2}}", &value2)
    // ...
    ;
debug_assert!(
    !rendered.contains("{{"),
    "unresolved template token in rendered Dockerfile:\n{rendered}"
);
```
The `debug_assert!` automatically catches any missed `{{FERRO_VERSION}}` substitution in debug builds. The existing `no_unresolved_tokens_in_dockerfile` test must be extended with a `has_frontend=true` variant to catch it in test runs.

### Cargo.lock TOML parsing
**Source:** `ferro-cli/Cargo.toml` (`toml = "0.8"` confirmed)
**Apply to:** `resolve_ferro_version` in `docker_init.rs` (and shared via `docker.rs` or a helper module)
```rust
let lock = std::fs::read_to_string(root.join("Cargo.lock"))?;
let parsed = lock.parse::<toml::Value>()?;
let pkgs = parsed.get("package").and_then(|v| v.as_array());
// iterate pkgs to find name == "ferro-rs"
```

---

## No Analog Found

All files in this phase have close analogs in the codebase. No entries.

---

## Metadata

**Analog search scope:** `ferro-cli/src/doctor/`, `ferro-cli/src/templates/`, `ferro-cli/src/commands/`, `docs/src/cli/`
**Files scanned:** 13 source files
**Pattern extraction date:** 2026-05-14
