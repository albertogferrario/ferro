# Phase 122: Deploy Scaffold Core Rewrite - Research

**Researched:** 2026-04-07
**Domain:** ferro-cli code generation (Dockerfile / .dockerignore / DO App Platform spec)
**Confidence:** HIGH (internal codebase) / MEDIUM (external deployment semantics — gestiscilo/mkmenu reference apps not present on disk)

## Summary

Phase 122 rewrites `ferro docker:init` and `ferro do:init` plus their three templates so that generated deploy artifacts work for real Ferro apps (multi-bin, chromium, themes/, postgres, path-dep ferro crates) without hand-patching. The current implementation is a straight string-replace of a single static `.tpl` (one binary, one stage shape, no path-dep rewrite, no workers, no region flag, no `--force`). All substitution goes through `src/templates/docker.rs`, which calls `include_str!` on `.tpl` files under `src/templates/files/{docker,do}/`. `clap` derive and the `toml` crate (0.8) are already workspace deps, so no new dependencies are required.

The rewrite is almost entirely mechanical given SCOPE.md's 21 locked decisions. The only non-trivial engineering is: (a) workspace-aware cargo-chef recipe copying, (b) the `scripts/rewrite-ferro-deps.sh` generator + its Dockerfile invocation point, and (c) `.env.example` → DO envs/databases block generation with SECRET auto-classification.

**Primary recommendation:** Keep the template-as-`include_str!`-with-placeholder pattern that already exists — do **not** pull in a templating engine (tera/handlebars). Introduce a builder-style context struct per template that is populated by the command, then passed to the renderer. Write golden-file tests per scenario (single-bin, multi-bin, frontend, no-frontend, workspace, non-workspace). Unit-test the three parsers (Cargo.toml bin enumeration, workspace member discovery, `.env.example` → DO envs).

## User Constraints (from CONTEXT.md)

### Locked Decisions
All 21 decisions from SCOPE.md are locked (see 122-CONTEXT.md `<decisions>` block for the full list). Summary by area:

- **Dockerfile (D-01 … D-07):** conditional frontend stage; multi-bin via `[[bin]]`; `--runtime-deps` flag with preserved marker block; detect-copy `themes/`/`lang/`/`public/`/`migrations/`; `ARG GITHUB_TOKEN=""` + `git config insteadOf`; honor `rust-toolchain.toml`; workspace-aware cargo-chef (copy `crates/`, `migration/`, sibling members).
- **Path → git ferro rewrite (D-08 … D-11):** generate `scripts/rewrite-ferro-deps.sh`; Dockerfile invokes it in planner+builder stages after `COPY .`; `--ferro-ref <branch|tag|sha>` flag (default `main`) persisted in script header; `ferro deploy:check` pre-flight validates ref is pushed via `git ls-remote`; also blocks `docker:build`.
- **app.yaml (D-12 … D-15):** `--region` flag default `fra1`; `envs:` from `.env.example` with `SCOPE: RUN_TIME` and `type: SECRET` auto-classification on `*_KEY|*_SECRET|*PASSWORD|*TOKEN|DATABASE_URL`; `databases:` block when `DATABASE_URL` present, referencing `${db.DATABASE_URL}`; one `workers:` entry per non-server `[[bin]]`.
- **Command plumbing (D-16 … D-19):** `--force` overwrite; walk up to find `Cargo.toml`; validate `--repo owner/repo` format; lift `get_package_name()` into shared `templates::project::package_name()` (or new `ferro-cli::project` module).
- **`.dockerignore` (D-20 … D-21):** add `database.db`, `*.sqlite*`, `.planning/`, `storage/`, `data/`; note drift vs gitignore but defer sync to Phase 124.

### Claude's Discretion
- Internal module layout for templates and helpers.
- Test strategy (unit tests + golden-file tests recommended — see Validation Architecture).
- Exact CLI error messages and progress output style (must follow existing ferro-cli `console::style` conventions already visible in `docker_init.rs` / `do_init.rs`).

### Deferred Ideas (OUT OF SCOPE)
- MCP deploy tools → Phase 123.
- `ferro doctor`, `routes --json`, CI scaffold → Phase 124.
- `.gitignore` ↔ `.dockerignore` drift sync automation → Phase 124.
- `make:module`, json-ui runtime split → Phase 125.

## Phase Requirements

No REQUIREMENTS.md entries for Phase 122. SCOPE.md `## Verification` section is the contract. The planner should treat these as acceptance criteria:

1. Regenerate gestiscilo deploy artifacts from scratch, build succeeds with zero hand edits, image contains both `gestiscilo` and `screenshot-worker` binaries plus chromium runtime.
2. Regenerate mkmenu deploy artifacts from scratch, frontend bundle still works, matches currently deployed shape.
3. `ferro deploy:check` fails loudly when ferro local commits are not pushed.
4. `ferro do:init --region nyc --repo owner/foo` writes `region: nyc` and a valid envs block derived from `.env.example`.

## Project Constraints (from CLAUDE.md)

- Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` before every commit. `-D warnings` is CI-enforced.
- No co-author lines in commits.
- Public API lives in `framework/src/lib.rs`; CLI commands in `ferro-cli/src/commands/`; templates in `ferro-cli/src/templates/`.
- Update `docs/src/` when CLI flags or behavior change (new `--force`, `--ferro-ref`, `--region`, `--runtime-deps`, `deploy:check` command).
- Update ferro-mcp `list_commands` introspection if new CLI commands are added — `deploy:check` is new.
- Concrete types only; no `interface{}`/`any`. Early returns. `fmt::Error`-style error chaining via `?` and `Result`.
- Prefer editing existing files over creating new ones; keep changes focused.

## Standard Stack

### Core (already in workspace — do NOT add new deps)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `clap` | 4 (derive) | CLI arg parsing | Already the ferro-cli standard; `DockerInit` / `DoInit` variants in `main.rs:322+` |
| `toml` | 0.8 | Parse `Cargo.toml`, `rust-toolchain.toml` | Already used in `docker_init.rs` and `do_init.rs` |
| `console` | (in use) | Colored output (`style(...).red().bold()`) | Existing ferro-cli convention |
| `std::fs` / `std::path` | std | File I/O, walk-up `Cargo.toml` discovery | No dep needed |
| `std::process::Command` | std | Shell out to `git ls-remote` for deploy:check | No dep needed |
| `include_str!` | std macro | Embed `.tpl` files at compile time | Existing convention in `src/templates/docker.rs` |

### Supporting (consider only if scope grows)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde_yaml` | — | Structured YAML emission for app.yaml | Only if the planner decides placeholder-replace is too fragile for the envs/databases/workers composition. Recommended: **stay with placeholder replace + line-assembled YAML blocks** to keep zero-new-deps. |
| `regex` | — | `.env.example` key classification | NOT needed — simple `str::ends_with` / `contains` on uppercase key suffixes is sufficient and already idiomatic in ferro-cli. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `include_str!` + `.replace()` | `tera` / `handlebars` | New dep, new DSL for contributors, lint noise. Current string-replace scales fine for ~5 templates with ~15 placeholders. Reject. |
| Line-assembled YAML for app.yaml | `serde_yaml` round-trip | serde_yaml is safer for nested structures; placeholder-replace is simpler for a mostly-static shape with a few injected sections. **Recommend: keep placeholder approach but assemble the `envs:`, `databases:`, `workers:` blocks in Rust as pre-indented strings, then substitute.** |
| `cargo_metadata` crate | raw `toml` parsing | `cargo_metadata` gives a typed workspace view (`Metadata::workspace_members`, `Package::targets` with `TargetKind::Bin`), which simplifies D-02 (bin enumeration) and D-07 (workspace members). However it shells out to `cargo metadata` (slower, requires cargo on PATH). Since `toml` is already in use and the needed data is simple (top-level `[[bin]]` array and workspace `members` array), **recommend: stay with raw `toml` parsing** to keep behavior deterministic and avoid shelling out. |

**Installation:** No new dependencies needed.

**Version verification:** Skipped — all recommended tools are std or already pinned in `ferro-cli/Cargo.toml`.

## Architecture Patterns

### Recommended Module Layout
```
ferro-cli/src/
├── commands/
│   ├── docker_init.rs         # rewritten: flag parsing, orchestration
│   ├── do_init.rs             # rewritten: flag parsing, orchestration
│   └── deploy_check.rs        # NEW: git ls-remote pre-flight
├── project.rs                 # NEW (or templates/project.rs extended):
│                              #   package_name(), find_project_root(),
│                              #   read_bins(), read_workspace_members(),
│                              #   read_rust_toolchain(), detect_dirs()
├── deploy/                    # NEW module:
│   ├── mod.rs
│   ├── env_example.rs         # parse .env.example → Vec<EnvEntry>
│   ├── classify.rs            # is_secret(key) via suffix match
│   └── ferro_deps.rs          # path→git rewrite script generator
└── templates/
    ├── docker.rs              # rewritten renderers accepting context structs
    └── files/
        ├── docker/
        │   ├── Dockerfile.tpl           # rewritten with new placeholders
        │   ├── dockerignore.tpl         # updated with D-20 entries
        │   └── rewrite-ferro-deps.sh.tpl  # NEW
        └── do/
            └── app.yaml.tpl             # rewritten skeleton
```

