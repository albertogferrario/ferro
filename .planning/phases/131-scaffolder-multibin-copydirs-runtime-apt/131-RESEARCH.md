# Phase 131: Scaffolder multi-bin, copy_dirs, runtime_apt, DO app.yaml robustness, drift detection — Research

**Researched:** 2026-04-09
**Domain:** ferro-cli deploy scaffolders (`docker:init`, `do:init`, `doctor`)
**Confidence:** HIGH

## Summary

Most of the infrastructure this phase needs already exists — the gestiscilo
field report was written against ferro 0.1.72 but the workspace has since
advanced to 0.2.0 (Phases 122.2, 127, 128). Concretely:

- `[package.metadata.ferro.deploy]` with `copy_dirs`, `runtime_apt`, `web_bin`
  is already parsed by `crate::project::read_deploy_metadata`.
- `read_bins` already returns every `[[bin]]` in declaration order.
- `detect_web_bin` already implements the 4-step precedence.
- The Dockerfile renderer already emits per-bin `COPY` lines, `COPY {dir}
  {dir}` for present `copy_dirs`, and a `# ferro:runtime-apt` apt layer.
- The `.do/app.yaml` renderer already emits a real `workers:` block from
  non-web, non-test-like `[[bin]]` entries.
- `.dockerignore` already whitelists `!README.md` via the Phase 127 pattern.
- `copy_dirs_dockerignore_collision` doctor check (Phase 128) already flags
  the `copy_dirs` vs `.dockerignore` collision class.

What is actually missing for the phase goal "byte-identical regeneration of
gestiscilo-it hand-maintained files":

1. **`.do/app.yaml` identity preservation on `--force`**. The renderer treats
   `region` (hardcoded `fra1`), `name` (derived from package), and `github.repo`
   (derived from git remote) as scaffolder-owned. None of them read the
   existing file. `--force` clobbers user-chosen identity fields.
2. **Unconditional `health_check` / dead-code checks**. Grep confirms the
   current `app.yaml.tpl` does NOT emit `health_check:` — the backlog note is
   stale on this specific point. BUT the current template DOES unconditionally
   reference `github.repo` / `branch: main` / `deploy_on_push: true` without
   reading the existing file's binding.
3. **Dockerfile frontend stage detection**. Today `has_frontend` is purely
   `frontend/package.json` existence. That is already correct for gestiscilo
   (no `frontend/`), so the backlog claim "produced a Node.js frontend build
   stage" is stale — Phase 127 already fixed this. **Must verify by running
   the scaffolder against the current gestiscilo tree before writing the
   plan.**
4. **`.env.example` envs-block "didn't fire"**. Code path in `do_init.rs` IS
   wired (Phase 127 D-06) and has a passing test
   (`run_inner_succeeds_with_missing_env_example`). Most likely gestiscilo
   simply had no `.env.example` at field-test time — not a bug in current
   code, but the plan should verify.
5. **Drift detection**. No `docker_template_drift` check exists. Scaffolder
   rendering is already pure (`render_dockerfile`, `render_app_yaml`) so a
   drift check can call them directly with a reconstructed context and diff
   against committed files.
6. **Dockerfile copy_dirs default list caveat**. `FerroDeployMetadata::default`
   already ships with `copy_dirs = ["themes", "lang", "public", "migrations"]`
   — this is the "copy_dirs already works by default" path. gestiscilo's
   `themes/` should already be picked up automatically.

