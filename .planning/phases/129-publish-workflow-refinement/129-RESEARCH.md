# Phase 129: Publish Workflow Refinement — Research

**Researched:** 2026-04-09
**Domain:** GitHub Actions workflow gating + ferro-cli deploy metadata schema
**Confidence:** HIGH

## Summary

Phase 129 absorbs REPORT items 8 (publish workflow auto-bumps on every push) and 14 (single global `ferro_version` field, no per-crate override). Work is split across three files: `.github/workflows/publish.yml` (gate step), `ferro-cli/src/project.rs` (schema parser — the REAL home of `ferro_version` parsing, not `rewrite_ferro_version.rs` as CONTEXT.md suggests), and `PUBLISHING.md` (doc sections).

**Primary recommendation:** Add a gate step in `check-version` that computes `git diff --name-only <last-tag>..HEAD` filtered through an exclusion list; when no library crate files remain, emit `should_publish=no` (new third value alongside existing `bump`/`yes`). Add `ferro_versions: Option<BTreeMap<String,String>>` field to `FerroDeployMetadata` in `project.rs` with parse + reject-on-wrong-type logic mirroring the existing fields. Add round-trip regression test in `rewrite_ferro_version.rs` proving `toml_edit` leaves the unknown metadata table untouched.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Detect "library crate changed" via `git diff` of paths between last published tag and `HEAD`, not `cargo metadata` timestamps.
- **D-02:** Library paths = all workspace members EXCEPT `ferro-cli/`. Future binary-only crates extend the exclusion list.
- **D-03:** Non-crate excluded paths: `docs/`, `.github/`, `README*`, `CHANGELOG*`, `PUBLISHING.md`, `.planning/`, top-level `*.md`, top-level config (`.gitignore`, `.editorconfig`, `rustfmt.toml`, etc.).
- **D-04:** Zero library changes → `should_publish=no` → `bump-version`/`test`/`publish` all skipped.
- **D-05:** One or more library changes → unchanged behavior (bump patch, tag, publish waves).
- **D-06:** Gate check lives inside existing `check-version` job. No new jobs.
- **D-07:** Optional `ferro_versions` table keyed by crate name in `[package.metadata.ferro.deploy]`.
- **D-08:** Parser accepts + round-trips `ferro_versions`; rewrite logic unchanged (global `ferro_version` still authoritative). TODO comment + tracking ref in code.
- **D-09:** No CLI flag, no UX, no doctor check for `ferro_versions` yet. Schema reservation only.
- **D-10:** `PUBLISHING.md` gains "Version Model" section: lockstep release, single `ferro_version` applied to every ferro dep, intentional simplification, `ferro_versions` reserved.
- **D-11:** `PUBLISHING.md` gains "Publish Gating" section documenting the rule and excluded paths.
- **D-12:** Workflow changes verified by inline comments + manual scenario table in `PUBLISHING.md`. No CI-for-CI.
- **D-13:** Rust regression test `parses_and_roundtrips_ferro_versions_override` sibling of `preserves_package_rename_and_features`.

### Claude's Discretion

- Exact `git diff --name-only` invocation.
- Wording of `PUBLISHING.md` sections.
- Excluded-paths list as top-of-file env var OR inline in gate step.
- New output value name (`no` vs `skip`).

### Deferred Ideas (OUT OF SCOPE)

- Functional per-crate version resolution / desync wiring.
- Doctor check for `ferro_versions` correctness.
- `ferro deploy:check` CLI promotion (REPORT §12).
- Dockerfile ENTRYPOINT/CMD (REPORT §18).
- `copy_dirs`/`.dockerignore` collision (REPORT §3) — already in flight.
- `toml_edit` migration (REPORT §5) — already landed in `rewrite_ferro_version.rs`.
</user_constraints>

<phase_requirements>
## Phase Requirements

Phase 129 does not map to a v13.0 REQ-ID in `REQUIREMENTS.md`; it is a tactical polish phase absorbing REPORT items 8 and 14 from Phase 126. Tracked as:

| ID | Description | Research Support |
|----|-------------|------------------|
| REPORT-08 | Publish workflow auto-bumps on pushes touching only `ferro-cli/`, docs, CI, or planning — churns library crate versions | Workflow `check-version` job structure; `git diff --name-only` against last `v$VERSION` tag |
| REPORT-14 | Single global `ferro_version` field cannot express per-crate desync; document lockstep + reserve extension point | `FerroDeployMetadata` struct location in `project.rs`; `toml_edit` inherent preservation of unrelated tables |
</phase_requirements>

## Correction to CONTEXT.md

CONTEXT.md states the parser lives in `ferro-cli/src/deploy/rewrite_ferro_version.rs` and that file uses value-level `toml`. Both are stale:

1. **Parser location:** `[package.metadata.ferro.deploy]` is parsed in `ferro-cli/src/project.rs::read_deploy_metadata` (lines ~46–115). The `FerroDeployMetadata` struct is defined there with hand-rolled `toml::Value` field extraction (no serde derive). `rewrite_ferro_version.rs` does NOT parse that table — it only rewrites `[dependencies]` path→version.
2. **toml_edit already landed:** `rewrite_ferro_version.rs` now uses `toml_edit::DocumentMut` (see file header "Phase 127 D-11, D-12 toml_edit migration"). REPORT §5's concern is resolved. This is load-bearing: round-trip of `ferro_versions` works automatically through `toml_edit` because the rewriter only touches dep tables and leaves `[package.metadata.ferro.deploy]` byte-identical.

**Implication for plan:** Parser work goes in `project.rs`. Round-trip regression test may still go in `rewrite_ferro_version.rs` per D-13 (it proves the rewriter leaves the new field untouched), but a parser-level test should also live in `project.rs`'s existing test module alongside `reads_deploy_metadata_*` tests.

## Current Workflow Structure (`.github/workflows/publish.yml`)

**Jobs and gating:**

| Job | Needs | Gate (`if:`) | Outputs |
|-----|-------|--------------|---------|
| `check-version` | — | always runs on push to master | `should_publish`, `version`, `new_version` |
| `bump-version` | `check-version` | `should_publish == 'bump'` | `version` |
| `test` | `check-version`, `bump-version` | `always() && should_publish != ''` | — |
| `publish` | `check-version`, `bump-version`, `test` | `always() && test.result == 'success' && should_publish != ''` | — |

**Current `should_publish` values:** `bump` (version tagged, needs patch bump) or `yes` (version not yet tagged, publish as-is). Empty string is used in downstream `if:` guards to detect "job not run" — D-04's new `no` value must NOT be empty string; it must be the literal `no` so downstream `!= ''` guards still succeed, which means we also need to flip them to explicit `!= 'no'` OR rely on `no` being truthy-but-skip by adding `should_publish == 'bump' || should_publish == 'yes'` checks.

**Recommendation on value name:** Use `none` (not `no` — `no` is a YAML boolean literal and will be coerced to `false` in some contexts). Or quote it defensively. Safest: introduce a rewrite so downstream jobs gate on `should_publish == 'bump' || should_publish == 'yes'` explicitly, and the skip case emits `should_publish=none`.

**Tag format:** Workflow creates tags as `v$VERSION` via `gh api repos/.../git/refs -f ref="refs/tags/v$VERSION"` — these are **lightweight** refs, not annotated tags. Detection uses `git tag | grep -q "^v$VERSION$"`. Last-tag detection for the gate should use `git describe --tags --abbrev=0 --match 'v*'` to match only `v<semver>` shaped tags and ignore pre-release variants per `<specifics>` guidance.

**First-run edge case:** If no `v*` tag exists yet, `git describe` exits non-zero. Gate must treat this as "publish" (every file is a library change by default). Suggested shape:

```bash
LAST_TAG=$(git describe --tags --abbrev=0 --match 'v*' 2>/dev/null || echo "")
if [ -z "$LAST_TAG" ]; then
  CHANGED="ALL_FIRST_RUN"
else
  CHANGED=$(git diff --name-only "$LAST_TAG"..HEAD)
fi
```

