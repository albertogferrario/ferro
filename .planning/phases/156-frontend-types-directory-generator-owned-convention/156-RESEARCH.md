# Phase 156: frontend/src/types/ — Generator-Owned Convention Cleanup - Research

**Researched:** 2026-05-14
**Domain:** ferro-cli doctor checks, Dockerfile renderer, generate_types generator, docs
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Convention direction**
- D-01: Ignore generator output. Do not track it.
- D-02: `frontend/src/types/` is reserved for `ferro generate-types` output only.
- D-03: Hand-written types live in `frontend/src/lib/types/`.
- D-04: No determinism work in this phase.

**Reference app cleanup**
- D-05: `git rm --cached app/frontend/src/types/{inertia-props,routes}.ts` — untrack only.
- D-06: Add load-bearing-comment annotation to `gitignore.tpl:14`.
- D-07: No edits to existing scaffolded projects' gitignore.

**Documentation**
- D-08: One new docs page. Title: "Frontend types: generator-owned convention."
- D-10: `ferro doctor` is opt-in for v0. Does not run automatically.
- D-11: Document bootstrap sequence in scaffold README template.
- D-12: No frontend `package.json` `prebuild`/`predev` hook.
- D-19: No `ferro setup` command. Docs say "run `cargo run` once."

**Doctor check**
- D-09: `ferro doctor` flags hand-written files under `frontend/src/types/` at WARNING severity.
- D-20: `FrontendTypesConventionCheck` uses explicit allowlist (`inertia-props.ts`, `routes.ts`).

**Versioning**
- D-13: Cascade workspace version bump. No new crate.
- D-14: No `ferro-cli` API breaking change.

**Dockerfile reconciliation**
- D-15: Add a `types-gen` Rust stage to the Dockerfile renderer (unconditional when `has_frontend == true`).
- D-16: Pin `ferro-cli` version in the rendered Dockerfile via `{{FERRO_VERSION}}` token.
- D-17: `types-gen` stage only affects `frontend-builder` via `COPY --from=types-gen`. Backend build chain unchanged.
- D-21: `FERRO_VERSION` source: parse from project's `Cargo.lock` (package `ferro-rs`). Fallback: `env!("CARGO_PKG_VERSION")`.

**Generator header fix**
- D-18: Fix `generate_types.rs` lines 710-711 comment from `frontend/src/types/` to `frontend/src/lib/types/`.

### Claude's Discretion

- Exact docs page path (`docs/dx/frontend-types.md` vs another location) — verify during planning what the docs file naming convention is.
- Whether the `debug_assert!` in `render_dockerfile` for unresolved `{{` tokens needs a counterpart for `{{FERRO_VERSION}}` in the types-gen path.
- CI Docker build verification approach: if no existing CI Docker step exists, add a local verification recipe to the docs page rather than a new CI step, unless CI already has one.

### Deferred Ideas (OUT OF SCOPE)

- Making the type generator output deterministic
- CI drift-check (`ferro generate-types --check`)
- Migration scripts for consumer apps with hand-written types in `frontend/src/types/`
- `ferro setup` command
- Generating `parsed-menu.ts`-style domain types from Rust structs
- `ferro doctor` check detecting outdated rendered Dockerfile (no `types-gen` stage heuristic)
- Cheaper alternatives to `cargo install` in types-gen stage (cargo-binstall, prebuilt binary)
</user_constraints>

---

## Summary

Phase 156 closes a convention contradiction: `ferro-cli`'s scaffold gitignore marks `frontend/src/types/` as generator-owned, but the Ferro reference app (`app/frontend/src/types/`) tracks both generated files (`inertia-props.ts`, `routes.ts`) in git, and the scaffolded Dockerfile has no mechanism to regenerate them during a production build. This phase reconciles all three entry points — dev loop, fresh clone, Docker build — by adopting the ignore-the-output convention consistently.