**Primary recommendation:** frame this phase as a **gap-closing + verification
pass**, not a greenfield rewrite. The killer feature (byte-identical
regeneration of gestiscilo `6f6d397`) hinges on two small deltas
(identity-field preservation in `.do/app.yaml`; possibly dropping
`deploy_on_push`/`branch` defaults) plus the new drift check. Step 1 of the
plan must be to actually run `cargo run -- docker:init --dry-run` and
`do:init --dry-run` against the gestiscilo tree and diff — most of the
backlog claims are likely already fixed and the real delta list is smaller
than the report suggests.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-131-01 | `docker:init` builds and wires every `[[bin]]` in Dockerfile | Already implemented (`render_dockerfile` emits per-bin COPY; single `cargo build --release` builds all). Verify no gap. |
| REQ-131-02 | `do:init` emits first `[[bin]]` as web service, others as workers | Already implemented via `detect_web_bin` + workers filter in `do_init.rs`. Verify. |
| REQ-131-03 | Dockerfile emits `COPY {dir} {dir}` for `copy_dirs` present on disk | Already implemented (`copy_dirs_present` filter in `docker_init.rs`). |
| REQ-131-04 | `.dockerignore` does not block `copy_dirs` entries | Doctor check already guards this. Template has `!README.md`-style whitelist pattern but no generated whitelist for `copy_dirs`. Plan: either emit `!theme/` entries in rendered `.dockerignore` when `copy_dirs` is non-default, OR rely on the doctor check (current posture). Decide. |
| REQ-131-05 | Dockerfile runtime stage apt layer from `runtime_apt` | Already implemented. |
| REQ-131-06 | `.do/app.yaml` preserves existing `region`, `name`, `github.repo`/`branch` on `--force` | **NOT implemented** — new work. Requires reading existing file via `serde_yaml` (or a targeted line-scan parser) and surgical merge. |
| REQ-131-07 | Phase 127 `.env.example` → envs path actually fires | Already wired; regression test exists. Plan: add a second test that parallels the gestiscilo layout exactly (empty env_lines vs missing file) and verify. |
| REQ-131-08 | Drop unconditional `health_check` block | **NOT emitted today** — claim is stale. No action required; add regression test asserting absence. |
| REQ-131-09 | Drop dead Node.js frontend build stage for server-rendered projects | Already implemented (`has_frontend = root.join("frontend/package.json").is_file()`). Add regression test for "empty tree ⇒ no frontend-builder". Likely already covered by `frontend_stage_present_only_when_has_frontend`. |
| REQ-131-10 | `ferro doctor` `docker_template_drift` check | **NOT implemented** — new work. Re-renders via pure functions and diffs. |
| REQ-131-11 | Byte-identical regeneration of gestiscilo `6f6d397` Dockerfile + `.do/app.yaml` | Single measurable success criterion. Must be verified in Wave 0 by actually running the scaffolders against a checkout. |

## Project Constraints (from CLAUDE.md + MEMORY)

- **Pre-1.0, no backward compat required.** Breaking changes to metadata
  keys or template tokens are acceptable.
- **Every commit:** `cargo fmt --all -- --check && cargo clippy --all
  --all-targets -- -D warnings && cargo test --all-features`. Matches CI.
- **No co-author lines**, no "Generated with Claude" in commits.
- **Killer feature framing:** the single user-visible payoff here is "field
  projects can point at the scaffolder instead of hand-maintaining". The
  byte-identical gestiscilo regeneration is the concrete forcing function.
- **Conceptual coherence:** `[package.metadata.ferro.deploy]` is already the
  blessed metadata surface. Do NOT invent a parallel `[package.metadata.ferro.docker]`
  — the backlog note suggested it but the existing table already covers it.
- **ferro-mcp:** any new doctor check must be discoverable via `ferro doctor`;
  the MCP `deploy_check` tool (Phase 128) already filters on category, so
  `docker_template_drift` should return `CheckCategory::Deploy`.
- **Repository documents must read as neutral** — no "killer feature" /
  "the bet" language in any committed planning doc.

## Standard Stack (already in tree — no new deps expected)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `toml` | workspace | Parse `Cargo.toml` + `[package.metadata.ferro.deploy]` | Already used across `project.rs`, `deploy/`, `templates/docker.rs` |
| `anyhow` | workspace | Error propagation from scaffolder commands | Already the CLI-wide convention |
| `tempfile` | dev | Integration tests that render into a throwaway tree | Used throughout `docker_init`/`do_init` tests |
| `console` | workspace | Styled warnings/errors in `do_init` | Already used |
| `serde_yaml` *(possible new)* | 0.9 | Parse existing `.do/app.yaml` for identity-field preservation | Ferro does not currently depend on it. **Alternative:** a targeted `regex`-free line scanner for `region:`, `name:`, `github: { repo, branch }`. Recommended: scanner (~50 lines) over adding a new dep for three fields. |

