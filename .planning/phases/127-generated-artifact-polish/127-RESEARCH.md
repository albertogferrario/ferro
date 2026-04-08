# Phase 127: Generated artifact polish — Research

**Researched:** 2026-04-08
**Domain:** `ferro-cli` deploy scaffold (templates, command wiring, toml rewriting)
**Confidence:** HIGH

## Summary

Phase 127 is a tightly-scoped QoL pass over the `docker:init` / `do:init` generators.
Every change lives under `ferro-cli/src/{templates,commands,deploy}/`. The only
non-trivial piece is the `toml` → `toml_edit` migration in `rewrite_ferro_version.rs`;
everything else is straight template/token work, a new shared bin-detection helper, a
secret-classification heuristic, a `--dry-run` plumb-through, and two footers. All 21
CONTEXT decisions (D-01..D-21) are already locked, so research is confirmatory.

**Primary recommendation:** Land as 3 waves — (1) shared extraction + toml_edit swap,
(2) template/token changes (ENTRYPOINT, envs block, dockerignore, build dedupe), (3)
command wiring (`--dry-run`, footer). Waves 1 and 2 have no ordering dependency
between themselves; wave 3 depends on both.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Dockerfile entrypoint (item 18)**
- **D-01:** Generated `Dockerfile.tpl` MUST emit both `ENTRYPOINT` and `CMD` lines so the image is runnable with no extra arguments.
- **D-02:** Bin selection reuses the **same `web_bin` detection** that `do:init` already uses (`templates/do.rs`), so the Dockerfile ENTRYPOINT and the DO web service stay in sync by construction. Detection order:
  1. `[package.metadata.ferro.deploy].web_bin` if explicitly set
  2. The bin matching the `package.name` (single-bin or matching multi-bin)
  3. The first declared `[[bin]]`
  4. Fall back to package name if no `[[bin]]` is declared
- **D-03:** Emit `ENTRYPOINT ["/usr/local/bin/<bin>"]` and `CMD ["serve"]`. Regenerate-on-`--force` only (Phase 122.2 §2).
- **D-04:** Add a new template token (e.g. `{{ENTRYPOINT}}`) wired through `templates/docker.rs`. Token block at the bottom of the runtime stage.

**DO `web` service (item 18 corollary)**
- **D-05:** `do:init` does NOT add a `run_command:` to the `web` service. Dockerfile ENTRYPOINT is single source of truth. Document inline with a one-line comment.

**`do:init` env entries (item 16)**
- **D-06:** Replace comment-only `envs:` block with real entries derived from `.env.example`. `value: ""` left empty but structurally `doctl apps update`-ready.
- **D-07:** Secret keys: `type: SECRET` + `scope: RUN_AND_BUILD_TIME`. Non-secret: `scope: RUN_TIME`, no explicit type.
- **D-08:** Secret heuristic — case-insensitive substring match against `{secret, password, passwd, token, key, api_key, dsn, private, credential}`. `_URL` suffix is non-secret unless it also matches (e.g. `DATABASE_URL` non-secret, `STRIPE_SECRET_KEY` secret).
- **D-09:** Order matches `.env.example` order. Skip blank/comment lines, preserve blank-line separators where source had one.

**Build dedupe (item 6)**
- **D-10:** Remove per-bin `cargo build --release --bin <name>` lines. Keep only the single `cargo build --release` invocation. Empty or remove `{{BIN_BUILDS}}` token.

**Dep table ordering (item 5)**
- **D-11:** Switch `ferro-cli/src/deploy/rewrite_ferro_version.rs` from `toml` to `toml_edit`. Must not reorder sibling tables or sibling keys.
- **D-12:** Existing regression tests MUST keep passing. Add new `preserves_dep_table_order` test.

**"Next steps" footer (item 7)**
- **D-13:** Both commands print a 3-5 line footer after success, stdout, cargo-style, no emoji.
- **D-14:** `docker:init` footer: `docker build -t <name>:test .` and `docker run --rm -p 8080:8080 --env-file .env.production <name>:test`.
- **D-15:** `do:init` footer: review `.do/app.yaml`, populate envs (dashboard or `doctl apps update <id> --spec .do/app.yaml`), first-deploy `doctl apps create --spec .do/app.yaml`.
- **D-16:** Footer suppressed in `--dry-run`.

**`--dry-run` flag (item 9)**
- **D-17:** Both commands accept `--dry-run`. Render every file to string, print `--- <relative/path> ---` header per file, exit 0, no filesystem writes.
- **D-18:** Short-circuits BEFORE `Cargo.docker.toml` rewrite is persisted, but the rewrite is still computed in memory and printed.
- **D-19:** Exit 0 on successful render; non-zero only if rendering itself fails. `--dry-run` does NOT soften rendering errors.

**`.dockerignore` README warning (item 10)**
- **D-20:** Whitelist `README.md` in `.dockerignore` — add `!README.md` after the `*.md` exclusion.
- **D-21:** One-line comment explaining why.

### Claude's Discretion
- Exact footer wording (cargo-style, no emoji).
- Name of the new ENTRYPOINT token (`{{ENTRYPOINT}}` / `{{ENTRYPOINT_BLOCK}}` / split).
- Extract secret heuristic to helper module vs inline. **Recommended: extract** (Phase 128 preflight will reuse).
- Test layout: extend existing vs new integration file. **Recommended: extend existing** (`templates/docker.rs`, `templates/do.rs`, `deploy/rewrite_ferro_version.rs`, `commands/*`).