Work spans four code sites (generate_types.rs header, gitignore.tpl comment, docker.rs renderer, docker_init.rs/docker_template_drift.rs call sites), one new doctor check, a new docs page, and two doc-site updates (SUMMARY.md, doctor.md check count). The reference app cleanup is a pure git operation (`git rm --cached`). No new crate is introduced; the workspace version bumps per existing convention.

The scope is well-defined by CONTEXT.md decisions; all key implementation files are confirmed via codebase scan. The work is additive in the doctor registry and Dockerfile renderer. The most complex piece is the `types-gen` Docker stage and the `FERRO_VERSION` resolution from `Cargo.lock`.

**Primary recommendation:** Sequence as: (1) trivial one-line fixes (header comment, gitignore comment, git rm), (2) doctor check (new file + registry + mod.rs + docs update), (3) Dockerfile renderer (DockerContext field, token, template, call site updates), (4) docs page, (5) version bump + CHANGELOG + publish.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Git untrack generated files | Developer tooling | — | `git rm --cached` is a one-time repo operation |
| Gitignore template comment | Scaffolder (ferro-cli) | — | Template is emitted at `ferro new` time |
| Doctor check for convention | CLI runtime (ferro doctor) | — | `DoctorCheck` trait in ferro-cli |
| Dockerfile types-gen stage | CLI renderer (ferro-cli) | Docker build runtime | Renderer generates; Docker executes |
| FERRO_VERSION resolution | CLI command layer (docker_init.rs, docker_template_drift.rs) | — | Caller-resolved per existing "pure render" boundary |
| Generator header fix | CLI generate_types command | — | Hardcoded string in generate_types.rs |
| Documentation | docs/src/ | Scaffold README template | mdBook source + tpl file |

---

## Standard Stack

This phase is purely internal to the ferro workspace. No new dependencies.

### Confirmed Existing Dependencies
| Crate | Already in ferro-cli/Cargo.toml | Use in This Phase |
|-------|--------------------------------|-------------------|
| `toml` 0.8 | Yes [VERIFIED: codebase] | Parse Cargo.lock to extract `ferro-rs` version |
| `toml_edit` 0.22 | Yes [VERIFIED: codebase] | Not needed for this phase |
| `tempfile` 3.24.0 (dev) | Yes [VERIFIED: codebase] | Doctor check tests use `TempDir` |
| `walkdir` 2 | Yes [VERIFIED: codebase] | Available for directory scanning in doctor check |

**No new dependencies required.** `Cargo.lock` parsing uses the existing `toml` crate.

---

## Architecture Patterns

### System Architecture Diagram

```
ferro generate-types
  └─► frontend/src/types/inertia-props.ts   (gitignored — never committed)
  └─► frontend/src/types/routes.ts          (gitignored — never committed)

Dev loop:
  cargo run  ──► generate-types runs ──► frontend/src/types/ populated on disk ──► npm run dev works

Fresh clone:
  git clone  ──► frontend/src/types/ absent
  cargo run  ──► types regenerated ──► frontend works

Docker build (AFTER this phase):
  COPY . .  ──► types/ absent (gitignored, not in build context)
  types-gen stage: cargo install ferro-cli + ferro generate-types  ──► types present in stage
  frontend-builder: COPY --from=types-gen /app/frontend/src/types ./src/types ──► tsc resolves
  npm run build  ──► success

ferro doctor:
  frontend_types_convention check:
    frontend/src/types/ absent  ──► ok (skip)
    contains only inertia-props.ts and/or routes.ts  ──► ok
    contains any other file  ──► warn (hand-written file found, move to frontend/src/lib/types/)
```

### Recommended Project Structure for New Files