**Installation:** none expected unless the plan chooses `serde_yaml`. Run
`cargo tree -p ferro-cli` to confirm before adding.

## Architecture Patterns

### Pure render + I/O at the edge

Every existing scaffolder renderer (`render_dockerfile`, `render_app_yaml`,
`render_envs_block_from_lines`) is an I/O-free function taking a `*Context`
struct. The command layer (`commands/docker_init.rs`, `commands/do_init.rs`)
resolves all inputs first (Cargo.toml, rust-toolchain, `.env.example`, git
remote, existing `.do/app.yaml`), builds the context, then calls the
renderer. This boundary is LOAD-BEARING for the drift check: it can
reconstruct a context, call the renderer, and diff the result without
re-implementing anything.

**Rule for Phase 131:** any new logic (identity-field preservation, drift
diff) stays behind this boundary. I/O goes in `commands/`, pure transforms
go in `templates/` or a new `deploy/` helper.

### Pattern: template string tokens

Templates are `include_str!`-ed and rendered via chained `.replace(
"{{TOKEN}}", ...)` calls, guarded by a `debug_assert!(!rendered.contains("{{
"))`. No handlebars, no tera. Loop-shaped content (bin copies, worker
entries, envs entries) is pre-built as a single string in Rust and
substituted as one token (`{{BIN_COPIES}}`, `{{WORKERS_BLOCK}}`,
`{{ENVS_BLOCK}}`).

**Rule for Phase 131:** preserve this pattern. If new conditional blocks are
needed (e.g., `{{GITHUB_BLOCK}}` vs preserved existing binding), render the
block as a string at the caller and substitute a single token.

### Pattern: doctor check scaffold

Every check is a unit struct implementing `DoctorCheck` with
`name`/`run`/`category`. The actual logic lives in a free `check_impl(root:
&Path) -> CheckResult` function beside the struct, and tests call
`check_impl` directly. New check files are declared in `doctor/checks/mod.rs`
and added to `doctor/registry.rs::default_checks()` in declared order.

### Recommended task structure

```
ferro-cli/src/
├── commands/
│   ├── docker_init.rs    # verify + add regression tests for gestiscilo shape
│   └── do_init.rs        # read existing .do/app.yaml, preserve identity fields
├── deploy/
│   └── app_yaml_existing.rs  # new: parse identity fields from existing file
├── templates/
│   └── do.rs             # extend AppYamlContext with preserved fields
└── doctor/
    └── checks/
        └── docker_template_drift.rs  # new
```

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| `Cargo.toml` metadata parsing | A second parser | `crate::project::read_deploy_metadata` | Already handles defaults, type errors, field validation |
| `[[bin]]` enumeration | A fresh walker | `crate::project::read_bins` (also `templates::docker::read_bins`) | Two already exist; plan must pick one and retire the other to avoid drift |
| Web-bin resolution | New precedence logic | `crate::deploy::bin_detect::detect_web_bin` | D-02 4-step order already tested |
| `.env.example` parsing | Ad-hoc line splitting | `crate::deploy::env_production::parse_env_example_structured` | Preserves blank separators for envs block |
| GitHub remote parsing | Regex URL cracking | `crate::templates::do_::parse_git_remote` | Already handles https/ssh/.git forms |

**Key insight:** two `read_bins` functions exist today (one in
`project.rs`, one in `templates/docker.rs`). This is a latent inconsistency
the phase should collapse. The `templates/docker.rs::read_bins` returns
`Vec<String>`; `project.rs::read_bins` returns `Vec<BinEntry>`. Pick one,
delete the other, update callers — this is the kind of "continuous
conceptual coherence" refactor CLAUDE.md asks for.

## Runtime State Inventory