### Pattern 1: Context struct per template
**What:** Instead of `dockerfile_template(package_name: &str)`, introduce `DockerfileContext { package_name, bins, has_frontend, has_themes, has_lang, has_public, has_migrations, runtime_deps, rust_toolchain, workspace_members, ferro_ref }` and render via `fn render(ctx: &DockerfileContext) -> String`.
**When to use:** Any template with ≥3 variable inputs or conditional sections.
**Example:**
```rust
// ferro-cli/src/templates/docker.rs (target shape)
pub struct DockerfileContext<'a> {
    pub package_name: &'a str,
    pub bins: &'a [BinEntry],                  // name + is_server
    pub has_frontend: bool,
    pub has_themes: bool,
    pub has_lang: bool,
    pub has_public: bool,
    pub has_migrations: bool,
    pub runtime_deps: &'a [String],            // e.g. ["chromium", "fonts-liberation"]
    pub rust_base_image: &'a str,              // resolved from rust-toolchain.toml
    pub workspace_members: &'a [String],       // relative paths to COPY in planner stage
    pub ferro_ref: &'a str,
}

pub fn render_dockerfile(ctx: &DockerfileContext) -> String { /* assemble */ }
```

### Pattern 2: Walk-up Cargo.toml discovery (D-17)
**What:** From CWD, ascend parents until a `Cargo.toml` containing `[package]` or `[workspace]` is found.
```rust
pub fn find_project_root() -> Result<PathBuf, io::Error> {
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir);
        }
        if !dir.pop() { return Err(io::Error::new(io::ErrorKind::NotFound, "Cargo.toml not found")); }
    }
}
```
This replaces the bare `Path::new("Cargo.toml").exists()` check in both current commands.

### Pattern 3: Preserved marker blocks for regeneration (D-03)
**What:** The `--runtime-deps` block in the runtime stage must be wrapped in marker comments so a future regeneration with the same flag produces the same output, and users who edit between markers know their edits will be overwritten.
```dockerfile
# >>> ferro:runtime-deps (regenerated by `ferro docker:init --runtime-deps=...`)
RUN apt-get update && apt-get install -y --no-install-recommends \
    chromium fonts-liberation \
    && rm -rf /var/lib/apt/lists/*
# <<< ferro:runtime-deps
```
Note: SCOPE.md says "without losing them on regeneration" — interpret as "the flag value is the source of truth, passed on each regeneration." Do **not** implement round-tripping (reading existing Dockerfile to preserve user edits); that is out of scope and violates "no hand patches" intent. Document the flag as idempotent given same inputs.

### Pattern 4: Workers derived from non-server bins (D-15)
**What:** A Ferro project's primary server binary is the one named equal to the package (or the sole bin). All other `[[bin]]` entries are workers.
```rust
fn classify_bins(package_name: &str, bins: &[BinEntry]) -> (BinEntry, Vec<BinEntry>) {
    // server = bin whose name == package_name, else first bin
    // workers = everything else
}
```

### Anti-Patterns to Avoid
- **Shelling out to `cargo metadata`** — adds startup latency and a hard PATH dependency; direct `toml` parsing is fine for `[[bin]]` and `[workspace] members`.
- **Parsing `.env.example` with `dotenvy`** — the file format is trivial (`KEY=value`, `# comment`), a 20-line hand parser is clearer than a dep.
- **Round-tripping existing Dockerfiles** — explicitly out of scope; `--force` overwrites.
- **Hardcoding the runtime base image** — must derive from `rust-toolchain.toml` when present (D-06).
- **Generating YAML via `format!`** with user-controlled values without YAML-escaping — env values containing `:` or `#` must be quoted. Use double-quoted scalars for env values always.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI arg parsing | hand-rolled `args.iter()` | `clap` derive (already in use — add fields to `Commands::DockerInit { force, runtime_deps, ferro_ref }` and `Commands::DoInit { repo, region, force, ferro_ref }`) | Consistency with all other ferro-cli commands |
| Cargo.toml parsing | regex | `toml` crate already in deps | Handles whitespace, comments, multi-line arrays |
| Colored output | ANSI escapes | `console::style` (already used) | Cross-platform, existing pattern |
| git remote check | HTTP to GitHub API | `std::process::Command::new("git").args(["ls-remote", ...])` | Works for any remote, no auth assumptions, cheap |
| Running shell scripts in Docker | embedding sed one-liners | standalone `scripts/rewrite-ferro-deps.sh` invoked as `RUN bash scripts/rewrite-ferro-deps.sh` | Testable in isolation on the host; readable |