## Workspace Member Inventory

Verified from root `Cargo.toml` `[workspace] members`:

| Member dir | Type | Library? |
|------------|------|----------|
| `framework` | lib (published as `ferro-rs`) | YES |
| `ferro-macros` | proc-macro lib | YES |
| `ferro-events` | lib | YES |
| `ferro-queue` | lib | YES |
| `ferro-notifications` | lib | YES |
| `ferro-broadcast` | lib | YES |
| `ferro-storage` | lib | YES |
| `ferro-cache` | lib | YES |
| `ferro-mcp` | lib | YES |
| `ferro-inertia` | lib | YES |
| `ferro-json-ui` | lib | YES |
| `ferro-lang` | lib | YES |
| `ferro-api-mcp` | lib | YES |
| `ferro-projections` | lib | YES |
| `ferro-stripe` | lib | YES |
| `ferro-theme` | lib | YES |
| `ferro-ai` | lib | YES |
| `ferro-whatsapp` | lib | YES |
| **`ferro-cli`** | **binary** | **NO — excluded** |
| **`app`** | **sample/binary** | **NO — not published** |

`ferro-cli` is the sole binary confirmed by CONTEXT.md D-02. `app` is the sample application and is already not published (no `cargo publish -p app` in the workflow), but it is a workspace member and changes under `app/` should also NOT trigger a bump — add to exclusion list alongside `ferro-cli/`.

**Non-crate top-level paths to exclude:** `docs/`, `.github/`, `.planning/`, `scripts/`, top-level `*.md` (README, PUBLISHING, CLAUDE, AGENTS, FERRO-BRIEF, FERRO-THEME-REPORT, LICENSE), `Cargo.lock` (regenerated automatically), `bacon.toml`, `deny.toml`, `dev.sh`, `llms.txt`, `rust-toolchain.toml`, `.gitignore`, `.editorconfig`, `rustfmt.toml` (if present).

**`Cargo.toml` at root is load-bearing** — it contains the workspace version. A root `Cargo.toml` change IS a library-relevant change (version bump, dependency update) and must NOT be excluded.

## Gate Step Design

**Recommended invocation:**

```bash
# Determine base ref
LAST_TAG=$(git describe --tags --abbrev=0 --match 'v*' 2>/dev/null || echo "")
if [ -z "$LAST_TAG" ]; then
  echo "No prior v* tag — treating as library change (first publish)"
  LIB_CHANGED=1
else
  # List changed paths since last tag
  CHANGED=$(git diff --name-only "$LAST_TAG"..HEAD)
  echo "Changed paths since $LAST_TAG:"
  echo "$CHANGED"

  # Filter out excluded paths; anything remaining counts
  LIB_CHANGED=0
  while IFS= read -r path; do
    case "$path" in
      ferro-cli/*|app/*) continue ;;
      docs/*|.github/*|.planning/*|scripts/*) continue ;;
      *.md|LICENSE|.gitignore|.editorconfig|rustfmt.toml) continue ;;
      bacon.toml|deny.toml|dev.sh|llms.txt|rust-toolchain.toml) continue ;;
      "") continue ;;
      *) LIB_CHANGED=1; echo "Library change: $path"; break ;;
    esac
  done <<< "$CHANGED"
fi
```

**Placement:** First step inside `check-version`, before the "Check if version is tagged" step. If `LIB_CHANGED=0`, set `should_publish=none` and early-exit without computing `new_version`. The existing tag-check logic runs only when `LIB_CHANGED=1`.

**Downstream gate rewrite:** Each downstream job's `if:` must change from `should_publish != ''` to `should_publish != '' && should_publish != 'none'`, OR the cleaner form `should_publish == 'bump' || should_publish == 'yes'`. The second form is more explicit and future-proof — recommend it.