*(Phase 131 is a scaffolder-output change; no runtime datastore state. Still,
here is the audit.)*

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — scaffolders are write-only, no DB or cache state | None |
| Live service config | `.do/app.yaml` in downstream project repos (e.g. gestiscilo) holds identity fields (region, name, github binding) that will be preserved by REQ-131-06. No ferro-side state. | Document in phase SUMMARY that downstream projects need to re-run `do:init --force` to pick up new worker blocks. |
| OS-registered state | None | None |
| Secrets / env vars | None — scaffolder does not touch secrets. The `.env.example`-derived envs block emits `value: ""` placeholders only. | None |
| Build artifacts | `Dockerfile`, `.dockerignore`, `.do/app.yaml` in downstream repos become stale after this phase ships. Drift check will flag them. | Downstream migration is out of scope for this phase (per `<additional_context>`). |

## Common Pitfalls

### Pitfall 1: Double-source of `[[bin]]` enumeration
**What goes wrong:** Dockerfile and `.do/app.yaml` disagree about which bins
exist because they call different readers.
**Why it happens:** `project.rs::read_bins` and
`templates/docker.rs::read_bins` coexist today.
**How to avoid:** Collapse to a single function early in the phase.
**Warning sign:** A test that passes for docker but fails for DO (or vice
versa) when new bins are added.

### Pitfall 2: Byte-identical diff failing on trailing newline
**What goes wrong:** Renderer emits file without trailing `\n`; hand-maintained
file has one. `byte_eq` check fails loudly, scaffolder looks broken.
**Why it happens:** Rust `format!` + `push_str` patterns don't always end in
`\n`.
**How to avoid:** Plan the drift check to normalize trailing newline before
comparison, and add a unit test for "scaffolder output ends in a single \n".

### Pitfall 3: `.do/app.yaml` identity-field parse fragility
**What goes wrong:** Existing file uses non-standard indentation or comments
between `github:` and its fields; a line-scanner misses the binding.
**Why it happens:** YAML is not regular; a real parser is safer.
**How to avoid:** Either accept `serde_yaml` as a dependency for this narrow
case, OR constrain the preservation to top-level keys (`name:`, `region:`)
plus the literal `github:` block under the web service and error clearly if
the shape is unexpected. Prefer the second: it matches the "scaffolder emits
a known shape" contract.

### Pitfall 4: Drift check false positive on ambient env
**What goes wrong:** `detect_github_repo` runs `git remote get-url origin` —
in CI without a checked-out origin the drift check will see a different
repo string than the committed file.
**Why it happens:** Context reconstruction depends on the environment.
**How to avoid:** The drift check should reconstruct context **from the
committed file itself** for identity fields (the same preservation path used
by `--force`), not from `git`. Drift check compares: "if I re-rendered with
the committed identity fields, would I get the committed file?"

### Pitfall 5: `copy_dirs` default list hides user intent
**What goes wrong:** `FerroDeployMetadata::default` bakes in `["themes",
"lang", "public", "migrations"]`. A project without an explicit
`[package.metadata.ferro.deploy]` gets the default list; if their
`.dockerignore` excludes one of them, behavior depends on on-disk presence.
**Why it happens:** Default list was a convenience in Phase 122.
**How to avoid:** Document the default list in the phase SUMMARY; consider
narrowing it to `["migrations"]` only if the byte-identical diff against
gestiscilo demands it. Decide during Wave 0 verification.

## Code Examples

### Pattern for drift check (sketch)