**Key insight:** Every piece of machinery this phase needs already has a blessed ferro-cli pattern. The rewrite is mostly about decomposition (small functions, context structs, a `deploy/` module) not new infrastructure.

## Runtime State Inventory

This is a rewrite/refactor phase (replacing templates and commands), but NOT a rename — there are no stored strings, no runtime-registered identifiers, no cached data to migrate. Inventory by category:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — templates are compile-time `include_str!`, no DB or disk state owned by ferro-cli | None |
| Live service config | None — this phase affects template output consumed by user projects; user apps already deployed (gestiscilo, mkmenu) will regenerate their own Dockerfile/app.yaml manually as part of Verification | User apps regenerate on demand; no service migration |
| OS-registered state | None — no launchd/systemd/scheduler entries owned by ferro-cli | None |
| Secrets/env vars | `GITHUB_TOKEN` is a new `ARG` in generated Dockerfiles; users must pass it at `docker build` time. This is a **user-facing doc change**, not a code migration. | Document in `docs/src/` deploy guide |
| Build artifacts | Existing compiled `ferro` binaries continue to work; no egg-info equivalents | None |

**Canonical question — after every file is updated, what runtime systems still have the old shape cached?** Only user projects that already ran `ferro docker:init` — and those projects, per SCOPE.md Verification, are explicitly expected to delete + regenerate as the acceptance test.

## Common Pitfalls

### Pitfall 1: Cargo.toml `[[bin]]` can be implicit
**What goes wrong:** A project with `src/main.rs` and no `[[bin]]` section still has one binary (package-named). Multi-bin projects must have explicit `src/bin/*.rs` or `[[bin]]` entries.
**Why it happens:** Cargo's target auto-discovery.
**How to avoid:** When `[[bin]]` is empty, synthesize a single entry `{ name: package_name, path: "src/main.rs" }` before rendering. Also scan `src/bin/` for `*.rs` when `[[bin]]` is absent (cargo auto-discovery rules). For D-02 multi-bin support, document that `[[bin]]` entries are preferred; bare `src/bin/*.rs` is supported as a fallback.
**Warning signs:** gestiscilo/mkmenu would need to have both `screenshot-worker` and `gestiscilo` visible — verify whichever pattern they use before implementation.

### Pitfall 2: cargo-chef recipe cache invalidation on workspace members
**What goes wrong:** `cargo chef prepare` only sees what you `COPY` into the planner stage. If the Dockerfile copies `Cargo.toml + src/` but the workspace has `crates/foo`, chef produces an incomplete recipe and the builder re-downloads everything.
**Why it happens:** cargo-chef inspects `Cargo.lock` plus all member manifests referenced from the workspace root.
**How to avoid:** Read `[workspace] members` from the root `Cargo.toml`, copy each member's `Cargo.toml` (and `src/lib.rs` or `src/main.rs` stub — cargo-chef only needs manifests but cargo requires source files to exist for path deps to resolve). The standard cargo-chef workspace pattern is: `COPY Cargo.toml Cargo.lock ./` then for each member `COPY crates/foo/Cargo.toml crates/foo/Cargo.toml` and create a stub `src/lib.rs`. **Verify this against the cargo-chef docs** — MEDIUM confidence, recommend a cargo-chef docs reference during implementation.
**Warning signs:** Docker build time on second run is same as first run = chef cache isn't hitting.