**Exclusion list as env var:** Recommend inlining in the gate step as a shell `case` (above) rather than a top-of-file env var — the case statement handles glob patterns natively and is the most readable form for maintainers extending it.

## Schema Parser: `project.rs::FerroDeployMetadata`

**Current struct** (`ferro-cli/src/project.rs`):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FerroDeployMetadata {
    pub runtime_apt: Vec<String>,
    pub copy_dirs: Vec<String>,
    pub ferro_version: Option<String>,
    pub web_bin: Option<String>,
}
```

**Parser shape:** Hand-written field extraction from `toml::Value`. Each field uses `table.get(name).and_then(...).ok_or_else(|| anyhow!("... must be a <type>"))`. No serde derive. Error-on-wrong-type is explicit.

**New field:**

```rust
use std::collections::BTreeMap;

pub struct FerroDeployMetadata {
    pub runtime_apt: Vec<String>,
    pub copy_dirs: Vec<String>,
    pub ferro_version: Option<String>,
    /// Reserved schema hook for future per-crate version desync (Phase 129 D-07..D-09).
    /// Accepted and round-tripped today; NOT consulted by any rewrite logic.
    /// When the lockstep release model breaks, wire this through
    /// `rewrite_cargo_docker_toml`'s per-dep resolution and add doctor
    /// coverage. Until then, `ferro_version` above is authoritative.
    pub ferro_versions: Option<BTreeMap<String, String>>,
    pub web_bin: Option<String>,
}
```

**Parse block to add in `read_deploy_metadata`:**

```rust
if let Some(v) = table.get("ferro_versions") {
    let t = v.as_table().ok_or_else(|| {
        anyhow::anyhow!(
            "[package.metadata.ferro.deploy].ferro_versions must be a table"
        )
    })?;
    let mut map = BTreeMap::new();
    for (k, val) in t {
        let s = val.as_str().ok_or_else(|| {
            anyhow::anyhow!(
                "[package.metadata.ferro.deploy].ferro_versions.{k} must be a string"
            )
        })?;
        map.insert(k.clone(), s.to_string());
    }
    meta.ferro_versions = Some(map);
}
```

Update `Default` impl to set `ferro_versions: None`.

**Parser test (`project.rs` tests module):** Accepts + round-trips:

```rust
#[test]
fn reads_ferro_versions_override_table() {
    // Cargo.toml with [package.metadata.ferro.deploy.ferro_versions]
    // Assert FerroDeployMetadata::ferro_versions == Some(BTreeMap { ... })
    // Assert wrong-type (string instead of table) → Err
    // Assert wrong-value-type (int instead of string) → Err
}
```

## Round-trip Test in `rewrite_ferro_version.rs`

Per D-13, add `parses_and_roundtrips_ferro_versions_override` as sibling of `preserves_package_rename_and_features`. Purpose: prove that `rewrite_contents` leaves `[package.metadata.ferro.deploy].ferro_versions` byte-identical after rewriting dep tables.

**Why it works naturally:** `rewrite_contents` only touches keys under `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`. `[package.metadata.*]` is never inspected. `toml_edit::DocumentMut` preserves whitespace, comments, and key order in all untouched tables.

**Test skeleton:**

```rust
#[test]
fn parses_and_roundtrips_ferro_versions_override() {
    let tmp = TempDir::new().unwrap();
    let project = tmp.path().join("project");
    write(
        &project.join("Cargo.toml"),
        r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
ferro = { path = "../framework" }

[package.metadata.ferro.deploy]
ferro_version = "0.2.0"

[package.metadata.ferro.deploy.ferro_versions]
ferro-json-ui = "0.2.1"
ferro-whatsapp = "0.2.0"
"#,
    );
    write(
        &tmp.path().join("framework/Cargo.toml"),
        "[package]\nname = \"ferro\"\nversion = \"0.2.0\"\n",
    );

    let out = rewrite_cargo_docker_toml(&project, Some("0.2.0")).unwrap();
    let body = fs::read_to_string(&out).unwrap();

    // Dep table rewritten as expected
    assert!(body.contains("ferro = { version = \"0.2.0\""));
    assert!(!body.contains("../framework"));

    // ferro_versions override survives byte-for-byte
    assert!(body.contains("[package.metadata.ferro.deploy.ferro_versions]"));
    assert!(body.contains("ferro-json-ui = \"0.2.1\""));
    assert!(body.contains("ferro-whatsapp = \"0.2.0\""));

    // Parse round-trip semantically too
    let parsed: toml::Value = body.parse().unwrap();
    let overrides = parsed
        .get("package").unwrap()
        .get("metadata").unwrap()
        .get("ferro").unwrap()
        .get("deploy").unwrap()
        .get("ferro_versions").unwrap()
        .as_table().unwrap();
    assert_eq!(overrides.get("ferro-json-ui").and_then(|v| v.as_str()), Some("0.2.1"));
    assert_eq!(overrides.get("ferro-whatsapp").and_then(|v| v.as_str()), Some("0.2.0"));
}
```

## `PUBLISHING.md` Structure

Current file (132 lines): Prerequisites → Pre-publish Verification → Publishing Order (Wave 1/2/3) → Path Dependency Handling → Post-publish Verification → Version Coordination → Troubleshooting → Crate Summary.

**Also noted:** Wave tables are stale relative to the actual workflow (`publish.yml` now has Wave 1a/1b/2/3 including `ferro-ai`, `ferro-projections`, `ferro-stripe`, `ferro-whatsapp`, `ferro-theme`, `ferro-api-mcp`, `ferro-lang`). Updating the wave tables is OUT OF SCOPE for this phase but worth flagging — the "Version Coordination" section is where the new "Version Model" content slots cleanly.

**Recommended insertion points:**

1. **"Version Model"** — replace/expand the existing "Version Coordination" section (line 94) since they overlap. Content per D-10: lockstep, single `ferro_version`, intentional simplification, `ferro_versions` reserved.
2. **"Publish Gating"** — new section between "Publishing Order" (line 26) and "Wave 1" OR at the end before "Troubleshooting". Content per D-11: bump only if library change, list of excluded paths, scenario table (push touching only docs/ → skipped, push touching framework/src/ → bumped, first push ever → published, etc.).

## Architecture Patterns

### Pattern 1: `toml_edit` round-trip for byte-preserving rewrites
**What:** Use `toml_edit::DocumentMut` when any part of a TOML file must be preserved exactly (whitespace, comments, key order) while surgically mutating specific tables.
**When:** Already in place in `rewrite_ferro_version.rs`. The new `ferro_versions` field benefits passively — no code change needed in the rewriter.

### Pattern 2: Hand-rolled `toml::Value` parsing with explicit error messages
**What:** Parse metadata tables field-by-field from `toml::Value` rather than using serde derive. Each field wrong-type produces a targeted error message naming the exact TOML path.
**When:** Consumer-facing schemas where error messages must point at user input (e.g., `[package.metadata.ferro.deploy].runtime_apt must be an array`).
**Pattern in `project.rs`:** `table.get(name).as_<type>().ok_or_else(|| anyhow!(...))?`

### Pattern 3: `should_publish` as multi-valued output
**What:** GitHub Actions job output as a three-state enum (`bump`/`yes`/`none`), gated by downstream `if:` expressions.
**Gotcha:** Empty string is currently used as the "not set" sentinel. `none` (literal string) is safer than `no` because `no` coerces to YAML boolean `false` in some contexts. Quote defensively: `should_publish == 'none'`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Detect library changes | `cargo metadata` + mtime inspection | `git diff --name-only <tag>..HEAD` + path filter | Deterministic, matches workflow reasoning, no build required |
| Preserve TOML metadata table through rewrite | Custom serializer that re-emits untouched tables | `toml_edit::DocumentMut` (already in place) | Byte-for-byte preservation is free |
| Last-tag detection with version prefix filter | Manual `git tag` parsing + semver sort | `git describe --tags --abbrev=0 --match 'v*'` | One command, handles prerelease exclusion via `--match` |

## Common Pitfalls

### Pitfall 1: YAML boolean coercion on `should_publish: no`
**What goes wrong:** `should_publish: no` in YAML context (if copied into a YAML string without quoting) coerces to boolean `false`, breaking `!= ''` guards.
**Why:** YAML 1.1 treats `no`, `off`, `n` as false.
**How to avoid:** Use `none` instead of `no`, OR always emit through `$GITHUB_OUTPUT` as `should_publish=no` (env file, not YAML — safe) and gate downstream with quoted string comparison `should_publish == 'no'`. The env-file path IS safe; the risk is only if the value ever appears unquoted in `.yml`. Recommend `none` anyway for clarity.

### Pitfall 2: First-run `git describe` failure
**What goes wrong:** Before any `v*` tag exists, `git describe --tags --abbrev=0 --match 'v*'` exits non-zero and returns nothing, causing `git diff "$LAST_TAG"..HEAD` to fail obscurely.
**How to avoid:** Guard with `2>/dev/null || echo ""` and branch on empty string → treat as library-changed (publish).
**Warning sign:** CI logs showing `fatal: No names found` or `fatal: bad revision '..HEAD'`.

### Pitfall 3: Shallow clone missing tags
**What goes wrong:** `actions/checkout@v4` default `fetch-depth: 1` fetches no tags; `git describe` finds nothing even when tags exist on origin.
**How to avoid:** Workflow already uses `fetch-depth: 0` in every job — confirmed safe. Don't reduce it.

### Pitfall 4: `Cargo.lock` churn triggering false positives
**What goes wrong:** `Cargo.lock` regenerates on dep updates and can appear in diffs for commits that touched no library source.
**How to avoid:** Exclude `Cargo.lock` from gate detection. Cargo.lock changes alone never warrant a library release.

### Pitfall 5: Root `Cargo.toml` false negatives
**What goes wrong:** Excluding all top-level files would exclude `Cargo.toml` — but a workspace version bump or workspace dep update IS a release-worthy change.
**How to avoid:** Exclude only specific top-level files by name (README*, LICENSE, *.md, configs). Leave `Cargo.toml` detectable.

### Pitfall 6: `app/` changes triggering releases
**What goes wrong:** `app/` is a workspace member but is the sample application, not a published crate. Its changes would appear in `git diff` and pass a naive "crate dir" filter.
**How to avoid:** Exclude `app/` alongside `ferro-cli/`.

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — phase touches CI config + source parsers only | None |
| Live service config | GitHub Actions workflow state — existing runs will finish under old logic, next push runs new gate | None — behavior change is intended |
| OS-registered state | None | None |
| Secrets/env vars | `CARGO_REGISTRY_TOKEN`, `GITHUB_TOKEN` — unchanged | None |
| Build artifacts | None — `ferro_versions` is schema-only, no regen | None |

## Environment Availability

Phase 129 touches CI workflow YAML, one Rust source file, one Rust test file, and one Markdown file. Build/test requirements are the project's existing toolchain (Rust 1.88.0, `cargo fmt`/`clippy`/`test`). No new external dependencies.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (rustc 1.88.0) |
| Config file | `Cargo.toml` workspace |
| Quick run command | `cargo test -p ferro-cli --lib deploy::rewrite_ferro_version project::tests` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| REPORT-08 | Gate skips bump/publish when no library crate changed | manual scenario | N/A — workflow runs in CI only; documented in PUBLISHING.md scenario table per D-12 | N/A |
| REPORT-08 | Gate allows bump/publish when a library crate changed | manual scenario | N/A — same | N/A |
| REPORT-14 parser | `ferro_versions` table parsed into `BTreeMap<String,String>` | unit | `cargo test -p ferro-cli project::tests::reads_ferro_versions_override_table` | ❌ Wave 0 |
| REPORT-14 parser | Wrong type on `ferro_versions` → Err | unit | `cargo test -p ferro-cli project::tests::ferro_versions_wrong_type_errors` | ❌ Wave 0 |
| REPORT-14 round-trip | `ferro_versions` survives `rewrite_cargo_docker_toml` byte-wise and semantically | unit | `cargo test -p ferro-cli deploy::rewrite_ferro_version::tests::parses_and_roundtrips_ferro_versions_override` | ❌ Wave 0 |
| REPORT-14 docs | `PUBLISHING.md` contains "Version Model" section mentioning `ferro_versions` | grep check | `grep -q 'ferro_versions' PUBLISHING.md && grep -q 'Version Model' PUBLISHING.md` | ❌ Wave 0 |
| REPORT-08 docs | `PUBLISHING.md` contains "Publish Gating" section with excluded paths | grep check | `grep -q 'Publish Gating' PUBLISHING.md` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-cli` (scoped to the crate being modified)
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-cli --all-features`
- **Phase gate:** Full suite green (`cargo fmt + clippy + test --all-features`) before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `ferro-cli/src/project.rs` tests: add `reads_ferro_versions_override_table` + `ferro_versions_wrong_type_errors` to existing `tests` module
- [ ] `ferro-cli/src/deploy/rewrite_ferro_version.rs` tests: add `parses_and_roundtrips_ferro_versions_override` sibling of `preserves_package_rename_and_features`
- [ ] `PUBLISHING.md`: Version Model section + Publish Gating section (verified via grep)
- [ ] No framework install needed — existing test infrastructure covers all phase requirements

## Project Constraints (from CLAUDE.md)

- **Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` before every commit.** CI enforces `-D warnings`; any warning blocks merge.
- **`--all-targets` required** — catches test-code issues that plain `--all` misses.
- **No co-author lines in commits.** No "Generated with Claude" attribution.
- **Neutral voice in repo docs** — `PUBLISHING.md` additions must read as architectural documentation, not internal strategy. Avoid trigger phrases ("killer feature", "the bet", "load-bearing", session provenance). Frame lockstep as "current release model" not "the bet we're taking".
- **Minimalistic doc voice** — scientific, no marketing language.
- **Delete old code completely** — if the gate step replaces the current "Check if version is tagged" logic, the old bash is replaced in place, not commented out.
- **Prefer editing existing files** — `PUBLISHING.md` already exists; expand it rather than creating a new doc.
- **Update ferro-mcp when needed** — no ferro-mcp changes in this phase; `ferro_versions` is not surfaced through introspection per D-09.