```rust
// ferro-cli/src/doctor/checks/docker_template_drift.rs
use crate::doctor::check::{CheckCategory, CheckResult, DoctorCheck};
use crate::project::{find_project_root, read_deploy_metadata, package_name};
use crate::deploy::bin_detect::detect_web_bin;
use crate::templates::docker::{render_dockerfile, DockerContext, read_bins, read_rust_channel};
use std::fs;
use std::path::Path;

const NAME: &str = "docker_template_drift";

pub struct DockerTemplateDriftCheck;

impl DoctorCheck for DockerTemplateDriftCheck {
    fn name(&self) -> &'static str { NAME }
    fn category(&self) -> CheckCategory { CheckCategory::Deploy }
    fn run(&self, root: &Path) -> CheckResult { check_impl(root) }
}

fn check_impl(root: &Path) -> CheckResult {
    let dockerfile = root.join("Dockerfile");
    if !dockerfile.is_file() {
        return CheckResult::ok(NAME, "skipped (Dockerfile absent)");
    }
    let committed = match fs::read_to_string(&dockerfile) {
        Ok(s) => s,
        Err(e) => return CheckResult::error(NAME, format!("read failed: {e}")),
    };
    let metadata = match read_deploy_metadata(root) {
        Ok(m) => m,
        Err(e) => return CheckResult::error(NAME, format!("metadata: {e}")),
    };
    let bins = match read_bins(root) {
        Ok(b) => b,
        Err(e) => return CheckResult::error(NAME, format!("read_bins: {e}")),
    };
    let web_bin = match detect_web_bin(root) {
        Ok(w) => w,
        Err(e) => return CheckResult::error(NAME, format!("web_bin: {e}")),
    };
    let ctx = DockerContext {
        rust_channel: read_rust_channel(root),
        has_frontend: root.join("frontend/package.json").is_file(),
        bins,
        web_bin,
        copy_dirs_present: metadata.copy_dirs.iter().filter(|d| root.join(d).exists()).cloned().collect(),
        runtime_apt: metadata.runtime_apt.clone(),
    };
    let rendered = render_dockerfile(&ctx);
    if rendered.trim_end() == committed.trim_end() {
        CheckResult::ok(NAME, "Dockerfile matches scaffolder output")
    } else {
        CheckResult::warn(NAME, "Dockerfile has drifted from scaffolder")
            .with_details("run `ferro docker:init --dry-run` to inspect the delta")
    }
}
```

(Drift is `Warn`, not `Error` — hand-editing remains legitimate; users need
to know but not be blocked.)

### Pattern for `.do/app.yaml` identity preservation (sketch)

```rust
// ferro-cli/src/deploy/app_yaml_existing.rs
pub struct PreservedAppYamlIdentity {
    pub name: Option<String>,
    pub region: Option<String>,
    pub repo: Option<String>,
    pub branch: Option<String>,
}

pub fn parse_existing(path: &std::path::Path) -> Option<PreservedAppYamlIdentity> {
    let src = std::fs::read_to_string(path).ok()?;
    let mut out = PreservedAppYamlIdentity { name: None, region: None, repo: None, branch: None };
    for line in src.lines() {
        let trimmed = line.trim_start();
        if let Some(v) = trimmed.strip_prefix("name: ") { if line.starts_with("name: ") { out.name = Some(v.trim().to_string()); } }
        if let Some(v) = trimmed.strip_prefix("region: ") { if line.starts_with("region: ") { out.region = Some(v.trim().to_string()); } }
        if let Some(v) = trimmed.strip_prefix("repo: ") { out.repo = Some(v.trim().to_string()); }
        if let Some(v) = trimmed.strip_prefix("branch: ") { out.branch = Some(v.trim().to_string()); }
    }
    Some(out)
}
```

Top-level `name:` / `region:` are disambiguated by checking that the line
starts at column 0 (no indent) — the only `repo:` / `branch:` in the emitted
template live under `services[0].github:`, so indented matches are safe.

## State of the Art

| Old approach | Current approach | When changed | Impact |
|---|---|---|---|
| Single-bin Dockerfile with hardcoded binary name | Multi-bin via `read_bins` + per-bin COPY | Phase 122.2 | This phase verifies, does not introduce |
| `Cargo.docker.toml` dual manifest for path→git rewriting | Single `Cargo.toml`, local dev via uncommitted `[patch.crates-io]` | Phase 130 | Scaffolder no longer generates overlay |
| Unconditional frontend build stage | `has_frontend` gates the stage | Phase 127 | Backlog claim about dead stage is stale |
| `.env.production` drove envs block | `.env.example` drives envs block | Phase 127 D-06 | Values stay on dev machine |

**No deprecated paths remain.** All prior scaffolder phases (122, 122.2,
127, 128, 130) are landed.

## Open Questions