### Pitfall 3: `git config insteadOf` requires the token to be set non-empty at build time
**What goes wrong:** Build fails with `Permission denied (publickey)` when `GITHUB_TOKEN` is empty.
**Why it happens:** `git config --global url."https://${GITHUB_TOKEN}@github.com/".insteadOf "git@github.com:"` with empty token produces `https://@github.com/` which git rejects differently.
**How to avoid:** Only emit the `git config` line when token is non-empty: wrap in `RUN if [ -n "$GITHUB_TOKEN" ]; then ...; fi` so public-repo builds still work. Document that `docker build --build-arg GITHUB_TOKEN=ghp_xxx .` is required when ferro deps are private.

### Pitfall 4: `.env.example` comments and multi-line values
**What goes wrong:** Parser treats `# comment` as a key or fails on `KEY="value with spaces"` or `KEY=` (empty).
**How to avoid:** Strip full-line comments (`^\s*#`), strip trailing inline comments only outside quotes (or don't support inline comments at all — simpler), trim whitespace, tolerate empty values, reject keys not matching `[A-Z_][A-Z0-9_]*`.

### Pitfall 5: DigitalOcean App Platform spec schema drift
**What goes wrong:** `type: SECRET` / `scope: RUN_TIME` / `databases.name` / `${db.DATABASE_URL}` interpolation syntax — DO's app spec evolves and historical docs show variations.
**How to avoid:** LOW confidence on these exact field names without fetching current DO docs. The planner MUST verify via DO's current app-spec reference (https://docs.digitalocean.com/products/app-platform/reference/app-spec/) or preferably read mkmenu's currently-deployed working `app.yaml` which is documented in CONTEXT.md as the ground truth. **Flag for validation at planning time.**

### Pitfall 6: `rust-toolchain.toml` can pin a channel, not a base image
**What goes wrong:** `rust-toolchain.toml` contains `channel = "1.88"` or `"stable"` or `"nightly-2024-12-01"` — you can't blindly interpolate it into `rust:{version}-slim-bookworm`.
**How to avoid:** Only substitute when the channel looks like a semver (`N.M` or `N.M.P`). For `stable`/`nightly`/date-stamped channels, fall back to the hardcoded `rust:1.88-slim-bookworm` and emit a warning. Alternatively use `rustup` inside the image to install the exact toolchain — but that defeats cargo-chef caching. Recommend: **semver only, warn otherwise**.

### Pitfall 7: gestiscilo/mkmenu reference apps not on disk
**What goes wrong:** SCOPE.md + CONTEXT.md reference `../../gestiscilo-it/app/` and `../../gestiscilo-it/mkmenu/` as ground truth, but these directories are **not present** at `/Users/alberto/repositories/albertogferrario/gestiscilo-it/`. The researcher could not cross-check current hand-patched Dockerfile/app.yaml shapes.
**How to avoid:** Before Wave 1 implementation, the executor must have access to these files (clone repos or confirm paths). Otherwise the only validation will be golden-file tests written against assumed shapes, and the Verification step will fail in unpredictable ways. **Flag as blocker for planning.**

## Code Examples

### Walking up to find Cargo.toml (D-17)
```rust
// ferro-cli/src/project.rs (new module)
use std::io;
use std::path::PathBuf;

pub fn find_project_root() -> io::Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Cargo.toml not found in current directory or any parent",
            ));
        }
    }
}
```

### Reading `[[bin]]` entries (D-02, D-15)
```rust
use toml::Value;

pub struct BinEntry {
    pub name: String,
    pub path: Option<String>,
}

pub fn read_bins(cargo_toml: &Value, package_name: &str) -> Vec<BinEntry> {
    if let Some(arr) = cargo_toml.get("bin").and_then(|v| v.as_array()) {
        return arr
            .iter()
            .filter_map(|entry| {
                let name = entry.get("name")?.as_str()?.to_string();
                let path = entry.get("path").and_then(|p| p.as_str()).map(String::from);
                Some(BinEntry { name, path })
            })
            .collect();
    }
    // Fallback: single implicit bin = package name
    vec![BinEntry { name: package_name.to_string(), path: None }]
}
```

### SECRET classification (D-13)
```rust
pub fn is_secret_env(key: &str) -> bool {
    let k = key.to_uppercase();
    k == "DATABASE_URL"
        || k.ends_with("_KEY")
        || k.ends_with("_SECRET")
        || k.contains("PASSWORD")
        || k.contains("TOKEN")
}
```

### `git ls-remote` pre-flight (D-11)
```rust
use std::process::Command;

pub fn ferro_ref_exists_on_remote(ferro_remote: &str, git_ref: &str) -> bool {
    let output = Command::new("git")
        .args(["ls-remote", "--exit-code", ferro_remote, git_ref])
        .output();
    matches!(output, Ok(o) if o.status.success())
}
```

### Workspace member discovery (D-07)
```rust
pub fn read_workspace_members(root_cargo: &Value) -> Vec<String> {
    root_cargo
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}
```

Note: `members` can contain glob patterns like `crates/*`. The copy logic must expand globs against the filesystem — use `std::fs::read_dir` for the trailing `*` segment, keep it simple; reject `**` recursive globs for this phase.

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Single static Dockerfile template, string replace of one placeholder | Context struct + assembled conditional blocks | Supports N binaries, N path deps, optional stages |
| `get_package_name()` duplicated in two commands | Shared `project::package_name()` | DRY; future CLI commands reuse |
| `Path::new("Cargo.toml").exists()` | `find_project_root()` walk-up | Works from any subdirectory |
| No pre-flight | `ferro deploy:check` gated on `git ls-remote` | Catches "I forgot to push" before Docker builds 10 minutes of deps |
| Manual `.dockerignore` editing post-generation | `.planning/`, `storage/`, `data/`, `database.db`, `*.sqlite*` baked in | One less foot-gun per deploy |

**Deprecated/outdated:**
- The `dockerfile_template(package_name)` signature — replace with context struct.
- Hardcoded `rust:1.88-slim-bookworm` — honor `rust-toolchain.toml`.

## Open Questions

1. **How does gestiscilo actually declare its two binaries?**
   - What we know: SCOPE.md says `gestiscilo + screenshot-worker`, 4 ferro path deps.
   - What's unclear: whether they are `[[bin]]` array entries or `src/bin/screenshot-worker.rs` auto-discovery.
   - Recommendation: Blocker — planner should require the gestiscilo repo be cloned/accessible before Wave 1. Fall back to supporting **both** representations in `read_bins()`.

2. **Does mkmenu's currently-deployed `app.yaml` use `type: SECRET` or `type: secret`, `scope: RUN_TIME` or `scope: runtime`?**
   - What we know: DO docs have evolved; CONTEXT.md says mkmenu is the ground truth.
   - What's unclear: exact casing and field names in the currently-working spec.
   - Recommendation: Planner fetches DO App Platform spec reference docs + reads mkmenu's live app.yaml before finalizing template. HIGH priority for Task P1.

3. **Where should `ferro deploy:check` live when invoked from `docker:build`?**
   - What we know: SCOPE.md says pre-flight blocks `docker:build`.
   - What's unclear: `docker:build` command does not exist in the current CLI (grep for `DockerBuild` returned nothing). Is it a future command or an existing alias?
   - Recommendation: Treat `deploy:check` as a **standalone command first**. Wire it into `docker:build` only if that command exists; otherwise document the integration point for Phase 123 and surface a planning note.

4. **Should `rewrite-ferro-deps.sh` edit `Cargo.toml` in place or emit a patch?**
   - What we know: D-09 says invoked in planner and builder stages after `COPY .`.
   - What's unclear: cargo-chef's planner runs `cargo chef prepare` which reads `Cargo.toml`. So the rewrite must happen **before** `cargo chef prepare`, in-place. Confirm: script writes new `Cargo.toml` to the Docker build context (not the host), leaving the host path deps untouched.
   - Recommendation: in-place `sed -i` on the Docker-side copy. Script must be idempotent (running twice should produce the same file).

5. **`ferro deploy:check` — which remote URL does it probe?**
   - What we know: Must match the ferro git dep URL that will be baked into the generated `Cargo.toml`.
   - What's unclear: Is there a single canonical ferro remote, or does the user configure it?
   - Recommendation: Derive from `--ferro-ref` context — the script generator already knows the remote URL because it writes it into `Cargo.toml`. Pass both to `deploy_check`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build + test this crate | ✓ (assumed) | — | — |
| `git` CLI | `ferro deploy:check` runtime | ✓ (assumed present on any dev machine) | — | If `git` missing, deploy:check fails with clear message pointing to install |
| `docker` | Acceptance test (regenerate + build gestiscilo/mkmenu) | ✗ (not probed; user-side) | — | N/A — only needed for Verification step, run by user |
| gestiscilo repo (`../../gestiscilo-it/app/`) | Verification step | ✗ — directory missing at `/Users/alberto/repositories/albertogferrario/gestiscilo-it/` | — | **BLOCKER** — planner/executor must clone or confirm alternate path before Wave N verification |
| mkmenu repo (`../../gestiscilo-it/mkmenu/`) | Verification step | ✗ — same as above | — | **BLOCKER** — same |
| DO App Platform spec docs | D-12, D-13, D-14, D-15 validation | ✓ (WebFetch to docs.digitalocean.com) | — | — |

**Missing dependencies with no fallback:**
- gestiscilo and mkmenu reference apps — Verification section of SCOPE.md cannot execute without them. Planner must either (a) clone them into a known location, (b) obtain copies of their current `Dockerfile` and `.do/app.yaml` files to use as golden-file test fixtures, or (c) descope verification to golden-file tests only and run end-to-end manually on a machine that has the repos.

**Missing dependencies with fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` via `cargo test` |
| Config file | none — `Cargo.toml` per crate |
| Quick run command | `cargo test -p ferro-cli` |
| Full suite command | `cargo test --all-features` |

Existing ferro-cli tests live alongside modules (e.g. `templates/mod.rs:533` already has `test_dockerfile_template_substitution` and `test_dockerignore_template_not_empty`). Pattern: inline `#[cfg(test)] mod tests { ... }` blocks.

### Phase Requirements → Test Map

Since there are no REQ-IDs, this maps SCOPE.md Verification bullets + key decisions to tests:

| SCOPE Item | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| D-01 | Frontend stage absent when no `frontend/package.json` | unit/golden | `cargo test -p ferro-cli docker::render_no_frontend` | Wave 0 |
| D-01 | Frontend stage present when `frontend/package.json` exists | unit/golden | `cargo test -p ferro-cli docker::render_with_frontend` | Wave 0 |
| D-02 | Multi-bin emits one `--bin` per entry + runtime COPY per bin | unit/golden | `cargo test -p ferro-cli docker::multi_bin_dockerfile` | Wave 0 |
| D-02 | Single implicit bin works (no `[[bin]]` section) | unit | `cargo test -p ferro-cli project::read_bins_implicit` | Wave 0 |
| D-03 | `--runtime-deps` renders apt-install block between markers | unit/golden | `cargo test -p ferro-cli docker::runtime_deps_block` | Wave 0 |
| D-04 | `themes/`/`lang/`/`public/`/`migrations/` copy lines appear only when dirs exist | unit/golden | `cargo test -p ferro-cli docker::detect_copy_dirs` | Wave 0 |
| D-05 | `ARG GITHUB_TOKEN` + `git config insteadOf` present | unit/golden | `cargo test -p ferro-cli docker::github_token_arg` | Wave 0 |
| D-06 | rust-toolchain.toml channel `1.89.0` → `rust:1.89-slim-bookworm`; `stable` → fallback | unit | `cargo test -p ferro-cli project::resolve_rust_base_image` | Wave 0 |
| D-07 | Workspace members copied in planner stage | unit/golden | `cargo test -p ferro-cli docker::workspace_members_copied` | Wave 0 |
| D-08 | `scripts/rewrite-ferro-deps.sh` generated with `--ferro-ref` header | unit/golden | `cargo test -p ferro-cli deploy::rewrite_script_header` | Wave 0 |
| D-09 | Dockerfile invokes script in planner and builder stages | unit/golden | `cargo test -p ferro-cli docker::invokes_rewrite_script` | Wave 0 |
| D-10 | `--ferro-ref main` default; flag propagates to script | unit | `cargo test -p ferro-cli cli::ferro_ref_default` | Wave 0 |
| D-11 | `deploy:check` returns error when `git ls-remote` fails | integration | `cargo test -p ferro-cli deploy_check::missing_ref` | Wave 0 (needs temp git repo fixture) |
| D-12 | `--region nyc` propagates to `app.yaml` | unit/golden | `cargo test -p ferro-cli do::region_flag` | Wave 0 |
| D-13 | `DATABASE_URL` → SECRET; `APP_ENV` → plain | unit | `cargo test -p ferro-cli deploy::classify_env` | Wave 0 |
| D-14 | `databases:` block emitted when `DATABASE_URL` in env.example | unit/golden | `cargo test -p ferro-cli do::database_block` | Wave 0 |
| D-15 | One `workers:` entry per non-server `[[bin]]` | unit/golden | `cargo test -p ferro-cli do::workers_from_bins` | Wave 0 |
| D-16 | `--force` overwrites existing files | unit | `cargo test -p ferro-cli docker_init::force_flag` | Wave 0 |
| D-17 | `find_project_root()` ascends from subdirectory | unit | `cargo test -p ferro-cli project::find_root_walks_up` | Wave 0 |
| D-18 | `--repo foo` (no slash) rejected; `--repo a/b` accepted | unit | `cargo test -p ferro-cli do::validate_repo_format` | Wave 0 |
| D-19 | `project::package_name()` returns package name | unit | `cargo test -p ferro-cli project::package_name` | Wave 0 |
| D-20 | `.dockerignore` contains all new entries | unit | `cargo test -p ferro-cli docker::dockerignore_entries` | Wave 0 |
| Verification-1 | Regenerated gestiscilo Dockerfile matches golden fixture | golden | `cargo test -p ferro-cli golden::gestiscilo` | Wave 0 (fixture from gestiscilo repo) |
| Verification-2 | Regenerated mkmenu Dockerfile matches golden fixture | golden | `cargo test -p ferro-cli golden::mkmenu` | Wave 0 (fixture from mkmenu repo) |
| Verification-3 | End-to-end `docker build` on gestiscilo succeeds | manual-only | documented in Verification section | manual |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-cli` (fast, < 10s).
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
- **Phase gate:** Full suite green + manual Verification run of `ferro docker:init` + `ferro do:init` against gestiscilo and mkmenu repos.

### Wave 0 Gaps
- [ ] `ferro-cli/tests/fixtures/` — golden files for each rendered scenario (Dockerfile, dockerignore, app.yaml, rewrite-ferro-deps.sh).
- [ ] `ferro-cli/tests/fixtures/gestiscilo/Cargo.toml` + `.env.example` — synthetic or copied from real gestiscilo repo for golden-file comparison.
- [ ] `ferro-cli/tests/fixtures/mkmenu/` — same for mkmenu.
- [ ] Test helper for temp `git init` + `git ls-remote` fixture (deploy_check integration test).
- [ ] No framework install needed — `#[test]` is built in.

## Sources

### Primary (HIGH confidence — internal codebase)
- `ferro-cli/src/commands/docker_init.rs` — current implementation (read in full)
- `ferro-cli/src/commands/do_init.rs` — current implementation (read in full)
- `ferro-cli/src/templates/docker.rs` — template renderer functions
- `ferro-cli/src/templates/files/docker/Dockerfile.tpl` — current Dockerfile template
- `ferro-cli/src/templates/files/docker/dockerignore.tpl` — current dockerignore
- `ferro-cli/src/templates/files/do/app.yaml.tpl` — current app.yaml
- `ferro-cli/src/templates/project.rs` — scaffolding template patterns (reference)
- `ferro-cli/src/main.rs:322+,570+` — CLI dispatcher for `DockerInit`/`DoInit`
- `ferro-cli/Cargo.toml` — confirms `clap` 4 derive + `toml` 0.8 already present
- `Cargo.toml` — workspace member list (20 crates)
- `.planning/phases/122-deploy-scaffold-core-rewrite/SCOPE.md` — authoritative scope
- `.planning/phases/122-deploy-scaffold-core-rewrite/122-CONTEXT.md` — locked decisions
- `./CLAUDE.md`, `~/.claude/CLAUDE.md` — project constraints and conventions

### Secondary (MEDIUM confidence — needs planner verification)
- cargo-chef workspace recipe pattern — widely documented but details vary by version; planner should cross-check with current cargo-chef README.
- DigitalOcean App Platform spec field names (`type: SECRET`, `scope: RUN_TIME`, `${db.DATABASE_URL}`) — planner should fetch current DO docs.

### Tertiary (LOW confidence — unverified)
- Exact gestiscilo and mkmenu Dockerfile/app.yaml shapes — reference repos not accessible on disk at the expected path.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps already in `ferro-cli/Cargo.toml`, patterns verified in current code.
- Architecture: HIGH — context-struct pattern is a natural extension of existing `include_str!` renderers.
- Pitfalls: MEDIUM — cargo-chef workspace pattern, DO spec field casing, and gestiscilo bin declaration shape all need verification at planning time.
- Environment: LOW — gestiscilo/mkmenu repos not present; Verification section of SCOPE.md cannot be executed without them.

**Research date:** 2026-04-07
**Valid until:** 2026-05-07 (stable domain, but DO App Platform spec and cargo-chef are fast-moving enough to warrant a fresh check if planning is delayed).