### Deferred Ideas (OUT OF SCOPE)
- Preflight checks (items 3, 4, 12, 13, 17) — Phase 128.
- `ferro deploy:init` interactive scaffolder (item 15) — Phase 128.
- Publish workflow gating, per-crate version overrides (items 8, 14) — Phase 129.
- `gsd-tools` phase-numbering bug (item 11) — wrong repo.
- Per-crate `ferro_version` overrides (item 14 long form) — do not speculate.
</user_constraints>

## Project Constraints (from CLAUDE.md)

- Pre-commit command (CI-matching): `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
- No co-author lines in commits.
- Delete old code when replacing — no versioned function names.
- Keep changes focused and minimal; prefer editing existing files.
- Templates already use `{{TOKEN}}` substitution — follow the convention.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| D-01..D-04 | Dockerfile ENTRYPOINT/CMD with bin detection | Shared `web_bin` helper (§2), new token in `Dockerfile.tpl` after `EXPOSE 8080` |
| D-05 | DO web service omits `run_command` | `app.yaml.tpl` already has no `run_command` on web — only add inline comment |
| D-06..D-09 | Real `envs:` block from `.env.example` with secret typing | New renderer in `templates/do.rs`, reuses `read_env_production_keys`-style parser against `.env.example` |
| D-10 | Drop per-bin build | `templates/docker.rs::render_dockerfile` lines 48-52 compute `{{BIN_BUILDS}}` — remove |
| D-11, D-12 | `toml_edit` dep-order preservation | Direct dep must be added to `ferro-cli/Cargo.toml` |
| D-13..D-16 | Footers on both commands | Add after success `println!` in `commands/docker_init.rs::execute` and `commands/do_init.rs::run_inner` |
| D-17..D-19 | `--dry-run` flag | Plumb `dry_run: bool` through `execute` / `run_inner`; render functions are already pure |
| D-20, D-21 | `!README.md` whitelist | Static edit to `dockerignore.tpl` |

## Current State of Touched Files

### `ferro-cli/src/templates/files/docker/Dockerfile.tpl` (34 lines)
- Ends at line 33 with `EXPOSE 8080`. No ENTRYPOINT/CMD. **Append** new token slot after line 33.
- `{{BIN_BUILDS}}` token at line 23 — drop per D-10.

### `ferro-cli/src/templates/files/docker/dockerignore.tpl` (59 lines)
- Line 54: `*.md`. Insert `!README.md` + comment after this line per D-20/D-21.

### `ferro-cli/src/templates/files/do/app.yaml.tpl` (22 lines)
- Line 21: `{{ENV_COMMENTS}}` → rename to `{{ENVS_BLOCK}}`. Strip the "Set values in DO dashboard…" comment above it (replaced by structured entries).
- Add one-line comment on the `web` service stanza per D-05 explaining why there is no `run_command`.

### `ferro-cli/src/templates/docker.rs` (~360 lines)
- `DockerContext` struct (lines 26-38): add `entrypoint_bin: String` field.
- `render_dockerfile` (lines 41-96): drop `bin_builds` computation (lines 48-53); add `entrypoint` composition block; wire `{{ENTRYPOINT}}` replace; remove `{{BIN_BUILDS}}` replace.
- `read_bins` (lines 128-157) is already perfect for bin enumeration — keep.
- Tests at lines 237-250 (`multi_bin_emits_per_bin_build_and_copy`) will fail after D-10 — rewrite to assert only COPY lines.

### `ferro-cli/src/templates/do.rs` (~280 lines)
- `AppYamlContext` (lines 9-21): add no new fields (env_keys becomes `env_entries: Vec<EnvEntry>` or similar). Simplest: introduce `pub env_entries: Vec<(String, bool /* is_secret */)>` and keep `env_keys` deleted.
- `render_app_yaml` (lines 24-32): replace `{{ENV_COMMENTS}}` with `{{ENVS_BLOCK}}`.
- `render_env_comments` (lines 61-69): replace with `render_envs_block` that emits full YAML entries per D-06..D-09.
- **Bin-detection extraction** — `do_init.rs` lines 33-38 currently own the `web_bin` selection logic, NOT `templates/do.rs`. See §2 below.

### `ferro-cli/src/commands/docker_init.rs` (77 lines)
- `run`, `run_with`, `execute` — thread `dry_run: bool`.
- `write_if_absent_or_force` (line 65): skip in dry-run.
- `execute` line 56: `rewrite_cargo_docker_toml` currently writes unconditionally. D-18 requires splitting "compute rewritten string" from "persist" — add a sibling `compute_cargo_docker_toml(root, override_str) -> anyhow::Result<String>` and have the old function be a thin wrapper that calls compute + `fs::write`.
- Add footer after the success `println!` (line 58) per D-13/D-14.

### `ferro-cli/src/commands/do_init.rs` (143 lines)
- `run`, `run_inner` — thread `dry_run: bool`.
- `run_inner` lines 34-38: the **web_bin detection** happens here. Extract to shared helper (§2).
- `write_with_force` (line 82): skip in dry-run.
- Read `.env.example` instead of (or in addition to — see §4) `.env.production` keys for the envs block. **CONTEXT D-09 is explicit: source is `.env.example`, not `.env.production`.** This is a behavioral change from the current code.
- Add footer per D-13/D-15.

### `ferro-cli/src/deploy/rewrite_ferro_version.rs` (325 lines)
- Currently uses `toml` crate (`use toml::{Value, map::Map}`). Full rewrite of the mutation loop to `toml_edit::DocumentMut`. See §3.

### `ferro-cli/src/deploy/env_production.rs` (77 lines)
- `read_env_production_keys` trims blank/comment lines and returns only keys, no blank-line separators. Does NOT preserve blank-line grouping required by D-09. See §4.

### `ferro-cli/Cargo.toml`
- Has `toml = "0.8"` (line 37). Does NOT have `toml_edit`. Must add `toml_edit = "0.22"` (current). Keep `toml` — still used by `templates/docker.rs::read_rust_channel`, `read_bins`, and elsewhere. Don't rip it out.

### `ferro-cli/src/main.rs`
- Clap definitions at lines 343-358. Add `#[arg(long)] dry_run: bool` to both `DockerInit` and `DoInit` variants. Match arms at lines 628-636 pass through.