1. **Is the gestiscilo backlog report still accurate against 0.2.0?**
   - What we know: Most of the claimed gaps (frontend stage, multi-bin COPY,
     workers block, copy_dirs, runtime_apt) appear to be fixed already.
   - What's unclear: Until the scaffolders are actually run against the
     gestiscilo tree, the real delta list is unverified.
   - Recommendation: **Wave 0 of the plan MUST be "run scaffolders against
     gestiscilo, diff, document real gaps"**. Per MEMORY.md
     `feedback_validate_scope_premises.md` — verify world-state claims
     before building on them.

2. **Should `.dockerignore` emit explicit `!{dir}` whitelist lines for
   `copy_dirs`?**
   - What we know: Today the dockerignore is a static include_str template
     with `!README.md` only. The doctor collision check catches mistakes.
   - What's unclear: Whether byte-identity against gestiscilo requires
     generated whitelist lines, or whether gestiscilo simply removed the
     broad excludes.
   - Recommendation: Decide after Wave 0 diff. Lean: keep static, rely on
     doctor check.

3. **`serde_yaml` vs line-scanner for identity preservation?**
   - What we know: Three fields (name, region, github.{repo,branch}).
   - What's unclear: Future expansion (does any other field become
     identity-owned?).
   - Recommendation: Line scanner now; migrate to `serde_yaml` only if the
     preserved field set grows beyond four.

4. **Drift check severity: `Warn` or `Error`?**
   - Recommendation: `Warn`. Hand-editing remains legitimate — the check
     exists to *inform*, not to block. Error would punish legitimate users.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | All build + test | ✓ | workspace-managed | — |
| `git` | `detect_github_repo` + drift check ambient test skip | ✓ | system | Test harness already injects `tempfile` dirs with no git remote; code handles `None` gracefully |
| gestiscilo checkout | Wave 0 verification diff | **user-local** | commit `6f6d397` | Plan must specify where the checkout lives or hard-code the reference file snapshots in `tests/fixtures/` |

**Missing dependencies with no fallback:** gestiscilo checkout must be
reachable by the plan author. If it is not, copy the two files from
`6f6d397` into `ferro-cli/tests/fixtures/gestiscilo-{Dockerfile,app.yaml}`
and assert byte-equality against those fixtures.

## Validation Architecture

### Test framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (std test harness) with `tempfile` |
| Config file | `ferro-cli/Cargo.toml` `[dev-dependencies]` already carries `tempfile` |
| Quick run command | `cargo test -p ferro-cli --lib docker_init do_init doctor -- --nocapture` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase requirements → test map

| Req ID | Behavior | Test Type | Automated command | File exists? |
|--------|----------|-----------|-------------------|--------------|
| REQ-131-01 | Multi-bin Dockerfile COPY | unit | `cargo test -p ferro-cli multi_bin_emits_per_bin_copy_without_per_bin_build` | ✅ |
| REQ-131-02 | `.do/app.yaml` workers from extra bins | unit | `cargo test -p ferro-cli render_app_yaml_emits_each_worker` | ✅ |
| REQ-131-03 | `copy_dirs` COPY emission | unit | `cargo test -p ferro-cli copy_dirs_emits_only_present_entries` | ✅ |
| REQ-131-04 | `.dockerignore` collision detection | unit | `cargo test -p ferro-cli copy_dirs_dockerignore_collision` | ✅ |
| REQ-131-05 | `runtime_apt` layer | unit | `cargo test -p ferro-cli runtime_apt_nonempty_emits_marker_and_packages` | ✅ |
| REQ-131-06 | Identity preservation on `--force` | integration | `cargo test -p ferro-cli do_init_preserves_identity` | ❌ Wave 0 |
| REQ-131-07 | `.env.example` envs path fires | integration | `cargo test -p ferro-cli do_init_envs_from_env_example` | ❌ Wave 0 |
| REQ-131-08 | No `health_check` block | unit | `cargo test -p ferro-cli app_yaml_has_no_health_check` | ❌ Wave 0 (regression) |
| REQ-131-09 | No frontend stage without `frontend/package.json` | unit | `cargo test -p ferro-cli frontend_stage_present_only_when_has_frontend` | ✅ |
| REQ-131-10 | Drift check | unit | `cargo test -p ferro-cli docker_template_drift` | ❌ Wave 0 |
| REQ-131-11 | Byte-identical gestiscilo | fixture | `cargo test -p ferro-cli gestiscilo_byte_identical` | ❌ Wave 0 (fixtures) |