```
ferro-cli/src/
├── doctor/
│   ├── checks/
│   │   ├── frontend_types_convention.rs   # NEW — FrontendTypesConventionCheck
│   │   └── mod.rs                         # +1 pub mod, +1 pub use
│   └── registry.rs                        # +1 import, +1 Box::new in default_checks()
└── templates/
    └── docker.rs                          # DockerContext.ferro_version field + TYPES_GEN_STAGE_BODY + token

ferro-cli/src/commands/
├── docker_init.rs                         # resolve ferro_version from Cargo.lock, pass to DockerContext
└── generate_types.rs                      # lines 710-711 comment fix

ferro-cli/src/templates/files/root/
└── gitignore.tpl                          # line 14 comment augment

docs/src/
├── SUMMARY.md                             # add entry under Reference section
├── cli/
│   ├── doctor.md                          # update check count + add new check row
│   └── frontend-types.md                  # NEW docs page
└── reference/
    └── cli.md                             # update generate-types section

app/frontend/src/types/                    # git rm --cached both files
```

### Pattern 1: Doctor Check (Canonical)

Every check follows the same three-part structure. `FrontendTypesConventionCheck` is no exception.

```rust
// Source: ferro-cli/src/doctor/checks/generated_artifacts.rs (existing check)
pub struct FrontendTypesConventionCheck;

const NAME: &str = "frontend_types_convention";

// These are the only two files the generator writes (confirmed from generate_types.rs lines 882, 922)
const GENERATED_ALLOWLIST: &[&str] = &["inertia-props.ts", "routes.ts"];

impl DoctorCheck for FrontendTypesConventionCheck {
    fn name(&self) -> &'static str { NAME }
    fn run(&self, root: &Path) -> CheckResult { check_impl(root) }
    // category() defaults to General — no override needed
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let types_dir = root.join("frontend/src/types");
    if !types_dir.is_dir() {
        return CheckResult::ok(NAME, "frontend/src/types absent (clean)");
    }
    // Collect entries that are NOT in the allowlist
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

[VERIFIED: codebase] — pattern matches `generated_artifacts.rs`, `toolchain.rs`, and `copy_dirs_dockerignore_collision.rs`.

### Pattern 2: DockerContext Extension + Token Resolution

`render_dockerfile` is a pure function; callers resolve all I/O. The existing pattern from `docker_init.rs` and `docker_template_drift.rs` must be extended identically at both call sites.

```rust
// Source: ferro-cli/src/templates/docker.rs — field addition
pub struct DockerContext {
    // ... existing fields ...
    /// Resolved ferro-cli version for the types-gen stage `cargo install` pin.
    /// Resolved by the caller from Cargo.lock (`ferro-rs` package) or
    /// `env!("CARGO_PKG_VERSION")` as fallback. Never empty.
    pub ferro_version: String,
}
```

```rust
// Source: ferro-cli/src/commands/docker_init.rs — resolution pattern
fn resolve_ferro_version(root: &Path) -> String {
    // 1. Try Cargo.lock — most accurate (matches what the project compiles against)
    if let Ok(lock) = std::fs::read_to_string(root.join("Cargo.lock")) {
        if let Ok(parsed) = lock.parse::<toml::Value>() {
            let pkgs = parsed.get("package").and_then(|v| v.as_array());
            if let Some(pkgs) = pkgs {
                for pkg in pkgs {
                    let name = pkg.get("name").and_then(|n| n.as_str());
                    let ver  = pkg.get("version").and_then(|v| v.as_str());
                    if name == Some("ferro-rs") {
                        if let Some(v) = ver { return v.to_string(); }
                    }
                }
            }
        }
    }
    // 2. Fallback: CLI binary's own version
    env!("CARGO_PKG_VERSION").to_string()
}
```

[VERIFIED: codebase] — `toml` crate is already a dependency. `ferro-rs` package name confirmed in `Cargo.lock` at version `0.2.33`.

### Pattern 3: Dockerfile Template Stage Addition

`FRONTEND_STAGE_BODY` is a `const &str` inlined into `render_dockerfile` when `has_frontend == true`. The new `types-gen` stage follows the same pattern. D-15 sketch from CONTEXT.md is the canonical form.

```rust
// Source: ferro-cli/src/templates/docker.rs
const TYPES_GEN_STAGE_BODY: &str = r#"
FROM rust:{{RUST_IMAGE_TAG}} AS types-gen
WORKDIR /app
RUN cargo install ferro-cli --version {{FERRO_VERSION}} --locked
COPY . .
RUN ferro generate-types
"#;
```

In `render_dockerfile`:

```rust
let frontend_stage = if ctx.has_frontend {
    format!("{TYPES_GEN_STAGE_BODY}{FRONTEND_STAGE_BODY_WITH_COPY}")
} else {
    String::new()
};
```

Where `FRONTEND_STAGE_BODY_WITH_COPY` differs from the current `FRONTEND_STAGE_BODY` only by adding:

```dockerfile
COPY --from=types-gen /app/frontend/src/types ./src/types
```

immediately before `RUN npm run build`.

The `.replace("{{FERRO_VERSION}}", &ctx.ferro_version)` call joins the existing chain of `.replace(...)` calls in `render_dockerfile`. The `debug_assert!` at line 110 that checks for unresolved `{{` tokens will automatically catch any missed substitution.

[VERIFIED: codebase] — current `render_dockerfile` already does chained `.replace()` for `{{RUST_IMAGE_TAG}}`, `{{FRONTEND_STAGE}}`, etc. The `debug_assert!` fires in debug builds only.

### Anti-Patterns to Avoid

- **Doing I/O in `render_dockerfile`:** The renderer is a pure function by design (Phase 127). `ferro_version` must be resolved in `docker_init.rs` and `docker_template_drift.rs` before calling `render_dockerfile`, not inside it.
- **Using `#[serde(rename_all)]` or anything on `DockerContext`:** It is a plain Rust struct, not serialized. Just add a field.
- **Adding the check to `CheckCategory::Deploy`:** This is a general developer-facing convention check, not a deploy-specific check. No `category()` override needed — defaults to `General`.
- **Checking the `types-gen` stage only for `has_frontend && has_docker`:** The renderer emits `types-gen` whenever `has_frontend == true`. The `docker_template_drift` check constructs the expected output from the same renderer, so it will auto-detect drift on the next `ferro doctor` run.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TOML parsing for Cargo.lock | Custom line-by-line parser | `toml` crate (already a dep) | Cargo.lock is valid TOML; the `toml` crate handles edge cases |
| Directory traversal for doctor check | `walkdir` | `std::fs::read_dir` (one level only) | The check only looks at the immediate children of `frontend/src/types/`, not recursively |
| Version fallback | Hardcoded constant | `env!("CARGO_PKG_VERSION")` | Compile-time macro always matches the binary's own version |

---

## Common Pitfalls

### Pitfall 1: `debug_assert!` Does Not Cover `{{FERRO_VERSION}}` in Release Builds

**What goes wrong:** `debug_assert!` fires only in debug mode. If `{{FERRO_VERSION}}` is accidentally left unreplaced in a release build, it silently reaches the emitted Dockerfile.

**Why it happens:** The existing `debug_assert!` at docker.rs line 110 is sufficient for local development but not for release binaries.

**How to avoid:** The `debug_assert!` already catches `{{` tokens — as long as `{{FERRO_VERSION}}` is in a code path that reaches the output string and the replace is omitted, the assert fires locally. Additionally, the renderer test `no_unresolved_tokens_in_dockerfile` (already present in entrypoint_tests) covers the single-bin case and must be expanded with a `has_frontend = true` variant.

**Warning signs:** `{{FERRO_VERSION}}` appears literally in a rendered Dockerfile.

### Pitfall 2: `docker_template_drift` Check Will Always Warn After Phase Until Both Call Sites Are Updated

**What goes wrong:** If `DockerContext` gains `ferro_version: String` but only `docker_init.rs` is updated (not `docker_template_drift.rs`), the drift check will construct a `DockerContext` without the field (compile error) or with an incorrect fallback.

**Why it happens:** Two independent call sites (`docker_init.rs` and `docker_template_drift.rs`) both construct `DockerContext` directly. Adding a non-`Option` field is a breaking struct literal — the compiler will catch this, but both must be updated in the same task.

**How to avoid:** Update both call sites in the same commit / task. The compiler enforces this via struct literal exhaustiveness — if `ferro_version` is not `Option`, omitting it is a compile error.

### Pitfall 3: `default_checks_returns_ten_in_declared_order` Test Will Fail

**What goes wrong:** `registry.rs` has a test asserting exactly 10 checks and listing their names. Adding `FrontendTypesConventionCheck` (check 11) breaks this test.

**Why it happens:** [VERIFIED: codebase] — `registry.rs` line 35-48 has `assert_eq!(checks.len(), 10)` and a hardcoded name list.

**How to avoid:** Update the test to assert 11 checks and add `"frontend_types_convention"` to the name list. Also update `check.rs` line 224 (`general_names` and `deploy_names` arrays). Also update `cli/doctor.md` check count ("nine checks" appears at line 3 and in the check table) and `reference/cli.md` if it mentions a count.

### Pitfall 4: The `inertia-props.ts.tpl` Scaffold Template Remains in `frontend/src/types/`

**What goes wrong:** The scaffolder writes `frontend/src/types/inertia-props.ts.tpl` as an initial placeholder at `ferro new` time. This file is fine (it is replaced by the real generator on first run), but it must not be confused with the tracked reference app files being untracked by D-05.

**Why it happens:** [VERIFIED: codebase] — `ferro-cli/src/templates/files/frontend/src/types/inertia-props.ts.tpl` exists and is emitted during scaffolding. The reference app's tracked files are separate from this template.

**How to avoid:** The git rm --cached in D-05 targets only `app/frontend/src/types/inertia-props.ts` and `app/frontend/src/types/routes.ts`. The `.tpl` file in the scaffolder is not changed. No confusion arises as long as the task is specific about paths.

### Pitfall 5: Docker stage uses `RUST_IMAGE_TAG` but types-gen needs full Rust toolchain (not slim)

**What goes wrong:** The current Dockerfile uses `rust:{{RUST_IMAGE_TAG}}` for the build stages where `RUST_IMAGE_TAG` resolves to e.g. `slim-bookworm`. For the `types-gen` stage, `cargo install ferro-cli` requires building from source — this works fine with the standard `rust:slim-bookworm` image since it includes `cargo`.

**Why it happens:** [VERIFIED: codebase] — `rust:slim-bookworm` does include the full Rust toolchain (it is the Rust official image, slim variant removes only documentation). `cargo install` works.

**How to avoid:** Use the same `rust:{{RUST_IMAGE_TAG}}` base for types-gen (as sketched in D-15). No separate image needed.

### Pitfall 6: `cargo install ferro-cli --locked` Requires `Cargo.lock` in `ferro-cli`

**What goes wrong:** `cargo install --locked` uses the lock file from the published crate. Since `ferro-cli` is published on crates.io with its `Cargo.lock`, `--locked` is valid for `cargo install`. However, if the version passed via `{{FERRO_VERSION}}` is not yet published (e.g., during a development cycle before the version bump is pushed), `cargo install` will fail.

**Why it happens:** The types-gen stage installs `ferro-cli` from crates.io, not from the project's local source.

**How to avoid:** The `FERRO_VERSION` is parsed from the project's `Cargo.lock` (which reflects what was used to build the project). This version must already be published. Per D-21 fallback, `env!("CARGO_PKG_VERSION")` is the ferro binary's own version — which is published (ferro-rs is published on crates.io per project memory). For development/pre-release versions, the Docker build naturally fails in CI before publish — acceptable because the types-gen fix targets consumer production deploys, not Ferro's own CI.

---

## Code Examples

### Example 1: FrontendTypesConventionCheck Unit Tests

```rust
// Source: pattern from ferro-cli/src/doctor/checks/generated_artifacts.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn name_is_frontend_types_convention() {
        assert_eq!(FrontendTypesConventionCheck.name(), "frontend_types_convention");
    }

    #[test]
    fn absent_directory_is_ok() {
        let tmp = TempDir::new().unwrap();
        // no frontend/src/types/ created
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }

    #[test]
    fn only_generated_files_is_ok() {
        let tmp = TempDir::new().unwrap();
        let types = tmp.path().join("frontend/src/types");
        fs::create_dir_all(&types).unwrap();
        fs::write(types.join("inertia-props.ts"), "").unwrap();
        fs::write(types.join("routes.ts"), "").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }

    #[test]
    fn hand_written_file_warns() {
        let tmp = TempDir::new().unwrap();
        let types = tmp.path().join("frontend/src/types");
        fs::create_dir_all(&types).unwrap();
        fs::write(types.join("parsed-menu.ts"), "").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Warn);
        assert!(r.details.as_ref().unwrap().contains("parsed-menu.ts"));
        assert!(r.details.as_ref().unwrap().contains("frontend/src/lib/types/"));
    }

    #[test]
    fn mixed_generated_and_hand_written_warns_on_hand_written_only() {
        let tmp = TempDir::new().unwrap();
        let types = tmp.path().join("frontend/src/types");
        fs::create_dir_all(&types).unwrap();
        fs::write(types.join("inertia-props.ts"), "").unwrap(); // allowed
        fs::write(types.join("theme-config.ts"), "").unwrap();  // hand-written
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Warn);
        assert!(r.details.as_ref().unwrap().contains("theme-config.ts"));
        assert!(!r.details.as_ref().unwrap().contains("inertia-props.ts"));
    }
}
```

### Example 2: FERRO_VERSION Resolution in docker_init.rs

```rust
// Call site pattern (docker_init.rs execute() function)
let ferro_version = resolve_ferro_version(&root);
let ctx = DockerContext {
    rust_channel,
    has_frontend,
    bins,
    web_bin,
    copy_dirs_present,
    runtime_apt: metadata.runtime_apt.clone(),
    ferro_version,  // NEW field
};
```

### Example 3: Registry Update (registry.rs)

```rust
// After change — excerpt from default_checks()
use super::checks::{
    // ... existing imports ...
    FrontendTypesConventionCheck,
};

pub fn default_checks() -> Vec<Box<dyn DoctorCheck>> {
    vec![
        // ... existing 10 checks ...
        Box::new(FrontendTypesConventionCheck),
    ]
}
// Test: assert_eq!(checks.len(), 11) and updated names vec
```

---

## State of the Art

| Old Approach | Current Approach (This Phase) | Impact |
|--------------|-------------------------------|--------|
| Generator output tracked in git | Generator output gitignored, regenerated from Rust source | No git noise; no drift possible |
| Docker build relies on committed generated files | Docker builds have a `types-gen` stage that regenerates | Production deploys no longer fail |
| Generator header directs users to wrong path | Header points to `frontend/src/lib/types/` | New users follow the convention |
| No enforcement of convention | `ferro doctor` flags violations | Convention is discoverable |

---

## Verified Implementation Facts

### Files Confirmed to Exist and Be Tracked
- [VERIFIED: git ls-files] `app/frontend/src/types/inertia-props.ts` — tracked, must be untracked.
- [VERIFIED: git ls-files] `app/frontend/src/types/routes.ts` — tracked, must be untracked.

### Generator Writes Exactly These Two Files
- [VERIFIED: codebase, generate_types.rs line 882] `frontend/src/types/inertia-props.ts`
- [VERIFIED: codebase, generate_types.rs line 922] `frontend/src/types/routes.ts`
D-20 allowlist is accurate; no other files are written by the generator.

### gitignore.tpl Line 14-15
- [VERIFIED: codebase] Line 14 is the comment `# generated_types`, line 15 is `frontend/src/types/`.
- Comment needs strengthening to "load-bearing" annotation per D-06.

### generate_types.rs Header Bug
- [VERIFIED: codebase, lines 710-711] Comment currently says `frontend/src/types/` — must become `frontend/src/lib/types/`.

### DockerContext Has No ferro_version Field
- [VERIFIED: codebase, docker.rs lines 33-53] Struct confirmed. Field must be added.

### docker_init.rs Already Has _ferro_version_flag Stub
- [VERIFIED: codebase, docker_init.rs line 55] `_ferro_version_flag: Option<&str>` parameter exists but is unused (underscore-prefixed, ignored). The real resolution logic (D-21, parse from Cargo.lock) must be implemented and passed into `DockerContext`.

### Cargo.lock Contains ferro-rs
- [VERIFIED: codebase] `Cargo.lock` has `name = "ferro-rs"` with `version = "0.2.33"`. Parse path confirmed.

### Docs Location for New Page
- [VERIFIED: codebase] No `docs/src/dx/` directory exists. CLI-adjacent docs live in `docs/src/cli/`. Recommended path: `docs/src/cli/frontend-types.md` to match `doctor.md`, `do-init.md`, `ci-init.md`.
- SUMMARY.md already has a Reference section with CLI sub-entries. New entry goes under `[doctor](cli/doctor.md)`.

### Doctor Count Hardcoding
- [VERIFIED: codebase, registry.rs line 37] `assert_eq!(checks.len(), 10)` — must become 11.
- [VERIFIED: codebase, check.rs line 224] `general_names` array — must add `"frontend_types_convention"`.
- [VERIFIED: codebase, cli/doctor.md line 3] "Runs nine checks" — must become "ten checks" (and #10 row added to table).

---

## Open Questions

1. **Should `frontend_types_convention` appear in the `--deploy` filter?**
   - What we know: the check is advisory (WARNING), not deploy-specific. Convention violations don't break deploys directly (they break deploys only in the absence of the types-gen Docker stage fix).
   - What's unclear: whether surfacing it in `ferro doctor --deploy` is useful.
   - Recommendation: Keep `General` category (default). Adding it to `Deploy` is deferred per D-10 (advisory only, not blocking).

2. **Does the scaffold README template need a troubleshooting entry for "missing types"?**
   - What we know: D-11 says document bootstrap sequence in scaffold README template. The current `README.md.tpl` has a Troubleshooting section with existing entries.
   - What's unclear: exactly what text to add.
   - Recommendation: Add a troubleshooting entry under "Frontend assets missing" — "TypeScript errors about missing `./types/inertia-props` — run `cargo run` once to generate types before running `npm run dev`."

3. **Does `reference/cli.md` mention a specific check count for `ferro doctor`?**
   - What we know: `docs/src/cli/doctor.md` says "nine checks" (must become ten). The reference page may also.
   - What's unclear: whether `reference/cli.md` duplicates this count.
   - Recommendation: Executor must grep `reference/cli.md` for "nine checks" and update if found.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is purely code/config/docs changes with no external runtime dependencies beyond the existing Rust toolchain and git.

---

## Validation Architecture

`nyquist_validation` key absent from `.planning/config.json` — treated as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | `Cargo.toml` (workspace) |
| Quick run command | `cargo test -p ferro-cli -- doctor` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req | Behavior | Test Type | Automated Command | File Exists? |
|-----|----------|-----------|-------------------|--------------|
| D-05 | `app/frontend/src/types/` files not tracked | git check | `git ls-files app/frontend/src/types/` (expect empty) | N/A — git op |
| D-06 | gitignore.tpl load-bearing comment present | string assertion | template test in `templates/mod.rs` | Extend existing |
| D-09/D-20 | `FrontendTypesConventionCheck` warns on hand-written files, ok on generated-only | unit | `cargo test -p ferro-cli -- checks::frontend_types_convention` | ❌ Wave 0 |
| D-15 | `types-gen` stage present when `has_frontend == true` | unit | `cargo test -p ferro-cli -- docker::tests::types_gen_stage_present_when_has_frontend` | ❌ Wave 0 |
| D-15 | `types-gen` stage absent when `has_frontend == false` | unit | same test module | ❌ Wave 0 |
| D-15 | `COPY --from=types-gen` before `RUN npm run build` | unit | `cargo test -p ferro-cli -- docker::tests::copy_from_types_gen_before_npm_build` | ❌ Wave 0 |
| D-16 | `{{FERRO_VERSION}}` resolved (no unresolved tokens) | unit | Extend existing `no_unresolved_tokens_in_dockerfile` with `has_frontend=true` | Extend existing |
| registry | 11 checks in declared order | unit | `cargo test -p ferro-cli -- doctor::registry::tests` | Update existing |
| D-18 | generate_types.rs header says `frontend/src/lib/types/` | string check | Manual / grep | Fix is trivial |

### Wave 0 Gaps
- [ ] `ferro-cli/src/doctor/checks/frontend_types_convention.rs` — new file, covers D-09/D-20
- [ ] Update `ferro-cli/src/doctor/checks/mod.rs` — pub mod + pub use
- [ ] Update `ferro-cli/src/doctor/registry.rs` — import + `default_checks()` + test update
- [ ] Update `ferro-cli/src/doctor/check.rs` — `general_names` array in test

---

## Security Domain

No security-sensitive changes in this phase. The doctor check reads filesystem paths (read-only, relative to the project root already determined by caller). No user input, no network, no credentials.

---

## Sources

### Primary (HIGH confidence — all verified via codebase read)
- `ferro-cli/src/templates/docker.rs` — `DockerContext`, `render_dockerfile`, `FRONTEND_STAGE_BODY`, existing tests
- `ferro-cli/src/commands/docker_init.rs` — call site structure, `_ferro_version_flag` stub
- `ferro-cli/src/commands/generate_types.rs` lines 700-922 — header comment bug, output file paths
- `ferro-cli/src/doctor/check.rs` — `DoctorCheck` trait, `CheckResult`, `CheckStatus`, `CheckCategory`
- `ferro-cli/src/doctor/registry.rs` — `default_checks()`, count assertion test
- `ferro-cli/src/doctor/checks/mod.rs` — module registration pattern
- `ferro-cli/src/doctor/checks/generated_artifacts.rs` — canonical check implementation pattern
- `ferro-cli/src/doctor/checks/docker_template_drift.rs` — second `DockerContext` call site
- `ferro-cli/src/templates/files/root/gitignore.tpl` — current comment text at line 14
- `ferro-cli/src/templates/files/root/README.md.tpl` — troubleshooting section structure
- `ferro-cli/src/templates/files/frontend/src/types/inertia-props.ts.tpl` — scaffold template (separate from reference app)
- `ferro-cli/Cargo.toml` — confirms `toml = "0.8"` and `tempfile` dev-dep
- `docs/src/SUMMARY.md` — docs structure, Reference section, CLI sub-entries
- `docs/src/cli/doctor.md` — check count ("nine checks"), check table
- `Cargo.lock` — `ferro-rs` at version `0.2.33` confirmed
- `git ls-files app/frontend/src/types/` — confirmed both files are tracked

### Secondary (MEDIUM confidence)
- `.planning/phases/156-frontend-types-directory-generator-owned-convention/156-CONTEXT.md` — design decisions, confirmed allowlist, confirmed file paths

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `rust:slim-bookworm` includes `cargo` and supports `cargo install` | Common Pitfalls §5 | types-gen stage fails in Docker; mitigation: use standard `rust:{{RUST_IMAGE_TAG}}` which always has cargo | [ASSUMED — standard Docker Hub rust image behavior; LOW risk] |

**All other claims are VERIFIED via codebase read.** No assumptions beyond A1.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps, all existing tools confirmed in Cargo.toml
- Architecture: HIGH — all implementation files read and confirmed
- Pitfalls: HIGH — derived from direct code inspection of the exact files being modified
- Docs location: HIGH — SUMMARY.md and docs/src/cli/ structure confirmed

**Research date:** 2026-05-14
**Valid until:** 2026-06-14 (stable internal codebase; only changes if docker.rs or doctor check API changes)