## §2 Bin Detection Extraction

Current location: `commands/do_init.rs` lines 33-38:
```rust
let bins = read_bins(&root)?;
let web_bin = bins
    .iter()
    .find(|b| **b == name || **b == pkg)
    .cloned()
    .unwrap_or_else(|| name.clone());
```

This is missing step 1 of CONTEXT's D-02 detection order (`[package.metadata.ferro.deploy].web_bin` explicit override). **Phase 127 must add step 1** while extracting — it is part of D-02, not an extra feature.

**Recommendation:** new module `ferro-cli/src/deploy/bin_detect.rs`:

```rust
pub struct BinDetectInput<'a> {
    pub explicit_web_bin: Option<&'a str>,  // from metadata.ferro.deploy.web_bin
    pub package_name: &'a str,
    pub bins: &'a [String],
}

pub fn detect_web_bin(input: BinDetectInput<'_>) -> String {
    // 1. explicit override
    if let Some(b) = input.explicit_web_bin {
        return b.to_string();
    }
    // 2. bin matching package name
    if let Some(m) = input.bins.iter().find(|b| b.as_str() == input.package_name) {
        return m.clone();
    }
    // 3. first declared [[bin]]
    if let Some(first) = input.bins.first() {
        return first.clone();
    }
    // 4. fall back to package name
    input.package_name.to_string()
}
```

Pure, easily unit-tested, zero I/O. Both `commands/do_init.rs` and `commands/docker_init.rs` call it. `read_deploy_metadata` already exists (`project::read_deploy_metadata`) — need to verify it exposes `web_bin`; if not, add the field.

**Check before planning:** does `DeployMetadata` already have a `web_bin: Option<String>`? If not, that's a small addition to `project.rs` (out of scope per §1 of CONTEXT but unavoidable to honor D-02 step 1).

## §3 `toml_edit` Migration

**Is `toml_edit` already in `ferro-cli/Cargo.toml`?** No — only transitive via Cargo.lock. Add as direct dep.

**Is `toml` used elsewhere in ferro-cli?** Yes — `templates/docker.rs::read_rust_channel` (line 114) and `read_bins` (line 132) use `toml::Value`. Leave those alone. Only `rewrite_ferro_version.rs` migrates.

**Minimal `toml_edit` API for the rewrite:**

```rust
use toml_edit::{DocumentMut, Item, Table, value};

let mut doc: DocumentMut = content.parse()?;

for table_name in DEP_TABLES {
    let Some(deps) = doc.get_mut(table_name).and_then(|i| i.as_table_mut()) else {
        continue;
    };
    // Collect ferro* keys first to avoid borrow conflicts
    let ferro_keys: Vec<String> = deps
        .iter()
        .filter(|(k, v)| {
            k.starts_with("ferro")
                && v.as_inline_table().map(|t| t.contains_key("path")).unwrap_or(false)
                || v.as_table().map(|t| t.contains_key("path")).unwrap_or(false)
        })
        .map(|(k, _)| k.to_string())
        .collect();

    for key in ferro_keys {
        let dep_item = deps.get_mut(&key).unwrap();
        // Mutate in place: remove "path", insert/overwrite "version",
        // leave other keys (package, features, default-features, optional,
        // registry, rename) in their original positions.
        if let Some(t) = dep_item.as_table_mut() {
            t.remove("path");
            t["version"] = value(new_version.clone());
        } else if let Some(t) = dep_item.as_inline_table_mut() {
            t.remove("path");
            t.insert("version", new_version.clone().into());
        }
    }
}

let serialized = doc.to_string();  // Preserves whitespace, order, comments
```

**Key property:** `toml_edit` preserves insertion order, whitespace, and comments. In-place mutation of an existing dep table (remove `path`, set `version`) keeps every other key where the user wrote it. This is exactly what D-11 requires.

**Gotcha — inline vs expanded tables:** `ferro = { path = "...", features = [...] }` parses as an **inline table**. `[dependencies.ferro]` with body parses as a regular table. The mutation code must handle both; above sketch does. Existing tests use the inline form.

**Gotcha — `version` insertion position:** `toml_edit` appends new keys at the end of the table by default. To keep the visual convention "version first", insert `version` and then the existing keys preserved naturally after it — OR just accept "version appended after preserved keys" since the decision bar is "preserve existing order", not "version first". The simplest path: set `version` directly with `t["version"] = value(...)`; if `version` didn't exist before it lands at the end. This is fine and keeps the code one-liner-per-key simple.