## Sources

### Primary (HIGH confidence)
- `.planning/phases/129-publish-workflow-refinement/129-CONTEXT.md` — locked decisions
- `.planning/phases/126-deploy-experience-feedback/REPORT.md` §8, §14 — source items
- `.github/workflows/publish.yml` — current workflow structure (read in full)
- `ferro-cli/src/deploy/rewrite_ferro_version.rs` — verified `toml_edit` already in use
- `ferro-cli/src/project.rs` lines 19–115 — verified real parser location and hand-rolled shape
- Root `Cargo.toml` — verified workspace member list
- `PUBLISHING.md` — verified current section structure
- `CLAUDE.md` (project) — verified testing + doc voice constraints

### Secondary (MEDIUM confidence)
- YAML 1.1 boolean coercion (`no`/`off`/`n` → false) — general knowledge, cross-checked against recommendation to use `none`.

### Tertiary (LOW confidence)
- None. Phase is entirely internal; no external library research needed.

## Metadata

**Confidence breakdown:**
- Workflow structure: HIGH — read the full file
- Parser location: HIGH — grepped and verified (CONTEXT.md was incorrect; corrected above)
- `toml_edit` round-trip behavior: HIGH — already tested in the file for dep-table cases; untouched-table preservation is a documented toml_edit guarantee
- Excluded paths list: MEDIUM — listed from `ls` of repo root; extension list should be reviewed by user during plan gate

**Research date:** 2026-04-09
**Valid until:** 2026-05-09 (stable — workflow and parser code are unlikely to move within a month)