### Sampling rate
- **Per task commit:** `cargo test -p ferro-cli --lib`
- **Per wave merge:** full suite command above
- **Phase gate:** full suite + manual `cargo run -- docker:init --dry-run` and
  `do:init --dry-run` against a real gestiscilo checkout, producing a
  byte-equal diff.

### Wave 0 gaps
- [ ] `ferro-cli/tests/fixtures/gestiscilo/Dockerfile` — committed fixture from `6f6d397`
- [ ] `ferro-cli/tests/fixtures/gestiscilo/app.yaml` — committed fixture from `6f6d397`
- [ ] `ferro-cli/tests/fixtures/gestiscilo/Cargo.toml` — minimal reproduction with the two `[[bin]]`, `[package.metadata.ferro.deploy]` copy_dirs + runtime_apt
- [ ] New test module `docker_init_gestiscilo_fixture` asserting byte-identical render
- [ ] New test module `do_init_gestiscilo_fixture` idem
- [ ] New `doctor::checks::docker_template_drift` file
- [ ] New `deploy::app_yaml_existing` module (or equivalent) for identity parsing

## Sources

### Primary (HIGH confidence)
- `ferro-cli/src/project.rs` — `FerroDeployMetadata`, `read_deploy_metadata`, `find_project_root`, `package_name`
- `ferro-cli/src/deploy/bin_detect.rs` — `detect_web_bin` 4-step precedence
- `ferro-cli/src/deploy/env_production.rs` — `parse_env_example_structured`, `EnvLine`
- `ferro-cli/src/templates/docker.rs` — `DockerContext`, `render_dockerfile`, `read_bins`, `read_rust_channel`
- `ferro-cli/src/templates/do.rs` — `AppYamlContext`, `render_app_yaml`, `sanitize_do_app_name`, `parse_git_remote`, `is_test_like_bin`
- `ferro-cli/src/templates/files/docker/Dockerfile.tpl` — current template with `{{BIN_COPIES}} {{COPY_DIRS}} {{RUNTIME_APT}} {{ENTRYPOINT}}` tokens
- `ferro-cli/src/templates/files/do/app.yaml.tpl` — current template; does NOT emit `health_check:` (claim in backlog is stale)
- `ferro-cli/src/commands/docker_init.rs` — command-layer composition
- `ferro-cli/src/commands/do_init.rs` — command-layer composition + `.env.example` read path (D-06)
- `ferro-cli/src/doctor/{check,registry}.rs` — check trait and ordered registry
- `ferro-cli/src/doctor/checks/copy_dirs_dockerignore_collision.rs` — reference scaffold for a new deploy-category check
- `.planning/backlog/gestiscilo-scaffolder-multibin-gap.md` — source field report (treat as hypothesis, not ground truth — several items appear already fixed)
- `.planning/STATE.md` — phases 122.2 / 127 / 128 / 130 landed

### Secondary (MEDIUM confidence)
- MEMORY `feedback_validate_scope_premises.md` — verify world-state claims with one command before building on them (drives the "Wave 0 diff against gestiscilo" recommendation)
- MEMORY `feedback_killer_feature_framing.md` — frame work against projection/intent; note this phase is deploy-path, so killer-feature framing is instead "byte-identical regeneration closes the hand-maintenance drift loop"

### Tertiary (LOW confidence)
- Version 0.1.72 vs current 0.2.0 delta — inferred from `STATE.md` phase log, not directly verified against a 0.1.72 checkout

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all infrastructure is in-tree and read directly
- Architecture patterns: HIGH — patterns are consistent across existing code
- Pitfalls: MEDIUM — pitfall 2 (trailing newline) and pitfall 3 (YAML
  fragility) are inferred from the code shape, not empirically hit yet
- Backlog claim accuracy: LOW — several claims appear stale; Wave 0 must
  verify before the plan commits to deltas

**Research date:** 2026-04-09
**Valid until:** 2026-05-09 (stable area; ferro-cli deploy scaffolder surface
moves slowly post-Phase 130)