**Version to pin:** `toml_edit = "0.22"` (stable API, in workspace's transitive deps already, matches `toml 0.8` era). Verify with `cargo tree -p ferro-cli | grep toml_edit` before pinning.

## §4 `.env.example` Parser Reuse

`deploy/env_production.rs::read_env_production_keys` walks lines, skips blank/comment, returns keys only. **It does NOT preserve blank-line separators.** D-09 requires preserving them ("keep a blank-line separator in the output where the source had one").

**Options:**
- **(a) Extend the existing parser** with a richer return type (`Vec<EnvLine>` where `EnvLine = Key(String) | BlankSeparator`) and adapt `do:init`'s comment-only usage of the old function. The comment-only call site (line 52 of `do_init.rs`) becomes a `.iter().filter_map()` over the new enum.
- **(b) Add a sibling function** `read_env_example_lines(path) -> Vec<EnvLine>` and leave `read_env_production_keys` untouched.

**Recommendation: (a)**. The old function has one caller. CLAUDE.md says "delete old code when replacing — no versioned function names". Replace in place, update the one call site. New signature:

```rust
pub enum EnvLine {
    Key(String),
    BlankSeparator,
}
pub fn read_env_lines(path: &Path) -> anyhow::Result<Vec<EnvLine>>;
```

**Source path change:** Also note D-09 says "matches `.env.example` order". Current `do:init` reads `.env.production`. D-06/D-09 point to `.env.example` as the source. **This is a source-file change, not just a parser change.** The planner must decide:
- Switch `do:init` to read `.env.example` (structural keys always present in repo), OR
- Keep reading `.env.production` (current behavior, user must have it).

CONTEXT D-09 is explicit: **`.env.example`**. That's the contract. `.env.production` existence check can stay as a separate precondition if we want to keep the friendly error, but the source of the key list is `.env.example`.

## §5 Secret Heuristic Dry Run Against `env.example.tpl`

Substring set from D-08 (case-insensitive): `{secret, password, passwd, token, key, api_key, dsn, private, credential}`. Exception: `_URL` suffix is non-secret unless it also matches the set.

Walk of keys in `ferro-cli/src/templates/files/root/env.example.tpl`:

| Key | Classification | Rationale |
|-----|----------------|-----------|
| `APP_NAME` | non-secret | no hit |
| `APP_ENV` | non-secret | no hit |
| `APP_DEBUG` | non-secret | no hit |
| `APP_URL` | non-secret | `_URL`, no hit in set |
| `APP_LOCALE` | non-secret | no hit |
| `APP_FALLBACK_LOCALE` | non-secret | no hit |
| `LANG_PATH` | non-secret | no hit |
| `SERVER_HOST` | non-secret | no hit |
| `SERVER_PORT` | non-secret | no hit |
| `SERVER_MAX_BODY_SIZE` | non-secret | no hit |
| `VITE_PORT` | non-secret | no hit |
| `CARGO_SWEEP_DAYS` | non-secret | no hit |
| `DATABASE_URL` | non-secret | `_URL`, no hit |
| `DB_MAX_CONNECTIONS` | non-secret | no hit |
| `DB_MIN_CONNECTIONS` | non-secret | no hit |
| `DB_CONNECT_TIMEOUT` | non-secret | no hit |
| `DB_LOGGING` | non-secret | no hit |
| `SESSION_LIFETIME` | non-secret | no hit |
| `SESSION_ABSOLUTE_LIFETIME` | non-secret | no hit |
| `SESSION_COOKIE` | non-secret | no hit |
| `SESSION_SECURE` | non-secret | no hit |
| `SESSION_PATH` | non-secret | no hit |
| `SESSION_SAME_SITE` | non-secret | no hit |
| `REDIS_URL` | non-secret | `_URL`, no hit |
| `REDIS_HOST` | non-secret | no hit |
| `REDIS_PORT` | non-secret | no hit |
| `REDIS_PASSWORD` | **SECRET** | `password` hit |
| `REDIS_DATABASE` | non-secret | no hit |
| `CACHE_DRIVER` | non-secret | no hit |
| `CACHE_PREFIX` | non-secret | no hit |
| `CACHE_TTL` | non-secret | no hit |
| `CACHE_MEMORY_CAPACITY` | non-secret | no hit |
| `REDIS_PREFIX` | non-secret | no hit |
| `CACHE_DEFAULT_TTL` | non-secret | no hit |
| `QUEUE_CONNECTION` | non-secret | no hit |
| `QUEUE_DEFAULT` | non-secret | no hit |
| `QUEUE_PREFIX` | non-secret | no hit |
| `QUEUE_BLOCK_TIMEOUT` | non-secret | no hit |
| `QUEUE_MAX_CONCURRENT` | non-secret | no hit |
| `FILESYSTEM_DISK` | non-secret | no hit |
| `FILESYSTEM_LOCAL_ROOT` | non-secret | no hit |
| `FILESYSTEM_LOCAL_URL` | non-secret | `_URL`, no hit |
| `FILESYSTEM_PUBLIC_ROOT` | non-secret | no hit |
| `FILESYSTEM_PUBLIC_URL` | non-secret | `_URL`, no hit |
| `AWS_ACCESS_KEY_ID` | **SECRET** | `key` hit |
| `AWS_SECRET_ACCESS_KEY` | **SECRET** | `secret` + `key` hit |
| `AWS_DEFAULT_REGION` | non-secret | no hit |
| `AWS_BUCKET` | non-secret | no hit |
| `AWS_URL` | non-secret | `_URL`, no hit |
| `BROADCAST_MAX_SUBSCRIBERS` | non-secret | no hit |
| `BROADCAST_MAX_CHANNELS` | non-secret | no hit |
| `BROADCAST_HEARTBEAT_INTERVAL` | non-secret | no hit |
| `BROADCAST_CLIENT_TIMEOUT` | non-secret | no hit |
| `BROADCAST_ALLOW_CLIENT_EVENTS` | non-secret | no hit |
| `MAIL_DRIVER` | non-secret | no hit |
| `MAIL_HOST` | non-secret | no hit |
| `MAIL_PORT` | non-secret | no hit |
| `MAIL_USERNAME` | non-secret | no hit |
| `MAIL_PASSWORD` | **SECRET** | `password` hit |
| `MAIL_FROM_ADDRESS` | non-secret | no hit |
| `MAIL_FROM_NAME` | non-secret | no hit |
| `MAIL_ENCRYPTION` | non-secret | no hit |
| `RESEND_API_KEY` | **SECRET** | `api_key` + `key` hit |
| `SLACK_WEBHOOK_URL` | non-secret | `_URL`, no hit **— ⚠ FALSE NEGATIVE** |
| `FERRO_DEBUG_ENDPOINTS` | non-secret | no hit |
| `FERRO_COLLECT_METRICS` | non-secret | no hit |
| `ANTHROPIC_API_KEY` | **SECRET** | `api_key` + `key` hit |
| `FERRO_AI_MODEL` | non-secret | no hit |

**Findings:**
- **True positives:** `REDIS_PASSWORD`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `MAIL_PASSWORD`, `RESEND_API_KEY`, `ANTHROPIC_API_KEY` — all correctly classified as secrets.
- **False negative: `SLACK_WEBHOOK_URL`** — Slack webhook URLs are secrets (URL contains the auth token in-band), but heuristic rule D-08 treats any `_URL` as non-secret unless it also substring-matches the set. `webhook` is not in the set. This will emit `SLACK_WEBHOOK_URL` as a non-secret env, which is wrong.
- **No false positives** on this template.

**Recommendation for planner:** Either (a) expand D-08's set to include `webhook` — probably worth doing since webhook URLs are ubiquitously secret; or (b) document the false negative in the generated file's header comment and flag it for Phase 128 preflight. D-08 is locked; planner should flag this back to user before inventing a new heuristic. In the meantime, implement D-08 exactly as specified and add a test asserting the known `SLACK_WEBHOOK_URL` false-negative so future work to tighten it is visible.

## §6 `--dry-run` Plumbing

**Render purity:**
- `templates/docker.rs::render_dockerfile(&DockerContext) -> String` — pure.
- `templates/docker.rs::dockerignore_template() -> &'static str` — pure.
- `templates/do.rs::render_app_yaml(&AppYamlContext) -> String` — pure.
- `rewrite_ferro_version.rs::rewrite_cargo_docker_toml` — **impure** (writes `Cargo.docker.toml`). Per D-18, split into `compute_cargo_docker_toml(root, override) -> anyhow::Result<(PathBuf, String)>` + `persist_cargo_docker_toml(path, content)`. Existing public API keeps the old name as a wrapper that calls both.

**Minimal plumbing:**

```rust
// commands/docker_init.rs
pub fn run_with(force: bool, ferro_version: Option<String>, dry_run: bool) { ... }
fn execute(force: bool, ferro_version_flag: Option<&str>, dry_run: bool) -> Result<()> {
    // ... compute ctx, dockerfile, dockerignore, (path, cargo_docker_contents) ...
    if dry_run {
        print_dry_run("Dockerfile", &dockerfile);
        print_dry_run(".dockerignore", dockerignore_template());
        print_dry_run("Cargo.docker.toml", &cargo_docker_contents);
        return Ok(()); // no footer (D-16)
    }
    write_if_absent_or_force(...)?;
    persist_cargo_docker_toml(...)?;
    print_footer_docker_init(&package_name);
    Ok(())
}

fn print_dry_run(rel: &str, body: &str) {
    println!("--- {rel} ---");
    println!("{body}");
}
```

Same pattern in `do_init.rs`. Clap variants in `main.rs` gain `#[arg(long)] dry_run: bool` — zero friction.

## §7 Existing `--force` Flag Pattern

Already present on both commands (`docker_init.rs` line 15, `do_init.rs` line 18). Clap arg style (`#[arg(long)] force: bool`) is the template for `--dry-run`. Parallel and independent: `--force` is "may overwrite existing files"; `--dry-run` is "never writes, period". Combining `--force --dry-run` should print what WOULD overwrite without prompting for confirmation. No semantic conflict.

## Standard Stack

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `toml_edit` | 0.22 | Order-preserving TOML mutation | Sibling of `toml`, written by same author, the canonical choice for "edit a toml file without reordering". Cargo itself uses it. |
| `toml` | 0.8 | Read-only TOML parsing | Already in `ferro-cli`, keep for non-mutating reads (`read_rust_channel`, `read_bins`). |
| `clap` | 4 | Flag parsing | Already in use for `--force` and `--ferro-version`. |

**Verify before commit:** `cargo tree -p ferro-cli | grep toml_edit` (expected already present via transitive dep; promoting to direct is free).

## Architecture Patterns

- **Pure render functions, impure command wrappers.** Every `render_*` in `templates/` takes a fully-resolved context struct and returns `String`. Commands in `commands/` do the I/O. `--dry-run` exploits this separation — extend `rewrite_cargo_docker_toml` to match.
- **Token substitution, not templating.** No Tera, no Askama. Just `str::replace("{{TOKEN}}", ...)`. Stay consistent.
- **Shared helpers under `src/deploy/`.** `env_production.rs`, `rewrite_ferro_version.rs` already live here. New `bin_detect.rs` and (optionally) `secret_heuristic.rs` belong next to them.
- **Test layout:** unit tests live in `#[cfg(test)] mod tests { ... }` at the bottom of the same file. Extend existing modules, don't add top-level `tests/`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Preserve TOML key order on mutation | a "merge by reading original line-by-line" hack | `toml_edit::DocumentMut` | Built for this; `cargo`'s own implementation; handles inline vs expanded tables, comments, whitespace. |
| YAML emission for `envs:` block | pulling in `serde_yaml` | `format!` + string concat | The template is already string-substitution. Adding a YAML lib for one block is overkill and fights the existing pattern. |
| `.env` line-by-line parsing | `dotenvy` | existing hand-rolled parser in `env_production.rs` | Already written, tested, and correct for the key-only use case. Extend it for blank-line separators; don't replace with a library that discards that structure. |

## Common Pitfalls

### Pitfall 1: toml_edit appends new keys
**What:** setting `t["version"] = value(...)` on a table that didn't previously have `version` lands the new key at the end, not the top.
**Avoid:** decide up-front whether "preserve order" means "preserve the keys that already existed" (acceptable) or "put version first" (requires manual positioning via `Table::insert_formatted`).
**Recommendation:** preserve existing keys only. The user's test `preserves_dep_table_order` should assert original-key order is untouched and accept `version` appended last.

### Pitfall 2: Inline table vs expanded table
**What:** `ferro = { path = "..." }` is an inline table; `[dependencies.ferro]\npath = "..."` is an expanded table. `toml_edit` exposes different APIs (`as_inline_table_mut` vs `as_table_mut`). Forgetting to handle both produces silent no-ops.
**Avoid:** match on both variants in the ferro-key loop; add a regression test for each form.

### Pitfall 3: `cargo build --release` still builds all bins
**What:** D-10 is correct that plain `cargo build --release` builds every declared `[[bin]]` by default. Do not worry that removing per-bin builds breaks multi-bin scaffolds. Verified in the existing Dockerfile flow (gestiscilo built web + worker with a single invocation after the dep fix).

### Pitfall 4: Dockerfile stage boundary
**What:** `ENTRYPOINT` / `CMD` must live in the FINAL stage (`runtime`), not the builder. Place the new token block AFTER `EXPOSE 8080` at the bottom of the runtime stage, inside the same `FROM debian:bookworm-slim AS runtime` scope.

### Pitfall 5: `--dry-run` + `--force` combination
**What:** if the user passes both, `--force` MUST become a no-op (dry-run trumps). Document this in the clap help strings. Test it.

### Pitfall 6: `.env.example` vs `.env.production` source swap breaks existing error path
**What:** current `do:init` errors hard when `.env.production` is missing. Switching the parse source to `.env.example` changes that UX. Keep the existing pre-check if `.env.production` is still an expected artifact, or move the precondition to "`.env.example` must exist" (which it always does for a ferro-new project).
**Recommendation:** read keys from `.env.example`; keep a softer warning (not an error) if `.env.production` is absent, since D-06 is about structural shape, not about user having already populated values.

## Code Examples

### `toml_edit` mutation preserving order

```rust
// Source: https://docs.rs/toml_edit/latest/toml_edit/
use toml_edit::{value, DocumentMut};

let mut doc: DocumentMut = cargo_toml_contents.parse()?;
if let Some(deps) = doc.get_mut("dependencies").and_then(|i| i.as_table_mut()) {
    if let Some(ferro) = deps.get_mut("ferro") {
        if let Some(t) = ferro.as_inline_table_mut() {
            t.remove("path");
            t.insert("version", "0.2.1".into());
        } else if let Some(t) = ferro.as_table_mut() {
            t.remove("path");
            t["version"] = value("0.2.1");
        }
    }
}
let out = doc.to_string(); // round-trips with original formatting
```

### Composing ENTRYPOINT token

```rust
// In render_dockerfile:
let entrypoint = format!(
    "ENTRYPOINT [\"/usr/local/bin/{bin}\"]\nCMD [\"serve\"]",
    bin = ctx.entrypoint_bin
);
DOCKERFILE_TPL.replace("{{ENTRYPOINT}}", &entrypoint)
```

### Envs block emission

```rust
fn render_envs_block(lines: &[EnvLine]) -> String {
    let mut out = String::from("envs:\n");
    for line in lines {
        match line {
            EnvLine::BlankSeparator => out.push('\n'),
            EnvLine::Key(k) => {
                if is_secret_key(k) {
                    out.push_str(&format!(
                        "  - key: {k}\n    value: \"\"\n    scope: RUN_AND_BUILD_TIME\n    type: SECRET\n"
                    ));
                } else {
                    out.push_str(&format!(
                        "  - key: {k}\n    value: \"\"\n    scope: RUN_TIME\n"
                    ));
                }
            }
        }
    }
    out
}
```

## Runtime State Inventory

Not applicable — this is a greenfield refactor of generator code. No stored data, live services, OS-registered tasks, secrets, or build artifacts depend on the changed strings.

## Environment Availability

Skip — phase is code-only changes to `ferro-cli`. No new external tools, services, runtimes, or package managers required beyond the workspace's existing Rust toolchain.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (built-in), `tempfile = "3.24"` for filesystem fixtures |
| Config file | none needed; uses standard `#[cfg(test)] mod tests` pattern |
| Quick run command | `cargo test -p ferro-cli --all-features` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | Dockerfile contains both `ENTRYPOINT` and `CMD` lines | unit | `cargo test -p ferro-cli --all-features render_dockerfile_emits_entrypoint_and_cmd` | ❌ Wave 0 |
| D-02 | `detect_web_bin` honors override → package-match → first bin → fallback | unit | `cargo test -p ferro-cli --all-features detect_web_bin_` | ❌ Wave 0 (new `bin_detect.rs`) |
| D-03 | ENTRYPOINT uses `/usr/local/bin/<bin>`, CMD uses `["serve"]` | unit | `cargo test -p ferro-cli --all-features dockerfile_entrypoint_shape` | ❌ Wave 0 |
| D-04 | `{{ENTRYPOINT}}` token exists and is replaced | unit | `cargo test -p ferro-cli --all-features dockerfile_no_unreplaced_tokens` | ❌ Wave 0 |
| D-05 | `.do/app.yaml` `web` service has NO `run_command` line | unit | `cargo test -p ferro-cli --all-features do_init_web_has_no_run_command` | ❌ Wave 0 |
| D-06 | Envs block emits real YAML entries, one per key | unit | `cargo test -p ferro-cli --all-features envs_block_emits_entries_per_key` | ❌ Wave 0 |
| D-07 | Secret keys get `type: SECRET` + `RUN_AND_BUILD_TIME`; others get `RUN_TIME` only | unit | `cargo test -p ferro-cli --all-features envs_block_classifies_secrets` | ❌ Wave 0 |
| D-08 | `is_secret_key` matches {secret,password,passwd,token,key,api_key,dsn,private,credential}, case-insensitive; `_URL` exception | unit | `cargo test -p ferro-cli --all-features is_secret_key_` | ❌ Wave 0 (new helper) |
| D-09 | Envs block order matches source; blank lines preserved | unit | `cargo test -p ferro-cli --all-features envs_block_preserves_order_and_blanks` | ❌ Wave 0 |
| D-10 | Rendered Dockerfile has exactly one `cargo build --release` invocation; no `--bin` builds | unit | `cargo test -p ferro-cli --all-features dockerfile_single_cargo_build` | ❌ Wave 0 (replaces existing `multi_bin_emits_per_bin_build_and_copy`) |
| D-11 | `rewrite_cargo_docker_toml` preserves sibling keys AND dep-table order | unit | `cargo test -p ferro-cli --all-features preserves_dep_table_order` | ❌ Wave 0 |
| D-12 | Existing `preserves_package_rename_and_features` still passes post-migration | unit | `cargo test -p ferro-cli --all-features preserves_package_rename_and_features` | ✅ |
| D-13, D-14 | `docker:init` prints footer with `docker build` + `docker run` lines | integration | `cargo test -p ferro-cli --all-features docker_init_prints_footer` | ❌ Wave 0 |
| D-15 | `do:init` prints footer with review/populate/`doctl` lines | integration | `cargo test -p ferro-cli --all-features do_init_prints_footer` | ❌ Wave 0 |
| D-16 | Footer is suppressed in `--dry-run` | integration | `cargo test -p ferro-cli --all-features dry_run_suppresses_footer` | ❌ Wave 0 |
| D-17 | `--dry-run` writes no files; prints per-file headers | integration | `cargo test -p ferro-cli --all-features dry_run_writes_no_files` | ❌ Wave 0 |
| D-18 | `--dry-run` prints computed `Cargo.docker.toml` contents without persisting | integration | `cargo test -p ferro-cli --all-features dry_run_prints_cargo_docker_toml` | ❌ Wave 0 |
| D-19 | Rendering errors in `--dry-run` exit non-zero (no metadata → hard fail) | integration | `cargo test -p ferro-cli --all-features dry_run_propagates_render_errors` | ❌ Wave 0 |
| D-20 | `.dockerignore` contains `!README.md` after `*.md` | unit | `cargo test -p ferro-cli --all-features dockerignore_whitelists_readme` | ❌ Wave 0 |
| D-21 | `.dockerignore` has a one-line comment explaining the whitelist | unit | `cargo test -p ferro-cli --all-features dockerignore_readme_comment_present` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-cli --all-features`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`.

### Wave 0 Gaps
- [ ] New file: `ferro-cli/src/deploy/bin_detect.rs` with tests for D-02.
- [ ] New file (optional, recommended): `ferro-cli/src/deploy/secret_heuristic.rs` with tests for D-08 + the known `SLACK_WEBHOOK_URL` false-negative case.
- [ ] Extend `ferro-cli/src/templates/docker.rs` tests for D-01, D-03, D-04, D-10, D-20, D-21.
- [ ] Extend `ferro-cli/src/templates/do.rs` tests for D-05, D-06, D-07, D-09.
- [ ] Extend `ferro-cli/src/deploy/rewrite_ferro_version.rs` tests for D-11 (new test) and confirm D-12 (existing test).
- [ ] Extend `ferro-cli/src/commands/docker_init.rs` tests for D-13, D-14, D-16, D-17, D-18, D-19.
- [ ] Extend `ferro-cli/src/commands/do_init.rs` tests for D-15, D-16, D-17.
- [ ] Framework install: none — `tempfile` and `anyhow` already in `[dev-dependencies]` / `[dependencies]`.
- [ ] Add direct dep `toml_edit = "0.22"` to `ferro-cli/Cargo.toml`.

## Sequencing (Suggested Wave Layout)

**Wave 1 — foundations (parallelizable internally, no deps):**
- W1-T1: Add `toml_edit` to `ferro-cli/Cargo.toml`; migrate `rewrite_ferro_version.rs`; add `preserves_dep_table_order` test; verify D-12 still green. Split compute/persist per D-18.
- W1-T2: Create `deploy/bin_detect.rs` with `detect_web_bin` + tests. Add `web_bin` field to `DeployMetadata` if missing.
- W1-T3: Create `deploy/secret_heuristic.rs` with `is_secret_key` + tests (including known false-negative assertion).
- W1-T4: Extend `deploy/env_production.rs` to `read_env_lines -> Vec<EnvLine>` preserving blank separators. Update sole caller.

**Wave 2 — templates & renderers (depends on Wave 1 for bin_detect + secret_heuristic):**
- W2-T1: Edit `Dockerfile.tpl` — add `{{ENTRYPOINT}}` slot, drop `{{BIN_BUILDS}}`. Edit `dockerignore.tpl` — whitelist `README.md`.
- W2-T2: Update `templates/docker.rs::DockerContext` + `render_dockerfile` — drop bin_builds, add entrypoint. Update tests.
- W2-T3: Edit `app.yaml.tpl` — replace `{{ENV_COMMENTS}}` with `{{ENVS_BLOCK}}`, add D-05 comment on web service.
- W2-T4: Update `templates/do.rs` — replace `render_env_comments` with `render_envs_block` using `EnvLine` + `is_secret_key`. Update `AppYamlContext`. Update tests.

W1 and W2 can run in parallel if W2 mocks the helpers that W1 is producing; safer to run W1 first, then W2.

**Wave 3 — command wiring (depends on W1 + W2):**
- W3-T1: `commands/docker_init.rs` — plumb `dry_run`, call `detect_web_bin`, call compute+persist split, print footer. Add tests D-13..D-21 relevant to docker.
- W3-T2: `commands/do_init.rs` — plumb `dry_run`, call `detect_web_bin`, read `.env.example` via `read_env_lines`, print footer. Add tests D-13..D-21 relevant to do.
- W3-T3: `main.rs` clap — add `#[arg(long)] dry_run: bool` to both variants; match-arm pass-through.

**Parallelization:** within each wave, tasks touch disjoint files and can run in parallel. Across waves there is a strict dependency. Rough budget: 1 coding pass per wave.

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `toml_edit` round-trip whitespace drift on existing test fixtures | MEDIUM | Write `preserves_dep_table_order` as a key-order assertion (parse output and compare key sequence), not a whitespace-exact string compare. Avoid snapshot tests. |
| Dockerfile ENTRYPOINT ends up in wrong stage | MEDIUM | Template placement is post-`EXPOSE 8080`, which is inside the runtime stage. Add a test asserting ENTRYPOINT appears AFTER `FROM debian:bookworm-slim AS runtime` in the rendered output. |
| `--dry-run` tests depend on project-root discovery (`find_project_root`) walking up to Cargo.toml | MEDIUM | Use `tempfile::TempDir` + `std::env::set_current_dir` (pattern already used in `do_init.rs` tests). Acquire a mutex to avoid parallel CWD races, OR refactor `execute` to accept an explicit root override for testing. Recommend the latter — cleaner. |
| `SLACK_WEBHOOK_URL` false negative ships unflagged | LOW | Add a test asserting the false-negative; document in generated `.do/app.yaml` header comment that users should review `*_URL` entries for embedded secrets. Raise to user before planning if they want to extend D-08's substring set. |
| `DeployMetadata.web_bin` may not exist yet | LOW | Verify via grep; if missing, add single field to `project::DeployMetadata` — trivially small. |
| `Cargo.docker.toml` output with `toml_edit` may differ textually from current `toml` crate output | LOW | Acceptable — the file is human-readable and the goal is fewer reordering diffs, not byte-exact back-compat. Update any golden tests. |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `toml` crate for mutations | `toml_edit` for mutations, `toml` for reads | toml_edit 0.20+ (2023) | Preserves order/whitespace/comments; standard since cargo itself uses it. |
| Dockerfile without ENTRYPOINT, relying on `docker run <img> <cmd>` | `ENTRYPOINT [...] CMD [...]` | Docker best practice, always | Deploy targets (DO, ECS, Fly) expect images to be runnable without explicit command. |

**Deprecated / outdated:** none relevant to this phase.

## Open Questions

1. **Should D-08's substring set include `webhook`?**
   - What we know: `SLACK_WEBHOOK_URL` is a secret in practice; D-08's rule classifies it as non-secret.
   - What's unclear: whether to extend the locked decision or accept the false negative.
   - Recommendation: surface to user during planning. Default: honor D-08 as written and document the false negative in generated comments and a test.

2. **`DeployMetadata.web_bin` existence.**
   - What we know: `project::read_deploy_metadata` is called in `docker_init.rs:30`.
   - What's unclear: whether the struct already has `web_bin: Option<String>` or needs adding.
   - Recommendation: first task in W1-T2 is to grep and add if missing. Trivial.

3. **`.env.example` vs `.env.production` source for envs block.**
   - What we know: D-09 says `.env.example` order.
   - What's unclear: should `.env.production` still be a precondition? Current code hard-errors on its absence.
   - Recommendation: read from `.env.example`; downgrade `.env.production` absence to a warning, not an error. Flag for user review during planning.

## Sources

### Primary (HIGH confidence)
- Direct read of all 14 files in `<files_to_read>` — `CONTEXT.md`, `REPORT.md`, `Dockerfile.tpl`, `dockerignore.tpl`, `app.yaml.tpl`, `docker.rs`, `do.rs`, `docker_init.rs`, `do_init.rs`, `rewrite_ferro_version.rs`, `env_production.rs`, `Cargo.toml`, `CLAUDE.md`, and `env.example.tpl` (for §5 dry run).
- `main.rs` clap definitions grep'd for `DockerInit` / `DoInit` variants.
- Workspace `Cargo.lock` grep confirmed `toml_edit` is already a transitive dependency.

### Secondary (MEDIUM confidence)
- `toml_edit` public API shape — from general ecosystem knowledge; the exact method names (`as_inline_table_mut`, `as_table_mut`, `DocumentMut`) should be verified against `https://docs.rs/toml_edit` before writing code. No Context7 call made in this research pass since the surface area is small and the migration is straightforward.

### Tertiary (LOW confidence)
- none — every load-bearing claim is backed by a direct file read.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — `toml_edit` is the industry standard and already a transitive dep.
- Architecture: HIGH — code surface read in full, pure/impure boundary is clean.
- Pitfalls: HIGH — every pitfall is grounded in a specific file line range.
- Secret heuristic dry-run: HIGH — exhaustive walk of `env.example.tpl` performed inline.

**Research date:** 2026-04-08
**Valid until:** 2026-05-08 (stable surface, small scope)
